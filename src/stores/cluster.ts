import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { K8sContext, ContextStatus, ConnectionStatus } from '@/types'

export const useClusterStore = defineStore('cluster', () => {
  // State
  const selectedContext = ref<K8sContext | null>(null)
  const attemptedContext = ref<K8sContext | null>(null) // Track context being attempted even if it fails
  const contextConnectionStatus = ref<Record<string, ContextStatus>>({})
  const k8sConnected = ref(false)
  const connectionStatus = ref<ConnectionStatus>('connecting')
  const connectionError = ref<string | null>(null)
  const contextErrors = ref<Record<string, string>>({})
  const namespaces = ref<string[]>([])
  const selectedNamespaces = ref<string[]>(['default'])

  // Actions
  async function connectToCluster(): Promise<void> {
    try {
      connectionStatus.value = 'connecting'
      connectionError.value = null
      
      // First try to get contexts to see what's available
      const contexts = await invoke<K8sContext[]>('get_k8s_contexts')
      
      if (contexts.length > 0) {
        // Get the current context from kubeconfig
        try {
          const currentContextName = await invoke<string>('get_current_k8s_context')
          const currentContext = contexts.find(ctx => ctx.name === currentContextName) || contexts[0]
          await selectContext(currentContext)
        } catch (error) {
          console.warn('Failed to get current context, using first available:', error)
          await selectContext(contexts[0])
        }
      } else {
        // Fallback to default connection if no contexts found
        await invoke('connect_k8s')
        k8sConnected.value = true
        connectionStatus.value = 'connected'
        await fetchNamespaces()
      }
    } catch (error: any) {
      console.error('Failed to connect to Kubernetes:', error)
      connectionStatus.value = 'failed'
      connectionError.value = formatConnectionError(error)
      throw error
    }
  }

  async function selectContext(context: K8sContext): Promise<void> {
    
    // Track the context being attempted
    attemptedContext.value = context
    
    // Update connection status
    contextConnectionStatus.value[context.name] = 'connecting'
    
    try {
      // Clear current state
      k8sConnected.value = false
      connectionStatus.value = 'connecting'
      connectionError.value = null // Clear any previous errors
      
      // Connect with the selected context
      await invoke('connect_k8s_with_context', { contextName: context.name })
      
      // Update state after successful connection
      selectedContext.value = context
      k8sConnected.value = true
      connectionStatus.value = 'connected'
      contextConnectionStatus.value[context.name] = 'connected'
      connectionError.value = null // Clear any previous errors
      
      // Fetch namespaces for the new context
      await fetchNamespaces()
      
      // Reset selected namespaces when switching clusters
      if (namespaces.value.includes('default')) {
        selectedNamespaces.value = ['default']
      } else if (namespaces.value.length > 0) {
        selectedNamespaces.value = [namespaces.value[0]]
      } else {
        selectedNamespaces.value = []
      }
      
    } catch (error: any) {
      console.error('❌ Failed to switch context:', error)
      contextConnectionStatus.value[context.name] = 'failed'
      connectionStatus.value = 'failed'
      const errorMessage = formatConnectionError(error)
      contextErrors.value[context.name] = errorMessage
      connectionError.value = errorMessage
      
      throw error
    }
  }

  async function fetchNamespaces(): Promise<void> {
    try {
      const ns = await invoke<string[]>('get_namespaces')
      namespaces.value = ns
    } catch (error) {
      console.error('Failed to fetch namespaces:', error)
      namespaces.value = ['default'] // Fallback to default namespace
      throw error
    }
  }

  function updateSelectedNamespaces(newNamespaces: string[]): void {
    selectedNamespaces.value = newNamespaces
  }

  // Helper function to format connection errors
  function formatConnectionError(error: any): string {
    if (typeof error === 'string') {
      return error
    }
    
    if (error?.message) {
      let message = error.message
      
      // Common Kubernetes connection error patterns
      if (message.includes('connection refused')) {
        return `Connection refused - The Kubernetes API server is not reachable. Please check:\n• Cluster is running\n• Network connectivity\n• Correct API server address in kubeconfig\n\nOriginal error: ${message}`
      }
      
      if (message.includes('certificate')) {
        return `Certificate error - There's an issue with SSL/TLS certificates. Please check:\n• Certificate validity\n• CA certificate configuration\n• Client certificates\n\nOriginal error: ${message}`
      }
      
      if (message.includes('authentication') || message.includes('unauthorized') || message.includes('auth exec')) {
        return `Authentication failed - Credentials are invalid or expired. Please check:\n• User credentials in kubeconfig\n• Token expiration\n• Service account permissions\n\nOriginal error: ${message}`
      }
      
      if (message.includes('timeout')) {
        return `Connection timeout - The request to Kubernetes API timed out. Please check:\n• Network connectivity\n• Cluster responsiveness\n• Firewall settings\n\nOriginal error: ${message}`
      }
      
      if (message.includes('kubeconfig') || message.includes('config')) {
        return `Configuration error - Issue with kubeconfig file. Please check:\n• Kubeconfig file exists and is readable\n• Valid kubeconfig format\n• Context and cluster configuration\n\nOriginal error: ${message}`
      }
      
      if (message.includes('not found') || message.includes('No such')) {
        return `Resource not found - The specified context or cluster was not found. Please check:\n• Context name is correct\n• Kubeconfig contains the context\n• Cluster configuration exists\n\nOriginal error: ${message}`
      }
      
      // Return the original message if no pattern matches
      return `Connection failed: ${message}`
    }
    
    // Fallback for unknown error types
    return `Unknown connection error: ${JSON.stringify(error)}`
  }

  // Getters
  const isConnected = () => k8sConnected.value && connectionStatus.value === 'connected'
  const currentContextName = () => selectedContext.value?.name || null

  return {
    // State
    selectedContext,
    attemptedContext,
    contextConnectionStatus,
    k8sConnected,
    connectionStatus,
    connectionError,
    contextErrors,
    namespaces,
    selectedNamespaces,
    
    // Actions
    connectToCluster,
    selectContext,
    fetchNamespaces,
    updateSelectedNamespaces,
    
    // Getters
    isConnected,
    currentContextName,
  }
})