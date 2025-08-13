//! Shell session management module
//!
//! This module provides a dedicated system for managing pod shell connections
//! with proper lifecycle management, RAII cleanup, and session tracking.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use async_trait::async_trait;
use uuid::Uuid;
use tauri::AppHandle;

use crate::errors::{AppResult, AppError, ShellError};
use crate::cleanup::{Cleanup, TaskManager, CleanupGuard};
use crate::security::{ShellValidator, InputSanitizer};
use crate::k8s::K8sClient;

/// Shell session configuration
#[derive(Debug, Clone)]
pub struct ShellSessionConfig {
    pub pod_name: String,
    pub namespace: String,
    pub container_name: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub shell_command: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for ShellSessionConfig {
    fn default() -> Self {
        Self {
            pod_name: String::new(),
            namespace: String::new(),
            container_name: None,
            cols: 80,
            rows: 24,
            shell_command: None,
            timeout_seconds: 300, // 5 minutes default
        }
    }
}

/// Active shell session with cleanup capabilities
pub struct ShellSession {
    pub id: String,
    pub config: ShellSessionConfig,
    pub handle: JoinHandle<()>,
    pub input_tx: mpsc::UnboundedSender<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    pub is_active: AtomicBool,
}

impl ShellSession {
    /// Create a new shell session
    pub fn new(
        config: ShellSessionConfig,
        handle: JoinHandle<()>,
        input_tx: mpsc::UnboundedSender<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            config,
            handle,
            input_tx,
            started_at: now,
            last_activity: Arc::new(Mutex::new(now)),
            is_active: AtomicBool::new(true),
        }
    }
    
    /// Send input to the shell session
    pub async fn send_input(&self, input: String) -> AppResult<()> {
        if !self.is_active.load(Ordering::SeqCst) {
            return Err(AppError::Shell(ShellError::SessionNotActive {
                session_id: self.id.clone(),
            }));
        }
        
        self.input_tx.send(input).map_err(|_| {
            AppError::Shell(ShellError::InputFailed {
                session_id: self.id.clone(),
                message: "Channel closed".to_string(),
            })
        })?;
        
        // Update last activity
        let mut last_activity = self.last_activity.lock().await;
        *last_activity = chrono::Utc::now();
        
        Ok(())
    }
    
    /// Check if the session is still active
    pub fn is_session_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst) && !self.handle.is_finished()
    }
    
    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now().signed_duration_since(self.started_at)
    }
    
    /// Check if session has been idle for too long
    pub async fn is_idle(&self, idle_timeout: chrono::Duration) -> bool {
        let last_activity = *self.last_activity.lock().await;
        chrono::Utc::now().signed_duration_since(last_activity) > idle_timeout
    }
    
    /// Terminate the session
    pub fn terminate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
        self.handle.abort();
    }
}

#[async_trait]
impl Cleanup for ShellSession {
    async fn cleanup(&self) -> AppResult<()> {
        self.terminate();
        Ok(())
    }
    
    fn is_cleaned_up(&self) -> bool {
        !self.is_active.load(Ordering::SeqCst)
    }
    
    fn component_name(&self) -> &str {
        &self.id
    }
}

/// Manager for shell sessions with RAII cleanup
pub struct ShellSessionManager {
    sessions: Arc<Mutex<HashMap<String, CleanupGuard<ShellSession>>>>,
    k8s_client: K8sClient,
    shell_validator: Arc<ShellValidator>,
    input_sanitizer: Arc<InputSanitizer>,
    task_manager: TaskManager,
    max_sessions: usize,
    idle_timeout: chrono::Duration,
    cleanup_interval: std::time::Duration,
}

impl ShellSessionManager {
    /// Create a new shell session manager
    pub fn new(
        k8s_client: K8sClient,
        shell_validator: Arc<ShellValidator>,
        input_sanitizer: Arc<InputSanitizer>,
    ) -> Self {
        let task_manager = TaskManager::new("shell_sessions");
        
        let manager = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            k8s_client,
            shell_validator,
            input_sanitizer,
            task_manager,
            max_sessions: 10,
            idle_timeout: chrono::Duration::minutes(30),
            cleanup_interval: std::time::Duration::from_secs(60),
        };
        
        // Start cleanup task
        manager.start_cleanup_task();
        
