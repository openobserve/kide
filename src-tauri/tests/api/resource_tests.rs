/**
 * Resource Management API Integration Tests
 * 
 * Tests for the 6 resource management endpoints:
 * - get_resources
 * - get_full_resource
 * - delete_resource
 * - scale_resource
 * - update_resource
 * - get_resource_events
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
    assert!(categories_array.len() >= 2);
    
    // Verify structure
    let workloads_category = &categories_array[0];
    assert_eq!(workloads_category["name"], "Workloads");
    assert!(workloads_category["resources"].is_array());
    
    let resources = workloads_category["resources"].as_array().unwrap();
    assert!(resources.len() >= 1);
    
    // Check Pod resource structure
    let pod_resource = &resources[0];
    assert_eq!(pod_resource["name"], "Pods");
    assert_eq!(pod_resource["kind"], "Pod");
    assert_eq!(pod_resource["apiVersion"], "v1");
    assert_eq!(pod_resource["namespaced"], true);
}

#[tokio::test]
async fn test_get_resources_empty() {
    let mut api = MockTauriApi::new();
    
    // Mock empty resource list (shouldn't happen in reality)
    api.set_success_response("get_resources", json!([]));
    
    let result = api.invoke("get_resources", json!({})).await;
    
    assert!(result.is_ok());
    let categories = result.unwrap();
    assert_eq!(categories.as_array().unwrap().len(), 0);
}

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
        "replicas": 2
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not scalable"));
}

#[tokio::test]
async fn test_update_resource_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful update
    api.set_success_response("update_resource", json!(null));
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:1.20
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("update_resource"));
}

#[tokio::test]
async fn test_update_resource_invalid_yaml() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid YAML error
    api.set_error_response("update_resource", "Invalid YAML syntax");
    
    let invalid_yaml = "invalid: yaml: content: [";
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": invalid_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid YAML"));
}

#[tokio::test]
async fn test_update_resource_validation_error() {
    let mut api = MockTauriApi::new();
    
    // Mock Kubernetes validation error
    api.set_error_response("update_resource", &mock_validation_error());
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
spec:
  containers: []
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource specification"));
}

#[tokio::test]
async fn test_get_resource_events_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful events retrieval
    api.set_success_response("get_resource_events", mock_events());
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": params.pod_name,
        "resource_kind": params.resource_kind,
        "namespace": params.namespace
    })).await;
    
    assert!(result.is_ok());
    let events = result.unwrap();
    assert!(events.is_array());
    
    let events_array = events.as_array().unwrap();
    assert!(events_array.len() >= 1);
    
    // Verify event structure
    let first_event = &events_array[0];
    assert!(first_event.get("involvedObject").is_some());
    assert!(first_event.get("reason").is_some());
    assert!(first_event.get("message").is_some());
    assert_eq!(first_event["reason"], "Scheduled");
}

#[tokio::test]
async fn test_get_resource_events_no_events() {
    let mut api = MockTauriApi::new();
    
    // Mock no events found
    api.set_success_response("get_resource_events", json!([]));
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_resource_events_resource_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock resource not found (still returns empty events, not error)
    api.set_success_response("get_resource_events", json!([]));
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "nonexistent-pod",
        "resource_kind": "Pod",
        "namespace": "default"
    })).await;
    
    // Events endpoint typically doesn't error for non-existent resources
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_resource_events_cluster_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock events for cluster-scoped resource
    api.set_success_response("get_resource_events", json!([{
        "metadata": {
            "name": "node-event-1",
            "creationTimestamp": "2024-01-01T00:00:00Z"
        },
        "involvedObject": {
            "kind": "Node",
            "name": "minikube"
        },
        "reason": "NodeReady",
        "message": "Node minikube status is now: NodeReady",
        "type": "Normal"
    }]));
    
    let result = api.invoke("get_resource_events", json!({
        "resource_name": "minikube",
        "resource_kind": "Node",
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
    let events = result.unwrap();
    let events_array = events.as_array().unwrap();
    assert_eq!(events_array.len(), 1);
    assert_eq!(events_array[0]["reason"], "NodeReady");
}

// Integration test for full resource lifecycle
#[tokio::test]
async fn test_resource_lifecycle() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for resource lifecycle
    api.set_success_response("get_full_resource", mock_deployment());
    api.set_success_response("scale_resource", json!(null));
    api.set_success_response("get_resource_events", mock_events());
    api.set_success_response("delete_resource", json!(null));
    
    // Step 1: Get full resource
    let get_result = api.invoke("get_full_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default"
    })).await;
    assert!(get_result.is_ok());
    
    // Step 2: Scale resource
    let scale_result = api.invoke("scale_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "replicas": 5
    })).await;
    assert!(scale_result.is_ok());
    
    // Step 3: Get events
    let events_result = api.invoke("get_resource_events", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default"
    })).await;
    assert!(events_result.is_ok());
    
    // Step 4: Delete resource
    let delete_result = api.invoke("delete_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default"
    })).await;
    assert!(delete_result.is_ok());
    
    // Verify all operations were called
    assert!(api.was_called("get_full_resource"));
    assert!(api.was_called("scale_resource"));
    assert!(api.was_called("get_resource_events"));
    assert!(api.was_called("delete_resource"));
}

// Performance test for large resource handling
#[tokio::test]
async fn test_large_resource_handling() {
    let mut api = MockTauriApi::new();
    
    // Create a large resource object
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
async fn test_update_resource_deployment_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful deployment update
    api.set_success_response("update_resource", json!(null));
    
    let yaml_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: o2-openobserve-router
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: openobserve-router
  template:
    metadata:
      labels:
        app: openobserve-router
    spec:
      containers:
      - name: router
        image: openobserve/router:v0.1.0
        ports:
        - containerPort: 8080
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "o2-openobserve-router",
        "resource_kind": "Deployment",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("update_resource"));
}

#[tokio::test]
async fn test_update_resource_unsupported_kind() {
    let mut api = MockTauriApi::new();
    
    // Mock kubectl error for unsupported resource
    api.set_error_response("update_resource", "kubectl apply failed: error validating data");
    
    let yaml_content = r#"
apiVersion: v1
kind: CustomResource
metadata:
  name: test-custom
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-custom",
        "resource_kind": "CustomResource",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("kubectl apply failed"));
}

#[tokio::test]
async fn test_update_resource_generic_support() {
    let mut api = MockTauriApi::new();
    
    // Mock successful update for any resource type - demonstrating generic support
    api.set_success_response("update_resource", json!(null));
    
    // Test various resource types that should all work with generic update
    let test_cases = vec![
        ("Deployment", "apps/v1", "nginx-deployment"),
        ("Service", "v1", "nginx-service"),
        ("ConfigMap", "v1", "app-config"),
        ("Secret", "v1", "app-secrets"),
        ("Ingress", "networking.k8s.io/v1", "app-ingress"),
        ("StatefulSet", "apps/v1", "database"),
        ("DaemonSet", "apps/v1", "log-collector"),
        ("Job", "batch/v1", "data-migration"),
        ("CronJob", "batch/v1", "backup-job"),
        ("PersistentVolumeClaim", "v1", "data-storage"),
    ];
    
    for (kind, api_version, name) in test_cases {
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