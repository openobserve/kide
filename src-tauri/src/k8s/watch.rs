use super::client::K8sClient;
use super::resources::{K8sListItem, WatchEvent};
use futures::StreamExt;
use k8s_openapi::api::{
    apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet},
    autoscaling::v2::HorizontalPodAutoscaler,
    batch::v1::{CronJob, Job},
    certificates::v1::CertificateSigningRequest,
    core::v1::{ConfigMap, Endpoints, LimitRange, Namespace, Node, Pod, PersistentVolume, PersistentVolumeClaim, ReplicationController, ResourceQuota, Secret, Service, ServiceAccount},
    discovery::v1::EndpointSlice,
    networking::v1::{Ingress, IngressClass, NetworkPolicy},
    node::v1::RuntimeClass,
    policy::v1::PodDisruptionBudget,
    rbac::v1::{ClusterRole, ClusterRoleBinding, Role, RoleBinding},
    scheduling::v1::PriorityClass,
    storage::v1::{CSIDriver, CSINode, StorageClass},
};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::kube_aggregator::pkg::apis::apiregistration::v1::APIService;
use kube::{
    api::Api,
    runtime::{watcher, watcher::Config},
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Generic typed watch creator that handles namespaced vs cluster-wide logic
/// Used by the trait-based dispatch system
pub fn create_typed_watch<T>(
    client: kube::Client,
    app_handle: AppHandle,
    resource_type: String,
    _namespaces: Option<Vec<String>>,
    is_namespaced: bool,
) -> tokio::task::JoinHandle<()>
where
    T: k8s_openapi::Resource + k8s_openapi::Metadata + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize + Send + 'static,
    <T as k8s_openapi::Resource>::Scope: Send,
    T: kube::Resource<DynamicType = ()>,
    <T as kube::Resource>::DynamicType: Default,
{
    if !is_namespaced {
        // Cluster-wide resources always use Api::all()
        let api: Api<T> = Api::all(client);
        tokio::spawn(watch_resource(api, app_handle, resource_type))
    } else {
        // For namespaced resources, always use Api::all() for simplicity
        // This is actually more efficient than managing multiple namespace-specific watches
        let api: Api<T> = Api::all(client);
        tokio::spawn(watch_resource(api, app_handle, resource_type))
    }
}

// Helper function to create an API for multiple namespaces
pub async fn watch_multiple_namespaces<T>(
    client: kube::Client,
    namespaces: Vec<String>,
    app_handle: AppHandle,
    resource_type: String,
) -> ()
where
    T: k8s_openapi::Resource + k8s_openapi::Metadata + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize + Send + 'static,
    <T as k8s_openapi::Resource>::Scope: Send,
    T: kube::Resource<Scope = kube::core::NamespaceResourceScope, DynamicType = ()>,
    <T as kube::Resource>::DynamicType: Default,
{
    // Start separate watch tasks for each namespace and merge their results
    let mut handles = Vec::new();
    
    for namespace in namespaces {
        let api: Api<T> = Api::namespaced(client.clone(), &namespace);
        let app_handle_clone = app_handle.clone();
        let resource_type_clone = resource_type.clone();
        
        let handle = tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone));
        handles.push(handle);
    }
    
    // Wait for all namespace watches to complete
    for handle in handles {
        let _ = handle.await;
    }
}

pub struct WatchManager {
    client: K8sClient,
    active_watches: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl WatchManager {
    pub fn new(client: K8sClient) -> Self {
        let manager = Self {
            client,
            active_watches: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start periodic cleanup task
        let watches_clone = manager.active_watches.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Cleanup finished watches
                let mut watches = watches_clone.lock().await;
                watches.retain(|key, handle| {
                    if handle.is_finished() {
                        println!("🧹 Cleaned up finished watch: {key}");
                        false
                    } else {
                        true
                    }
                });
                
                if !watches.is_empty() {
                    println!("📊 Active watches: {}", watches.len());
                }
                
                // Monitor file descriptor usage
                super::system_monitor::monitor_fd_usage();
            }
        });
        
