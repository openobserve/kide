<template>
  <div v-if="hasError" class="error-boundary bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6 m-4">
    <div class="flex items-start space-x-3">
      <div class="flex-shrink-0">
        <svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.464 0L4.35 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
      </div>
      
      <div class="flex-1">
        <h3 class="text-lg font-semibold text-red-800 mb-2">Something went wrong</h3>
        
        <div v-if="userFriendlyMessage" class="text-red-700 mb-3">
          {{ userFriendlyMessage }}
        </div>
        
        <div class="space-y-2">
          <button 
            @click="retry"
            class="btn-danger inline-flex items-center px-3 py-2 text-sm leading-4 font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500">
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Try Again
          </button>
          
          <button 
            v-if="showDetails"
            @click="toggleDetails"
            class="ml-3 inline-flex items-center px-3 py-2 border border-red-300 dark:border-red-600 text-sm leading-4 font-medium rounded-md text-red-700 dark:text-red-300 bg-white dark:bg-red-900/20 hover:bg-red-50 dark:hover:bg-red-800/30 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors">
            {{ showErrorDetails ? 'Hide Details' : 'Show Details' }}
          </button>
        </div>
        
        <div v-if="showErrorDetails && error" class="mt-4 p-4 bg-red-100 dark:bg-red-900/20 rounded-md">
          <h4 class="text-sm font-semibold text-red-800 mb-2">Error Details:</h4>
          <pre class="text-xs text-red-700 whitespace-pre-wrap">{{ formatError(error) }}</pre>
        </div>
      </div>
    </div>
  </div>
  <slot v-else />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured, getCurrentInstance, computed } from 'vue'

interface Props {
  fallbackMessage?: string
  showDetails?: boolean
  onError?: (error: any, instance: any, info: string) => void
  onRetry?: () => void
}

const props = withDefaults(defineProps<Props>(), {
  fallbackMessage: '',
  showDetails: true,
})

const hasError = ref(false)
const error = ref<any>(null)
const errorInfo = ref('')
const showErrorDetails = ref(false)

const userFriendlyMessage = computed(() => {
  if (props.fallbackMessage) return props.fallbackMessage
  
  // Try to provide user-friendly messages for common error types
  if (error.value) {
    const errorMessage = error.value.message || error.value.toString()
    
    if (errorMessage.includes('Network')) {
      return 'Unable to connect to the server. Please check your internet connection.'
    }
    if (errorMessage.includes('timeout')) {
      return 'The request timed out. Please try again.'
    }
    if (errorMessage.includes('permission') || errorMessage.includes('authorized')) {
      return 'You don\'t have permission to access this resource.'
    }
    if (errorMessage.includes('not found')) {
      return 'The requested resource was not found.'
    }
    if (errorMessage.includes('cluster')) {
      return 'Unable to connect to the Kubernetes cluster. Please check your configuration.'
    }
  }
  
  return 'An unexpected error occurred. Please try again or contact support if the problem persists.'
})

// Error boundary hook
onErrorCaptured((err, instance, info) => {
  console.error('Error caught by boundary:', err, info)
  
  hasError.value = true
  error.value = err
  errorInfo.value = info
  
  // Call custom error handler if provided
  if (props.onError) {
    props.onError(err, instance, info)
  }
  
  // Report to error tracking service (could be Sentry, LogRocket, etc.)
  if (typeof window !== 'undefined' && window.reportError) {
    window.reportError(err, { component: instance?.$options?.name || instance?.$options?.__name || 'Unknown', info })
  }
  
  // Prevent the error from propagating further
  return false
})

function retry() {
  hasError.value = false
  error.value = null
  errorInfo.value = ''
  showErrorDetails.value = false
  
  if (props.onRetry) {
    props.onRetry()
  }
  
  // Force re-render of child components
  const instance = getCurrentInstance()
  if (instance) {
    instance.proxy?.$forceUpdate()
  }
}

function toggleDetails() {
  showErrorDetails.value = !showErrorDetails.value
}

function formatError(err: any): string {
  if (err.stack) {
    return err.stack
  }
  if (err.message) {
    return `${err.name || 'Error'}: ${err.message}`
  }
  return err.toString()
}

// Expose method for programmatic error triggering (useful for testing)
defineExpose({
  triggerError: (err: any) => {
    hasError.value = true
    error.value = err
  },
  clearError: () => {
    hasError.value = false
    error.value = null
  },
  retry
})
</script>

<style scoped>
.error-boundary {
  max-width: 100%;
  overflow-x: auto;
}

.error-boundary pre {
  max-height: 200px;
  overflow-y: auto;
}
</style>