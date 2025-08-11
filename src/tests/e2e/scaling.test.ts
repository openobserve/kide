import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import ResourceList from '@/components/ResourceList.vue'
import ResourcePanel from '@/components/ResourcePanel/index.vue'
import type { K8sListItem, K8sResource } from '@/types'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock Tauri event API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

describe('Scaling E2E Tests', () => {
  let wrapper: VueWrapper<any>
  let mockInvoke: ReturnType<typeof vi.fn>
  let pinia: ReturnType<typeof createPinia>

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    
    mockInvoke = vi.mocked(invoke)
    mockInvoke.mockClear()
    
    // Mock successful scaling by default
    mockInvoke.mockResolvedValue(undefined)
    
    vi.useFakeTimers()
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  const mockDeployment: K8sListItem = {
    metadata: {
      name: 'test-deployment',
      namespace: 'default',
      uid: 'deployment-123',
      creationTimestamp: '2025-08-05T04:51:15.136893673Z',
      labels: { app: 'test' }
    },
    kind: 'Deployment',
    apiVersion: 'apps/v1',
    deploymentSpec: {
      replicas: 3,
      selector: {
        matchLabels: { app: 'test' }
      }
    },
    deploymentStatus: {
      readyReplicas: 3,
      availableReplicas: 3,
      updatedReplicas: 3
    }
  }

  const mockDeploymentResource: K8sResource = {
    name: 'Deployments',
    apiVersion: 'apps/v1',
    kind: 'Deployment',
    namespaced: true,
    description: 'Manage deployment of applications'
  }

  const createResourceListWrapper = (items: K8sListItem[] = [mockDeployment]) => {
    return mount(ResourceList, {
      props: {
        resource: mockDeploymentResource,
        items,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default']
      },
      global: {
        plugins: [pinia]
      }
    })
  }

  describe('Full Scaling Workflow', () => {
    it('should complete full scaling workflow from resource list to panel', async () => {
      wrapper = createResourceListWrapper()
      
      // Click on the deployment to open the resource panel
      // First try to find the resource by name using our data attribute
      const deploymentNameCell = wrapper.find('[data-resource-name="test-deployment"]')
      
      if (deploymentNameCell.exists()) {
        await deploymentNameCell.trigger('click')
      } else {
        // Fallback to row selection
        const deploymentRow = wrapper.find('tr[data-testid="resource-row"]')
        expect(deploymentRow.exists()).toBe(true)
        await deploymentRow.trigger('click')
      }
      
      await nextTick()
      
      // Verify resource panel opens
      const resourcePanel = wrapper.findComponent(ResourcePanel)
      expect(resourcePanel.exists()).toBe(true)
      
      // Navigate to Overview tab (should be default)
      const overviewTab = wrapper.find('[data-tab="overview"]') || 
                          wrapper.find('button:contains("Overview")')
      
      if (overviewTab.exists()) {
        await overviewTab.trigger('click')
        await nextTick()
      }
      
      // Find scaling controls
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      
      expect(scaleUpButton.exists()).toBe(true)
      expect(scaleDownButton.exists()).toBe(true)
      
      // Perform scale up operation
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Verify scaling command was called
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-deployment',
        resourceKind: 'Deployment',
        namespace: 'default',
        replicas: 4
      })
      
      // Verify scaling indicator appears
      expect(wrapper.text()).toContain('Scaling...')
    })

    it('should handle status updates during scaling', async () => {
      const items = [mockDeployment]
      wrapper = createResourceListWrapper(items)
      
      // Open resource panel
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      // Start scaling
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Simulate backend status update by updating the props
      // In a real scenario, this would come from watch events
      const updatedDeployment = {
        ...mockDeployment,
        deploymentStatus: {
          readyReplicas: 4,
          availableReplicas: 4,
          updatedReplicas: 4
        }
      }
      
      await wrapper.setProps({
        items: [updatedDeployment]
      })
      await nextTick()
      
      // Scaling should complete automatically
      expect(wrapper.text()).not.toContain('Scaling...')
      
      // Status fields should show updated values
      expect(wrapper.text()).toContain('4')
    })

    it('should handle scaling timeout', async () => {
      wrapper = createResourceListWrapper()
      
      // Open resource panel and start scaling
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      expect(wrapper.text()).toContain('Scaling...')
      
      // Fast forward past the timeout (15 seconds)
      vi.advanceTimersByTime(15000)
      await nextTick()
      
      // Scaling indicator should disappear
      expect(wrapper.text()).not.toContain('Scaling...')
    })
  })

  describe('Multiple Resource Types', () => {
    it('should support StatefulSet scaling', async () => {
      const statefulSet: K8sListItem = {
        ...mockDeployment,
        kind: 'StatefulSet',
        metadata: {
          ...mockDeployment.metadata,
          name: 'test-statefulset'
        },
        // Remove deployment fields and add StatefulSet fields
        deploymentSpec: undefined,
        deploymentStatus: undefined,
        statefulSetSpec: {
          replicas: 3,
          selector: {
            matchLabels: { app: 'test' }
          }
        },
        statefulSetStatus: {
          replicas: 3,
          readyReplicas: 3,
          availableReplicas: 3,
          updatedReplicas: 3
        }
      }
      
      const statefulSetResource: K8sResource = {
        ...mockDeploymentResource,
        name: 'StatefulSets',
        kind: 'StatefulSet'
      }
      
      wrapper = mount(ResourceList, {
        props: {
          resource: statefulSetResource,
          items: [statefulSet],
          loading: false,
          namespaces: ['default'],
          selectedNamespaces: ['default']
        },
        global: {
          plugins: [pinia]
        }
      })
      
      // Open resource panel and scale
      const statefulSetCell = wrapper.find('[data-resource-name="test-statefulset"]')
      expect(statefulSetCell.exists()).toBe(true)
      
      await statefulSetCell.trigger('click')
      await nextTick()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      
      expect(mockInvoke).toHaveBeenCalledWith('scale_resource', {
        resourceName: 'test-statefulset',
        resourceKind: 'StatefulSet',
        namespace: 'default',
        replicas: 4
      })
    })

    it('should not show scaling controls for DaemonSets', async () => {
      const daemonSet: K8sListItem = {
        ...mockDeployment,
        kind: 'DaemonSet',
        metadata: {
          ...mockDeployment.metadata,
          name: 'test-daemonset'
        }
      }
      
      const daemonSetResource: K8sResource = {
        ...mockDeploymentResource,
        name: 'DaemonSets',
        kind: 'DaemonSet'
      }
      
      wrapper = mount(ResourceList, {
        props: {
          resource: daemonSetResource,
          items: [daemonSet],
          loading: false,
          namespaces: ['default'],
          selectedNamespaces: ['default']
        },
        global: {
          plugins: [pinia]
        }
      })
      
      // Open resource panel
      const daemonSetCell = wrapper.find('[data-resource-name="test-daemonset"]')
      expect(daemonSetCell.exists()).toBe(true)
      
      await daemonSetCell.trigger('click')
      await nextTick()
      
      // Should not have scaling controls
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      
      expect(scaleUpButton.exists()).toBe(false)
      expect(scaleDownButton.exists()).toBe(false)
    })
  })

  describe('Error Scenarios', () => {
    it('should handle scaling API errors', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})
      
      mockInvoke.mockRejectedValue(new Error('API Error: Insufficient permissions'))
      
      wrapper = createResourceListWrapper()
      
      // Open resource panel and attempt scaling
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Should show error and clear scaling state
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to scale'),
        expect.any(Error)
      )
      expect(alertSpy).toHaveBeenCalledWith(
        expect.stringContaining('API Error: Insufficient permissions')
      )
      expect(wrapper.text()).not.toContain('Scaling...')
      
      consoleSpy.mockRestore()
      alertSpy.mockRestore()
    })

    it('should prevent scaling below 0', async () => {
      const deploymentWith0Replicas: K8sListItem = {
        ...mockDeployment,
        deploymentSpec: {
          ...mockDeployment.deploymentSpec,
          replicas: 0
        },
        deploymentStatus: {
          readyReplicas: 0,
          availableReplicas: 0,
          updatedReplicas: 0
        }
      }
      
      wrapper = createResourceListWrapper([deploymentWith0Replicas])
      
      // Open resource panel
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      // Scale down button should be disabled
      const scaleDownButton = wrapper.find('button[title="Scale down"]')
      expect(scaleDownButton.attributes('disabled')).toBeDefined()
      
      // Clicking should not trigger scaling API call
      await scaleDownButton.trigger('click')
      expect(mockInvoke).not.toHaveBeenCalledWith('scale_resource', expect.anything())
    })
  })

  describe('Optimistic UI Updates', () => {
    it('should show optimistic replica counts during scaling', async () => {
      wrapper = createResourceListWrapper()
      
      // Open resource panel
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      // Start scaling
      const scaleUpButton = wrapper.find('button[title="Scale up"]')
      await scaleUpButton.trigger('click')
      await nextTick()
      
      // Desired replicas should update immediately
      expect(wrapper.text()).toContain('4')
      expect(wrapper.text()).toContain('Scaling...')
      
      // Fast forward to see optimistic updates
      vi.advanceTimersByTime(2000)
      await nextTick()
      
      // Should show progress indicators
      expect(wrapper.text()).toContain('→ 4')
    })
  })

  describe('Resource Panel Data Updates', () => {
    it('should update panel data when store is updated', async () => {
      const initialItems = [mockDeployment]
      wrapper = createResourceListWrapper(initialItems)
      
      // Open resource panel
      const deploymentCell = wrapper.find('[data-resource-name="test-deployment"]')
      expect(deploymentCell.exists()).toBe(true)
      
      await deploymentCell.trigger('click')
      await nextTick()
      
      // Verify initial status
      expect(wrapper.text()).toContain('3')
      
      // Simulate store update (as would happen from watch events)
      const updatedDeployment: K8sListItem = {
        ...mockDeployment,
        deploymentSpec: {
          ...mockDeployment.deploymentSpec,
          replicas: 5
        },
        deploymentStatus: {
          readyReplicas: 5,
          availableReplicas: 5,
          updatedReplicas: 5
        }
      }
      
      await wrapper.setProps({
        items: [updatedDeployment]
      })
      await nextTick()
      
      // Panel should show updated values
      expect(wrapper.text()).toContain('5')
    })
  })
})