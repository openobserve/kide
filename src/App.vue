<template>
  <div class="h-screen app-background flex flex-col overflow-hidden">
    <!-- Custom Title Bar with cluster dropdown -->
    <div class="flex-none">
      <TitleBar 
        :selectedContext="clusterStore.selectedContext"
        :connectionStatus="clusterStore.contextConnectionStatus"
        @context-selected="handleContextSelect"
        @refresh-contexts="handleRefreshContexts"
      />
    </div>
    
    <!-- Main content area with navigation and content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Resource Navigation -->
      <div class="flex-none">
        <ErrorBoundary fallback-message="Unable to load resource navigation">
          <ResourceNavigation 
            :categories="resourceStore.resourceCategories" 
            :selectedResource="resourceStore.selectedResource"
            @select-resource="handleResourceSelect"
            :connected="clusterStore.k8sConnected"
            :connectionStatus="clusterStore.connectionStatus"
            :currentContextName="clusterStore.selectedContext?.name"
          />
        </ErrorBoundary>
      </div>
      
      <!-- Main Content -->
      <MainContent
        :selectedResource="resourceStore.selectedResource"
        :resourceItems="resourceStore.resourceItems"
        :loading="resourceStore.loading"
        :isChangingNamespaces="resourceStore.isChangingNamespaces"
        :namespaces="clusterStore.namespaces"
        :selectedNamespaces="clusterStore.selectedNamespaces"
        :selectedContext="clusterStore.selectedContext"
        :attemptedContext="clusterStore.attemptedContext"
        :connectionStatus="clusterStore.connectionStatus"
        :connectionError="clusterStore.connectionError"
        :error="resourceStore.error"
        :watchError="resourceStore.watchError"
        :hasInitialData="resourceStore.hasInitialData"
        :isLoadingInBackground="resourceStore.isLoadingInBackground"
        @namespace-change="handleNamespaceChange"
        @resource-deleted="handleResourceDeleted"
        @retry="handleRetry"
        @reconnect="handleReconnect"
      />
      
    </div>
    
    <!-- Loading Progress Bar -->
    <LoadingProgressBar />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import TitleBar from './components/TitleBar.vue'
import ResourceNavigation from './components/ResourceNavigation.vue'
import MainContent from './components/MainContent.vue'
import ErrorBoundary from './components/ErrorBoundary.vue'
import LoadingProgressBar from './components/LoadingProgressBar.vue'
import { useClusterStore } from './stores/cluster'
import { useResourceStore } from './stores/resources'
import type { WatchEvent } from '@/types'

// Use Pinia stores
const clusterStore = useClusterStore()
const resourceStore = useResourceStore()

let unlisten: UnlistenFn | null = null
let unlistenBackgroundData: UnlistenFn | null = null

onMounted(async () => {
  try {
    // Load initial data
    await Promise.all([
      clusterStore.connectToCluster(),
      resourceStore.loadResourceCategories()
    ])
    
    // Set up event listener for watch events
    unlisten = await listen<WatchEvent>('k8s-watch-event', (event) => {
      console.log('🎯 RAW watch event received in App.vue:', event.payload)
      resourceStore.processWatchEvent(event.payload)
    })
    
    // Set up event listener for background data loading completion
    unlistenBackgroundData = await listen<string>('background-data-loaded', (event) => {
      const resourceType = event.payload
      console.log(`📦 Background data loaded for ${resourceType}`)
      
      // Handle background loading completion and refresh if current resource
      resourceStore.handleBackgroundDataLoaded(resourceType)
      
      // If this resource type is currently selected, refresh the resource list
      if (resourceStore.selectedResource?.name.toLowerCase() === resourceType.toLowerCase()) {
        resourceStore.refreshAfterResourceDeleted(clusterStore.selectedNamespaces)
      }
    })
  } catch (error) {
    console.error('Failed to initialize application:', error)
  }
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenBackgroundData) {
    unlistenBackgroundData()
  }
  resourceStore.cleanup()
})

// Event handlers
async function handleContextSelect(context: any): Promise<void> {
  try {
    // Reset resource store state when switching clusters
    await resourceStore.resetForClusterChange()
    
    // Clear any existing connection errors before attempting to connect
    clusterStore.connectionError = null
    
    await clusterStore.selectContext(context)
  } catch (error) {
    console.error('Failed to select context:', error)
    // Error is already handled by clusterStore.selectContext and stored in connectionError
    // The error will be displayed in the EmptyState component when selectedResource is null
  }
}

function handleRefreshContexts(): void {
  // The ClusterHotbar component handles its own refresh logic
}

async function handleResourceSelect(resource: any): Promise<void> {
  try {
    await resourceStore.selectResource(resource, clusterStore.selectedNamespaces)
  } catch (error) {
    console.error('Failed to select resource:', error)
    alert('Failed to start watching resource: ' + error)
  }
}

async function handleNamespaceChange(newNamespaces: string[]): Promise<void> {
  clusterStore.updateSelectedNamespaces(newNamespaces)
  await resourceStore.changeNamespaces(newNamespaces)
}

async function handleResourceDeleted(): Promise<void> {
  try {
    await resourceStore.refreshAfterResourceDeleted(clusterStore.selectedNamespaces)
  } catch (error) {
    console.error('Failed to refresh after resource deletion:', error)
  }
}

async function handleRetry(): Promise<void> {
  if (resourceStore.selectedResource) {
    try {
      await resourceStore.selectResource(resourceStore.selectedResource, clusterStore.selectedNamespaces)
    } catch (error) {
      console.error('Failed to retry resource watch:', error)
    }
  }
}

async function handleReconnect(): Promise<void> {
  try {
    await clusterStore.connectToCluster()
  } catch (error) {
    console.error('❌ Failed to reconnect to cluster:', error)
  }
}
</script>