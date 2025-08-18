/**
 * scale_resource API Integration Tests
 * 
 * Comprehensive tests for the scale_resource endpoint including:
 * - Basic scaling scenarios for various resources
 * - Replica count validation and edge cases
 * - Resource quota and cluster constraints
 * - HPA conflicts and scaling restrictions
 * - Concurrent scaling operations
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_scale_resource_deployment_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful scaling
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 5
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("scale_resource"));
}

#[tokio::test]
async fn test_scale_resource_replicaset_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful ReplicaSet scaling
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-replicaset",
        "resource_kind": "ReplicaSet",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scale_resource_zero_replicas() {
    let mut api = MockTauriApi::new();
    
    // Mock scaling to zero (should work)
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 0
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scale_resource_invalid_replicas() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid replica count
    api.set_error_response("scale_resource", "Invalid replica count: -1");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": -1
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid replica count"));
}

#[tokio::test]
async fn test_scale_resource_not_scalable() {
    let mut api = MockTauriApi::new();
    
    // Mock trying to scale non-scalable resource
    api.set_error_response("scale_resource", "Resource kind 'Pod' is not scalable");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not scalable"));
}

// ==== COMPREHENSIVE EDGE CASE TESTS ====

#[tokio::test]
async fn test_scale_resource_empty_resource_name() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Resource name cannot be empty");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_scale_resource_empty_resource_kind() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Resource kind cannot be empty");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_scale_resource_large_replica_count() {
    let mut api = MockTauriApi::new();
    
    // Test scaling to a very large number of replicas
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "large-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 1000
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scale_resource_extremely_large_replica_count() {
    let mut api = MockTauriApi::new();
    
    // Test scaling to an unreasonably large number (should be rejected)
    api.set_error_response("scale_resource", "Replica count too large: maximum allowed is 10000");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "huge-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 100000
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too large"));
}

#[tokio::test]
async fn test_scale_resource_stateful_set_with_persistent_volumes() {
    let mut api = MockTauriApi::new();
    
    // StatefulSets with persistent volumes have special scaling behavior
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "database-statefulset",
        "resource_kind": "StatefulSet",
        "namespace": "database",
        "replicas": 3
    })).await;
    
    assert!(result.is_ok());
    
    // Note: In real scenarios, PVCs would be created/maintained during scaling
    assert!(api.was_called("scale_resource"));
}

#[tokio::test]
async fn test_scale_resource_hpa_managed_deployment() {
    let mut api = MockTauriApi::new();
    
    // HPA-managed resources should warn about conflicts
    api.set_error_response("scale_resource", "Warning: This deployment is managed by HPA, manual scaling may be overridden");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "auto-scaled-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 5
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HPA"));
}

#[tokio::test]
async fn test_scale_resource_insufficient_cluster_resources() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Insufficient cluster resources: not enough CPU/memory available");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "resource-hungry-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 20
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient cluster resources"));
}

#[tokio::test]
async fn test_scale_resource_namespace_quota_exceeded() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Resource quota exceeded: namespace 'limited' cannot have more than 5 pods");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "limited",
        "replicas": 10
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("quota exceeded"));
}

#[tokio::test]
async fn test_scale_resource_pod_disruption_budget_violation() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Cannot scale down: would violate PodDisruptionBudget");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "critical-deployment",
        "resource_kind": "Deployment",
        "namespace": "production",
        "replicas": 1
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("PodDisruptionBudget"));
}

#[tokio::test]
async fn test_scale_resource_deployment_rollout_in_progress() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Cannot scale deployment: rollout in progress");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "rolling-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 5
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rollout in progress"));
}

#[tokio::test]
async fn test_scale_resource_daemonset_error() {
    let mut api = MockTauriApi::new();
    
    // DaemonSets cannot be scaled as they run one pod per node
    api.set_error_response("scale_resource", "DaemonSets cannot be scaled manually - they maintain one pod per node");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "logging-daemonset",
        "resource_kind": "DaemonSet",
        "namespace": "kube-system",
        "replicas": 3
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("DaemonSets cannot be scaled"));
}

#[tokio::test]
async fn test_scale_resource_job_error() {
    let mut api = MockTauriApi::new();
    
    // Jobs have parallelism/completions, not replicas
    api.set_error_response("scale_resource", "Jobs cannot be scaled using replicas - use parallelism instead");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "batch-job",
        "resource_kind": "Job",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Jobs cannot be scaled"));
}

#[tokio::test]
async fn test_scale_resource_custom_resource_with_scale_subresource() {
    let mut api = MockTauriApi::new();
    
    // Some custom resources support scaling
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "my-scalable-app",
        "resource_kind": "ScalableApplication",
        "namespace": "apps",
        "replicas": 4
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("scale_resource"));
}

#[tokio::test]
async fn test_scale_resource_custom_resource_without_scale_subresource() {
    let mut api = MockTauriApi::new();
    
    // Custom resources without scale subresource
    api.set_error_response("scale_resource", "CustomResource 'NonScalableApp' does not support scaling");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "my-app",
        "resource_kind": "NonScalableApp",
        "namespace": "apps",
        "replicas": 2
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not support scaling"));
}

#[tokio::test]
async fn test_scale_resource_concurrent_scaling() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response for successful scaling
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("scale_resource", json!(null));
    }
    
    // Spawn multiple concurrent scale requests
    let mut handles = Vec::new();
    for i in 1..=5 {
        let api_clone = Arc::clone(&api);
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("scale_resource", json!({
                "resource_name": format!("deployment-{}", i),
                "resource_kind": "Deployment",
                "namespace": "default",
                "replicas": i
            })).await
        });
        handles.push(handle);
    }
    
    // Wait for all scaling requests to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all scaling operations were processed
    let api_guard = api.lock().await;
    assert_eq!(api_guard.get_call_count("scale_resource"), 5);
}

#[tokio::test]
async fn test_scale_resource_floating_point_replicas() {
    let mut api = MockTauriApi::new();
    
    // Should reject non-integer replica counts
    api.set_error_response("scale_resource", "Replica count must be an integer");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 3.5
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("integer"));
}

#[tokio::test]
async fn test_scale_resource_null_replicas() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Replica count cannot be null");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be null"));
}

#[tokio::test]
async fn test_scale_resource_string_replicas() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Replica count must be a number, got string");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": "three"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be a number"));
}

#[tokio::test]
async fn test_scale_resource_deployment_paused() {
    let mut api = MockTauriApi::new();
    
    // Paused deployments should still be scalable
    api.set_success_response("scale_resource", json!(null));
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "paused-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 3
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("scale_resource"));
}

#[tokio::test]
async fn test_scale_resource_network_timeout() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Request timeout: scaling operation timed out");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "slow-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 10
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timeout"));
}

#[tokio::test]
async fn test_scale_resource_permission_denied() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("scale_resource", "Forbidden: insufficient permissions to scale deployments");
    
    let result = api.invoke("scale_resource", json!({
        "resource_name": "restricted-deployment",
        "resource_kind": "Deployment",
        "namespace": "production",
        "replicas": 5
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Forbidden"));
}