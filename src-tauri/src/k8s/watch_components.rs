//! Focused watch components replacing the monolithic WatchManager.
//!
//! This module breaks down the watch functionality into focused, single-responsibility components:
//! - WatchDispatcher: Routes watch requests to appropriate handlers
//! - WatchLifecycleManager: Manages watch lifecycle (start/stop/cleanup)
//! - WatchEventHandler: Processes and emits watch events
//! - ResourceWatcher: Generic trait for resource-specific watchers

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tauri::{AppHandle, Emitter};
use serde_json::Value;

use crate::errors::{K8sError, K8sResult};
use crate::k8s::RESOURCE_REGISTRY;

/// Identifier for a watch session.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchId {
    pub resource_type: String,
    pub namespaces: Vec<String>, // Empty vec means all namespaces
}

impl WatchId {
    pub fn new(resource_type: String, namespaces: Option<Vec<String>>) -> Self {
        let mut namespaces = namespaces.unwrap_or_default();
        namespaces.sort(); // Ensure consistent ordering
        
        Self {
            resource_type,
            namespaces,
        }
    }

    pub fn to_key(&self) -> String {
        if self.namespaces.is_empty() {
            format!("{}:all", self.resource_type)
        } else {
            format!("{}:{}", self.resource_type, self.namespaces.join(","))
        }
    }
}

/// Status of a watch session.
#[derive(Debug, Clone, PartialEq)]
pub enum WatchStatus {
    Starting,
    Active,
    Stopping,
    Stopped,
    Failed(String),
}

/// Information about an active watch.
#[derive(Debug)]
pub struct WatchSession {
    pub id: WatchId,
    pub status: WatchStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub events_received: u64,
    pub handle: tokio::task::JoinHandle<()>,
}

/// Trait for resource-specific watchers.
#[async_trait]
pub trait ResourceWatcher: Send + Sync {
    /// Start watching a resource type.
    async fn start_watch(
        &self,
        client: &kube::Client,
        app_handle: AppHandle,
        watch_id: WatchId,
    ) -> K8sResult<tokio::task::JoinHandle<()>>;

    /// Validate that this watcher can handle the resource type.
    fn can_watch(&self, resource_type: &str) -> bool;

    /// Get metadata about the resource type this watcher handles.
    fn resource_type(&self) -> &str;
}

/// Generic watcher for Kubernetes resources using the resource registry.
pub struct GenericResourceWatcher;

impl GenericResourceWatcher {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ResourceWatcher for GenericResourceWatcher {
    async fn start_watch(
        &self,
        client: &kube::Client,
        app_handle: AppHandle,
        watch_id: WatchId,
    ) -> K8sResult<tokio::task::JoinHandle<()>> {
        let handler = RESOURCE_REGISTRY.get_handler(&watch_id.resource_type)?;
        let metadata = handler.metadata();

        // Create appropriate API based on namespace requirements
        let handle = if metadata.is_namespaced && !watch_id.namespaces.is_empty() {
            // Watch specific namespaces
            tokio::spawn(watch_namespaced_resource(
                client.clone(),
                app_handle,
                watch_id,
            ))
        } else {
            // Watch all namespaces or cluster-wide resource
            tokio::spawn(watch_all_resource(
                client.clone(),
                app_handle,
                watch_id,
            ))
        };

        Ok(handle)
    }

    fn can_watch(&self, resource_type: &str) -> bool {
        RESOURCE_REGISTRY.is_registered(resource_type)
    }

