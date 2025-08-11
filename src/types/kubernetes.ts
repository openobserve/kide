// Import official Kubernetes types from the JavaScript client
import {
  V1ObjectMeta,
  V1OwnerReference,
  V1PodSpec,
  V1PodStatus,
  V1ServiceSpec,
  V1ServiceStatus,
  V1DeploymentSpec,
  V1DeploymentStatus,
  V1NodeSpec,
  V1NodeStatus,
  V1NamespaceSpec,
  V1NamespaceStatus,
  V1PersistentVolumeSpec,
  V1PersistentVolumeStatus,
  V1PersistentVolumeClaimSpec,
  V1PersistentVolumeClaimStatus,
  V1ReplicaSetSpec,
  V1ReplicaSetStatus,
  V1StatefulSetSpec,
  V1StatefulSetStatus,
  V1DaemonSetSpec,
  V1DaemonSetStatus,
  V1JobSpec,
  V1JobStatus,
  V1CronJobSpec,
  V1CronJobStatus,
  V1IngressSpec,
  V1IngressStatus,
  V1NetworkPolicySpec,
  V1StorageClass,
  V1PodDisruptionBudgetSpec,
  V1PodDisruptionBudgetStatus,
  V2HorizontalPodAutoscalerSpec,
  V2HorizontalPodAutoscalerStatus,
  V1ContainerStatus,
  V1ContainerState
} from '@kubernetes/client-node'

// Re-export official types for convenience
export type K8sObjectMeta = V1ObjectMeta
export type K8sOwnerReference = V1OwnerReference
export type PodSpec = V1PodSpec
export type PodStatus = V1PodStatus
export type ServiceSpec = V1ServiceSpec
export type ServiceStatus = V1ServiceStatus
export type DeploymentSpec = V1DeploymentSpec
export type DeploymentStatus = V1DeploymentStatus
export type NodeSpec = V1NodeSpec
export type NodeStatus = V1NodeStatus
export type NamespaceSpec = V1NamespaceSpec
export type NamespaceStatus = V1NamespaceStatus
export type PersistentVolumeSpec = V1PersistentVolumeSpec
export type PersistentVolumeStatus = V1PersistentVolumeStatus
export type PersistentVolumeClaimSpec = V1PersistentVolumeClaimSpec
export type PersistentVolumeClaimStatus = V1PersistentVolumeClaimStatus
export type ReplicaSetSpec = V1ReplicaSetSpec
export type ReplicaSetStatus = V1ReplicaSetStatus
export type StatefulSetSpec = V1StatefulSetSpec
export type StatefulSetStatus = V1StatefulSetStatus
export type DaemonSetSpec = V1DaemonSetSpec
export type DaemonSetStatus = V1DaemonSetStatus
export type JobSpec = V1JobSpec
export type JobStatus = V1JobStatus
export type CronJobSpec = V1CronJobSpec
export type CronJobStatus = V1CronJobStatus
export type IngressSpec = V1IngressSpec
export type IngressStatus = V1IngressStatus
export type NetworkPolicySpec = V1NetworkPolicySpec
export type StorageClass = V1StorageClass
export type PodDisruptionBudgetSpec = V1PodDisruptionBudgetSpec
export type PodDisruptionBudgetStatus = V1PodDisruptionBudgetStatus
export type HorizontalPodAutoscalerSpec = V2HorizontalPodAutoscalerSpec
export type HorizontalPodAutoscalerStatus = V2HorizontalPodAutoscalerStatus
export type ContainerStatus = V1ContainerStatus
export type ContainerState = V1ContainerState

// Base interface for all K8s resources using official Kubernetes client types
export interface K8sListItem {
  metadata: K8sObjectMeta
  kind: string
  apiVersion: string
  complete_object?: Record<string, any>
  
