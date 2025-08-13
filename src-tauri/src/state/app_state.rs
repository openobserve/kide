use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::k8s::{K8sClient, LogStreamManager, WatchManager, SharedWatchCache};
use crate::security::{ShellValidator, InputSanitizer};
use crate::cleanup::{TaskManager, CleanupGuard};
use crate::errors::AppResult;
use super::KideConfig;

/// Shell session handle for managing pod shell connections
pub struct ShellSession {
    pub handle: tokio::task::JoinHandle<()>,
    pub tx: tokio::sync::mpsc::UnboundedSender<String>,
}

/// Main application state containing all managers and configuration
pub struct AppState {
    pub k8s_client: K8sClient,
    pub watch_manager: Arc<Mutex<Option<WatchManager>>>,
    pub shared_cache: Arc<Mutex<Option<SharedWatchCache>>>,
    pub log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    pub shell_sessions: Arc<Mutex<HashMap<String, ShellSession>>>,
    pub shell_validator: Arc<ShellValidator>,
    pub input_sanitizer: Arc<InputSanitizer>,
    pub config: KideConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Create new application state with default configuration
    pub fn new() -> Self {
        Self::with_config(KideConfig::default())
    }
    
    /// Create a builder for constructing AppState with dependency injection
    pub fn builder() -> crate::state::AppStateBuilder {
        crate::state::AppStateBuilder::new()
    }
    
    /// Create new application state with custom configuration
    pub fn with_config(config: KideConfig) -> Self {
        Self {
            k8s_client: K8sClient::new(),
            watch_manager: Arc::new(Mutex::new(None)),
            shared_cache: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_sessions: Arc::new(Mutex::new(HashMap::new())),
            shell_validator: Arc::new(ShellValidator::new()),
            input_sanitizer: Arc::new(InputSanitizer::new()),
            config,
        }
    }
    
    /// Initialize Kubernetes managers after successful connection
    pub async fn initialize_managers(&self) -> Result<(), String> {
        // Initialize watch manager
        let watch_manager = WatchManager::new(self.k8s_client.clone());
        let mut manager_lock = self.watch_manager.lock().await;
        *manager_lock = Some(watch_manager);
        
        // Initialize shared cache
        let mut shared_cache = SharedWatchCache::new(self.k8s_client.clone());
        shared_cache.start_cleanup_task();
        let mut cache_lock = self.shared_cache.lock().await;
        *cache_lock = Some(shared_cache);
        
        // Initialize log stream manager  
        let log_stream_manager = LogStreamManager::new(self.k8s_client.clone());
        let mut log_manager_lock = self.log_stream_manager.lock().await;
        *log_manager_lock = Some(log_stream_manager);
        
        Ok(())
    }
    
    /// Clean up all managers and sessions
    pub async fn cleanup(&self) -> Result<(), String> {
        // Stop all watches
        if let Some(watch_manager) = self.watch_manager.lock().await.as_ref() {
            watch_manager.stop_all_watches().await.map_err(|e| e.to_string())?;
        }
        
        // Stop shared cache
        if let Some(shared_cache) = self.shared_cache.lock().await.as_ref() {
            shared_cache.shutdown().await;
        }
        
        // Stop all log streams
        if let Some(log_manager) = self.log_stream_manager.lock().await.as_ref() {
            log_manager.stop_all_streams().await.map_err(|e| e.to_string())?;
        }
        
        // Stop all shell sessions
        let mut sessions = self.shell_sessions.lock().await;
        for (_, session) in sessions.drain() {
            session.handle.abort();
        }
        
        Ok(())
    }
}

/// Enhanced AppState with RAII cleanup patterns
pub struct ManagedAppState {
    pub k8s_client: K8sClient,
    pub watch_manager: Arc<Mutex<Option<WatchManager>>>,
    pub shared_cache: Arc<Mutex<Option<SharedWatchCache>>>,
    pub log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    pub shell_validator: Arc<ShellValidator>,
    pub input_sanitizer: Arc<InputSanitizer>,
    pub config: KideConfig,
    
    // RAII managed resources
    pub task_manager: CleanupGuard<TaskManager>,
}

impl ManagedAppState {
    /// Create new managed application state with RAII cleanup
    pub fn new() -> Self {
        Self::with_config(KideConfig::default())
    }
    
    /// Create new managed application state with custom configuration
    pub fn with_config(config: KideConfig) -> Self {
        let task_manager = CleanupGuard::new(TaskManager::new("app_tasks"));
        
        Self {
            k8s_client: K8sClient::new(),
            watch_manager: Arc::new(Mutex::new(None)),
            shared_cache: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_validator: Arc::new(ShellValidator::new()),
            input_sanitizer: Arc::new(InputSanitizer::new()),
            config,
            task_manager,
        }
    }
    
    /// Initialize managers with proper cleanup registration
    pub async fn initialize_managers(&mut self) -> Result<(), String> {
        // Initialize watch manager
        let watch_manager = WatchManager::new(self.k8s_client.clone());
        let mut manager_lock = self.watch_manager.lock().await;
        *manager_lock = Some(watch_manager);
        
        // Initialize shared cache with cleanup task
        let mut shared_cache = SharedWatchCache::new(self.k8s_client.clone());
        shared_cache.start_cleanup_task();
        let mut cache_lock = self.shared_cache.lock().await;
        *cache_lock = Some(shared_cache);
        
        // Initialize log stream manager
        let log_stream_manager = LogStreamManager::new(self.k8s_client.clone());
        let mut log_manager_lock = self.log_stream_manager.lock().await;
        *log_manager_lock = Some(log_stream_manager);
        
        // Start background cleanup task for finished operations
        if let Some(task_manager) = self.task_manager.get() {
            let cleanup_interval = self.config.cleanup_interval;
            
            task_manager.spawn_task("periodic_cleanup", "Periodic cleanup task", async move {
                let mut interval = tokio::time::interval(cleanup_interval);
                
                loop {
                    interval.tick().await;
                    // Periodic cleanup logic would go here
                    // For now, just tick periodically
                }
            }).await.map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }
    
    /// Clean up with RAII patterns
    pub async fn cleanup(&mut self) -> AppResult<()> {
        // Stop all watches
        if let Some(watch_manager) = self.watch_manager.lock().await.as_ref() {
            watch_manager.stop_all_watches().await.map_err(|e| {
                crate::errors::AppError::State(crate::errors::StateError::CleanupFailed {
                    component: "watch_manager".to_string(),
                    reason: e.to_string(),
                })
            })?;
        }
        
        // Stop shared cache
        if let Some(shared_cache) = self.shared_cache.lock().await.as_ref() {
            shared_cache.shutdown().await;
        }
        
        // Stop all log streams
        if let Some(log_manager) = self.log_stream_manager.lock().await.as_ref() {
            log_manager.stop_all_streams().await.map_err(|e| {
                crate::errors::AppError::State(crate::errors::StateError::CleanupFailed {
                    component: "log_manager".to_string(),
                    reason: e.to_string(),
                })
            })?;
        }
        
        // Task manager will be cleaned up automatically via RAII when dropped
        // We can't call cleanup() here as it consumes the guard, but cleanup happens on drop
        
        Ok(())
    }
}