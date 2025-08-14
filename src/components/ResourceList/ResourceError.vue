<template>
  <div v-if="hasError" class="flex items-center justify-center h-full">
    <div class="text-center max-w-md mx-auto p-6">
      <!-- Error Icon -->
      <div class="flex justify-center mb-4">
        <div class="w-16 h-16 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
          <svg class="w-8 h-8 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
        </div>
      </div>
      
      <!-- Error Title -->
      <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
        {{ errorTitle }}
      </h3>
      
      <!-- Error Message -->
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
        {{ errorMessage }}
      </p>
      
      <!-- Error Details -->
      <div v-if="errorDetails" class="mb-4">
        <details class="text-left">
          <summary class="cursor-pointer text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300">
            Show technical details
          </summary>
          <div class="mt-2 text-xs text-gray-500 dark:text-gray-400 font-mono bg-gray-100 dark:bg-gray-800 p-3 rounded border">
            <pre class="whitespace-pre-wrap">{{ errorDetails }}</pre>
          </div>
        </details>
      </div>
      
      <!-- Suggested Actions -->
      <div class="space-y-2">
        <button 
          v-if="showRetry"
          @click="$emit('retry')"
          class="btn-primary w-full text-sm"
        >
          Retry
        </button>
        
        <div class="text-xs text-gray-500 dark:text-gray-400 space-y-1">
          <p v-if="isNamespaceError">
            • Check if the selected namespace exists
          </p>
          <p v-if="isPermissionError">
            • Verify your Kubernetes permissions for this resource type
          </p>
          <p v-if="isConnectionError">
            • Check your Kubernetes cluster connection
          </p>
          <p v-if="!isSpecificError">
            • Check your cluster connection and permissions
            • Verify the namespace exists and is accessible
            • Try selecting a different namespace or resource type
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  error?: string | null
  watchError?: string | null
  resourceName?: string
  showRetry?: boolean
}

const props = defineProps<Props>()

defineEmits<{
  retry: []
}>()

// Computed properties for error analysis
const hasError = computed(() => !!(props.error || props.watchError))

const primaryError = computed(() => props.watchError || props.error)

const errorTitle = computed(() => {
  if (!hasError.value) return ''
  
  if (isConnectionError.value) {
    return 'Connection Error'
  }
  
  if (isPermissionError.value) {
    return 'Permission Denied'
  }
  
  if (isNamespaceError.value) {
    return 'Namespace Error'
  }
  
  return `Failed to Load ${props.resourceName || 'Resources'}`
})

const errorMessage = computed(() => {
  if (!hasError.value) return ''
  
  if (isConnectionError.value) {
    return 'Unable to connect to the Kubernetes cluster. Please check your connection and try again.'
  }
  
  if (isPermissionError.value) {
    return 'You do not have permission to access this resource type in the selected namespace.'
  }
  
  if (isNamespaceError.value) {
    return 'The selected namespace may not exist or is not accessible.'
  }
  
  // Generic error message
  return `There was a problem loading ${props.resourceName?.toLowerCase() || 'resources'}. This might be due to connection issues, permissions, or the namespace not being accessible.`
})

const errorDetails = computed(() => primaryError.value)

// Error type detection
const isConnectionError = computed(() => {
  const error = primaryError.value?.toLowerCase() || ''
  return error.includes('connection') || 
         error.includes('timeout') || 
         error.includes('network') ||
         error.includes('unreachable')
})

const isPermissionError = computed(() => {
  const error = primaryError.value?.toLowerCase() || ''
  return error.includes('permission') || 
         error.includes('forbidden') || 
         error.includes('unauthorized') ||
         error.includes('access denied')
})

const isNamespaceError = computed(() => {
  const error = primaryError.value?.toLowerCase() || ''
  return error.includes('namespace') && (
    error.includes('not found') || 
    error.includes('does not exist') ||
    error.includes('invalid')
  )
})

const isSpecificError = computed(() => 
  isConnectionError.value || isPermissionError.value || isNamespaceError.value
)
</script>