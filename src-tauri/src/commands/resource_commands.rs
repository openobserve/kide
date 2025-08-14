//! Resource management commands using the new resource registry pattern.

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::commands::command_wrapper::*;
use crate::k8s::RESOURCE_REGISTRY;
use crate::errors::{AppError, AppResult, K8sError, K8sResult};

/// Command to get all resource categories with metadata.
pub struct GetResourceCategoriesCommand;

#[async_trait::async_trait]
impl StateCommand<Vec<ResourceCategory>> for GetResourceCategoriesCommand {
    async fn execute(&self, _state: &AppState) -> AppResult<Vec<ResourceCategory>> {
        let categories_map = RESOURCE_REGISTRY.get_resources_by_category();
        
        let mut categories: Vec<ResourceCategory> = categories_map
            .into_iter()
            .map(|(name, resources)| ResourceCategory {
                name,
                resources: resources
                    .into_iter()
                    .map(|metadata| K8sResourceInfo {
                        name: metadata.kind.clone(),
                        kind: metadata.kind,
                        api_version: metadata.api_version,
                        namespaced: metadata.is_namespaced,
                        short_names: metadata.short_names,
                        scalable: metadata.scalable,
                        watchable: metadata.watchable,
                    })
                    .collect(),
            })
            .collect();

        // Sort categories by name for consistent output
        categories.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(categories)
    }
}

/// Command to get information about a specific resource type.
pub struct GetResourceInfoCommand {
    pub resource_type: String,
}

#[async_trait::async_trait]
impl StateCommand<K8sResourceInfo> for GetResourceInfoCommand {
    async fn execute(&self, _state: &AppState) -> AppResult<K8sResourceInfo> {
        let handler = RESOURCE_REGISTRY
            .get_handler(&self.resource_type)
            .map_err(AppError::Kubernetes)?;
        
        let metadata = handler.metadata();
        
        Ok(K8sResourceInfo {
            name: metadata.kind.clone(),
            kind: metadata.kind.clone(),
            api_version: metadata.api_version.clone(),
            namespaced: metadata.is_namespaced,
            short_names: metadata.short_names.clone(),
            scalable: metadata.scalable,
            watchable: metadata.watchable,
        })
    }
}

/// Command to list resources of a specific type.
pub struct ListResourcesCommand {
    pub resource_type: String,
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
}

#[async_trait::async_trait]
impl K8sCommand<serde_json::Value> for ListResourcesCommand {
    async fn validate(&self) -> K8sResult<()> {
        if !RESOURCE_REGISTRY.is_registered(&self.resource_type) {
            return Err(K8sError::InvalidResourceType {
                resource_type: self.resource_type.clone(),
            });
        }
        Ok(())
    }

    async fn execute(&self, client: &kube::Client) -> K8sResult<serde_json::Value> {
        let handler = RESOURCE_REGISTRY.get_handler(&self.resource_type)?;
        
        handler
            .list_resources(client, self.namespace.clone(), self.label_selector.clone())
            .await
    }
}

/// Command to get a specific resource.
pub struct GetResourceCommand {
    pub resource_type: String,
    pub name: String,
    pub namespace: Option<String>,
}

#[async_trait::async_trait]
impl K8sCommand<serde_json::Value> for GetResourceCommand {
    async fn validate(&self) -> K8sResult<()> {
        if !RESOURCE_REGISTRY.is_registered(&self.resource_type) {
            return Err(K8sError::InvalidResourceType {
                resource_type: self.resource_type.clone(),
            });
        }
        
        let handler = RESOURCE_REGISTRY.get_handler(&self.resource_type)?;
        
        if handler.metadata().is_namespaced && self.namespace.is_none() {
            return Err(K8sError::ValidationFailed {
                message: format!(
                    "Namespace required for namespaced resource '{}'",
                    self.resource_type
                ),
            });
        }
        
        Ok(())
    }

    async fn execute(&self, client: &kube::Client) -> K8sResult<serde_json::Value> {
        let handler = RESOURCE_REGISTRY.get_handler(&self.resource_type)?;
        
        handler
            .get_resource(client, self.name.clone(), self.namespace.clone())
            .await
    }
}

/// Command to delete a resource.
pub struct DeleteResourceCommand {
    pub resource_type: String,
    pub name: String,
    pub namespace: Option<String>,
}

#[async_trait::async_trait]
impl K8sCommand<()> for DeleteResourceCommand {
    async fn validate(&self) -> K8sResult<()> {
        if !RESOURCE_REGISTRY.is_registered(&self.resource_type) {
            return Err(K8sError::InvalidResourceType {
                resource_type: self.resource_type.clone(),
            });
        }
        
        let handler = RESOURCE_REGISTRY.get_handler(&self.resource_type)?;
        
        if handler.metadata().is_namespaced && self.namespace.is_none() {
            return Err(K8sError::ValidationFailed {
                message: format!(
                    "Namespace required for namespaced resource '{}'",
                    self.resource_type
                ),
            });
        }
        
        Ok(())
    }

