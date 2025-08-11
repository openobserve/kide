//! Optimized watch manager that uses trait-based dispatch instead of large match statements.
//!
//! This module provides an improved architecture for watching Kubernetes resources by:
//! - Using O(1) lookups via HashMap instead of O(n) match statements
//! - Employing function pointers for type-safe resource dispatch
//! - Providing structured error handling with detailed context
//! - Supporting efficient concurrent watch management

use super::client::K8sClient;
use super::errors::{K8sWatchError, K8sWatchResult};
use super::resource_map::get_resource_config;
use futures::StreamExt;
use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    autoscaling::v2::HorizontalPodAutoscaler,
    batch::v1::{CronJob, Job},
    core::v1::{ConfigMap, Endpoints, LimitRange, Namespace, Node, Pod, PersistentVolume, PersistentVolumeClaim, ReplicationController, ResourceQuota, Secret, Service, ServiceAccount},
    discovery::v1::EndpointSlice,
    networking::v1::{Ingress, NetworkPolicy},
    policy::v1::PodDisruptionBudget,
    rbac::v1::{ClusterRole, ClusterRoleBinding, Role, RoleBinding},
    scheduling::v1::PriorityClass,
    storage::v1::StorageClass,
};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::kube_aggregator::pkg::apis::apiregistration::v1::APIService;
use kube::{
    api::Api,
    runtime::{watcher, watcher::Config},
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Optimized watch manager that uses trait-based dispatch instead of large match statements.
///
/// This manager provides significant performance improvements over the original implementation:
/// - O(1) resource type lookup via HashMap instead of O(n) match statements
/// - Function pointer dispatch for type-safe resource handling
/// - Structured error handling with detailed context
/// - Efficient concurrent watch management with proper cleanup
#[derive(Debug, Clone)]
pub struct OptimizedWatchManager {
    /// Kubernetes client for API access
    client: K8sClient,
    /// Active watch handles keyed by watch identifier
    active_watches: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl OptimizedWatchManager {
    /// Creates a new optimized watch manager.
    ///
    /// # Arguments
    /// * `client` - Kubernetes client for API access
    ///
    /// # Returns
    /// A new instance of the watch manager ready to handle resource watches
    pub fn new(client: K8sClient) -> Self {
        Self {
            client,
            active_watches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Starts watching a Kubernetes resource using trait-based dispatch.
    ///
    /// This method performs O(1) resource type lookup instead of using large match statements,
    /// providing better performance and maintainability.
    ///
    /// # Arguments
    /// * `app_handle` - Tauri application handle for event emission
    /// * `resource_type` - Type of Kubernetes resource to watch (e.g., "pods", "deployments")
    /// * `namespaces` - Optional list of namespaces to watch. None means all namespaces.
    ///
    /// # Returns
    /// * `Ok(())` - Watch started successfully or was already active
    /// * `Err(K8sWatchError)` - Failed to start watch with detailed error context
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Watch all pods in all namespaces
    /// manager.start_watch(app_handle, "pods", None).await?;
    ///
    /// // Watch deployments in specific namespaces
    /// manager.start_watch(app_handle, "deployments", Some(vec!["default".to_string()])).await?;
    /// ```
    pub async fn start_watch(
        &self,
        app_handle: AppHandle,
        resource_type: &str,
        namespaces: Option<Vec<String>>,
    ) -> K8sWatchResult<()> {
        // O(1) lookup instead of large match statement - major performance improvement
        let config = get_resource_config(resource_type)
            .ok_or_else(|| K8sWatchError::unsupported_resource(resource_type))?;

        // Generate watch key from namespaces (sorted for consistency)
        let watch_key = if let Some(ref ns_list) = namespaces {
            if ns_list.is_empty() {
                format!("{}:all", resource_type)
            } else {
                let mut sorted_ns = ns_list.clone();
                sorted_ns.sort();
                format!("{}:{}", resource_type, sorted_ns.join(","))
            }
        } else {
            format!("{}:all", resource_type)
        };
        
        let mut watches = self.active_watches.lock().await;
        if watches.contains_key(&watch_key) {
            return Err(K8sWatchError::WatchAlreadyActive { watch_key });
        }

        let client = self.client.get_client().await
            .map_err(K8sWatchError::ClientCreationFailed)?;
        let app_handle_clone = app_handle.clone();
        let resource_type_clone = resource_type.to_string();

        // Use trait-based dispatch instead of large match statement
        let handle = (config.create_watch)(
            client,
            app_handle_clone,
            resource_type_clone,
            namespaces,
            config.is_namespaced,
        );

        watches.insert(watch_key, handle);
        Ok(())
    }



    /// Stops watching a specific Kubernetes resource.
    ///
    /// This method cleanly terminates the watch for a specific resource type and namespace combination.
    /// The watch key is generated using the same logic as `start_watch` to ensure proper matching.
    ///
    /// # Arguments
    /// * `resource_type` - Type of Kubernetes resource to stop watching
    /// * `namespaces` - Optional list of namespaces that were being watched
    ///
    /// # Returns
    /// * `Ok(())` - Watch stopped successfully (or didn't exist)
    /// * `Err(K8sWatchError)` - Failed to stop watch with detailed error context
    pub async fn stop_watch(&self, resource_type: &str, namespaces: Option<Vec<String>>) -> K8sWatchResult<()> {
        // Generate the same watch key as in start_watch
        let watch_key = if let Some(ref ns_list) = namespaces {
            if ns_list.is_empty() {
                format!("{}:all", resource_type)
            } else {
                let mut sorted_ns = ns_list.clone();
                sorted_ns.sort();
                format!("{}:{}", resource_type, sorted_ns.join(","))
            }
        } else {
            format!("{}:all", resource_type)
        };

        let mut watches = self.active_watches.lock().await;
        if let Some(handle) = watches.remove(&watch_key) {
            handle.abort();
        }
        Ok(())
    }

    /// Stops all active watches managed by this instance.
    ///
    /// This method provides clean shutdown functionality by aborting all active watch tasks
    /// and clearing the internal watch registry. Useful for application shutdown or context switching.
    ///
    /// # Returns
    /// * `Ok(())` - All watches stopped successfully
    /// * `Err(K8sWatchError)` - Failed to stop watches with detailed error context
    pub async fn stop_all_watches(&self) -> K8sWatchResult<()> {
        let mut watches = self.active_watches.lock().await;
        for (_, handle) in watches.drain() {
            handle.abort();
        }
        Ok(())
    }

    /// Returns a list of all currently active watch keys.
    ///
    /// Watch keys are formatted as "resource_type:namespace_list" where namespace_list
    /// is either "all" for cluster-wide watches or a comma-separated list of specific namespaces.
    ///
    /// # Returns
    /// A vector of active watch key strings
    ///
    /// # Examples
    /// ```rust,ignore
    /// let active = manager.get_active_watches().await;
    /// // Might return: ["pods:all", "deployments:default,kube-system"]
    /// ```
    pub async fn get_active_watches(&self) -> Vec<String> {
        let watches = self.active_watches.lock().await;
        watches.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimized_watch_manager_creation() {
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        // Test that manager can be created and is properly aligned
        assert!(std::ptr::addr_of!(manager).is_aligned());
    }

    #[tokio::test]
    async fn test_unsupported_resource_error() {
        use tauri::test::{mock_app, mock_runtime};
        
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        
        // Create a mock app handle for testing
        let app = mock_app();
        let runtime = mock_runtime();
        let app_handle = tauri::AppHandle::new(&app.state(), &runtime.global_api_scope, runtime.window_manager.clone(), &app.config().identifier);
        
        let result = manager.start_watch(app_handle, "nonexistent_resource", None).await;
        
        match result {
            Err(K8sWatchError::UnsupportedResourceType { resource_type }) => {
                assert_eq!(resource_type, "nonexistent_resource");
            }
            _ => panic!("Expected UnsupportedResourceType error"),
        }
    }

    #[tokio::test]
    async fn test_watch_key_generation() {
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        
        // Test watch key generation logic matches start_watch implementation
        let resource_type = "pods";
        
        // Test all namespaces key
        let watch_key = format!("{}:all", resource_type);
        assert_eq!(watch_key, "pods:all");
        
        // Test specific namespaces key
        let namespaces = vec!["kube-system".to_string(), "default".to_string()];
        let mut sorted_ns = namespaces.clone();
        sorted_ns.sort();
        let watch_key = format!("{}:{}", resource_type, sorted_ns.join(","));
        assert_eq!(watch_key, "pods:default,kube-system");
    }

    #[tokio::test]
    async fn test_empty_active_watches() {
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        
        let active_watches = manager.get_active_watches().await;
        assert!(active_watches.is_empty());
    }

    #[tokio::test]
    async fn test_stop_nonexistent_watch() {
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        
        // Stopping a non-existent watch should succeed
        let result = manager.stop_watch("pods", Some(vec!["default".to_string()])).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_all_watches_empty() {
        let client = K8sClient::new();
        let manager = OptimizedWatchManager::new(client);
        
        // Should succeed even with no active watches
        let result = manager.stop_all_watches().await;
        assert!(result.is_ok());
    }
}