    fn resource_type(&self) -> &str {
        "*" // Handles all registered resource types
    }
}

/// Manages the lifecycle of watch sessions.
pub struct WatchLifecycleManager {
    active_watches: Arc<RwLock<HashMap<WatchId, WatchSession>>>,
    watchers: Vec<Box<dyn ResourceWatcher>>,
}

impl WatchLifecycleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            active_watches: Arc::new(RwLock::new(HashMap::new())),
            watchers: Vec::new(),
        };

        // Register default watchers
        manager.register_watcher(Box::new(GenericResourceWatcher::new()));

        // Start cleanup task
        manager.start_cleanup_task();

        manager
    }

    /// Register a resource watcher.
    pub fn register_watcher(&mut self, watcher: Box<dyn ResourceWatcher>) {
        self.watchers.push(watcher);
    }

    /// Start watching a resource type.
    pub async fn start_watch(
        &self,
        client: &kube::Client,
        app_handle: AppHandle,
        resource_type: String,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        let watch_id = WatchId::new(resource_type.clone(), namespaces);

        // Check if already watching
        {
            let watches = self.active_watches.read().await;
            if watches.contains_key(&watch_id) {
                return Ok(());
            }
        }

        // Find appropriate watcher
        let watcher = self
            .watchers
            .iter()
            .find(|w| w.can_watch(&resource_type))
            .ok_or_else(|| K8sError::InvalidResourceType {
                resource_type: resource_type.clone(),
            })?;

        // Start the watch
        let handle = watcher.start_watch(client, app_handle, watch_id.clone()).await?;

        // Track the watch
        let watch_info = WatchSession {
            id: watch_id.clone(),
            status: WatchStatus::Active,
            started_at: chrono::Utc::now(),
            events_received: 0,
            handle,
        };

        let mut watches = self.active_watches.write().await;
        watches.insert(watch_id, watch_info);

        Ok(())
    }

    /// Stop watching a resource type.
    pub async fn stop_watch(
        &self,
        resource_type: String,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        let watch_id = WatchId::new(resource_type, namespaces);

        let mut watches = self.active_watches.write().await;
        if let Some(mut watch_info) = watches.remove(&watch_id) {
            watch_info.status = WatchStatus::Stopping;
            watch_info.handle.abort();
        }

        Ok(())
    }

    /// Stop all active watches.
    pub async fn stop_all_watches(&self) -> K8sResult<()> {
        let mut watches = self.active_watches.write().await;
        for (_, mut watch_info) in watches.drain() {
            watch_info.status = WatchStatus::Stopping;
            watch_info.handle.abort();
        }
        Ok(())
    }

    /// Get information about all active watches.
    pub async fn get_active_watches(&self) -> Vec<(WatchId, WatchStatus)> {
        let watches = self.active_watches.read().await;
        watches
            .iter()
            .map(|(id, info)| (id.clone(), info.status.clone()))
            .collect()
    }

    /// Get count of active watches.
    pub async fn active_watch_count(&self) -> usize {
        let watches = self.active_watches.read().await;
        watches.len()
    }

    /// Start periodic cleanup of finished watches.
    fn start_cleanup_task(&self) {
        let watches_clone = self.active_watches.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut watches = watches_clone.write().await;
                let initial_count = watches.len();

                watches.retain(|_id, info| {
                    if info.handle.is_finished() {
                        false // Remove finished watches
                    } else {
                        true
                    }
                });

                let cleaned_count = initial_count - watches.len();
                if cleaned_count > 0 {
                    println!("🧹 Cleaned up {} finished watches", cleaned_count);
                }

                if !watches.is_empty() {
                    println!("📊 Active watches: {}", watches.len());
                }
            }
        });
    }
}

impl Default for WatchLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispatches watch requests to appropriate handlers.
pub struct WatchDispatcher {
    lifecycle_manager: Arc<WatchLifecycleManager>,
    event_handler: Arc<WatchEventHandler>,
}

impl WatchDispatcher {
    pub fn new(
        lifecycle_manager: Arc<WatchLifecycleManager>,
        event_handler: Arc<WatchEventHandler>,
    ) -> Self {
        Self {
            lifecycle_manager,
            event_handler,
        }
    }

    /// Start watching resources with validation and error handling.
    pub async fn start_watch(
        &self,
        client: &kube::Client,
        app_handle: AppHandle,
        resource_type: String,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        // Validate resource type
        if !RESOURCE_REGISTRY.is_registered(&resource_type) {
            return Err(K8sError::InvalidResourceType {
                resource_type,
            });
        }

        // Delegate to lifecycle manager
        self.lifecycle_manager
            .start_watch(client, app_handle, resource_type, namespaces)
            .await
    }

    /// Stop watching resources.
    pub async fn stop_watch(
        &self,
        resource_type: String,
        namespaces: Option<Vec<String>>,
    ) -> K8sResult<()> {
        self.lifecycle_manager
            .stop_watch(resource_type, namespaces)
            .await
    }

    /// Get event handler reference.
    pub fn event_handler(&self) -> &Arc<WatchEventHandler> {
        &self.event_handler
    }

    /// Get watch statistics.
    pub async fn get_watch_stats(&self) -> WatchStatistics {
        let active_watches = self.lifecycle_manager.get_active_watches().await;
        let total_count = active_watches.len();

        let mut by_status = HashMap::new();
        let mut by_resource_type = HashMap::new();

        for (watch_id, status) in active_watches {
            // Count by status
            let status_key = match status {
                WatchStatus::Starting => "starting",
                WatchStatus::Active => "active",
                WatchStatus::Stopping => "stopping",
                WatchStatus::Stopped => "stopped",
                WatchStatus::Failed(_) => "failed",
            };
            *by_status.entry(status_key.to_string()).or_insert(0) += 1;

            // Count by resource type
            *by_resource_type
                .entry(watch_id.resource_type)
                .or_insert(0) += 1;
        }

        WatchStatistics {
            total_watches: total_count,
            by_status,
            by_resource_type,
        }
    }
}

