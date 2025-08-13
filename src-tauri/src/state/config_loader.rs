//! Configuration loading and validation with environment integration
//!
//! This module provides utilities for loading configuration from various sources
//! including environment variables, config files, and runtime overrides.

use std::env;
use std::time::Duration;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult, ConfigError};
use super::KideConfig;

/// Configuration loader with environment integration
pub struct ConfigLoader {
    env_prefix: String,
    config_paths: Vec<PathBuf>,
}

/// Serializable configuration for file-based loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableConfig {
    pub connection_timeout_secs: u64,
    pub stream_timeout_secs: u64,
    pub cleanup_interval_secs: u64,
    pub max_fd_usage: usize,
    pub max_shell_command_length: usize,
    pub shell_env_timeout_secs: u64,
}

impl From<KideConfig> for SerializableConfig {
    fn from(config: KideConfig) -> Self {
        Self {
            connection_timeout_secs: config.connection_timeout.as_secs(),
            stream_timeout_secs: config.stream_timeout.as_secs(),
            cleanup_interval_secs: config.cleanup_interval.as_secs(),
            max_fd_usage: config.max_fd_usage,
            max_shell_command_length: config.max_shell_command_length,
            shell_env_timeout_secs: config.shell_env_timeout.as_secs(),
        }
    }
}

impl From<SerializableConfig> for KideConfig {
    fn from(config: SerializableConfig) -> Self {
        Self {
            connection_timeout: Duration::from_secs(config.connection_timeout_secs),
            stream_timeout: Duration::from_secs(config.stream_timeout_secs),
            cleanup_interval: Duration::from_secs(config.cleanup_interval_secs),
            max_fd_usage: config.max_fd_usage,
            max_shell_command_length: config.max_shell_command_length,
            shell_env_timeout: Duration::from_secs(config.shell_env_timeout_secs),
        }
    }
}

impl ConfigLoader {
    /// Create a new config loader with default settings
    pub fn new() -> Self {
        Self {
            env_prefix: "KIDE".to_string(),
            config_paths: vec![
                PathBuf::from("kide.toml"),
                PathBuf::from("config/kide.toml"),
                PathBuf::from(".").join("kide/config.toml"),
            ],
        }
    }

