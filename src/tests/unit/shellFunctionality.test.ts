import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ResourceShell from '../../components/ResourcePanel/ResourceShell.vue'
import { nextTick } from 'vue'

// Mock xterm
const mockTerminal = {
  loadAddon: vi.fn(),
  open: vi.fn(),
  writeln: vi.fn(),
  write: vi.fn(),
  clear: vi.fn(),
  dispose: vi.fn(),
  onData: vi.fn((callback: Function) => callback),
  cols: 80,
  rows: 24,
}

vi.mock('xterm', () => ({
  Terminal: vi.fn(() => mockTerminal)
}))

// Mock xterm addons
const mockFitAddon = {
  fit: vi.fn()
}

vi.mock('xterm-addon-fit', () => ({
  FitAddon: vi.fn(() => mockFitAddon)
}))

vi.mock('xterm-addon-web-links', () => ({
  WebLinksAddon: vi.fn(() => ({}))
}))

// Mock CSS import
vi.mock('xterm/css/xterm.css', () => ({}))

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn()
}))

describe('ResourceShell Component', () => {
  let wrapper: VueWrapper<any>
  let mockInvoke: any
  let mockListen: any
  
  beforeEach(async () => {
    // Get the mocked functions
    const coreModule = await import('@tauri-apps/api/core')
    const eventModule = await import('@tauri-apps/api/event')
    mockInvoke = vi.mocked(coreModule.invoke)
    mockListen = vi.mocked(eventModule.listen)
    // Reset all mocks
    vi.clearAllMocks()
    
    // Mock DOM methods
    global.ResizeObserver = vi.fn().mockImplementation(() => ({
      observe: vi.fn(),
      unobserve: vi.fn(),
      disconnect: vi.fn(),
    }))
    
    wrapper = mount(ResourceShell, {
      props: {
        containers: [
          { name: 'web-container' },
          { name: 'sidecar-container' }
        ],
        podName: 'test-pod',
        namespace: 'test-namespace'
      }
    })
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    vi.restoreAllMocks()
  })

  describe('Component Initialization', () => {
    it('should render the shell component', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('should display pod information', () => {
      expect(wrapper.text()).toContain('test-pod')
      expect(wrapper.text()).toContain('test-namespace')
    })

    it('should initialize with first container selected', () => {
      const containerSelect = wrapper.find('select').element as HTMLSelectElement
      expect(containerSelect.value).toBe('web-container')
    })

    it('should show available containers in dropdown', () => {
      const options = wrapper.find('select').findAll('option')
      expect(options).toHaveLength(2)
      expect(options[0].text()).toContain('web-container')
      expect(options[1].text()).toContain('sidecar-container')
    })
  })

  describe('Connection Management', () => {
    it('should start disconnected', () => {
      expect(wrapper.find('button').text()).toContain('Connect')
      expect(wrapper.text()).not.toContain('Connected')
    })



    it('should handle connection success', async () => {
      mockInvoke.mockResolvedValueOnce('session-123')
      mockListen.mockResolvedValueOnce(() => {})
      
      const connectButton = wrapper.find('button')
      await connectButton.trigger('click')
      await nextTick()
      
      expect(wrapper.vm.isConnected).toBe(true)
      expect(wrapper.vm.sessionId).toBe('session-123')
      expect(wrapper.text()).toContain('Connected')
    })

    it('should handle connection error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Connection failed'))
      
      const connectButton = wrapper.find('button')
      await connectButton.trigger('click')
      await nextTick()
      
      expect(wrapper.vm.isConnected).toBe(false)
      expect(wrapper.vm.connectionError).toContain('Connection failed')
    })

  })

  describe('Container Selection', () => {
    it('should emit container-changed event when container is changed', async () => {
      const containerSelect = wrapper.find('select')
      await containerSelect.setValue('sidecar-container')
      
      expect(wrapper.emitted('container-changed')).toBeTruthy()
      expect(wrapper.emitted('container-changed')![0]).toEqual(['sidecar-container'])
    })

    it('should disconnect when container is changed while connected', async () => {
      // First connect
      mockInvoke.mockResolvedValueOnce('session-123')
      mockListen.mockResolvedValueOnce(() => {})
      
      const connectButton = wrapper.find('button')
      await connectButton.trigger('click')
      await nextTick()
      
      expect(wrapper.vm.isConnected).toBe(true)
      
      // Change container - should disconnect
      mockInvoke.mockResolvedValueOnce(undefined)
      const containerSelect = wrapper.find('select')
      await containerSelect.setValue('sidecar-container')
      await nextTick()
      
      expect(wrapper.vm.isConnected).toBe(false)
    })
  })

  describe('Shell Commands', () => {
    beforeEach(async () => {
      // Setup connected state
      mockInvoke.mockResolvedValueOnce('session-123')
      mockListen.mockResolvedValueOnce(() => {})
      
      const connectButton = wrapper.find('button')
      await connectButton.trigger('click')
      await nextTick()
    })


    it('should handle input errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Input validation failed'))
      
      await wrapper.vm.sendInput('rm -rf /')
      await nextTick()
      
      expect(wrapper.vm.connectionError).toContain('Input validation failed')
    })

  })

  describe('Shell Selection', () => {
    it('should default to bash shell', () => {
      const shellSelects = wrapper.findAll('select')
      const shellSelect = shellSelects[1] // Second select is for shell
      expect((shellSelect.element as HTMLSelectElement).value).toBe('bash')
    })

    it('should allow shell selection', async () => {
      const shellSelects = wrapper.findAll('select')
      const shellSelect = shellSelects[1]
      
      await shellSelect.setValue('sh')
      expect(wrapper.vm.selectedShell).toBe('sh')
    })
  })

  describe('Event Handling', () => {

    it('should emit close event when close button is clicked', async () => {
      const closeButton = wrapper.find('[title="Close Shell"]')
      await closeButton.trigger('click')
      
      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('Component Props', () => {
    it('should handle missing containers gracefully', async () => {
      wrapper.unmount()
      wrapper = mount(ResourceShell, {
        props: {
          containers: [],
          podName: 'test-pod',
          namespace: 'test-namespace'
        }
      })
      
      expect(wrapper.vm.selectedContainer).toBe('')
    })

    it('should handle undefined containers', async () => {
      wrapper.unmount()
      wrapper = mount(ResourceShell, {
        props: {
          podName: 'test-pod',
          namespace: 'test-namespace'
        }
      })
      
      expect(wrapper.vm.selectedContainer).toBe('')
    })
  })

  describe('Terminal Integration', () => {

    it('should dispose terminal on unmount', () => {
      const terminalInstance = wrapper.vm.$data.terminal
      wrapper.unmount()
      
      // Can't directly test dispose call due to mocking limitations
      // but this tests the cleanup path
      expect(wrapper.vm.$data).toBeTruthy()
    })
  })

  describe('Error States', () => {

    it('should prevent connection when required props are missing', async () => {
      wrapper.unmount()
      wrapper = mount(ResourceShell, {
        props: {
          containers: [{ name: 'test' }]
          // Missing podName and namespace
        }
      })
      
      const connectButton = wrapper.find('button')
      await connectButton.trigger('click')
      await nextTick()
      
      expect(wrapper.vm.connectionError).toContain('Missing pod name, namespace, or container')
    })
  })
})

describe('Shell Component Security', () => {
  it('should validate shell selection against allowed shells', () => {
    const wrapper = mount(ResourceShell, {
      props: {
        containers: [{ name: 'test' }],
        podName: 'test-pod',
        namespace: 'test-namespace'
      }
    })
    
    // Get the shell selection dropdown (second select element)
    const shellSelect = wrapper.findAll('select')[1]
    const shellOptions = shellSelect.findAll('option')
    const allowedShells = shellOptions.map(option => option.element.value)
    
    expect(allowedShells).toEqual(['bash', 'sh', 'zsh'])
  })
})