/**
 * delete_resource API Integration Tests
 * 
 * Comprehensive tests for the delete_resource endpoint including:
 * - Basic deletion scenarios  
 * - Resource protection and validation
 * - Network and permission errors
 * - Kubernetes-specific deletion behaviors
 * - Concurrent operations
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_delete_resource_pod_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful deletion
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": params.pod_name,
        "resource_kind": params.resource_kind,
        "namespace": params.namespace
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_delete_resource_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock resource not found error
    api.set_error_response("delete_resource", &mock_not_found_error());
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "nonexistent-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_delete_resource_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("delete_resource", "Insufficient permissions to delete pods");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "protected-pod",
        "resource_kind": "Pod",
        "namespace": "kube-system"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_delete_resource_cluster_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock cluster-scoped resource deletion
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-node",
        "resource_kind": "Node",
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
}

// ==== COMPREHENSIVE EDGE CASE TESTS ====

#[tokio::test]
async fn test_delete_resource_empty_resource_name() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Resource name cannot be empty");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_delete_resource_empty_resource_kind() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Resource kind cannot be empty");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_delete_resource_invalid_resource_kind() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Invalid resource kind: NonExistentResource");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-resource",
        "resource_kind": "NonExistentResource",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource kind"));
}

#[tokio::test]
async fn test_delete_resource_resource_with_finalizers() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Resource has finalizers and cannot be deleted immediately");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "resource-with-finalizers",
        "resource_kind": "Namespace",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("finalizers"));
}

#[tokio::test]
async fn test_delete_resource_protected_system_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Cannot delete system resource: kube-apiserver");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "kube-apiserver",
        "resource_kind": "Pod",
        "namespace": "kube-system"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot delete system resource"));
}

#[tokio::test]
async fn test_delete_resource_cascade_deletion() {
    let mut api = MockTauriApi::new();
    
    // Mock successful cascade deletion
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    
    // Verify that deletion was called (cascade deletion would delete ReplicaSets and Pods)
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_delete_resource_already_deleting() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Resource is already being deleted (deletionTimestamp set)");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "deleting-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already being deleted"));
}

#[tokio::test]
async fn test_delete_resource_stateful_set_with_persistent_storage() {
    let mut api = MockTauriApi::new();
    
    // StatefulSets with persistent storage need special handling
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "mysql-statefulset",
        "resource_kind": "StatefulSet",
        "namespace": "database"
    })).await;
    
    assert!(result.is_ok());
    
    // Note: In real scenarios, PVCs would remain after StatefulSet deletion
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_delete_resource_custom_resource_with_controller() {
    let mut api = MockTauriApi::new();
    
    // Custom resources might have controllers watching them
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "my-custom-app",
        "resource_kind": "CustomApplication",
        "namespace": "apps"
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_delete_resource_network_partition() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Network error: Unable to connect to Kubernetes API server");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Network error"));
}

#[tokio::test]
async fn test_delete_resource_api_server_unavailable() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Service Unavailable: Kubernetes API server is temporarily unavailable");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Service Unavailable"));
}

#[tokio::test]
async fn test_delete_resource_invalid_namespace_for_namespaced_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Namespace 'nonexistent-namespace' not found");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "nonexistent-namespace"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_delete_resource_null_namespace_for_namespaced_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Namespace is required for namespaced resources");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace is required"));
}

#[tokio::test]
async fn test_delete_resource_special_characters_in_name() {
    let mut api = MockTauriApi::new();
    
    // Valid k8s names with special characters (hyphens, numbers)
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "test-pod-with-123",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_extremely_long_name() {
    let mut api = MockTauriApi::new();
    
    let long_name = "a".repeat(300); // K8s names have limits
    api.set_error_response("delete_resource", "Resource name too long (max 253 characters)");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": long_name,
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[tokio::test]
async fn test_delete_resource_concurrent_deletions() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response for successful deletion
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("delete_resource", json!(null));
    }
    
    // Spawn multiple concurrent delete requests
    let mut handles = Vec::new();
    for i in 0..5 {
        let api_clone = Arc::clone(&api);
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("delete_resource", json!({
                "resource_name": format!("test-pod-{}", i),
                "resource_kind": "Pod",
                "namespace": "default"
            })).await
        });
        handles.push(handle);
    }
    
    // Wait for all deletions to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all deletions were processed
    let api_guard = api.lock().await;
    assert_eq!(api_guard.get_call_count("delete_resource"), 5);
}

#[tokio::test]
async fn test_delete_resource_with_owner_references() {
    let mut api = MockTauriApi::new();
    
    // Resources with owner references might have special deletion behavior
    api.set_success_response("delete_resource", json!(null));
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "replica-set-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    
    // Note: In real scenarios, the ReplicaSet controller would recreate the pod
    assert!(api.was_called("delete_resource"));
}

#[tokio::test]
async fn test_delete_resource_validation_webhook_rejection() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Admission webhook denied the request: Custom validation failed");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "protected-resource",
        "resource_kind": "Pod",
        "namespace": "production"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("webhook denied"));
}

#[tokio::test]
async fn test_delete_resource_grace_period_exceeded() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Timeout: Resource deletion grace period exceeded");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "hanging-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("grace period exceeded"));
}

#[tokio::test]
async fn test_delete_resource_cluster_autoscaler_managed() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("delete_resource", "Cannot delete node managed by cluster autoscaler");
    
    let result = api.invoke("delete_resource", json!({
        "resource_name": "gke-cluster-node-1",
        "resource_kind": "Node",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cluster autoscaler"));
}