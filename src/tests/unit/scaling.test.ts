import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import WorkloadConfiguration from '@/components/ResourcePanel/WorkloadConfiguration.vue'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('WorkloadConfiguration Scaling', () => {
  let wrapper: VueWrapper<any>
  let mockInvoke: ReturnType<typeof vi.fn>

  beforeEach(() => {
    mockInvoke = vi.mocked(invoke)
    mockInvoke.mockClear()
    
    // Mock successful scaling by default
    mockInvoke.mockResolvedValue(undefined)
    
    // Set up fake timers properly
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

  const createWrapper = (props = {}) => {
    const defaultProps = {
      resourceKind: 'Deployment',
      resourceName: 'test-deployment',
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
      props: { ...defaultProps, ...props }
    })
  }

  describe('Scaling Controls', () => {
    it('should render scaling controls for scalable resources', () => {
      wrapper = createWrapper()
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      
      expect(scaleDownButton.exists()).toBe(true)
      expect(scaleUpButton.exists()).toBe(true)
      expect(scaleDownButton.text()).toBe('−')
      expect(scaleUpButton.text()).toBe('+')
    })

    it('should not render scaling controls for non-scalable resources', () => {
      wrapper = createWrapper({ resourceKind: 'DaemonSet' })
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      
      expect(scaleDownButton.exists()).toBe(false)
      expect(scaleUpButton.exists()).toBe(false)
    })

    it('should display current replica count', () => {
      wrapper = createWrapper({ spec: { replicas: 5 } })
      
      const replicasText = wrapper.find('dd').text()
      expect(replicasText).toContain('5')
    })
  })

  describe('Scale Up Operation', () => {
    it('should call scale_resource command when scaling up', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-deployment',
        resourceKind: 'Deployment',
        namespace: 'default',
        replicas: 4
      })
    })

    it('should emit scaled event when scaling up succeeds', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.emitted('scaled')).toBeTruthy()
      expect(wrapper.emitted('scaled')?.[0]).toEqual([4])
    })

    it('should show scaling indicator when scaling up', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
    })

    it('should disable buttons during scaling', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(scaleUpButton.attributes('disabled')).toBeDefined()
      expect(scaleDownButton.attributes('disabled')).toBeDefined()
    })
  })

  describe('Scale Down Operation', () => {
    it('should call scale_resource command when scaling down', async () => {
      wrapper = createWrapper()
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      await scaleDownButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-deployment',
        resourceKind: 'Deployment',
        namespace: 'default',
        replicas: 2
      })
    })

    it('should not allow scaling below 0', async () => {
      wrapper = createWrapper({ spec: { replicas: 0 } })
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      expect(scaleDownButton.attributes('disabled')).toBeDefined()
      
      await scaleDownButton.trigger('click')
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should emit scaled event when scaling down succeeds', async () => {
      wrapper = createWrapper()
      
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      await scaleDownButton.trigger('click')
      await nextTick()
      
      expect(wrapper.emitted('scaled')).toBeTruthy()
      expect(wrapper.emitted('scaled')?.[0]).toEqual([2])
    })
  })

  describe('Status Updates', () => {
    it('should detect scaling completion when status matches target', async () => {
      wrapper = createWrapper({
        spec: { replicas: 3 },
        status: { readyReplicas: 3, availableReplicas: 3 }
      })
      
      // Start scaling
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Simulate status update to match target
      await wrapper.setProps({
        status: { readyReplicas: 4, availableReplicas: 4 }
      })
      await nextTick()
      
      expect(wrapper.text()).not.toContain('Scaling...')
    })

    it('should show optimistic ready replicas during scaling', async () => {
      wrapper = createWrapper({
        spec: { replicas: 3 },
        status: { readyReplicas: 3, availableReplicas: 3 }
      })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Fast forward time to trigger optimistic updates
      vi.advanceTimersByTime(2000)
      await nextTick()
      
      // Should show optimistic progress toward target
      const readyReplicasText = wrapper.text()
      expect(readyReplicasText).toContain('→ 4')
    })

    it('should timeout after 15 seconds if status never updates', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Fast forward 15 seconds
      vi.advanceTimersByTime(15000)
      await nextTick()
      
      expect(wrapper.text()).not.toContain('Scaling...')
    })
  })

  describe('Error Handling', () => {
    it('should handle scaling failures gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
      
      mockInvoke.mockRejectedValue(new Error('Scaling failed'))
      
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to scale Deployment'),
        expect.any(Error)
      )
      expect(alertSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to scale Deployment')
      )
      expect(wrapper.text()).not.toContain('Scaling...')
      
      consoleSpy.mockRestore()
      alertSpy.mockRestore()
    })

    it('should not scale when already scaling', async () => {
      wrapper = createWrapper()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      
      // First click
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Second click should be ignored
      mockInvoke.mockClear()
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).not.toHaveBeenCalled()
    })
  })

  describe('StatefulSet Scaling', () => {
    it('should support StatefulSet scaling', async () => {
      wrapper = createWrapper({ resourceKind: 'StatefulSet' })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-deployment',
        resourceKind: 'StatefulSet',
        namespace: 'default',
        replicas: 4
      })
    })
  })

  describe('ReplicaSet Scaling', () => {
    it('should support ReplicaSet scaling', async () => {
      wrapper = createWrapper({ resourceKind: 'ReplicaSet' })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-deployment',
        resourceKind: 'ReplicaSet',
        namespace: 'default',
        replicas: 4
      })
    })
  })

  describe('Namespace-less Resources', () => {
    it('should handle cluster-wide resources without namespace', async () => {
      wrapper = createWrapper({ 
        namespace: undefined,
        resourceName: 'cluster-deployment'
      })
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'cluster-deployment',
        resourceKind: 'Deployment',
        namespace: undefined,
        replicas: 4
      })
    })
  })
})