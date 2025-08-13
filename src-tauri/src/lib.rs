pub mod k8s;
pub mod security;

use k8s::{K8sClient, K8sContext, LogStreamManager, WatchManager, get_resource_categories};
use security::{ShellValidator, InputSanitizer};
use std::sync::Arc;
use std::collections::HashMap;
use std::env;
use tauri::{AppHandle, State, Emitter};
use tokio::sync::Mutex;

struct ShellSession {
    handle: tokio::task::JoinHandle<()>,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
}

struct AppState {
    k8s_client: K8sClient,
    watch_manager: Arc<Mutex<Option<WatchManager>>>,
    log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    shell_sessions: Arc<Mutex<HashMap<String, ShellSession>>>,
    shell_validator: Arc<ShellValidator>,
    input_sanitizer: Arc<InputSanitizer>,
}

#[tauri::command]
async fn connect_k8s(state: State<'_, AppState>) -> Result<(), String> {
    state.k8s_client.connect().await.map_err(|e| e.to_string())?;
    
    let watch_manager = WatchManager::new(state.k8s_client.clone());
    let mut manager_lock = state.watch_manager.lock().await;
    *manager_lock = Some(watch_manager);
    
    let log_stream_manager = LogStreamManager::new(state.k8s_client.clone());
    let mut log_manager_lock = state.log_stream_manager.lock().await;
    *log_manager_lock = Some(log_stream_manager);
    
    Ok(())
}

#[tauri::command]
async fn connect_k8s_with_context(state: State<'_, AppState>, context_name: String) -> Result<(), String> {
    state.k8s_client.connect_with_context(&context_name).await.map_err(|e| e.to_string())?;
    
    let watch_manager = WatchManager::new(state.k8s_client.clone());
    let mut manager_lock = state.watch_manager.lock().await;
    *manager_lock = Some(watch_manager);
    
    let log_stream_manager = LogStreamManager::new(state.k8s_client.clone());
    let mut log_manager_lock = state.log_stream_manager.lock().await;
    *log_manager_lock = Some(log_stream_manager);
    
    Ok(())
}

#[tauri::command]
async fn get_k8s_contexts() -> Result<Vec<K8sContext>, String> {
    K8sClient::get_contexts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_current_k8s_context() -> Result<String, String> {
    K8sClient::get_current_context().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_resources() -> Result<Vec<k8s::K8sResourceCategory>, String> {
    Ok(get_resource_categories())
}

#[tauri::command]
async fn get_namespaces(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    use k8s_openapi::api::core::v1::Namespace;
    use kube::api::{Api, ListParams};
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    let api: Api<Namespace> = Api::all(client);
    
    let namespaces = api.list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;
    
    let namespace_names: Vec<String> = namespaces
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect();
    
    Ok(namespace_names)
}

#[tauri::command]
async fn start_resource_watch(
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
async fn stop_resource_watch(
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
async fn get_pod_logs(
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
async fn get_full_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
) -> Result<serde_json::Value, String> {
    use kube::api::Api;
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    // Convert the resource kind to the appropriate API type
    match resource_kind.as_str() {
        "Pod" => {
            use k8s_openapi::api::core::v1::Pod;
            let api: Api<Pod> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let pod = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(pod).map_err(|e| e.to_string())?)
        }
        "Deployment" => {
            use k8s_openapi::api::apps::v1::Deployment;
            let api: Api<Deployment> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let deployment = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(deployment).map_err(|e| e.to_string())?)
        }
        "StatefulSet" => {
            use k8s_openapi::api::apps::v1::StatefulSet;
            let api: Api<StatefulSet> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let statefulset = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(statefulset).map_err(|e| e.to_string())?)
        }
        "DaemonSet" => {
            use k8s_openapi::api::apps::v1::DaemonSet;
            let api: Api<DaemonSet> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let daemonset = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(daemonset).map_err(|e| e.to_string())?)
        }
        "ReplicaSet" => {
            use k8s_openapi::api::apps::v1::ReplicaSet;
            let api: Api<ReplicaSet> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let replicaset = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(replicaset).map_err(|e| e.to_string())?)
        }
        "Service" => {
            use k8s_openapi::api::core::v1::Service;
            let api: Api<Service> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let service = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(service).map_err(|e| e.to_string())?)
        }
        "ConfigMap" => {
            use k8s_openapi::api::core::v1::ConfigMap;
            let api: Api<ConfigMap> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let configmap = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(configmap).map_err(|e| e.to_string())?)
        }
        "Secret" => {
            use k8s_openapi::api::core::v1::Secret;
            let api: Api<Secret> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let secret = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(secret).map_err(|e| e.to_string())?)
        }
        "Ingress" => {
            use k8s_openapi::api::networking::v1::Ingress;
            let api: Api<Ingress> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let ingress = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(ingress).map_err(|e| e.to_string())?)
        }
        "Role" => {
            use k8s_openapi::api::rbac::v1::Role;
            let api: Api<Role> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let role = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(role).map_err(|e| e.to_string())?)
        }
        "RoleBinding" => {
            use k8s_openapi::api::rbac::v1::RoleBinding;
            let api: Api<RoleBinding> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let rolebinding = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(rolebinding).map_err(|e| e.to_string())?)
        }
        "ClusterRole" => {
            use k8s_openapi::api::rbac::v1::ClusterRole;
            let api: Api<ClusterRole> = Api::all(client);
            let clusterrole = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(clusterrole).map_err(|e| e.to_string())?)
        }
        "ClusterRoleBinding" => {
            use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
            let api: Api<ClusterRoleBinding> = Api::all(client);
            let clusterrolebinding = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(clusterrolebinding).map_err(|e| e.to_string())?)
        }
        "Namespace" => {
            use k8s_openapi::api::core::v1::Namespace;
            let api: Api<Namespace> = Api::all(client);
            let namespace_obj = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(namespace_obj).map_err(|e| e.to_string())?)
        }
        "Node" => {
            use k8s_openapi::api::core::v1::Node;
            let api: Api<Node> = Api::all(client);
            let node = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(node).map_err(|e| e.to_string())?)
        }
        "PersistentVolume" => {
            use k8s_openapi::api::core::v1::PersistentVolume;
            let api: Api<PersistentVolume> = Api::all(client);
            let pv = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(pv).map_err(|e| e.to_string())?)
        }
        "PersistentVolumeClaim" => {
            use k8s_openapi::api::core::v1::PersistentVolumeClaim;
            let api: Api<PersistentVolumeClaim> = if let Some(ns) = namespace {
                Api::namespaced(client, &ns)
            } else {
                Api::all(client)
            };
            let pvc = api.get(&resource_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(pvc).map_err(|e| e.to_string())?)
        }
        _ => Err(format!("Unsupported resource kind: {}", resource_kind))
    }
}

