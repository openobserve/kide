import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ResourceShell from '../../components/ResourcePanel/ResourceShell.vue'
import { nextTick } from 'vue'

// Mock terminal dependencies
vi.mock('xterm', () => ({
  Terminal: vi.fn(() => ({
    onData: vi.fn(),
    open: vi.fn(),
    writeln: vi.fn(),
    write: vi.fn(),
    clear: vi.fn(),
    focus: vi.fn(),
    dispose: vi.fn(),
    loadAddon: vi.fn(), // Add missing loadAddon method
    scrollToBottom: vi.fn(),
    scrollToLine: vi.fn(), // Add scrollToLine method for enhanced scroll functionality
    cols: 80,
    rows: 24,
    buffer: {
      active: {
        length: 100
      }
    }
  }))
}))

vi.mock('xterm-addon-fit', () => ({
  FitAddon: vi.fn(() => ({
    fit: vi.fn(),
    // Note: FitAddon doesn't have dispose method - this is correct
  }))
}))

vi.mock('xterm-addon-web-links', () => ({
  WebLinksAddon: vi.fn(() => ({
    // WebLinksAddon methods if needed
  }))
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

// Mock CSS import
vi.mock('xterm/css/xterm.css', () => ({}))

describe('Terminal Cleanup Functionality', () => {
  let wrapper: VueWrapper<any>
  let consoleSpy: any

  beforeEach(() => {
    // Spy on console methods to catch any errors
    consoleSpy = {
      warn: vi.spyOn(console, 'warn').mockImplementation(() => {}),
      error: vi.spyOn(console, 'error').mockImplementation(() => {})
    }

    wrapper = mount(ResourceShell, {
      props: {
        containers: [
          { name: 'main' },
          { name: 'sidecar' }
        ],
        podName: 'test-pod',
        namespace: 'default'
      }
    })
  })

  afterEach(() => {
    // Restore console methods
    consoleSpy.warn.mockRestore()
    consoleSpy.error.mockRestore()
    
    if (wrapper) {
      wrapper.unmount()
    }
  })

  describe('Component Cleanup', () => {
    it('should cleanup without errors when component unmounts', async () => {
      // Initialize the component
      await nextTick()
      
      // Simulate connection to create terminal state
      wrapper.vm.isConnected = true
      wrapper.vm.sessionId = 'test-session'
      await nextTick()
      
      // Unmount should not throw errors
      expect(() => {
        wrapper.unmount()
      }).not.toThrow()
      
      // Should not have logged any warnings or errors during cleanup
      expect(consoleSpy.error).not.toHaveBeenCalled()
    })

    it('should handle cleanup when terminal is already disposed', async () => {
      await nextTick()
      
      // Manually call cleanup multiple times
      wrapper.vm.cleanup()
      wrapper.vm.cleanup() // Second call should be prevented by isCleaningUp flag
      
      // Should not throw or log errors
      expect(consoleSpy.error).not.toHaveBeenCalled()
    })

    it('should handle cleanup when fitAddon is null', async () => {
      await nextTick()
      
      // Set fitAddon to null
      wrapper.vm.fitAddon = null
      
      // Cleanup should not throw
      expect(() => {
        wrapper.vm.cleanup()
      }).not.toThrow()
    })

    it('should handle cleanup when terminal is null', async () => {
      await nextTick()
      
      // Set terminal to null
      wrapper.vm.terminal = null
      
      // Cleanup should not throw
      expect(() => {
        wrapper.vm.cleanup()
      }).not.toThrow()
    })

    it('should gracefully handle terminal dispose errors', async () => {
      await nextTick()
      
      // Mock terminal.dispose to throw an error
      if (wrapper.vm.terminal) {
        wrapper.vm.terminal.dispose = vi.fn(() => {
          throw new Error('Dispose failed')
        })
      }
      
      // Cleanup should catch the error and log a warning
      wrapper.vm.cleanup()
      
      // Should have warned about the error but not thrown
      expect(consoleSpy.warn).toHaveBeenCalledWith('Error disposing terminal:', expect.any(Error))
    })

    it('should prevent multiple cleanup calls with isCleaningUp flag', async () => {
      await nextTick()
      
      // Start cleanup
      const cleanupPromise1 = new Promise(resolve => {
        setTimeout(() => {
          wrapper.vm.cleanup()
          resolve(true)
        }, 0)
      })
      
      // Try to cleanup again immediately
      const cleanupPromise2 = new Promise(resolve => {
        setTimeout(() => {
          wrapper.vm.cleanup()
          resolve(true)
        }, 0)
      })
      
      await Promise.all([cleanupPromise1, cleanupPromise2])
      
      // Should not have any errors from concurrent cleanup
      expect(consoleSpy.error).not.toHaveBeenCalled()
    })

    it('should handle disconnect errors gracefully during cleanup', async () => {
      await nextTick()
      
      // Set connected state
      wrapper.vm.isConnected = true
      wrapper.vm.sessionId = 'test-session'
      
      // Mock invoke to throw error on disconnect
      const mockInvoke = vi.mocked(await import('@tauri-apps/api/core')).invoke
      mockInvoke.mockRejectedValue(new Error('Disconnect failed'))
      
      // Cleanup should handle disconnect error
      await wrapper.vm.cleanup()
      
      // Should have logged the disconnect error but continued cleanup
      expect(consoleSpy.error).toHaveBeenCalledWith('Error disconnecting from shell:', expect.any(Error))
    })
  })

  describe('Event Listener Cleanup', () => {
    it('should cleanup shell event listeners without errors', async () => {
      await nextTick()
      
      // Mock event listener
      const mockUnlistener = vi.fn()
      wrapper.vm.shellEventUnlistener = mockUnlistener
      
      // Cleanup should call the unlisten function
      wrapper.vm.cleanup()
      
      expect(mockUnlistener).toHaveBeenCalled()
      expect(wrapper.vm.shellEventUnlistener).toBe(null)
    })

    it('should handle event listener cleanup errors', async () => {
      await nextTick()
      
      // Mock event listener that throws on cleanup
      wrapper.vm.shellEventUnlistener = vi.fn(() => {
        throw new Error('Unlisten failed')
      })
      
      // Cleanup should catch the error
      wrapper.vm.cleanup()
      
      expect(consoleSpy.warn).toHaveBeenCalledWith('Error cleaning up shell event listener:', expect.any(Error))
    })
  })
})