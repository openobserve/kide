//! Enhanced resource registry system for Kubernetes resource management.
//!
//! This module provides a comprehensive registry pattern for managing Kubernetes resources
//! with structured error handling, type safety, and extensibility.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use crate::errors::{K8sError, K8sResult};
use crate::k8s::watch::WatchManager;

/// Metadata information about a Kubernetes resource type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// Resource kind (e.g., "Pod", "Deployment")
    pub kind: String,
    /// API version (e.g., "v1", "apps/v1")
    pub api_version: String,
    /// Whether the resource is namespaced
    pub is_namespaced: bool,
    /// Resource category for grouping
    pub category: String,
    /// Short names for the resource
    pub short_names: Vec<String>,
    /// Whether the resource supports scaling
    pub scalable: bool,
    /// Whether the resource can be watched
    pub watchable: bool,
}

/// Trait for handling operations on a specific Kubernetes resource type.
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Get metadata for this resource type.
    fn metadata(&self) -> &ResourceMetadata;

    /// Start watching this resource type.
    async fn start_watch(
        &self,
        client: &kube::Client,
        watch_manager: &WatchManager,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()>;

    /// Stop watching this resource type.
    async fn stop_watch(
        &self,
        watch_manager: &WatchManager,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()>;

    /// Get resources of this type with optional filtering.
    async fn list_resources(
        &self,
        client: &kube::Client,
        namespace: Option<String>,
        label_selector: Option<String>,
    ) -> K8sResult<serde_json::Value>;

    /// Get a specific resource by name.
    async fn get_resource(
        &self,
        client: &kube::Client,
        name: String,
        namespace: Option<String>,
    ) -> K8sResult<serde_json::Value>;

    /// Delete a resource by name.
    async fn delete_resource(
        &self,
        client: &kube::Client,
        name: String,
        namespace: Option<String>,
    ) -> K8sResult<()>;

    /// Scale a resource (if supported).
    async fn scale_resource(
        &self,
        _client: &kube::Client,
        _name: String,
        _namespace: Option<String>,
        _replicas: i32,
    ) -> K8sResult<()> {
        if !self.metadata().scalable {
            return Err(K8sError::ValidationFailed {
                message: format!("Resource type '{}' is not scalable", self.metadata().kind),
            });
        }
        Err(K8sError::ValidationFailed {
            message: "Scale operation not implemented for this resource type".to_string(),
        })
    }
}

/// Generic resource handler for standard Kubernetes resources.
pub struct GenericResourceHandler<T>
where
    T: kube::Resource + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
    <T as kube::Resource>::DynamicType: Default + Send + Sync,
{
    metadata: ResourceMetadata,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GenericResourceHandler<T>
where
    T: kube::Resource + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
    <T as kube::Resource>::DynamicType: Default + Send + Sync,
{
    pub fn new(metadata: ResourceMetadata) -> Self {
        Self {
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> ResourceHandler for GenericResourceHandler<T>
where
    T: kube::Resource + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
    <T as kube::Resource>::DynamicType: Default + Send + Sync,
    T: k8s_openapi::Resource + k8s_openapi::Metadata,
{
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    async fn start_watch(
        &self,
        _client: &kube::Client,
        _watch_manager: &WatchManager,
        _namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        // Implementation would delegate to WatchManager
        Ok(())
    }

    async fn stop_watch(
        &self,
        _watch_manager: &WatchManager,
        _namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        // Implementation would delegate to WatchManager
        Ok(())
    }

    async fn list_resources(
        &self,
        _client: &kube::Client,
        _namespace: Option<String>,
        _label_selector: Option<String>,
    ) -> K8sResult<serde_json::Value> {
        // TODO: Implement generic listing logic
        // For now, return empty array
        Ok(serde_json::json!([]))
    }

    async fn get_resource(
        &self,
        _client: &kube::Client,
        name: String,
        namespace: Option<String>,
    ) -> K8sResult<serde_json::Value> {
        // TODO: Implement generic get logic
        Err(K8sError::ResourceNotFound {
            resource_type: self.metadata.kind.clone(),
            name,
            namespace,
        })
    }

    async fn delete_resource(
        &self,
        _client: &kube::Client,
        name: String,
        namespace: Option<String>,
    ) -> K8sResult<()> {
        // TODO: Implement generic delete logic
        Err(K8sError::ResourceNotFound {
            resource_type: self.metadata.kind.clone(),
            name,
            namespace,
        })
    }
}

/// Central registry for all Kubernetes resource handlers.
pub struct ResourceRegistry {
    handlers: HashMap<String, Arc<dyn ResourceHandler>>,
}

impl ResourceRegistry {
    /// Create a new resource registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a resource handler.
    pub fn register<H>(&mut self, resource_type: String, handler: H)
    where
        H: ResourceHandler + 'static,
    {
        self.handlers.insert(resource_type, Arc::new(handler));
    }

    /// Get a resource handler by type.
    pub fn get_handler(&self, resource_type: &str) -> K8sResult<Arc<dyn ResourceHandler>> {
        self.handlers
            .get(resource_type)
            .cloned()
            .ok_or_else(|| K8sError::InvalidResourceType {
                resource_type: resource_type.to_string(),
            })
    }

    /// List all registered resource types.
    pub fn list_resource_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Get metadata for all registered resources.
    pub fn get_all_metadata(&self) -> Vec<ResourceMetadata> {
        self.handlers
            .values()
            .map(|handler| handler.metadata().clone())
            .collect()
    }

    /// Check if a resource type is registered.
    pub fn is_registered(&self, resource_type: &str) -> bool {
        self.handlers.contains_key(resource_type)
    }

    /// Get resources grouped by category.
    pub fn get_resources_by_category(&self) -> HashMap<String, Vec<ResourceMetadata>> {
        let mut categories: HashMap<String, Vec<ResourceMetadata>> = HashMap::new();
        
        for handler in self.handlers.values() {
            let metadata = handler.metadata().clone();
            categories
                .entry(metadata.category.clone())
                .or_insert_with(Vec::new)
                .push(metadata);
        }
        
        categories
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global resource registry instance.
pub static RESOURCE_REGISTRY: Lazy<ResourceRegistry> = Lazy::new(|| {
    let mut registry = ResourceRegistry::new();
    
    // Register core resources
    register_core_resources(&mut registry);
    
    registry
});

/// Register all core Kubernetes resources.
fn register_core_resources(registry: &mut ResourceRegistry) {
    use k8s_openapi::api::{
        apps::v1::{Deployment, DaemonSet, ReplicaSet, StatefulSet},
        core::v1::{Pod, Service, ConfigMap, Secret, Namespace, Node},
        batch::v1::{Job, CronJob},
        networking::v1::{Ingress, NetworkPolicy},
        rbac::v1::{Role, RoleBinding, ClusterRole, ClusterRoleBinding},
    };

    // Workloads
    registry.register(
        "pods".to_string(),
        GenericResourceHandler::<Pod>::new(ResourceMetadata {
            kind: "Pod".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["po".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "deployments".to_string(),
        GenericResourceHandler::<Deployment>::new(ResourceMetadata {
            kind: "Deployment".to_string(),
            api_version: "apps/v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["deploy".to_string()],
            scalable: true,
            watchable: true,
        }),
    );

    registry.register(
        "daemonsets".to_string(),
        GenericResourceHandler::<DaemonSet>::new(ResourceMetadata {
            kind: "DaemonSet".to_string(),
            api_version: "apps/v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["ds".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "statefulsets".to_string(),
        GenericResourceHandler::<StatefulSet>::new(ResourceMetadata {
            kind: "StatefulSet".to_string(),
            api_version: "apps/v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["sts".to_string()],
            scalable: true,
            watchable: true,
        }),
    );

    registry.register(
        "replicasets".to_string(),
        GenericResourceHandler::<ReplicaSet>::new(ResourceMetadata {
            kind: "ReplicaSet".to_string(),
            api_version: "apps/v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["rs".to_string()],
            scalable: true,
            watchable: true,
        }),
    );

    // Services and Discovery
    registry.register(
        "services".to_string(),
        GenericResourceHandler::<Service>::new(ResourceMetadata {
            kind: "Service".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: true,
            category: "Services and Discovery".to_string(),
            short_names: vec!["svc".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    // Configuration and Storage
    registry.register(
        "configmaps".to_string(),
        GenericResourceHandler::<ConfigMap>::new(ResourceMetadata {
            kind: "ConfigMap".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: true,
            category: "Configuration and Storage".to_string(),
            short_names: vec!["cm".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "secrets".to_string(),
        GenericResourceHandler::<Secret>::new(ResourceMetadata {
            kind: "Secret".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: true,
            category: "Configuration and Storage".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );

    // Cluster Resources
    registry.register(
        "namespaces".to_string(),
        GenericResourceHandler::<Namespace>::new(ResourceMetadata {
            kind: "Namespace".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: false,
            category: "Cluster".to_string(),
            short_names: vec!["ns".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "nodes".to_string(),
        GenericResourceHandler::<Node>::new(ResourceMetadata {
            kind: "Node".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: false,
            category: "Cluster".to_string(),
            short_names: vec!["no".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    // Batch Jobs
    registry.register(
        "jobs".to_string(),
        GenericResourceHandler::<Job>::new(ResourceMetadata {
            kind: "Job".to_string(),
            api_version: "batch/v1".to_string(),
            is_namespaced: true,
            category: "Batch".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "cronjobs".to_string(),
        GenericResourceHandler::<CronJob>::new(ResourceMetadata {
            kind: "CronJob".to_string(),
            api_version: "batch/v1".to_string(),
            is_namespaced: true,
            category: "Batch".to_string(),
            short_names: vec!["cj".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    // Networking
    registry.register(
        "ingresses".to_string(),
        GenericResourceHandler::<Ingress>::new(ResourceMetadata {
            kind: "Ingress".to_string(),
            api_version: "networking.k8s.io/v1".to_string(),
            is_namespaced: true,
            category: "Networking".to_string(),
            short_names: vec!["ing".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "networkpolicies".to_string(),
        GenericResourceHandler::<NetworkPolicy>::new(ResourceMetadata {
            kind: "NetworkPolicy".to_string(),
            api_version: "networking.k8s.io/v1".to_string(),
            is_namespaced: true,
            category: "Networking".to_string(),
            short_names: vec!["netpol".to_string()],
            scalable: false,
            watchable: true,
        }),
    );

    // RBAC
    registry.register(
        "roles".to_string(),
        GenericResourceHandler::<Role>::new(ResourceMetadata {
            kind: "Role".to_string(),
            api_version: "rbac.authorization.k8s.io/v1".to_string(),
            is_namespaced: true,
            category: "RBAC".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "rolebindings".to_string(),
        GenericResourceHandler::<RoleBinding>::new(ResourceMetadata {
            kind: "RoleBinding".to_string(),
            api_version: "rbac.authorization.k8s.io/v1".to_string(),
            is_namespaced: true,
            category: "RBAC".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "clusterroles".to_string(),
        GenericResourceHandler::<ClusterRole>::new(ResourceMetadata {
            kind: "ClusterRole".to_string(),
            api_version: "rbac.authorization.k8s.io/v1".to_string(),
            is_namespaced: false,
            category: "RBAC".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );

    registry.register(
        "clusterrolebindings".to_string(),
        GenericResourceHandler::<ClusterRoleBinding>::new(ResourceMetadata {
            kind: "ClusterRoleBinding".to_string(),
            api_version: "rbac.authorization.k8s.io/v1".to_string(),
            is_namespaced: false,
            category: "RBAC".to_string(),
            short_names: vec![],
            scalable: false,
            watchable: true,
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_registry_creation() {
        let registry = ResourceRegistry::new();
        assert_eq!(registry.list_resource_types().len(), 0);
    }

    #[test]
    fn test_resource_metadata() {
        let metadata = ResourceMetadata {
            kind: "Pod".to_string(),
            api_version: "v1".to_string(),
            is_namespaced: true,
            category: "Workloads".to_string(),
            short_names: vec!["po".to_string()],
            scalable: false,
            watchable: true,
        };

        assert_eq!(metadata.kind, "Pod");
        assert!(metadata.is_namespaced);
        assert!(!metadata.scalable);
        assert!(metadata.watchable);
    }

    #[test]
    fn test_global_registry() {
        let resource_types = RESOURCE_REGISTRY.list_resource_types();
        assert!(!resource_types.is_empty());
        assert!(resource_types.contains(&"pods".to_string()));
        assert!(resource_types.contains(&"deployments".to_string()));
    }

    #[test]
    fn test_resources_by_category() {
        let categories = RESOURCE_REGISTRY.get_resources_by_category();
        assert!(categories.contains_key("Workloads"));
        assert!(categories.contains_key("Services and Discovery"));
        assert!(categories.contains_key("Cluster"));
        
        let workloads = &categories["Workloads"];
        assert!(workloads.iter().any(|r| r.kind == "Pod"));
        assert!(workloads.iter().any(|r| r.kind == "Deployment"));
    }

    #[tokio::test]
    async fn test_invalid_resource_type() {
        let result = RESOURCE_REGISTRY.get_handler("invalid-resource");
        assert!(result.is_err());
        
        if let Err(K8sError::InvalidResourceType { resource_type }) = result {
            assert_eq!(resource_type, "invalid-resource");
        } else {
            panic!("Expected InvalidResourceType error");
        }
    }
}