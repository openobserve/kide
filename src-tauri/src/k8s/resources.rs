//! Kubernetes resource type definitions and categorization.
//!
//! This module provides type-safe representations of Kubernetes resources and their metadata.
//! It includes both the resource definitions themselves and helper functions for organizing
//! them into logical categories for UI presentation.
//!
//! # Key Types
//!
//! - `K8sResource`: Metadata about a specific resource type
//! - `K8sResourceCategory`: Logical grouping of related resources
//! - `K8sListItem`: Unified representation of any Kubernetes object
//! - `WatchEvent`: Events emitted when resources change

use serde::{Deserialize, Serialize};

/// A logical grouping of related Kubernetes resource types.
///
/// Categories help organize the many Kubernetes resource types into manageable groups
/// for UI presentation and navigation. For example, "Workloads" contains pods,
/// deployments, and other compute resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sResourceCategory {
    /// Display name for this category (e.g., "Workloads", "Storage")
    pub name: String,
    /// List of resources belonging to this category
    pub resources: Vec<K8sResource>,
}

/// Metadata about a specific Kubernetes resource type.
///
/// This struct contains all the information needed to interact with a specific
/// type of Kubernetes resource, including its API details and scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sResource {
    /// Human-readable display name (e.g., "Pods", "Deployments")
    pub name: String,
    /// Kubernetes API version for this resource (e.g., "v1", "apps/v1")
    pub api_version: String,
    /// Kubernetes Kind for this resource (e.g., "Pod", "Deployment")
    pub kind: String,
    /// Whether this resource is namespaced (true) or cluster-wide (false)
    pub namespaced: bool,
    /// Brief description of what this resource type is used for
    pub description: String,
}

/// Reference to another Kubernetes object that owns this resource.
///
/// Owner references create a hierarchy of Kubernetes objects and enable
/// garbage collection when parent objects are deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sOwnerReference {
    /// API version of the owner object
    pub api_version: String,
    /// Kind of the owner object (e.g., "ReplicaSet", "Deployment")
    pub kind: String,
    /// Name of the owner object
    pub name: String,
    /// Unique identifier of the owner object
    pub uid: String,
    /// Whether this owner is the controller (primary owner)
    pub controller: Option<bool>,
    /// Whether deletion of this object is blocked by the owner
    pub block_owner_deletion: Option<bool>,
}

// Use the official k8s-openapi metadata type instead of our own custom struct
// This ensures we automatically get all fields without missing anything
pub use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta as K8sObjectMeta;

// Use official k8s-openapi types instead of custom struct definitions
pub use k8s_openapi::api::core::v1::{EndpointAddress, EndpointPort, EndpointSubset};

// We leverage the official k8s-openapi crate types instead of defining our own
// This ensures we stay in sync with the official Kubernetes API definitions