#[tauri::command]
async fn start_pod_logs_stream(
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
async fn stop_pod_logs_stream(
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
async fn get_resource_events(
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

#[tauri::command]
async fn delete_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
) -> Result<(), String> {
    use kube::api::{Api, DeleteParams};
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    
    // Create the appropriate API client using concrete types for better compatibility
    match resource_kind.as_str() {
        "Pod" => {
            use k8s_openapi::api::core::v1::Pod;
            let api: Api<Pod> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Pod resources".to_string());
            };
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "Service" => {
            use k8s_openapi::api::core::v1::Service;
            let api: Api<Service> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Service resources".to_string());
            };
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "ConfigMap" => {
            use k8s_openapi::api::core::v1::ConfigMap;
            let api: Api<ConfigMap> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for ConfigMap resources".to_string());
            };
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "Secret" => {
            use k8s_openapi::api::core::v1::Secret;
            let api: Api<Secret> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Secret resources".to_string());
            };
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "Deployment" => {
            use k8s_openapi::api::apps::v1::Deployment;
            let api: Api<Deployment> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Deployment resources".to_string());
            };
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "Namespace" => {
            use k8s_openapi::api::core::v1::Namespace;
            let api: Api<Namespace> = Api::all(client);
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        "Node" => {
            use k8s_openapi::api::core::v1::Node;
            let api: Api<Node> = Api::all(client);
            api.delete(&resource_name, &DeleteParams::default()).await.map_err(|e| e.to_string())?;
        },
        _ => return Err(format!("Delete operation not yet supported for resource kind: {}", resource_kind)),
    }
    
    println!("🗑️ Successfully deleted {} '{}'{}",
        resource_kind,
        resource_name,
        namespace.map(|ns| format!(" in namespace '{}'", ns)).unwrap_or_default()
    );
    
    Ok(())
}

#[tauri::command]
async fn start_pod_shell(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
    _cols: u16,
    _rows: u16,
) -> Result<String, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, AttachParams};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    let api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    // Generate unique session ID
    let session_id = Uuid::new_v4().to_string();
    let session_id_clone = session_id.clone();
    
    // Create channel for sending input to shell
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    
    // Clone necessary data before moving into closure
    let shell_validator = state.shell_validator.clone();
    
    // Spawn task to handle shell interaction
    let app_handle_clone = app_handle.clone();
    let pod_name_clone = pod_name.clone();
    let container_name_clone = container_name.clone();
    
    let handle = tokio::spawn(async move {
        let mut attach_params = AttachParams {
            stdin: true,
            stdout: true,
            stderr: false, // Must be false when tty is true
            tty: true,
            ..Default::default()
        };
        
        if let Some(ref container) = container_name_clone {
            attach_params.container = Some(container.clone());
        }
        
        // Get safe shell command from validator
        let shell_command = shell_validator.get_initial_command();
        
        // Attach to pod with secure shell command
        match api.exec(&pod_name_clone, shell_command, &attach_params).await {
            Ok(mut attached) => {
                // Get streams
                let mut stdout = attached.stdout().unwrap();
                let mut stdin = attached.stdin().unwrap();
                
                // Handle stdout (stderr is merged with stdout in TTY mode)
                let session_id_out = session_id_clone.clone();
                let app_handle_out = app_handle_clone.clone();
                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    loop {
                        match stdout.read(&mut buffer).await {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                let data = String::from_utf8_lossy(&buffer[..n]);
                                let _ = app_handle_out.emit("shell-output", serde_json::json!({
                                    "session_id": session_id_out,
                                    "data": data
                                }));
                            }
                            Err(_) => break,
                        }
                    }
                });
                
                // Handle stdin
                while let Some(input) = rx.recv().await {
                    if stdin.write_all(input.as_bytes()).await.is_err() {
                        break;
                    }
                }
                
                // Send exit event
                let _ = app_handle_clone.emit("shell-exit", serde_json::json!({
                    "session_id": session_id_clone
                }));
            }
            Err(e) => {
                let _ = app_handle_clone.emit("shell-error", serde_json::json!({
                    "session_id": session_id_clone,
                    "error": format!("Failed to attach to pod: {}", e)
                }));
            }
        }
    });
    
    // Store session
    let session = ShellSession {
        handle,
        tx,
    };
    
    let mut sessions = state.shell_sessions.lock().await;
    sessions.insert(session_id.clone(), session);
    
    Ok(session_id)
}