    async fn execute(&self, client: &kube::Client) -> K8sResult<()> {
        let handler = RESOURCE_REGISTRY.get_handler(&self.resource_type)?;
        
        handler
            .delete_resource(client, self.name.clone(), self.namespace.clone())
            .await
    }
}

/// Response structures for resource commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCategory {
    pub name: String,
    pub resources: Vec<K8sResourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sResourceInfo {
    pub name: String,
    pub kind: String,
    pub api_version: String,
    pub namespaced: bool,
    pub short_names: Vec<String>,
    pub scalable: bool,
    pub watchable: bool,
}

// Tauri command implementations
#[tauri::command]
pub async fn get_resource_categories_v2(state: State<'_, AppState>) -> Result<Vec<ResourceCategory>, String> {
    let command = GetResourceCategoriesCommand;
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn get_resource_info(
    state: State<'_, AppState>,
    resource_type: String,
) -> Result<K8sResourceInfo, String> {
    let command = GetResourceInfoCommand { resource_type };
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn list_resources_v2(
    state: State<'_, AppState>,
    resource_type: String,
    namespace: Option<String>,
    label_selector: Option<String>,
) -> Result<serde_json::Value, String> {
    let command = ListResourcesCommand {
        resource_type,
        namespace,
        label_selector,
    };
    to_tauri_result(execute_k8s_command(&state, command).await)
}

#[tauri::command]
pub async fn get_resource_v2(
    state: State<'_, AppState>,
    resource_type: String,
    name: String,
    namespace: Option<String>,
) -> Result<serde_json::Value, String> {
    let command = GetResourceCommand {
        resource_type,
        name,
        namespace,
    };
    to_tauri_result(execute_k8s_command(&state, command).await)
}

#[tauri::command]
pub async fn delete_resource_v2(
    state: State<'_, AppState>,
    resource_type: String,
    name: String,
    namespace: Option<String>,
) -> Result<(), String> {
    let command = DeleteResourceCommand {
        resource_type,
        name,
        namespace,
    };
    to_tauri_result(execute_k8s_command(&state, command).await)
}

/// Command to toggle CronJob suspend state.
pub struct ToggleCronJobSuspendCommand {
    pub name: String,
    pub namespace: Option<String>,
    pub suspend: bool,
}

#[async_trait::async_trait]
impl K8sCommand<()> for ToggleCronJobSuspendCommand {
    async fn validate(&self) -> K8sResult<()> {
        if self.namespace.is_none() {
            return Err(K8sError::ValidationFailed {
                message: "Namespace required for CronJob".to_string(),
            });
        }
        Ok(())
    }

    async fn execute(&self, client: &kube::Client) -> K8sResult<()> {
        use kube::api::{Api, Patch, PatchParams};
        use k8s_openapi::api::batch::v1::CronJob;
        use serde_json::json;
        
        let api: Api<CronJob> = if let Some(namespace) = &self.namespace {
            Api::namespaced(client.clone(), namespace)
        } else {
            return Err(K8sError::ValidationFailed {
                message: "Namespace is required for CronJob operations".to_string(),
            });
        };

        let patch = json!({
            "spec": {
                "suspend": self.suspend
            }
        });

        let patch_params = PatchParams::default();
        let patch_data = Patch::Merge(&patch);

        api.patch(&self.name, &patch_params, &patch_data)
            .await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to update CronJob suspend state: {}", e),
            })?;

        Ok(())
    }
}

#[tauri::command]
pub async fn toggle_cronjob_suspend(
    state: State<'_, AppState>,
    name: String,
    namespace: Option<String>,
    suspend: bool,
) -> Result<(), String> {
    let command = ToggleCronJobSuspendCommand {
        name,
        namespace,
        suspend,
    };
    to_tauri_result(execute_k8s_command(&state, command).await)
}

/// Command to trigger a CronJob manually by creating a Job from it.
pub struct TriggerCronJobCommand {
    pub name: String,
    pub namespace: Option<String>,
}

#[async_trait::async_trait]
impl K8sCommand<()> for TriggerCronJobCommand {
    async fn validate(&self) -> K8sResult<()> {
        if self.namespace.is_none() {
            return Err(K8sError::ValidationFailed {
                message: "Namespace required for CronJob".to_string(),
            });
        }
        Ok(())
    }

