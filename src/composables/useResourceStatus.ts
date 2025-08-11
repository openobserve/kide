import type { K8sListItem } from '@/types'

export function useResourceStatus() {
  // Helper function to get status object based on resource kind
  function getStatusForKind(item: K8sListItem): any {
    if (!item) return null
    switch (item.kind) {
      case 'Pod': return (item as any).podStatus
      case 'Deployment': return (item as any).deploymentStatus
      case 'Service': return (item as any).serviceStatus
      case 'Node': return (item as any).nodeStatus
      case 'Namespace': return (item as any).namespaceStatus
      case 'PersistentVolume': return (item as any).persistentVolumeStatus
      case 'PersistentVolumeClaim': return (item as any).persistentVolumeClaimStatus
      case 'ReplicaSet': return (item as any).replicaSetStatus
      case 'StatefulSet': return (item as any).statefulSetStatus
      case 'DaemonSet': return (item as any).daemonSetStatus
      case 'Job': return (item as any).jobStatus
      case 'CronJob': return (item as any).cronJobStatus
      case 'Ingress': return (item as any).ingressStatus
      case 'PodDisruptionBudget': return (item as any).podDisruptionBudgetStatus
      case 'HorizontalPodAutoscaler': return (item as any).horizontalPodAutoscalerStatus
      default: return null
    }
  }
  
  // Helper function to get spec object based on resource kind
  function getSpecForKind(item: K8sListItem): any {
    if (!item) return null
    switch (item.kind) {
      case 'Pod': return (item as any).podSpec
      case 'Deployment': return (item as any).deploymentSpec
      case 'Service': return (item as any).serviceSpec
      case 'Node': return (item as any).nodeSpec
      case 'Namespace': return (item as any).namespaceSpec
      case 'PersistentVolume': return (item as any).persistentVolumeSpec
      case 'PersistentVolumeClaim': return (item as any).persistentVolumeClaimSpec
      case 'ReplicaSet': return (item as any).replicaSetSpec
      case 'StatefulSet': return (item as any).statefulSetSpec
      case 'DaemonSet': return (item as any).daemonSetSpec
      case 'Job': return (item as any).jobSpec
      case 'CronJob': return (item as any).cronJobSpec
      case 'Ingress': return (item as any).ingressSpec
      case 'NetworkPolicy': return (item as any).networkPolicySpec
      case 'StorageClass': return (item as any).storageClassSpec
      case 'PodDisruptionBudget': return (item as any).podDisruptionBudgetSpec
      case 'HorizontalPodAutoscaler': return (item as any).horizontalPodAutoscalerSpec
      default: return null
    }
  }
  function getStatusText(item: K8sListItem): string {
    if (!item) return 'Unknown'
    switch (item.kind) {
      case 'Pod':
        return (item as any).podStatus?.phase || 'Unknown'
        
      case 'Job':
        // Check for job conditions first
        if ((item as any).jobStatus?.conditions) {
          const completeCondition = (item as any).jobStatus.conditions.find((c: any) => c.type === 'Complete')
          const failedCondition = (item as any).jobStatus.conditions.find((c: any) => c.type === 'Failed')
          
          if (completeCondition?.status === 'True') {
            return 'Complete'
          }
          if (failedCondition?.status === 'True') {
            return 'Failed'
          }
        }
        
        // Check job status fields
        const completions = (item as any).jobSpec?.completions
        const succeeded = (item as any).jobStatus?.succeeded || 0
        const failed = (item as any).jobStatus?.failed || 0
        const active = (item as any).jobStatus?.active || 0
        
        if (failed > 0) return 'Failed'
        if (completions && succeeded >= completions) return 'Complete'
        if (active > 0) return 'Running'
        if (succeeded > 0 && (!completions || succeeded < completions)) return 'Running'
        
        return 'Pending'
        
      case 'PersistentVolumeClaim':
        return (item as any).persistentVolumeClaimStatus?.phase || 'Unknown'
        
      case 'PersistentVolume':
        return (item as any).persistentVolumeStatus?.phase || 'Unknown'
        
      case 'Namespace':
        return (item as any).namespaceStatus?.phase || 'Unknown'
        
      case 'Node':
        if ((item as any).nodeSpec?.unschedulable) return 'SchedulingDisabled'
        if ((item as any).nodeStatus?.conditions) {
          const readyCondition = (item as any).nodeStatus.conditions.find((c: any) => c.type === 'Ready')
          if (readyCondition) {
            return readyCondition.status === 'True' ? 'Ready' : 'NotReady'
          }
        }
        return 'Unknown'
        
      case 'Service':
        if ((item as any).serviceSpec?.type === 'LoadBalancer') {
          return ((item as any).serviceStatus?.loadBalancer?.ingress && (item as any).serviceStatus.loadBalancer.ingress.length > 0) ? 'Ready' : 'Pending'
        }
        return 'Ready'
        
      case 'Ingress':
        return ((item as any).ingressStatus?.loadBalancer?.ingress && (item as any).ingressStatus.loadBalancer.ingress.length > 0) ? 'Ready' : 'Pending'
        
      case 'Endpoints':
        // Endpoints are ready if they have at least one subset with addresses
        if (item.subsets && item.subsets.length > 0) {
          const hasAddresses = item.subsets.some((subset: any) => subset.addresses && subset.addresses.length > 0)
          return hasAddresses ? 'Ready' : 'NotReady'
        }
        return 'NotReady'
        
      case 'EndpointSlice':
        // EndpointSlices are ready if they have endpoints
        if (item.endpoint_slice?.endpoints && item.endpoint_slice.endpoints.length > 0) {
          const hasReadyEndpoints = item.endpoint_slice.endpoints.some((endpoint: any) => 
            endpoint.conditions?.ready !== false
          )
          return hasReadyEndpoints ? 'Ready' : 'NotReady'
        }
        return 'NotReady'
        
      case 'DaemonSet':
        if ((item as any).daemonSetStatus) {
          const {
            desiredNumberScheduled = 0,
            currentNumberScheduled = 0,
            numberReady = 0,
            numberMisscheduled = 0,
            updatedNumberScheduled,
            conditions = []
          } = (item as any).daemonSetStatus

          // Check for failed conditions first
          const failedCondition = conditions.find((c: any) => 
            c.status === 'False' && ['Ready', 'Available'].includes(c.type)
          )
          if (failedCondition) return 'Failed'

          // Check if updating (rolling update in progress)
          if (updatedNumberScheduled !== undefined && 
              updatedNumberScheduled < currentNumberScheduled) {
            return 'Updating'
          }

          // Check for misscheduled pods
          if (numberMisscheduled > 0) return 'NotReady'

          // Check if all desired pods are ready
          if (desiredNumberScheduled > 0 && numberReady === desiredNumberScheduled) {
            return 'Ready'
          }

          // If some pods are not ready
          if (numberReady < desiredNumberScheduled) return 'NotReady'

          // Fallback for edge cases
          return desiredNumberScheduled === 0 ? 'Unknown' : 'Ready'
        }
        return 'Unknown'
        
      case 'ReplicaSet':
        if ((item as any).replicaSetStatus) {
          const {
            replicas = 0,
            readyReplicas = 0,
            availableReplicas = 0,
            conditions = []
          } = (item as any).replicaSetStatus
          
          const desiredReplicas = (item as any).replicaSetSpec?.replicas || 0
          
          // Check for ReplicaFailure condition
          const failureCondition = conditions.find((c: any) => 
            c.type === 'ReplicaFailure' && c.status === 'True'
          )
          if (failureCondition) return 'Failed'
          
          // Scaling in progress (check first to catch scaling scenarios)
          if (replicas !== desiredReplicas) {
            return 'Scaling'
          }
          
          // All replicas ready and available
          if (desiredReplicas > 0 && readyReplicas === desiredReplicas && 
              availableReplicas === desiredReplicas) {
            return 'Ready'
          }
          
          // Some replicas not ready or not available
          if (readyReplicas < desiredReplicas || availableReplicas < desiredReplicas) {
            return 'NotReady'
          }
          
          // Edge case: zero replicas desired
          return desiredReplicas === 0 ? 'Ready' : 'Unknown'
        }
        return 'Unknown'
        
      case 'ConfigMap':
      case 'Secret':
      case 'StorageClass':
        return 'Ready' // These are typically ready once created
        
      default:
        // Generic status handling for other resource types
        // Check deployment status
        if ((item as any).deploymentStatus?.conditions) {
          const availableCondition = (item as any).deploymentStatus.conditions.find((c: any) => c.type === 'Available')
          if (availableCondition) {
            return availableCondition.status === 'True' ? 'Ready' : 'NotReady'
          }
        }
        
        // Check stateful set status
        if ((item as any).statefulSetStatus?.conditions) {
          const readyCondition = (item as any).statefulSetStatus.conditions.find((c: any) => c.type === 'Ready')
          if (readyCondition) {
            return readyCondition.status === 'True' ? 'Ready' : 'NotReady'
          }
        }
        
        // Check persistent volume claim status  
        if ((item as any).persistentVolumeClaimStatus?.conditions) {
          const boundCondition = (item as any).persistentVolumeClaimStatus.conditions.find((c: any) => c.type === 'Bound')
          if (boundCondition) {
            return boundCondition.status === 'True' ? 'Bound' : 'Pending'
          }
        }
        
        return 'Unknown'
    }
  }

  function getStatusClass(item: K8sListItem): string {
    const status = getStatusText(item)
    
    switch (status) {
      case 'Running':
      case 'Ready':
      case 'Active':
      case 'Bound':
      case 'Available':
        return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
      case 'Complete':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
      case 'Pending':
      case 'Terminating':
      case 'Updating':
      case 'Scaling':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
      case 'Failed':
      case 'Error':
      case 'NotReady':
      case 'Released':
        return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
      case 'SchedulingDisabled':
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
    }
  }

  // Generic helper functions that work like the old .status and .spec access
  function getGenericStatus(item: K8sListItem): any {
    return getStatusForKind(item)
  }
  
  function getGenericSpec(item: K8sListItem): any {
    return getSpecForKind(item)
  }
  
  return {
    getStatusText,
    getStatusClass,
    getStatusForKind,
    getSpecForKind,
    getGenericStatus,
    getGenericSpec
  }
}