//! Integration tests for the refactoring changes
//!
//! This module tests that our refactored components work together correctly.

use kide_lib::errors::{AppError, ConfigError, StateError, K8sError};
use kide_lib::k8s::{RESOURCE_REGISTRY, WatchLifecycleManager, WatchEventHandler, WatchDispatcher};
use kide_lib::commands::{K8sCommand, StateCommand};
use kide_lib::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_structured_errors_integration() {
    // Test that our error types work correctly
    let k8s_error = K8sError::InvalidResourceType {
        resource_type: "test-resource".to_string(),
    };
    let app_error = AppError::Kubernetes(k8s_error);
    
    let error_string = app_error.to_string();
    assert!(error_string.contains("Kubernetes error"));
    assert!(error_string.contains("test-resource"));
}

#[test]
fn test_resource_registry_integration() {
    // Test that resource registry works correctly
    let registry = &RESOURCE_REGISTRY;
    
    // Should have core resources registered
    assert!(registry.is_registered("pods"));
    assert!(registry.is_registered("deployments"));
    assert!(registry.is_registered("services"));
    assert!(!registry.is_registered("invalid-resource"));
    
    // Test handler retrieval
    let pod_handler = registry.get_handler("pods");
    assert!(pod_handler.is_ok());
    
    let handler = pod_handler.unwrap();
    let metadata = handler.metadata();
    assert_eq!(metadata.kind, "Pod");
    assert_eq!(metadata.api_version, "v1");
    assert!(metadata.is_namespaced);
    assert!(!metadata.scalable);
}

#[test]
fn test_resource_categories_integration() {
    let registry = &RESOURCE_REGISTRY;
    let categories = registry.get_resources_by_category();
    
    // Should have expected categories
    assert!(categories.contains_key("Workloads"));
    assert!(categories.contains_key("Services and Discovery"));
    assert!(categories.contains_key("Cluster"));
    
    // Workloads should contain expected resources
    let workloads = &categories["Workloads"];
    let workload_kinds: Vec<String> = workloads.iter().map(|r| r.kind.clone()).collect();
    assert!(workload_kinds.contains(&"Pod".to_string()));
    assert!(workload_kinds.contains(&"Deployment".to_string()));
}

#[tokio::test]
async fn test_command_wrapper_integration() {
    let state = AppState::new();
    
    // Test GetResourceCategoriesCommand
    let command = kide_lib::commands::resource_commands::GetResourceCategoriesCommand;
    let result = command.execute(&state).await;
    
    assert!(result.is_ok());
    let categories = result.unwrap();
    assert!(!categories.is_empty());
    
    // Categories should be properly structured
    let workloads_category = categories.iter()
        .find(|c| c.name == "Workloads")
        .expect("Should have Workloads category");
    
    assert!(!workloads_category.resources.is_empty());
    
    // Should have Pod resource
    let pod_resource = workloads_category.resources.iter()
        .find(|r| r.kind == "Pod")
        .expect("Should have Pod resource");
    
    assert_eq!(pod_resource.api_version, "v1");
    assert!(pod_resource.namespaced);
}

#[tokio::test]
async fn test_watch_components_integration() {
    // Test that watch components can be created and work together
    let lifecycle_manager = Arc::new(WatchLifecycleManager::new());
    let event_handler = Arc::new(WatchEventHandler::new());
    
    let dispatcher = WatchDispatcher::new(
        lifecycle_manager.clone(),
        event_handler,
    );
    
    // Should be able to get watch stats
    let stats = dispatcher.get_watch_stats().await;
    assert_eq!(stats.total_watches, 0); // No watches started yet
}

#[tokio::test]
async fn test_app_state_integration() {
    // Test that AppState can be created and used
    let state = AppState::new();
    
    // Should not be connected initially
    assert!(!state.k8s_client.is_connected().await);
    
    // Should have proper configuration
    assert!(state.config.connection_timeout.as_secs() > 0);
    assert!(state.config.max_fd_usage > 0);
}

#[test]
fn test_error_conversion_integration() {
    // Test error conversion chains work properly
    let k8s_error = K8sError::ResourceNotFound {
        resource_type: "Pod".to_string(),
        name: "test-pod".to_string(),
        namespace: Some("default".to_string()),
    };
    
    let app_error = AppError::Kubernetes(k8s_error);
    let string_result = kide_lib::commands::command_wrapper::to_tauri_result::<()>(Err(app_error));
    
    assert!(string_result.is_err());
    let error_message = string_result.unwrap_err();
    assert!(error_message.contains("Pod"));
    assert!(error_message.contains("test-pod"));
    assert!(error_message.contains("default"));
}

#[tokio::test]
async fn test_command_validation_integration() {
    // Test that command validation works across the system
    use kide_lib::commands::resource_commands::{ListResourcesCommand, GetResourceCommand};
    
    // Valid resource command should pass validation
    let valid_command = ListResourcesCommand {
        resource_type: "pods".to_string(),
        namespace: Some("default".to_string()),
        label_selector: None,
    };
    assert!(valid_command.validate().await.is_ok());
    
    // Invalid resource command should fail validation
    let invalid_command = ListResourcesCommand {
        resource_type: "invalid-resource".to_string(),
        namespace: None,
        label_selector: None,
    };
    assert!(invalid_command.validate().await.is_err());
    
    // Namespaced resource without namespace should fail
    let missing_namespace_command = GetResourceCommand {
        resource_type: "pods".to_string(),
        name: "test-pod".to_string(),
        namespace: None,
    };
    assert!(missing_namespace_command.validate().await.is_err());
    
    // Cluster-wide resource should not require namespace
    let cluster_command = GetResourceCommand {
        resource_type: "nodes".to_string(),
        name: "test-node".to_string(),
        namespace: None,
    };
    assert!(cluster_command.validate().await.is_ok());
}

#[test]
fn test_performance_improvements() {
    use std::time::Instant;
    
    // Test that resource lookup is fast (O(1) vs O(n))
    let start = Instant::now();
    
    // Perform many lookups
    for _ in 0..10000 {
        let _ = RESOURCE_REGISTRY.is_registered("pods");
        let _ = RESOURCE_REGISTRY.is_registered("deployments");
        let _ = RESOURCE_REGISTRY.is_registered("services");
        let _ = RESOURCE_REGISTRY.is_registered("invalid");
    }
    
    let duration = start.elapsed();
    
    // Should complete reasonably quickly (under 50ms for 40k lookups, accounting for lazy initialization)
    assert!(duration.as_millis() < 50, "Resource lookups should be fast, took {:?}", duration);
}

#[test]
fn test_type_safety_improvements() {
    // Test that we get compile-time type safety
    
    // Should be able to create specific error types
    let _connection_error = K8sError::connection_failed("test message");
    let _resource_error = K8sError::resource_not_found("Pod", "test-pod", Some("default".to_string()));
    let _watch_error = K8sError::watch_failed("pods", "test failure");
    
    // Error helpers should work
    // Error types are available for testing
    let _config_error = ConfigError::validation_failed("test_field", "test message");
    let _state_error = StateError::manager_not_initialized("WatchManager");
    
    // All these should compile without issues, demonstrating type safety
}

// Import the command modules to ensure they compile
#[allow(unused_imports)]
use kide_lib::commands::{resource_commands, k8s_commands};
#[allow(unused_imports)]
use kide_lib::k8s::{resource_registry, watch_components};