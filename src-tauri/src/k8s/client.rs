use kube::{Client, Config};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sContext {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
}

#[derive(Clone)]
pub struct K8sClient {
    client: Arc<Mutex<Option<Client>>>,
}

impl Default for K8sClient {
    fn default() -> Self {
        Self::new()
    }
}

impl K8sClient {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<(), anyhow::Error> {
        let config = Config::infer().await?;
        let client = Client::try_from(config)?;
        
        let mut client_lock = self.client.lock().await;
        *client_lock = Some(client);
        
        Ok(())
    }

    pub async fn connect_with_context(&self, context_name: &str) -> Result<(), anyhow::Error> {
        let mut config = Config::infer().await?;
        
        // Override the context if specified
        if !context_name.is_empty() {
            config = Config::from_kubeconfig(&kube::config::KubeConfigOptions {
                context: Some(context_name.to_string()),
                cluster: None,
                user: None,
            }).await?;
        }
        
        let client = Client::try_from(config)?;
        
        let mut client_lock = self.client.lock().await;
        *client_lock = Some(client);
        
        Ok(())
    }

    pub async fn get_client(&self) -> Result<Client, anyhow::Error> {
        let client_lock = self.client.lock().await;
        client_lock
            .clone()
            .ok_or_else(|| anyhow::anyhow!("K8s client not initialized"))
    }

    pub async fn is_connected(&self) -> bool {
        let client_lock = self.client.lock().await;
        client_lock.is_some()
    }

    pub async fn get_contexts() -> Result<Vec<K8sContext>, anyhow::Error> {
        use kube::config::Kubeconfig;
        
        let kubeconfig = Kubeconfig::read().map_err(|e| {
            anyhow::anyhow!("Failed to read or parse kubeconfig: {}", e)
        })?;

        let mut contexts = Vec::new();
        
        // kubeconfig.contexts is a Vec<NamedContext>, not Option<Vec<NamedContext>>
        for named_context in kubeconfig.contexts {
            if let Some(context) = named_context.context {
                contexts.push(K8sContext {
                    name: named_context.name,
                    cluster: context.cluster,
                    user: context.user.unwrap_or_else(|| "unknown".to_string()),
                    namespace: context.namespace,
                });
            }
        }

        Ok(contexts)
    }

    pub async fn get_current_context() -> Result<String, anyhow::Error> {
        use kube::config::Kubeconfig;
        
        let kubeconfig = Kubeconfig::read().map_err(|e| {
            anyhow::anyhow!("Failed to read or parse kubeconfig: {}", e)
        })?;

        Ok(kubeconfig.current_context.unwrap_or_else(|| "default".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_k8s_client_creation() {
        let client = K8sClient::new();
        assert!(!client.is_connected().await, "Client should not be connected initially");
    }

    #[tokio::test]
    async fn test_get_client_before_connection() {
        let client = K8sClient::new();
        let result = client.get_client().await;
        assert!(result.is_err(), "Should return error when not connected");
        
        if let Err(error) = result {
            let error_message = error.to_string();
            assert!(error_message.contains("not initialized"), "Error should mention initialization issue");
        }
    }

    #[tokio::test]
    async fn test_client_cloning() {
        let client1 = K8sClient::new();
        let client2 = client1.clone();
        
        // Both should be disconnected initially
        assert!(!client1.is_connected().await);
        assert!(!client2.is_connected().await);
    }

    #[test]
    fn test_client_implements_required_traits() {
        // Test that K8sClient implements Clone (required for sharing across threads)
        let client = K8sClient::new();
        let _cloned = client.clone();
        
        // Test that it can be sent across thread boundaries
        fn requires_send<T: Send>(_t: T) {}
        fn requires_sync<T: Sync>(_t: T) {}
        
        requires_send(client.clone());
        requires_sync(client);
    }

    #[tokio::test]
    async fn test_connection_state_management() {
        let client = K8sClient::new();
        
        // Initially not connected
        assert!(!client.is_connected().await);
        
        // After connection attempt (will fail without K8s cluster, but that's OK for structure test)
        let result = client.connect().await;
        
        if result.is_ok() {
            // If connection succeeded (real K8s cluster available)
            assert!(client.is_connected().await);
            
            // Should be able to get client
            let client_result = client.get_client().await;
            assert!(client_result.is_ok());
        } else {
            // If connection failed (no K8s cluster), state should remain disconnected
            assert!(!client.is_connected().await);
        }
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let client = K8sClient::new();
        
        // Test that multiple connection attempts don't cause issues
        let result1 = client.connect().await;
        let result2 = client.connect().await;
        
        // Both should either succeed or fail with the same type of result
        assert_eq!(result1.is_ok(), result2.is_ok());
        
        // Connection state should be consistent
        let is_connected_after = client.is_connected().await;
        if result1.is_ok() {
            assert!(is_connected_after);
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let client = Arc::new(K8sClient::new());
        let mut handles = vec![];
        
        // Spawn multiple tasks trying to check connection status
        for _ in 0..10 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                client_clone.is_connected().await
            });
            handles.push(handle);
        }
        
        // All should complete without panicking
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}