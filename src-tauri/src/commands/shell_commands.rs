use tauri::{AppHandle, State, Emitter};
use crate::state::{AppState, ShellSession};
use uuid::Uuid;

#[tauri::command]
pub async fn start_pod_shell(
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
pub async fn send_shell_input(
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
pub async fn resize_shell(
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
pub async fn stop_pod_shell(
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