<template>
  <ApiErrorBoundary 
    :resource-type="resource?.name || 'resources'"
    operation="load"
    :on-retry="handleRetry"
  >
    <div class="panel-background rounded-lg shadow h-full flex flex-col relative">
      <!-- Resource Details Panel -->
      <ResourcePanel 
        ref="resourcePanelRef"
        :isOpen="isResourcePanelOpen"
        :resourceData="selectedResourceData"
        :resourceKind="selectedResourceData?.kind || resource?.kind || 'Resource'"
        @close="closeResourcePanel"
        @viewPod="handleViewPod"
      />
    
    <!-- Header -->
    <div class="px-4 py-2 border-b border-border-primary">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-shrink-0">
          <h1 class="text-lg font-semibold text-text-primary">
            {{ resource?.name || 'Unknown Resource' }}
          </h1>
        </div>
        <div class="flex items-center space-x-4 flex-1 justify-end">
          <!-- Namespace multi-select for namespaced resources -->
          <div v-if="resource?.namespaced && namespaces && namespaces.length > 0" class="w-80">
            <MultiSelectNamespace
              :namespaces="namespaces || []"
              :selectedNamespaces="selectedNamespaces || []"
              @update:selectedNamespaces="handleNamespaceChange"
            />
          </div>
          
          <!-- Filter text field -->
          <div class="w-80">
            <div class="mt-1 relative">
              <input
                v-model="filterText"
                type="text"
                placeholder="Search resources..."
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                data-form-type="other"
                data-lpignore="true"
                readonly
                onfocus="this.removeAttribute('readonly');"
                class="w-full pl-3 pr-10 py-2 text-left form-input border border-border-primary rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500  sm:text-sm"
              >
              <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
            </div>
            <!-- Filter results indicator -->
            <div v-if="filterText && filteredItems.length !== items.length" class="mt-1 text-xs text-text-secondary">
              {{ filteredItems.length }} of {{ items.length }} resources
            </div>
          </div>
          
          <div class="flex items-center space-x-2" v-if="resource">
            <span :class="[
              'px-2 py-1 rounded-full text-xs font-medium',
              resource.namespaced 
                ? 'bg-blue-100 text-blue-800' 
                : 'bg-purple-100 text-purple-800'
            ]">
              {{ resource.namespaced ? 'Namespaced' : 'Cluster-wide' }}
            </span>
            <span class="text-sm text-gray-500">{{ resource.apiVersion }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Table content -->
    <div class="flex-1 overflow-hidden">
      <!-- Error State -->
      <ResourceError 
        v-if="error || watchError"
        :error="error"
        :watchError="watchError"
        :resourceName="resource?.name"
        :showRetry="true"
        @retry="handleRetry"
      />
      
      <!-- Empty State (only show when we have received initial data and have zero results) -->
      <div v-else-if="hasInitialData && items.length === 0" class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="text-text-muted mb-4">
            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2 2v-5m16 0h-2M4 13h2m0 0V9a2 2 0 012-2h2m0 0V6a2 2 0 012-2h2a2 2 0 012 2v1M6 7h.01M6 11h.01" />
            </svg>
          </div>
          <h3 class="text-lg font-medium text-text-primary mb-2">No {{ resource?.name || 'Resources' }} found</h3>
          <p class="text-text-secondary">There are no {{ resource?.name.toLowerCase() || 'resources' }} in the current context.</p>
        </div>
      </div>

      <div v-else class="h-full flex flex-col">
        <!-- Error Banner (when we have data but also errors) -->
        <div 
          v-if="(error || watchError) && items.length > 0" 
          class="bg-status-warning/10 border-b border-status-warning/30 p-3"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center">
              <svg class="w-4 h-4 text-status-warning mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              <span class="text-sm text-status-warning">
                {{ watchError || error }}
              </span>
            </div>
            <button 
              @click="handleRetry"
              class="text-sm text-status-warning hover:opacity-80 underline"
            >
              Retry
            </button>
          </div>
        </div>
        
        <!-- TanStack Table -->
        <ResourceTable
          :resource="resource"
          :items="filteredItems"
          :selectedItems="selectedItems"
          :selectedItem="selectedItem"
          :hoveredRowId="hoveredRowId"
          :isMouseOverTable="isMouseOverTable"
          :getContainerStatusColor="getContainerStatusColor"
          :getContainerStatusText="getContainerStatusText"
          :getTotalRestartCount="getTotalRestartCount"
          :getControlledBy="getControlledBy"
          :getQoSClass="getQoSClass"
          :getStatusClass="getStatusClass"
          :getStatusText="getStatusText"
          :getAge="getAge"
          @toggleSelectAll="toggleSelectAll"
          @setHoveredRow="setHoveredRow"
          @clearHoveredRow="forceClearHoveredRow"
          @toggleItemSelection="toggleItemSelection"
          @selectItem="selectItemForPanel"
          @openPodLogs="$emit('openPodLogs', $event)"
          @openPodShell="$emit('openPodShell', $event)"
          @deleteResource="handleDeleteResource"
          @selectNamespace="handleNamespaceSelect"
          @toggleCronJobSuspend="handleToggleCronJobSuspend"
          @triggerCronJob="handleTriggerCronJob"
        />
          
          <!-- Empty state -->
          <div v-if="filteredItems.length === 0 && items.length > 0" class="flex items-center justify-center py-12">
            <div class="text-center">
              <div class="text-gray-400 mb-4">
                <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">No matches found</h3>
              <p class="text-gray-600">No resources match "{{ filterText }}". Try a different search term.</p>
            </div>
          </div>
          
          <div v-else-if="hasInitialData && items.length === 0" class="flex items-center justify-center py-12">
            <div class="text-center">
              <div class="text-gray-400 mb-4">
                <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2 2v-5m16 0h-2M4 13h2m0 0V9a2 2 0 012-2h2m0 0V6a2 2 0 012-2h2a2 2 0 012 2v1M6 7h.01M6 11h.01" />
                </svg>
              </div>
              <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">No {{ resource?.name || 'Resources' }} found</h3>
              <p class="text-gray-600">There are no {{ resource?.name.toLowerCase() || 'resources' }} in the current context.</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Floating Delete Button -->
    <div 
      v-if="selectedItems.size > 0"
      class="fixed bottom-6 right-6 z-30"
    >
      <button
        @click="showDeleteConfirmation = true"
        class="bg-status-error text-white hover:opacity-90 transition-all duration-200 transform hover:scale-110 rounded-full p-4 shadow-lg hover:shadow-xl font-medium"
        :title="`Delete ${selectedItems.size} selected resource${selectedItems.size > 1 ? 's' : ''}`"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
        </svg>
        <span class="absolute -top-2 -right-2 bg-red-800 text-white rounded-full w-6 h-6 flex items-center justify-center text-xs font-bold">
          {{ selectedItems.size }}
        </span>
      </button>
    </div>

    <!-- Bulk Delete Confirmation Dialog -->
    <DeleteConfirmationDialog
      :show="showDeleteConfirmation"
      :selectedCount="selectedItems.size"
      :selectedResources="getSelectedResourceItems(props.items)"
      @cancel="showDeleteConfirmation = false"
      @confirm="deleteSelectedResources"
    />

    <!-- Single Resource Delete Confirmation Dialog -->
    <DeleteConfirmationDialog
      :show="showSingleDeleteConfirmation"
      :selectedCount="1"
      :selectedResources="pendingDeleteItem ? [pendingDeleteItem] : []"
      @cancel="cancelSingleDelete"
      @confirm="confirmSingleDelete"
    />

  </ApiErrorBoundary>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Import composables
import { useTimeouts } from '@/composables/useTimeouts'
import { useResourceStatus } from '@/composables/useResourceStatus'
import { useContainerStatus } from '@/composables/useContainerStatus'
import { useDebouncedSearch } from '@/composables/useDebouncedSearch'
import { useResourceSelection } from '@/composables/useResourceSelection'
import { useResourceFiltering } from '@/composables/useResourceFiltering'
import { useResourceSorting } from '@/composables/useResourceSorting'
import { TIMEOUTS } from '@/constants/timeouts'

// Import utilities
import { getAge, getDetailedAge } from '@/utils/timeFormatters'

// Import new components
import ResourceTable from './ResourceList/ResourceTable.vue'
import DeleteConfirmationDialog from './ResourceList/DeleteConfirmationDialog.vue'
import ResourceError from './ResourceList/ResourceError.vue'

// Import existing components
import MultiSelectNamespace from './MultiSelectNamespace.vue'
import ResourcePanel from './ResourcePanel/index.vue'
import ApiErrorBoundary from './ApiErrorBoundary.vue'

import type { 
  K8sResource, 
  K8sListItem, 
  ContainerStatus 
} from '@/types'


interface Props {
  resource: K8sResource | null
  items: K8sListItem[]
  loading?: boolean
  namespaces?: string[]
  selectedNamespaces?: string[]
  error?: string | null
  watchError?: string | null
  hasInitialData?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'namespace-change': [namespaces: string[]]
  'resource-deleted': []
  'retry': []
  'openPodLogs': [pod: K8sListItem]
  'openPodShell': [pod: K8sListItem]
}>()

// Composables
const { createTimeout, clearTimeoutSafe } = useTimeouts()
const { getStatusText, getStatusClass, getStatusForKind } = useResourceStatus()
const { getContainerStatusColor, getContainerStatusText, getTotalRestartCount, getControlledBy, getQoSClass } = useContainerStatus()
const { searchQuery: filterText, debouncedQuery: debouncedFilterText } = useDebouncedSearch(300)
const {
  selectedItems,
  selectedItem,
  selectedCount,
  toggleSelectAll: toggleSelectAllFn,
  toggleItemSelection,
  selectItem,
  clearSelection,
  getSelectedResourceItems,
  isItemSelected
} = useResourceSelection()
const { filteredItems: baseFilteredItems } = useResourceFiltering(computed(() => props.items), debouncedFilterText)
const { sortColumn, sortDirection, handleSort, sortItems } = useResourceSorting(computed(() => props.resource))

// Template refs
const resourcePanelRef = ref<InstanceType<typeof ResourcePanel> | null>(null)

// State management
const showDeleteConfirmation = ref(false)
const showSingleDeleteConfirmation = ref(false)
const pendingDeleteItem = ref<K8sListItem | null>(null)
const hoveredRowId = ref<string | null>(null)
const hoverTimeout = ref<NodeJS.Timeout | null>(null)
const maxHoverTimeout = ref<NodeJS.Timeout | null>(null)
const isMouseOverTable = ref(false)

// Note: Sorting state now handled by useResourceSorting composable

// Resource panel state (unified for all resources)
const isResourcePanelOpen = ref(false)
const selectedResourceId = ref<string | null>(null)
const externalResourceData = ref<any | null>(null)

// Computed property to get the current state of the selected resource from the store
const selectedResourceData = computed(() => {
  if (!selectedResourceId.value) return null
  
  // If we have external resource data (like a pod from another view), use that
  if (externalResourceData.value && externalResourceData.value.metadata?.uid === selectedResourceId.value) {
    return externalResourceData.value
  }
  
  // Find the current item with the selected ID in the items array
  const currentItem = props.items.find((item: K8sListItem) => item.metadata?.uid === selectedResourceId.value)
  return currentItem || null
})



// Single resource deletion functions
function cancelSingleDelete(): void {
  showSingleDeleteConfirmation.value = false
  pendingDeleteItem.value = null
}

async function confirmSingleDelete(): Promise<void> {
  if (pendingDeleteItem.value) {
    await deleteResourceInternal(pendingDeleteItem.value)
  }
  showSingleDeleteConfirmation.value = false
  pendingDeleteItem.value = null
}

// Hover management functions
function setHoveredRow(itemId: string | undefined, source = 'unknown'): void {
  const id = itemId || null
  
  // Clear existing timeouts safely
  clearTimeoutSafe(hoverTimeout.value)
  clearTimeoutSafe(maxHoverTimeout.value)
  hoverTimeout.value = null
  maxHoverTimeout.value = null
  
  hoveredRowId.value = id
  
  if (id) {
    maxHoverTimeout.value = createTimeout(() => {
      hoveredRowId.value = null
      maxHoverTimeout.value = null
    }, TIMEOUTS.HOVER_MAX_DURATION)
  }
}

function clearHoveredRow(source = 'unknown'): void {
  // Don't clear hover when source is from button container - let the row handle it
  if (source === 'button-container') {
    return
  }
  
  clearTimeoutSafe(hoverTimeout.value)
  hoverTimeout.value = null
  
  if (!isMouseOverTable.value) {
    hoveredRowId.value = null
    return
  }
  
  // Increase delay to make hover more sticky and give time to move to buttons
  hoverTimeout.value = createTimeout(() => {
    hoveredRowId.value = null
    hoverTimeout.value = null
  }, TIMEOUTS.HOVER_DELAY * 2) // Double the delay for better UX
}

function forceClearHoveredRow(reason = 'unknown'): void {
  clearTimeoutSafe(hoverTimeout.value)
  clearTimeoutSafe(maxHoverTimeout.value)
  hoverTimeout.value = null
  maxHoverTimeout.value = null
  hoveredRowId.value = null
}

// Sorting functions now handled by useResourceSorting composable

// Computed property for filtered and sorted items
const filteredItems = computed(() => {
  // Apply sorting to filtered items
  return sortItems(baseFilteredItems.value)
})

// Event handlers
function handleNamespaceChange(newNamespaces: string[]): void {
  emit('namespace-change', newNamespaces)
}

function handleNamespaceSelect(namespace: string): void {
  // When a namespace is clicked, filter to show only that namespace
  emit('namespace-change', [namespace])
}

function selectItemForPanel(item: K8sListItem): void {
  selectItem(item) // Use composable function
  selectedResourceId.value = item.metadata?.uid || null
  isResourcePanelOpen.value = true
}

function closeResourcePanel(): void {
  isResourcePanelOpen.value = false
  selectedResourceId.value = null
  externalResourceData.value = null
}

function handleViewPod(pod: any): void {
  // Normalize pod data structure to match what useResourceStatus expects
  const normalizedPod = {
    ...pod,
    kind: 'Pod',
    // Map standard Kubernetes fields to the expected format
    podStatus: pod.status,
    podSpec: pod.spec
  }
  
  // Store external pod data and show it in the resource panel
  externalResourceData.value = normalizedPod
  selectedResourceId.value = pod.metadata?.uid || null
  isResourcePanelOpen.value = true
}

function toggleSelectAll(): void {
  toggleSelectAllFn(filteredItems.value)
}

// toggleItemSelection now handled by composable

// getSelectedResourceItems now handled by composable

async function deleteSelectedResources(): Promise<void> {
  showDeleteConfirmation.value = false
  
  for (const item of getSelectedResourceItems(props.items)) {
    try {
      await deleteResourceInternal(item, false)
    } catch (error) {
      console.error('Failed to delete resource:', item.metadata?.name, error)
    }
  }
  
  selectedItems.value.clear()
  emit('resource-deleted')
}

// Handle delete resource request (shows confirmation)
function handleDeleteResource(item: K8sListItem): void {
  pendingDeleteItem.value = item
  showSingleDeleteConfirmation.value = true
}


// Actually delete the resource (internal function)
async function deleteResourceInternal(item: K8sListItem, shouldEmitDeleted = true): Promise<void> {
  if (!props.resource) return
  
  try {
    await invoke('delete_resource', {
      resourceKind: item.kind || props.resource.kind,
      resourceName: item.metadata?.name,
      namespace: item.metadata?.namespace
    })
    
    
    if (shouldEmitDeleted) {
      emit('resource-deleted')
    }
  } catch (error) {
    console.error('❌ Failed to delete resource:', error)
    throw error
  }
}


// Helper functions are now provided by composables"}, {"old_string": "            <div v-if=\"filterText && filteredItems.length !== items.length\" class=\"mt-1 text-xs text-text-secondary\">\n              {{ filteredItems.length }} of {{ items.length }} resources\n            </div>", "new_string": "            <div v-if=\"debouncedFilterText && filteredItems.length !== items.length\" class=\"mt-1 text-xs text-text-secondary\">\n              {{ filteredItems.length }} of {{ items.length }} resources\n            </div>"}, {"old_string": "              <p class=\"text-gray-600\">No resources match \"{{ filterText }}\". Try a different search term.</p>", "new_string": "              <p class=\"text-gray-600\">No resources match \"{{ debouncedFilterText }}\". Try a different search term.</p>"}]

function handleRetry(): void {
  emit('retry')
}

// Handle CronJob suspend/resume
async function handleToggleCronJobSuspend(item: K8sListItem, suspend: boolean): Promise<void> {
  if (!props.resource || props.resource.kind !== 'CronJob') return
  
  try {
    await invoke('toggle_cronjob_suspend', {
      name: item.metadata?.name,
      namespace: item.metadata?.namespace,
      suspend: suspend
    })
    
    // Emit event to refresh the data
    emit('resource-deleted') // Reusing this event to trigger a refresh
  } catch (error) {
    console.error('❌ Failed to toggle CronJob suspend state:', error)
    // TODO: Show user-friendly error notification
  }
}

// Handle CronJob trigger
async function handleTriggerCronJob(item: K8sListItem): Promise<void> {
  if (!props.resource || props.resource.kind !== 'CronJob') return
  
  try {
    await invoke('trigger_cronjob', {
      name: item.metadata?.name,
      namespace: item.metadata?.namespace
    })
    
    // Emit event to refresh the data to show new job
    emit('resource-deleted') // Reusing this event to trigger a refresh
  } catch (error) {
    console.error('❌ Failed to trigger CronJob:', error)
    // TODO: Show user-friendly error notification
  }
}



// Note: Timeout cleanup is handled automatically by useTimeouts composable
</script>