    async fn execute(&self, client: &kube::Client) -> K8sResult<()> {
        use kube::api::Api;
        use k8s_openapi::api::batch::v1::{CronJob, Job};
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
        
        let namespace = self.namespace.as_ref().unwrap();
        let cronjob_api: Api<CronJob> = Api::namespaced(client.clone(), namespace);
        let job_api: Api<Job> = Api::namespaced(client.clone(), namespace);

        // Get the CronJob to extract its job template
        let cronjob = cronjob_api.get(&self.name)
            .await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to get CronJob {}: {}", self.name, e),
            })?;

        let Some(cronjob_spec) = cronjob.spec else {
            return Err(K8sError::ValidationFailed {
                message: "CronJob has no spec".to_string(),
            });
        };

        let job_template = cronjob_spec.job_template;

        // Create a unique name for the triggered job
        let job_name = format!("{}-manual-{}", self.name, 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());

        // Create the Job from the CronJob's template
        let mut job_spec = job_template.spec.unwrap_or_default();
        
        // Ensure the job doesn't restart indefinitely on failure
        if job_spec.backoff_limit.is_none() {
            job_spec.backoff_limit = Some(3);
        }

        let mut job_labels = job_template.metadata
            .and_then(|m| m.labels)
            .unwrap_or_default();
        job_labels.insert("cronjob".to_string(), self.name.clone());
        job_labels.insert("triggered-by".to_string(), "manual".to_string());

        let job = Job {
            metadata: ObjectMeta {
                name: Some(job_name.clone()),
                namespace: Some(namespace.to_string()),
                labels: Some(job_labels),
                owner_references: cronjob.metadata.uid.map(|uid| {
                    vec![OwnerReference {
                        api_version: "batch/v1".to_string(),
                        kind: "CronJob".to_string(),
                        name: self.name.clone(),
                        uid,
                        controller: Some(false),
                        block_owner_deletion: Some(false),
                    }]
                }),
                ..Default::default()
            },
            spec: Some(job_spec),
            status: None,
        };

        // Create the Job
        job_api.create(&Default::default(), &job)
            .await
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to create Job from CronJob {}: {}", self.name, e),
            })?;

        Ok(())
    }
}

#[tauri::command]
pub async fn trigger_cronjob(
    state: State<'_, AppState>,
    name: String,
    namespace: Option<String>,
) -> Result<(), String> {
    let command = TriggerCronJobCommand {
        name,
        namespace,
    };
    to_tauri_result(execute_k8s_command(&state, command).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[tokio::test]
    async fn test_get_resource_categories() {
        let command = GetResourceCategoriesCommand;
        let state = AppState::new();
        
        let result = command.execute(&state).await;
        assert!(result.is_ok());
        
        let categories = result.unwrap();
        assert!(!categories.is_empty());
        
        // Should have common categories
        let category_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        assert!(category_names.contains(&"Workloads".to_string()));
        assert!(category_names.contains(&"Cluster".to_string()));
    }

    #[tokio::test]
    async fn test_get_resource_info() {
        let command = GetResourceInfoCommand {
            resource_type: "pods".to_string(),
        };
        let state = AppState::new();
        
        let result = command.execute(&state).await;
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert_eq!(info.kind, "Pod");
        assert_eq!(info.api_version, "v1");
        assert!(info.namespaced);
        assert!(!info.scalable);
        assert!(info.watchable);
    }

    #[tokio::test]
    async fn test_get_invalid_resource_info() {
        let command = GetResourceInfoCommand {
            resource_type: "invalid-resource".to_string(),
        };
        let state = AppState::new();
        
        let result = command.execute(&state).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_resources_validation() {
        let command = ListResourcesCommand {
            resource_type: "invalid-resource".to_string(),
            namespace: None,
            label_selector: None,
        };
        
        let result = command.validate().await;
        assert!(result.is_err());
        
        if let Err(K8sError::InvalidResourceType { resource_type }) = result {
            assert_eq!(resource_type, "invalid-resource");
        } else {
            panic!("Expected InvalidResourceType error");
        }
    }

    #[tokio::test]
    async fn test_get_resource_validation() {
        // Test missing namespace for namespaced resource
        let command = GetResourceCommand {
            resource_type: "pods".to_string(),
            name: "test-pod".to_string(),
            namespace: None,
        };
        
        let result = command.validate().await;
        assert!(result.is_err());
        
        if let Err(K8sError::ValidationFailed { message }) = result {
            assert!(message.contains("Namespace required"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[tokio::test]
    async fn test_delete_resource_validation() {
        // Test with valid namespaced resource
        let command = DeleteResourceCommand {
            resource_type: "pods".to_string(),
            name: "test-pod".to_string(),
            namespace: Some("default".to_string()),
        };
        
        let result = command.validate().await;
        assert!(result.is_ok());
        
        // Test with cluster-wide resource
        let command = DeleteResourceCommand {
            resource_type: "nodes".to_string(),
            name: "test-node".to_string(),
            namespace: None,
        };
        
        let result = command.validate().await;
        assert!(result.is_ok());
    }
}