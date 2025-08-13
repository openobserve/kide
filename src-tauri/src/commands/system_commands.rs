use tauri::State;
use crate::k8s::{handle_resource_by_kind, ResourceOperation, ResourceParams};
use crate::state::AppState;
use std::process::Command;

#[tauri::command]
pub async fn get_full_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    let result = handle_resource_by_kind(
        &resource_kind,
        ResourceOperation::Get,
        client,
        &resource_name,
        namespace.as_deref(),
        None
    ).await?;
    
    result.ok_or_else(|| "No resource data returned".to_string())
}

#[tauri::command]
pub async fn delete_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
) -> Result<(), String> {
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    handle_resource_by_kind(
        &resource_kind,
        ResourceOperation::Delete,
        client,
        &resource_name,
        namespace.as_deref(),
        None
    ).await?;
    
    println!("🗑️ Successfully deleted {} '{}'{}",
        resource_kind,
        resource_name,
        namespace.map(|ns| format!(" in namespace '{}'", ns)).unwrap_or_default()
    );
    
    Ok(())
}

#[tauri::command]
pub async fn scale_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
    replicas: i32,
) -> Result<(), String> {
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    let params = ResourceParams {
        replicas: Some(replicas),
        yaml_content: None,
    };
    
    handle_resource_by_kind(
        &resource_kind,
        ResourceOperation::Scale,
        client,
        &resource_name,
        namespace.as_deref(),
        Some(params)
    ).await?;
    
    println!("✅ Successfully scaled {} '{}' to {} replicas", resource_kind, resource_name, replicas);
    
    // Force a brief delay to allow the Kubernetes API server to propagate the change
    // This helps ensure that subsequent watch events will include the updated status
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("🔄 Scaling operation completed, watch should capture status updates soon");
    Ok(())
}

#[tauri::command]
pub async fn update_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
    yaml_content: String,
) -> Result<(), String> {
    use kube::api::{Api, Patch, PatchParams};
    
    // Validate inputs
    state.input_sanitizer.validate_resource_name(&resource_name)
        .map_err(|e| format!("Invalid resource name: {}", e))?;
    
    if let Some(ref ns) = namespace {
        state.input_sanitizer.validate_namespace(ns)
            .map_err(|e| format!("Invalid namespace: {}", e))?;
    }
    
    state.input_sanitizer.validate_yaml_content(&yaml_content)
        .map_err(|e| format!("Invalid YAML content: {}", e))?;
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    // Parse the YAML content to validate it
    let _yaml_value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml_content)
        .map_err(|e| format!("Invalid YAML: {}", e))?;
    
    // Convert to JSON for Kubernetes API
    let mut json_value: serde_json::Value = serde_json::to_value(&_yaml_value)
        .map_err(|e| format!("Failed to convert YAML to JSON: {}", e))?;
    
    // Clean the JSON by removing system-managed fields that cannot be updated
    if let Some(metadata) = json_value.get_mut("metadata").and_then(|m| m.as_object_mut()) {
        // Remove fields that must be nil or are system-managed
        metadata.remove("managedFields");
        metadata.remove("resourceVersion");
        metadata.remove("generation");
        metadata.remove("uid");
        metadata.remove("selfLink");
        metadata.remove("creationTimestamp");
        metadata.remove("deletionTimestamp");
        metadata.remove("deletionGracePeriodSeconds");
        metadata.remove("finalizers");
        
        // Remove ownerReferences as they should not be modified during updates
        metadata.remove("ownerReferences");
    }
    
    // Remove status field as it's read-only and managed by controllers
    if json_value.get("status").is_some() {
        json_value.as_object_mut().unwrap().remove("status");
    }
    
    // TODO: Use generic update operation when implemented in resource_api
    // For now, handle specific resource types manually
    match resource_kind.as_str() {
        "Pod" => {
            use k8s_openapi::api::core::v1::Pod;
            let api: Api<Pod> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Pod resources".to_string());
            };
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Pod: {}", e))?;
        },
        // Add other resource types as needed
        _ => return Err(format!("Update operation not yet supported for resource kind: {}", resource_kind)),
    }
    
    println!("✅ Successfully updated {} '{}'{}",
        resource_kind,
        resource_name,
        namespace.map(|ns| format!(" in namespace '{}'", ns)).unwrap_or_default()
    );
    
    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    // Validate URL to ensure it starts with http or https
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL: must start with http:// or https://".to_string());
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/c", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_pods_by_selector(
    state: State<'_, AppState>,
    namespace: Option<String>,
    selector: String,
) -> Result<Vec<serde_json::Value>, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, ListParams};
    use crate::k8s::watch::convert_to_list_item;
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    let api: Api<Pod> = if let Some(ref ns) = namespace {
        Api::namespaced(client, ns)
    } else {
        Api::all(client)
    };
    
    let list_params = ListParams::default().labels(&selector);
    
    let pods = api
        .list(&list_params)
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;
    
    // Convert pods to K8sListItem format with proper podSpec and podStatus extraction
    let converted_pods: Result<Vec<_>, _> = pods.items.into_iter()
        .map(|pod| convert_to_list_item(&pod, "pods"))
        .collect();
        
    let pod_items = converted_pods.map_err(|e| format!("Failed to convert pod data: {}", e))?;
    
    // Convert to JSON values for easier frontend handling
    let pod_list: Vec<serde_json::Value> = pod_items
        .into_iter()
        .map(|item| serde_json::to_value(item).unwrap_or_default())
        .collect();
    
    Ok(pod_list)
}

#[tauri::command]
pub async fn get_node_pods(
    state: State<'_, AppState>,
    node_name: String,
) -> Result<serde_json::Value, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, ListParams};
    use crate::k8s::watch::convert_to_list_item;
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    let api: Api<Pod> = Api::all(client);
    
    // Use field selector to filter pods by node name
    let list_params = ListParams::default().fields(&format!("spec.nodeName={}", node_name));
    
    let pods = api
        .list(&list_params)
        .await
        .map_err(|e| format!("Failed to list pods for node '{}': {}", node_name, e))?;
    
    // Convert pods to K8sListItem format with proper podSpec and podStatus extraction
    let converted_pods: Result<Vec<_>, _> = pods.items.into_iter()
        .map(|pod| convert_to_list_item(&pod, "pods"))
        .collect();
        
    let pod_items = converted_pods.map_err(|e| format!("Failed to convert pod data: {}", e))?;
    
    // Return in a format similar to other list endpoints
    Ok(serde_json::json!({
        "items": pod_items
    }))
}