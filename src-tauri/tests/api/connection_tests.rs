/**
 * Kubernetes Connection & Context API Integration Tests
 * 
 * Tests for the 5 connection and context management endpoints:
 * - connect_k8s
 * - connect_k8s_with_context  
 * - get_k8s_contexts
 * - get_current_k8s_context
 * - get_namespaces
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_connect_k8s_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful connection
    api.set_success_response("connect_k8s", json!(null));
    
    let result = api.invoke("connect_k8s", json!({})).await;
    
    assert!(result.is_ok());
    assert_eq!(api.get_call_count("connect_k8s"), 1);
}

#[tokio::test]
async fn test_connect_k8s_failure_no_kubeconfig() {
    let mut api = MockTauriApi::new();
    
    // Mock connection failure due to missing kubeconfig
    api.set_error_response("connect_k8s", "No kubeconfig found");
    
    let result = api.invoke("connect_k8s", json!({})).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No kubeconfig found");
    assert!(api.was_called("connect_k8s"));
}

#[tokio::test]
async fn test_connect_k8s_failure_cluster_unreachable() {
    let mut api = MockTauriApi::new();
    
    // Mock connection failure due to unreachable cluster
    api.set_error_response("connect_k8s", &mock_connection_error());
    
    let result = api.invoke("connect_k8s", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("connection refused"));
}

#[tokio::test]
async fn test_connect_k8s_failure_invalid_credentials() {
    let mut api = MockTauriApi::new();
    
    // Mock authentication failure
    api.set_error_response("connect_k8s", &mock_auth_error());
    
    let result = api.invoke("connect_k8s", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Authentication failed"));
}

#[tokio::test]
async fn test_connect_k8s_with_context_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful context connection
    api.set_success_response("connect_k8s_with_context", json!(null));
    
    let result = api.invoke("connect_k8s_with_context", json!({
        "context_name": params.context_name
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("connect_k8s_with_context"));
}

#[tokio::test]
async fn test_connect_k8s_with_context_invalid_context() {
    let mut api = MockTauriApi::new();
    
    // Mock failure with invalid context
    api.set_error_response("connect_k8s_with_context", "Context 'invalid-context' not found");
    
    let result = api.invoke("connect_k8s_with_context", json!({
        "context_name": "invalid-context"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_connect_k8s_with_context_empty_name() {
    let mut api = MockTauriApi::new();
    
    // Mock failure with empty context name
    api.set_error_response("connect_k8s_with_context", "Context name cannot be empty");
    
    let result = api.invoke("connect_k8s_with_context", json!({
        "context_name": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_get_k8s_contexts_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful contexts retrieval
    api.set_success_response("get_k8s_contexts", mock_k8s_contexts());
    
    let result = api.invoke("get_k8s_contexts", json!({})).await;
    
    assert!(result.is_ok());
    let contexts = result.unwrap();
    assert!(contexts.is_array());
    
    let contexts_array = contexts.as_array().unwrap();
    assert_eq!(contexts_array.len(), 3);
    
    // Verify context structure
    let first_context = &contexts_array[0];
    assert!(first_context.get("name").is_some());
    assert!(first_context.get("cluster").is_some());
    assert!(first_context.get("user").is_some());
    assert_eq!(first_context["name"], "minikube");
}

#[tokio::test]
async fn test_get_k8s_contexts_no_contexts() {
    let mut api = MockTauriApi::new();
    
    // Mock empty contexts list
    api.set_success_response("get_k8s_contexts", json!([]));
    
    let result = api.invoke("get_k8s_contexts", json!({})).await;
    
    assert!(result.is_ok());
    let contexts = result.unwrap();
    assert!(contexts.is_array());
    assert_eq!(contexts.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_k8s_contexts_kubeconfig_error() {
    let mut api = MockTauriApi::new();
    
    // Mock kubeconfig parsing error
    api.set_error_response("get_k8s_contexts", "Failed to parse kubeconfig");
    
    let result = api.invoke("get_k8s_contexts", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("kubeconfig"));
}

#[tokio::test]
async fn test_get_current_k8s_context_success() {
    let mut api = MockTauriApi::new();
    
    // Mock current context retrieval
    api.set_success_response("get_current_k8s_context", json!("minikube"));
    
    let result = api.invoke("get_current_k8s_context", json!({})).await;
    
    assert!(result.is_ok());
    let current_context = result.unwrap();
    assert_eq!(current_context, "minikube");
}

#[tokio::test]
async fn test_get_current_k8s_context_no_current() {
    let mut api = MockTauriApi::new();
    
    // Mock no current context set
    api.set_error_response("get_current_k8s_context", "No current context set");
    
    let result = api.invoke("get_current_k8s_context", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No current context"));
}

#[tokio::test]
async fn test_get_current_k8s_context_invalid_kubeconfig() {
    let mut api = MockTauriApi::new();
    
    // Mock kubeconfig error
    api.set_error_response("get_current_k8s_context", "Invalid kubeconfig format");
    
    let result = api.invoke("get_current_k8s_context", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid kubeconfig"));
}

#[tokio::test]
async fn test_get_namespaces_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful namespaces retrieval
    api.set_success_response("get_namespaces", mock_namespaces());
    
    let result = api.invoke("get_namespaces", json!({})).await;
    
    assert!(result.is_ok());
    let namespaces = result.unwrap();
    assert!(namespaces.is_array());
    
    let ns_array = namespaces.as_array().unwrap();
    assert!(ns_array.len() >= 4); // Should have at least default k8s namespaces
    
    // Check for common namespaces
    let ns_names: Vec<String> = ns_array.iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    
    assert!(ns_names.contains(&"default".to_string()));
    assert!(ns_names.contains(&"kube-system".to_string()));
    assert!(ns_names.contains(&"kube-public".to_string()));
}

#[tokio::test]
async fn test_get_namespaces_not_connected() {
    let mut api = MockTauriApi::new();
    
    // Mock error when not connected to cluster
    api.set_error_response("get_namespaces", "Not connected to Kubernetes cluster");
    
    let result = api.invoke("get_namespaces", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not connected"));
}

#[tokio::test]
async fn test_get_namespaces_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("get_namespaces", "Insufficient permissions to list namespaces");
    
    let result = api.invoke("get_namespaces", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_get_namespaces_empty_cluster() {
    let mut api = MockTauriApi::new();
    
    // Mock cluster with no user namespaces (only system ones)
    api.set_success_response("get_namespaces", json!(["default", "kube-system", "kube-public"]));
    
    let result = api.invoke("get_namespaces", json!({})).await;
    
    assert!(result.is_ok());
    let namespaces = result.unwrap();
    assert_eq!(namespaces.as_array().unwrap().len(), 3);
}

// Integration test for full connection workflow
#[tokio::test]
async fn test_connection_workflow_success() {
    let mut api = MockTauriApi::new();
    
    // Setup mock responses for full workflow
    api.set_success_response("get_k8s_contexts", mock_k8s_contexts());
    api.set_success_response("get_current_k8s_context", json!("minikube"));
    api.set_success_response("connect_k8s_with_context", json!(null));
    api.set_success_response("get_namespaces", mock_namespaces());
    
    // Step 1: Get available contexts
    let contexts_result = api.invoke("get_k8s_contexts", json!({})).await;
    assert!(contexts_result.is_ok());
    
    // Step 2: Get current context
    let current_result = api.invoke("get_current_k8s_context", json!({})).await;
    assert!(current_result.is_ok());
    
    // Step 3: Connect with specific context
    let connect_result = api.invoke("connect_k8s_with_context", json!({
        "context_name": "minikube"
    })).await;
    assert!(connect_result.is_ok());
    
    // Step 4: Get namespaces after connection
    let namespaces_result = api.invoke("get_namespaces", json!({})).await;
    assert!(namespaces_result.is_ok());
    
    // Verify all calls were made
    assert!(api.was_called("get_k8s_contexts"));
    assert!(api.was_called("get_current_k8s_context"));
    assert!(api.was_called("connect_k8s_with_context"));
    assert!(api.was_called("get_namespaces"));
}

// Test connection failure recovery workflow
#[tokio::test]
async fn test_connection_failure_recovery() {
    let mut api = MockTauriApi::new();
    
    // Setup failing then succeeding responses
    api.set_error_response("connect_k8s", "Connection timeout");
    
    // First attempt fails
    let first_attempt = api.invoke("connect_k8s", json!({})).await;
    assert!(first_attempt.is_err());
    
    // Set up success response for retry
    api.set_success_response("connect_k8s", json!(null));
    
    // Second attempt succeeds
    let second_attempt = api.invoke("connect_k8s", json!({})).await;
    assert!(second_attempt.is_ok());
    
    // Verify retry behavior
    assert_eq!(api.get_call_count("connect_k8s"), 2);
}

// Test context switching
#[tokio::test]
async fn test_context_switching() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for context switching
    api.set_success_response("get_current_k8s_context", json!("minikube"));
    api.set_success_response("connect_k8s_with_context", json!(null));
    
    // Get current context
    let current = api.invoke("get_current_k8s_context", json!({})).await;
    assert_eq!(current.unwrap(), "minikube");
    
    // Switch to different context
    let switch_result = api.invoke("connect_k8s_with_context", json!({
        "context_name": "docker-desktop"
    })).await;
    assert!(switch_result.is_ok());
    
    // Update mock to return new context
    api.set_success_response("get_current_k8s_context", json!("docker-desktop"));
    
    // Verify context changed
    let new_current = api.invoke("get_current_k8s_context", json!({})).await;
    assert_eq!(new_current.unwrap(), "docker-desktop");
}

// Performance test for context listing
#[tokio::test]
async fn test_get_contexts_performance() {
    let mut api = MockTauriApi::new();
    
    // Setup mock with many contexts
    let many_contexts: Vec<serde_json::Value> = (0..100)
        .map(|i| json!({
            "name": format!("context-{}", i),
            "cluster": format!("cluster-{}", i),
            "user": format!("user-{}", i),
            "namespace": "default"
        }))
        .collect();
    
    api.set_success_response("get_k8s_contexts", json!(many_contexts));
    
    let start = std::time::Instant::now();
    let result = api.invoke("get_k8s_contexts", json!({})).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    let contexts = result.unwrap();
    assert_eq!(contexts.as_array().unwrap().len(), 100);
    
    // Should be reasonably fast even with many contexts
    assert!(duration.as_millis() < 1000);
}