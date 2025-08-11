//! Resource configuration and dispatch system for Kubernetes resource types.
//!
//! This module provides a high-performance lookup system for Kubernetes resource metadata
//! and watch creation functions. Instead of using large match statements (O(n) complexity),
//! it uses HashMap-based lookups (O(1) complexity) with function pointers for type-safe dispatch.
//!
//! # Architecture
//!
//! The system is built around:
//! - `ResourceConfig`: Metadata and dispatch function for each resource type
//! - `RESOURCE_CONFIGS`: Static HashMap for O(1) resource lookups
//! - Function pointers: Type-safe dispatch without runtime type checking
//!
//! # Performance Benefits
//!
//! - O(1) lookup time vs O(n) match statements
//! - Zero-cost function pointer dispatch
//! - Compile-time type safety for all resource types
//! - Lazy initialization prevents startup overhead

use std::collections::HashMap;
use once_cell::sync::Lazy;
use tauri::AppHandle;
use tokio::task::JoinHandle;

/// Configuration for a Kubernetes resource type including metadata and watch dispatch.
///
/// This struct contains all the information needed to create and manage watches for
/// a specific Kubernetes resource type, including whether it's namespaced and
/// a function pointer for type-safe watch creation.
#[derive(Clone)]
pub struct ResourceConfig {
    /// Whether this resource type is namespaced (true) or cluster-wide (false)
    pub is_namespaced: bool,
    /// Human-readable category for grouping related resources
    pub category: &'static str,
    /// Function pointer to create a typed watch for this resource type.
    /// 
    /// This enables O(1) dispatch without large match statements while maintaining
    /// compile-time type safety for all supported resource types.
    pub create_watch: fn(
        client: kube::Client,
        app_handle: AppHandle,
        resource_type: String,
        namespaces: Option<Vec<String>>,
        is_namespaced: bool,
    ) -> JoinHandle<()>,
}

// Custom Debug implementation since function pointers don't implement Debug
impl std::fmt::Debug for ResourceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceConfig")
            .field("is_namespaced", &self.is_namespaced)
            .field("category", &self.category)
            .field("create_watch", &"<function pointer>")
            .finish()
    }
}

use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    autoscaling::v2::HorizontalPodAutoscaler,
    batch::v1::{CronJob, Job},
    certificates::v1::CertificateSigningRequest,
    core::v1::{
        ConfigMap, Endpoints, LimitRange, Namespace, Node, Pod, PersistentVolume,
        PersistentVolumeClaim, ReplicationController, ResourceQuota, Secret, Service,
        ServiceAccount,
    },
    discovery::v1::EndpointSlice,
    networking::v1::{Ingress, IngressClass, NetworkPolicy},
    node::v1::RuntimeClass,
    policy::v1::PodDisruptionBudget,
    rbac::v1::{ClusterRole, ClusterRoleBinding, Role, RoleBinding},
    scheduling::v1::PriorityClass,
    storage::v1::{CSIDriver, CSINode, StorageClass},
};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::kube_aggregator::pkg::apis::apiregistration::v1::APIService;