        manager
    }
    
    /// Configure maximum number of concurrent sessions
    pub fn with_max_sessions(mut self, max_sessions: usize) -> Self {
        self.max_sessions = max_sessions;
        self
    }
    
    /// Configure idle timeout
    pub fn with_idle_timeout(mut self, timeout: chrono::Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }
    
    /// Start a new shell session
    pub async fn start_session(
        &self,
        app_handle: AppHandle,
        config: ShellSessionConfig,
    ) -> AppResult<String> {
        // Check session limits
        {
            let sessions = self.sessions.lock().await;
            if sessions.len() >= self.max_sessions {
                return Err(AppError::Shell(ShellError::TooManySessions {
                    current: sessions.len(),
                    max: self.max_sessions,
                }));
            }
        }
        
        // Validate configuration
        self.validate_config(&config).await?;
        
        // Get Kubernetes client
        let client = self.k8s_client.get_client().await
            .map_err(|e| AppError::Shell(ShellError::SessionStartFailed {
                pod_name: config.pod_name.clone(),
                namespace: config.namespace.clone(),
                message: format!("K8s client error: {}", e),
            }))?;
        
        // Create communication channel
        let (input_tx, input_rx) = mpsc::unbounded_channel::<String>();
        
        // Clone necessary data for the task
        let pod_name = config.pod_name.clone();
        let namespace = config.namespace.clone();
        let container_name = config.container_name.clone();
        let shell_validator = self.shell_validator.clone();
        let input_sanitizer = self.input_sanitizer.clone();
        
        // Spawn shell interaction task
        let handle = tokio::spawn(async move {
            if let Err(e) = run_shell_session(
                client,
                app_handle,
                pod_name,
                namespace,
                container_name,
                input_rx,
                shell_validator,
                input_sanitizer,
            ).await {
                eprintln!("Shell session error: {}", e);
            }
        });
        
        // Create session
        let session = ShellSession::new(config, handle, input_tx);
        let session_id = session.id.clone();
        
        // Store session with cleanup guard
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), CleanupGuard::new(session));
        
        Ok(session_id)
    }
    
    /// Send input to a specific session
    pub async fn send_input(&self, session_id: &str, input: String) -> AppResult<()> {
        let sessions = self.sessions.lock().await;
        let session_guard = sessions.get(session_id)
            .ok_or_else(|| AppError::Shell(ShellError::SessionNotFound {
                session_id: session_id.to_string(),
            }))?;
            
        if let Some(session) = session_guard.get() {
            session.send_input(input).await
        } else {
            Err(AppError::Shell(ShellError::SessionNotActive {
                session_id: session_id.to_string(),
            }))
        }
    }
    
    /// Terminate a specific session
    pub async fn terminate_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session_guard) = sessions.remove(session_id) {
            session_guard.cleanup().await?;
            Ok(())
        } else {
            Err(AppError::Shell(ShellError::SessionNotFound {
                session_id: session_id.to_string(),
            }))
        }
    }
    
    /// Get list of active sessions
    pub async fn list_sessions(&self) -> Vec<ShellSessionInfo> {
        let sessions = self.sessions.lock().await;
        let mut info = Vec::new();
        
        for (id, session_guard) in sessions.iter() {
            if let Some(session) = session_guard.get() {
                if session.is_session_active() {
                    info.push(ShellSessionInfo {
                        id: id.clone(),
                        pod_name: session.config.pod_name.clone(),
                        namespace: session.config.namespace.clone(),
                        container_name: session.config.container_name.clone(),
                        started_at: session.started_at,
                        duration: session.duration(),
                        is_active: session.is_session_active(),
                    });
                }
            }
        }
        
        info
    }
    
    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions.len()
    }
    
    /// Validate session configuration
    async fn validate_config(&self, config: &ShellSessionConfig) -> AppResult<()> {
        if config.pod_name.is_empty() {
            return Err(AppError::Shell(ShellError::ValidationFailed {
                field: "pod_name".to_string(),
                message: "Pod name cannot be empty".to_string(),
            }));
        }
        
        if config.namespace.is_empty() {
            return Err(AppError::Shell(ShellError::ValidationFailed {
                field: "namespace".to_string(),
                message: "Namespace cannot be empty".to_string(),
            }));
        }
        
        // Validate shell command if provided
        if let Some(ref command) = config.shell_command {
            // Placeholder validation - in real implementation this would call the validator
            if command.is_empty() {
                return Err(AppError::Shell(ShellError::ValidationFailed {
                    field: "shell_command".to_string(),
                    message: "Shell command cannot be empty".to_string(),
                }));
            }
        }
        
        Ok(())
    }
    
    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let idle_timeout = self.idle_timeout;
        let cleanup_interval = self.cleanup_interval;
        
        let _handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let mut sessions_lock = sessions.lock().await;
                let mut to_remove = Vec::new();
                
                for (session_id, session_guard) in sessions_lock.iter() {
                    if let Some(session) = session_guard.get() {
                        if !session.is_session_active() || session.is_idle(idle_timeout).await {
                            to_remove.push(session_id.clone());
                        }
                    } else {
                        to_remove.push(session_id.clone());
                    }
                }
                
                for session_id in to_remove {
                    if let Some(session_guard) = sessions_lock.remove(&session_id) {
                        // Cleanup happens automatically via RAII
                        drop(session_guard);
                        println!("🧹 Cleaned up idle shell session: {}", session_id);
                    }
                }
            }
        });
    }
}