#[tauri::command]
async fn send_shell_input(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    // Validate shell input for security
    let validated_data = state.shell_validator.validate_input(&data)
        .map_err(|e| format!("Invalid shell input: {}", e))?;
    
    let sessions = state.shell_sessions.lock().await;
    
    if let Some(session) = sessions.get(&session_id) {
        session.tx.send(validated_data).map_err(|e| format!("Failed to send input: {}", e))?;
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn resize_shell(
    _state: State<'_, AppState>,
    _session_id: String,
    _cols: u16,
    _rows: u16,
) -> Result<(), String> {
    // Terminal resize is not directly supported by the Kubernetes API
    // This is a placeholder for future implementation
    // You might need to send terminal control sequences through the shell
    Ok(())
}

#[tauri::command]
async fn stop_pod_shell(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.shell_sessions.lock().await;
    
    if let Some(session) = sessions.remove(&session_id) {
        session.handle.abort();
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn update_resource(
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
    
    // Apply the update based on resource type using concrete types
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
        "Service" => {
            use k8s_openapi::api::core::v1::Service;
            let api: Api<Service> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Service resources".to_string());
            };
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Service: {}", e))?;
        },
        "ConfigMap" => {
            use k8s_openapi::api::core::v1::ConfigMap;
            let api: Api<ConfigMap> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for ConfigMap resources".to_string());
            };
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update ConfigMap: {}", e))?;
        },
        "Secret" => {
            use k8s_openapi::api::core::v1::Secret;
            let api: Api<Secret> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Secret resources".to_string());
            };
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Secret: {}", e))?;
        },
        "Deployment" => {
            use k8s_openapi::api::apps::v1::Deployment;
            let api: Api<Deployment> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Deployment resources".to_string());
            };
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Deployment: {}", e))?;
        },
        "Namespace" => {
            use k8s_openapi::api::core::v1::Namespace;
            let api: Api<Namespace> = Api::all(client);
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Namespace: {}", e))?;
        },
        "Node" => {
            use k8s_openapi::api::core::v1::Node;
            let api: Api<Node> = Api::all(client);
            let patch_params = PatchParams::apply("kide-client").force();
            let patch = Patch::Apply(&json_value);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to update Node: {}", e))?;
        },
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
async fn scale_resource(
    state: State<'_, AppState>,
    resource_name: String,
    resource_kind: String,
    namespace: Option<String>,
    replicas: i32,
) -> Result<(), String> {
    use kube::api::{Api, Patch, PatchParams};
    use serde_json::json;
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    
    // Create the scale patch
    let scale_patch = json!({
        "spec": {
            "replicas": replicas
        }
    });
    
    // Apply the scale operation based on resource type
    match resource_kind.as_str() {
        "Deployment" => {
            use k8s_openapi::api::apps::v1::Deployment;
            let api: Api<Deployment> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for Deployment resources".to_string());
            };
            let patch_params = PatchParams::default();
            let patch = Patch::Merge(&scale_patch);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to scale Deployment: {}", e))?;
        },
        "StatefulSet" => {
            use k8s_openapi::api::apps::v1::StatefulSet;
            let api: Api<StatefulSet> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for StatefulSet resources".to_string());
            };
            let patch_params = PatchParams::default();
            let patch = Patch::Merge(&scale_patch);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to scale StatefulSet: {}", e))?;
        },
        "ReplicaSet" => {
            use k8s_openapi::api::apps::v1::ReplicaSet;
            let api: Api<ReplicaSet> = if let Some(ref ns) = namespace {
                Api::namespaced(client, ns)
            } else {
                return Err("Namespace is required for ReplicaSet resources".to_string());
            };
            let patch_params = PatchParams::default();
            let patch = Patch::Merge(&scale_patch);
            api.patch(&resource_name, &patch_params, &patch)
                .await
                .map_err(|e| format!("Failed to scale ReplicaSet: {}", e))?;
        },
        _ => {
            return Err(format!("Resource type '{}' does not support scaling", resource_kind));
        }
    }
    
    println!("✅ Successfully scaled {} '{}' to {} replicas", resource_kind, resource_name, replicas);
    
    // Force a brief delay to allow the Kubernetes API server to propagate the change
    // This helps ensure that subsequent watch events will include the updated status
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("🔄 Scaling operation completed, watch should capture status updates soon");
    Ok(())
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    use std::process::Command;
    
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
async fn get_pods_by_selector(
    state: State<'_, AppState>,
    namespace: Option<String>,
    selector: String,
) -> Result<Vec<serde_json::Value>, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, ListParams};
    
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
    
    // Convert to JSON values for easier frontend handling
    let pod_list: Vec<serde_json::Value> = pods
        .items
        .into_iter()
        .map(|pod| serde_json::to_value(pod).unwrap_or_default())
        .collect();
    
    Ok(pod_list)
}

