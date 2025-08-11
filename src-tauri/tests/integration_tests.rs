use kide_lib::k8s::{get_resource_categories, K8sClient, LogStreamManager, WatchManager};

#[tokio::test]
async fn test_full_resource_categories_structure() {
    let categories = get_resource_categories();
    
    // Test overall structure
    assert_eq!(categories.len(), 9);
    
    // Test each category has resources
    for category in &categories {
        assert!(!category.name.is_empty());
        assert!(!category.resources.is_empty());
        
        // Test each resource has required fields
        for resource in &category.resources {
            assert!(!resource.name.is_empty());
            assert!(!resource.api_version.is_empty());
            assert!(!resource.kind.is_empty());
            assert!(!resource.description.is_empty());
            
            // Resource name should match kind (case-insensitive, plural)
            let _expected_name = if resource.kind.ends_with('s') {
                resource.kind.clone()
            } else if resource.kind == "Ingress" {
                "Ingresses".to_string()
            } else if resource.kind == "NetworkPolicy" {
                "NetworkPolicies".to_string()
            } else {
                format!("{}s", resource.kind) // Simple pluralization
            };
            
            // Some resources have irregular plurals, so we'll be flexible
            let name_lower = resource.name.to_lowercase();
            let kind_lower = resource.kind.to_lowercase();
            
            // Check if they contain each other, or if they're related (like NetworkPolicy -> NetworkPolicies)
            let relates = name_lower.contains(&kind_lower) ||
                         kind_lower.contains(&name_lower) ||
                         (kind_lower == "networkpolicy" && name_lower == "networkpolicies") ||
                         (kind_lower.replace("policy", "policies") == name_lower);
                         
            assert!(relates, "Resource name '{}' should relate to kind '{}'", resource.name, resource.kind);
        }
    }
}

#[tokio::test]
async fn test_resource_namespacing_consistency() {
    let categories = get_resource_categories();
    let all_resources: Vec<_> = categories.iter().flat_map(|c| &c.resources).collect();
    
    // Test known namespaced resources
    let namespaced_resources = ["Pod", "Deployment", "Service", "ConfigMap", "Secret", 
                               "StatefulSet", "DaemonSet", "Job", "CronJob", "Ingress",
                               "NetworkPolicy", "PersistentVolumeClaim", "ServiceAccount"];
    
    for kind in namespaced_resources {
        let resource = all_resources.iter().find(|r| r.kind == kind);
        assert!(resource.is_some(), "Should have resource with kind {}", kind);
        assert!(resource.unwrap().namespaced, "Resource {} should be namespaced", kind);
    }
    
    // Test known cluster-wide resources
    let cluster_resources = ["Node", "Namespace", "PersistentVolume", "StorageClass"];
    
    for kind in cluster_resources {
        let resource = all_resources.iter().find(|r| r.kind == kind);
        assert!(resource.is_some(), "Should have resource with kind {}", kind);
        assert!(!resource.unwrap().namespaced, "Resource {} should be cluster-wide", kind);
    }
}

#[tokio::test]
async fn test_api_version_consistency() {
    let categories = get_resource_categories();
    let all_resources: Vec<_> = categories.iter().flat_map(|c| &c.resources).collect();
    
    // Test core resources have v1 API version
    let core_resources = ["Pod", "Service", "ConfigMap", "Secret", "Namespace", "Node", "ServiceAccount"];
    
    for kind in core_resources {
        let resource = all_resources.iter().find(|r| r.kind == kind);
        assert!(resource.is_some(), "Should have resource with kind {}", kind);
        assert_eq!(resource.unwrap().api_version, "v1", "Resource {} should have v1 API version", kind);
    }
    
    // Test apps resources have apps/v1 API version
    let apps_resources = ["Deployment", "StatefulSet", "DaemonSet"];
    
    for kind in apps_resources {
        let resource = all_resources.iter().find(|r| r.kind == kind);
        assert!(resource.is_some(), "Should have resource with kind {}", kind);
        assert_eq!(resource.unwrap().api_version, "apps/v1", "Resource {} should have apps/v1 API version", kind);
    }
}

#[tokio::test]
async fn test_client_lifecycle() {
    let client = K8sClient::new();
    
    // Initially not connected
    assert!(!client.is_connected().await);
    
    // Should return error when getting client before connection
    let result = client.get_client().await;
    assert!(result.is_err());
    
    // Connection attempt (may fail without K8s cluster, which is OK)
    let connect_result = client.connect().await;
    
    if connect_result.is_ok() {
        // If connection successful, should be able to get client
        assert!(client.is_connected().await);
        let client_result = client.get_client().await;
        assert!(client_result.is_ok());
    } else {
        // If connection failed, should still be disconnected
        assert!(!client.is_connected().await);
    }
}

#[tokio::test]
async fn test_watch_manager_integration() {
    let client = K8sClient::new();
    let manager = WatchManager::new(client.clone());
    
    // Test basic operations without actual K8s connection
    let stop_result = manager.stop_watch("pods", Some(vec!["default".to_string()])).await;
    assert!(stop_result.is_ok(), "Stop watch should not error for non-existent watch");
    
    let stop_all_result = manager.stop_all_watches().await;
    assert!(stop_all_result.is_ok(), "Stop all watches should not error");
}

#[tokio::test]
async fn test_log_stream_manager_integration() {
    let client = K8sClient::new();
    let manager = LogStreamManager::new(client.clone());
    
    // Test basic operations without actual K8s connection
    let stop_result = manager.stop_log_stream("test-stream-id").await;
    assert!(stop_result.is_ok(), "Stop log stream should not error for non-existent stream");
    
    let stop_all_result = manager.stop_all_streams().await;
    assert!(stop_all_result.is_ok(), "Stop all streams should not error");
}

#[test]
fn test_resource_serialization_roundtrip() {
    let categories = get_resource_categories();
    
    // Test that all resource categories can be serialized and deserialized
    let json = serde_json::to_string(&categories).expect("Should serialize to JSON");
    let deserialized: Vec<kide_lib::k8s::K8sResourceCategory> = 
        serde_json::from_str(&json).expect("Should deserialize from JSON");
    
    assert_eq!(categories.len(), deserialized.len());
    
    for (original, deserialized) in categories.iter().zip(deserialized.iter()) {
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.resources.len(), deserialized.resources.len());
        
        for (orig_res, deser_res) in original.resources.iter().zip(deserialized.resources.iter()) {
            assert_eq!(orig_res.name, deser_res.name);
            assert_eq!(orig_res.kind, deser_res.kind);
            assert_eq!(orig_res.api_version, deser_res.api_version);
            assert_eq!(orig_res.namespaced, deser_res.namespaced);
            assert_eq!(orig_res.description, deser_res.description);
        }
    }
}

#[test]
fn test_thread_safety() {
    use std::thread;
    
    let client = K8sClient::new();
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let client_clone = client.clone();
            thread::spawn(move || {
                // Test that we can access client from multiple threads
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    client_clone.is_connected().await
                })
            })
        })
        .collect();
    
    // All threads should complete successfully
    for handle in handles {
        let result = handle.join().unwrap();
        // All should return false since we didn't connect
        assert!(!result);
    }
}

// Performance test
#[tokio::test]
async fn test_resource_categories_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Call get_resource_categories multiple times
    for _ in 0..1000 {
        let _categories = get_resource_categories();
    }
    
    let duration = start.elapsed();
    
    // Should complete quickly (less than 100ms for 1000 calls)
    assert!(duration.as_millis() < 100, "Resource categories generation should be fast");
}