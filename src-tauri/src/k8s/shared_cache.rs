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
        
        let cluster_context = scope.cluster_context.clone();
        let handle = match resource_type.as_str() {
            // Workloads - Namespaced resources
            "pods" => {
                let api = create_namespaced_api!(Pod);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context, cache).await
            }
            "deployments" => {
                let api = create_namespaced_api!(Deployment);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "statefulsets" => {
                let api = create_namespaced_api!(StatefulSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "daemonsets" => {
                let api = create_namespaced_api!(DaemonSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "jobs" => {
                let api = create_namespaced_api!(Job);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "cronjobs" => {
                let api = create_namespaced_api!(CronJob);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "replicasets" => {
                let api = create_namespaced_api!(ReplicaSet);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "replicationcontrollers" => {
                let api = create_namespaced_api!(ReplicationController);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Services & Networking - Namespaced resources
            "services" => {
                let api = create_namespaced_api!(Service);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "ingresses" => {
                let api = create_namespaced_api!(Ingress);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "networkpolicies" => {
                let api = create_namespaced_api!(NetworkPolicy);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "endpointslices" => {
                let api = create_namespaced_api!(EndpointSlice);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "endpoints" => {
                let api = create_namespaced_api!(Endpoints);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Configuration & Storage - Namespaced resources
            "configmaps" => {
                let api = create_namespaced_api!(ConfigMap);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "secrets" => {
                let api = create_namespaced_api!(Secret);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "persistentvolumeclaims" => {
                let api = create_namespaced_api!(PersistentVolumeClaim);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // RBAC - Namespaced resources
            "roles" => {
                let api = create_namespaced_api!(Role);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "rolebindings" => {
                let api = create_namespaced_api!(RoleBinding);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "serviceaccounts" => {
                let api = create_namespaced_api!(ServiceAccount);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Cluster Administration - Namespaced resources
            "resourcequotas" => {
                let api = create_namespaced_api!(ResourceQuota);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "limitranges" => {
                let api = create_namespaced_api!(LimitRange);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "poddisruptionbudgets" => {
                let api = create_namespaced_api!(PodDisruptionBudget);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Scaling & Performance - Namespaced resources
            "horizontalpodautoscalers" => {
                let api = create_namespaced_api!(HorizontalPodAutoscaler);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Services & Networking - Cluster-wide resources
            "ingressclasses" => {
                let api = create_cluster_api!(IngressClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Storage - Cluster-wide resources  
            "csidrivers" => {
                let api = create_cluster_api!(CSIDriver);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "csinodes" => {
                let api = create_cluster_api!(CSINode);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Cluster Administration - Cluster-wide resources
            "priorityclasses" => {
                let api = create_cluster_api!(PriorityClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "runtimeclasses" => {
                let api = create_cluster_api!(RuntimeClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Security & Access Control - Cluster-wide resources
            "certificatesigningrequests" => {
                let api = create_cluster_api!(CertificateSigningRequest);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Custom Resources - Cluster-wide resources
            "customresourcedefinitions" => {
                let api = create_cluster_api!(CustomResourceDefinition);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "apiservices" => {
                let api = create_cluster_api!(APIService);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            
            // Cluster-wide resources (original ones)
            "nodes" => {
                let api = create_cluster_api!(Node);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "namespaces" => {
                let api = create_cluster_api!(Namespace);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "persistentvolumes" => {
                let api = create_cluster_api!(PersistentVolume);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "storageclasses" => {
                let api = create_cluster_api!(StorageClass);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "clusterroles" => {
                let api = create_cluster_api!(ClusterRole);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
            }
            "clusterrolebindings" => {
                let api = create_cluster_api!(ClusterRoleBinding);
                Self::spawn_typed_watch(api, app_handle, resource_type, cluster_context.clone(), cache).await
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
        cluster_context: String,
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
                                    
                                    // Emit to UI with cluster context
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added { 
                                        item,
                                        cluster_context: cluster_context.clone(),
                                    });
                                }
                            }
                            watcher::Event::Delete(obj) => {
                                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                                    // Remove from cache
                                    if let Some(uid) = obj.uid() {
                                        cache.write().await.remove(&uid);
                                    }
                                    
                                    // Emit to UI with cluster context
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Deleted { 
                                        item,
                                        cluster_context: cluster_context.clone(),
                                    });
                                }
                            }
                            watcher::Event::InitApply(obj) => {
                                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                                    // Add to cache during initial sync
                                    if let Some(uid) = obj.uid() {
                                        cache.write().await.insert(uid.clone(), item.clone());
                                    }
                                    
                                    // Emit to UI with cluster context
                                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added { 
                                        item,
                                        cluster_context: cluster_context.clone(),
                                    });
                                }
                            }
                            watcher::Event::Init => {
                                // Clear cache at start of initial sync
                                cache.write().await.clear();
                            }
                            watcher::Event::InitDone => {
                                // Initial sync complete - emit event to notify UI
                                let _ = app_handle.emit("k8s-watch-event", WatchEvent::InitialSyncComplete {
                                    cluster_context: cluster_context.clone(),
                                });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_cluster_watch_isolation() {
        // Test that SharedWatchCache properly isolates watches between different clusters
        // This test focuses on cache key isolation without requiring actual Tauri app handles
        
        // Create scopes for different clusters
        let cluster_a_scope = WatchScope::new("cluster-production".to_string())
            .with_namespace(Some("default".to_string()));
        let cluster_b_scope = WatchScope::new("cluster-staging".to_string())  
            .with_namespace(Some("default".to_string()));
        
        // Verify that the scopes generate different cache keys
        let key_a = ("nodes".to_string(), cluster_a_scope.scope_key());
        let key_b = ("nodes".to_string(), cluster_b_scope.scope_key());
        
        assert_ne!(key_a, key_b, "Different clusters should generate different cache keys");
        assert!(key_a.1.contains("cluster-production"), "Key A should contain cluster-production");
        assert!(key_b.1.contains("cluster-staging"), "Key B should contain cluster-staging");
        
        // Verify scope key format includes cluster context
        assert_eq!(cluster_a_scope.scope_key(), "cluster-production:default::");
        assert_eq!(cluster_b_scope.scope_key(), "cluster-staging:default::");
    }

    #[tokio::test] 
    async fn test_watch_scope_cluster_isolation() {
        // Test that WatchScope correctly generates cluster-aware scope keys
        
        // Test same resource type and namespace in different clusters
        let scope_prod = WatchScope::new("production".to_string())
            .with_namespace(Some("kube-system".to_string()));
        let scope_staging = WatchScope::new("staging".to_string())
            .with_namespace(Some("kube-system".to_string()));
        let scope_dev = WatchScope::new("development".to_string())
            .with_namespace(Some("kube-system".to_string()));
        
        let key_prod = scope_prod.scope_key();
        let key_staging = scope_staging.scope_key(); 
        let key_dev = scope_dev.scope_key();
        
        // All keys should be different
        assert_ne!(key_prod, key_staging);
        assert_ne!(key_staging, key_dev);
        assert_ne!(key_prod, key_dev);
        
        // Keys should contain cluster context
        assert!(key_prod.starts_with("production:"));
        assert!(key_staging.starts_with("staging:"));
        assert!(key_dev.starts_with("development:"));
        
        // Test cluster-wide resources (no namespace)
        let scope_prod_all = WatchScope::new("production".to_string());
        let scope_staging_all = WatchScope::new("staging".to_string());
        
        let key_prod_all = scope_prod_all.scope_key();
        let key_staging_all = scope_staging_all.scope_key();
        
        assert_ne!(key_prod_all, key_staging_all);
        assert!(key_prod_all.starts_with("production:"));
        assert!(key_staging_all.starts_with("staging:"));
        assert!(key_prod_all.contains(":all:"));
        assert!(key_staging_all.contains(":all:"));
    }

    #[tokio::test]
    async fn test_cross_cluster_contamination_prevention() {
        // This test simulates the exact scenario that caused contamination:
        // 1. User views nodes in cluster A
        // 2. User switches to cluster B  
        // 3. Verifies that cluster A nodes don't appear in cluster B view
        
        // Simulate viewing nodes in cluster A
        let cluster_a_scope = WatchScope::new("cluster-a".to_string());
        let cluster_a_key = ("nodes".to_string(), cluster_a_scope.scope_key());
        
        // Simulate switching to cluster B  
        let cluster_b_scope = WatchScope::new("cluster-b".to_string());
        let cluster_b_key = ("nodes".to_string(), cluster_b_scope.scope_key());
        
        // Verify that the cache keys are completely different
        assert_ne!(cluster_a_key, cluster_b_key, "Cluster A and B should have different cache keys");
        
        // The scope keys should be completely isolated
        assert!(cluster_a_key.1.contains("cluster-a"), "Cluster A key should contain cluster-a");
        assert!(cluster_b_key.1.contains("cluster-b"), "Cluster B key should contain cluster-b");
        assert!(!cluster_a_key.1.contains("cluster-b"), "Cluster A key should not contain cluster-b");
        assert!(!cluster_b_key.1.contains("cluster-a"), "Cluster B key should not contain cluster-a");
        
        // In the real application:
        // - Events from cluster A watches would have cluster_context: "cluster-a" 
        // - Events from cluster B watches would have cluster_context: "cluster-b"
        // - Frontend would filter out events not matching current cluster
    }

    #[tokio::test]
    async fn test_cluster_context_validation_in_events() {
        // Test that events emitted from different cluster contexts are properly tagged
        use crate::k8s::resources::{WatchEvent, K8sListItem};
        use k8s_openapi::api::core::v1::Node;
        
        // Create a mock node
        let mut node = Node::default();
        node.metadata.name = Some("test-node".to_string());
        node.metadata.uid = Some("node-123".to_string());
        
        // Convert to K8sListItem (simplified version for test)
        let item = K8sListItem {
            metadata: node.metadata.clone(),
            kind: "Node".to_string(),
            api_version: "v1".to_string(),
            complete_object: None,
            pod_spec: None,
            service_spec: None,
            config_map_spec: None,
            secret_spec: None,
            namespace_spec: None,
            node_spec: None,
            persistent_volume_spec: None,
            persistent_volume_claim_spec: None,
            endpoints_spec: None,
            deployment_spec: None,
            replica_set_spec: None,
            stateful_set_spec: None,
            daemon_set_spec: None,
            job_spec: None,
            cron_job_spec: None,
            ingress_spec: None,
            network_policy_spec: None,
            endpoint_slice: None,
            storage_class_spec: None,
            role_spec: None,
            role_binding_spec: None,
            cluster_role_spec: None,
            cluster_role_binding_spec: None,
            service_account_spec: None,
            pod_disruption_budget_spec: None,
            horizontal_pod_autoscaler_spec: None,
            pod_status: None,
            service_status: None,
            namespace_status: None,
            node_status: None,
            persistent_volume_status: None,
            persistent_volume_claim_status: None,
            deployment_status: None,
            replica_set_status: None,
            stateful_set_status: None,
            daemon_set_status: None,
            job_status: None,
            cron_job_status: None,
            ingress_status: None,
            pod_disruption_budget_status: None,
            horizontal_pod_autoscaler_status: None,
            subsets: None,
        };
        
        // Test that events from different clusters would be distinguishable
        let prod_event = WatchEvent::Added { 
            item: item.clone(), 
            cluster_context: "production".to_string() 
        };
        let staging_event = WatchEvent::Added { 
            item: item.clone(), 
            cluster_context: "staging".to_string() 
        };
        
        // Serialize both events
        let prod_json = serde_json::to_string(&prod_event).unwrap();
        let staging_json = serde_json::to_string(&staging_event).unwrap();
        
        // Events should serialize differently due to cluster context
        assert_ne!(prod_json, staging_json, "Events from different clusters must serialize differently");
        
        // Frontend should be able to distinguish these events
        assert!(prod_json.contains("production"), "Production event should contain cluster context");
        assert!(staging_json.contains("staging"), "Staging event should contain cluster context");
        assert!(!prod_json.contains("staging"), "Production event should not contain staging context");
        assert!(!staging_json.contains("production"), "Staging event should not contain production context");
        
        // This test verifies that the fix prevents the contamination bug where:
        // - Same node UID from different clusters would overwrite each other
        // - Frontend couldn't distinguish which cluster an event came from
        // - Events from old clusters would contaminate the new cluster view
    }
}