#[async_trait]
impl Cleanup for ShellSessionManager {
    async fn cleanup(&self) -> AppResult<()> {
        let mut sessions = self.sessions.lock().await;
        
        println!("🧹 Cleaning up {} shell sessions", sessions.len());
        
        // Cleanup all sessions
        for (id, session_guard) in sessions.drain() {
            if let Err(e) = session_guard.cleanup().await {
                eprintln!("⚠️  Failed to cleanup shell session {}: {}", id, e);
            }
        }
        
        // Cleanup task manager
        self.task_manager.cleanup().await?;
        
        Ok(())
    }
    
    fn is_cleaned_up(&self) -> bool {
        self.task_manager.is_cleaned_up()
    }
    
    fn component_name(&self) -> &str {
        "shell_session_manager"
    }
}

/// Information about a shell session
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShellSessionInfo {
    pub id: String,
    pub pod_name: String,
    pub namespace: String,
    pub container_name: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub duration: chrono::Duration,
    pub is_active: bool,
}

/// Run the actual shell session (placeholder implementation)
async fn run_shell_session(
    _client: kube::Client,
    _app_handle: AppHandle,
    _pod_name: String,
    _namespace: String,
    _container_name: Option<String>,
    mut _input_rx: mpsc::UnboundedReceiver<String>,
    _shell_validator: Arc<ShellValidator>,
    _input_sanitizer: Arc<InputSanitizer>,
) -> AppResult<()> {
    // TODO: Implement actual shell session logic
    // This would involve:
    // 1. Setting up AttachParams
    // 2. Attaching to the pod
    // 3. Handling I/O streams
    // 4. Processing input/output
    
    // For now, just simulate a shell session
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{ShellValidator, InputSanitizer};
    
    #[tokio::test]
    async fn test_shell_session_config_validation() {
        let k8s_client = K8sClient::new();
        let shell_validator = Arc::new(ShellValidator::new());
        let input_sanitizer = Arc::new(InputSanitizer::new());
        
        let manager = ShellSessionManager::new(k8s_client, shell_validator, input_sanitizer);
        
        // Valid config
        let valid_config = ShellSessionConfig {
            pod_name: "test-pod".to_string(),
            namespace: "default".to_string(),
            ..Default::default()
        };
        assert!(manager.validate_config(&valid_config).await.is_ok());
        
        // Invalid config - empty pod name
        let invalid_config = ShellSessionConfig {
            pod_name: "".to_string(),
            namespace: "default".to_string(),
            ..Default::default()
        };
        assert!(manager.validate_config(&invalid_config).await.is_err());
    }
    
    #[tokio::test]
    async fn test_shell_session_manager_limits() {
        let k8s_client = K8sClient::new();
        let shell_validator = Arc::new(ShellValidator::new());
        let input_sanitizer = Arc::new(InputSanitizer::new());
        
        let manager = ShellSessionManager::new(k8s_client, shell_validator, input_sanitizer)
            .with_max_sessions(2);
        
        assert_eq!(manager.session_count().await, 0);
        assert_eq!(manager.max_sessions, 2);
    }
}