//! Test serialization behavior of k8s-openapi types
//! 
//! This module investigates why complex k8s-openapi types fail to serialize
//! through Tauri IPC and explores potential solutions.

use k8s_openapi::api::core::v1::{Pod, PodSpec, PodStatus, Container};
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test if a minimal PodSpec can serialize/deserialize
    #[test]
    fn test_minimal_pod_spec_serialization() {
        // Create a minimal PodSpec
        let pod_spec = PodSpec {
            containers: vec![Container {
                name: "test-container".to_string(),
                image: Some("nginx".to_string()),
                ..Default::default()
            }],
            node_name: Some("test-node".to_string()),
            ..Default::default()
        };
        
        // Try to serialize to JSON
        let json_result = serde_json::to_string(&pod_spec);
        match json_result {
            Ok(json) => {
                println!("✅ PodSpec serialized successfully!");
                println!("JSON length: {} bytes", json.len());
                
                // Try to deserialize back
                let deserialize_result: Result<PodSpec, _> = serde_json::from_str(&json);
                match deserialize_result {
                    Ok(_) => println!("✅ PodSpec deserialized successfully!"),
                    Err(e) => println!("❌ PodSpec deserialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ PodSpec serialization failed: {}", e),
        }
    }
    
    /// Test if a minimal PodStatus can serialize/deserialize
    #[test] 
    fn test_minimal_pod_status_serialization() {
        let pod_status = PodStatus {
            phase: Some("Running".to_string()),
            ..Default::default()
        };
        
        let json_result = serde_json::to_string(&pod_status);
        match json_result {
            Ok(json) => {
                println!("✅ PodStatus serialized successfully!");
                println!("JSON length: {} bytes", json.len());
                
                let deserialize_result: Result<PodStatus, _> = serde_json::from_str(&json);
                match deserialize_result {
                    Ok(_) => println!("✅ PodStatus deserialized successfully!"),
                    Err(e) => println!("❌ PodStatus deserialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ PodStatus serialization failed: {}", e),
        }
    }
    
    /// Test if a complete Pod can serialize/deserialize
    #[test]
    fn test_complete_pod_serialization() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "app".to_string(),
                    image: Some("nginx".to_string()),
                    ..Default::default()
                }],
                node_name: Some("test-node".to_string()),
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
        };
        
        let json_result = serde_json::to_string(&pod);
        match json_result {
            Ok(json) => {
                println!("✅ Complete Pod serialized successfully!");
                println!("JSON length: {} bytes", json.len());
                println!("Sample JSON: {}", &json[..std::cmp::min(200, json.len())]);
                
                let deserialize_result: Result<Pod, _> = serde_json::from_str(&json);
                match deserialize_result {
                    Ok(_) => println!("✅ Complete Pod deserialized successfully!"),
                    Err(e) => println!("❌ Complete Pod deserialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ Complete Pod serialization failed: {}", e),
        }
    }
    
    /// Test if Tauri IPC events can handle k8s-openapi types
    #[test]
    fn test_tauri_ipc_compatibility() {
        use crate::k8s::resources::{K8sListItem, K8sObjectMeta};
        
        // Create a K8sListItem with k8s-openapi PodSpec (instead of simplified JSON)
        let pod_spec = PodSpec {
            containers: vec![Container {
                name: "test-container".to_string(),
                image: Some("nginx".to_string()),
                ..Default::default()
            }],
            node_name: Some("test-node".to_string()),
            ..Default::default()
        };
        
        // Convert to JSON Value to simulate what would be sent through Tauri IPC
        match serde_json::to_value(&pod_spec) {
            Ok(_) => {
                println!("✅ PodSpec converted to JSON Value successfully!");
                
                // Now test if this can be included in a K8sListItem and serialized
                let list_item = K8sListItem {
                    metadata: K8sObjectMeta::default(),
                    kind: "Pod".to_string(),
                    api_version: "v1".to_string(),
                    complete_object: None,
                    pod_spec: Some(pod_spec.clone()), // Using direct k8s-openapi types
                    // ... initialize other fields as None
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
                    pod_status: None,
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
                
                match serde_json::to_string(&list_item) {
                    Ok(_) => println!("✅ K8sListItem with k8s-openapi PodSpec serialized successfully!"),
                    Err(e) => println!("❌ K8sListItem serialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ PodSpec to JSON Value conversion failed: {}", e),
        }
    }
}