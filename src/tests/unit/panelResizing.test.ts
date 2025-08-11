import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import BottomPanel from '../../components/BottomPanel.vue'
import { nextTick } from 'vue'

// Mock the UnifiedTerminalTabs component
vi.mock('../../components/ResourcePanel/UnifiedTerminalTabs.vue', () => ({
  default: {
    name: 'UnifiedTerminalTabs',
    template: '<div class="mock-unified-terminal-tabs">Mock Terminal Tabs</div>',
    props: ['maxTabs', 'isMaximized', 'isResizing'],
    emits: ['close', 'minimize', 'toggle-maximize', 'start-resize', 'refresh-logs', 'toggle-live-logging'],
    methods: {
      addLogTab: vi.fn(),
      addShellTab: vi.fn(),
      closeAllTabs: vi.fn()
    }
  }
}))

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Panel Resizing and Maximization', () => {
  let wrapper: VueWrapper<any>
  
  const mockPod = {
    metadata: {
      uid: 'test-pod-1',
      name: 'test-pod',
      namespace: 'default'
    },
    spec: {
      containers: [{ name: 'main' }]
    }
  }

  beforeEach(() => {
    // Clear localStorage
    localStorage.clear()
    
    wrapper = mount(BottomPanel, {
      props: {
        isOpen: true
      }
    })
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    localStorage.clear()
  })

  describe('Panel Height Management', () => {
    it('should initialize with default panel height', () => {
      expect(wrapper.vm.panelHeight).toBe(400)
    })

    it('should save panel height to localStorage', async () => {
      wrapper.vm.panelHeight = 500
      wrapper.vm.savePanelPreferences()
      
      expect(localStorage.getItem('kide-panel-height')).toBe('500')
    })

    it('should load panel height from localStorage', async () => {
      localStorage.setItem('kide-panel-height', '600')
      
      wrapper.vm.loadPanelPreferences()
      
      expect(wrapper.vm.panelHeight).toBe(600)
    })

    it('should not load invalid panel height from localStorage', async () => {
      localStorage.setItem('kide-panel-height', '50') // Below minimum
      const initialHeight = wrapper.vm.panelHeight
      
      wrapper.vm.loadPanelPreferences()
      
      expect(wrapper.vm.panelHeight).toBe(initialHeight) // Should remain unchanged
    })
  })

  describe('Maximize/Minimize Functionality', () => {
    it('should toggle maximize state', async () => {
      expect(wrapper.vm.isMaximized).toBe(false)
      
      wrapper.vm.handleToggleMaximize()
      await nextTick()
      
      expect(wrapper.vm.isMaximized).toBe(true)
    })

    it('should save maximized state to localStorage', async () => {
      wrapper.vm.handleToggleMaximize()
      
      expect(localStorage.getItem('kide-panel-maximized')).toBe('true')
    })

    it('should load maximized state from localStorage', async () => {
      localStorage.setItem('kide-panel-maximized', 'true')
      
      wrapper.vm.loadPanelPreferences()
      
      expect(wrapper.vm.isMaximized).toBe(true)
    })

    it('should handle close event', async () => {
      const closeSpy = vi.fn()
      wrapper.vm.$emit = closeSpy
      
      wrapper.vm.handleClose()
      await nextTick()
      
      // Verify that close method calls emit
      expect(wrapper.emitted('update:isOpen')).toBeTruthy()
      expect(wrapper.emitted('close')).toBeTruthy()
    })

    it('should handle minimize event', async () => {
      wrapper.vm.handleMinimize()
      await nextTick()
      
      expect(wrapper.emitted('update:isOpen')).toBeTruthy()
    })
  })

  describe('Panel UI', () => {
    it('should render panel when open', async () => {
      await nextTick()
      
      const panel = wrapper.find('.flex-none.bg-white')
      expect(panel.exists()).toBe(true)
    })

    it('should not render panel when closed', async () => {
      await wrapper.setProps({ isOpen: false })
      await nextTick()
      
      const panel = wrapper.find('.flex-none.bg-white')
      expect(panel.exists()).toBe(false)
    })

    it('should handle maximize functionality', async () => {
      wrapper.vm.isMaximized = false
      await nextTick()
      
      expect(wrapper.vm.isMaximized).toBe(false)
      wrapper.vm.handleToggleMaximize()
      expect(wrapper.vm.isMaximized).toBe(true)
    })

    it('should handle restore functionality when maximized', async () => {
      wrapper.vm.isMaximized = true
      await nextTick()
      
      expect(wrapper.vm.isMaximized).toBe(true)
      wrapper.vm.handleToggleMaximize()
      expect(wrapper.vm.isMaximized).toBe(false)
    })

    it('should show terminal tabs component', async () => {
      await nextTick()
      
      expect(wrapper.text()).toContain('Mock Terminal Tabs')
    })
  })

  describe('Panel Content', () => {
    it('should apply correct height styles when not maximized', async () => {
      wrapper.vm.panelHeight = 500
      wrapper.vm.isMaximized = false
      await nextTick()
      
      const panel = wrapper.find('.flex-none.bg-white')
      expect(panel.attributes('style')).toContain('height: 500px')
    })

    it('should apply fullscreen height when maximized', async () => {
      wrapper.vm.isMaximized = true
      await nextTick()
      
      const panel = wrapper.find('.flex-none.bg-white')
      const style = panel.attributes('style')
      expect(style).toContain('height: 100vh')
    })

    it('should expose openPodLogs method', () => {
      expect(typeof wrapper.vm.openPodLogs).toBe('function')
    })

    it('should expose openPodShell method', () => {
      expect(typeof wrapper.vm.openPodShell).toBe('function')
    })

    it('should have terminalTabsRef', () => {
      expect(wrapper.vm.terminalTabsRef).toBeDefined()
    })
  })

  describe('Resize Functionality', () => {
    it('should handle resize functionality', async () => {
      // Test that resize functions exist and work
      expect(typeof wrapper.vm.startResize).toBe('function')
      expect(typeof wrapper.vm.stopResize).toBe('function')
      
      // Mock mouse event
      const mockEvent = new MouseEvent('mousedown', { clientY: 100 })
      wrapper.vm.startResize(mockEvent)
      
      expect(wrapper.vm.isResizing).toBe(true)
    })

    it('should stop resize and save preferences', async () => {
      wrapper.vm.isResizing = true
      
      wrapper.vm.stopResize()
      
      expect(wrapper.vm.isResizing).toBe(false)
    })

    it('should handle start resize event from child component', async () => {
      const mockEvent = new MouseEvent('mousedown', { clientY: 100 })
      
      wrapper.vm.handleStartResize(mockEvent)
      
      expect(wrapper.vm.isResizing).toBe(true)
    })
  })

  describe('Public API Methods', () => {
    it('should call openPodLogs and emit update:isOpen when panel is closed', async () => {
      await wrapper.setProps({ isOpen: false })
      
      wrapper.vm.openPodLogs(mockPod)
      
      expect(wrapper.emitted('update:isOpen')).toBeTruthy()
      expect(wrapper.emitted('update:isOpen')[0]).toEqual([true])
    })

    it('should call openPodShell and emit update:isOpen when panel is closed', async () => {
      await wrapper.setProps({ isOpen: false })
      
      wrapper.vm.openPodShell(mockPod)
      
      expect(wrapper.emitted('update:isOpen')).toBeTruthy()
      expect(wrapper.emitted('update:isOpen')[0]).toEqual([true])
    })
  })
})