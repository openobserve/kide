use std::time::Duration;

/// Centralized configuration for Kide application
#[derive(Debug, Clone, PartialEq)]
pub struct KideConfig {
    /// Timeout for Kubernetes API connections
    pub connection_timeout: Duration,
    /// Timeout for streaming operations (logs, watch)
    pub stream_timeout: Duration,
    /// Interval for cleanup tasks (removing finished handles)
    pub cleanup_interval: Duration,
    /// Maximum file descriptor usage before warning
    pub max_fd_usage: usize,
    /// Maximum length for shell commands
    pub max_shell_command_length: usize,
    /// Timeout for shell environment detection
    pub shell_env_timeout: Duration,
}

impl Default for KideConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            stream_timeout: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(60),
            max_fd_usage: 500,
            max_shell_command_length: 1024,
            shell_env_timeout: Duration::from_secs(10),
        }
    }
}

impl KideConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create configuration optimized for development (shorter timeouts)
    pub fn development() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            stream_timeout: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(30),
            shell_env_timeout: Duration::from_secs(5),
            ..Default::default()
        }
    }
    
    /// Create configuration optimized for production (longer timeouts, higher limits)
    pub fn production() -> Self {
        Self {
            connection_timeout: Duration::from_secs(60),
            stream_timeout: Duration::from_secs(300),
            cleanup_interval: Duration::from_secs(120),
            max_fd_usage: 1000,
            max_shell_command_length: 2048,
            ..Default::default()
        }
    }
    
    /// Create configuration optimized for testing (minimal timeouts, safe defaults)
    pub fn testing() -> Self {
        Self {
            connection_timeout: Duration::from_secs(5),
            stream_timeout: Duration::from_secs(10),
            cleanup_interval: Duration::from_secs(10),
            max_fd_usage: 100,
            max_shell_command_length: 512,
            shell_env_timeout: Duration::from_secs(2),
        }
    }
}