use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::k8s::{K8sClient, LogStreamManager, WatchManager};
use crate::security::{ShellValidator, InputSanitizer};
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
    
    /// Create new application state with custom configuration
    pub fn with_config(config: KideConfig) -> Self {
        Self {
            k8s_client: K8sClient::new(),
            watch_manager: Arc::new(Mutex::new(None)),
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