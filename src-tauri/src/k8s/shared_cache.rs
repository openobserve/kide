use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tauri::{AppHandle, Emitter};
use anyhow::Result;
use kube::{Api, Client, ResourceExt};
use kube_runtime::{watcher, watcher::Config};
use futures::StreamExt;

use super::resources::{WatchEvent, K8sListItem};
use super::watch::convert_to_list_item;
use super::K8sClient;

/// Scope defines the boundaries of a watch (cluster + namespace + selector)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchScope {
    pub cluster_context: String,
    pub namespace: Option<String>, // None = cluster-wide
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
}

impl WatchScope {
    pub fn new(cluster_context: String) -> Self {
        Self {
            cluster_context,
            namespace: None,
            label_selector: None,
            field_selector: None,
        }
    }

    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }

    pub fn with_label_selector(mut self, selector: Option<String>) -> Self {
        self.label_selector = selector;
        self
    }

    pub fn scope_key(&self) -> String {
        let ns = self.namespace.as_deref().unwrap_or("all");
        let label = self.label_selector.as_deref().unwrap_or("");
        let field = self.field_selector.as_deref().unwrap_or("");
        format!("{}:{}:{}:{}", self.cluster_context, ns, label, field)
    }
}

/// Information about an active watch
#[derive(Debug)]
pub struct WatchInfo {
    pub scope: WatchScope,
    pub resource_type: String,
    pub last_accessed: Arc<Mutex<Instant>>,
    pub handle: JoinHandle<()>,
    pub resource_cache: Arc<RwLock<HashMap<String, K8sListItem>>>, // key = UID
    pub subscribers: Arc<Mutex<u32>>, // subscription count
}

impl WatchInfo {
    pub async fn touch(&self) {
        *self.last_accessed.lock().await = Instant::now();
    }

    pub async fn subscribe(&self) {
        *self.subscribers.lock().await += 1;
        self.touch().await;
    }

    pub async fn unsubscribe(&self) {
        let mut subs = self.subscribers.lock().await;
        if *subs > 0 {
            *subs -= 1;
        }
    }

    pub async fn subscriber_count(&self) -> u32 {
        *self.subscribers.lock().await
    }
}

/// Shared cache manager that maintains long-lived watches
pub struct SharedWatchCache {
    client: K8sClient,
    /// Active watches: (resource_type, scope_key) -> WatchInfo
    active_watches: Arc<Mutex<HashMap<(String, String), Arc<WatchInfo>>>>,
    /// Cleanup task handle
    cleanup_handle: Option<JoinHandle<()>>,
    /// Configuration
    idle_timeout: Duration,
    max_watches: usize,
}

impl SharedWatchCache {
    /// Create a new shared watch cache with hybrid reference + time cleanup
    /// 
    /// Uses a desktop-friendly approach that:
    /// - Keeps watches active while they have subscribers (like Freelens)
    /// - Only cleans up watches with 0 subscribers after 20 minutes of inactivity
    /// - Checks for cleanup every 5 minutes (less aggressive than server apps)
    pub fn new(client: K8sClient) -> Self {
        Self {
            client,
            active_watches: Arc::new(Mutex::new(HashMap::new())),
            cleanup_handle: None,
            idle_timeout: Duration::from_secs(1200), // 20 minutes (desktop-friendly)
            max_watches: 50,
        }
    }

    /// Start the cleanup task that removes idle watches
    pub fn start_cleanup_task(&mut self) {
        let watches = Arc::clone(&self.active_watches);
        let idle_timeout = self.idle_timeout;
        
        self.cleanup_handle = Some(tokio::spawn(async move {
            // Check every 5 minutes - more desktop-friendly with longer idle timeout
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            
            loop {
                interval.tick().await;
                Self::cleanup_idle_watches(&watches, idle_timeout).await;
            }
        }));
    }

