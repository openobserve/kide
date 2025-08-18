/**
 * get_full_resource API Integration Tests
 * 
 * Comprehensive tests for the get_full_resource endpoint including:
 * - Basic success scenarios
 * - Error handling and edge cases
 * - Input validation
 * - Network and permission scenarios
 * - Concurrent operations
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_get_full_resource_pod_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful full resource retrieval
    api.set_success_response("get_full_resource", mock_pod());
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": params.pod_name,
        "resource_kind": params.resource_kind,
        "namespace": params.namespace
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    
    // Verify full resource structure
    assert!(resource.get("metadata").is_some());
    assert!(resource.get("spec").is_some());
    assert!(resource.get("status").is_some());
    
    let metadata = &resource["metadata"];
    assert_eq!(metadata["name"], params.pod_name);
    assert_eq!(metadata["namespace"], params.namespace);
}

#[tokio::test]
async fn test_get_full_resource_deployment_success() {
    let mut api = MockTauriApi::new();
    
    // Mock deployment resource
    api.set_success_response("get_full_resource", mock_deployment());
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    
    // Verify deployment-specific fields
    assert_eq!(resource["metadata"]["name"], "test-deployment");
    assert!(resource.get("spec").is_some());
    assert_eq!(resource["spec"]["replicas"], 3);
}

#[tokio::test]
async fn test_get_full_resource_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock resource not found error
    api.set_error_response("get_full_resource", &mock_not_found_error());
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "nonexistent-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_full_resource_invalid_kind() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid resource kind error
    api.set_error_response("get_full_resource", "Invalid resource kind: InvalidKind");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-resource",
        "resource_kind": "InvalidKind",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource kind"));
}

#[tokio::test]
async fn test_get_full_resource_cluster_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock cluster-scoped resource (Node)
    api.set_success_response("get_full_resource", json!({
        "metadata": {
            "name": "minikube",
            "uid": "node-12345"
        },
        "spec": {
            "podCIDR": "10.244.0.0/24"
        },
        "status": {
            "conditions": []
        }
    }));
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "minikube",
        "resource_kind": "Node",
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource["metadata"]["name"], "minikube");
    assert!(resource["metadata"].get("namespace").is_none());
}

// ==== COMPREHENSIVE EDGE CASE TESTS ====

#[tokio::test]
async fn test_get_full_resource_empty_resource_name() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Resource name cannot be empty");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_get_full_resource_empty_resource_kind() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Resource kind cannot be empty");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_get_full_resource_invalid_namespace_format() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Invalid namespace format");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "invalid..namespace"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid"));
}

#[tokio::test]
async fn test_get_full_resource_special_characters_in_name() {
    let mut api = MockTauriApi::new();
    
    // Mock a resource with special characters (valid k8s names can contain hyphens)
    let mut special_pod = mock_pod();
    special_pod["metadata"]["name"] = json!("test-pod-with-special-123");
    
    api.set_success_response("get_full_resource", special_pod);
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod-with-special-123",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource["metadata"]["name"], "test-pod-with-special-123");
}

#[tokio::test]
async fn test_get_full_resource_unicode_namespace() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Invalid namespace: contains non-ASCII characters");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "测试命名空间"
    })).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_full_resource_extremely_long_name() {
    let mut api = MockTauriApi::new();
    
    let long_name = "a".repeat(300); // K8s names have limits
    api.set_error_response("get_full_resource", "Resource name too long");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": long_name,
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_full_resource_null_namespace_for_namespaced_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Namespace required for namespaced resources");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace required"));
}

#[tokio::test]
async fn test_get_full_resource_namespace_for_cluster_resource() {
    let mut api = MockTauriApi::new();
    
    // Mock successful node retrieval (namespace should be ignored for cluster resources)
    api.set_success_response("get_full_resource", json!({
        "metadata": {
            "name": "test-node",
            "uid": "node-12345"
        },
        "spec": {
            "podCIDR": "10.244.0.0/24"
        },
        "status": {
            "conditions": []
        }
    }));
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-node",
        "resource_kind": "Node",
        "namespace": "should-be-ignored"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource["metadata"]["name"], "test-node");
}

#[tokio::test]
async fn test_get_full_resource_case_sensitive_resource_kind() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Invalid resource kind: pod (should be Pod)");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "pod", // lowercase
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_full_resource_network_timeout() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Request timeout: connection to Kubernetes API server timed out");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timeout"));
}

#[tokio::test]
async fn test_get_full_resource_permission_denied() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("get_full_resource", "Forbidden: User does not have permission to get pods in namespace default");
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Forbidden"));
}

#[tokio::test]
async fn test_get_full_resource_malformed_response() {
    let mut api = MockTauriApi::new();
    
    // Mock a malformed resource (missing required fields)
    api.set_success_response("get_full_resource", json!({
        "invalid": "response",
        "missing": "metadata"
    }));
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok()); // API succeeds but returns malformed data
    let resource = result.unwrap();
    assert!(resource.get("metadata").is_none());
}

#[tokio::test]
async fn test_get_full_resource_resource_in_terminating_state() {
    let mut api = MockTauriApi::new();
    
    let mut terminating_pod = mock_pod();
    terminating_pod["metadata"]["deletionTimestamp"] = json!("2024-01-01T00:05:00Z");
    terminating_pod["status"]["phase"] = json!("Terminating");
    
    api.set_success_response("get_full_resource", terminating_pod);
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "terminating-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert!(resource["metadata"].get("deletionTimestamp").is_some());
    assert_eq!(resource["status"]["phase"], "Terminating");
}

#[tokio::test]
async fn test_get_full_resource_custom_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("get_full_resource", json!({
        "apiVersion": "example.com/v1",
        "kind": "CustomResource",
        "metadata": {
            "name": "my-custom-resource",
            "namespace": "default"
        },
        "spec": {
            "customField": "customValue"
        },
        "status": {
            "customStatus": "active"
        }
    }));
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "my-custom-resource",
        "resource_kind": "CustomResource",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource["apiVersion"], "example.com/v1");
    assert_eq!(resource["spec"]["customField"], "customValue");
}

#[tokio::test]
async fn test_get_full_resource_concurrent_requests() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("get_full_resource", mock_pod());
    }
    
    // Spawn multiple concurrent requests
    let mut handles = Vec::new();
    for i in 0..10 {
        let api_clone = Arc::clone(&api);
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("get_full_resource", json!({
                "resource_name": format!("test-pod-{}", i),
                "resource_kind": "Pod",
                "namespace": "default"
            })).await
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all requests were processed
    let api_guard = api.lock().await;
    assert_eq!(api_guard.get_call_count("get_full_resource"), 10);
}

#[tokio::test]
async fn test_get_full_resource_json_edge_cases() {
    let mut api = MockTauriApi::new();
    
    // Test resource with various JSON edge cases
    let edge_case_resource = json!({
        "metadata": {
            "name": "edge-case-pod",
            "namespace": "default",
            "annotations": {
                "special/characters": "value with spaces and ÜñĨçødé",
                "json-string": "{\"nested\": \"json\"}",
                "empty-string": "",
                "null-value": null,
                "number-as-string": "42",
                "boolean-as-string": "true"
            },
            "labels": {
                "app": "test",
                "version": "1.0.0"
            }
        },
        "spec": {
            "containers": [{
                "name": "test",
                "image": "nginx:latest",
                "env": [
                    {"name": "EMPTY_VAR", "value": ""},
                    {"name": "NULL_VAR", "value": null},
                    {"name": "SPECIAL_CHARS", "value": "!@#$%^&*()"}
                ]
            }]
        },
        "status": {
            "phase": "Running",
            "conditions": []
        }
    });
    
    api.set_success_response("get_full_resource", edge_case_resource);
    
    let result = api.invoke("get_full_resource", json!({
        "resource_name": "edge-case-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource["metadata"]["annotations"]["special/characters"], "value with spaces and ÜñĨçødé");
    assert!(resource["metadata"]["annotations"]["null-value"].is_null());
}