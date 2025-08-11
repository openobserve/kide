<template>
  <ErrorBoundary 
    :fallback-message="getApiErrorMessage()"
    :show-details="showDetails"
    :on-error="handleApiError"
    :on-retry="handleRetry"
  >
    <slot />
  </ErrorBoundary>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ErrorBoundary from './ErrorBoundary.vue'

interface Props {
  showDetails?: boolean
  resourceType?: string
  operation?: string
  onRetry?: () => Promise<void> | void
}

const props = withDefaults(defineProps<Props>(), {
  showDetails: true,
  resourceType: 'resource',
  operation: 'load'
})

const lastError = ref<any>(null)

function getApiErrorMessage(): string {
  const resource = props.resourceType
  const operation = props.operation
  
  if (lastError.value) {
    const errorMessage = lastError.value.message || lastError.value.toString()
    
    // Kubernetes-specific error messages
    if (errorMessage.includes('RBAC') || errorMessage.includes('Forbidden') || errorMessage.includes('permission') || errorMessage.includes('authorized')) {
      return `You don't have sufficient permissions to ${operation} ${resource}. Please check your Kubernetes RBAC configuration.`
    }
    if (errorMessage.includes('namespace')) {
      return `Unable to access the specified namespace. It may not exist or you may not have permission.`
    }
    if (errorMessage.includes('context')) {
      return `Kubernetes context error. Please check your kubeconfig and ensure the cluster is accessible.`
    }
    if (errorMessage.includes('connection refused')) {
      return `Cannot connect to the Kubernetes cluster. Please verify the cluster is running and accessible.`
    }
    if (errorMessage.includes('timeout')) {
      return `Request timed out while trying to ${operation} ${resource}. The cluster may be slow to respond.`
    }
    if (errorMessage.includes('not found')) {
      return `The ${resource} was not found. It may have been deleted or moved to another namespace.`
    }
  }
  
  return `Failed to ${operation} ${resource}. Please check your connection and try again.`
}

function handleApiError(error: any, instance: any, info: string) {
  lastError.value = error
  
  // Log API errors for debugging
  console.error(`API Error in ${props.resourceType} ${props.operation}:`, error)
  
  // Could integrate with error reporting service here
  if (typeof window !== 'undefined' && window.reportApiError) {
    window.reportApiError(error, {
      resourceType: props.resourceType,
      operation: props.operation,
      component: instance?.type?.__name || instance?.$options?.__name || 'Unknown'
    })
  }
}

async function handleRetry() {
  if (props.onRetry) {
    try {
      await props.onRetry()
    } catch (error) {
      console.error('Retry failed:', error)
      // The error boundary will catch this automatically
      throw error
    }
  }
}
</script>