use super::client::K8sClient;
use futures::{AsyncBufReadExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

pub struct LogStreamManager {
    client: K8sClient,
    active_streams: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl LogStreamManager {
    pub fn new(client: K8sClient) -> Self {
        let manager = Self {
            client,
            active_streams: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start periodic cleanup task for log streams
        let streams_clone = manager.active_streams.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Cleanup finished log streams
                let mut streams = streams_clone.lock().await;
                streams.retain(|key, handle| {
                    if handle.is_finished() {
                        println!("🧹 Cleaned up finished log stream: {key}");
                        false
                    } else {
                        true
                    }
                });
                
                if !streams.is_empty() {
                    println!("📊 Active log streams: {}", streams.len());
                }
            }
        });
        
        manager
    }

    pub async fn start_log_stream(
        &self,
        app_handle: AppHandle,
        pod_name: String,
        namespace: String,
        container_name: Option<String>,
    ) -> Result<String, anyhow::Error> {
        let stream_id = format!(
            "{}:{}:{}",
            namespace,
            pod_name,
            container_name.as_ref().unwrap_or(&"default".to_string())
        );

        // Check if stream already exists
        let mut streams = self.active_streams.lock().await;
        if streams.contains_key(&stream_id) {
            return Ok(stream_id);
        }

        let client = self.client.get_client().await?;
        let api: Api<Pod> = Api::namespaced(client, &namespace);

        let mut log_params = LogParams {
            follow: true,
            timestamps: true,
            ..Default::default()
        };

        if let Some(container) = container_name {
            log_params.container = Some(container);
        }

        // Start with last 50 lines
        log_params.tail_lines = Some(50);

        let stream_id_clone = stream_id.clone();
        let app_handle_clone = app_handle.clone();
        let pod_name_clone = pod_name.clone();

        let handle = tokio::spawn(async move {
            use tokio::time::{timeout, Duration};
            
            // Add connection timeout
            match timeout(Duration::from_secs(30), api.log_stream(&pod_name_clone, &log_params)).await {
                Ok(Ok(stream)) => {
                    // Read lines from the async stream with timeout protection
                    let mut lines = stream.lines();
                    
                    loop {
                        let line_timeout_result = timeout(Duration::from_secs(30), lines.next()).await;
                        
                        match line_timeout_result {
                            Ok(Some(line_result)) => {
                                match line_result {
                                    Ok(line) => {
                                        let _ = app_handle_clone.emit(
                                            "pod-log-line",
                                            serde_json::json!({
                                                "stream_id": stream_id_clone,
                                                "line": line
                                            }),
                                        );
                                    }
                                    Err(e) => {
                                        let _ = app_handle_clone.emit(
                                            "pod-log-error",
                                            serde_json::json!({
                                                "stream_id": stream_id_clone,
                                                "error": format!("Stream error: {}", e)
                                            }),
                                        );
                                        break;
                                    }
                                }
                            }
                            Ok(None) => {
                                // Stream ended normally
                                println!("🔄 Log stream ended for {stream_id_clone}");
                                break;
                            }
                            Err(_) => {
                                // Timeout occurred
                                println!("⚠️  Log stream timeout for {stream_id_clone} - ending stream");
                                let _ = app_handle_clone.emit(
                                    "pod-log-error",
                                    serde_json::json!({
                                        "stream_id": stream_id_clone,
                                        "error": "Log stream timeout"
                                    }),
                                );
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    let _ = app_handle_clone.emit(
                        "pod-log-error",
                        serde_json::json!({
                            "stream_id": stream_id_clone,
                            "error": format!("Failed to start log stream: {}", e)
                        }),
                    );
                }
                Err(_) => {
                    let _ = app_handle_clone.emit(
                        "pod-log-error",
                        serde_json::json!({
                            "stream_id": stream_id_clone,
                            "error": "Log stream connection timeout"
                        }),
                    );
                }
            }
            
            // Emit stream end event
            let _ = app_handle_clone.emit(
                "pod-log-end",
                serde_json::json!({
                    "stream_id": stream_id_clone
                }),
            );
        });

        streams.insert(stream_id.clone(), handle);
        Ok(stream_id)
    }

    pub async fn stop_log_stream(&self, stream_id: &str) -> Result<(), anyhow::Error> {
        let mut streams = self.active_streams.lock().await;
        if let Some(handle) = streams.remove(stream_id) {
            handle.abort();
        }
        Ok(())
    }

    pub async fn stop_all_streams(&self) -> Result<(), anyhow::Error> {
        let mut streams = self.active_streams.lock().await;
        for (_, handle) in streams.drain() {
            handle.abort();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id_generation() {
        let namespace = "default";
        let pod_name = "test-pod";
        let container_name = Some("nginx".to_string());
        
        let stream_id = format!(
            "{}:{}:{}",
            namespace,
            pod_name,
            container_name.as_ref().unwrap_or(&"default".to_string())
        );
        
        assert_eq!(stream_id, "default:test-pod:nginx");
    }

    #[test]
    fn test_stream_id_without_container() {
        let namespace = "kube-system";
        let pod_name = "coredns";
        let container_name: Option<String> = None;
        
        let stream_id = format!(
            "{}:{}:{}",
            namespace,
            pod_name,
            container_name.as_ref().unwrap_or(&"default".to_string())
        );
        
        assert_eq!(stream_id, "kube-system:coredns:default");
    }

    #[tokio::test]
    async fn test_log_stream_manager_creation() {
        let client = K8sClient::new();
        let manager = LogStreamManager::new(client);
        
        let streams = manager.active_streams.lock().await;
        assert!(streams.is_empty());
    }
}