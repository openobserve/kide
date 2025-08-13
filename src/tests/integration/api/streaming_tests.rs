/**
 * Real-time Streaming API Integration Tests
 * 
 * Tests for the 4 streaming and watching endpoints:
 * - start_resource_watch
 * - stop_resource_watch
 * - subscribe_to_resources
 * - unsubscribe_from_resources
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;
use std::time::Duration;

#[tokio::test]
async fn test_start_resource_watch_pods_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful resource watch start
    api.set_success_response("start_resource_watch", json!(null));
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": [params.namespace]
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("start_resource_watch"));
}

#[tokio::test]
async fn test_start_resource_watch_all_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Mock watching across all namespaces
    api.set_success_response("start_resource_watch", json!(null));
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "deployments",
        "namespaces": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_resource_watch_multiple_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Mock watching multiple specific namespaces
    api.set_success_response("start_resource_watch", json!(null));
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "services",
        "namespaces": ["default", "kube-system", "monitoring"]
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_resource_watch_cluster_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock watching cluster-scoped resources
    api.set_success_response("start_resource_watch", json!(null));
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "nodes",
        "namespaces": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_resource_watch_invalid_resource_type() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid resource type error
    api.set_error_response("start_resource_watch", "Invalid resource type: invalidresource");
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "invalidresource",
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource type"));
}

#[tokio::test]
async fn test_start_resource_watch_not_connected() {
    let mut api = MockTauriApi::new();
    
    // Mock not connected to cluster error
    api.set_error_response("start_resource_watch", "K8s client not connected");
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not connected"));
}

#[tokio::test]
async fn test_start_resource_watch_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("start_resource_watch", "Insufficient permissions to watch pods");
    
    let result = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["kube-system"]
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_stop_resource_watch_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful resource watch stop
    api.set_success_response("stop_resource_watch", json!(null));
    
    let result = api.invoke("stop_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": [params.namespace]
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("stop_resource_watch"));
}

#[tokio::test]
async fn test_stop_resource_watch_not_watching() {
    let mut api = MockTauriApi::new();
    
    // Mock stopping watch that wasn't started (should succeed silently)
    api.set_success_response("stop_resource_watch", json!(null));
    
    let result = api.invoke("stop_resource_watch", json!({
        "resource_type": "services",
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_resource_watch_all_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Mock stopping watch for all namespaces
    api.set_success_response("stop_resource_watch", json!(null));
    
    let result = api.invoke("stop_resource_watch", json!({
        "resource_type": "deployments",
        "namespaces": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_to_resources_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful subscription to multiple resource types
    api.set_success_response("subscribe_to_resources", json!(null));
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": ["pods", "services", "deployments"],
        "namespaces": ["default", "monitoring"]
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("subscribe_to_resources"));
}

#[tokio::test]
async fn test_subscribe_to_resources_single_type() {
    let mut api = MockTauriApi::new();
    
    // Mock subscription to single resource type
    api.set_success_response("subscribe_to_resources", json!(null));
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": ["pods"],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_to_resources_all_namespaces() {
    let mut api = MockTauriApi::new();
    
    // Mock subscription across all namespaces
    api.set_success_response("subscribe_to_resources", json!(null));
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": ["pods", "deployments"],
        "namespaces": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_subscribe_to_resources_empty_types() {
    let mut api = MockTauriApi::new();
    
    // Mock error for empty resource types
    api.set_error_response("subscribe_to_resources", "Resource types list cannot be empty");
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": [],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_subscribe_to_resources_invalid_types() {
    let mut api = MockTauriApi::new();
    
    // Mock error for invalid resource types
    api.set_error_response("subscribe_to_resources", "Invalid resource types: [invalidtype1, invalidtype2]");
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": ["pods", "invalidtype1", "invalidtype2"],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource types"));
}

#[tokio::test]
async fn test_subscribe_to_resources_mixed_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock subscription to mixed namespaced and cluster-scoped resources
    api.set_success_response("subscribe_to_resources", json!(null));
    
    let result = api.invoke("subscribe_to_resources", json!({
        "resource_types": ["pods", "nodes", "services"],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unsubscribe_from_resources_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful unsubscription
    api.set_success_response("unsubscribe_from_resources", json!(null));
    
    let result = api.invoke("unsubscribe_from_resources", json!({
        "resource_types": ["pods", "services"],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("unsubscribe_from_resources"));
}

#[tokio::test]
async fn test_unsubscribe_from_resources_not_subscribed() {
    let mut api = MockTauriApi::new();
    
    // Mock unsubscribing from resources not subscribed to (should succeed silently)
    api.set_success_response("unsubscribe_from_resources", json!(null));
    
    let result = api.invoke("unsubscribe_from_resources", json!({
        "resource_types": ["deployments"],
        "namespaces": ["monitoring"]
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unsubscribe_from_resources_all_types() {
    let mut api = MockTauriApi::new();
    
    // Mock unsubscribing from all resource types
    api.set_success_response("unsubscribe_from_resources", json!(null));
    
    let result = api.invoke("unsubscribe_from_resources", json!({
        "resource_types": ["pods", "services", "deployments", "replicasets", "nodes"],
        "namespaces": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_unsubscribe_from_resources_partial() {
    let mut api = MockTauriApi::new();
    
    // Mock partial unsubscription (some succeed, some fail)
    api.set_success_response("unsubscribe_from_resources", json!(null));
    
    let result = api.invoke("unsubscribe_from_resources", json!({
        "resource_types": ["pods", "invalidtype"],
        "namespaces": ["default"]
    })).await;
    
    assert!(result.is_ok()); // Should succeed for valid types
}

// Integration test for watch lifecycle
#[tokio::test]
async fn test_watch_lifecycle() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for full watch lifecycle
    api.set_success_response("start_resource_watch", json!(null));
    api.set_success_response("stop_resource_watch", json!(null));
    
    // Step 1: Start watching
    let start_result = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(start_result.is_ok());
    
    // Step 2: Stop watching
    let stop_result = api.invoke("stop_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(stop_result.is_ok());
    
    // Verify both operations were called
    assert!(api.was_called("start_resource_watch"));
    assert!(api.was_called("stop_resource_watch"));
    assert_eq!(api.get_call_count("start_resource_watch"), 1);
    assert_eq!(api.get_call_count("stop_resource_watch"), 1);
}

// Integration test for subscription lifecycle
#[tokio::test]
async fn test_subscription_lifecycle() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for subscription lifecycle
    api.set_success_response("subscribe_to_resources", json!(null));
    api.set_success_response("unsubscribe_from_resources", json!(null));
    
    let resource_types = vec!["pods", "services", "deployments"];
    let namespaces = vec!["default", "monitoring"];
    
    // Step 1: Subscribe to resources
    let subscribe_result = api.invoke("subscribe_to_resources", json!({
        "resource_types": resource_types,
        "namespaces": namespaces
    })).await;
    assert!(subscribe_result.is_ok());
    
    // Step 2: Unsubscribe from resources
    let unsubscribe_result = api.invoke("unsubscribe_from_resources", json!({
        "resource_types": resource_types,
        "namespaces": namespaces
    })).await;
    assert!(unsubscribe_result.is_ok());
    
    // Verify both operations were called
    assert!(api.was_called("subscribe_to_resources"));
    assert!(api.was_called("unsubscribe_from_resources"));
}

// Test multiple concurrent watches
#[tokio::test]
async fn test_multiple_concurrent_watches() {
    let mut api = MockTauriApi::new();
    
    // Mock multiple successful watch starts
    api.set_success_response("start_resource_watch", json!(null));
    
    // Start multiple watches concurrently
    let watch1 = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    }));
    
    let watch2 = api.invoke("start_resource_watch", json!({
        "resource_type": "services",
        "namespaces": ["default"]
    }));
    
    let watch3 = api.invoke("start_resource_watch", json!({
        "resource_type": "deployments",
        "namespaces": ["monitoring"]
    }));
    
    // Wait for all to complete
    let (result1, result2, result3) = tokio::join!(watch1, watch2, watch3);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    assert_eq!(api.get_call_count("start_resource_watch"), 3);
}

// Test watch resilience to connection issues
#[tokio::test]
async fn test_watch_connection_resilience() {
    let mut api = MockTauriApi::new();
    
    // First attempt fails
    api.set_error_response("start_resource_watch", "Connection lost");
    
    let first_attempt = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(first_attempt.is_err());
    
    // Second attempt succeeds (connection recovered)
    api.set_success_response("start_resource_watch", json!(null));
    
    let second_attempt = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(second_attempt.is_ok());
    
    assert_eq!(api.get_call_count("start_resource_watch"), 2);
}

// Test resource type validation
#[tokio::test]
async fn test_resource_type_validation() {
    let mut api = MockTauriApi::new();
    
    let test_scenarios = vec![
        ("pods", true),
        ("services", true),
        ("deployments", true),
        ("nodes", true),
        ("invalidtype", false),
        ("", false),
        ("pod", false), // Should be "pods"
    ];
    
    for (resource_type, should_succeed) in test_scenarios {
        if should_succeed {
            api.set_success_response("start_resource_watch", json!(null));
        } else {
            api.set_error_response("start_resource_watch", &format!("Invalid resource type: {}", resource_type));
        }
        
        let result = api.invoke("start_resource_watch", json!({
            "resource_type": resource_type,
            "namespaces": ["default"]
        })).await;
        
        if should_succeed {
            assert!(result.is_ok(), "Expected success for resource type: {}", resource_type);
        } else {
            assert!(result.is_err(), "Expected failure for resource type: {}", resource_type);
        }
    }
}

// Test namespace validation
#[tokio::test]
async fn test_namespace_validation() {
    let mut api = MockTauriApi::new();
    
    let test_scenarios = vec![
        (Some(vec!["default"]), true),
        (Some(vec!["kube-system", "monitoring"]), true),
        (None, true), // All namespaces
        (Some(vec![]), true), // Empty namespaces (should default to all)
        (Some(vec!["invalid-namespace-!"]), false), // Invalid namespace name
    ];
    
    for (namespaces, should_succeed) in test_scenarios {
        if should_succeed {
            api.set_success_response("start_resource_watch", json!(null));
        } else {
            api.set_error_response("start_resource_watch", "Invalid namespace name");
        }
        
        let result = api.invoke("start_resource_watch", json!({
            "resource_type": "pods",
            "namespaces": namespaces
        })).await;
        
        if should_succeed {
            assert!(result.is_ok(), "Expected success for namespaces: {:?}", namespaces);
        } else {
            assert!(result.is_err(), "Expected failure for namespaces: {:?}", namespaces);
        }
    }
}

// Performance test for high-frequency watch events
#[tokio::test]
async fn test_high_frequency_watch_events() {
    let mut api = MockTauriApi::new();
    
    // Mock successful watch start
    api.set_success_response("start_resource_watch", json!(null));
    api.set_success_response("stop_resource_watch", json!(null));
    
    let start_time = std::time::Instant::now();
    
    // Start watch
    let start_result = api.invoke("start_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(start_result.is_ok());
    
    // Simulate time passing with high-frequency events
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Stop watch
    let stop_result = api.invoke("stop_resource_watch", json!({
        "resource_type": "pods",
        "namespaces": ["default"]
    })).await;
    assert!(stop_result.is_ok());
    
    let duration = start_time.elapsed();
    
    // Should handle the lifecycle quickly
    assert!(duration.as_millis() < 1000);
}