#[tauri::command]
async fn get_node_pods(
    state: State<'_, AppState>,
    node_name: String,
) -> Result<serde_json::Value, String> {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{Api, ListParams};
    
    let client = state.k8s_client.get_client().await.map_err(|e| e.to_string())?;
    let api: Api<Pod> = Api::all(client);
    
    // Use field selector to filter pods by node name
    let list_params = ListParams::default().fields(&format!("spec.nodeName={}", node_name));
    
    let pods = api
        .list(&list_params)
        .await
        .map_err(|e| format!("Failed to list pods for node '{}': {}", node_name, e))?;
    
    // Return in a format similar to other list endpoints
    Ok(serde_json::json!({
        "items": pods.items.into_iter()
            .map(|pod| serde_json::to_value(pod).unwrap_or_default())
            .collect::<Vec<serde_json::Value>>()
    }))
}

/// Get environment variables from the user's shell (equivalent to shell-env npm package)
/// This solves the issue where GUI apps don't inherit shell environment from dotfiles
fn get_shell_environment() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // Determine user's default shell
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    
    // Command to source shell profile and print environment
    let mut cmd = Command::new(&shell);
    
    // Use different approaches based on shell type
    if shell.contains("zsh") {
        cmd.args(&["-l", "-i", "-c", "env"]);
    } else if shell.contains("bash") {
        cmd.args(&["-l", "-i", "-c", "env"]);
    } else {
        // Fallback for other shells
        cmd.args(&["-c", "env"]);
    }
    
    // Execute shell command to get environment
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(format!("Shell command failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Parse environment variables from output
    let env_output = String::from_utf8(output.stdout)?;
    let mut shell_env = HashMap::new();
    
    for line in env_output.lines() {
        if let Some(eq_pos) = line.find('=') {
            let key = &line[..eq_pos];
            let value = &line[eq_pos + 1..];
            
            // Only include if key looks valid (alphanumeric + underscore)
            if key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                shell_env.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    Ok(shell_env)
}

/// Setup enhanced environment by inheriting shell environment (like Lens does)
/// This solves the common issue where packaged desktop apps don't inherit
/// the full shell environment that includes custom PATH modifications.
fn setup_environment() {
    println!("🔧 Setting up shell environment for authentication tools...");
    
    // First, try to get shell environment (like shell-env npm package)
    match get_shell_environment() {
        Ok(shell_env) => {
            // Apply shell environment variables, particularly PATH
            for (key, value) in shell_env.iter() {
                // Only set important environment variables
                match key.as_str() {
                    "PATH" | "HOME" | "USER" | "AWS_PROFILE" | "AWS_DEFAULT_REGION" | 
                    "GOOGLE_APPLICATION_CREDENTIALS" | "KUBECONFIG" => {
                        env::set_var(key, value);
                        if key == "PATH" {
                            println!("🔧 Inherited shell PATH: {}", value);
                        }
                    }
                    _ if key.starts_with("AWS_") || key.starts_with("GOOGLE_") || 
                         key.starts_with("AZURE_") || key.starts_with("KUBE") => {
                        env::set_var(key, value);
                    }
                    _ => {} // Skip other variables
                }
            }
            
            println!("✅ Successfully inherited shell environment");
        }
        Err(e) => {
            println!("⚠️  Could not get shell environment ({}), falling back to manual PATH enhancement", e);
            
            // Fallback to manual PATH enhancement if shell method fails
            let current_path = env::var("PATH").unwrap_or_default();
            
            let mut additional_paths = vec![
                "/usr/local/bin".to_string(),
                "/opt/homebrew/bin".to_string(),
                "/usr/bin".to_string(),
                "/bin".to_string(),
                "/usr/local/aws-cli/v2/current/bin".to_string(),
                "/snap/bin".to_string(),
                "/usr/local/google-cloud-sdk/bin".to_string(),
            ];
            
            if let Ok(home) = env::var("HOME") {
                additional_paths.push(format!("{}/.local/bin", home));
            }
            
            let mut all_paths: Vec<String> = additional_paths.into_iter()
                .filter(|path| !current_path.contains(path))
                .collect();
            
            if !current_path.is_empty() {
                all_paths.push(current_path);
            }
            
            let enhanced_path = all_paths.join(":");
            env::set_var("PATH", &enhanced_path);
            
            println!("🔧 Fallback PATH: {}", enhanced_path);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Setup enhanced environment before initializing Kubernetes client
    setup_environment();
    let k8s_client = K8sClient::new();
    let app_state = AppState {
        k8s_client,
        watch_manager: Arc::new(Mutex::new(None)),
        log_stream_manager: Arc::new(Mutex::new(None)),
        shell_sessions: Arc::new(Mutex::new(HashMap::new())),
        shell_validator: Arc::new(ShellValidator::new()),
        input_sanitizer: Arc::new(InputSanitizer::new()),
    };

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
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn test_get_resources_command() {
        let result = tokio_test::block_on(get_resources());
        
        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 9);
        
        // Verify we have the expected categories
        let category_names: Vec<&String> = categories.iter().map(|c| &c.name).collect();
        assert!(category_names.contains(&&"Commonly used".to_string()));
        assert!(category_names.contains(&&"Workloads".to_string()));
        assert!(category_names.contains(&&"Services & Networking".to_string()));
        assert!(category_names.contains(&&"Configuration".to_string()));
        assert!(category_names.contains(&&"Storage".to_string()));
        assert!(category_names.contains(&&"Cluster Administration".to_string()));
        assert!(category_names.contains(&&"Security & Access Control".to_string()));
        assert!(category_names.contains(&&"Scaling".to_string()));
        assert!(category_names.contains(&&"Custom Resources".to_string()));
    }

    #[fixture]
    fn app_state() -> AppState {
        AppState {
            k8s_client: K8sClient::new(),
            watch_manager: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_sessions: Arc::new(Mutex::new(HashMap::new())),
            shell_validator: Arc::new(ShellValidator::new()),
            input_sanitizer: Arc::new(InputSanitizer::new()),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_watch_manager_state(app_state: AppState) {
        // Test the watch manager state without requiring AppHandle
        // The watch manager should start as None
        let manager_lock = app_state.watch_manager.lock().await;
        assert!(manager_lock.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn test_client_state_management(app_state: AppState) {
        // Test that the client starts disconnected
        let is_connected = app_state.k8s_client.is_connected().await;
        assert!(!is_connected);
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState {
            k8s_client: K8sClient::new(),
            watch_manager: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            shell_sessions: Arc::new(Mutex::new(HashMap::new())),
            shell_validator: Arc::new(ShellValidator::new()),
            input_sanitizer: Arc::new(InputSanitizer::new()),
        };
        
        // Verify initial state
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
        let all_resources: Vec<&k8s::K8sResource> = categories
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