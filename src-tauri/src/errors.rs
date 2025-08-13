//! Comprehensive error types for the Kide application.
//!
//! This module provides structured error handling across all components,
//! replacing generic string errors with specific, actionable error types.

use thiserror::Error;

/// Main application error type encompassing all possible failures.
#[derive(Error, Debug)]
pub enum AppError {
    /// Kubernetes-related errors
    #[error("Kubernetes error: {0}")]
    Kubernetes(#[from] K8sError),

    /// Shell session errors
    #[error("Shell error: {0}")]
    Shell(#[from] ShellError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// State management errors
    #[error("State error: {0}")]
    State(#[from] StateError),

    /// Security validation errors
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    /// System/OS-level errors
    #[error("System error: {0}")]
    System(#[from] SystemError),

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Kubernetes-specific error types.
#[derive(Error, Debug)]
pub enum K8sError {
    /// Failed to connect to Kubernetes cluster
    #[error("Failed to connect to Kubernetes cluster: {message}")]
    ConnectionFailed { message: String },

    /// Failed to create Kubernetes client
    #[error("Failed to create Kubernetes client: {source}")]
    ClientCreationFailed {
        #[from]
        source: kube::Error,
    },

    /// Resource not found
    #[error("Resource not found: {resource_type}/{name} in namespace {namespace:?}")]
    ResourceNotFound {
        resource_type: String,
        name: String,
        namespace: Option<String>,
    },

    /// Invalid resource type
    #[error("Invalid resource type: {resource_type}")]
    InvalidResourceType { resource_type: String },

    /// Watch operation failed
    #[error("Watch operation failed for {resource_type}: {message}")]
    WatchFailed {
        resource_type: String,
        message: String,
    },

    /// Authentication/authorization failed
    #[error("Kubernetes authentication failed: {message}")]
    AuthFailed { message: String },

    /// Context switching failed
    #[error("Failed to switch to context '{context}': {message}")]
    ContextSwitchFailed { context: String, message: String },

    /// Resource validation failed
    #[error("Resource validation failed: {message}")]
    ValidationFailed { message: String },

    /// API server communication error
    #[error("API server communication error: {message}")]
    ApiError { message: String },
}

/// Shell session error types.
#[derive(Error, Debug)]
pub enum ShellError {
    /// Failed to start shell session
    #[error("Failed to start shell session for pod {pod_name} in namespace {namespace}: {message}")]
    SessionStartFailed {
        pod_name: String,
        namespace: String,
        message: String,
    },


    /// Shell input validation failed
    #[error("Shell input validation failed: {message}")]
    InputValidationFailed { message: String },

    /// Shell session terminated unexpectedly
    #[error("Shell session {session_id} terminated unexpectedly")]
    SessionTerminated { session_id: String },

    /// Failed to send input to shell
    #[error("Failed to send input to shell session {session_id}: {message}")]
    InputSendFailed {
        session_id: String,
        message: String,
    },

    /// Failed to resize shell
    #[error("Failed to resize shell session {session_id}: {message}")]
    ResizeFailed {
        session_id: String,
        message: String,
    },

    /// Container not found in pod
    #[error("Container '{container}' not found in pod {pod_name}")]
    ContainerNotFound {
        container: String,
        pod_name: String,
    },

    /// Pod not in running state
    #[error("Pod {pod_name} is not in running state")]
    PodNotRunning { pod_name: String },

    /// Session not found
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    /// Session not active
    #[error("Session not active: {session_id}")]
    SessionNotActive { session_id: String },

    /// Too many sessions
    #[error("Too many sessions: {current}/{max}")]
    TooManySessions { current: usize, max: usize },

    /// Input failed
    #[error("Input failed for session {session_id}: {message}")]
    InputFailed { session_id: String, message: String },

    /// Validation failed
    #[error("Validation failed for {field}: {message}")]
    ValidationFailed { field: String, message: String },
}

/// Configuration error types.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration validation failed
    #[error("Configuration validation failed: {field} - {message}")]
    ValidationFailed { field: String, message: String },

    /// Environment variable parsing failed
    #[error("Environment variable parsing failed: {variable} - {message}")]
    EnvironmentParsing { variable: String, message: String },

    /// Missing required configuration
    #[error("Missing required configuration: {field}")]
    MissingRequired { field: String },

    /// Invalid configuration value
    #[error("Invalid configuration value for {field}: {value} - expected {expected}")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },

    /// Configuration file not found
    #[error("Configuration file not found in any of: {paths:?}")]
    NotFound { paths: Vec<String> },

    /// File read error
    #[error("Failed to read configuration file {path}: {error}")]
    FileReadError { path: String, error: String },

    /// File write error
    #[error("Failed to write configuration file {path}: {error}")]
    FileWriteError { path: String, error: String },

    /// Configuration parsing error
    #[error("Failed to parse configuration file {path}: {error}")]
    ParseError { path: String, error: String },

    /// Configuration serialization error
    #[error("Failed to serialize configuration: {error}")]
    SerializationError { error: String },
}

/// State management error types.
#[derive(Error, Debug)]
pub enum StateError {
    /// Manager not initialized
    #[error("Manager not initialized: {manager_type}")]
    ManagerNotInitialized { manager_type: String },

    /// Manager already initialized
    #[error("Manager already initialized: {manager_type}")]
    ManagerAlreadyInitialized { manager_type: String },

    /// State corruption detected
    #[error("State corruption detected in {component}: {message}")]
    CorruptionDetected { component: String, message: String },

    /// Lock acquisition failed
    #[error("Failed to acquire lock for {resource}: {message}")]
    LockFailed { resource: String, message: String },

    /// Initialization sequence failed
    #[error("Initialization sequence failed at step {step}: {message}")]
    InitializationFailed { step: String, message: String },

    /// Cleanup failed
    #[error("Cleanup failed for component {component}: {reason}")]
    CleanupFailed { component: String, reason: String },
}

/// Security validation error types.
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Input sanitization failed
    #[error("Input sanitization failed: {input_type} - {message}")]
    SanitizationFailed {
        input_type: String,
        message: String,
    },

    /// Forbidden operation
    #[error("Forbidden operation: {operation} - {reason}")]
    ForbiddenOperation { operation: String, reason: String },

    /// Invalid command detected
    #[error("Invalid command detected: {command}")]
    InvalidCommand { command: String },

    /// Security policy violation
    #[error("Security policy violation: {policy} - {message}")]
    PolicyViolation { policy: String, message: String },
}

/// System-level error types.
#[derive(Error, Debug)]
pub enum SystemError {
    /// File descriptor limit exceeded
    #[error("File descriptor limit exceeded: current={current}, limit={limit}")]
    FdLimitExceeded { current: u64, limit: u64 },

    /// Process spawn failed
    #[error("Process spawn failed: {command} - {message}")]
    ProcessSpawnFailed { command: String, message: String },

    /// URL opening failed
    #[error("Failed to open URL {url}: {message}")]
    UrlOpenFailed { url: String, message: String },

    /// System resource unavailable
    #[error("System resource unavailable: {resource} - {message}")]
    ResourceUnavailable { resource: String, message: String },
}

// Convenience type aliases for commonly used result types
pub type AppResult<T> = Result<T, AppError>;
pub type K8sResult<T> = Result<T, K8sError>;
pub type ShellResult<T> = Result<T, ShellError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type StateResult<T> = Result<T, StateError>;

// Conversion implementations for better ergonomics
impl From<&str> for K8sError {
    fn from(message: &str) -> Self {
        K8sError::ApiError {
            message: message.to_string(),
        }
    }
}

impl From<String> for K8sError {
    fn from(message: String) -> Self {
        K8sError::ApiError { message }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::System(SystemError::ResourceUnavailable {
            resource: "unknown".to_string(),
            message: error.to_string(),
        })
    }
}

// Helper functions for common error creation patterns
impl K8sError {
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
        }
    }

    pub fn resource_not_found(
        resource_type: impl Into<String>,
        name: impl Into<String>,
        namespace: Option<String>,
    ) -> Self {
        Self::ResourceNotFound {
            resource_type: resource_type.into(),
            name: name.into(),
            namespace,
        }
    }

    pub fn watch_failed(resource_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WatchFailed {
            resource_type: resource_type.into(),
            message: message.into(),
        }
    }
}

impl ShellError {
    pub fn session_start_failed(
        pod_name: impl Into<String>,
        namespace: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::SessionStartFailed {
            pod_name: pod_name.into(),
            namespace: namespace.into(),
            message: message.into(),
        }
    }

    pub fn session_not_found(session_id: impl Into<String>) -> Self {
        Self::SessionNotFound {
            session_id: session_id.into(),
        }
    }
}

impl ConfigError {
    pub fn validation_failed(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationFailed {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn missing_required(field: impl Into<String>) -> Self {
        Self::MissingRequired {
            field: field.into(),
        }
    }
}

impl StateError {
    pub fn manager_not_initialized(manager_type: impl Into<String>) -> Self {
        Self::ManagerNotInitialized {
            manager_type: manager_type.into(),
        }
    }

    pub fn initialization_failed(step: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InitializationFailed {
            step: step.into(),
            message: message.into(),
        }
    }
}