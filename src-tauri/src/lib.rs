pub mod k8s;
pub mod security;
pub mod commands;
pub mod state;
pub mod environment;
pub mod errors;
pub mod cleanup;
pub mod shell_session;

use commands::*;
use state::*;
use environment::*;

pub fn run() {
    let config = KideConfig::default();
    
    // Setup enhanced environment before initializing Kubernetes client
    setup_environment(&config);
    
    // Run system diagnostics to check for file descriptor limits
    k8s::system_diagnostics();
    
    let app_state = AppState::with_config(config);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            connect_k8s,
            connect_k8s_with_context,
            get_k8s_contexts,
            get_current_k8s_context,
            get_resources,
            get_namespaces,
            start_resource_watch,
            stop_resource_watch,
            subscribe_to_resources,
            unsubscribe_from_resources,
            get_cached_resources,
            get_pod_logs,
            start_pod_logs_stream,
            stop_pod_logs_stream,
            get_resource_events,
            delete_resource,
            start_pod_shell,
            send_shell_input,
            resize_shell,
            stop_pod_shell,
            update_resource,
            scale_resource,
            get_pods_by_selector,
            get_node_pods,
            get_full_resource,
            open_url,
            toggle_cronjob_suspend,
            trigger_cronjob
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use rstest::rstest;
    use tokio_test;
    use crate::k8s::{get_resource_categories, K8sResource};

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        
        // Verify initial state using the new method
        let manager_lock = tokio_test::block_on(state.watch_manager.lock());
        assert!(manager_lock.is_none());
    }

    // Test resource validation
    #[rstest]
    #[case("pods", true)]
    #[case("nodes", true)]
    #[case("deployments", true)]
    #[case("invalid-resource", false)]
    fn test_resource_type_validation(#[case] resource_type: &str, #[case] should_be_valid: bool) {
        let categories = get_resource_categories();
        let all_resources: Vec<String> = categories
            .iter()
            .flat_map(|c| &c.resources)
            .map(|r| r.name.to_lowercase())
            .collect();
        
        let is_valid = all_resources.contains(&resource_type.to_string());
        assert_eq!(is_valid, should_be_valid, "Resource type '{}' validation failed", resource_type);
    }

    #[test]
    fn test_namespaced_resources_identification() {
        let categories = get_resource_categories();
        let all_resources: Vec<&K8sResource> = categories
            .iter()
            .flat_map(|c| &c.resources)
            .collect();
        
        // Test some known namespaced resources
        let pods = all_resources.iter().find(|r| r.name == "Pods").unwrap();
        assert!(pods.namespaced, "Pods should be namespaced");
        
        let deployments = all_resources.iter().find(|r| r.name == "Deployments").unwrap();
        assert!(deployments.namespaced, "Deployments should be namespaced");
        
        // Test some known cluster-wide resources
        let nodes = all_resources.iter().find(|r| r.name == "Nodes").unwrap();
        assert!(!nodes.namespaced, "Nodes should be cluster-wide");
        
        let namespaces = all_resources.iter().find(|r| r.name == "Namespaces").unwrap();
        assert!(!namespaces.namespaced, "Namespaces should be cluster-wide");
    }
}