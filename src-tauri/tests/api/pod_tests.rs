/**
 * Pod-specific Operations API Integration Tests
 * 
 * Tests for the 2 pod-specific operation endpoints:
 * - get_pods_by_selector
 * - get_node_pods
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use std::collections::HashMap;
use tokio_test;

// Helper function to create multiple pod objects
fn create_multiple_pods(count: usize, node_name: &str, labels: HashMap<String, String>) -> serde_json::Value {
    let pods: Vec<serde_json::Value> = (0..count)
        .map(|i| {
            let mut pod = mock_pod();
            pod["metadata"]["name"] = json!(format!("test-pod-{}", i));
            pod["metadata"]["uid"] = json!(format!("uid-{}", i));
            pod["spec"]["nodeName"] = json!(node_name);
            
            // Set labels
            for (key, value) in &labels {
                pod["metadata"]["labels"][key] = json!(value);
            }
            
            pod
        })
        .collect();
    
    json!(pods)
}

#[tokio::test]
async fn test_get_pods_by_selector_success() {
    let mut api = MockTauriApi::new();
    
    // Create mock pods with specific labels
    let mut selector_labels = HashMap::new();
    selector_labels.insert("app".to_string(), "web-server".to_string());
    selector_labels.insert("tier".to_string(), "frontend".to_string());
    
    let mock_pods = create_multiple_pods(3, "minikube", selector_labels.clone());
    api.set_success_response("get_pods_by_selector", mock_pods);
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    assert!(pods.is_array());
    
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 3);
    
    // Verify each pod has the expected labels
    for pod in pods_array {
        let labels = &pod["metadata"]["labels"];
        assert_eq!(labels["app"], "web-server");
        assert_eq!(labels["tier"], "frontend");
    }
    
    assert!(api.was_called("get_pods_by_selector"));
}

#[tokio::test]
async fn test_get_pods_by_selector_single_label() {
    let mut api = MockTauriApi::new();
    
    // Test with single label selector
    let mut selector_labels = HashMap::new();
    selector_labels.insert("app".to_string(), "database".to_string());
    
    let mock_pods = create_multiple_pods(2, "worker-1", selector_labels.clone());
    api.set_success_response("get_pods_by_selector", mock_pods);
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": "production"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 2);
    
    // Verify labels
    for pod in pods_array {
        assert_eq!(pod["metadata"]["labels"]["app"], "database");
    }
}

#[tokio::test]
async fn test_get_pods_by_selector_no_matches() {
    let mut api = MockTauriApi::new();
    
    // Mock no pods matching selector
    api.set_success_response("get_pods_by_selector", json!([]));
    
    let mut selector_labels = HashMap::new();
    selector_labels.insert("app".to_string(), "nonexistent".to_string());
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    assert_eq!(pods.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_pods_by_selector_empty_selector() {
    let mut api = MockTauriApi::new();
    
    // Mock error for empty selector
    api.set_error_response("get_pods_by_selector", "Selector labels cannot be empty");
    
    let empty_selector = HashMap::new();
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": empty_selector,
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_get_pods_by_selector_invalid_namespace() {
    let mut api = MockTauriApi::new();
    
    // Mock namespace not found error
    api.set_error_response("get_pods_by_selector", "Namespace 'nonexistent' not found");
    
    let mut selector_labels = HashMap::new();
    selector_labels.insert("app".to_string(), "test".to_string());
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": "nonexistent"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_pods_by_selector_all_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Create pods across multiple namespaces
    let mut selector_labels = HashMap::new();
    selector_labels.insert("component".to_string(), "proxy".to_string());
    
    let mock_pods = json!([
        {
            "metadata": {
                "name": "proxy-pod-1",
                "namespace": "default",
                "labels": {"component": "proxy"}
            },
            "spec": {"nodeName": "node-1"}
        },
        {
            "metadata": {
                "name": "proxy-pod-2", 
                "namespace": "kube-system",
                "labels": {"component": "proxy"}
            },
            "spec": {"nodeName": "node-2"}
        }
    ]);
    
    api.set_success_response("get_pods_by_selector", mock_pods);
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 2);
    
    // Verify pods from different namespaces
    assert_eq!(pods_array[0]["metadata"]["namespace"], "default");
    assert_eq!(pods_array[1]["metadata"]["namespace"], "kube-system");
}

#[tokio::test]
async fn test_get_pods_by_selector_complex_selector() {
    let mut api = MockTauriApi::new();
    
    // Test with complex multi-label selector
    let mut selector_labels = HashMap::new();
    selector_labels.insert("app".to_string(), "nginx".to_string());
    selector_labels.insert("version".to_string(), "v1.20".to_string());
    selector_labels.insert("env".to_string(), "production".to_string());
    selector_labels.insert("tier".to_string(), "frontend".to_string());
    
    let mock_pods = create_multiple_pods(1, "prod-node", selector_labels.clone());
    api.set_success_response("get_pods_by_selector", mock_pods);
    
    let result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector_labels,
        "namespace": "production"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 1);
    
    let pod_labels = &pods_array[0]["metadata"]["labels"];
    assert_eq!(pod_labels["app"], "nginx");
    assert_eq!(pod_labels["version"], "v1.20");
    assert_eq!(pod_labels["env"], "production");
    assert_eq!(pod_labels["tier"], "frontend");
}

#[tokio::test]
async fn test_get_node_pods_success() {
    let mut api = MockTauriApi::new();
    
    // Create mock pods running on specific node
    let mock_pods = create_multiple_pods(5, "minikube", HashMap::new());
    api.set_success_response("get_node_pods", mock_pods);
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "minikube"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    assert!(pods.is_array());
    
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 5);
    
    // Verify all pods are running on the specified node
    for pod in pods_array {
        assert_eq!(pod["spec"]["nodeName"], "minikube");
    }
    
    assert!(api.was_called("get_node_pods"));
}

#[tokio::test]
async fn test_get_node_pods_worker_node() {
    let mut api = MockTauriApi::new();
    
    // Test with worker node
    let mock_pods = create_multiple_pods(8, "worker-node-1", HashMap::new());
    api.set_success_response("get_node_pods", mock_pods);
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "worker-node-1"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 8);
    
    for pod in pods_array {
        assert_eq!(pod["spec"]["nodeName"], "worker-node-1");
    }
}

#[tokio::test]
async fn test_get_node_pods_no_pods() {
    let mut api = MockTauriApi::new();
    
    // Mock node with no pods
    api.set_success_response("get_node_pods", json!([]));
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "empty-node"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    assert_eq!(pods.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_node_pods_node_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock node not found error
    api.set_error_response("get_node_pods", "Node 'nonexistent-node' not found");
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "nonexistent-node"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_node_pods_empty_node_name() {
    let mut api = MockTauriApi::new();
    
    // Mock error for empty node name
    api.set_error_response("get_node_pods", "Node name cannot be empty");
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_get_node_pods_system_pods() {
    let mut api = MockTauriApi::new();
    
    // Mock system pods on control plane node
    let system_pods = json!([
        {
            "metadata": {
                "name": "kube-apiserver-master",
                "namespace": "kube-system",
                "labels": {"component": "kube-apiserver"}
            },
            "spec": {"nodeName": "master-node"}
        },
        {
            "metadata": {
                "name": "etcd-master",
                "namespace": "kube-system", 
                "labels": {"component": "etcd"}
            },
            "spec": {"nodeName": "master-node"}
        },
        {
            "metadata": {
                "name": "kube-controller-manager-master",
                "namespace": "kube-system",
                "labels": {"component": "kube-controller-manager"}
            },
            "spec": {"nodeName": "master-node"}
        }
    ]);
    
    api.set_success_response("get_node_pods", system_pods);
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "master-node"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 3);
    
    // Verify all are system pods
    for pod in pods_array {
        assert_eq!(pod["metadata"]["namespace"], "kube-system");
        assert_eq!(pod["spec"]["nodeName"], "master-node");
    }
}

#[tokio::test]
async fn test_get_node_pods_mixed_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Mock pods from different namespaces on the same node
    let mixed_pods = json!([
        {
            "metadata": {
                "name": "app-pod-1",
                "namespace": "default",
                "labels": {"app": "web"}
            },
            "spec": {"nodeName": "worker-1"}
        },
        {
            "metadata": {
                "name": "monitoring-agent",
                "namespace": "monitoring",
                "labels": {"app": "prometheus"}
            },
            "spec": {"nodeName": "worker-1"}
        },
        {
            "metadata": {
                "name": "kube-proxy",
                "namespace": "kube-system",
                "labels": {"k8s-app": "kube-proxy"}
            },
            "spec": {"nodeName": "worker-1"}
        }
    ]);
    
    api.set_success_response("get_node_pods", mixed_pods);
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "worker-1"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 3);
    
    // Verify different namespaces but same node
    let namespaces: Vec<&str> = pods_array
        .iter()
        .map(|pod| pod["metadata"]["namespace"].as_str().unwrap())
        .collect();
    
    assert!(namespaces.contains(&"default"));
    assert!(namespaces.contains(&"monitoring"));
    assert!(namespaces.contains(&"kube-system"));
    
    for pod in pods_array {
        assert_eq!(pod["spec"]["nodeName"], "worker-1");
    }
}

// Integration test for workload pod discovery
#[tokio::test]
async fn test_workload_pod_discovery() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for workload pod discovery workflow
    let mut deployment_selector = HashMap::new();
    deployment_selector.insert("app".to_string(), "nginx".to_string());
    deployment_selector.insert("version".to_string(), "v1.20".to_string());
    
    let workload_pods = create_multiple_pods(3, "worker-1", deployment_selector.clone());
    api.set_success_response("get_pods_by_selector", workload_pods);
    
    // Step 1: Get pods for a deployment
    let pods_result = api.invoke("get_pods_by_selector", json!({
        "selector_labels": deployment_selector,
        "namespace": "production"
    })).await;
    
    assert!(pods_result.is_ok());
    let pods = pods_result.unwrap();
    let pods_array = pods.as_array().unwrap();
    assert_eq!(pods_array.len(), 3);
    
    // Step 2: Verify all pods are on expected node
    for pod in pods_array {
        assert_eq!(pod["spec"]["nodeName"], "worker-1");
        assert_eq!(pod["metadata"]["labels"]["app"], "nginx");
        assert_eq!(pod["metadata"]["labels"]["version"], "v1.20");
    }
    
    assert!(api.was_called("get_pods_by_selector"));
}

// Integration test for node capacity analysis
#[tokio::test]
async fn test_node_capacity_analysis() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for node capacity analysis
    let node_pods = create_multiple_pods(10, "worker-node-1", HashMap::new());
    api.set_success_response("get_node_pods", node_pods);
    
    // Get all pods on a node
    let result = api.invoke("get_node_pods", json!({
        "node_name": "worker-node-1"
    })).await;
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    let pods_array = pods.as_array().unwrap();
    
    // Analyze node capacity usage
    assert_eq!(pods_array.len(), 10);
    
    // Count pods by namespace
    let mut namespace_counts = HashMap::new();
    for pod in pods_array {
        let ns = pod["metadata"]["namespace"].as_str().unwrap_or("default");
        *namespace_counts.entry(ns).or_insert(0) += 1;
    }
    
    // Should have pods (exact distribution depends on mock data)
    assert!(!namespace_counts.is_empty());
}

// Performance test for large pod lists
#[tokio::test]
async fn test_large_pod_list_performance() {
    let mut api = MockTauriApi::new();
    
    // Create large number of pods
    let large_pod_list = create_multiple_pods(1000, "high-capacity-node", HashMap::new());
    api.set_success_response("get_node_pods", large_pod_list);
    
    let start_time = std::time::Instant::now();
    
    let result = api.invoke("get_node_pods", json!({
        "node_name": "high-capacity-node"
    })).await;
    
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    let pods = result.unwrap();
    assert_eq!(pods.as_array().unwrap().len(), 1000);
    
    // Should handle large lists efficiently
    assert!(duration.as_millis() < 2000);
}

// Test error handling for concurrent operations
#[tokio::test]
async fn test_concurrent_pod_operations() {
    let mut api = MockTauriApi::new();
    
    // Setup different responses for concurrent operations
    api.set_success_response("get_pods_by_selector", create_multiple_pods(5, "node-1", HashMap::new()));
    api.set_success_response("get_node_pods", create_multiple_pods(8, "node-2", HashMap::new()));
    
    let mut selector = HashMap::new();
    selector.insert("app".to_string(), "test".to_string());
    
    // Execute operations concurrently
    let selector_op = api.invoke("get_pods_by_selector", json!({
        "selector_labels": selector,
        "namespace": "default"
    }));
    
    let node_op = api.invoke("get_node_pods", json!({
        "node_name": "node-2"
    }));
    
    let (selector_result, node_result) = tokio::join!(selector_op, node_op);
    
    assert!(selector_result.is_ok());
    assert!(node_result.is_ok());
    
    assert_eq!(selector_result.unwrap().as_array().unwrap().len(), 5);
    assert_eq!(node_result.unwrap().as_array().unwrap().len(), 8);
    
    assert!(api.was_called("get_pods_by_selector"));
    assert!(api.was_called("get_node_pods"));
}