    /// Subscribe to a resource type within a scope
    /// Returns immediately with cached data and starts watch if needed
    pub async fn subscribe(
        &self,
        app_handle: AppHandle,
        resource_type: String,
        scope: WatchScope,
    ) -> Result<Vec<K8sListItem>> {
        let scope_key = scope.scope_key();
        let cache_key = (resource_type.clone(), scope_key.clone());
        
        let mut watches = self.active_watches.lock().await;
        
        // Check if we already have this watch
        if let Some(watch_info) = watches.get(&cache_key) {
            watch_info.subscribe().await;
            
            // Return current cached data
            let cache = watch_info.resource_cache.read().await;
            return Ok(cache.values().cloned().collect());
        }
        
        // Check if we've hit the max watch limit
        if watches.len() >= self.max_watches {
            return Err(anyhow::anyhow!("Maximum watch limit reached: {}", self.max_watches));
        }
        
        // Start new watch
        let watch_info = self.start_new_watch(app_handle.clone(), resource_type.clone(), scope.clone()).await?;
        watch_info.subscribe().await;
        
        watches.insert(cache_key, watch_info.clone());
        
        // Fetch initial data before returning to populate cache with existing resources
        let initial_data = self.fetch_initial_data(resource_type, scope).await.unwrap_or_else(|e| {
            eprintln!("Warning: Failed to fetch initial data: {}", e);
            Vec::new()
        });
        
        // Populate the cache with initial data
        {
            let mut cache = watch_info.resource_cache.write().await;
            for item in &initial_data {
                if let Some(uid) = &item.metadata.uid {
                    cache.insert(uid.clone(), item.clone());
                }
            }
        }
        
        Ok(initial_data)
    }

    /// Unsubscribe from a resource watch
    pub async fn unsubscribe(&self, resource_type: String, scope: WatchScope) {
        let scope_key = scope.scope_key();
        let cache_key = (resource_type, scope_key);
        
        let watches = self.active_watches.lock().await;
        if let Some(watch_info) = watches.get(&cache_key) {
            watch_info.unsubscribe().await;
        }
    }

    /// Get current cached data without subscribing
    pub async fn get_cached_data(&self, resource_type: String, scope: WatchScope) -> Option<Vec<K8sListItem>> {
        let scope_key = scope.scope_key();
        let cache_key = (resource_type, scope_key);
        
        let watches = self.active_watches.lock().await;
        if let Some(watch_info) = watches.get(&cache_key) {
            let cache = watch_info.resource_cache.read().await;
            Some(cache.values().cloned().collect())
        } else {
            None
        }
    }

    /// Fetch initial data for a resource type and scope to populate cache
    async fn fetch_initial_data(&self, resource_type: String, scope: WatchScope) -> Result<Vec<K8sListItem>> {
        use kube::Api;
        use kube::api::ListParams;
        use k8s_openapi::api::core::v1::{Pod, Service, Node, ConfigMap, Secret, Namespace};
        use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet, StatefulSet, DaemonSet};
        use k8s_openapi::api::networking::v1::Ingress;
        
        let client = self.client.get_client().await?;
        let lp = ListParams::default();
        
