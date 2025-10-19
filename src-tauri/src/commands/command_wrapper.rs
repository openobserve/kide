//! Command wrapper traits and utilities for consistent error handling and patterns.

use crate::errors::{AppError, AppResult, K8sError, K8sResult};
use crate::state::AppState;
use async_trait::async_trait;
use serde::Serialize;
use tauri::State;

/// Trait for commands that require a Kubernetes client.
#[async_trait]
pub trait K8sCommand<T>
where
    T: Send + Serialize,
{
    /// Execute the command with a connected Kubernetes client.
    async fn execute(&self, client: &kube::Client) -> K8sResult<T>;

    /// Optional validation before executing the command.
    async fn validate(&self) -> K8sResult<()> {
        Ok(())
    }
}

/// Trait for commands that only need application state.
#[async_trait]
pub trait StateCommand<T>
where
    T: Send + Serialize,
{
    /// Execute the command with application state.
    async fn execute(&self, state: &AppState) -> AppResult<T>;
}

/// Execute a Kubernetes command with proper error handling and client validation.
pub async fn execute_k8s_command<T, C>(
    state: &State<'_, AppState>, 
    command: C
) -> AppResult<T> 
where 
    T: Send + Serialize,
    C: K8sCommand<T> + Send + Sync,
{
    // Validate command before execution
    command.validate().await?;
    
    // Get client with proper error conversion
    let client = state.k8s_client.get_client().await
        .map_err(|e| K8sError::ConnectionFailed {
            message: e.to_string(),
        })?;
    
    // Execute command
    let result = command.execute(&client).await?;
    
    Ok(result)
}

/// Execute a state command with proper error handling.
pub async fn execute_state_command<T, C>(
    state: &State<'_, AppState>, 
    command: C
) -> AppResult<T> 
where 
    T: Send + Serialize,
    C: StateCommand<T> + Send + Sync,
{
    let result = command.execute(state).await?;
    Ok(result)
}

/// Convert AppResult to Tauri-compatible string result.
pub fn to_tauri_result<T: Serialize>(result: AppResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

/// Macro to simplify command implementation.
#[macro_export]
macro_rules! k8s_command {
    ($name:ident, $return_type:ty, $body:expr) => {
        pub struct $name {
            $(pub $field: $field_type,)*
        }

        #[async_trait::async_trait]
        impl crate::commands::command_wrapper::K8sCommand<$return_type> for $name {
            async fn execute(&self, client: &kube::Client) -> crate::errors::K8sResult<$return_type> {
                $body
            }
        }
    };
}

/// Specific command implementations for common operations.
/// Command to get Kubernetes namespaces.
pub struct GetNamespacesCommand;

#[async_trait]
impl K8sCommand<Vec<String>> for GetNamespacesCommand {
    async fn execute(&self, client: &kube::Client) -> K8sResult<Vec<String>> {
        use k8s_openapi::api::core::v1::Namespace;
        use kube::api::{Api, ListParams};
        
        let api: Api<Namespace> = Api::all(client.clone());
        
        let namespaces = api.list(&ListParams::default())
            .await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to list namespaces: {}", e),
            })?;
        
        let namespace_names: Vec<String> = namespaces
            .items
            .into_iter()
            .filter_map(|ns| ns.metadata.name)
            .collect();
        
        Ok(namespace_names)
    }
}

/// Command to connect to Kubernetes cluster.
pub struct ConnectK8sCommand;

#[async_trait]
impl StateCommand<()> for ConnectK8sCommand {
    async fn execute(&self, state: &AppState) -> AppResult<()> {
        state.k8s_client.connect().await
            .map_err(|e| K8sError::ConnectionFailed {
                message: e.to_string(),
            })?;
        
        state.initialize_managers().await
            .map_err(|e| AppError::State(crate::errors::StateError::InitializationFailed {
                step: "manager_initialization".to_string(),
                message: e,
            }))?;
        
        Ok(())
    }
}

/// Command to connect to Kubernetes with specific context.
pub struct ConnectK8sWithContextCommand {
    pub context_name: String,
}

#[async_trait]
impl StateCommand<()> for ConnectK8sWithContextCommand {
    async fn execute(&self, state: &AppState) -> AppResult<()> {
        state.k8s_client.connect_with_context(&self.context_name).await
            .map_err(|e| K8sError::ContextSwitchFailed {
                context: self.context_name.clone(),
                message: e.to_string(),
            })?;
        
        state.initialize_managers().await
            .map_err(|e| AppError::State(crate::errors::StateError::InitializationFailed {
                step: "manager_initialization".to_string(),
                message: e,
            }))?;
        
        Ok(())
    }
}

/// Command to get Kubernetes contexts.
pub struct GetK8sContextsCommand;

#[async_trait]
impl StateCommand<Vec<crate::k8s::K8sContext>> for GetK8sContextsCommand {
    async fn execute(&self, _state: &AppState) -> AppResult<Vec<crate::k8s::K8sContext>> {
        let contexts = crate::k8s::K8sClient::get_contexts().await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to get contexts: {}", e),
            })?;
        
        Ok(contexts)
    }
}

/// Command to get current Kubernetes context.
pub struct GetCurrentK8sContextCommand;

#[async_trait]
impl StateCommand<String> for GetCurrentK8sContextCommand {
    async fn execute(&self, _state: &AppState) -> AppResult<String> {
        let context = crate::k8s::K8sClient::get_current_context().await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to get current context: {}", e),
            })?;
        
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_namespaces_command() {
        let command = GetNamespacesCommand;
        
        // Test validation
        let validation_result = command.validate().await;
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_command_error_conversion() {
        let k8s_error = K8sError::ConnectionFailed {
            message: "test error".to_string(),
        };
        let app_error = AppError::Kubernetes(k8s_error);
        let string_error = app_error.to_string();
        
        assert!(string_error.contains("Kubernetes error"));
        assert!(string_error.contains("test error"));
    }
}