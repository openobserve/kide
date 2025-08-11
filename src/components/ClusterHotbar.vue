<!-- ClusterHotbar.vue - Top tabs for selecting Kubernetes clusters -->
<template>
  <div class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex items-center h-12 px-4">
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
              ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 border-b-2 border-blue-700 dark:border-blue-400' 
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700'
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
      <!-- Theme toggle button -->
      <button
        @click="toggleTheme"
        class="h-8 w-8 rounded bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 flex items-center justify-center transition-colors"
        :title="theme.value === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
      >
        <!-- Sun icon for light mode (shown in dark mode) -->
        <svg 
          v-if="theme.value === 'dark'"
          class="w-4 h-4 text-yellow-500" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"/>
        </svg>
        <!-- Moon icon for dark mode (shown in light mode) -->
        <svg 
          v-else
          class="w-4 h-4 text-gray-600" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/>
        </svg>
      </button>
      
      <!-- Refresh button -->
      <button
        @click="refreshContexts"
        :disabled="refreshing"
        class="h-8 w-8 rounded bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 flex items-center justify-center transition-colors"
        title="Refresh contexts"
      >
        <svg 
          :class="['w-4 h-4 text-gray-600 dark:text-gray-400', refreshing ? 'animate-spin' : '']" 
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
import { useTheme } from '@/composables/useTheme'
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

// Theme management
const { theme, toggleTheme } = useTheme()

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
