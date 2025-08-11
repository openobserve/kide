import type { ContainerStatus, K8sListItem } from '@/types'

export function useContainerStatus() {
  function getContainerStatusColor(container: any): string {
    // Backend sends camelCase field names as per k8s-openapi serialization
    if (container.state?.running) return 'bg-green-500'
    if (container.state?.waiting) return 'bg-yellow-500'
    if (container.state?.terminated) {
      return container.state.terminated.exitCode === 0 ? 'bg-blue-500' : 'bg-red-500'
    }
    return 'bg-gray-400'
  }

  function getContainerStatusText(container: any): string {
    // Backend sends camelCase field names as per k8s-openapi serialization
    if (container.state?.running) return 'Running'
    if (container.state?.waiting) return container.state.waiting.reason || 'Waiting'
    if (container.state?.terminated) {
      return `Terminated (${container.state.terminated.exitCode})`
    }
    return 'Unknown'
  }

  function getTotalRestartCount(item: K8sListItem): number {
    if (item.kind !== 'Pod') return 0
    
    // Access Pod data using camelCase (from Tauri serde serialization)
    const podStatus = (item as any).podStatus
    const containerStatuses = podStatus?.containerStatuses || []
    const initContainerStatuses = podStatus?.initContainerStatuses || []
    
    const mainContainerRestarts = containerStatuses.reduce((total: number, container: any) => {
      // Backend sends camelCase field names as per k8s-openapi serialization
      return total + (container.restartCount || 0)
    }, 0)
    
    const initContainerRestarts = initContainerStatuses.reduce((total: number, container: any) => {
      // Backend sends camelCase field names as per k8s-openapi serialization
      return total + (container.restartCount || 0)
    }, 0)
    
    return mainContainerRestarts + initContainerRestarts
  }

  function getControlledBy(item: K8sListItem): string | null {
    if (item.metadata?.ownerReferences && item.metadata.ownerReferences.length > 0) {
      const owner = item.metadata.ownerReferences[0]
      return `${owner.kind}/${owner.name}`
    }
    return null
  }

  function getQoSClass(item: K8sListItem): string {
    if (item.kind !== 'Pod') return ''
    // Access Pod status using camelCase
    return (item as any).podStatus?.qosClass || 'BestEffort'
  }

  return {
    getContainerStatusColor,
    getContainerStatusText,
    getTotalRestartCount,
    getControlledBy,
    getQoSClass
  }
}