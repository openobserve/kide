//! Tests for Pod data serialization and field extraction
//!
//! These tests ensure that Pod data flows correctly from backend to frontend
//! and catches serialization issues that could break Pod column display.

use crate::k8s::resources::K8sListItem;
use k8s_openapi::api::core::v1::Pod;
use serde_json::json;

/// Test that Pod objects can be converted to K8sListItem with proper field extraction
#[tokio::test]
async fn test_pod_conversion_and_serialization() {
    // Create a realistic Pod JSON (simplified version of real Pod data)
    let pod_json = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "test-pod",
            "namespace": "default",
            "uid": "12345"
        },
        "spec": {
            "nodeName": "test-node",
            "containers": [
                {
                    "name": "main-container",
                    "image": "nginx:latest"
                }
            ]
        },
        "status": {
            "phase": "Running",
            "containerStatuses": [
                {
                    "name": "main-container",
                    "restartCount": 2,
                    "ready": true,
                    "state": {
                        "running": {
                            "startedAt": "2023-01-01T00:00:00Z"
                        }
                    }
                }
            ],
            "initContainerStatuses": [
                {
                    "name": "init-container",
                    "restartCount": 1,
                    "ready": true
                }
            ]
        }
    });

    // Convert the JSON to a Pod object first (mimicking what happens in convert_to_list_item)
    let pod: Pod = serde_json::from_value(pod_json.clone()).expect("Should deserialize Pod");
    
    // Create K8sListItem and extract Pod fields manually (like our fix)
    let mut list_item = K8sListItem::default();
    list_item.kind = "Pod".to_string();
    list_item.api_version = "v1".to_string();
    
    // Use direct k8s-openapi types for complete compatibility
    list_item.pod_spec = pod.spec.clone();
    list_item.pod_status = pod.status.clone();

    // Test 1: Verify Pod fields are extracted
    assert!(list_item.pod_spec.is_some(), "Pod spec should be extracted");
    assert!(list_item.pod_status.is_some(), "Pod status should be extracted");
    
    // Test 2: Verify essential data is present
    let pod_spec = list_item.pod_spec.as_ref().unwrap();
    let pod_status = list_item.pod_status.as_ref().unwrap();
    
    assert_eq!(pod_spec.node_name.as_deref(), Some("test-node"), "Node name should be present");
    assert_eq!(pod_status.phase.as_deref(), Some("Running"), "Phase should be present");
    
    // Test 3: Verify container restart counts (critical for Restarts column)
    let container_statuses = pod_status.container_statuses.as_ref().unwrap();
    assert_eq!(container_statuses[0].restart_count, 2, "Main container restart count should be present");
    
    let init_container_statuses = pod_status.init_container_statuses.as_ref().unwrap();
    assert_eq!(init_container_statuses[0].restart_count, 1, "Init container restart count should be present");
    
    // Test 4: CRITICAL - Verify the K8sListItem can serialize to JSON (this was the root issue)
    let serialized = serde_json::to_string(&list_item).expect("K8sListItem should serialize to JSON");
    
    // Test 5: Verify camelCase field names in serialized JSON (critical for frontend)
    assert!(serialized.contains("podSpec"), "Serialized JSON should contain camelCase 'podSpec'");
    assert!(serialized.contains("podStatus"), "Serialized JSON should contain camelCase 'podStatus'");
    assert!(serialized.contains("nodeName"), "Serialized JSON should contain 'nodeName'");
    assert!(serialized.contains("restartCount"), "Serialized JSON should contain 'restartCount'");
    
    // Test 6: Verify the serialized JSON can be deserialized back (round-trip test)
    let deserialized: serde_json::Value = serde_json::from_str(&serialized).expect("Should deserialize back to JSON");
    
    assert_eq!(deserialized["podSpec"]["nodeName"], "test-node");
    assert_eq!(deserialized["podStatus"]["phase"], "Running");
    assert_eq!(deserialized["podStatus"]["containerStatuses"][0]["restartCount"], 2);
}

/// Test that missing Pod fields don't cause panics
#[tokio::test]
async fn test_pod_with_missing_fields() {
    let pod_json = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "minimal-pod",
            "namespace": "default",
            "uid": "12346"
        },
        "spec": {},
        "status": {}
    });

    let pod: Pod = serde_json::from_value(pod_json).expect("Should deserialize minimal Pod");
    
    let mut list_item = K8sListItem::default();
    list_item.kind = "Pod".to_string();
    
    // Use direct k8s-openapi types for complete compatibility
    list_item.pod_spec = pod.spec.clone();
    list_item.pod_status = pod.status.clone();

    // Should not panic and should serialize successfully
    let serialized = serde_json::to_string(&list_item).expect("Should serialize even with missing fields");
    assert!(serialized.contains("podSpec"));
    assert!(serialized.contains("podStatus"));
}

impl Default for K8sListItem {
    fn default() -> Self {
        Self {
            metadata: Default::default(),
            kind: String::new(),
            api_version: String::new(),
            complete_object: None,
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
        }
    }
}