/// A unified representation of any Kubernetes object for display and processing.
///
/// This struct provides a consistent interface for working with different types of
/// Kubernetes objects without needing to know their specific types at compile time.
/// It includes both the common metadata and the official k8s-openapi type-specific
/// spec and status structures.
///
/// By using the official k8s-openapi types, we ensure complete compatibility with
/// the Kubernetes API and automatically get all fields without duplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sListItem {
    /// Common metadata present on all Kubernetes objects
    pub metadata: K8sObjectMeta,
    /// The Kind of this object (e.g., "Pod", "Service")
    pub kind: String,
    /// The API version of this object (e.g., "v1", "apps/v1")
    pub api_version: String,
    /// Complete object data for full YAML display and detailed inspection
    pub complete_object: Option<serde_json::Value>,
    
    // ================================
    // Official k8s-openapi spec types
    // ================================
    
    // Core v1 specs - using direct k8s-openapi types for complete compatibility
    pub pod_spec: Option<k8s_openapi::api::core::v1::PodSpec>,
    pub service_spec: Option<k8s_openapi::api::core::v1::ServiceSpec>,
    pub config_map_spec: Option<serde_json::Value>, // ConfigMap doesn't have a separate spec
    pub secret_spec: Option<serde_json::Value>, // Secret doesn't have a separate spec
    pub namespace_spec: Option<k8s_openapi::api::core::v1::NamespaceSpec>,
    pub node_spec: Option<k8s_openapi::api::core::v1::NodeSpec>,
    pub persistent_volume_spec: Option<k8s_openapi::api::core::v1::PersistentVolumeSpec>,
    pub persistent_volume_claim_spec: Option<k8s_openapi::api::core::v1::PersistentVolumeClaimSpec>,
    pub endpoints_spec: Option<serde_json::Value>, // Endpoints doesn't have a separate spec
    
    // Apps v1 specs
    pub deployment_spec: Option<k8s_openapi::api::apps::v1::DeploymentSpec>,
    pub replica_set_spec: Option<k8s_openapi::api::apps::v1::ReplicaSetSpec>,
    pub stateful_set_spec: Option<k8s_openapi::api::apps::v1::StatefulSetSpec>,
    pub daemon_set_spec: Option<k8s_openapi::api::apps::v1::DaemonSetSpec>,
    
    // Batch v1 specs
    pub job_spec: Option<k8s_openapi::api::batch::v1::JobSpec>,
    pub cron_job_spec: Option<k8s_openapi::api::batch::v1::CronJobSpec>,
    
    // Networking specs
    pub ingress_spec: Option<k8s_openapi::api::networking::v1::IngressSpec>,
    pub network_policy_spec: Option<k8s_openapi::api::networking::v1::NetworkPolicySpec>,
    pub endpoint_slice: Option<k8s_openapi::api::discovery::v1::EndpointSlice>, // EndpointSlice object (no separate spec)
    
    // Storage specs
    pub storage_class_spec: Option<k8s_openapi::api::storage::v1::StorageClass>,
    
    // RBAC specs
    pub role_spec: Option<serde_json::Value>, // Role doesn't have a separate spec (uses rules directly)
    pub role_binding_spec: Option<serde_json::Value>, // RoleBinding doesn't have a separate spec
    pub cluster_role_spec: Option<serde_json::Value>, // ClusterRole doesn't have a separate spec
    pub cluster_role_binding_spec: Option<serde_json::Value>, // ClusterRoleBinding doesn't have a separate spec
    pub service_account_spec: Option<serde_json::Value>, // ServiceAccount doesn't have a separate spec
    
    // Policy specs
    pub pod_disruption_budget_spec: Option<k8s_openapi::api::policy::v1::PodDisruptionBudgetSpec>,
    
    // Autoscaling specs
    pub horizontal_pod_autoscaler_spec: Option<k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscalerSpec>,
    
    // ================================
    // Official k8s-openapi status types
    // ================================
    
    // Core v1 statuses - using direct k8s-openapi types for complete compatibility
    pub pod_status: Option<k8s_openapi::api::core::v1::PodStatus>,
    pub service_status: Option<k8s_openapi::api::core::v1::ServiceStatus>,
    pub namespace_status: Option<k8s_openapi::api::core::v1::NamespaceStatus>,
    pub node_status: Option<k8s_openapi::api::core::v1::NodeStatus>,
    pub persistent_volume_status: Option<k8s_openapi::api::core::v1::PersistentVolumeStatus>,
    pub persistent_volume_claim_status: Option<k8s_openapi::api::core::v1::PersistentVolumeClaimStatus>,
    
    // Apps v1 statuses
    pub deployment_status: Option<k8s_openapi::api::apps::v1::DeploymentStatus>,
    pub replica_set_status: Option<k8s_openapi::api::apps::v1::ReplicaSetStatus>,
    pub stateful_set_status: Option<k8s_openapi::api::apps::v1::StatefulSetStatus>,
    pub daemon_set_status: Option<k8s_openapi::api::apps::v1::DaemonSetStatus>,
    
    // Batch v1 statuses
    pub job_status: Option<k8s_openapi::api::batch::v1::JobStatus>,
    pub cron_job_status: Option<k8s_openapi::api::batch::v1::CronJobStatus>,
    
    // Networking statuses
    pub ingress_status: Option<k8s_openapi::api::networking::v1::IngressStatus>,
    
    // Policy statuses
    pub pod_disruption_budget_status: Option<k8s_openapi::api::policy::v1::PodDisruptionBudgetStatus>,
    
    // Autoscaling statuses
    pub horizontal_pod_autoscaler_status: Option<k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscalerStatus>,
    
    // ================================
    // Endpoints specific fields (legacy)
    // ================================
    
    // Endpoints specific fields
    /// Subset information for Endpoints
    pub subsets: Option<Vec<EndpointSubset>>,
}

