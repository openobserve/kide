import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useResourceStore } from '@/stores/resources'
import { useClusterStore } from '@/stores/cluster'
import type { K8sListItem, WatchEvent } from '@/types'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Resource Store Scaling Integration', () => {
  let resourceStore: ReturnType<typeof useResourceStore>
  let clusterStore: ReturnType<typeof useClusterStore>
  let mockInvoke: ReturnType<typeof vi.fn>

  beforeEach(() => {
    setActivePinia(createPinia())
    resourceStore = useResourceStore()
    clusterStore = useClusterStore()
    
    mockInvoke = vi.mocked(invoke)
    mockInvoke.mockClear()
    
    // Set up cluster store with test namespace
    clusterStore.selectedNamespaces = ['default']
    
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  const mockDeployment: K8sListItem = {
    metadata: {
      name: 'test-deployment',
      namespace: 'default',
      uid: 'deployment-123',
      creationTimestamp: '2025-08-05T04:51:15.136893673Z'
    },
    kind: 'Deployment',
    apiVersion: 'apps/v1',
    deploymentSpec: {
      replicas: 3
    },
    deploymentStatus: {
      readyReplicas: 3,
      availableReplicas: 3,
      updatedReplicas: 3
    }
  }

  describe('Watch Event Processing for Scaling', () => {
    it('should process Modified events with status updates', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      
      // Set up initial state
      resourceStore.resourceItems = [mockDeployment]
      
      // Create a modified deployment with updated status
      const scaledDeployment: K8sListItem = {
        ...mockDeployment,
        deploymentSpec: {
          replicas: 5
        },
        deploymentStatus: {
          readyReplicas: 5,
          availableReplicas: 5,
          updatedReplicas: 5
        }
      }
      
      // Process watch event - the store expects the event in this format
      resourceStore.processWatchEvent({ Modified: scaledDeployment } as any)
      
      // Allow batch processing
      vi.advanceTimersByTime(100)
      
      // Verify the deployment was updated
      expect(resourceStore.resourceItems).toHaveLength(1)
      expect(resourceStore.resourceItems[0].deploymentSpec?.replicas).toBe(5)
      expect(resourceStore.resourceItems[0].deploymentStatus?.readyReplicas).toBe(5)
      expect(resourceStore.resourceItems[0].deploymentStatus?.availableReplicas).toBe(5)
    })

    it('should handle multiple watch events for the same resource', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      resourceStore.resourceItems = [mockDeployment]
      
      // Process multiple events in quick succession
      const events = [
        // First update: spec change
        {
          Modified: {
            ...mockDeployment,
            deploymentSpec: { replicas: 4 },
            deploymentStatus: { readyReplicas: 3, availableReplicas: 3, updatedReplicas: 3 }
          }
        },
        // Second update: partial status change
        {
          Modified: {
            ...mockDeployment,
            deploymentSpec: { replicas: 4 },
            deploymentStatus: { readyReplicas: 4, availableReplicas: 3, updatedReplicas: 4 }
          }
        },
        // Third update: complete status change
        {
          Modified: {
            ...mockDeployment,
            deploymentSpec: { replicas: 4 },
            deploymentStatus: { readyReplicas: 4, availableReplicas: 4, updatedReplicas: 4 }
          }
        }
      ]
      
      events.forEach(event => resourceStore.processWatchEvent(event as any))
      
      // Allow batch processing
      vi.advanceTimersByTime(100)
      
      // Should have the final state
      expect(resourceStore.resourceItems).toHaveLength(1)
      expect(resourceStore.resourceItems[0].deploymentSpec?.replicas).toBe(4)
      expect(resourceStore.resourceItems[0].deploymentStatus?.readyReplicas).toBe(4)
      expect(resourceStore.resourceItems[0].deploymentStatus?.availableReplicas).toBe(4)
    })

    it('should handle watch events for different resources', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      
      const deployment1 = { ...mockDeployment, metadata: { ...mockDeployment.metadata, uid: 'deploy-1', name: 'app1' } }
      const deployment2 = { ...mockDeployment, metadata: { ...mockDeployment.metadata, uid: 'deploy-2', name: 'app2' } }
      
      resourceStore.resourceItems = [deployment1, deployment2]
      
      // Update only the first deployment
      const updatedDeployment1 = {
        ...deployment1,
        deploymentSpec: { replicas: 6 },
        deploymentStatus: { readyReplicas: 6, availableReplicas: 6, updatedReplicas: 6 }
      }
      
      resourceStore.processWatchEvent({ Modified: updatedDeployment1 } as any)
      vi.advanceTimersByTime(100)
      
      expect(resourceStore.resourceItems).toHaveLength(2)
      
      // First deployment should be updated
      const updatedItem1 = resourceStore.resourceItems.find(item => item.metadata?.uid === 'deploy-1')
      expect(updatedItem1?.deploymentSpec?.replicas).toBe(6)
      
      // Second deployment should remain unchanged
      const unchangedItem2 = resourceStore.resourceItems.find(item => item.metadata?.uid === 'deploy-2')
      expect(unchangedItem2?.deploymentSpec?.replicas).toBe(3)
    })
  })

  describe('Namespace Filtering During Scaling', () => {
    it('should only include items from selected namespaces', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      
      const deployment1 = { 
        ...mockDeployment, 
        metadata: { ...mockDeployment.metadata, uid: 'deploy-1', namespace: 'default' }
      }
      const deployment2 = { 
        ...mockDeployment, 
        metadata: { ...mockDeployment.metadata, uid: 'deploy-2', namespace: 'kube-system' }
      }
      
      resourceStore.resourceItems = [deployment1, deployment2]
      clusterStore.selectedNamespaces = ['default'] // Only default namespace selected
      
      // Update deployment in kube-system (not selected)
      const updatedDeployment2 = {
        ...deployment2,
        deploymentSpec: { replicas: 10 }
      }
      
      resourceStore.processWatchEvent({ Modified: updatedDeployment2 } as any)
      vi.advanceTimersByTime(100)
      
      // Should only have the default namespace deployment
      expect(resourceStore.resourceItems).toHaveLength(1)
      expect(resourceStore.resourceItems[0].metadata?.namespace).toBe('default')
    })

    it('should handle namespace changes during scaling operations', async () => {
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      
      resourceStore.selectedResource = mockResource
      resourceStore.resourceItems = [mockDeployment]
      
      // Mock both stop and start calls to succeed
      mockInvoke.mockResolvedValue(undefined)
      
      // Change to different namespace (this is debounced)
      resourceStore.changeNamespaces(['kube-system'])
      
      // Advance timers to let the debounced operation complete
      await vi.advanceTimersByTimeAsync(500)
      
      // Should have called stop and start watch with new namespace
      expect(mockInvoke).toHaveBeenCalledWith('stop_resource_watch', expect.objectContaining({
        resourceType: 'deployments',
        namespaces: ['kube-system']
      }))
      
      expect(mockInvoke).toHaveBeenCalledWith('start_resource_watch', expect.objectContaining({
        resourceType: 'deployments',
        namespaces: ['kube-system']
      }))
    })
  })

  describe('Event Batching During Scaling', () => {
    it('should batch multiple events within the window', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      
      resourceStore.resourceItems = [mockDeployment]
      
      // Send multiple events rapidly
      for (let i = 1; i <= 5; i++) {
        const modifiedDeployment = {
          ...mockDeployment,
          deploymentSpec: { replicas: i + 3 },
          deploymentStatus: { readyReplicas: i + 2, availableReplicas: i + 2, updatedReplicas: i + 2 }
        }
        resourceStore.processWatchEvent({ Modified: modifiedDeployment } as any)
      }
      
      // Advance past batch window
      vi.advanceTimersByTime(100)
      
      // Should have the final state
      expect(resourceStore.resourceItems[0].deploymentSpec?.replicas).toBe(8) // Last update: 5 + 3
    })

    it('should not process events during namespace changes', () => {
      // Set up a resource to be selected (required for event processing)
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.selectedResource = mockResource
      
      resourceStore.resourceItems = [mockDeployment]
      resourceStore.isChangingNamespaces = true
      
      const modifiedDeployment = {
        ...mockDeployment,
        deploymentSpec: { replicas: 10 }
      }
      
      resourceStore.processWatchEvent({ Modified: modifiedDeployment } as any)
      vi.advanceTimersByTime(100)
      
      // Should not have updated the item
      expect(resourceStore.resourceItems[0].deploymentSpec?.replicas).toBe(3)
    })
  })

  describe('Resource Store State Management', () => {
    it('should reset state correctly for cluster changes', async () => {
      // Set up some state
      resourceStore.selectedResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      resourceStore.resourceItems = [mockDeployment]
      resourceStore.loading = true
      
      mockInvoke.mockResolvedValue(undefined)
      
      await resourceStore.resetForClusterChange()
      
      // State should be reset
      expect(resourceStore.selectedResource).toBeNull()
      expect(resourceStore.resourceItems).toEqual([])
      expect(resourceStore.loading).toBe(false)
      expect(resourceStore.isChangingNamespaces).toBe(false)
    })

    it('should handle concurrent scaling operations properly', async () => {
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      
      resourceStore.selectedResource = mockResource
      mockInvoke.mockResolvedValue(undefined)
      
      // Start multiple namespace changes concurrently
      const promises = [
        resourceStore.changeNamespaces(['ns1']),
        resourceStore.changeNamespaces(['ns2']),
        resourceStore.changeNamespaces(['ns3'])
      ]
      
      // Wait for all promises and advance timers
      await Promise.all(promises)
      await vi.advanceTimersByTimeAsync(1000) // More time for all operations
      
      // Should have handled the operations correctly
      expect(resourceStore.isChangingNamespaces).toBe(false)
      expect(resourceStore.loading).toBe(false)
    })
  })

  describe('Error Handling in Store', () => {
    it('should handle watch restart failures gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      
      const mockResource = {
        name: 'Deployments',
        namespaced: true,
        kind: 'Deployment',
        apiVersion: 'apps/v1',
        description: 'Test'
      }
      
      resourceStore.selectedResource = mockResource
      mockInvoke.mockRejectedValue(new Error('Watch restart failed'))
      
      // Don't await, just start the operation
      resourceStore.changeNamespaces(['test'])
      
      // Advance timers asynchronously
      await vi.advanceTimersByTimeAsync(500)
      
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to restart watch for namespace change'),
        expect.any(Error)
      )
      
      consoleSpy.mockRestore()
    })
  })
})