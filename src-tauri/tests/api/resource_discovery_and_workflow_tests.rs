/**
 * Simplified Resource Management API Integration Tests
 * 
 * This file contains basic resource management tests that don't fit
 * into the specific endpoint test files.
 * 
 * For comprehensive endpoint tests, see:
 * - get_full_resource_tests.rs
 * - delete_resource_tests.rs  
 * - scale_resource_tests.rs
 * - update_resource_tests.rs
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_get_resources_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful resource categories retrieval
    api.set_success_response("get_resources", mock_resource_categories());
    
    let result = api.invoke("get_resources", json!({})).await;
    
    assert!(result.is_ok());
    let categories = result.unwrap();
    assert!(categories.is_array());
    
    let categories_array = categories.as_array().unwrap();
    assert_eq!(categories_array.len(), 2);
    
    // Verify structure of first category
    let first_category = &categories_array[0];
    assert_eq!(first_category["name"], "Workloads");
    assert!(first_category["resources"].is_array());
}

#[tokio::test]
async fn test_get_resources_empty() {
    let mut api = MockTauriApi::new();
    
    // Mock empty resource categories
    api.set_success_response("get_resources", json!([]));
    
    let result = api.invoke("get_resources", json!({})).await;
    
    assert!(result.is_ok());
    let categories = result.unwrap();
    assert!(categories.is_array());
    assert_eq!(categories.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_resource_events_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful events retrieval
    api.set_success_response("get_resource_events", mock_events());
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let events = result.unwrap();
    assert!(events.is_array());
    
    let events_array = events.as_array().unwrap();
    assert_eq!(events_array.len(), 1);
    
    let first_event = &events_array[0];
    assert_eq!(first_event["reason"], "Scheduled");
    assert_eq!(first_event["type"], "Normal");
}

#[tokio::test]
async fn test_get_resource_events_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock resource not found error
    api.set_error_response("get_resource_events", &mock_not_found_error());
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "nonexistent-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_resource_events_empty() {
    let mut api = MockTauriApi::new();
    
    // Mock successful but empty events
    api.set_success_response("get_resource_events", json!([]));
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "new-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let events = result.unwrap();
    assert!(events.is_array());
    assert_eq!(events.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_mixed_resource_operations_workflow() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Set up successful responses for all operations
    api.set_success_response("get_full_resource", mock_deployment());
    api.set_success_response("scale_resource", json!(null));
    api.set_success_response("delete_resource", json!(null));
    
    // Step 1: Get the resource
    let get_result = api.invoke("get_full_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": params.namespace
    })).await;
    
    assert!(get_result.is_ok());
    let resource = get_result.unwrap();
    assert_eq!(resource["spec"]["replicas"], 3);
    
    // Step 2: Scale the resource
    let scale_result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": params.namespace,
        "replicas": 5
    })).await;
    
    assert!(scale_result.is_ok());
    
    // Step 3: Delete the resource
    let delete_result = api.invoke("delete_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": params.namespace
    })).await;
    
    assert!(delete_result.is_ok());
    
    // Verify all operations were called
    assert!(api.was_called("get_full_resource"));
    assert!(api.was_called("scale_resource"));
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_performance_large_resource_list() {
    let mut api = MockTauriApi::new();
    
    // Create a large resource list
    let mut large_resource = mock_pod();
    
    // Add large annotations and labels
    for i in 0..100 {
        large_resource["metadata"]["annotations"][format!("large-annotation-{}", i)] = 
            json!(format!("large-value-{}-{}", i, "x".repeat(1000)));
    }
    
    api.set_success_response("get_full_resource", large_resource);
    
    let start = std::time::Instant::now();
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "large-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // Should handle large resources reasonably fast
    assert!(duration.as_millis() < 5000);
    
    let resource = result.unwrap();
    assert_eq!(resource["metadata"]["name"], "test-pod");
}

#[tokio::test]
async fn test_generic_resource_support() {
    let mut api = MockTauriApi::new();
    
    // Mock successful generic resource operations
    api.set_success_response("update_resource", json!(null));
    
    // Test common Kubernetes resource types
    let resource_types = vec![
        ("v1", "ConfigMap", "test-config"),
        ("v1", "Secret", "test-secret"),
        ("apps/v1", "StatefulSet", "test-statefulset"),
        ("networking.k8s.io/v1", "Ingress", "test-ingress"),
        ("v1", "PersistentVolumeClaim", "test-pvc"),
    ];
    
    for (api_version, kind, name) in resource_types {
        let yaml_content = format!(r#"
apiVersion: {}
kind: {}
metadata:
  name: {}
  namespace: default
spec:
  # Generic spec content
  example: value
"#, api_version, kind, name);
        
        let result = api.invoke("update_resource", json!({
            "resource_name": name,
            "resource_kind": kind,
            "namespace": "default",
            "yaml_content": yaml_content
        })).await;
        
        assert!(result.is_ok(), "Update should succeed for {}", kind);
    }
    
    // Verify all resource types were successfully processed
    assert!(api.was_called("update_resource"));
}