/// Handles watch events and emits them to the frontend.
pub struct WatchEventHandler {
    event_processors: Vec<Box<dyn EventProcessor>>,
}

impl WatchEventHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            event_processors: Vec::new(),
        };

        // Register default event processors
        handler.register_processor(Box::new(DefaultEventProcessor));

        handler
    }

    /// Register an event processor.
    pub fn register_processor(&mut self, processor: Box<dyn EventProcessor>) {
        self.event_processors.push(processor);
    }

    /// Process and emit a watch event.
    pub async fn handle_event(
        &self,
        app_handle: &AppHandle,
        resource_type: &str,
        event: serde_json::Value,
    ) -> K8sResult<()> {
        // Process event through all processors
        let mut processed_event = event;
        for processor in &self.event_processors {
            processed_event = processor.process_event(resource_type, processed_event).await?;
        }

        // Emit to frontend
        app_handle
            .emit("resource-watch-event", WatchEventData {
                resource_type: resource_type.to_string(),
                timestamp: chrono::Utc::now(),
                data: processed_event,
            })
            .map_err(|e| K8sError::ApiError {
                message: format!("Failed to emit watch event: {}", e),
            })?;

        Ok(())
    }
}

impl Default for WatchEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for processing watch events before emission.
#[async_trait]
pub trait EventProcessor: Send + Sync {
    /// Process an event and return the modified event.
    async fn process_event(
        &self,
        resource_type: &str,
        event: serde_json::Value,
    ) -> K8sResult<serde_json::Value>;
}

/// Default event processor that adds metadata.
pub struct DefaultEventProcessor;

#[async_trait]
impl EventProcessor for DefaultEventProcessor {
    async fn process_event(
        &self,
        _resource_type: &str,
        mut event: serde_json::Value,
    ) -> K8sResult<serde_json::Value> {
        // Add processing timestamp
        if let Some(obj) = event.as_object_mut() {
            obj.insert(
                "_processed_at".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        Ok(event)
    }
}

/// Statistics about active watches.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchStatistics {
    pub total_watches: usize,
    pub by_status: HashMap<String, usize>,
    pub by_resource_type: HashMap<String, usize>,
}

/// Watch event data structure for frontend emission.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchEventData {
    pub resource_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

// Placeholder watch functions (to be implemented with actual Kubernetes API calls)
async fn watch_namespaced_resource(
    _client: kube::Client,
    _app_handle: AppHandle,
    _watch_id: WatchId,
) {
    // TODO: Implement namespaced resource watching
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
}

async fn watch_all_resource(
    _client: kube::Client,
    _app_handle: AppHandle,
    _watch_id: WatchId,
) {
    // TODO: Implement all-namespace resource watching
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_id_creation() {
        let id1 = WatchId::new("pods".to_string(), None);
        assert_eq!(id1.to_key(), "pods:all");

        let id2 = WatchId::new(
            "deployments".to_string(),
            Some(vec!["default".to_string(), "kube-system".to_string()]),
        );
        assert_eq!(id2.to_key(), "deployments:default,kube-system");

        // Test namespace sorting
        let id3 = WatchId::new(
            "services".to_string(),
            Some(vec!["kube-system".to_string(), "default".to_string()]),
        );
        assert_eq!(id3.to_key(), "services:default,kube-system"); // Should be sorted
    }

    #[test]
    fn test_generic_resource_watcher() {
        let watcher = GenericResourceWatcher::new();
        
        assert!(watcher.can_watch("pods"));
        assert!(watcher.can_watch("deployments"));
        assert!(!watcher.can_watch("invalid-resource"));
        assert_eq!(watcher.resource_type(), "*");
    }

    #[tokio::test]
    async fn test_watch_lifecycle_manager() {
        let manager = WatchLifecycleManager::new();
        
        assert_eq!(manager.active_watch_count().await, 0);
        
        let watches = manager.get_active_watches().await;
        assert!(watches.is_empty());
    }

    #[tokio::test]
    async fn test_default_event_processor() {
        let processor = DefaultEventProcessor;
        let event = serde_json::json!({"test": "value"});
        
        let processed = processor.process_event("pods", event).await.unwrap();
        
        assert!(processed.get("_processed_at").is_some());
        assert_eq!(processed.get("test").unwrap(), "value");
    }
}