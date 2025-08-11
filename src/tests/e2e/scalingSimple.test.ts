import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import WorkloadConfiguration from '@/components/ResourcePanel/WorkloadConfiguration.vue'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Simple Scaling E2E Tests', () => {
  let wrapper: VueWrapper<any>
  let mockInvoke: ReturnType<typeof vi.fn>
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    
    mockInvoke = vi.mocked(invoke)
    mockInvoke.mockClear()
    mockInvoke.mockResolvedValue(undefined)
    
    vi.useRealTimers()
    vi.useFakeTimers()
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  const createScalingWrapper = (props = {}) => {
    const defaultProps = {
      resourceKind: 'Deployment',
      resourceName: 'test-app',
      namespace: 'default',
      spec: {
        replicas: 3,
        selector: {
          matchLabels: { app: 'test' }
        }
      },
      status: {
        readyReplicas: 3,
        availableReplicas: 3,
        updatedReplicas: 3
      }
    }

    return mount(WorkloadConfiguration, {
      props: { ...defaultProps, ...props },
      global: {
        plugins: [pinia]
      }
    })
  }

  describe('Complete Scaling Flow', () => {
    it('should handle complete scaling workflow with status updates', async () => {
      wrapper = createScalingWrapper()
      
      // Verify initial state
      expect(wrapper.text()).toContain('3') // Initial replica count
      
      // Find and click scale up button
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      expect(scaleUpButton.exists()).toBe(true)
      
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Verify scaling started
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-app',
        resourceKind: 'Deployment',
        namespace: 'default',
        replicas: 4
      })
      
      expect(wrapper.text()).toContain('Scaling...')
      expect(wrapper.text()).toContain('4') // Updated desired replicas
      
      // Simulate backend status update
      await wrapper.setProps({
        status: {
          readyReplicas: 4,
          availableReplicas: 4,
          updatedReplicas: 4
        }
      })
      await nextTick()
      
      // Scaling should complete automatically
      expect(wrapper.text()).not.toContain('Scaling...')
      
      // Verify scaled event was emitted
      expect(wrapper.emitted('scaled')).toBeTruthy()
      expect(wrapper.emitted('scaled')?.[0]).toEqual([4])
    })

    it('should handle scale down to zero', async () => {
      wrapper = createScalingWrapper({
        spec: { replicas: 1 },
        status: { readyReplicas: 1, availableReplicas: 1, updatedReplicas: 1 }
      })
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      await scaleDownButton.trigger('click')
      await nextTick()
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-app',
        resourceKind: 'Deployment',
        namespace: 'default',
        replicas: 0
      })
      
      // Simulate status update to zero
      await wrapper.setProps({
        spec: { replicas: 0 },
        status: { readyReplicas: 0, availableReplicas: 0, updatedReplicas: 0 }
      })
      await nextTick()
      
      expect(wrapper.text()).toContain('0')
      expect(wrapper.text()).not.toContain('Scaling...')
    })

    it('should handle rapid scaling operations', async () => {
      wrapper = createScalingWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      
      // First scaling operation
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Try to scale again (should be prevented)
      mockInvoke.mockClear()
      await scaleUpButton.trigger('click')
      
      // Second call should not have been made
      expect(mockInvoke).not.toHaveBeenCalled()
      
      // Buttons should be disabled
      expect(scaleUpButton.attributes('disabled')).toBeDefined()
      expect(wrapper.find('button[title="Scale down"]').attributes('disabled')).toBeDefined()
    })

    it('should show optimistic updates during scaling', async () => {
      wrapper = createScalingWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Fast forward to trigger optimistic updates
      vi.advanceTimersByTime(2000)
      await nextTick()
      
      // Should show progress toward target
      const readyReplicasSection = wrapper.html()
      expect(readyReplicasSection).toContain('→ 4')
    })

    it('should handle different resource types', async () => {
      const resourceTypes = ['Deployment', 'StatefulSet', 'ReplicaSet']
      
      for (const resourceKind of resourceTypes) {
        wrapper?.unmount()
        wrapper = createScalingWrapper({ resourceKind })
        
        const scaleUpButton = wrapper.find('button[title="Scale up"]')
        expect(scaleUpButton.exists()).toBe(true)
        
        await scaleUpButton.trigger('click')
        
        expect(mockInvoke).toHaveBeenCalledWith('scale_resource', 
          expect.objectContaining({
            resourceKind,
            replicas: 4
          })
        )
        
        mockInvoke.mockClear()
      }
    })

    it('should not show scaling for DaemonSets', async () => {
      wrapper = createScalingWrapper({ resourceKind: 'DaemonSet' })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      
      expect(scaleUpButton.exists()).toBe(false)
      expect(scaleDownButton.exists()).toBe(false)
    })

    it('should handle timeout gracefully', async () => {
      wrapper = createScalingWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Fast forward past timeout
      vi.advanceTimersByTime(15000)
      await nextTick()
      
      expect(wrapper.text()).not.toContain('Scaling...')
    })

    it('should handle API errors during scaling', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
      
      mockInvoke.mockRejectedValue(new Error('Network error'))
      
      wrapper = createScalingWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to scale'),
        expect.any(Error)
      )
      expect(alertSpy).toHaveBeenCalled()
      expect(wrapper.text()).not.toContain('Scaling...')
      
      consoleSpy.mockRestore()
      alertSpy.mockRestore()
    })

    it('should emit scaled events correctly', async () => {
      wrapper = createScalingWrapper()
      
      // Scale up
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.emitted('scaled')?.[0]).toEqual([4])
      
      // Complete scaling to allow scale down
      await wrapper.setProps({
        spec: { replicas: 4 },
        status: { readyReplicas: 4, availableReplicas: 4, updatedReplicas: 4 }
      })
      await nextTick()
      
      // Scale down
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      await scaleDownButton.trigger('click')
      await nextTick()
      
      expect(wrapper.emitted('scaled')?.[1]).toEqual([3])
    })

    it('should handle namespace-less resources', async () => {
      wrapper = createScalingWrapper({ namespace: undefined })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-app',
        resourceKind: 'Deployment',
        namespace: undefined,
        replicas: 4
      })
    })
  })

  describe('Status Field Updates', () => {
    it('should update all status fields during scaling', async () => {
      wrapper = createScalingWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Verify initial status is shown
      expect(wrapper.text()).toContain('Ready Replicas')
      expect(wrapper.text()).toContain('Available Replicas')
      
      // Simulate gradual status updates
      await wrapper.setProps({
        status: {
          readyReplicas: 3,
          availableReplicas: 3,
          updatedReplicas: 4 // Updated first
        }
      })
      await nextTick()
      
      await wrapper.setProps({
        status: {
          readyReplicas: 4,
          availableReplicas: 3,
          updatedReplicas: 4 // Ready next
        }
      })
      await nextTick()
      
      await wrapper.setProps({
        status: {
          readyReplicas: 4,
          availableReplicas: 4,
          updatedReplicas: 4 // All complete
        }
      })
      await nextTick()
      
      // Should complete scaling
      expect(wrapper.text()).not.toContain('Scaling...')
    })
  })
})