        // Create appropriate API based on resource type and scope
        match resource_type.to_lowercase().as_str() {
            "pods" => {
                let api: Api<Pod> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "pods") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "services" => {
                let api: Api<Service> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "services") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "nodes" => {
                let api: Api<Node> = Api::all(client);
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "nodes") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "configmaps" => {
                let api: Api<ConfigMap> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "configmaps") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "secrets" => {
                let api: Api<Secret> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "secrets") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "namespaces" => {
                let api: Api<Namespace> = Api::all(client);
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "namespaces") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "deployments" => {
                let api: Api<Deployment> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "deployments") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "replicasets" => {
                let api: Api<ReplicaSet> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "replicasets") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "statefulsets" => {
                let api: Api<StatefulSet> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "statefulsets") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "daemonsets" => {
                let api: Api<DaemonSet> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "daemonsets") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            "ingresses" => {
                let api: Api<Ingress> = if let Some(namespace) = &scope.namespace {
                    Api::namespaced(client, namespace)
                } else {
                    Api::all(client)
                };
                
                let list = api.list(&lp).await?;
                let mut items = Vec::new();
                for resource in list.items {
                    if let Ok(item) = convert_to_list_item(&resource, "ingresses") {
                        items.push(item);
                    }
                }
                Ok(items)
            }
            _ => {
                // For unsupported resource types, return empty for now
                eprintln!("Unsupported resource type for initial data fetch: {}", resource_type);
                Ok(Vec::new())
            }
        }
    }

    /// Start a new watch for the given resource type and scope
    async fn start_new_watch(
        &self,
        app_handle: AppHandle,
        resource_type: String,
        scope: WatchScope,
    ) -> Result<Arc<WatchInfo>> {
        let client = self.client.get_client().await?;
        let resource_cache = Arc::new(RwLock::new(HashMap::new()));
        let last_accessed = Arc::new(Mutex::new(Instant::now()));
        let subscribers = Arc::new(Mutex::new(0u32));
        
        let handle = self.spawn_watch_task(
            app_handle,
            client,
            resource_type.clone(),
            scope.clone(),
            Arc::clone(&resource_cache),
        ).await?;
        
        Ok(Arc::new(WatchInfo {
            scope,
            resource_type,
            last_accessed,
            handle,
            resource_cache,
            subscribers,
        }))
    }

    /// Spawn the actual watch task for a specific resource type
    async fn spawn_watch_task(
        &self,
        app_handle: AppHandle,
        client: Client,
        resource_type: String,
        scope: WatchScope,
        cache: Arc<RwLock<HashMap<String, K8sListItem>>>,
    ) -> Result<JoinHandle<()>> {
        use k8s_openapi::api::{
            apps::v1::*,
            autoscaling::v2::*,
            batch::v1::*,
            certificates::v1::*,
            core::v1::*,
            discovery::v1::*,
            networking::v1::*,
            node::v1::*,
            policy::v1::*,
            rbac::v1::*,
            scheduling::v1::*,
            storage::v1::*,
        };
        use k8s_openapi::{
            apiextensions_apiserver::pkg::apis::apiextensions::v1::*,
            kube_aggregator::pkg::apis::apiregistration::v1::*,
        };

        // Helper macro to create namespaced APIs
        macro_rules! create_namespaced_api {
            ($resource_type:ty) => {
                if let Some(ns) = &scope.namespace {
                    Api::<$resource_type>::namespaced(client, ns)
                } else {
                    Api::<$resource_type>::all(client)
                }
            };
        }

        // Helper macro to create cluster-wide APIs
        macro_rules! create_cluster_api {
            ($resource_type:ty) => {
                Api::<$resource_type>::all(client)
            };
        }
        
        let handle = match resource_type.as_str() {
            // Workloads - Namespaced resources
            "pods" => {
                let api = create_namespaced_api!(Pod);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "deployments" => {
                let api = create_namespaced_api!(Deployment);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "statefulsets" => {
                let api = create_namespaced_api!(StatefulSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "daemonsets" => {
                let api = create_namespaced_api!(DaemonSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "jobs" => {
                let api = create_namespaced_api!(Job);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "cronjobs" => {
                let api = create_namespaced_api!(CronJob);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "replicasets" => {
                let api = create_namespaced_api!(ReplicaSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "replicationcontrollers" => {
                let api = create_namespaced_api!(ReplicationController);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Services & Networking - Namespaced resources
            "services" => {
                let api = create_namespaced_api!(Service);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "ingresses" => {
                let api = create_namespaced_api!(Ingress);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "networkpolicies" => {
                let api = create_namespaced_api!(NetworkPolicy);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "endpointslices" => {
                let api = create_namespaced_api!(EndpointSlice);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "endpoints" => {
                let api = create_namespaced_api!(Endpoints);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Configuration & Storage - Namespaced resources
            "configmaps" => {
                let api = create_namespaced_api!(ConfigMap);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "secrets" => {
                let api = create_namespaced_api!(Secret);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "persistentvolumeclaims" => {
                let api = create_namespaced_api!(PersistentVolumeClaim);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // RBAC - Namespaced resources
            "roles" => {
                let api = create_namespaced_api!(Role);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "rolebindings" => {
                let api = create_namespaced_api!(RoleBinding);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "serviceaccounts" => {
                let api = create_namespaced_api!(ServiceAccount);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Cluster Administration - Namespaced resources
            "resourcequotas" => {
                let api = create_namespaced_api!(ResourceQuota);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "limitranges" => {
                let api = create_namespaced_api!(LimitRange);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "poddisruptionbudgets" => {
                let api = create_namespaced_api!(PodDisruptionBudget);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Scaling & Performance - Namespaced resources
            "horizontalpodautoscalers" => {
                let api = create_namespaced_api!(HorizontalPodAutoscaler);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Services & Networking - Cluster-wide resources
            "ingressclasses" => {
                let api = create_cluster_api!(IngressClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Storage - Cluster-wide resources  
            "csidrivers" => {
                let api = create_cluster_api!(CSIDriver);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "csinodes" => {
                let api = create_cluster_api!(CSINode);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Cluster Administration - Cluster-wide resources
            "priorityclasses" => {
                let api = create_cluster_api!(PriorityClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "runtimeclasses" => {
                let api = create_cluster_api!(RuntimeClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Security & Access Control - Cluster-wide resources
            "certificatesigningrequests" => {
                let api = create_cluster_api!(CertificateSigningRequest);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Custom Resources - Cluster-wide resources
            "customresourcedefinitions" => {
                let api = create_cluster_api!(CustomResourceDefinition);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "apiservices" => {
                let api = create_cluster_api!(APIService);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            // Cluster-wide resources (original ones)
            "nodes" => {
                let api = create_cluster_api!(Node);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "namespaces" => {
                let api = create_cluster_api!(Namespace);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "persistentvolumes" => {
                let api = create_cluster_api!(PersistentVolume);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "storageclasses" => {
                let api = create_cluster_api!(StorageClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "clusterroles" => {
                let api = create_cluster_api!(ClusterRole);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            "clusterrolebindings" => {
                let api = create_cluster_api!(ClusterRoleBinding);
                Self::spawn_typed_watch(api, app_handle, resource_type, cache).await
            }
            
            _ => return Err(anyhow::anyhow!("Unsupported resource type for shared cache: {}", resource_type)),
        };
        
        Ok(handle)
    }

    /// Generic watch task spawner
    async fn spawn_typed_watch<K>(
        api: Api<K>,
        app_handle: AppHandle,
        resource_type: String,
        cache: Arc<RwLock<HashMap<String, K8sListItem>>>,
    ) -> JoinHandle<()>
    where
        K: kube::Resource<DynamicType = ()> + Clone + Send + Sync + 'static,
        K: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug,
        K: ResourceExt,
    {
        tokio::spawn(async move {
            let config = Config::default()
                .timeout(30)
                .any_semantic();
            
            let mut stream = watcher(api, config).boxed();
            
            loop {
                match tokio::time::timeout(Duration::from_secs(60), futures::StreamExt::next(&mut stream)).await {
                    Ok(Some(Ok(event))) => {
                        match event {
                            watcher::Event::Apply(obj) => {
                                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                                    // Update cache
                                    if let Some(uid) = obj.uid() {
                                        cache.write().await.insert(uid.clone(), item.clone());
                                    }
                                    
                                    // Emit to UI
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added(item));
                                }
                            }
                            watcher::Event::Delete(obj) => {
                                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                                    // Remove from cache
                                    if let Some(uid) = obj.uid() {
                                        cache.write().await.remove(&uid);
                                    }
                                    
                                    // Emit to UI
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Deleted(item));
                                }
                            }
                            watcher::Event::InitApply(obj) => {
                                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                                    // Add to cache during initial sync
                                    if let Some(uid) = obj.uid() {
                                        cache.write().await.insert(uid.clone(), item.clone());
                                    }
                                    
                                    // Emit to UI
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added(item));
                                }
                            }
                            watcher::Event::Init => {
                                // Clear cache at start of initial sync
                                cache.write().await.clear();
                            }
                            watcher::Event::InitDone => {
                                // Initial sync complete - emit event to notify UI
                                let _ = app_handle.emit("k8s-watch-event", WatchEvent::InitialSyncComplete);
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        eprintln!("Watch error for {resource_type}: {e:?}");
                        // Could implement exponential backoff here
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    Ok(None) => {
                        // Stream ended
                        break;
                    }
                    Err(_) => {
                        // Timeout - this is normal, continue
                        continue;
                    }
                }
            }
        })
    }

    /// Clean up idle watches using hybrid reference + time strategy
    /// Only removes watches that have:
    /// 1. Zero subscribers (reference-based like Freelens)
    /// 2. Been idle for the timeout period (time-based protection)
    async fn cleanup_idle_watches(
        watches: &Arc<Mutex<HashMap<(String, String), Arc<WatchInfo>>>>,
        idle_timeout: Duration,
    ) {
        let mut to_remove = Vec::new();
        let now = Instant::now();
        
        {
            let watch_map = watches.lock().await;
            for (key, watch_info) in watch_map.iter() {
                let subscriber_count = watch_info.subscriber_count().await;
                
                // Only consider watches with no active subscribers (Freelens-like behavior)
                if subscriber_count == 0 {
                    let last_accessed = *watch_info.last_accessed.lock().await;
                    
                    // Remove if idle for the configured timeout period
                    if now.duration_since(last_accessed) > idle_timeout {
                        to_remove.push(key.clone());
                    }
                }
                // Watches with active subscribers are never cleaned up automatically
            }
        }
        
        if !to_remove.is_empty() {
            let mut watch_map = watches.lock().await;
            for key in to_remove {
                if let Some(watch_info) = watch_map.remove(&key) {
                    watch_info.handle.abort();
                    println!("🧹 Cleaned up idle watch (0 subscribers, {}min idle): {:?}", 
                             idle_timeout.as_secs() / 60, key);
                }
            }
        }
    }

    /// Stop all watches and cleanup
    pub async fn shutdown(&self) {
        if let Some(handle) = &self.cleanup_handle {
            handle.abort();
        }
        
        let mut watches = self.active_watches.lock().await;
        for (_, watch_info) in watches.drain() {
            watch_info.handle.abort();
        }
    }
}

impl Drop for SharedWatchCache {
    fn drop(&mut self) {
        if let Some(handle) = &self.cleanup_handle {
            handle.abort();
        }
    }
}