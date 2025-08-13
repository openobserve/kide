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
    // Note: We use kubectl apply instead of direct Kube API for generic resource updating
    
    // Validate inputs
    state.input_sanitizer.validate_resource_name(&resource_name)
        .map_err(|e| format!("Invalid resource name: {}", e))?;
    
    if let Some(ref ns) = namespace {
        state.input_sanitizer.validate_namespace(ns)
            .map_err(|e| format!("Invalid namespace: {}", e))?;
    }
    
    state.input_sanitizer.validate_yaml_content(&yaml_content)
        .map_err(|e| format!("Invalid YAML content: {}", e))?;
    
    // We don't need the client for kubectl apply, but validate the connection exists
    let _client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    // Validate YAML syntax only - kubectl apply will handle the rest
    let _yaml_value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml_content)
        .map_err(|e| format!("Invalid YAML syntax: {}", e))?;
    
    // Use generic update operation with kubectl apply for all resource types
    // This approach works for any Kubernetes resource without needing specific type handling
    use std::process::Command;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    // Write YAML content to a temporary file
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary file: {}", e))?;
    
    temp_file.write_all(yaml_content.as_bytes())
        .map_err(|e| format!("Failed to write YAML to temporary file: {}", e))?;
    
    // Build kubectl apply command
    let mut cmd = Command::new("kubectl");
    cmd.arg("apply")
       .arg("-f")
       .arg(temp_file.path());
    
    // Add namespace if provided
    if let Some(ref ns) = namespace {
        cmd.arg("-n").arg(ns);
    }
    
    // Execute kubectl apply
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute kubectl apply: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("kubectl apply failed: {}", error_msg));
    }
    
    // Log success message
    let success_msg = String::from_utf8_lossy(&output.stdout);
    println!("✅ kubectl apply output: {}", success_msg.trim());
    
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