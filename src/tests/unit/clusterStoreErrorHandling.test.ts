import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useClusterStore } from '../../stores/cluster'
import type { MockedFunction } from 'vitest'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Cluster Store Error Handling', () => {
  let mockInvoke: MockedFunction<any>
  let clusterStore: ReturnType<typeof useClusterStore>

  beforeEach(async () => {
    vi.clearAllMocks()
    setActivePinia(createPinia())
    
    const coreModule = await import('@tauri-apps/api/core')
    mockInvoke = vi.mocked(coreModule.invoke)
    
    clusterStore = useClusterStore()
  })

  describe('selectContext error handling', () => {
    it('should set connectionStatus to failed and store error message on connection failure', async () => {
      const errorMessage = 'auth exec command error: token expired'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'prod-cluster', cluster: 'prod', user: 'admin' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionStatus).toBe('failed')
      expect(clusterStore.connectionError).toContain(errorMessage)
      expect(clusterStore.k8sConnected).toBe(false)
      expect(clusterStore.contextConnectionStatus['prod-cluster']).toBe('failed')
    })

    it('should clear previous error when starting new connection attempt', async () => {
      // First, set an error state
      clusterStore.connectionError = 'previous error'
      clusterStore.connectionStatus = 'failed'

      const mockContext = { name: 'dev-cluster', cluster: 'dev', user: 'user' }

      // Mock successful connection this time
      mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
      mockInvoke.mockResolvedValueOnce(['default', 'kube-system']) // get_namespaces

      await clusterStore.selectContext(mockContext)

      expect(clusterStore.connectionError).toBe(null)
      expect(clusterStore.connectionStatus).toBe('connected')
      expect(clusterStore.selectedContext).toEqual(mockContext)
    })

    it('should set connecting status before attempting connection', async () => {
      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }
      
      // Mock delayed response to test intermediate state
      let resolvePromise: (value: any) => void
      const delayedPromise = new Promise(resolve => {
        resolvePromise = resolve
      })
      
      mockInvoke.mockReturnValueOnce(delayedPromise)

      // Start connection (don't await yet)
      const connectionPromise = clusterStore.selectContext(mockContext)
      
      // Check intermediate state
      expect(clusterStore.connectionStatus).toBe('connecting')
      expect(clusterStore.contextConnectionStatus[mockContext.name]).toBe('connecting')
      expect(clusterStore.k8sConnected).toBe(false)
      expect(clusterStore.connectionError).toBe(null)

      // Complete the connection
      resolvePromise!(undefined)
      mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces
      await connectionPromise

      expect(clusterStore.connectionStatus).toBe('connected')
    })
  })

  describe('formatConnectionError function', () => {
    it('should format connection refused errors with helpful context', async () => {
      const errorMessage = 'connection refused: dial tcp 127.0.0.1:8080: connect: connection refused'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toContain('Connection refused')
      expect(clusterStore.connectionError).toContain('Cluster is running')
      expect(clusterStore.connectionError).toContain('Network connectivity')
      expect(clusterStore.connectionError).toContain(errorMessage)
    })

    it('should format authentication errors with helpful context', async () => {
      const errorMessage = 'auth exec command error: token has expired'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toContain('Authentication failed')
      expect(clusterStore.connectionError).toContain('User credentials in kubeconfig')
      expect(clusterStore.connectionError).toContain('Token expiration')
      expect(clusterStore.connectionError).toContain(errorMessage)
    })

    it('should format certificate errors with helpful context', async () => {
      const errorMessage = 'certificate verify failed: unable to get local issuer certificate'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toContain('Certificate error')
      expect(clusterStore.connectionError).toContain('Certificate validity')
      expect(clusterStore.connectionError).toContain('CA certificate configuration')
      expect(clusterStore.connectionError).toContain(errorMessage)
    })

    it('should format timeout errors with helpful context', async () => {
      const errorMessage = 'request timeout: context deadline exceeded'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toContain('Connection timeout')
      expect(clusterStore.connectionError).toContain('Network connectivity')
      expect(clusterStore.connectionError).toContain('Cluster responsiveness')
      expect(clusterStore.connectionError).toContain(errorMessage)
    })

    it('should handle string errors directly', async () => {
      const errorMessage = 'simple string error'
      mockInvoke.mockRejectedValueOnce(errorMessage)

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toBe(errorMessage)
    })

    it('should handle unknown error types gracefully', async () => {
      const unknownError = { code: 500, detail: 'internal error' }
      mockInvoke.mockRejectedValueOnce(unknownError)

      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionError).toContain('Unknown connection error')
      expect(clusterStore.connectionError).toContain(JSON.stringify(unknownError))
    })
  })

  describe('connectToCluster error handling', () => {
    it('should set connectionStatus to failed on initial connection failure', async () => {
      const errorMessage = 'No kubeconfig found'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      try {
        await clusterStore.connectToCluster()
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionStatus).toBe('failed')
      expect(clusterStore.connectionError).toContain(errorMessage)
      expect(clusterStore.k8sConnected).toBe(false)
    })

    it('should handle contexts retrieval failure gracefully', async () => {
      // Mock get_k8s_contexts to fail
      mockInvoke.mockRejectedValueOnce(new Error('kubeconfig not readable'))

      try {
        await clusterStore.connectToCluster()
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.connectionStatus).toBe('failed')
      expect(clusterStore.connectionError).toBeTruthy()
    })

    it('should fall back to default connection when no contexts available', async () => {
      // Mock get_k8s_contexts to return empty array
      mockInvoke.mockResolvedValueOnce([])
      // Mock successful default connection
      mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s
      mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces

      await clusterStore.connectToCluster()

      expect(clusterStore.connectionStatus).toBe('connected')
      expect(clusterStore.k8sConnected).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('connect_k8s')
    })
  })

  describe('contextErrors tracking', () => {
    it('should store context-specific errors', async () => {
      const errorMessage = 'context-specific error'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

      const mockContext = { name: 'failing-cluster', cluster: 'failing', user: 'user' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.contextErrors['failing-cluster']).toContain(errorMessage)
      expect(clusterStore.contextConnectionStatus['failing-cluster']).toBe('failed')
    })

    it('should clear context error on successful connection', async () => {
      // First, create an error state
      const errorMessage = 'previous error'
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage))
      
      const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }

      try {
        await clusterStore.selectContext(mockContext)
      } catch (error) {
        // Expected to throw
      }

      expect(clusterStore.contextErrors['test-cluster']).toBeTruthy()

      // Now succeed on retry
      mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
      mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces

      await clusterStore.selectContext(mockContext)

      expect(clusterStore.connectionError).toBe(null)
      expect(clusterStore.connectionStatus).toBe('connected')
      expect(clusterStore.contextConnectionStatus['test-cluster']).toBe('connected')
    })
  })
})