/// Events emitted when Kubernetes resources change.
///
/// These events represent the different types of changes that can occur to
/// Kubernetes objects, allowing clients to maintain up-to-date views of cluster state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchEvent {
    /// A new resource was created or discovered
    Added(K8sListItem),
    /// An existing resource was updated
    Modified(K8sListItem),
    /// A resource was deleted
    Deleted(K8sListItem),
    /// Initial watch synchronization is complete (no more items)
    InitialSyncComplete,
}

/// Returns all supported Kubernetes resource types organized by category.
///
/// This function provides the complete catalog of supported resource types,
/// grouped into logical categories for UI presentation. Each category contains
/// metadata about the resources including their API versions, scope, and descriptions.
///
/// # Returns
/// A vector of `K8sResourceCategory` containing all supported resource types
///
/// # Categories Included
/// - **Commonly used**: Most frequently used resources across all categories
/// - **Workloads**: Pods, Deployments, StatefulSets, etc.
/// - **Services & Networking**: Services, Ingresses, NetworkPolicies, etc.
/// - **Configuration**: ConfigMaps, Secrets, ResourceQuotas, etc.
/// - **Storage**: PersistentVolumes, StorageClasses, etc.
/// - **Cluster Administration**: Namespaces, Nodes, ServiceAccounts, etc.
/// - **Security & Access Control**: Roles, RoleBindings, etc.
/// - **Scaling**: HorizontalPodAutoscalers, etc.
/// - **Custom Resources**: CustomResourceDefinitions, APIServices, etc.
///
/// # Examples
/// ```rust,ignore
/// let categories = get_resource_categories();
/// let workloads = categories.iter()
///     .find(|cat| cat.name == "Workloads")
///     .unwrap();
/// assert!(workloads.resources.iter().any(|r| r.kind == "Pod"));
/// ```
pub fn get_resource_categories() -> Vec<K8sResourceCategory> {
    vec![
        K8sResourceCategory {
            name: "Commonly used".to_string(),
            resources: vec![
                K8sResource {
                    name: "Pods".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Pod".to_string(),
                    namespaced: true,
                    description: "Smallest deployable units in Kubernetes".to_string(),
                },
                K8sResource {
                    name: "Deployments".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "Deployment".to_string(),
                    namespaced: true,
                    description: "Manages ReplicaSets and provides declarative updates to Pods".to_string(),
                },
                K8sResource {
                    name: "StatefulSets".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "StatefulSet".to_string(),
                    namespaced: true,
                    description: "Manages stateful applications with persistent storage".to_string(),
                },
                K8sResource {
                    name: "CronJobs".to_string(),
                    api_version: "batch/v1".to_string(),
                    kind: "CronJob".to_string(),
                    namespaced: true,
                    description: "Creates Jobs on a schedule".to_string(),
                },
                K8sResource {
                    name: "Services".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Service".to_string(),
                    namespaced: true,
                    description: "Exposes applications running on Pods".to_string(),
                },
                K8sResource {
                    name: "Ingresses".to_string(),
                    api_version: "networking.k8s.io/v1".to_string(),
                    kind: "Ingress".to_string(),
                    namespaced: true,
                    description: "Manages external access to services".to_string(),
                },
                K8sResource {
                    name: "ConfigMaps".to_string(),
                    api_version: "v1".to_string(),
                    kind: "ConfigMap".to_string(),
                    namespaced: true,
                    description: "Stores non-confidential configuration data".to_string(),
                },
                K8sResource {
                    name: "Secrets".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Secret".to_string(),
                    namespaced: true,
                    description: "Stores sensitive data like passwords and keys".to_string(),
                },
                K8sResource {
                    name: "PersistentVolumes".to_string(),
                    api_version: "v1".to_string(),
                    kind: "PersistentVolume".to_string(),
                    namespaced: false,
                    description: "Cluster-level storage resources".to_string(),
                },
                K8sResource {
                    name: "PersistentVolumeClaims".to_string(),
                    api_version: "v1".to_string(),
                    kind: "PersistentVolumeClaim".to_string(),
                    namespaced: true,
                    description: "User requests for storage".to_string(),
                },
                K8sResource {
                    name: "StorageClasses".to_string(),
                    api_version: "storage.k8s.io/v1".to_string(),
                    kind: "StorageClass".to_string(),
                    namespaced: false,
                    description: "Dynamic volume provisioning".to_string(),
                },
                K8sResource {
                    name: "Namespaces".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Namespace".to_string(),
                    namespaced: false,
                    description: "Virtual clusters within a cluster".to_string(),
                },
                K8sResource {
                    name: "Nodes".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Node".to_string(),
                    namespaced: false,
                    description: "Worker machines in the cluster".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Workloads".to_string(),
            resources: vec![
                K8sResource {
                    name: "Pods".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Pod".to_string(),
                    namespaced: true,
                    description: "Smallest deployable units in Kubernetes".to_string(),
                },
                K8sResource {
                    name: "Deployments".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "Deployment".to_string(),
                    namespaced: true,
                    description: "Manages ReplicaSets and provides declarative updates to Pods".to_string(),
                },
                K8sResource {
                    name: "StatefulSets".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "StatefulSet".to_string(),
                    namespaced: true,
                    description: "Manages stateful applications with persistent storage".to_string(),
                },
                K8sResource {
                    name: "DaemonSets".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "DaemonSet".to_string(),
                    namespaced: true,
                    description: "Ensures all nodes run a copy of a Pod".to_string(),
                },
                K8sResource {
                    name: "ReplicaSets".to_string(),
                    api_version: "apps/v1".to_string(),
                    kind: "ReplicaSet".to_string(),
                    namespaced: true,
                    description: "Maintains a stable set of replica Pods".to_string(),
                },
                K8sResource {
                    name: "Jobs".to_string(),
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    namespaced: true,
                    description: "Creates one or more Pods and ensures successful completion".to_string(),
                },
                K8sResource {
                    name: "CronJobs".to_string(),
                    api_version: "batch/v1".to_string(),
                    kind: "CronJob".to_string(),
                    namespaced: true,
                    description: "Creates Jobs on a schedule".to_string(),
                },
                K8sResource {
                    name: "ReplicationControllers".to_string(),
                    api_version: "v1".to_string(),
                    kind: "ReplicationController".to_string(),
                    namespaced: true,
                    description: "Legacy replica management (deprecated)".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Services & Networking".to_string(),
            resources: vec![
                K8sResource {
                    name: "Services".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Service".to_string(),
                    namespaced: true,
                    description: "Exposes applications running on Pods".to_string(),
                },
                K8sResource {
                    name: "Ingresses".to_string(),
                    api_version: "networking.k8s.io/v1".to_string(),
                    kind: "Ingress".to_string(),
                    namespaced: true,
                    description: "Manages external access to services".to_string(),
                },
                K8sResource {
                    name: "IngressClasses".to_string(),
                    api_version: "networking.k8s.io/v1".to_string(),
                    kind: "IngressClass".to_string(),
                    namespaced: false,
                    description: "Defines ingress controller configuration".to_string(),
                },
                K8sResource {
                    name: "NetworkPolicies".to_string(),
                    api_version: "networking.k8s.io/v1".to_string(),
                    kind: "NetworkPolicy".to_string(),
                    namespaced: true,
                    description: "Controls traffic flow between Pods".to_string(),
                },
                K8sResource {
                    name: "EndpointSlices".to_string(),
                    api_version: "discovery.k8s.io/v1".to_string(),
                    kind: "EndpointSlice".to_string(),
                    namespaced: true,
                    description: "Scalable tracking of network endpoints".to_string(),
                },
                K8sResource {
                    name: "Endpoints".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Endpoints".to_string(),
                    namespaced: true,
                    description: "Legacy endpoint tracking".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Configuration".to_string(),
            resources: vec![
                K8sResource {
                    name: "ConfigMaps".to_string(),
                    api_version: "v1".to_string(),
                    kind: "ConfigMap".to_string(),
                    namespaced: true,
                    description: "Stores non-confidential configuration data".to_string(),
                },
                K8sResource {
                    name: "Secrets".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Secret".to_string(),
                    namespaced: true,
                    description: "Stores sensitive data like passwords and keys".to_string(),
                },
                K8sResource {
                    name: "ResourceQuotas".to_string(),
                    api_version: "v1".to_string(),
                    kind: "ResourceQuota".to_string(),
                    namespaced: true,
                    description: "Resource usage limits per namespace".to_string(),
                },
                K8sResource {
                    name: "LimitRanges".to_string(),
                    api_version: "v1".to_string(),
                    kind: "LimitRange".to_string(),
                    namespaced: true,
                    description: "Min/max resource constraints for pods and containers".to_string(),
                },
                K8sResource {
                    name: "PodDisruptionBudgets".to_string(),
                    api_version: "policy/v1".to_string(),
                    kind: "PodDisruptionBudget".to_string(),
                    namespaced: true,
                    description: "Limits on voluntary disruptions during maintenance".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Storage".to_string(),
            resources: vec![
                K8sResource {
                    name: "PersistentVolumes".to_string(),
                    api_version: "v1".to_string(),
                    kind: "PersistentVolume".to_string(),
                    namespaced: false,
                    description: "Cluster-level storage resources".to_string(),
                },
                K8sResource {
                    name: "PersistentVolumeClaims".to_string(),
                    api_version: "v1".to_string(),
                    kind: "PersistentVolumeClaim".to_string(),
                    namespaced: true,
                    description: "User requests for storage".to_string(),
                },
                K8sResource {
                    name: "StorageClasses".to_string(),
                    api_version: "storage.k8s.io/v1".to_string(),
                    kind: "StorageClass".to_string(),
                    namespaced: false,
                    description: "Dynamic volume provisioning".to_string(),
                },
                K8sResource {
                    name: "CSIDrivers".to_string(),
                    api_version: "storage.k8s.io/v1".to_string(),
                    kind: "CSIDriver".to_string(),
                    namespaced: false,
                    description: "Container Storage Interface drivers".to_string(),
                },
                K8sResource {
                    name: "CSINodes".to_string(),
                    api_version: "storage.k8s.io/v1".to_string(),
                    kind: "CSINode".to_string(),
                    namespaced: false,
                    description: "Node-specific storage information".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Cluster Administration".to_string(),
            resources: vec![
                K8sResource {
                    name: "Namespaces".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Namespace".to_string(),
                    namespaced: false,
                    description: "Virtual clusters within a cluster".to_string(),
                },
                K8sResource {
                    name: "Nodes".to_string(),
                    api_version: "v1".to_string(),
                    kind: "Node".to_string(),
                    namespaced: false,
                    description: "Worker machines in the cluster".to_string(),
                },
                K8sResource {
                    name: "PriorityClasses".to_string(),
                    api_version: "scheduling.k8s.io/v1".to_string(),
                    kind: "PriorityClass".to_string(),
                    namespaced: false,
                    description: "Pod scheduling priorities".to_string(),
                },
                K8sResource {
                    name: "RuntimeClasses".to_string(),
                    api_version: "node.k8s.io/v1".to_string(),
                    kind: "RuntimeClass".to_string(),
                    namespaced: false,
                    description: "Container runtime configurations".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Security & Access Control".to_string(),
            resources: vec![
                K8sResource {
                    name: "ServiceAccounts".to_string(),
                    api_version: "v1".to_string(),
                    kind: "ServiceAccount".to_string(),
                    namespaced: true,
                    description: "Identities for Pods and applications".to_string(),
                },
                K8sResource {
                    name: "Roles".to_string(),
                    api_version: "rbac.authorization.k8s.io/v1".to_string(),
                    kind: "Role".to_string(),
                    namespaced: true,
                    description: "Namespace-scoped permissions".to_string(),
                },
                K8sResource {
                    name: "ClusterRoles".to_string(),
                    api_version: "rbac.authorization.k8s.io/v1".to_string(),
                    kind: "ClusterRole".to_string(),
                    namespaced: false,
                    description: "Cluster-wide permissions".to_string(),
                },
                K8sResource {
                    name: "RoleBindings".to_string(),
                    api_version: "rbac.authorization.k8s.io/v1".to_string(),
                    kind: "RoleBinding".to_string(),
                    namespaced: true,
                    description: "Binds roles to users/groups".to_string(),
                },
                K8sResource {
                    name: "ClusterRoleBindings".to_string(),
                    api_version: "rbac.authorization.k8s.io/v1".to_string(),
                    kind: "ClusterRoleBinding".to_string(),
                    namespaced: false,
                    description: "Binds cluster roles to users/groups".to_string(),
                },
                K8sResource {
                    name: "CertificateSigningRequests".to_string(),
                    api_version: "certificates.k8s.io/v1".to_string(),
                    kind: "CertificateSigningRequest".to_string(),
                    namespaced: false,
                    description: "Certificate approval requests".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Scaling".to_string(),
            resources: vec![
                K8sResource {
                    name: "HorizontalPodAutoscalers".to_string(),
                    api_version: "autoscaling/v2".to_string(),
                    kind: "HorizontalPodAutoscaler".to_string(),
                    namespaced: true,
                    description: "Automatic Pod scaling".to_string(),
                },
            ],
        },
        K8sResourceCategory {
            name: "Custom Resources".to_string(),
            resources: vec![
                K8sResource {
                    name: "CustomResourceDefinitions".to_string(),
                    api_version: "apiextensions.k8s.io/v1".to_string(),
                    kind: "CustomResourceDefinition".to_string(),
                    namespaced: false,
                    description: "Extends Kubernetes API".to_string(),
                },
                K8sResource {
                    name: "APIServices".to_string(),
                    api_version: "apiregistration.k8s.io/v1".to_string(),
                    kind: "APIService".to_string(),
                    namespaced: false,
                    description: "Registers API extensions".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resource_categories() {
        let categories = get_resource_categories();
        
        assert_eq!(categories.len(), 9);
        
        // Test first category
        let commonly_used = &categories[0];
        assert_eq!(commonly_used.name, "Commonly used");
        assert_eq!(commonly_used.resources.len(), 13);
        
        // Test that we have both namespaced and cluster-wide resources
        let has_namespaced = categories.iter()
            .flat_map(|c| &c.resources)
            .any(|r| r.namespaced);
        let has_cluster_wide = categories.iter()
            .flat_map(|c| &c.resources)
            .any(|r| !r.namespaced);
        
        assert!(has_namespaced, "Should have namespaced resources");
        assert!(has_cluster_wide, "Should have cluster-wide resources");
    }

    #[test]
    fn test_resource_serialization() {
        let resource = K8sResource {
            name: "TestResource".to_string(),
            api_version: "v1".to_string(),
            kind: "Test".to_string(),
            namespaced: true,
            description: "Test resource".to_string(),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: K8sResource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.name, deserialized.name);
        assert_eq!(resource.api_version, deserialized.api_version);
        assert_eq!(resource.namespaced, deserialized.namespaced);
    }

    #[test]
    fn test_watch_event_serialization() {
        let metadata = K8sObjectMeta {
            name: Some("test-pod".to_string()),
            namespace: Some("default".to_string()),
            uid: Some("123-456-789".to_string()),
            creation_timestamp: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(k8s_openapi::chrono::Utc::now())),
            generation: Some(1),
            resource_version: Some("123456".to_string()),
            labels: None,
            annotations: None,
            owner_references: None,
            deletion_grace_period_seconds: None,
            deletion_timestamp: None,
            finalizers: None,
            generate_name: None,
            managed_fields: None,
            self_link: None,
        };

        let item = K8sListItem {
            metadata,
            kind: "Pod".to_string(),
            api_version: "v1".to_string(),
            complete_object: None,
            // Initialize all official k8s-openapi spec/status fields as None for test
            pod_spec: None,
            service_spec: None,
            config_map_spec: None,
            secret_spec: None,
            namespace_spec: None,
            node_spec: None,
            persistent_volume_spec: None,
            persistent_volume_claim_spec: None,
            endpoints_spec: None,
            deployment_spec: None,
            replica_set_spec: None,
            stateful_set_spec: None,
            daemon_set_spec: None,
            job_spec: None,
            cron_job_spec: None,
            ingress_spec: None,
            network_policy_spec: None,
            endpoint_slice: None,
            storage_class_spec: None,
            role_spec: None,
            role_binding_spec: None,
            cluster_role_spec: None,
            cluster_role_binding_spec: None,
            service_account_spec: None,
            pod_disruption_budget_spec: None,
            horizontal_pod_autoscaler_spec: None,
            pod_status: Some(k8s_openapi::api::core::v1::PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            service_status: None,
            namespace_status: None,
            node_status: None,
            persistent_volume_status: None,
            persistent_volume_claim_status: None,
            deployment_status: None,
            replica_set_status: None,
            stateful_set_status: None,
            daemon_set_status: None,
            job_status: None,
            cron_job_status: None,
            ingress_status: None,
            pod_disruption_budget_status: None,
            horizontal_pod_autoscaler_status: None,
            subsets: None,
        };

        let event = WatchEvent::Added(item);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WatchEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            WatchEvent::Added(item) => {
                assert_eq!(item.metadata.name, Some("test-pod".to_string()));
                assert_eq!(item.kind, "Pod");
                assert_eq!(item.pod_status.as_ref().unwrap().phase, Some("Running".to_string()));
            }
            _ => panic!("Expected Added event"),
        }
    }

    #[test]
    fn test_k8s_object_meta_with_labels() {
        let mut labels = std::collections::BTreeMap::new();
        labels.insert("app".to_string(), "test".to_string());
        labels.insert("version".to_string(), "1.0".to_string());

        let metadata = K8sObjectMeta {
            name: Some("test-resource".to_string()),
            namespace: Some("test-ns".to_string()),
            uid: Some("test-uid".to_string()),
            creation_timestamp: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                k8s_openapi::chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&k8s_openapi::chrono::Utc)
            )),
            generation: Some(2),
            resource_version: Some("789012".to_string()),
            labels: Some(labels.clone()),
            annotations: None,
            owner_references: None,
            deletion_grace_period_seconds: None,
            deletion_timestamp: None,
            finalizers: None,
            generate_name: None,
            managed_fields: None,
            self_link: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: K8sObjectMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.labels.unwrap(), labels);
    }
}