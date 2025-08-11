import { computed } from 'vue'
import type { Ref } from 'vue'
import type { K8sListItem } from '@/types'
import { useResourceStatus } from './useResourceStatus'

export function useResourceFiltering(
  items: Ref<K8sListItem[]>,
  searchQuery: Ref<string>
) {
  const { getStatusText } = useResourceStatus()

  const filteredItems = computed(() => {
    if (!searchQuery.value.trim()) {
      return items.value
    }

    const searchTerm = searchQuery.value.toLowerCase().trim()
    
    return items.value.filter(item => {
      // Search in name
      if (item.metadata?.name?.toLowerCase().includes(searchTerm)) {
        return true
      }
      
      // Search in namespace
      if (item.metadata?.namespace?.toLowerCase().includes(searchTerm)) {
        return true
      }
      
      // Search in resource kind
      if (item.kind?.toLowerCase().includes(searchTerm)) {
        return true
      }
      
      // Search in labels
      if (item.metadata?.labels) {
        for (const [key, value] of Object.entries(item.metadata.labels)) {
          if (key.toLowerCase().includes(searchTerm) || value.toLowerCase().includes(searchTerm)) {
            return true
          }
        }
      }
      
      // Search in status text
      if (getStatusText(item).toLowerCase().includes(searchTerm)) {
        return true
      }
      
      return false
    })
  })

  return {
    filteredItems
  }
}