<!-- ClusterHotbar.vue - Top tabs for selecting Kubernetes clusters -->
<template>
  <div class="bg-gray-800 border-b border-gray-700 flex items-center h-12 px-4">
    <!-- Cluster tabs -->
    <div class="flex-1 flex items-center space-x-1 overflow-x-auto">
      <div 
        v-for="context in contexts" 
        :key="context.name"
        class="relative group flex-shrink-0"
      >
        <button
          @click="selectContext(context)"
          :class="[
            'px-4 py-2 rounded-t-lg text-sm font-medium transition-colors flex items-center space-x-2 min-w-0',
            context.name === selectedContext?.name 
              ? 'bg-blue-900/30 text-blue-300 border-b-2 border-blue-400' 
              : 'text-gray-400 hover:text-gray-200 hover:bg-gray-700'
          ]"
          :title="context.name"
        >
          <!-- Connection status indicator -->
          <div 
            :class="[
              'w-2 h-2 rounded-full flex-shrink-0',
              getContextStatus(context) === 'connected' ? 'bg-green-500' :
              getContextStatus(context) === 'connecting' ? 'bg-yellow-500' :
              getContextStatus(context) === 'failed' ? 'bg-red-500' : 'bg-gray-400'
            ]"
          />
          
          <!-- Truncated context name -->
          <span class="truncate max-w-48" :title="context.name">
            {{ truncateContextName(context.name) }}
          </span>
        </button>

      </div>
    </div>

    <!-- Controls -->
    <div class="flex-shrink-0 ml-4 flex items-center space-x-2">
      <!-- Refresh button -->
      <button
        @click="refreshContexts"
        :disabled="refreshing"
        class="h-8 w-8 rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-50 flex items-center justify-center transition-colors"
        title="Refresh contexts"
      >
        <svg 
          :class="['w-4 h-4 text-gray-400', refreshing ? 'animate-spin' : '']" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { K8sContext, ContextStatus } from '@/types'

interface Props {
  selectedContext?: K8sContext | null
  connectionStatus?: Record<string, ContextStatus>
}

interface Emits {
  (e: 'context-selected', context: K8sContext): void
  (e: 'refresh-contexts'): void
}

const props = withDefaults(defineProps<Props>(), {
  selectedContext: null,
  connectionStatus: () => ({})
})

const emit = defineEmits<Emits>()

const contexts: Ref<K8sContext[]> = ref([])
const refreshing = ref(false)

onMounted(async () => {
  await loadContexts()
})

async function loadContexts(): Promise<void> {
  try {
    refreshing.value = true
    const fetchedContexts = await invoke<K8sContext[]>('get_k8s_contexts')
    contexts.value = fetchedContexts
  } catch (error) {
    console.error('Failed to load contexts:', error)
    contexts.value = []
  } finally {
    refreshing.value = false
  }
}

async function refreshContexts(): Promise<void> {
  emit('refresh-contexts')
  await loadContexts()
}

function selectContext(context: K8sContext): void {
  emit('context-selected', context)
}

function getContextStatus(context: K8sContext): ContextStatus {
  return props.connectionStatus[context.name] || 'disconnected'
}

function truncateContextName(name: string): string {
  if (name.length <= 25) {
    return name
  }
  return name.substring(0, 25) + '...'
}
</script>

<style scoped>
/* Custom scrollbar for horizontal tabs overflow */
.overflow-x-auto::-webkit-scrollbar {
  height: 4px;
}

.overflow-x-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-x-auto::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 2px;
}

.overflow-x-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}

/* Ensure tabs container scrolls horizontally */
.overflow-x-auto {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}
</style>
