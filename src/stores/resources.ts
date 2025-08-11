import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { K8sResource, K8sListItem, K8sResourceCategory, WatchEvent } from '@/types'
import { useClusterStore } from './cluster'
import { useStoreTimeouts } from '@/composables/useStoreTimeouts'
import { TIMEOUTS } from '@/constants/timeouts'

export const useResourceStore = defineStore('resources', (): {
  resourceCategories: import('vue').Ref<K8sResourceCategory[]>
  selectedResource: import('vue').Ref<K8sResource | null>
  resourceItems: import('vue').Ref<K8sListItem[]>
  loading: import('vue').Ref<boolean>
  isChangingNamespaces: import('vue').Ref<boolean>
  error: import('vue').Ref<string | null>
  watchError: import('vue').Ref<string | null>
  loadResourceCategories: () => Promise<void>
  selectResource: (resource: K8sResource, namespaces: string[]) => Promise<void>
  changeNamespaces: (newNamespaces: string[]) => Promise<void>
  processWatchEvent: (event: WatchEvent) => void
  refreshAfterResourceDeleted: (namespaces: string[]) => Promise<void>
  resetForClusterChange: () => Promise<void>
  cleanup: () => void
} => {
  // Timeout management
  const { createTimeout, clearTimeout } = useStoreTimeouts()

  // State
  const resourceCategories = ref<K8sResourceCategory[]>([])
  const selectedResource = ref<K8sResource | null>(null)
  const resourceItems = ref<K8sListItem[]>([])
  const loading = ref(false)
  const isChangingNamespaces = ref(false)
  const error = ref<string | null>(null)
  const watchError = ref<string | null>(null)

  // Event batching for performance
  let eventBatch: WatchEvent[] = []
  let eventBatchTimeout: NodeJS.Timeout | null = null
  let namespaceChangeTimeout: NodeJS.Timeout | null = null


  // Actions
  async function loadResourceCategories(): Promise<void> {
    try {
      const categories = await invoke<K8sResourceCategory[]>('get_resources')
      resourceCategories.value = categories
    } catch (error) {
      console.error('Failed to load resource categories:', error)
      throw error
    }
  }

  async function selectResource(resource: K8sResource, namespaces: string[]): Promise<void> {
    // Clear previous errors
    error.value = null
    watchError.value = null
    
    if (selectedResource.value) {
      try {
        await invoke('stop_resource_watch', {
          resourceType: selectedResource.value.name.toLowerCase(),
          namespaces: selectedResource.value.namespaced ? namespaces : null
        })
      } catch (stopError) {
        console.warn('Failed to stop previous watch:', stopError)
        // Don't throw here as this shouldn't prevent new watch
      }
    }

    selectedResource.value = resource
    resourceItems.value = []
    loading.value = true

    try {
      await invoke('start_resource_watch', {
        resourceType: resource.name.toLowerCase(),
        namespaces: resource.namespaced ? namespaces : null
      })
      
      // Clear any previous watch errors on successful start
      watchError.value = null
    } catch (watchErr: any) {
      console.error('Failed to start watch:', watchErr)
      const errorMessage = `Failed to watch ${resource.name}: ${watchErr?.toString() || 'Unknown error'}`
      watchError.value = errorMessage
      error.value = errorMessage
      throw new Error(errorMessage)
    } finally {
      loading.value = false
    }
  }

  async function changeNamespaces(newNamespaces: string[]): Promise<void> {
    // Clear any existing timeout safely
    if (namespaceChangeTimeout) {
      clearTimeout(namespaceChangeTimeout)
      namespaceChangeTimeout = null
    }
    
    // Clear previous errors
    watchError.value = null
    
    // Don't restart watch if no resource is selected or resource is not namespaced
    if (!selectedResource.value || !selectedResource.value.namespaced) {
      return
    }
    
    // Immediately filter out items that are not in the new namespaces
    if (resourceItems.value.length > 0) {
      resourceItems.value = resourceItems.value.filter(item => {
        const itemNamespace = item.metadata?.namespace
        return itemNamespace ? newNamespaces.includes(itemNamespace) : false
      })
    }
    
    // Set loading state immediately
    isChangingNamespaces.value = true
    loading.value = true
    
    // Debounce the actual watch restart to prevent rapid successive calls
    namespaceChangeTimeout = createTimeout(async () => {
      try {
        // Clear existing items to show immediate feedback
        resourceItems.value = []
        
        // Stop current watch
        if (selectedResource.value) {
          try {
            await invoke('stop_resource_watch', {
              resourceType: selectedResource.value.name.toLowerCase(),
              namespaces: selectedResource.value.namespaced ? newNamespaces : null
            })
          } catch (stopError) {
            console.warn('Failed to stop watch during namespace change:', stopError)
            // Continue with restart attempt
          }
        }
        
        // Small delay to allow backend cleanup
        await new Promise(resolve => setTimeout(resolve, TIMEOUTS.RESOURCE_CLEANUP_DELAY))
        
        // Start new watch with updated namespaces
        if (selectedResource.value && selectedResource.value.namespaced) {
          await invoke('start_resource_watch', {
            resourceType: selectedResource.value.name.toLowerCase(),
            namespaces: newNamespaces
          })
        }
      } catch (namespaceError: any) {
        console.error('Failed to restart watch for namespace change:', namespaceError)
        const errorMessage = `Failed to switch namespaces for ${selectedResource.value?.name}: ${namespaceError?.toString() || 'Unknown error'}`
        watchError.value = errorMessage
        error.value = errorMessage
      } finally {
        isChangingNamespaces.value = false
        loading.value = false
        namespaceChangeTimeout = null
      }
    }, TIMEOUTS.NAMESPACE_CHANGE_DEBOUNCE)
  }

  function processWatchEvent(event: WatchEvent): void {
    // Only process events for the currently selected resource
    if (!selectedResource.value) {
      return
    }
    
    // Skip events during namespace changes to prevent conflicts
    if (isChangingNamespaces.value) {
      return
    }
    
    // Add event to batch
    eventBatch.push(event)
    
    // Clear existing timeout and set new one
    if (eventBatchTimeout) {
      clearTimeout(eventBatchTimeout)
      eventBatchTimeout = null
    }
    
    // Process batch after a short delay to collect multiple events
    eventBatchTimeout = createTimeout(() => {
      processBatchedEvents()
      eventBatch = []
      eventBatchTimeout = null
    }, TIMEOUTS.EVENT_BATCH_WINDOW)
  }

  function processBatchedEvents(): void {
    if (eventBatch.length === 0) return
    
    // Get selected namespaces from cluster store
    const clusterStore = useClusterStore()
    const selectedNamespaces = clusterStore.selectedNamespaces
    
    const itemsMap = new Map<string, K8sListItem>()
    
    // Initialize map with current items (only include items that match selected namespaces)
    resourceItems.value.forEach(item => {
      if (item.metadata?.uid) {
        // For namespaced resources, check namespace match
        if (selectedResource.value?.namespaced) {
          const itemNamespace = item.metadata?.namespace
          if (itemNamespace && selectedNamespaces.includes(itemNamespace)) {
            itemsMap.set(item.metadata.uid, item)
          }
        } else {
          // For non-namespaced resources, include all
          itemsMap.set(item.metadata.uid, item)
        }
      }
    })
    
    // Process all events in batch
    for (const event of eventBatch) {
      // Check different possible event structures
      const addedItem = (event as any).Added || (event as any).added || event
      const modifiedItem = (event as any).Modified || (event as any).modified
      const deletedItem = (event as any).Deleted || (event as any).deleted
      
      // Helper function to check if item should be included based on resource type and namespace
      const shouldIncludeItem = (item: K8sListItem): boolean => {
        // First check if the item's kind matches the selected resource type
        if (!selectedResource.value) return false
        
        // Check if the event is for the correct resource type
        if (item.kind !== selectedResource.value.kind) {
          return false
        }
        
        // For non-namespaced resources, include if resource type matches
        if (!selectedResource.value.namespaced) {
          return true
        }
        
        // For namespaced resources, check if item's namespace is in selected namespaces
        const itemNamespace = item.metadata?.namespace
        return itemNamespace ? selectedNamespaces.includes(itemNamespace) : false
      }
      
      if (addedItem && addedItem.metadata?.uid) {
        if (shouldIncludeItem(addedItem)) {
          itemsMap.set(addedItem.metadata.uid, addedItem as K8sListItem)
        }
      } else if (modifiedItem && modifiedItem.metadata?.uid) {
        if (shouldIncludeItem(modifiedItem)) {
          itemsMap.set(modifiedItem.metadata.uid, modifiedItem as K8sListItem)
        } else {
          // Remove item if it's no longer in selected namespaces
          itemsMap.delete(modifiedItem.metadata.uid)
        }
      } else if (deletedItem && deletedItem.metadata?.uid) {
        itemsMap.delete(deletedItem.metadata.uid)
      }
    }
    
    // Update resource items in a single operation
    resourceItems.value = Array.from(itemsMap.values())
  }

  async function refreshAfterResourceDeleted(namespaces: string[]): Promise<void> {
    if (!selectedResource.value) {
      return
    }
    
    try {
      // Stop current watch
      await invoke('stop_resource_watch', {
        resourceType: selectedResource.value.name.toLowerCase(),
        namespaces: selectedResource.value.namespaced ? namespaces : null
      })
      
      // Clear current items
      resourceItems.value = []
      
      // Small delay to allow backend cleanup
      await new Promise(resolve => setTimeout(resolve, TIMEOUTS.RESOURCE_CLEANUP_DELAY))
      
      // Restart watch to get updated resource list
      await invoke('start_resource_watch', {
        resourceType: selectedResource.value.name.toLowerCase(),
        namespaces: selectedResource.value.namespaced ? namespaces : null
      })
    } catch (error) {
      console.error('Failed to refresh resource list after deletion:', error)
      throw error
    }
  }

  async function resetForClusterChange(): Promise<void> {
    // Stop any current watch
    if (selectedResource.value) {
      try {
        await invoke('stop_resource_watch', {
          resourceType: selectedResource.value.name.toLowerCase(),  
          namespaces: selectedResource.value.namespaced ? ['default'] : null
        })
      } catch (error) {
        console.warn('Failed to stop watch during cluster change:', error)
      }
    }
    
    // Clear all state
    selectedResource.value = null
    resourceItems.value = []
    loading.value = false
    isChangingNamespaces.value = false
    error.value = null
    watchError.value = null
    
    // Clear any pending timeouts safely
    if (namespaceChangeTimeout) {
      clearTimeout(namespaceChangeTimeout)
      namespaceChangeTimeout = null
    }
    if (eventBatchTimeout) {
      clearTimeout(eventBatchTimeout)
      eventBatchTimeout = null
    }
  }


  function cleanup(): void {
    if (namespaceChangeTimeout) {
      clearTimeout(namespaceChangeTimeout)
      namespaceChangeTimeout = null
    }
    if (eventBatchTimeout) {
      clearTimeout(eventBatchTimeout)
      eventBatchTimeout = null
    }
  }

  return {
    // State
    resourceCategories,
    selectedResource,
    resourceItems,
    loading,
    isChangingNamespaces,
    error,
    watchError,
    
    // Actions
    loadResourceCategories,
    selectResource,
    changeNamespaces,
    processWatchEvent,
    refreshAfterResourceDeleted,
    resetForClusterChange,
    cleanup,
  }
})