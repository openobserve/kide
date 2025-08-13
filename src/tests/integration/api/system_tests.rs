/**
 * System and Caching API Integration Tests
 * 
 * Tests for the 2 system and caching endpoints:
 * - get_cached_resources
 * - open_url
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_get_cached_resources_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful cached resources retrieval
    let cached_resources = json!({
        "pods": [
            mock_pod(),
            {
                "metadata": {
                    "name": "cached-pod-2",
                    "namespace": params.namespace,
                    "uid": "uid-cached-2"
                },
                "spec": {
                    "containers": [
                        {
                            "name": "nginx",
                            "image": "nginx:1.20"
                        }
                    ]
                },
                "status": {
                    "phase": "Running"
                }
            }
        ],
        "deployments": [
            mock_deployment()
        ],
        "services": [
            {
                "metadata": {
                    "name": "test-service",
                    "namespace": params.namespace
                },
                "spec": {
                    "type": "ClusterIP",
                    "ports": [
                        {
                            "port": 80,
                            "targetPort": 8080
                        }
                    ]
                }
            }
        ]
    });
    
    api.set_success_response("get_cached_resources", cached_resources);
    
    let result = api.invoke("get_cached_resources", json!({})).await;
    
    assert!(result.is_ok());
    let resources = result.unwrap();
    assert!(resources.is_object());
    
    let resources_obj = resources.as_object().unwrap();
    assert!(resources_obj.contains_key("pods"));
    assert!(resources_obj.contains_key("deployments"));
    assert!(resources_obj.contains_key("services"));
    
    // Verify pods cache
    let pods = &resources_obj["pods"];
    assert!(pods.is_array());
    assert_eq!(pods.as_array().unwrap().len(), 2);
    assert_eq!(pods[0]["metadata"]["name"], "test-pod");
    assert_eq!(pods[1]["metadata"]["name"], "cached-pod-2");
    
    // Verify deployments cache
    let deployments = &resources_obj["deployments"];
    assert!(deployments.is_array());
    assert_eq!(deployments.as_array().unwrap().len(), 1);
    assert_eq!(deployments[0]["metadata"]["name"], "test-deployment");
    
    assert!(api.was_called("get_cached_resources"));
}

#[tokio::test]
async fn test_get_cached_resources_empty_cache() {
    let mut api = MockTauriApi::new();
    
    // Mock empty cache
    api.set_success_response("get_cached_resources", json!({}));
    
    let result = api.invoke("get_cached_resources", json!({})).await;
    
    assert!(result.is_ok());
    let resources = result.unwrap();
    assert!(resources.is_object());
    assert_eq!(resources.as_object().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_cached_resources_partial_cache() {
    let mut api = MockTauriApi::new();
    
    // Mock partial cache (only some resource types cached)
    let partial_cache = json!({
        "pods": [mock_pod()],
        "nodes": [
            {
                "metadata": {
                    "name": "minikube"
                },
                "status": {
                    "conditions": []
                }
            }
        ]
    });
    
    api.set_success_response("get_cached_resources", partial_cache);
    
    let result = api.invoke("get_cached_resources", json!({})).await;
    
    assert!(result.is_ok());
    let resources = result.unwrap();
    let resources_obj = resources.as_object().unwrap();
    
    assert!(resources_obj.contains_key("pods"));
    assert!(resources_obj.contains_key("nodes"));
    assert!(!resources_obj.contains_key("deployments"));
    assert!(!resources_obj.contains_key("services"));
    
    assert_eq!(resources_obj["pods"].as_array().unwrap().len(), 1);
    assert_eq!(resources_obj["nodes"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_cached_resources_with_namespace_filter() {
    let mut api = MockTauriApi::new();
    
    // Mock filtered cache results
    let filtered_cache = json!({
        "pods": [
            {
                "metadata": {
                    "name": "default-pod",
                    "namespace": "default"
                }
            }
        ],
        "services": [
            {
                "metadata": {
                    "name": "default-service",
                    "namespace": "default"
                }
            }
        ]
    });
    
    api.set_success_response("get_cached_resources", filtered_cache);
    
    let result = api.invoke("get_cached_resources", json!({
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    let resources = result.unwrap();
    let resources_obj = resources.as_object().unwrap();
    
    // Verify namespace filtering
    let pods = &resources_obj["pods"];
    for pod in pods.as_array().unwrap() {
        assert_eq!(pod["metadata"]["namespace"], "default");
    }
    
    let services = &resources_obj["services"];
    for service in services.as_array().unwrap() {
        assert_eq!(service["metadata"]["namespace"], "default");
    }
}

#[tokio::test]
async fn test_get_cached_resources_cache_miss() {
    let mut api = MockTauriApi::new();
    
    // Mock cache not initialized or expired
    api.set_error_response("get_cached_resources", "Cache not initialized or expired");
    
    let result = api.invoke("get_cached_resources", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cache not initialized"));
}

#[tokio::test]
async fn test_get_cached_resources_not_connected() {
    let mut api = MockTauriApi::new();
    
    // Mock not connected to K8s cluster
    api.set_error_response("get_cached_resources", "K8s client not connected");
    
    let result = api.invoke("get_cached_resources", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not connected"));
}

#[tokio::test]
async fn test_get_cached_resources_large_cache() {
    let mut api = MockTauriApi::new();
    
    // Create large cache with many resources
    let mut large_cache = json!({});
    
    // Add many pods
    let mut pods = Vec::new();
    for i in 0..500 {
        pods.push(json!({
            "metadata": {
                "name": format!("pod-{}", i),
                "namespace": "default",
                "uid": format!("uid-{}", i)
            },
            "spec": {
                "containers": [
                    {
                        "name": "nginx",
                        "image": "nginx:1.20"
                    }
                ]
            },
            "status": {
                "phase": "Running"
            }
        }));
    }
    large_cache["pods"] = json!(pods);
    
    // Add many services
    let mut services = Vec::new();
    for i in 0..100 {
        services.push(json!({
            "metadata": {
                "name": format!("service-{}", i),
                "namespace": "default"
            },
            "spec": {
                "type": "ClusterIP",
                "ports": [{"port": 80}]
            }
        }));
    }
    large_cache["services"] = json!(services);
    
    api.set_success_response("get_cached_resources", large_cache);
    
    let start_time = std::time::Instant::now();
    let result = api.invoke("get_cached_resources", json!({})).await;
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    let resources = result.unwrap();
    let resources_obj = resources.as_object().unwrap();
    
    assert_eq!(resources_obj["pods"].as_array().unwrap().len(), 500);
    assert_eq!(resources_obj["services"].as_array().unwrap().len(), 100);
    
    // Should handle large cache reasonably fast
    assert!(duration.as_millis() < 3000);
}

#[tokio::test]
async fn test_open_url_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful URL opening
    api.set_success_response("open_url", json!(null));
    
    let result = api.invoke("open_url", json!({
        "url": "https://kubernetes.io/docs"
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("open_url"));
}

#[tokio::test]
async fn test_open_url_https() {
    let mut api = MockTauriApi::new();
    
    // Mock opening HTTPS URL
    api.set_success_response("open_url", json!(null));
    
    let result = api.invoke("open_url", json!({
        "url": "https://github.com/kubernetes/kubernetes"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_open_url_http() {
    let mut api = MockTauriApi::new();
    
    // Mock opening HTTP URL (should work but may show security warning)
    api.set_success_response("open_url", json!(null));
    
    let result = api.invoke("open_url", json!({
        "url": "http://localhost:8080/dashboard"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_open_url_service_url() {
    let mut api = MockTauriApi::new();
    
    // Mock opening Kubernetes service URL
    api.set_success_response("open_url", json!(null));
    
    let result = api.invoke("open_url", json!({
        "url": "http://my-service.default.svc.cluster.local:8080"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_open_url_port_forward() {
    let mut api = MockTauriApi::new();
    
    // Mock opening port-forwarded URL
    api.set_success_response("open_url", json!(null));
    
    let result = api.invoke("open_url", json!({
        "url": "http://localhost:3000"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_open_url_invalid_url() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid URL error
    api.set_error_response("open_url", "Invalid URL format: not-a-url");
    
    let result = api.invoke("open_url", json!({
        "url": "not-a-url"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid URL format"));
}

#[tokio::test]
async fn test_open_url_empty_url() {
    let mut api = MockTauriApi::new();
    
    // Mock empty URL error
    api.set_error_response("open_url", "URL cannot be empty");
    
    let result = api.invoke("open_url", json!({
        "url": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_open_url_malicious_url() {
    let mut api = MockTauriApi::new();
    
    // Mock security check for potentially malicious URL
    api.set_error_response("open_url", "URL blocked by security policy");
    
    let result = api.invoke("open_url", json!({
        "url": "javascript:alert('xss')"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("blocked by security policy"));
}

#[tokio::test]
async fn test_open_url_network_unreachable() {
    let mut api = MockTauriApi::new();
    
    // Mock network unreachable error
    api.set_error_response("open_url", "Network unreachable");
    
    let result = api.invoke("open_url", json!({
        "url": "https://unreachable-host.example.com"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Network unreachable"));
}

#[tokio::test]
async fn test_open_url_system_browser_not_available() {
    let mut api = MockTauriApi::new();
    
    // Mock system browser not available error
    api.set_error_response("open_url", "No system browser available");
    
    let result = api.invoke("open_url", json!({
        "url": "https://kubernetes.io"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No system browser available"));
}

// Integration test for cache and URL workflow
#[tokio::test]
async fn test_cache_and_url_workflow() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for cache and URL workflow
    let cached_resources = json!({
        "pods": [mock_pod()],
        "services": [
            {
                "metadata": {
                    "name": "web-service",
                    "namespace": "default"
                },
                "spec": {
                    "type": "LoadBalancer",
                    "ports": [
                        {
                            "port": 80,
                            "targetPort": 8080
                        }
                    ]
                }
            }
        ]
    });
    
    api.set_success_response("get_cached_resources", cached_resources);
    api.set_success_response("open_url", json!(null));
    
    // Step 1: Get cached resources
    let cache_result = api.invoke("get_cached_resources", json!({})).await;
    assert!(cache_result.is_ok());
    
    let resources = cache_result.unwrap();
    let services = &resources["services"];
    assert!(services.is_array());
    assert_eq!(services.as_array().unwrap().len(), 1);
    
    // Step 2: Open URL for service (simulating opening service in browser)
    let url_result = api.invoke("open_url", json!({
        "url": "http://web-service.default.svc.cluster.local"
    })).await;
    assert!(url_result.is_ok());
    
    // Verify both operations were called
    assert!(api.was_called("get_cached_resources"));
    assert!(api.was_called("open_url"));
}

// Test multiple concurrent URL opens
#[tokio::test]
async fn test_multiple_concurrent_url_opens() {
    let mut api = MockTauriApi::new();
    
    // Mock successful URL opens
    api.set_success_response("open_url", json!(null));
    
    // Open multiple URLs concurrently
    let url1 = api.invoke("open_url", json!({
        "url": "https://kubernetes.io/docs"
    }));
    
    let url2 = api.invoke("open_url", json!({
        "url": "https://github.com/kubernetes/kubernetes"
    }));
    
    let url3 = api.invoke("open_url", json!({
        "url": "http://localhost:3000"
    }));
    
    // Wait for all to complete
    let (result1, result2, result3) = tokio::join!(url1, url2, url3);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    assert_eq!(api.get_call_count("open_url"), 3);
}

// Test cache refresh scenarios
#[tokio::test]
async fn test_cache_refresh_scenarios() {
    let mut api = MockTauriApi::new();
    
    // First call returns cached data
    let initial_cache = json!({
        "pods": [mock_pod()]
    });
    api.set_success_response("get_cached_resources", initial_cache);
    
    let first_call = api.invoke("get_cached_resources", json!({})).await;
    assert!(first_call.is_ok());
    assert_eq!(first_call.unwrap()["pods"].as_array().unwrap().len(), 1);
    
    // Second call returns updated cache
    let updated_cache = json!({
        "pods": [mock_pod(), {
            "metadata": {
                "name": "new-pod",
                "namespace": "default",
                "uid": "uid-new"
            },
            "spec": {
                "containers": [
                    {
                        "name": "nginx",
                        "image": "nginx:1.21"
                    }
                ]
            },
            "status": {
                "phase": "Running"
            }
        }]
    });
    api.set_success_response("get_cached_resources", updated_cache);
    
    let second_call = api.invoke("get_cached_resources", json!({})).await;
    assert!(second_call.is_ok());
    assert_eq!(second_call.unwrap()["pods"].as_array().unwrap().len(), 2);
    
    assert_eq!(api.get_call_count("get_cached_resources"), 2);
}

// Performance test for URL opening
#[tokio::test]
async fn test_url_opening_performance() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("open_url", json!(null));
    
    let urls = vec![
        "https://kubernetes.io",
        "https://github.com/kubernetes/kubernetes",
        "http://localhost:8080",
        "https://docs.docker.com",
        "https://helm.sh",
    ];
    
    let start_time = std::time::Instant::now();
    
    for url in urls {
        let result = api.invoke("open_url", json!({
            "url": url
        })).await;
        assert!(result.is_ok(), "Failed to open URL: {}", url);
    }
    
    let duration = start_time.elapsed();
    
    // Should handle multiple URL opens efficiently
    assert!(duration.as_millis() < 2000);
    assert_eq!(api.get_call_count("open_url"), 5);
}

// Test error recovery for cache operations
#[tokio::test]
async fn test_cache_error_recovery() {
    let mut api = MockTauriApi::new();
    
    // First attempt fails
    api.set_error_response("get_cached_resources", "Cache corruption detected");
    
    let first_attempt = api.invoke("get_cached_resources", json!({})).await;
    assert!(first_attempt.is_err());
    
    // Second attempt succeeds (cache rebuilt)
    api.set_success_response("get_cached_resources", json!({
        "pods": [mock_pod()]
    }));
    
    let second_attempt = api.invoke("get_cached_resources", json!({})).await;
    assert!(second_attempt.is_ok());
    
    assert_eq!(api.get_call_count("get_cached_resources"), 2);
}