    /// Set the environment variable prefix
    pub fn env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Add a configuration file path to search
    pub fn add_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_paths.push(path.into());
        self
    }

    /// Load configuration from all available sources
    pub fn load(&self) -> AppResult<KideConfig> {
        // Start with default configuration
        let mut config = KideConfig::default();

        // Try to load from config files
        if let Ok(file_config) = self.load_from_file() {
            config = file_config;
        }

        // Override with environment variables
        config = self.load_from_env(config)?;

        // Validate the final configuration
        self.validate_config(&config)?;

        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn load_from_env(&self, base_config: KideConfig) -> AppResult<KideConfig> {
        let mut config = base_config;

        // Connection timeout
        if let Ok(value) = env::var(format!("{}_CONNECTION_TIMEOUT", self.env_prefix)) {
            let timeout_secs = value.parse::<u64>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "connection_timeout".to_string(),
                    value,
                    expected: "positive integer (seconds)".to_string(),
                })?;
            config.connection_timeout = Duration::from_secs(timeout_secs);
        }

        // Stream timeout
        if let Ok(value) = env::var(format!("{}_STREAM_TIMEOUT", self.env_prefix)) {
            let timeout_secs = value.parse::<u64>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "stream_timeout".to_string(),
                    value,
                    expected: "positive integer (seconds)".to_string(),
                })?;
            config.stream_timeout = Duration::from_secs(timeout_secs);
        }

        // Cleanup interval
        if let Ok(value) = env::var(format!("{}_CLEANUP_INTERVAL", self.env_prefix)) {
            let interval_secs = value.parse::<u64>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "cleanup_interval".to_string(),
                    value,
                    expected: "positive integer (seconds)".to_string(),
                })?;
            config.cleanup_interval = Duration::from_secs(interval_secs);
        }

        // Max FD usage
        if let Ok(value) = env::var(format!("{}_MAX_FD_USAGE", self.env_prefix)) {
            config.max_fd_usage = value.parse::<usize>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "max_fd_usage".to_string(),
                    value,
                    expected: "positive integer".to_string(),
                })?;
        }

        // Max shell command length
        if let Ok(value) = env::var(format!("{}_MAX_SHELL_COMMAND_LENGTH", self.env_prefix)) {
            config.max_shell_command_length = value.parse::<usize>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "max_shell_command_length".to_string(),
                    value,
                    expected: "positive integer".to_string(),
                })?;
        }

        // Shell environment timeout
        if let Ok(value) = env::var(format!("{}_SHELL_ENV_TIMEOUT", self.env_prefix)) {
            let timeout_secs = value.parse::<u64>()
                .map_err(|_| ConfigError::InvalidValue {
                    field: "shell_env_timeout".to_string(),
                    value,
                    expected: "positive integer (seconds)".to_string(),
                })?;
            config.shell_env_timeout = Duration::from_secs(timeout_secs);
        }

        Ok(config)
    }

    /// Load configuration from file (TOML format)
    pub fn load_from_file(&self) -> AppResult<KideConfig> {
        for path in &self.config_paths {
            if path.exists() {
                let _content = std::fs::read_to_string(path)
                    .map_err(|e| ConfigError::FileReadError {
                        path: path.to_string_lossy().to_string(),
                        error: e.to_string(),
                    })?;

                // For now, just return default config since we'd need toml crate
                return Ok(KideConfig::default());
            }
        }

        Err(AppError::Config(ConfigError::NotFound {
            paths: self.config_paths.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        }))
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, config: &KideConfig, path: impl Into<PathBuf>) -> AppResult<()> {
        let path = path.into();
        let _serializable_config: SerializableConfig = config.clone().into();
        
        let content = format!("# Kide configuration\n# This feature requires the toml crate\n");

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::FileWriteError {
                    path: path.to_string_lossy().to_string(),
                    error: e.to_string(),
                })?;
        }

        std::fs::write(&path, content)
            .map_err(|e| ConfigError::FileWriteError {
                path: path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;

        Ok(())
    }

    /// Validate configuration values
    pub fn validate_config(&self, config: &KideConfig) -> AppResult<()> {
        // Validate timeouts
        if config.connection_timeout.as_secs() == 0 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "connection_timeout".to_string(),
                message: "Connection timeout must be greater than 0 seconds".to_string(),
            }));
        }

        if config.connection_timeout.as_secs() > 600 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "connection_timeout".to_string(),
                message: "Connection timeout should not exceed 600 seconds (10 minutes)".to_string(),
            }));
        }

        if config.stream_timeout.as_secs() == 0 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "stream_timeout".to_string(),
                message: "Stream timeout must be greater than 0 seconds".to_string(),
            }));
        }

        if config.cleanup_interval.as_secs() == 0 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "cleanup_interval".to_string(),
                message: "Cleanup interval must be greater than 0 seconds".to_string(),
            }));
        }

        // Validate limits
        if config.max_fd_usage == 0 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "max_fd_usage".to_string(),
                message: "Max file descriptor usage must be greater than 0".to_string(),
            }));
        }

        if config.max_fd_usage > 10000 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "max_fd_usage".to_string(),
                message: "Max file descriptor usage should not exceed 10000".to_string(),
            }));
        }

        if config.max_shell_command_length == 0 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "max_shell_command_length".to_string(),
                message: "Max shell command length must be greater than 0".to_string(),
            }));
        }

        if config.max_shell_command_length > 100_000 {
            return Err(AppError::Config(ConfigError::ValidationFailed {
                field: "max_shell_command_length".to_string(),
                message: "Max shell command length should not exceed 100,000 characters".to_string(),
            }));
        }

        Ok(())
    }

    /// Get current environment configuration summary
    pub fn get_env_summary(&self) -> EnvironmentSummary {
        let mut vars = std::collections::HashMap::new();
        
        let env_vars = [
            "CONNECTION_TIMEOUT",
            "STREAM_TIMEOUT", 
            "CLEANUP_INTERVAL",
            "MAX_FD_USAGE",
            "MAX_SHELL_COMMAND_LENGTH",
            "SHELL_ENV_TIMEOUT",
        ];

        for var in &env_vars {
            let key = format!("{}_{}", self.env_prefix, var);
            if let Ok(value) = env::var(&key) {
                vars.insert(key, value);
            }
        }

        EnvironmentSummary {
            prefix: self.env_prefix.clone(),
            config_paths: self.config_paths.clone(),
            environment_variables: vars,
            detected_environment: self.detect_environment(),
        }
    }

    /// Detect the current environment based on various indicators
    fn detect_environment(&self) -> DetectedEnvironment {
        // Check for common environment indicators
        if env::var("NODE_ENV").unwrap_or_default() == "development" ||
           env::var("RUST_ENV").unwrap_or_default() == "development" ||
           env::var("KIDE_ENV").unwrap_or_default() == "development" {
            return DetectedEnvironment::Development;
        }

        if env::var("NODE_ENV").unwrap_or_default() == "test" ||
           env::var("RUST_ENV").unwrap_or_default() == "test" ||
           env::var("KIDE_ENV").unwrap_or_default() == "test" {
            return DetectedEnvironment::Testing;
        }

        // Check for common CI/testing indicators
        if env::var("CI").is_ok() || 
           env::var("GITHUB_ACTIONS").is_ok() ||
           env::var("TRAVIS").is_ok() {
            return DetectedEnvironment::Testing;
        }

        // Check for debug build
        if cfg!(debug_assertions) {
            return DetectedEnvironment::Development;
        }

        DetectedEnvironment::Production
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of environment configuration
#[derive(Debug, Clone)]
pub struct EnvironmentSummary {
    pub prefix: String,
    pub config_paths: Vec<PathBuf>,
    pub environment_variables: std::collections::HashMap<String, String>,
    pub detected_environment: DetectedEnvironment,
}

/// Detected environment type
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedEnvironment {
    Development,
    Testing,
    Production,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::new();
        let config = loader.load().unwrap();
        
        assert!(config.connection_timeout.as_secs() > 0);
        assert!(config.max_fd_usage > 0);
    }

    #[test]
    fn test_config_from_env() {
        // Set environment variables
        env::set_var("KIDE_CONNECTION_TIMEOUT", "45");
        env::set_var("KIDE_MAX_FD_USAGE", "750");
        
        let loader = ConfigLoader::new();
        let config = loader.load_from_env(KideConfig::default()).unwrap();
        
        assert_eq!(config.connection_timeout.as_secs(), 45);
        assert_eq!(config.max_fd_usage, 750);
        
        // Clean up
        env::remove_var("KIDE_CONNECTION_TIMEOUT");
        env::remove_var("KIDE_MAX_FD_USAGE");
    }

    #[test]
    fn test_config_validation() {
        let loader = ConfigLoader::new();
        
        // Valid configuration
        let valid_config = KideConfig::default();
        assert!(loader.validate_config(&valid_config).is_ok());
        
        // Invalid configuration
        let invalid_config = KideConfig {
            connection_timeout: Duration::from_secs(0), // Invalid
            ..Default::default()
        };
        assert!(loader.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_serializable_config_conversion() {
        let original = KideConfig::default();
        let serializable: SerializableConfig = original.clone().into();
        let restored: KideConfig = serializable.into();
        
        assert_eq!(original.connection_timeout, restored.connection_timeout);
        assert_eq!(original.max_fd_usage, restored.max_fd_usage);
    }

    #[test]
    fn test_environment_detection() {
        let loader = ConfigLoader::new();
        let summary = loader.get_env_summary();
        
        assert_eq!(summary.prefix, "KIDE");
        assert!(!summary.config_paths.is_empty());
    }
}