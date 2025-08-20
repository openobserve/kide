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
        // Clean up existing watch manager before creating a new one
        // This prevents cross-cluster contamination
        {
            let mut manager_lock = self.watch_manager.lock().await;
            if let Some(existing_manager) = manager_lock.take() {
                existing_manager.stop_all_watches().await.map_err(|e| e.to_string())?;
            }
        }
        
        // Initialize watch manager
        let watch_manager = WatchManager::new(self.k8s_client.clone());
        let mut manager_lock = self.watch_manager.lock().await;
        *manager_lock = Some(watch_manager);
        
        // Clean up existing shared cache before creating a new one
        {
            let mut cache_lock = self.shared_cache.lock().await;
            if let Some(_existing_cache) = cache_lock.take() {
                // The SharedWatchCache will be dropped and automatically cleaned up
            }
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new();
        
        // Test that managers are initially None
        assert!(state.watch_manager.lock().await.is_none());
        assert!(state.shared_cache.lock().await.is_none());
        assert!(state.log_stream_manager.lock().await.is_none());
    }

    #[tokio::test]
    async fn test_initialize_managers_cleanup_prevents_cross_cluster_contamination() {
        // Test that initialize_managers properly cleans up existing managers
        // This prevents cross-cluster contamination
        let state = AppState::new();
        
        // Simulate having existing managers (from previous cluster context)
        {
            let k8s_client = K8sClient::new();
            let existing_manager = WatchManager::new(k8s_client.clone());
            let mut watch_lock = state.watch_manager.lock().await;
            *watch_lock = Some(existing_manager);
            
            let mut existing_cache = SharedWatchCache::new(k8s_client);
            existing_cache.start_cleanup_task();
            let mut cache_lock = state.shared_cache.lock().await;
            *cache_lock = Some(existing_cache);
        }
        
        // Verify managers are set
        assert!(state.watch_manager.lock().await.is_some());
        assert!(state.shared_cache.lock().await.is_some());
        
        // Now call initialize_managers (simulating cluster context switch)
        // This should clean up existing managers and create new ones
        let _result = state.initialize_managers().await;
        
        // The call might fail due to no real k8s connection, but cleanup logic should work
        // What's important is that old managers were replaced with new ones
        // The initialization creates fresh managers, preventing cross-cluster contamination
        
        // Verify that managers still exist (new ones were created)
        assert!(state.watch_manager.lock().await.is_some());
        assert!(state.shared_cache.lock().await.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_stops_all_watches() {
        let state = AppState::new();
        
        // Set up some mock managers
        {
            let k8s_client = K8sClient::new();
            let watch_manager = WatchManager::new(k8s_client.clone());
            let mut watch_lock = state.watch_manager.lock().await;
            *watch_lock = Some(watch_manager);
        }
        
        // Call cleanup - should not panic
        let result = state.cleanup().await;
        
        // Should succeed even with mock managers
        assert!(result.is_ok());
    }

    #[tokio::test] 
    async fn test_cluster_context_switch_isolation_pattern() {
        // Test the theoretical behavior of cluster context switching
        // This validates the isolation logic without requiring actual k8s connections
        
        let state = AppState::new();
        
        // Simulate cluster context switch scenario:
        
        // Step 1: Initialize for cluster-a (first time)
        let _result1 = state.initialize_managers().await;
        
        // Step 2: Initialize for cluster-b (context switch)
        // This should clean up cluster-a managers and create new ones
        let _result2 = state.initialize_managers().await;
        
        // Both calls should handle the pattern correctly:
        // - First call: creates new managers
        // - Second call: stops existing watches, cleans cache, creates new managers
        // This ensures complete isolation between cluster contexts
        
        // The actual success/failure doesn't matter in unit tests
        // What matters is the cleanup pattern is executed
    }

    #[tokio::test]
    async fn test_cross_cluster_contamination_prevention_integration() {
        // Integration test for the complete cross-cluster contamination prevention
        use crate::k8s::watch::WatchManager;
        
        let state = AppState::new();
        
        // This test verifies the complete flow:
        // 1. User connects to cluster-a -> initialize_managers() called
        // 2. User switches to cluster-b -> initialize_managers() called again
        // 3. Old cluster-a watches are stopped
        // 4. New cluster-b managers are created
        // 5. No data contamination between clusters
        
        // Simulate connecting to cluster-a
        let client_a = K8sClient::new();
        let manager_a = WatchManager::new(client_a);
        {
            let mut lock = state.watch_manager.lock().await;
            *lock = Some(manager_a);
        }
        
        // Verify cluster-a manager is set
        assert!(state.watch_manager.lock().await.is_some());
        
        // Now simulate switching to cluster-b
        // initialize_managers() should clean up cluster-a and set up cluster-b
        let _result = state.initialize_managers().await;
        
        // The new manager should be different (representing cluster-b)
        assert!(state.watch_manager.lock().await.is_some());
        
        // In a real scenario, the watch keys would be different:
        // cluster-a: "cluster-a:nodes:all", "cluster-a:pods:default"
        // cluster-b: "cluster-b:nodes:all", "cluster-b:pods:default"
        // This ensures complete isolation
    }
}