/// Static resource configuration map for O(1) lookup instead of large match statements
pub static RESOURCE_CONFIGS: Lazy<HashMap<&'static str, ResourceConfig>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Workloads - Namespaced resources
    map.insert("pods", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<Pod> 
    });
    map.insert("deployments", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<Deployment> 
    });
    map.insert("statefulsets", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<StatefulSet> 
    });
    map.insert("daemonsets", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<DaemonSet> 
    });
    map.insert("jobs", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<Job> 
    });
    map.insert("cronjobs", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<CronJob> 
    });
    map.insert("replicasets", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<ReplicaSet> 
    });
    map.insert("replicationcontrollers", ResourceConfig { 
        is_namespaced: true, 
        category: "Workloads", 
        create_watch: super::watch::create_typed_watch::<ReplicationController> 
    });
    
    // Services & Networking - Namespaced resources  
    map.insert("services", ResourceConfig { 
        is_namespaced: true, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<Service> 
    });
    map.insert("ingresses", ResourceConfig { 
        is_namespaced: true, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<Ingress> 
    });
    map.insert("ingressclasses", ResourceConfig { 
        is_namespaced: false, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<IngressClass> 
    });
    map.insert("networkpolicies", ResourceConfig { 
        is_namespaced: true, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<NetworkPolicy> 
    });
    map.insert("endpointslices", ResourceConfig { 
        is_namespaced: true, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<EndpointSlice> 
    });
    map.insert("endpoints", ResourceConfig { 
        is_namespaced: true, 
        category: "Services & Networking", 
        create_watch: super::watch::create_typed_watch::<Endpoints> 
    });
    
    // Configuration & Storage - Mixed scope
    map.insert("configmaps", ResourceConfig { 
        is_namespaced: true, 
        category: "Configuration", 
        create_watch: super::watch::create_typed_watch::<ConfigMap> 
    });
    map.insert("secrets", ResourceConfig { 
        is_namespaced: true, 
        category: "Configuration", 
        create_watch: super::watch::create_typed_watch::<Secret> 
    });
    map.insert("persistentvolumeclaims", ResourceConfig { 
        is_namespaced: true, 
        category: "Storage", 
        create_watch: super::watch::create_typed_watch::<PersistentVolumeClaim> 
    });
    map.insert("persistentvolumes", ResourceConfig { 
        is_namespaced: false, 
        category: "Storage", 
        create_watch: super::watch::create_typed_watch::<PersistentVolume> 
    });
    map.insert("storageclasses", ResourceConfig { 
        is_namespaced: false, 
        category: "Storage", 
        create_watch: super::watch::create_typed_watch::<StorageClass> 
    });
    map.insert("csidrivers", ResourceConfig { 
        is_namespaced: false, 
        category: "Storage", 
        create_watch: super::watch::create_typed_watch::<CSIDriver> 
    });
    map.insert("csinodes", ResourceConfig { 
        is_namespaced: false, 
        category: "Storage", 
        create_watch: super::watch::create_typed_watch::<CSINode> 
    });
    
    // Cluster Administration - Mixed scope  
    map.insert("serviceaccounts", ResourceConfig { 
        is_namespaced: true, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<ServiceAccount> 
    });
    map.insert("resourcequotas", ResourceConfig { 
        is_namespaced: true, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<ResourceQuota> 
    });
    map.insert("limitranges", ResourceConfig { 
        is_namespaced: true, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<LimitRange> 
    });
    map.insert("poddisruptionbudgets", ResourceConfig { 
        is_namespaced: true, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<PodDisruptionBudget> 
    });
    map.insert("namespaces", ResourceConfig { 
        is_namespaced: false, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<Namespace> 
    });
    map.insert("nodes", ResourceConfig { 
        is_namespaced: false, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<Node> 
    });
    map.insert("priorityclasses", ResourceConfig { 
        is_namespaced: false, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<PriorityClass> 
    });
    map.insert("runtimeclasses", ResourceConfig { 
        is_namespaced: false, 
        category: "Cluster Administration", 
        create_watch: super::watch::create_typed_watch::<RuntimeClass> 
    });
    
    // Security & Access Control - Mixed scope
    map.insert("roles", ResourceConfig { 
        is_namespaced: true, 
        category: "Security & Access Control", 
        create_watch: super::watch::create_typed_watch::<Role> 
    });
    map.insert("rolebindings", ResourceConfig { 
        is_namespaced: true, 
        category: "Security & Access Control", 
        create_watch: super::watch::create_typed_watch::<RoleBinding> 
    });
    map.insert("clusterroles", ResourceConfig { 
        is_namespaced: false, 
        category: "Security & Access Control", 
        create_watch: super::watch::create_typed_watch::<ClusterRole> 
    });
    map.insert("clusterrolebindings", ResourceConfig { 
        is_namespaced: false, 
        category: "Security & Access Control", 
        create_watch: super::watch::create_typed_watch::<ClusterRoleBinding> 
    });
    map.insert("certificatesigningrequests", ResourceConfig { 
        is_namespaced: false, 
        category: "Security & Access Control", 
        create_watch: super::watch::create_typed_watch::<CertificateSigningRequest> 
    });
    
    // Scaling - Namespaced resources
    map.insert("horizontalpodautoscalers", ResourceConfig { 
        is_namespaced: true, 
        category: "Scaling", 
        create_watch: super::watch::create_typed_watch::<HorizontalPodAutoscaler> 
    });
    
    // Custom Resources - Cluster-wide resources
    map.insert("customresourcedefinitions", ResourceConfig { 
        is_namespaced: false, 
        category: "Custom Resources", 
        create_watch: super::watch::create_typed_watch::<CustomResourceDefinition> 
    });
    map.insert("apiservices", ResourceConfig { 
        is_namespaced: false, 
        category: "Custom Resources", 
        create_watch: super::watch::create_typed_watch::<APIService> 
    });
    
    map
});

