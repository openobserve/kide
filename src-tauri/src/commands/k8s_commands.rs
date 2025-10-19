use tauri::{AppHandle, State};
use crate::k8s::{K8sContext, get_resource_categories, K8sResourceCategory};
use crate::state::AppState;
use crate::commands::command_wrapper::*;

#[tauri::command]
pub async fn connect_k8s(state: State<'_, AppState>) -> Result<(), String> {
    let command = ConnectK8sCommand;
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn connect_k8s_with_context(state: State<'_, AppState>, context_name: String) -> Result<(), String> {
    let command = ConnectK8sWithContextCommand { context_name };
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn get_k8s_contexts(state: State<'_, AppState>) -> Result<Vec<K8sContext>, String> {
    let command = GetK8sContextsCommand;
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn get_current_k8s_context(state: State<'_, AppState>) -> Result<String, String> {
    let command = GetCurrentK8sContextCommand;
    to_tauri_result(execute_state_command(&state, command).await)
}

#[tauri::command]
pub async fn get_resources() -> Result<Vec<K8sResourceCategory>, String> {
    Ok(get_resource_categories())
}

#[tauri::command]
pub async fn get_namespaces(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let command = GetNamespacesCommand;
    to_tauri_result(execute_k8s_command(&state, command).await)
}

#[tauri::command]
pub async fn start_resource_watch(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    resource_type: String,
    namespaces: Option<Vec<String>>,
) -> Result<(), String> {
    let manager_lock = state.watch_manager.lock().await;
    if let Some(manager) = manager_lock.as_ref() {
        manager
            .start_watch(app_handle, &resource_type, namespaces)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        return Err("K8s client not connected".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_resource_watch(
    state: State<'_, AppState>,
    resource_type: String,
    namespaces: Option<Vec<String>>,
) -> Result<(), String> {
    let manager_lock = state.watch_manager.lock().await;
    if let Some(manager) = manager_lock.as_ref() {
        manager
            .stop_watch(&resource_type, namespaces)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_pod_logs(
    state: State<'_, AppState>,
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
    lines: Option<i64>,
) -> Result<String, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, LogParams};
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);
    
    let mut log_params = LogParams {
        follow: false,
        timestamps: true,
        ..Default::default()
    };
    
    if let Some(container) = container_name {
        log_params.container = Some(container);
    }
    
    if let Some(tail_lines) = lines {
        log_params.tail_lines = Some(tail_lines);
    }
    
    let logs = api
        .logs(&pod_name, &log_params)
        .await
        .map_err(|e| format!("Failed to fetch logs: {}", e))?;
    
    Ok(logs)
}

#[tauri::command]
pub async fn start_pod_logs_stream(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
) -> Result<String, String> {
    let manager_lock = state.log_stream_manager.lock().await;
    if let Some(manager) = manager_lock.as_ref() {
        let stream_id = manager
            .start_log_stream(app_handle, pod_name, namespace, container_name)
            .await
            .map_err(|e| e.to_string())?;
        Ok(stream_id)
    } else {
        Err("K8s client not connected".to_string())
    }
}

#[tauri::command]
pub async fn stop_pod_logs_stream(
    state: State<'_, AppState>,
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
) -> Result<(), String> {
    let stream_id = format!(
        "{}:{}:{}",
        namespace,
        pod_name,
        container_name.as_ref().unwrap_or(&"default".to_string())
    );
    
    let manager_lock = state.log_stream_manager.lock().await;
    if let Some(manager) = manager_lock.as_ref() {
        manager
            .stop_log_stream(&stream_id)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_resource_events(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    use k8s_openapi::api::core::v1::Event;
    use kube::api::{Api, ListParams};
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    let api: Api<Event> = if let Some(ns) = namespace {
        Api::namespaced(client, &ns)
    } else {
        Api::all(client)
    };
    
    let list_params = ListParams {
        field_selector: Some(format!("involvedObject.name={},involvedObject.kind={}", resource_name, resource_kind)),
        ..Default::default()
    };
    
    let events = api.list(&list_params)
        .await
        .map_err(|e| format!("Failed to fetch events: {}", e))?;
    
    let mut event_list: Vec<serde_json::Value> = events
        .items
        .into_iter()
        .map(|event| {
            serde_json::json!({
                "uid": event.metadata.uid.unwrap_or_default(),
                "type": event.type_.unwrap_or_default(),
                "reason": event.reason.unwrap_or_default(),
                "message": event.message.unwrap_or_default(),
                "source": {
                    "component": event.source.and_then(|s| s.component)
                },
                "firstTimestamp": event.first_timestamp.map(|t| t.0.to_rfc3339()),
                "lastTimestamp": event.last_timestamp.map(|t| t.0.to_rfc3339()),
                "count": event.count.unwrap_or(1)
            })
        })
        .collect();
    
    // Sort events by timestamp (most recent first)
    event_list.sort_by(|a, b| {
        // Use lastTimestamp if available, otherwise fallback to firstTimestamp
        let a_time = a["lastTimestamp"].as_str()
            .or_else(|| a["firstTimestamp"].as_str())
            .unwrap_or("");
        let b_time = b["lastTimestamp"].as_str()
            .or_else(|| b["firstTimestamp"].as_str())
            .unwrap_or("");
        
        // Sort in reverse chronological order (most recent first)
        b_time.cmp(a_time)
    });
    
    Ok(event_list)
}

// ===== SHARED CACHE COMMANDS =====

#[tauri::command]
pub async fn subscribe_to_resources(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    resource_type: String,
    namespace: Option<String>,
    immediate_fetch: Option<bool>,
) -> Result<Vec<crate::k8s::K8sListItem>, String> {
    use crate::k8s::{WatchScope, K8sClient};
    
    let cache_lock = state.shared_cache.lock().await;
    if let Some(cache) = cache_lock.as_ref() {
        // Get current cluster context
        let cluster_context = K8sClient::get_current_context()
            .await
            .unwrap_or_else(|_| "default".to_string());
        
        let scope = WatchScope::new(cluster_context)
            .with_namespace(namespace);
        
        let immediate = immediate_fetch.unwrap_or(false);
        cache.subscribe(app_handle, resource_type, scope, immediate)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Shared cache not initialized".to_string())
    }
}

#[tauri::command]
pub async fn unsubscribe_from_resources(
    state: State<'_, AppState>,
    resource_type: String,
    namespace: Option<String>,
) -> Result<(), String> {
    use crate::k8s::{WatchScope, K8sClient};
    
    let cache_lock = state.shared_cache.lock().await;
    if let Some(cache) = cache_lock.as_ref() {
        // Get current cluster context
        let cluster_context = K8sClient::get_current_context()
            .await
            .unwrap_or_else(|_| "default".to_string());
        
        let scope = WatchScope::new(cluster_context)
            .with_namespace(namespace);
        
        cache.unsubscribe(resource_type, scope).await;
        Ok(())
    } else {
        Err("Shared cache not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_cached_resources(
    state: State<'_, AppState>,
    resource_type: String,
    namespace: Option<String>,
) -> Result<Vec<crate::k8s::K8sListItem>, String> {
    use crate::k8s::{WatchScope, K8sClient};
    
    let cache_lock = state.shared_cache.lock().await;
    if let Some(cache) = cache_lock.as_ref() {
        // Get current cluster context
        let cluster_context = K8sClient::get_current_context()
            .await
            .unwrap_or_else(|_| "default".to_string());
        
        let scope = WatchScope::new(cluster_context)
            .with_namespace(namespace);
        
        Ok(cache.get_cached_data(resource_type, scope)
            .await
            .unwrap_or_default())
    } else {
        Err("Shared cache not initialized".to_string())
    }
}