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
  hasInitialData: import('vue').Ref<boolean>
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
  const hasInitialData = ref(false) // Track if we've received initial data (even if empty)

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
    // Clear previous errors and initial data flag
    error.value = null
    watchError.value = null
    hasInitialData.value = false
    
    // Unsubscribe from previous resource if any
    if (selectedResource.value) {
      try {
        await invoke('unsubscribe_from_resources', {
          resourceType: selectedResource.value.name.toLowerCase(),
          namespace: selectedResource.value.namespaced ? (namespaces.length === 1 ? namespaces[0] : null) : null
        })
      } catch (stopError) {
        console.warn('Failed to unsubscribe from previous resource:', stopError)
        // Don't throw here as this shouldn't prevent new subscription
      }
    }

    selectedResource.value = resource
    loading.value = true

    try {
      // Subscribe to new resource and get immediate cached data
      const cachedData = await invoke<K8sListItem[]>('subscribe_to_resources', {
        resourceType: resource.name.toLowerCase(),
        namespace: resource.namespaced ? (namespaces.length === 1 ? namespaces[0] : null) : null
      })
      
      // Set the cached data immediately (no more delay!)
      resourceItems.value = cachedData || []
      
      // Only set hasInitialData if we actually got cached data
      if (cachedData && cachedData.length > 0) {
        hasInitialData.value = true
      } else {
        // For empty or new subscriptions, wait for watch events or timeout
        createTimeout(() => {
          hasInitialData.value = true
          loading.value = false // Stop loading after timeout
        }, 2000) // 2 second timeout to allow initial watch events
      }
      
      // Clear any previous watch errors on successful start
      watchError.value = null
    } catch (watchErr: any) {
      console.error('Failed to subscribe to resource:', watchErr)
      const errorMessage = `Failed to watch ${resource.name}: ${watchErr?.toString() || 'Unknown error'}`
      watchError.value = errorMessage
      error.value = errorMessage
      resourceItems.value = []
      hasInitialData.value = true // Error responses count as "initial data received"
      throw new Error(errorMessage)
    } finally {
      // Only set loading = false if we have initial data
      // Otherwise, let the timeout handle it
      if (hasInitialData.value) {
        loading.value = false
      }
    }
  }

  async function changeNamespaces(newNamespaces: string[]): Promise<void> {
    // Clear any existing timeout safely
    if (namespaceChangeTimeout) {
      clearTimeout(namespaceChangeTimeout)
      namespaceChangeTimeout = null
    }
    
    // Clear previous errors but keep initial data flag (we're just switching namespaces)
    watchError.value = null
    
    // Don't restart watch if no resource is selected or resource is not namespaced
    if (!selectedResource.value || !selectedResource.value.namespaced) {
      return
    }
    
    // Set loading state immediately
    isChangingNamespaces.value = true
    loading.value = true
    
    try {
      // Subscribe to the same resource with new namespace - this will get immediate cached data!
      const cachedData = await invoke<K8sListItem[]>('subscribe_to_resources', {
        resourceType: selectedResource.value.name.toLowerCase(),
        namespace: newNamespaces.length === 1 ? newNamespaces[0] : null
      })
      
      // Set the cached data immediately (no delay!)
      resourceItems.value = cachedData || []
      
      // Always reset hasInitialData when changing namespaces - wait for watch events or timeout
      hasInitialData.value = false
      createTimeout(() => {
        hasInitialData.value = true
        loading.value = false // Stop loading after timeout
        isChangingNamespaces.value = false // Stop namespace changing state
      }, 2000) // 2 second timeout to allow initial watch events
      
      // Clear any previous watch errors on successful subscription
      watchError.value = null
    } catch (namespaceError: any) {
      console.error('Failed to switch namespaces:', namespaceError)
      const errorMessage = `Failed to switch namespaces for ${selectedResource.value?.name}: ${namespaceError?.toString() || 'Unknown error'}`
      watchError.value = errorMessage
      error.value = errorMessage
      resourceItems.value = []
      hasInitialData.value = true // Error responses count as "data received"
    } finally {
      // Don't set loading states to false here - let the timeout or InitialSyncComplete event handle it
      // This ensures we don't show "No X found" before we receive initial data
    }
  }

  function processWatchEvent(event: WatchEvent): void {
    // Only process events for the currently selected resource
    if (!selectedResource.value) {
      return
    }
    
    // Handle InitialSyncComplete specially - always process this even during namespace changes
    if (event === 'initialSyncComplete' || (typeof event === 'string' && event === 'initialSyncComplete')) {
      hasInitialData.value = true
      loading.value = false
      isChangingNamespaces.value = false
      return
    }
    
    // Skip other events during namespace changes to prevent conflicts
    if (isChangingNamespaces.value) {
      return
    }
    
    // Mark that we've received initial data from watch events
    hasInitialData.value = true
    loading.value = false
    isChangingNamespaces.value = false
    
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
      // Re-subscribe to get fresh data - the shared cache will handle this efficiently
      const cachedData = await invoke<K8sListItem[]>('subscribe_to_resources', {
        resourceType: selectedResource.value.name.toLowerCase(),
        namespace: selectedResource.value.namespaced ? (namespaces.length === 1 ? namespaces[0] : null) : null
      })
      
      // Update resource items with fresh data
      resourceItems.value = cachedData || []
    } catch (error) {
      console.error('Failed to refresh resource list after deletion:', error)
      throw error
    }
  }

  async function resetForClusterChange(): Promise<void> {
    // Unsubscribe from any current resource
    if (selectedResource.value) {
      try {
        await invoke('unsubscribe_from_resources', {
          resourceType: selectedResource.value.name.toLowerCase(),  
          namespace: selectedResource.value.namespaced ? 'default' : null
        })
      } catch (error) {
        console.warn('Failed to unsubscribe during cluster change:', error)
      }
    }
    
    // Clear all state
    selectedResource.value = null
    resourceItems.value = []
    loading.value = false
    isChangingNamespaces.value = false
    error.value = null
    watchError.value = null
    hasInitialData.value = false
    
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
    hasInitialData,
    
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