//! Builder pattern for constructing AppState with dependency injection
//!
//! This module provides a flexible builder pattern for creating AppState instances
//! with configurable dependencies, enabling better testability and modularity.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::k8s::{K8sClient, LogStreamManager, WatchLifecycleManager, WatchEventHandler, WatchDispatcher};
use crate::security::{ShellValidator, InputSanitizer};
use crate::errors::{AppError, AppResult};
use super::{AppState, KideConfig, ShellSession};

/// Builder for creating AppState with configurable dependencies
pub struct AppStateBuilder {
    config: Option<KideConfig>,
    k8s_client: Option<K8sClient>,
    shell_validator: Option<Arc<ShellValidator>>,
    input_sanitizer: Option<Arc<InputSanitizer>>,
    environment: Environment,
}

/// Environment configuration for the application
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Production,
    Custom(KideConfig),
}

impl AppStateBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            config: None,
            k8s_client: None,
            shell_validator: None,
            input_sanitizer: None,
            environment: Environment::Production,
        }
    }

    /// Set the environment configuration
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = env;
        self
    }

    /// Set a custom configuration
    pub fn config(mut self, config: KideConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Inject a custom Kubernetes client
    pub fn k8s_client(mut self, client: K8sClient) -> Self {
        self.k8s_client = Some(client);
        self
    }

    /// Inject a custom shell validator
    pub fn shell_validator(mut self, validator: Arc<ShellValidator>) -> Self {
        self.shell_validator = Some(validator);
        self
    }

    /// Inject a custom input sanitizer
    pub fn input_sanitizer(mut self, sanitizer: Arc<InputSanitizer>) -> Self {
        self.input_sanitizer = Some(sanitizer);
        self
    }

    /// Configure for development environment
    pub fn for_development(mut self) -> Self {
        self.environment = Environment::Development;
        self
    }

    /// Configure for testing environment
    pub fn for_testing(mut self) -> Self {
        self.environment = Environment::Testing;
        self
    }

    /// Configure for production environment
    pub fn for_production(mut self) -> Self {
        self.environment = Environment::Production;
        self
    }

    /// Build the AppState with validation
    pub fn build(self) -> AppResult<AppState> {
        let config = self.resolve_config()?;
        
        // Validate configuration
        self.validate_config(&config)?;

        let k8s_client = self.k8s_client.unwrap_or_else(|| K8sClient::new());
        let shell_validator = self.shell_validator.unwrap_or_else(|| Arc::new(ShellValidator::new()));
        let input_sanitizer = self.input_sanitizer.unwrap_or_else(|| Arc::new(InputSanitizer::new()));

        Ok(AppState {
            k8s_client,
            watch_manager: Arc::new(Mutex::new(None)),
            shared_cache: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_sessions: Arc::new(Mutex::new(HashMap::new())),
            shell_validator,
            input_sanitizer,
            config,
        })
    }

    /// Build with automatic manager initialization
    pub async fn build_and_initialize(self) -> AppResult<AppState> {
        let is_testing = matches!(self.environment, Environment::Testing);
        let state = self.build()?;
        
        // Initialize managers if we're not in testing mode
        if !is_testing {
            state.initialize_managers().await
                .map_err(|e| AppError::State(crate::errors::StateError::InitializationFailed {
                    step: "managers".to_string(),
                    message: e,
                }))?;
        }

        Ok(state)
    }

    /// Resolve the final configuration based on builder settings
    fn resolve_config(&self) -> AppResult<KideConfig> {
        if let Some(config) = &self.config {
            return Ok(config.clone());
        }

        let config = match &self.environment {
            Environment::Development => KideConfig::development(),
            Environment::Testing => KideConfig::testing(),
            Environment::Production => KideConfig::production(),
            Environment::Custom(config) => config.clone(),
        };

        Ok(config)
    }

    /// Validate the configuration
    fn validate_config(&self, config: &KideConfig) -> AppResult<()> {
        if config.connection_timeout.as_secs() == 0 {
            return Err(AppError::Config(crate::errors::ConfigError::ValidationFailed {
                field: "connection_timeout".to_string(),
                message: "Connection timeout must be greater than 0".to_string(),
            }));
        }

        if config.max_shell_command_length == 0 {
            return Err(AppError::Config(crate::errors::ConfigError::ValidationFailed {
                field: "max_shell_command_length".to_string(),
                message: "Max shell command length must be greater than 0".to_string(),
            }));
        }

        if config.max_fd_usage == 0 {
            return Err(AppError::Config(crate::errors::ConfigError::ValidationFailed {
                field: "max_fd_usage".to_string(),
                message: "Max file descriptor usage must be greater than 0".to_string(),
            }));
        }

        Ok(())
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced AppState with new watch components
pub struct ModernAppState {
    pub k8s_client: K8sClient,
    pub watch_lifecycle_manager: Arc<WatchLifecycleManager>,
    pub watch_event_handler: Arc<WatchEventHandler>,
    pub watch_dispatcher: Arc<WatchDispatcher>,
    pub log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    pub shell_sessions: Arc<Mutex<HashMap<String, ShellSession>>>,
    pub shell_validator: Arc<ShellValidator>,
    pub input_sanitizer: Arc<InputSanitizer>,
    pub config: KideConfig,
}

impl ModernAppState {
    /// Create a new modern app state using the builder pattern
    pub fn builder() -> ModernAppStateBuilder {
        ModernAppStateBuilder::new()
    }

    /// Initialize log stream manager
    pub async fn initialize_log_manager(&self) -> AppResult<()> {
        let log_stream_manager = LogStreamManager::new(self.k8s_client.clone());
        let mut log_manager_lock = self.log_stream_manager.lock().await;
        *log_manager_lock = Some(log_stream_manager);
        Ok(())
    }

    /// Clean up all resources
    pub async fn cleanup(&self) -> AppResult<()> {
        // Stop all watches
        self.watch_lifecycle_manager.stop_all_watches().await?;
        
        // Stop log streams
        if let Some(log_manager) = self.log_stream_manager.lock().await.as_ref() {
            log_manager.stop_all_streams().await
                .map_err(|e| AppError::State(crate::errors::StateError::CleanupFailed {
                    component: "log_manager".to_string(),
                    reason: e.to_string(),
                }))?;
        }
        
        // Stop shell sessions
        let mut sessions = self.shell_sessions.lock().await;
        for (_, session) in sessions.drain() {
            session.handle.abort();
        }
        
        Ok(())
    }
}

/// Builder for ModernAppState with new watch components
pub struct ModernAppStateBuilder {
    config: Option<KideConfig>,
    k8s_client: Option<K8sClient>,
    shell_validator: Option<Arc<ShellValidator>>,
    input_sanitizer: Option<Arc<InputSanitizer>>,
    environment: Environment,
}

impl ModernAppStateBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            k8s_client: None,
            shell_validator: None,
            input_sanitizer: None,
            environment: Environment::Production,
        }
    }

    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = env;
        self
    }

    pub fn config(mut self, config: KideConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn k8s_client(mut self, client: K8sClient) -> Self {
        self.k8s_client = Some(client);
        self
    }

    pub fn for_testing(mut self) -> Self {
        self.environment = Environment::Testing;
        self
    }

    pub fn build(self) -> AppResult<ModernAppState> {
        let config = self.resolve_config()?;
        
        let k8s_client = self.k8s_client.unwrap_or_else(|| K8sClient::new());
        let shell_validator = self.shell_validator.unwrap_or_else(|| Arc::new(ShellValidator::new()));
        let input_sanitizer = self.input_sanitizer.unwrap_or_else(|| Arc::new(InputSanitizer::new()));

        // Create new watch components
        let watch_lifecycle_manager = Arc::new(WatchLifecycleManager::new());
        let watch_event_handler = Arc::new(WatchEventHandler::new());
        let watch_dispatcher = Arc::new(WatchDispatcher::new(
            watch_lifecycle_manager.clone(),
            watch_event_handler.clone(),
        ));

        Ok(ModernAppState {
            k8s_client,
            watch_lifecycle_manager,
            watch_event_handler,
            watch_dispatcher,
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_sessions: Arc::new(Mutex::new(HashMap::new())),
            shell_validator,
            input_sanitizer,
            config,
        })
    }

    fn resolve_config(&self) -> AppResult<KideConfig> {
        if let Some(config) = &self.config {
            return Ok(config.clone());
        }

        let config = match &self.environment {
            Environment::Development => KideConfig::development(),
            Environment::Testing => KideConfig::testing(),
            Environment::Production => KideConfig::production(),
            Environment::Custom(config) => config.clone(),
        };

        Ok(config)
    }
}

impl Default for ModernAppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_builder_default() {
        let builder = AppStateBuilder::new();
        let state = builder.build().unwrap();
        
        assert!(state.config.connection_timeout.as_secs() > 0);
        assert!(state.config.max_fd_usage > 0);
    }

    #[test]
    fn test_app_state_builder_development() {
        let state = AppStateBuilder::new()
            .for_development()
            .build()
            .unwrap();
        
        // Development should have shorter timeouts
        assert_eq!(state.config.connection_timeout.as_secs(), 10);
        assert_eq!(state.config.stream_timeout.as_secs(), 30);
    }

    #[test]
    fn test_app_state_builder_validation() {
        let invalid_config = KideConfig {
            connection_timeout: std::time::Duration::from_secs(0), // Invalid
            ..Default::default()
        };
        
        let result = AppStateBuilder::new()
            .config(invalid_config)
            .build();
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_modern_app_state_builder() {
        let state = ModernAppState::builder()
            .for_testing()
            .build()
            .unwrap();
        
        assert_eq!(state.watch_lifecycle_manager.active_watch_count().await, 0);
    }
}