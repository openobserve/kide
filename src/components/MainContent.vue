<template>
  <div class="flex-1 flex flex-col overflow-hidden">
    <div class="flex-1 p-3 overflow-hidden">
      <!-- Show connection error state if connection failed, regardless of selected resource -->
      <EmptyState 
        v-if="connectionStatus === 'failed' || connectionStatus === 'connecting'"
        :selectedContext="selectedContext"
        :attemptedContext="attemptedContext"
        :connectionStatus="connectionStatus"
        :connectionError="connectionError"
        :hadSelectedResource="!!selectedResource"
        @reconnect="$emit('reconnect')"
      />
      <ResourceList 
        v-else-if="selectedResource && connectionStatus === 'connected'"
        :resource="selectedResource" 
        :items="resourceItems"
        :loading="loading || isChangingNamespaces"
        :namespaces="namespaces"
        :selectedNamespaces="selectedNamespaces"
        :error="error"
        :watchError="watchError"
        @namespace-change="$emit('namespace-change', $event)"
        @resource-deleted="$emit('resource-deleted')"
        @retry="$emit('retry')"
        @openPodLogs="openPodLogs"
        @openPodShell="openPodShell"
      />
      <EmptyState 
        v-else 
        :selectedContext="selectedContext"
        :attemptedContext="attemptedContext"
        :connectionStatus="connectionStatus"
        :connectionError="connectionError"
        :hadSelectedResource="!!selectedResource"
        @reconnect="$emit('reconnect')"
      />
    </div>
    
    <!-- Bottom Panel -->
    <BottomPanel
      ref="bottomPanelRef"
      v-model:isOpen="isBottomPanelOpen"
      @close="closeBottomPanel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, type Ref } from 'vue'
import ResourceList from './ResourceList.vue'
import EmptyState from './EmptyState.vue'
import BottomPanel from './BottomPanel.vue'
import type { 
  K8sResource, 
  K8sListItem, 
  K8sContext 
} from '@/types'

interface Props {
  selectedResource: K8sResource | null
  resourceItems: K8sListItem[]
  loading: boolean
  isChangingNamespaces: boolean
  namespaces: string[]
  selectedNamespaces: string[]
  selectedContext: K8sContext | null
  attemptedContext?: K8sContext | null
  connectionStatus?: string
  connectionError?: string | null
  error?: string | null
  watchError?: string | null
}

interface Emits {
  (e: 'namespace-change', namespaces: string[]): void
  (e: 'resource-deleted'): void
  (e: 'retry'): void
  (e: 'reconnect'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Template refs
const bottomPanelRef = ref<InstanceType<typeof BottomPanel> | null>(null)

// Bottom panel state
const isBottomPanelOpen = ref(false)

// Pod-specific functions
function openPodLogs(pod: K8sListItem): void {
  if (bottomPanelRef.value) {
    bottomPanelRef.value.openPodLogs(pod)
  }
}

function openPodShell(pod: K8sListItem): void {
  if (bottomPanelRef.value) {
    bottomPanelRef.value.openPodShell(pod)
  }
}

// Bottom panel functions
function closeBottomPanel(): void {
  isBottomPanelOpen.value = false
}
</script>