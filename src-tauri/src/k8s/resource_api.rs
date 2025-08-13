use kube::{Api, Client};
use serde_json::Value;

/// Trait for handling Kubernetes resources generically
pub trait KubernetesResource: k8s_openapi::Resource + serde::Serialize + Send + Sync + 'static 
where
    Self: serde::de::DeserializeOwned + Clone + std::fmt::Debug,
{
    /// Create an API client for this resource type
    fn create_api(client: Client, namespace: Option<&str>) -> Api<Self>;
    
    /// Get the resource kind name
    fn kind_name() -> &'static str;
    
    /// Whether this resource is namespaced
    fn is_namespaced() -> bool;
}

/// Generic function to get a Kubernetes resource
pub async fn get_resource_generic<T>(
    client: Client,
    name: &str,
    namespace: Option<&str>
) -> Result<Value, String> 
where
    T: KubernetesResource,
{
    let api = T::create_api(client, namespace);
    let resource = api.get(name).await.map_err(|e| e.to_string())?;
    serde_json::to_value(resource).map_err(|e| e.to_string())
}

/// Generic function to delete a Kubernetes resource
pub async fn delete_resource_generic<T>(
    client: Client,
    name: &str,
    namespace: Option<&str>
) -> Result<(), String>
where
    T: KubernetesResource,
{
    use kube::api::DeleteParams;
    
    let api = T::create_api(client, namespace);
    api.delete(name, &DeleteParams::default())
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Generic function to scale a Kubernetes resource
pub async fn scale_resource_generic<T>(
    client: Client,
    name: &str,
    namespace: Option<&str>,
    replicas: i32
) -> Result<(), String>
where
    T: KubernetesResource,
{
    use kube::api::{Patch, PatchParams};
    use serde_json::json;
    
    let api = T::create_api(client, namespace);
    let scale_patch = json!({
        "spec": {
            "replicas": replicas
        }
    });
    
    let patch_params = PatchParams::default();
    let patch = Patch::Merge(&scale_patch);
    api.patch(name, &patch_params, &patch)
        .await
        .map_err(|e| format!("Failed to scale {}: {}", T::kind_name(), e))?;
    
    Ok(())
}

// Separate macros for namespaced and cluster-wide resources
macro_rules! impl_namespaced_k8s_resource {
    ($type:ty, $kind:literal) => {
        impl KubernetesResource for $type {
            fn create_api(client: Client, namespace: Option<&str>) -> Api<Self> {
                if let Some(ns) = namespace {
                    Api::namespaced(client, ns)
                } else {
                    Api::all(client)
                }
            }
            
            fn kind_name() -> &'static str {
                $kind
            }
            
            fn is_namespaced() -> bool {
                true
            }
        }
    };
}

macro_rules! impl_cluster_k8s_resource {
    ($type:ty, $kind:literal) => {
        impl KubernetesResource for $type {
            fn create_api(client: Client, _namespace: Option<&str>) -> Api<Self> {
                Api::all(client)
            }
            
            fn kind_name() -> &'static str {
                $kind
            }
            
            fn is_namespaced() -> bool {
                false
            }
        }
    };
}

// Namespaced resources
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::Pod, "Pod");
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::Service, "Service");
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::ConfigMap, "ConfigMap");
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::Secret, "Secret");
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::PersistentVolumeClaim, "PersistentVolumeClaim");
impl_namespaced_k8s_resource!(k8s_openapi::api::core::v1::ServiceAccount, "ServiceAccount");
impl_namespaced_k8s_resource!(k8s_openapi::api::apps::v1::Deployment, "Deployment");
impl_namespaced_k8s_resource!(k8s_openapi::api::apps::v1::StatefulSet, "StatefulSet");
impl_namespaced_k8s_resource!(k8s_openapi::api::apps::v1::DaemonSet, "DaemonSet");
impl_namespaced_k8s_resource!(k8s_openapi::api::apps::v1::ReplicaSet, "ReplicaSet");
impl_namespaced_k8s_resource!(k8s_openapi::api::networking::v1::Ingress, "Ingress");
impl_namespaced_k8s_resource!(k8s_openapi::api::networking::v1::NetworkPolicy, "NetworkPolicy");
impl_namespaced_k8s_resource!(k8s_openapi::api::rbac::v1::Role, "Role");
impl_namespaced_k8s_resource!(k8s_openapi::api::rbac::v1::RoleBinding, "RoleBinding");

