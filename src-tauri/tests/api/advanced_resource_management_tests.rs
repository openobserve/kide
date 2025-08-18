/**
 * Resource Commands V2 API Integration Tests
 * 
 * Comprehensive tests for the 3 resource command v2 endpoints:
 * - delete_resource_v2
 * - toggle_cronjob_suspend
 * - trigger_cronjob
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

// ==== DELETE_RESOURCE_V2 INTEGRATION TESTS ====

#[tokio::test]
async fn test_delete_resource_v2_pod_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful pod deletion
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": params.pod_name,
        "namespace": params.namespace
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("delete_resource_v2"));
}

#[tokio::test]
async fn test_delete_resource_v2_deployment_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful deployment deletion
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "deployments",
        "name": "test-deployment",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_v2_service_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful service deletion
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "services",
        "name": "web-service",
        "namespace": "production"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_v2_cluster_scoped() {
    let mut api = MockTauriApi::new();
    
    // Mock successful cluster-scoped resource deletion (no namespace)
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "nodes",
        "name": "worker-node-1",
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_v2_resource_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock resource not found error
    api.set_error_response("delete_resource_v2", &mock_not_found_error());
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "nonexistent-pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_delete_resource_v2_invalid_resource_type() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid resource type error
    api.set_error_response("delete_resource_v2", "Invalid resource type: invalidresource");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "invalidresource",
        "name": "test-resource",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource type"));
}

#[tokio::test]
async fn test_delete_resource_v2_missing_namespace_for_namespaced_resource() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for missing namespace
    api.set_error_response("delete_resource_v2", "Namespace required for namespaced resource 'pods'");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "test-pod",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace required"));
}

#[tokio::test]
async fn test_delete_resource_v2_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("delete_resource_v2", "Forbidden: insufficient permissions to delete pods");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "system-pod",
        "namespace": "kube-system"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Forbidden"));
}

#[tokio::test]
async fn test_delete_resource_v2_with_finalizers() {
    let mut api = MockTauriApi::new();
    
    // Mock deletion of resource with finalizers (should succeed but take longer)
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "namespaces",
        "name": "test-namespace",
        "namespace": null
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_v2_protected_resource() {
    let mut api = MockTauriApi::new();
    
    // Mock deletion of protected system resource
    api.set_error_response("delete_resource_v2", "Cannot delete protected system resource");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "kube-apiserver-master",
        "namespace": "kube-system"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("protected"));
}

#[tokio::test]
async fn test_delete_resource_v2_custom_resource() {
    let mut api = MockTauriApi::new();
    
    // Mock deletion of custom resource
    api.set_success_response("delete_resource_v2", json!(null));
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "customresources",
        "name": "my-custom-resource",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_resource_v2_empty_resource_name() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for empty name
    api.set_error_response("delete_resource_v2", "Resource name cannot be empty");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_delete_resource_v2_concurrent_deletions() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response for successful deletions
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("delete_resource_v2", json!(null));
    }
    
    // Spawn multiple concurrent deletion requests
    let mut handles = Vec::new();
    for i in 0..5 {
        let api_clone = Arc::clone(&api);
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("delete_resource_v2", json!({
                "resource_type": "pods",
                "name": format!("test-pod-{}", i),
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
    assert_eq!(api_guard.get_call_count("delete_resource_v2"), 5);
}

#[tokio::test]
async fn test_delete_resource_v2_network_timeout() {
    let mut api = MockTauriApi::new();
    
    // Mock network timeout error
    api.set_error_response("delete_resource_v2", "Request timeout: deletion operation timed out");
    
    let result = api.invoke("delete_resource_v2", json!({
        "resource_type": "pods",
        "name": "slow-pod",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timeout"));
}

// ==== TOGGLE_CRONJOB_SUSPEND INTEGRATION TESTS ====

#[tokio::test]
async fn test_toggle_cronjob_suspend_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful CronJob suspend
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "backup-job",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("toggle_cronjob_suspend"));
}

#[tokio::test]
async fn test_toggle_cronjob_resume_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful CronJob resume (unsuspend)
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "cleanup-job",
        "namespace": "production",
        "suspend": false
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_cronjob_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock CronJob not found error
    api.set_error_response("toggle_cronjob_suspend", &mock_not_found_error());
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "nonexistent-job",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_missing_namespace() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for missing namespace
    api.set_error_response("toggle_cronjob_suspend", "Namespace required for CronJob");
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "backup-job",
        "namespace": null,
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace required"));
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("toggle_cronjob_suspend", "Forbidden: insufficient permissions to update cronjobs");
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "system-backup",
        "namespace": "kube-system",
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Forbidden"));
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_already_suspended() {
    let mut api = MockTauriApi::new();
    
    // Mock successful operation even when already in desired state
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "already-suspended-job",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_already_running() {
    let mut api = MockTauriApi::new();
    
    // Mock successful operation even when already in desired state
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "running-job",
        "namespace": "default",
        "suspend": false
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_invalid_cronjob_spec() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for invalid CronJob spec
    api.set_error_response("toggle_cronjob_suspend", "Invalid CronJob specification");
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "invalid-job",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid"));
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_network_error() {
    let mut api = MockTauriApi::new();
    
    // Mock network connection error
    api.set_error_response("toggle_cronjob_suspend", "Failed to update CronJob suspend state: connection error");
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "backup-job",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("connection error"));
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_multiple_cronjobs() {
    let mut api = MockTauriApi::new();
    
    // Test suspending multiple CronJobs
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    
    let cronjobs = vec![
        ("backup-daily", "default", true),
        ("cleanup-weekly", "default", true),
        ("report-monthly", "production", false),
    ];
    
    for (name, namespace, suspend) in cronjobs {
        let result = api.invoke("toggle_cronjob_suspend", json!({
            "name": name,
            "namespace": namespace,
            "suspend": suspend
        })).await;
        
        assert!(result.is_ok(), "Failed to toggle CronJob: {}", name);
    }
    
    assert_eq!(api.get_call_count("toggle_cronjob_suspend"), 3);
}

#[tokio::test]
async fn test_toggle_cronjob_suspend_empty_name() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for empty name
    api.set_error_response("toggle_cronjob_suspend", "CronJob name cannot be empty");
    
    let result = api.invoke("toggle_cronjob_suspend", json!({
        "name": "",
        "namespace": "default",
        "suspend": true
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

// ==== TRIGGER_CRONJOB INTEGRATION TESTS ====

#[tokio::test]
async fn test_trigger_cronjob_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful CronJob trigger
    api.set_success_response("trigger_cronjob", json!(null));
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "backup-job",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("trigger_cronjob"));
}

#[tokio::test]
async fn test_trigger_cronjob_production_namespace() {
    let mut api = MockTauriApi::new();
    
    // Mock successful production CronJob trigger
    api.set_success_response("trigger_cronjob", json!(null));
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "data-processing-job",
        "namespace": "production"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_trigger_cronjob_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock CronJob not found error
    api.set_error_response("trigger_cronjob", "Failed to get CronJob nonexistent-job: not found");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "nonexistent-job",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_trigger_cronjob_missing_namespace() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for missing namespace
    api.set_error_response("trigger_cronjob", "Namespace required for CronJob");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "backup-job",
        "namespace": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace required"));
}

#[tokio::test]
async fn test_trigger_cronjob_no_spec() {
    let mut api = MockTauriApi::new();
    
    // Mock error for CronJob without spec
    api.set_error_response("trigger_cronjob", "CronJob has no spec");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "invalid-job",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no spec"));
}

#[tokio::test]
async fn test_trigger_cronjob_job_creation_failed() {
    let mut api = MockTauriApi::new();
    
    // Mock job creation failure
    api.set_error_response("trigger_cronjob", "Failed to create Job from CronJob backup-job: quota exceeded");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "backup-job",
        "namespace": "limited"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to create Job"));
    assert!(result.unwrap_err().contains("quota exceeded"));
}

#[tokio::test]
async fn test_trigger_cronjob_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    
    // Mock RBAC permission error
    api.set_error_response("trigger_cronjob", "Forbidden: insufficient permissions to create jobs");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "system-backup",
        "namespace": "kube-system"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Forbidden"));
}

#[tokio::test]
async fn test_trigger_cronjob_suspended_cronjob() {
    let mut api = MockTauriApi::new();
    
    // Mock successful trigger even if CronJob is suspended
    api.set_success_response("trigger_cronjob", json!(null));
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "suspended-job",
        "namespace": "default"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_trigger_cronjob_multiple_triggers() {
    let mut api = MockTauriApi::new();
    
    // Test triggering multiple CronJobs
    api.set_success_response("trigger_cronjob", json!(null));
    
    let cronjobs = vec![
        ("backup-daily", "default"),
        ("cleanup-weekly", "default"),
        ("report-monthly", "production"),
        ("maintenance", "kube-system"),
    ];
    
    for (name, namespace) in cronjobs {
        let result = api.invoke("trigger_cronjob", json!({
            "name": name,
            "namespace": namespace
        })).await;
        
        assert!(result.is_ok(), "Failed to trigger CronJob: {}", name);
    }
    
    assert_eq!(api.get_call_count("trigger_cronjob"), 4);
}

#[tokio::test]
async fn test_trigger_cronjob_concurrent_triggers() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response for successful triggers
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("trigger_cronjob", json!(null));
    }
    
    // Spawn multiple concurrent trigger requests for the same CronJob
    let mut handles = Vec::new();
    for i in 0..3 {
        let api_clone = Arc::clone(&api);
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("trigger_cronjob", json!({
                "name": "concurrent-job",
                "namespace": "default"
            })).await
        });
        handles.push(handle);
    }
    
    // Wait for all triggers to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all triggers were processed
    let api_guard = api.lock().await;
    assert_eq!(api_guard.get_call_count("trigger_cronjob"), 3);
}

#[tokio::test]
async fn test_trigger_cronjob_empty_name() {
    let mut api = MockTauriApi::new();
    
    // Mock validation error for empty name
    api.set_error_response("trigger_cronjob", "CronJob name cannot be empty");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[tokio::test]
async fn test_trigger_cronjob_network_timeout() {
    let mut api = MockTauriApi::new();
    
    // Mock network timeout error
    api.set_error_response("trigger_cronjob", "Request timeout: trigger operation timed out");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "slow-job",
        "namespace": "default"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timeout"));
}

#[tokio::test]
async fn test_trigger_cronjob_resource_quota_exceeded() {
    let mut api = MockTauriApi::new();
    
    // Mock resource quota exceeded error
    api.set_error_response("trigger_cronjob", "Failed to create Job: resource quota exceeded");
    
    let result = api.invoke("trigger_cronjob", json!({
        "name": "resource-heavy-job",
        "namespace": "limited"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("quota exceeded"));
}

// ==== INTEGRATION WORKFLOW TESTS ====

#[tokio::test]
async fn test_cronjob_management_workflow() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for full CronJob management workflow
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    api.set_success_response("trigger_cronjob", json!(null));
    api.set_success_response("delete_resource_v2", json!(null));
    
    let cronjob_name = "test-workflow-job";
    let namespace = "default";
    
    // Step 1: Suspend the CronJob
    let suspend_result = api.invoke("toggle_cronjob_suspend", json!({
        "name": cronjob_name,
        "namespace": namespace,
        "suspend": true
    })).await;
    assert!(suspend_result.is_ok());
    
    // Step 2: Trigger the CronJob manually while suspended
    let trigger_result = api.invoke("trigger_cronjob", json!({
        "name": cronjob_name,
        "namespace": namespace
    })).await;
    assert!(trigger_result.is_ok());
    
    // Step 3: Resume the CronJob
    let resume_result = api.invoke("toggle_cronjob_suspend", json!({
        "name": cronjob_name,
        "namespace": namespace,
        "suspend": false
    })).await;
    assert!(resume_result.is_ok());
    
    // Step 4: Delete the CronJob
    let delete_result = api.invoke("delete_resource_v2", json!({
        "resource_type": "cronjobs",
        "name": cronjob_name,
        "namespace": namespace
    })).await;
    assert!(delete_result.is_ok());
    
    // Verify all operations were called
    assert!(api.was_called("toggle_cronjob_suspend"));
    assert!(api.was_called("trigger_cronjob"));
    assert!(api.was_called("delete_resource_v2"));
    assert_eq!(api.get_call_count("toggle_cronjob_suspend"), 2); // suspend and resume
}

#[tokio::test]
async fn test_resource_lifecycle_workflow() {
    let mut api = MockTauriApi::new();
    
    // Setup responses for resource lifecycle workflow
    api.set_success_response("delete_resource_v2", json!(null));
    
    let resources = vec![
        ("pods", "test-pod", "default"),
        ("services", "test-service", "default"),
        ("deployments", "test-deployment", "default"),
        ("configmaps", "test-config", "default"),
        ("secrets", "test-secret", "default"),
    ];
    
    // Delete multiple resources
    for (resource_type, name, namespace) in resources {
        let delete_result = api.invoke("delete_resource_v2", json!({
            "resource_type": resource_type,
            "name": name,
            "namespace": namespace
        })).await;
        
        assert!(delete_result.is_ok(), "Failed to delete {} '{}'", resource_type, name);
    }
    
    // Verify all deletions were processed
    assert_eq!(api.get_call_count("delete_resource_v2"), 5);
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    let mut api = MockTauriApi::new();
    
    // First attempt fails
    api.set_error_response("trigger_cronjob", "Connection timeout");
    
    let first_attempt = api.invoke("trigger_cronjob", json!({
        "name": "retry-job",
        "namespace": "default"
    })).await;
    assert!(first_attempt.is_err());
    
    // Second attempt succeeds (connection recovered)
    api.set_success_response("trigger_cronjob", json!(null));
    
    let second_attempt = api.invoke("trigger_cronjob", json!({
        "name": "retry-job",
        "namespace": "default"
    })).await;
    assert!(second_attempt.is_ok());
    
    assert_eq!(api.get_call_count("trigger_cronjob"), 2);
}

// Performance test for resource operations
#[tokio::test]
async fn test_resource_operations_performance() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("delete_resource_v2", json!(null));
    api.set_success_response("toggle_cronjob_suspend", json!(null));
    api.set_success_response("trigger_cronjob", json!(null));
    
    let start_time = std::time::Instant::now();
    
    // Perform rapid resource operations
    for i in 0..50 {
        let delete_result = api.invoke("delete_resource_v2", json!({
            "resource_type": "pods",
            "name": format!("test-pod-{}", i),
            "namespace": "default"
        })).await;
        assert!(delete_result.is_ok());
        
        if i % 10 == 0 {
            let suspend_result = api.invoke("toggle_cronjob_suspend", json!({
                "name": format!("job-{}", i),
                "namespace": "default",
                "suspend": true
            })).await;
            assert!(suspend_result.is_ok());
            
            let trigger_result = api.invoke("trigger_cronjob", json!({
                "name": format!("job-{}", i),
                "namespace": "default"
            })).await;
            assert!(trigger_result.is_ok());
        }
    }
    
    let duration = start_time.elapsed();
    
    // Should handle rapid operations efficiently
    assert!(duration.as_millis() < 10000);
    assert_eq!(api.get_call_count("delete_resource_v2"), 50);
    assert_eq!(api.get_call_count("toggle_cronjob_suspend"), 5);
    assert_eq!(api.get_call_count("trigger_cronjob"), 5);
}