/// Retrieves resource configuration by name with O(1) lookup complexity.
///
/// This function provides constant-time access to resource metadata and dispatch functions,
/// significantly outperforming traditional match-based lookups for large numbers of resource types.
///
/// # Arguments
/// * `resource_type` - The lowercase resource type name (e.g., "pods", "deployments")
///
/// # Returns
/// * `Some(&ResourceConfig)` - Configuration if the resource type is supported
/// * `None` - If the resource type is not supported
///
/// # Examples
/// ```rust,ignore
/// let config = get_resource_config("pods")?;
/// assert!(config.is_namespaced);
/// assert_eq!(config.category, "Workloads");
/// ```
pub fn get_resource_config(resource_type: &str) -> Option<&ResourceConfig> {
    RESOURCE_CONFIGS.get(resource_type)
}

/// Checks if a resource type is namespaced with O(1) lookup complexity.
///
/// This is a convenience function for quickly determining resource scope without
/// retrieving the full configuration object.
///
/// # Arguments
/// * `resource_type` - The lowercase resource type name
///
/// # Returns
/// * `Some(true)` - Resource is namespaced
/// * `Some(false)` - Resource is cluster-wide
/// * `None` - Resource type is not supported
///
/// # Examples
/// ```rust,ignore
/// assert_eq!(is_resource_namespaced("pods"), Some(true));
/// assert_eq!(is_resource_namespaced("nodes"), Some(false));
/// assert_eq!(is_resource_namespaced("invalid"), None);
/// ```
pub fn is_resource_namespaced(resource_type: &str) -> Option<bool> {
    RESOURCE_CONFIGS.get(resource_type).map(|config| config.is_namespaced)
}

/// Returns all supported resource types.
///
/// This function provides a complete list of resource types that can be watched
/// by the system. Useful for validation, UI population, and debugging.
///
/// # Returns
/// A vector containing all supported resource type names
///
/// # Examples
/// ```rust,ignore
/// let types = get_supported_resource_types();
/// assert!(types.contains(&"pods"));
/// assert!(types.contains(&"deployments"));
/// assert!(types.len() >= 25); // We support many resource types
/// ```
pub fn get_supported_resource_types() -> Vec<&'static str> {
    RESOURCE_CONFIGS.keys().copied().collect()
}

/// Returns all resource types belonging to a specific category.
///
/// Categories provide logical grouping of related resource types for UI organization
/// and bulk operations. Common categories include "Workloads", "Storage", etc.
///
/// # Arguments
/// * `category` - The category name to filter by
///
/// # Returns
/// A vector of resource type names in the specified category
///
/// # Examples
/// ```rust,ignore
/// let workloads = get_resources_by_category("Workloads");
/// assert!(workloads.contains(&"pods"));
/// assert!(workloads.contains(&"deployments"));
///
/// let storage = get_resources_by_category("Storage");
/// assert!(storage.contains(&"persistentvolumes"));
/// ```
pub fn get_resources_by_category(category: &str) -> Vec<&'static str> {
    RESOURCE_CONFIGS
        .iter()
        .filter_map(|(resource_type, config)| {
            if config.category == category {
                Some(*resource_type)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_config_lookup() {
        // Test namespaced resources
        assert_eq!(is_resource_namespaced("pods"), Some(true));
        assert_eq!(is_resource_namespaced("deployments"), Some(true));
        assert_eq!(is_resource_namespaced("services"), Some(true));
        
        // Test cluster-wide resources
        assert_eq!(is_resource_namespaced("nodes"), Some(false));
        assert_eq!(is_resource_namespaced("namespaces"), Some(false));
        assert_eq!(is_resource_namespaced("persistentvolumes"), Some(false));
        
        // Test unsupported resource
        assert_eq!(is_resource_namespaced("nonexistent"), None);
    }

    #[test]
    fn test_resource_categories() {
        let workload_resources = get_resources_by_category("Workloads");
        assert!(workload_resources.contains(&"pods"));
        assert!(workload_resources.contains(&"deployments"));
        assert!(!workload_resources.is_empty());
        
        let storage_resources = get_resources_by_category("Storage");
        assert!(storage_resources.contains(&"persistentvolumes"));
        assert!(storage_resources.contains(&"persistentvolumeclaims"));
    }

    #[test]
    fn test_supported_resource_types() {
        let supported = get_supported_resource_types();
        assert!(supported.len() >= 25); // Should have at least 25 resource types
        assert!(supported.contains(&"pods"));
        assert!(supported.contains(&"nodes"));
        assert!(supported.contains(&"deployments"));
    }

    #[test]
    fn test_resource_config_details() {
        let pod_config = get_resource_config("pods").unwrap();
        assert_eq!(pod_config.is_namespaced, true);
        assert_eq!(pod_config.category, "Workloads");
        
        let node_config = get_resource_config("nodes").unwrap();
        assert_eq!(node_config.is_namespaced, false);
        assert_eq!(node_config.category, "Cluster Administration");
    }
}