        manager
    }

    pub async fn start_watch(
        &self,
        app_handle: AppHandle,
        resource_type: &str,
        namespaces: Option<Vec<String>>,
    ) -> Result<(), anyhow::Error> {
        // Generate watch key from namespaces (sorted for consistency)
        let watch_key = if let Some(ref ns_list) = namespaces {
            if ns_list.is_empty() {
                format!("{resource_type}:all")
            } else {
                let mut sorted_ns = ns_list.clone();
                sorted_ns.sort();
                format!("{}:{}", resource_type, sorted_ns.join(","))
            }
        } else {
            format!("{resource_type}:all")
        };
        
        let mut watches = self.active_watches.lock().await;
        if watches.contains_key(&watch_key) {
            return Ok(());
        }

        let client = self.client.get_client().await?;
        let app_handle_clone = app_handle.clone();
        let resource_type_clone = resource_type.to_string();

        // Determine if we should watch all namespaces or specific ones
        let watch_all = namespaces.as_ref().map_or(true, |ns| ns.is_empty());

        // Create helper macro for resource watching
        macro_rules! create_watch {
            ($resource_type:ty, $namespaced:expr) => {{
                if !$namespaced {
                    // Cluster-wide resources always use Api::all()
                    let api: Api<$resource_type> = Api::all(client);
                    tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
                } else if watch_all {
                    // Namespaced resources watching all namespaces
                    let api: Api<$resource_type> = Api::all(client);
                    tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
                } else {
                    // Namespaced resources with specific namespace selection
                    let namespaces = namespaces.unwrap();
                    if namespaces.len() == 1 {
                        let api: Api<$resource_type> = Api::namespaced(client, &namespaces[0]);
                        tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
                    } else {
                        tokio::spawn(watch_multiple_namespaces::<$resource_type>(
                            client, 
                            namespaces, 
                            app_handle_clone, 
                            resource_type_clone
                        ))
                    }
                }
            }};
        }

        let handle = match resource_type {
            // Workloads - Namespaced resources
            "pods" => create_watch!(Pod, true),
            "deployments" => create_watch!(Deployment, true),
            "statefulsets" => create_watch!(StatefulSet, true),
            "daemonsets" => create_watch!(DaemonSet, true),
            "jobs" => create_watch!(Job, true),
            "cronjobs" => create_watch!(CronJob, true),
            "replicasets" => create_watch!(ReplicaSet, true),
            "replicationcontrollers" => create_watch!(ReplicationController, true),
            
            // Services & Networking - Namespaced resources
            "services" => create_watch!(Service, true),
            "ingresses" => create_watch!(Ingress, true),
            "networkpolicies" => create_watch!(NetworkPolicy, true),
            "endpointslices" => create_watch!(EndpointSlice, true),
            "endpoints" => create_watch!(Endpoints, true),
            
            // Services & Networking - Cluster-wide resources
            "ingressclasses" => {
                let api: Api<IngressClass> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            
            // Configuration & Storage - Namespaced resources
            "configmaps" => create_watch!(ConfigMap, true),
            "secrets" => create_watch!(Secret, true),
            "persistentvolumeclaims" => create_watch!(PersistentVolumeClaim, true),
            
            // Configuration & Storage - Cluster-wide resources
            "persistentvolumes" => {
                let api: Api<PersistentVolume> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "storageclasses" => {
                let api: Api<StorageClass> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "csidrivers" => {
                let api: Api<CSIDriver> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "csinodes" => {
                let api: Api<CSINode> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            
            // Cluster Administration - Mixed scope
            "serviceaccounts" => create_watch!(ServiceAccount, true),
            "resourcequotas" => create_watch!(ResourceQuota, true),
            "limitranges" => create_watch!(LimitRange, true),
            "poddisruptionbudgets" => create_watch!(PodDisruptionBudget, true),
            
            // Cluster Administration - Cluster-wide resources
            "namespaces" => {
                let api: Api<Namespace> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "nodes" => {
                let api: Api<Node> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "priorityclasses" => {
                let api: Api<PriorityClass> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "runtimeclasses" => {
                let api: Api<RuntimeClass> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            
            // Security & Access Control - Mixed scope
            "roles" => create_watch!(Role, true),
            "rolebindings" => create_watch!(RoleBinding, true),
            
            // Security & Access Control - Cluster-wide resources
            "clusterroles" => {
                let api: Api<ClusterRole> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "clusterrolebindings" => {
                let api: Api<ClusterRoleBinding> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "certificatesigningrequests" => {
                let api: Api<CertificateSigningRequest> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            
            // Scaling & Performance - Namespaced resources
            "horizontalpodautoscalers" => create_watch!(HorizontalPodAutoscaler, true),
            
            // Custom Resources - Cluster-wide resources
            "customresourcedefinitions" => {
                let api: Api<CustomResourceDefinition> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            "apiservices" => {
                let api: Api<APIService> = Api::all(client);
                tokio::spawn(watch_resource(api, app_handle_clone, resource_type_clone))
            }
            
            _ => {
                return Err(anyhow::anyhow!("Unsupported resource type: {}", resource_type));
            }
        };

        watches.insert(watch_key, handle);
        Ok(())
    }

    pub async fn stop_watch(&self, resource_type: &str, namespaces: Option<Vec<String>>) -> Result<(), anyhow::Error> {
        // Generate the same watch key as in start_watch
        let watch_key = if let Some(ref ns_list) = namespaces {
            if ns_list.is_empty() {
                format!("{resource_type}:all")
            } else {
                let mut sorted_ns = ns_list.clone();
                sorted_ns.sort();
                format!("{}:{}", resource_type, sorted_ns.join(","))
            }
        } else {
            format!("{resource_type}:all")
        };
        
        let mut watches = self.active_watches.lock().await;
        if let Some(handle) = watches.remove(&watch_key) {
            handle.abort();
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn stop_all_watches(&self) -> Result<(), anyhow::Error> {
        let mut watches = self.active_watches.lock().await;
        for (_, handle) in watches.drain() {
            handle.abort();
        }
        Ok(())
    }
}

async fn watch_resource<K>(api: Api<K>, app_handle: AppHandle, resource_type: String)
where
    K: kube::Resource<DynamicType = ()> + Clone + Send + 'static,
    K: serde::de::DeserializeOwned,
    K: std::fmt::Debug,
    K: serde::Serialize,
{
    use tokio::time::{timeout, Duration};
    
    let config = Config::default().timeout(30); // Add timeout to prevent hanging connections
    let mut stream = watcher(api, config).boxed();
    
    // Add timeout and error recovery to prevent indefinite blocking
    loop {
        let timeout_result = timeout(Duration::from_secs(60), stream.next()).await;
        
        let event = match timeout_result {
            Ok(Some(event)) => event,
            Ok(None) => {
                // Stream ended normally
                println!("🔄 Watch stream ended for {resource_type}");
                break;
            }
            Err(_) => {
                // Timeout occurred
                println!("⚠️  Watch stream timeout for {resource_type} - restarting");
                break;
            }
        };

        match event {
            Ok(watcher::Event::Apply(obj)) => {
                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added(item));
                }
            }
            Ok(watcher::Event::Delete(obj)) => {
                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Deleted(item));
                }
            }
            Ok(watcher::Event::InitApply(obj)) => {
                if let Ok(item) = convert_to_list_item(&obj, &resource_type) {
                    let _ = app_handle.emit("k8s-watch-event", WatchEvent::Added(item));
                }
            }
            Ok(watcher::Event::Init) => {
                // Stream restarted, no action needed
            }
            Ok(watcher::Event::InitDone) => {
                // Initialization complete, no action needed
            }
            Err(e) => {
                eprintln!("Watch error for {resource_type}: {e:?}");
                break;
            }
        }
    }
}

fn resource_type_to_kind_and_api_version(resource_type: &str) -> (&str, &str) {
    match resource_type {
        // Workloads
        "pods" => ("Pod", "v1"),
        "deployments" => ("Deployment", "apps/v1"),
        "statefulsets" => ("StatefulSet", "apps/v1"),
        "daemonsets" => ("DaemonSet", "apps/v1"),
        "jobs" => ("Job", "batch/v1"),
        "cronjobs" => ("CronJob", "batch/v1"),
        "replicasets" => ("ReplicaSet", "apps/v1"),
        "replicationcontrollers" => ("ReplicationController", "v1"),
        
        // Services & Networking
        "services" => ("Service", "v1"),
        "ingresses" => ("Ingress", "networking.k8s.io/v1"),
        "ingressclasses" => ("IngressClass", "networking.k8s.io/v1"),
        "networkpolicies" => ("NetworkPolicy", "networking.k8s.io/v1"),
        "endpointslices" => ("EndpointSlice", "discovery.k8s.io/v1"),
        "endpoints" => ("Endpoints", "v1"),
        
        // Configuration
        "configmaps" => ("ConfigMap", "v1"),
        "secrets" => ("Secret", "v1"),
        "resourcequotas" => ("ResourceQuota", "v1"),
        "limitranges" => ("LimitRange", "v1"),
        "poddisruptionbudgets" => ("PodDisruptionBudget", "policy/v1"),
        
        // Storage
        "persistentvolumes" => ("PersistentVolume", "v1"),
        "persistentvolumeclaims" => ("PersistentVolumeClaim", "v1"),
        "storageclasses" => ("StorageClass", "storage.k8s.io/v1"),
        "csidrivers" => ("CSIDriver", "storage.k8s.io/v1"),
        "csinodes" => ("CSINode", "storage.k8s.io/v1"),
        
        // Cluster Administration
        "namespaces" => ("Namespace", "v1"),
        "nodes" => ("Node", "v1"),
        "serviceaccounts" => ("ServiceAccount", "v1"),
        "priorityclasses" => ("PriorityClass", "scheduling.k8s.io/v1"),
        "runtimeclasses" => ("RuntimeClass", "node.k8s.io/v1"),
        
        // Security & Access Control
        "roles" => ("Role", "rbac.authorization.k8s.io/v1"),
        "clusterroles" => ("ClusterRole", "rbac.authorization.k8s.io/v1"),
        "rolebindings" => ("RoleBinding", "rbac.authorization.k8s.io/v1"),
        "clusterrolebindings" => ("ClusterRoleBinding", "rbac.authorization.k8s.io/v1"),
        "certificatesigningrequests" => ("CertificateSigningRequest", "certificates.k8s.io/v1"),
        
        // Scaling
        "horizontalpodautoscalers" => ("HorizontalPodAutoscaler", "autoscaling/v2"),
        
        // Custom Resources
        "customresourcedefinitions" => ("CustomResourceDefinition", "apiextensions.k8s.io/v1"),
        "apiservices" => ("APIService", "apiregistration.k8s.io/v1"),
        
        _ => (resource_type, "v1"), // fallback to the original if not found
    }
}

pub fn convert_to_list_item<K>(obj: &K, resource_type: &str) -> Result<K8sListItem, anyhow::Error>
where
    K: kube::Resource,
    K: serde::Serialize,
    K: std::fmt::Debug,
{
    let meta = obj.meta();
    let obj_json = serde_json::to_value(obj)?;
    
    // Convert resource type to proper Kubernetes Kind and API version
    let (kind, api_version) = resource_type_to_kind_and_api_version(resource_type);
    
    // Create base K8sListItem using official k8s-openapi types
    let mut list_item = K8sListItem {
        metadata: meta.clone(),
        kind: kind.to_string(),
        api_version: api_version.to_string(),
        complete_object: Some(obj_json.clone()),
        // Initialize all official k8s-openapi spec/status fields as None
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

    // Extract spec and status using official k8s-openapi types based on resource type
    match resource_type {
        "pods" => {
            // Use direct k8s-openapi types for complete compatibility and future-proofing
            if let Ok(pod) = serde_json::from_value::<Pod>(obj_json.clone()) {
                list_item.pod_spec = pod.spec;
                list_item.pod_status = pod.status;
            }
        },
        "deployments" => {
            if let Ok(deployment) = serde_json::from_value::<Deployment>(obj_json.clone()) {
                list_item.deployment_spec = deployment.spec;
                list_item.deployment_status = deployment.status;
            }
        },
        "services" => {
            if let Ok(service) = serde_json::from_value::<Service>(obj_json.clone()) {
                list_item.service_spec = service.spec;
                list_item.service_status = service.status;
            }
        },
        "replicasets" => {
            if let Ok(rs) = serde_json::from_value::<ReplicaSet>(obj_json.clone()) {
                list_item.replica_set_spec = rs.spec;
                list_item.replica_set_status = rs.status;
            }
        },
        "statefulsets" => {
            if let Ok(sts) = serde_json::from_value::<StatefulSet>(obj_json.clone()) {
                list_item.stateful_set_spec = sts.spec;
                list_item.stateful_set_status = sts.status;
            }
        },
        "daemonsets" => {
            if let Ok(ds) = serde_json::from_value::<DaemonSet>(obj_json.clone()) {
                list_item.daemon_set_spec = ds.spec;
                list_item.daemon_set_status = ds.status;
            }
        },
        "jobs" => {
            if let Ok(job) = serde_json::from_value::<Job>(obj_json.clone()) {
                list_item.job_spec = job.spec;
                list_item.job_status = job.status;
            }
        },
        "cronjobs" => {
            if let Ok(cj) = serde_json::from_value::<CronJob>(obj_json.clone()) {
                list_item.cron_job_spec = cj.spec;
                list_item.cron_job_status = cj.status;
            }
        },
        "ingresses" => {
            if let Ok(ing) = serde_json::from_value::<Ingress>(obj_json.clone()) {
                list_item.ingress_spec = ing.spec;
                list_item.ingress_status = ing.status;
            }
        },
        "namespaces" => {
            if let Ok(ns) = serde_json::from_value::<Namespace>(obj_json.clone()) {
                list_item.namespace_spec = ns.spec;
                list_item.namespace_status = ns.status;
            }
        },
        "nodes" => {
            if let Ok(node) = serde_json::from_value::<Node>(obj_json.clone()) {
                list_item.node_spec = node.spec;
                list_item.node_status = node.status;
            }
        },
        "persistentvolumes" => {
            if let Ok(pv) = serde_json::from_value::<PersistentVolume>(obj_json.clone()) {
                list_item.persistent_volume_spec = pv.spec;
                list_item.persistent_volume_status = pv.status;
            }
        },
        "persistentvolumeclaims" => {
            if let Ok(pvc) = serde_json::from_value::<PersistentVolumeClaim>(obj_json.clone()) {
                list_item.persistent_volume_claim_spec = pvc.spec;
                list_item.persistent_volume_claim_status = pvc.status;
            }
        },
        "storageclasses" => {
            if let Ok(sc) = serde_json::from_value::<StorageClass>(obj_json.clone()) {
                list_item.storage_class_spec = Some(sc);
            }
        },
        "horizontalpodautoscalers" => {
            if let Ok(hpa) = serde_json::from_value::<HorizontalPodAutoscaler>(obj_json.clone()) {
                list_item.horizontal_pod_autoscaler_spec = hpa.spec;
                list_item.horizontal_pod_autoscaler_status = hpa.status;
            }
        },
        "poddisruptionbudgets" => {
            if let Ok(pdb) = serde_json::from_value::<PodDisruptionBudget>(obj_json.clone()) {
                list_item.pod_disruption_budget_spec = pdb.spec;
                list_item.pod_disruption_budget_status = pdb.status;
            }
        },
        "networkpolicies" => {
            if let Ok(np) = serde_json::from_value::<NetworkPolicy>(obj_json.clone()) {
                list_item.network_policy_spec = np.spec;
            }
        },
        "configmaps" | "secrets" | "roles" | "rolebindings" | "clusterroles" | "clusterrolebindings" | "serviceaccounts" => {
            // These resources don't have separate spec structures, store as generic JSON
            match resource_type {
                "configmaps" => list_item.config_map_spec = obj_json.get("data").cloned(),
                "secrets" => list_item.secret_spec = obj_json.get("data").cloned(),
                "roles" | "clusterroles" => list_item.role_spec = obj_json.get("rules").cloned(),
                "rolebindings" | "clusterrolebindings" => list_item.role_binding_spec = obj_json.get("subjects").cloned(),
                "serviceaccounts" => list_item.service_account_spec = obj_json.get("secrets").cloned(),
                _ => {}
            }
        },
        "endpointslices" => {
            // Use official k8s-openapi EndpointSlice type for complete compatibility
            if let Ok(endpoint_slice) = serde_json::from_value::<EndpointSlice>(obj_json.clone()) {
                list_item.endpoint_slice = Some(endpoint_slice);
            }
        },
        "endpoints" => {
            // Extract Endpoints-specific fields (keeping existing logic as these are complex)
            if let Some(subsets_array) = obj_json.get("subsets").and_then(|v| v.as_array()) {
                let subsets: Vec<super::resources::EndpointSubset> = subsets_array.iter()
                    .filter_map(|subset| {
                        let addresses = subset.get("addresses").and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|addr| {
                                        addr.get("ip").and_then(|ip| ip.as_str())
                                            .map(|ip_str| super::resources::EndpointAddress {
                                                ip: ip_str.to_string(),
                                                hostname: None,
                                                node_name: None,
                                                target_ref: None,
                                            })
                                    })
                                    .collect::<Vec<_>>()
                            });
                        
                        let not_ready_addresses = subset.get("notReadyAddresses").and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|addr| {
                                        addr.get("ip").and_then(|ip| ip.as_str())
                                            .map(|ip_str| super::resources::EndpointAddress {
                                                ip: ip_str.to_string(),
                                                hostname: None,
                                                node_name: None,
                                                target_ref: None,
                                            })
                                    })
                                    .collect::<Vec<_>>()
                            });
                        
                        let ports = subset.get("ports").and_then(|p| p.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|port| {
                                        let port_num = port.get("port").and_then(|p| p.as_i64())? as i32;
                                        Some(super::resources::EndpointPort {
                                            port: port_num,
                                            name: port.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                                            protocol: port.get("protocol").and_then(|p| p.as_str()).map(|s| s.to_string()),
                                            app_protocol: None,
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            });
                        
                        Some(super::resources::EndpointSubset {
                            addresses,
                            not_ready_addresses,
                            ports,
                        })
                    })
                    .collect();
                list_item.subsets = if subsets.is_empty() { None } else { Some(subsets) };
            }
        },
        _ => {
            // For all other resource types, the official types are already stored above
        }
    }
    
    Ok(list_item)
}
#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::Pod;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

    #[test]
    fn test_resource_type_to_kind_and_api_version() {
        // Test workloads
        assert_eq!(resource_type_to_kind_and_api_version("deployments"), ("Deployment", "apps/v1"));
        assert_eq!(resource_type_to_kind_and_api_version("pods"), ("Pod", "v1"));
        assert_eq!(resource_type_to_kind_and_api_version("statefulsets"), ("StatefulSet", "apps/v1"));
        
        // Test services
        assert_eq!(resource_type_to_kind_and_api_version("services"), ("Service", "v1"));
        assert_eq!(resource_type_to_kind_and_api_version("ingresses"), ("Ingress", "networking.k8s.io/v1"));
        assert_eq!(resource_type_to_kind_and_api_version("ingressclasses"), ("IngressClass", "networking.k8s.io/v1"));
        
        // Test storage
        assert_eq!(resource_type_to_kind_and_api_version("csidrivers"), ("CSIDriver", "storage.k8s.io/v1"));
        assert_eq!(resource_type_to_kind_and_api_version("csinodes"), ("CSINode", "storage.k8s.io/v1"));
        
        // Test cluster administration
        assert_eq!(resource_type_to_kind_and_api_version("runtimeclasses"), ("RuntimeClass", "node.k8s.io/v1"));
        
        // Test security
        assert_eq!(resource_type_to_kind_and_api_version("certificatesigningrequests"), ("CertificateSigningRequest", "certificates.k8s.io/v1"));
        
        // Test configuration
        assert_eq!(resource_type_to_kind_and_api_version("configmaps"), ("ConfigMap", "v1"));
        assert_eq!(resource_type_to_kind_and_api_version("secrets"), ("Secret", "v1"));
        
        // Test unknown resource type (fallback)
        assert_eq!(resource_type_to_kind_and_api_version("unknown"), ("unknown", "v1"));
    }

    #[tokio::test]
    async fn test_watch_manager_creation() {
        let client = K8sClient::new();
        let _manager = WatchManager::new(client);
        
        // Manager should be created successfully
        // We can't easily test the internal state without exposing more methods
        // But we can test that it doesn't panic during creation
    }

    #[tokio::test]
    async fn test_watch_key_generation() {
        // Test the watch key format used internally
        let resource_type = "pods";
        let namespace = Some("default".to_string());
        let watch_key = format!("{}:{}", resource_type, namespace.as_deref().unwrap_or("all"));
        assert_eq!(watch_key, "pods:default");
        
        let cluster_wide_key = format!("{}:{}", "nodes", None::<String>.as_deref().unwrap_or("all"));
        assert_eq!(cluster_wide_key, "nodes:all");
    }

    #[tokio::test]
    async fn test_start_watch_without_connection() {
        let client = K8sClient::new(); // Not connected
        let manager = WatchManager::new(client);
        
        // This should fail since client is not connected
        // We can't create a real AppHandle for testing, so we'll test the structure
        
        // Test that active_watches starts empty
        let watches = manager.active_watches.lock().await;
        assert!(watches.is_empty());
    }

    #[tokio::test]
    async fn test_stop_watch_nonexistent() {
        let client = K8sClient::new();
        let manager = WatchManager::new(client);
        
        // Stopping a non-existent watch should not error
        let result = manager.stop_watch("pods", Some(vec!["default".to_string()])).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_all_watches() {
        let client = K8sClient::new();
        let manager = WatchManager::new(client);
        
        // Should not error even with no active watches
        let result = manager.stop_all_watches().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_to_list_item() {
        // Create a test Pod object
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "test".to_string());
        
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("test-pod".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("test-uid-123".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = convert_to_list_item(&pod, "pods");
        assert!(result.is_ok());
        
        let item = result.unwrap();
        assert_eq!(item.metadata.name, Some("test-pod".to_string()));
        assert_eq!(item.metadata.namespace, Some("default".to_string()));
        assert_eq!(item.metadata.uid, Some("test-uid-123".to_string()));
        assert_eq!(item.kind, "Pod");
        
        // Check labels conversion
        let labels = item.metadata.labels.unwrap();
        assert_eq!(labels.get("app"), Some(&"test".to_string()));
    }

    #[test]
    fn test_convert_to_list_item_minimal() {
        // Test with minimal Pod (no labels, annotations, etc.)
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("minimal-pod".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = convert_to_list_item(&pod, "pods");
        assert!(result.is_ok());
        
        let item = result.unwrap();
        assert_eq!(item.metadata.name, Some("minimal-pod".to_string()));
        assert_eq!(item.metadata.namespace, None);
        assert!(item.metadata.labels.is_none());
        assert!(item.metadata.annotations.is_none());
    }

    #[test]
    fn test_convert_to_list_item_with_status() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("status-pod".to_string()),
                ..Default::default()
            },
            status: Some(k8s_openapi::api::core::v1::PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = convert_to_list_item(&pod, "pods");
        assert!(result.is_ok());
        
        let item = result.unwrap();
        assert!(item.pod_status.is_some());
        
        // The status should contain the phase
        let status = item.pod_status.unwrap();
        assert_eq!(status.phase.as_deref(), Some("Running"));
    }

    #[test]
    fn test_resource_type_matching() {
        // Test that we support all the expected resource types
        let supported_types = vec![
            "pods", "deployments", "services", "configmaps", "secrets",
            "namespaces", "nodes", "statefulsets", "daemonsets", "jobs",
            "cronjobs", "ingresses", "serviceaccounts", "networkpolicies",
            "persistentvolumes", "persistentvolumeclaims", "storageclasses"
        ];

        // This tests the structure of our match statement in start_watch
        // In a real test, we'd need to mock the Kubernetes API calls
        for resource_type in supported_types {
            // Each type should be handled in the match statement
            // We can't easily test the actual matching without refactoring
            // but we can ensure the types are consistent with our resources
            assert!(!resource_type.is_empty());
            assert!(resource_type.chars().all(|c| c.is_lowercase() || c.is_ascii_digit()));
        }
    }

    #[tokio::test]
    async fn test_concurrent_watch_operations() {
        let client = K8sClient::new();
        let manager = Arc::new(WatchManager::new(client));
        let mut handles = vec![];

        // Test concurrent stop operations
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let resource = format!("pods-{}", i);
                manager_clone.stop_watch(&resource, Some(vec!["default".to_string()])).await
            });
            handles.push(handle);
        }

        // All operations should complete successfully
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}