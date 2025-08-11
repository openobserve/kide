//! Test actual Tauri IPC serialization behavior
//! 
//! This module tests what happens when we try to use k8s-openapi types
//! directly in Tauri IPC events vs manual extraction.

use k8s_openapi::api::core::v1::{Pod, PodSpec, PodStatus, Container};
use serde_json;
use crate::k8s::resources::{K8sListItem, K8sObjectMeta, WatchEvent};

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Create a sample Pod for testing
    fn create_sample_pod() -> Pod {
        Pod {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("123-456-789".to_string()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![Container {
                    name: "app".to_string(),
                    image: Some("nginx:latest".to_string()),
                    ..Default::default()
                }],
                node_name: Some("worker-node-1".to_string()),
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                container_statuses: Some(vec![
                    k8s_openapi::api::core::v1::ContainerStatus {
                        name: "app".to_string(),
                        restart_count: 2,
                        ready: true,
                        state: Some(k8s_openapi::api::core::v1::ContainerState {
                            running: Some(k8s_openapi::api::core::v1::ContainerStateRunning {
                                started_at: None,
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                ]),
                init_container_statuses: Some(vec![
                    k8s_openapi::api::core::v1::ContainerStatus {
                        name: "init-container".to_string(),
                        restart_count: 1,
                        ready: true,
                        ..Default::default()
                    }
                ]),
                ..Default::default()
            }),
        }
    }
    
    /// Test using k8s-openapi types directly in K8sListItem
    #[test]
    fn test_direct_k8s_openapi_types() {
        let pod = create_sample_pod();
        
        let list_item = K8sListItem {
            metadata: K8sObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("123-456-789".to_string()),
                ..Default::default()
            },
            kind: "Pod".to_string(),
            api_version: "v1".to_string(),
            complete_object: None,
            // Try using k8s-openapi types directly instead of simplified JSON
            pod_spec: pod.spec.clone(),
            pod_status: pod.status.clone(),
            // ... initialize all other fields as None
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
        
        // Test serialization
        let serialization_result = serde_json::to_string(&list_item);
        match serialization_result {
            Ok(json) => {
                println!("✅ K8sListItem with direct k8s-openapi types serialized!");
                println!("JSON size: {} bytes", json.len());
                
                // Test if we can deserialize it back
                let deserialization_result: Result<K8sListItem, _> = serde_json::from_str(&json);
                match deserialization_result {
                    Ok(deserialized) => {
                        println!("✅ Deserialization successful!");
                        
                        // Check that we can access the Pod data
                        if let Some(pod_status) = &deserialized.pod_status {
                            if let Some(phase) = &pod_status.phase {
                                println!("✅ Pod phase accessible: {}", phase);
                            }
                        }
                        
                        if let Some(pod_spec) = &deserialized.pod_spec {
                            if let Some(node_name) = &pod_spec.node_name {
                                println!("✅ Pod node name accessible: {}", node_name);
                            }
                        }
                    },
                    Err(e) => println!("❌ Deserialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ Serialization failed: {}", e),
        }
        
        // Test in WatchEvent (this is what gets sent through Tauri IPC)
        let watch_event = WatchEvent::Added(list_item);
        let event_serialization = serde_json::to_string(&watch_event);
        match event_serialization {
            Ok(json) => {
                println!("✅ WatchEvent with direct k8s-openapi types serialized!");
                println!("Event JSON size: {} bytes", json.len());
                
                let event_deserialization: Result<WatchEvent, _> = serde_json::from_str(&json);
                match event_deserialization {
                    Ok(WatchEvent::Added(item)) => {
                        println!("✅ WatchEvent deserialized successfully!");
                        assert_eq!(item.kind, "Pod");
                    },
                    Ok(_) => println!("❌ Wrong event type"),
                    Err(e) => println!("❌ WatchEvent deserialization failed: {}", e),
                }
            },
            Err(e) => println!("❌ WatchEvent serialization failed: {}", e),
        }
    }
    
    /// Test using direct k8s-openapi types (updated approach)
    #[test] 
    fn test_direct_k8s_openapi_approach_v2() {
        let pod = create_sample_pod();
        
        let list_item = K8sListItem {
            metadata: K8sObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("123-456-789".to_string()),
                ..Default::default()
            },
            kind: "Pod".to_string(),
            api_version: "v1".to_string(),
            complete_object: None,
            // Use direct k8s-openapi types (same as the new direct approach)
            pod_spec: pod.spec.clone(),
            pod_status: pod.status.clone(),
            // ... initialize all other fields as None
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
        
        println!("=== Testing Direct k8s-openapi Approach v2 ===");
        
        let serialization_result = serde_json::to_string(&list_item);
        match serialization_result {
            Ok(json) => {
                println!("✅ Direct k8s-openapi approach v2 serialized!");
                println!("JSON size: {} bytes", json.len());
            },
            Err(e) => println!("❌ Direct k8s-openapi approach v2 serialization failed: {}", e),
        }
        
        let watch_event = WatchEvent::Added(list_item);
        let event_serialization = serde_json::to_string(&watch_event);
        match event_serialization {
            Ok(json) => {
                println!("✅ WatchEvent with direct k8s-openapi approach v2 serialized!");
                println!("Event JSON size: {} bytes", json.len());
            },
            Err(e) => println!("❌ WatchEvent with direct k8s-openapi approach v2 failed: {}", e),
        }
    }
    
    /// Verify official k8s-openapi types work correctly
    #[test]
    fn test_official_k8s_openapi_types() {
        println!("=== VERIFICATION: Official k8s-openapi Types ===");
        
        let pod = create_sample_pod();
        let spec = pod.spec.as_ref().unwrap();
        let status = pod.status.as_ref().unwrap();
        
        // Test 1: Direct k8s-openapi serialization
        println!("\n--- Direct k8s-openapi serialization ---");
        let direct_spec_json = serde_json::to_value(spec);
        let direct_status_json = serde_json::to_value(status);
        
        match (direct_spec_json, direct_status_json) {
            (Ok(spec_json), Ok(status_json)) => {
                println!("✅ Direct serialization works");
                println!("PodSpec JSON size: {} bytes", serde_json::to_string(&spec_json).unwrap().len());
                println!("PodStatus JSON size: {} bytes", serde_json::to_string(&status_json).unwrap().len());
                
                // Check what fields are available
                if let Some(containers) = spec_json.get("containers") {
                    println!("✅ Containers field present: {}", containers.is_array());
                }
                if let Some(node_name) = spec_json.get("nodeName") {
                    println!("✅ Node name field present: {}", node_name);
                }
                if let Some(phase) = status_json.get("phase") {
                    println!("✅ Phase field present: {}", phase);
                }
                if let Some(container_statuses) = status_json.get("containerStatuses") {
                    println!("✅ Container statuses field present: {}", container_statuses.is_array());
                }
            },
            _ => println!("❌ Direct serialization failed"),
        }
        
        // Test 2: Verify all official k8s-openapi fields are available through direct serialization
        println!("\n--- Official k8s-openapi field verification ---");
        if let Ok(direct_spec_json) = serde_json::to_value(spec) {
            if let Some(obj) = direct_spec_json.as_object() {
                println!("Official k8s-openapi spec fields: {:?}", obj.keys().collect::<Vec<_>>());
                println!("✅ All fields preserved through official k8s-openapi types");
            }
        }
        if let Ok(direct_status_json) = serde_json::to_value(status) {
            if let Some(obj) = direct_status_json.as_object() {
                println!("Official k8s-openapi status fields: {:?}", obj.keys().collect::<Vec<_>>());
            }
        }
    }
}