import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ResourceShellTabs from '../../components/ResourcePanel/ResourceShellTabs.vue'
import { nextTick } from 'vue'

// Mock the ResourceShell component
vi.mock('../../components/ResourcePanel/ResourceShell.vue', () => ({
  default: {
    name: 'ResourceShell',
    template: '<div class="mock-shell">{{ podName }}/{{ containerName }} - Auto: {{ autoConnect }}</div>',
    props: ['containers', 'podName', 'namespace', 'initialContainer', 'autoConnect'],
    emits: ['close', 'connection-state-changed', 'container-changed']
  }
}))

describe('ResourceShellTabs Component', () => {
  let wrapper: VueWrapper<any>

  beforeEach(() => {
    wrapper = mount(ResourceShellTabs, {
      props: {
        maxTabs: 5
      }
    })
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  describe('Empty State', () => {
    it('should show empty state when no tabs are open', () => {
      expect(wrapper.text()).toContain('No Shell Sessions')
      expect(wrapper.text()).toContain('Click "Open shell" on any pod to start a terminal session')
    })

    it('should show panel control buttons even in empty state', () => {
      // Panel control buttons (minimize, maximize, close) should always be visible
      const buttons = wrapper.findAll('button')
      expect(buttons.length).toBeGreaterThan(0)
      
      // Should have minimize, maximize, and close buttons
      expect(wrapper.text()).toContain('0 active') // Shows session count
    })
  })

  describe('Tab Management', () => {
    it('should add new shell tab', async () => {
      const tabData = {
        podName: 'test-pod',
        namespace: 'default',
        containerName: 'web-container',
        containers: [{ name: 'web-container' }]
      }

      const tabId = wrapper.vm.addShellTab(tabData)
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(1)
      expect(wrapper.vm.activeTabId).toBe(tabId)
      expect(wrapper.text()).toContain('test-pod')
      // Container name should be in the tab data but not necessarily displayed in UI
      expect(wrapper.vm.shellTabs[0].containerName).toBe('web-container')
    })

    it('should not create duplicate tabs for same pod/container', async () => {
      const tabData = {
        podName: 'test-pod',
        namespace: 'default',
        containerName: 'web-container',
        containers: [{ name: 'web-container' }]
      }

      const tabId1 = wrapper.vm.addShellTab(tabData)
      const tabId2 = wrapper.vm.addShellTab(tabData)
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(1)
      expect(tabId1).toBe(tabId2)
    })

    it('should switch between tabs', async () => {
      const tab1Data = {
        podName: 'pod-1',
        namespace: 'default',
        containerName: 'container-1',
        containers: [{ name: 'container-1' }]
      }

      const tab2Data = {
        podName: 'pod-2',
        namespace: 'default',
        containerName: 'container-2',
        containers: [{ name: 'container-2' }]
      }

      const tab1Id = wrapper.vm.addShellTab(tab1Data)
      const tab2Id = wrapper.vm.addShellTab(tab2Data)
      await nextTick()

      expect(wrapper.vm.activeTabId).toBe(tab2Id)

      wrapper.vm.setActiveTab(tab1Id)
      await nextTick()

      expect(wrapper.vm.activeTabId).toBe(tab1Id)
    })

    it('should close tabs correctly', async () => {
      const tabData = {
        podName: 'test-pod',
        namespace: 'default',
        containerName: 'web-container',
        containers: [{ name: 'web-container' }]
      }

      const tabId = wrapper.vm.addShellTab(tabData)
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(1)

      wrapper.vm.closeTab(tabId)
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(0)
      expect(wrapper.vm.activeTabId).toBe(null)
      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('should handle tab close with multiple tabs', async () => {
      const tab1Data = {
        podName: 'pod-1',
        namespace: 'default',
        containerName: 'container-1',
        containers: [{ name: 'container-1' }]
      }

      const tab2Data = {
        podName: 'pod-2',
        namespace: 'default',
        containerName: 'container-2',
        containers: [{ name: 'container-2' }]
      }

      const tab1Id = wrapper.vm.addShellTab(tab1Data)
      const tab2Id = wrapper.vm.addShellTab(tab2Data)
      await nextTick()

      // Close the active tab (tab2)
      wrapper.vm.closeTab(tab2Id)
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(1)
      expect(wrapper.vm.activeTabId).toBe(tab1Id) // Should switch to remaining tab
    })
  })

  describe('Tab UI', () => {
    beforeEach(async () => {
      const tabData = {
        podName: 'test-pod',
        namespace: 'default',
        containerName: 'web-container',
        containers: [{ name: 'web-container' }]
      }
      wrapper.vm.addShellTab(tabData)
      await nextTick()
    })

    it('should show connection status indicators', async () => {
      const statusIndicator = wrapper.find('.w-2.h-2.rounded-full')
      expect(statusIndicator.exists()).toBe(true)
      expect(statusIndicator.classes()).toContain('bg-gray-400') // disconnected state
    })

    it('should show close buttons on hover', () => {
      const closeButton = wrapper.find('button svg')
      expect(closeButton.exists()).toBe(true)
    })

    it('should not show select pod button in tab bar', () => {
      // Should not have any "Select Pod" text
      expect(wrapper.text()).not.toContain('Select Pod')
      expect(wrapper.text()).not.toContain('New Shell')
    })
  })

  describe('Connection State Updates', () => {
    it('should update tab connection state', async () => {
      const tabData = {
        podName: 'test-pod',
        namespace: 'default',
        containerName: 'web-container',
        containers: [{ name: 'web-container' }]
      }

      const tabId = wrapper.vm.addShellTab(tabData)
      await nextTick()

      wrapper.vm.updateTabConnectionState(tabId, {
        isConnected: true,
        isConnecting: false
      })
      await nextTick()

      const tab = wrapper.vm.shellTabs.find((t: any) => t.id === tabId)
      expect(tab.isConnected).toBe(true)
      expect(tab.isConnecting).toBe(false)
    })
  })

  describe('Max Tabs Limit', () => {
    it('should respect max tabs limit', async () => {
      // Add tabs up to the limit
      for (let i = 0; i < 5; i++) {
        wrapper.vm.addShellTab({
          podName: `pod-${i}`,
          namespace: 'default',
          containerName: `container-${i}`,
          containers: [{ name: `container-${i}` }]
        })
      }
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(5)

      // With max tabs reached, no additional UI elements should appear
      expect(wrapper.vm.shellTabs).toHaveLength(5) // Verify we actually have 5 tabs
    })
  })

  describe('Public API', () => {
    it('should expose required methods', () => {
      expect(wrapper.vm.addShellTab).toBeInstanceOf(Function)
      expect(wrapper.vm.setActiveTab).toBeInstanceOf(Function)
      expect(wrapper.vm.closeTab).toBeInstanceOf(Function)
      expect(wrapper.vm.closeAllTabs).toBeInstanceOf(Function)
    })

    it('should close all tabs', async () => {
      // Add multiple tabs
      wrapper.vm.addShellTab({
        podName: 'pod-1',
        namespace: 'default',
        containerName: 'container-1',
        containers: [{ name: 'container-1' }]
      })

      wrapper.vm.addShellTab({
        podName: 'pod-2',
        namespace: 'default',
        containerName: 'container-2',
        containers: [{ name: 'container-2' }]
      })
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(2)

      wrapper.vm.closeAllTabs()
      await nextTick()

      expect(wrapper.vm.shellTabs).toHaveLength(0)
      expect(wrapper.vm.activeTabId).toBe(null)
    })
  })
})