//! Error types for Kubernetes watch operations.
//!
//! This module provides structured error handling for the Kubernetes watch system,
//! replacing generic anyhow errors with specific, actionable error types that provide
//! better context and enable more precise error handling.
//!
//! # Error Categories
//!
//! - **Resource Errors**: Invalid or unsupported resource types
//! - **Client Errors**: Kubernetes API client creation and connection issues
//! - **Watch Errors**: Problems starting, stopping, or managing watches
//! - **State Errors**: Conflicts with existing watch state

use thiserror::Error;

/// Comprehensive error types for Kubernetes watch operations.
///
/// These errors provide detailed context about failures in the watch system,
/// enabling better error reporting and recovery strategies.
#[derive(Error, Debug)]
pub enum K8sWatchError {
    /// Resource type is not supported by the watch system
    #[error("Unsupported resource type: {resource_type}")]
    UnsupportedResourceType { resource_type: String },
    
    /// Failed to create Kubernetes client
    #[error("Failed to create Kubernetes client: {source}")]
    ClientCreationFailed {
        #[from]
        source: kube::Error,
    },
    
    /// Failed to start watch for resource
    #[error("Failed to start watch for resource {resource_type}: {message}")]
    WatchStartFailed {
        resource_type: String,
        message: String,
    },
    
    /// Failed to stop watch for resource
    #[error("Failed to stop watch for resource {resource_type}: {message}")]
    WatchStopFailed {
        resource_type: String,
        message: String,
    },
    
    /// Watch is already active for the given key
    #[error("Watch is already active for key: {watch_key}")]
    WatchAlreadyActive { watch_key: String },
    
    /// Generic anyhow error for compatibility
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl K8sWatchError {
    /// Creates an error for unsupported resource types.
    ///
    /// # Arguments
    /// * `resource_type` - The invalid resource type that was requested
    ///
    /// # Returns
    /// A structured error with the resource type context
    pub fn unsupported_resource(resource_type: impl Into<String>) -> Self {
        Self::UnsupportedResourceType {
            resource_type: resource_type.into(),
        }
    }
    
    /// Creates an error for failed watch start operations.
    ///
    /// # Arguments
    /// * `resource_type` - The resource type for which the watch failed to start
    /// * `message` - Detailed error message explaining the failure
    ///
    /// # Returns
    /// A structured error with full context about the failure
    pub fn watch_start_failed(resource_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WatchStartFailed {
            resource_type: resource_type.into(),
            message: message.into(),
        }
    }
    
    /// Creates an error for failed watch stop operations.
    ///
    /// # Arguments
    /// * `resource_type` - The resource type for which the watch failed to stop
    /// * `message` - Detailed error message explaining the failure
    ///
    /// # Returns
    /// A structured error with full context about the failure
    pub fn watch_stop_failed(resource_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WatchStopFailed {
            resource_type: resource_type.into(),
            message: message.into(),
        }
    }
}

/// Convenience type alias for Kubernetes watch operation results.
///
/// This type alias provides a consistent return type across all watch operations,
/// making error handling more predictable and reducing boilerplate code.
///
/// # Examples
/// ```rust,ignore
/// fn start_pod_watch() -> K8sWatchResult<()> {
///     // ... watch logic
///     Ok(())
/// }
/// ```
pub type K8sWatchResult<T> = Result<T, K8sWatchError>;