// Cluster-wide resources
impl_cluster_k8s_resource!(k8s_openapi::api::core::v1::Namespace, "Namespace");
impl_cluster_k8s_resource!(k8s_openapi::api::core::v1::Node, "Node");
impl_cluster_k8s_resource!(k8s_openapi::api::core::v1::PersistentVolume, "PersistentVolume");
impl_cluster_k8s_resource!(k8s_openapi::api::networking::v1::IngressClass, "IngressClass");
impl_cluster_k8s_resource!(k8s_openapi::api::rbac::v1::ClusterRole, "ClusterRole");
impl_cluster_k8s_resource!(k8s_openapi::api::rbac::v1::ClusterRoleBinding, "ClusterRoleBinding");

/// Dispatch function to handle resources by kind string
pub async fn handle_resource_by_kind(
    kind: &str,
    operation: ResourceOperation,
    client: Client,
    name: &str,
    namespace: Option<&str>,
    params: Option<ResourceParams>
) -> Result<Option<Value>, String> {
    match kind {
        "Pod" => handle_resource_operation::<k8s_openapi::api::core::v1::Pod>(operation, client, name, namespace, params).await,
        "Service" => handle_resource_operation::<k8s_openapi::api::core::v1::Service>(operation, client, name, namespace, params).await,
        "ConfigMap" => handle_resource_operation::<k8s_openapi::api::core::v1::ConfigMap>(operation, client, name, namespace, params).await,
        "Secret" => handle_resource_operation::<k8s_openapi::api::core::v1::Secret>(operation, client, name, namespace, params).await,
        "Namespace" => handle_resource_operation::<k8s_openapi::api::core::v1::Namespace>(operation, client, name, namespace, params).await,
        "Node" => handle_resource_operation::<k8s_openapi::api::core::v1::Node>(operation, client, name, namespace, params).await,
        "PersistentVolume" => handle_resource_operation::<k8s_openapi::api::core::v1::PersistentVolume>(operation, client, name, namespace, params).await,
        "PersistentVolumeClaim" => handle_resource_operation::<k8s_openapi::api::core::v1::PersistentVolumeClaim>(operation, client, name, namespace, params).await,
        "Deployment" => handle_resource_operation::<k8s_openapi::api::apps::v1::Deployment>(operation, client, name, namespace, params).await,
        "StatefulSet" => handle_resource_operation::<k8s_openapi::api::apps::v1::StatefulSet>(operation, client, name, namespace, params).await,
        "DaemonSet" => handle_resource_operation::<k8s_openapi::api::apps::v1::DaemonSet>(operation, client, name, namespace, params).await,
        "ReplicaSet" => handle_resource_operation::<k8s_openapi::api::apps::v1::ReplicaSet>(operation, client, name, namespace, params).await,
        "Ingress" => handle_resource_operation::<k8s_openapi::api::networking::v1::Ingress>(operation, client, name, namespace, params).await,
        "Role" => handle_resource_operation::<k8s_openapi::api::rbac::v1::Role>(operation, client, name, namespace, params).await,
        "RoleBinding" => handle_resource_operation::<k8s_openapi::api::rbac::v1::RoleBinding>(operation, client, name, namespace, params).await,
        "ClusterRole" => handle_resource_operation::<k8s_openapi::api::rbac::v1::ClusterRole>(operation, client, name, namespace, params).await,
        "ClusterRoleBinding" => handle_resource_operation::<k8s_openapi::api::rbac::v1::ClusterRoleBinding>(operation, client, name, namespace, params).await,
        _ => Err(format!("Unsupported resource kind: {}", kind))
    }
}

/// Resource operation types
pub enum ResourceOperation {
    Get,
    Delete,
    Scale,
    Update,
}

/// Parameters for resource operations
pub struct ResourceParams {
    pub replicas: Option<i32>,
    pub yaml_content: Option<String>,
}

/// Handle a specific operation on a resource type
async fn handle_resource_operation<T>(
    operation: ResourceOperation,
    client: Client,
    name: &str,
    namespace: Option<&str>,
    params: Option<ResourceParams>
) -> Result<Option<Value>, String>
where
    T: KubernetesResource,
{
    match operation {
        ResourceOperation::Get => {
            let result = get_resource_generic::<T>(client, name, namespace).await?;
            Ok(Some(result))
        },
        ResourceOperation::Delete => {
            delete_resource_generic::<T>(client, name, namespace).await?;
            Ok(None)
        },
        ResourceOperation::Scale => {
            if let Some(params) = params {
                if let Some(replicas) = params.replicas {
                    scale_resource_generic::<T>(client, name, namespace, replicas).await?;
                } else {
                    return Err("Replicas parameter required for scale operation".to_string());
                }
            } else {
                return Err("Parameters required for scale operation".to_string());
            }
            Ok(None)
        },
        ResourceOperation::Update => {
            // TODO: Implement generic update operation
            Err("Update operation not yet implemented".to_string())
        }
    }
}