  // Official Kubernetes client spec types - direct k8s-openapi types for complete compatibility
  podSpec?: PodSpec
  serviceSpec?: ServiceSpec
  configMapSpec?: Record<string, any>
  secretSpec?: Record<string, any>
  namespaceSpec?: NamespaceSpec
  nodeSpec?: NodeSpec
  persistentVolumeSpec?: PersistentVolumeSpec
  persistentVolumeClaimSpec?: PersistentVolumeClaimSpec
  endpoints_spec?: Record<string, any>
  deploymentSpec?: DeploymentSpec
  replicaSetSpec?: ReplicaSetSpec
  statefulSetSpec?: StatefulSetSpec
  daemonSetSpec?: DaemonSetSpec
  jobSpec?: JobSpec
  cronJobSpec?: CronJobSpec
  ingressSpec?: IngressSpec
  networkPolicySpec?: NetworkPolicySpec
  endpointSliceSpec?: Record<string, any>
  storageClassSpec?: StorageClass
  roleSpec?: Record<string, any>
  roleBindingSpec?: Record<string, any>
  clusterRoleSpec?: Record<string, any>
  clusterRoleBindingSpec?: Record<string, any>
  serviceAccountSpec?: Record<string, any>
  podDisruptionBudgetSpec?: PodDisruptionBudgetSpec
  horizontalPodAutoscalerSpec?: HorizontalPodAutoscalerSpec
  
  // Official Kubernetes client status types - direct k8s-openapi types for complete compatibility  
  podStatus?: PodStatus
  serviceStatus?: ServiceStatus
  namespaceStatus?: NamespaceStatus
  nodeStatus?: NodeStatus
  persistentVolumeStatus?: PersistentVolumeStatus
  persistentVolumeClaimStatus?: PersistentVolumeClaimStatus
  deploymentStatus?: DeploymentStatus
  replicaSetStatus?: ReplicaSetStatus
  statefulSetStatus?: StatefulSetStatus
  daemonSetStatus?: DaemonSetStatus
  jobStatus?: JobStatus
  cronJobStatus?: CronJobStatus
  ingressStatus?: IngressStatus
  podDisruptionBudgetStatus?: PodDisruptionBudgetStatus
  horizontalPodAutoscalerStatus?: HorizontalPodAutoscalerStatus
  
  // EndpointSlice - using official k8s types
  endpoint_slice?: Record<string, any> // Full EndpointSlice object from k8s-openapi
  
  // Endpoints specific fields (legacy)
  subsets?: Array<{
    addresses?: Array<{ ip: string }>
    notReadyAddresses?: Array<{ ip: string }>
    ports?: Array<{ port: number; name?: string; protocol?: string }>
  }>
}

// Specific resource types using official Kubernetes client types
export interface PodResource extends K8sListItem {
  kind: 'Pod'
  podSpec?: PodSpec
  podStatus?: PodStatus
}

export interface DeploymentResource extends K8sListItem {
  kind: 'Deployment'
  deploymentSpec?: DeploymentSpec
  deploymentStatus?: DeploymentStatus
}

export interface ServiceResource extends K8sListItem {
  kind: 'Service'
  serviceSpec?: ServiceSpec
  serviceStatus?: ServiceStatus
}

// Union type for type-safe resource handling using official Kubernetes client types
export type TypedK8sResource = PodResource | DeploymentResource | ServiceResource | K8sListItem

export interface K8sResource {
  name: string
  apiVersion: string
  kind: string
  namespaced: boolean
  description: string
}

export interface K8sResourceCategory {
  name: string
  resources: K8sResource[]
}

export type WatchEventType = 'Added' | 'Modified' | 'Deleted'

export interface WatchEvent {
  [key: string]: K8sListItem
}

// Kubernetes context types
export interface K8sContext {
  name: string
  cluster: string
  user: string
  namespace?: string
}

export type ContextStatus = 'connected' | 'connecting' | 'failed' | 'disconnected'

// Container status types are imported from the official Kubernetes client above

// All status types are now imported from the official Kubernetes client
// No need to redefine them - they're available as type aliases above

// All spec and status types are now imported from the official Kubernetes client
// No need to redefine them - they're already available as type aliases above

// Connection status types
export type ConnectionStatus = 'connecting' | 'connected' | 'failed' | 'disconnected'

// Component prop types
export interface ResourceNavigationProps {
  categories: K8sResourceCategory[]
  selectedResource: K8sResource | null
  connected: boolean
  connectionStatus: ConnectionStatus
}

export interface ResourceListProps {
  resource: K8sResource | null
  items: K8sListItem[]
  connected: boolean
}

export interface MultiSelectNamespaceProps {
  namespaces: string[]
  selectedNamespaces: string[]
}

// Event types
export interface SelectResourceEvent {
  resource: K8sResource
}

export interface UpdateNamespacesEvent {
  namespaces: string[]
}