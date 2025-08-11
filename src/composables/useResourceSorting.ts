import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import type { K8sListItem } from '@/types'
import { useResourceStatus } from './useResourceStatus'
import { useContainerStatus } from './useContainerStatus'

export function useResourceSorting(resource: Ref<any>) {
  const { getStatusText } = useResourceStatus()
  const { getTotalRestartCount, getControlledBy, getQoSClass } = useContainerStatus()

  const sortColumn = ref<string>('name')
  const sortDirection = ref<'asc' | 'desc'>('asc')

  function handleSort(column: string): void {
    if (sortColumn.value === column) {
      sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
    } else {
      sortColumn.value = column
      sortDirection.value = 'asc'
    }
  }

  function sortItems(items: K8sListItem[]): K8sListItem[] {
    return [...items].sort((a, b) => {
      let aValue: any = ''
      let bValue: any = ''
      
      switch (sortColumn.value) {
        case 'name':
          aValue = a.metadata?.name || ''
          bValue = b.metadata?.name || ''
          break
        case 'namespace':
          aValue = a.metadata?.namespace || ''
          bValue = b.metadata?.namespace || ''
          break
        case 'status':
          aValue = getStatusText(a)
          bValue = getStatusText(b)
          break
        case 'age':
          aValue = new Date(a.metadata?.creationTimestamp || '').getTime()
          bValue = new Date(b.metadata?.creationTimestamp || '').getTime()
          break
        case 'restarts':
          if (resource.value?.kind === 'Pod') {
            aValue = getTotalRestartCount(a)
            bValue = getTotalRestartCount(b)
          }
          break
        case 'controlled_by':
          aValue = getControlledBy(a) || ''
          bValue = getControlledBy(b) || ''
          break
        case 'node':
          // Get node name based on resource kind
          if (resource.value?.kind === 'Pod') {
            aValue = (a as any).podSpec?.nodeName || ''
            bValue = (b as any).podSpec?.nodeName || ''
          } else {
            aValue = ''
            bValue = ''
          }
          break
        case 'qos':
          aValue = getQoSClass(a)
          bValue = getQoSClass(b)
          break
        default:
          return 0
      }
      
      // Handle different data types
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        const result = aValue.localeCompare(bValue, undefined, { numeric: true, sensitivity: 'base' })
        return sortDirection.value === 'asc' ? result : -result
      } else if (typeof aValue === 'number' && typeof bValue === 'number') {
        const result = aValue - bValue
        return sortDirection.value === 'asc' ? result : -result
      }
      
      return 0
    })
  }

  const sortedItems = computed(() => (items: K8sListItem[]) => sortItems(items))

  return {
    sortColumn,
    sortDirection,
    handleSort,
    sortItems,
    sortedItems
  }
}