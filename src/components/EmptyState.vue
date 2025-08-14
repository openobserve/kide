<template>
  <div class="flex items-center justify-center h-full">
    <div class="text-center">
      <h2 class="text-2xl font-semibold text-gray-700 dark:text-gray-200 mb-4">
        Kide (Kubernetes IDE) by <button @click="openOpenObserve" class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 underline cursor-pointer">OpenObserve</button>
      </h2>
      
      <!-- Connection Status -->
      <div class="mb-4">
        <div v-if="connectionStatus === 'connected' && selectedContext" class="flex items-center justify-center space-x-2 text-green-600 dark:text-green-400">
          <div class="status-indicator-success"></div>
          <span class="text-sm font-medium">Connected to {{ selectedContext.name }}</span>
        </div>
        <div v-else-if="connectionStatus === 'connecting'" class="flex items-center justify-center space-x-2 text-yellow-600 dark:text-yellow-400">
          <div class="status-indicator-warning animate-pulse"></div>
          <span class="text-sm font-medium">Connecting{{ getContextName() ? ` to ${getContextName()}` : ' to cluster' }}...</span>
        </div>
        <div v-else-if="connectionStatus === 'failed'" class="text-red-600 dark:text-red-400">
          <div class="flex items-center justify-center space-x-2 mb-3">
            <div class="w-2 h-2 rounded-full bg-red-500"></div>
            <span class="text-sm font-medium">Failed to connect{{ getContextName() ? ` to ${getContextName()}` : ' to cluster' }}</span>
          </div>
          <div v-if="connectionError" class="text-left bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3 text-xs text-red-700 dark:text-red-300 max-w-lg mx-auto mb-4">
            <pre class="whitespace-pre-wrap font-mono">{{ connectionError }}</pre>
          </div>
          <button 
            @click="handleReconnect"
            :disabled="isReconnecting"
            class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <svg v-if="isReconnecting" class="w-4 h-4 mr-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <svg v-else class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            {{ isReconnecting ? 'Reconnecting...' : 'Reconnect' }}
          </button>
        </div>
      </div>
      
      <p class="text-gray-500 dark:text-gray-400">
        {{ message }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { K8sContext } from '@/types'

interface Props {
  selectedContext: K8sContext | null
  attemptedContext?: K8sContext | null
  connectionStatus?: string
  connectionError?: string | null
  hadSelectedResource?: boolean
}

const props = defineProps<Props>()

// Define emits
const emit = defineEmits<{
  reconnect: []
}>()

// Local state
const isReconnecting = ref(false)

// Function to handle reconnect
async function handleReconnect() {
  if (isReconnecting.value) return
  
  try {
    isReconnecting.value = true
    emit('reconnect')
  } catch (error) {
    console.error('Reconnect failed:', error)
  } finally {
    // Reset reconnecting state after a delay to allow parent to handle the reconnection
    setTimeout(() => {
      isReconnecting.value = false
    }, 1000)
  }
}

// Function to open OpenObserve website in default browser
async function openOpenObserve() {
  try {
    await invoke('open_url', { url: 'https://openobserve.ai' })
  } catch (error) {
    console.error('Failed to open OpenObserve website:', error)
  }
}

// Get the context name to display, preferring selectedContext but falling back to attemptedContext
const getContextName = () => {
  return props.selectedContext?.name || props.attemptedContext?.name || null
}

const message = computed(() => {
  if (!props.selectedContext) {
    return 'Select a cluster context to get started.'
  }
  
  if (props.connectionStatus === 'connected') {
    return 'Select a resource type from the navigation to view resources.'
  } else if (props.connectionStatus === 'connecting') {
    return 'Establishing connection...'
  } else {
    // Connection failed
    if (props.hadSelectedResource) {
      return 'Lost connection to cluster. Unable to load resources. Check the error details below and try switching to a different context.'
    } else {
      return 'Unable to connect to cluster. Check your configuration and try again.'
    }
  }
})
</script>