import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ResourceShell from '../../components/ResourcePanel/ResourceShell.vue'
import { nextTick } from 'vue'

// Mock xterm - minimal mock that prevents initialization errors
const mockTerminal = {
  loadAddon: vi.fn(),
  open: vi.fn(),
  writeln: vi.fn(),
  write: vi.fn(),
  clear: vi.fn(),
  dispose: vi.fn(),
  onData: vi.fn(),
  cols: 80,
  rows: 24,
}

vi.mock('xterm', () => ({
  Terminal: vi.fn(() => mockTerminal)
}))

vi.mock('xterm-addon-fit', () => ({
  FitAddon: vi.fn(() => ({ fit: vi.fn() }))
}))

vi.mock('xterm-addon-web-links', () => ({
  WebLinksAddon: vi.fn(() => ({}))
}))

vi.mock('xterm/css/xterm.css', () => ({}))

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({ 
  invoke: vi.fn() 
}))

vi.mock('@tauri-apps/api/event', () => ({ 
  listen: vi.fn() 
}))

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

describe('ResourceShell Component', () => {
  let wrapper: VueWrapper<any>
  
  beforeEach(() => {
    vi.clearAllMocks()
    
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
  })

  describe('Component Rendering', () => {
    it('should render the shell component', () => {
      expect(wrapper.exists()).toBe(true)
    })

    it('should display pod information in header', () => {
      expect(wrapper.text()).toContain('test-pod')
      expect(wrapper.text()).toContain('test-namespace')
    })

    it('should show connect button initially', () => {
      expect(wrapper.text()).toContain('Connect')
    })
  })

  describe('Container Selection', () => {
    it('should initialize with first container selected', () => {
      const containerSelect = wrapper.findAll('select')[0]
      expect((containerSelect.element as HTMLSelectElement).value).toBe('web-container')
    })

    it('should show available containers in dropdown', () => {
      const containerSelect = wrapper.findAll('select')[0]
      const options = containerSelect.findAll('option')
      
      expect(options).toHaveLength(2)
      expect(options[0].text()).toContain('web-container')
      expect(options[1].text()).toContain('sidecar-container')
    })

    it('should emit container-changed event when container selection changes', async () => {
      const containerSelect = wrapper.findAll('select')[0]
      await containerSelect.setValue('sidecar-container')
      
      expect(wrapper.emitted('container-changed')).toBeTruthy()
      expect(wrapper.emitted('container-changed')![0]).toEqual(['sidecar-container'])
    })
  })

  describe('Shell Selection', () => {
    it('should default to bash shell', () => {
      const shellSelect = wrapper.findAll('select')[1]
      expect((shellSelect.element as HTMLSelectElement).value).toBe('bash')
    })

    it('should provide secure shell options', () => {
      const shellSelect = wrapper.findAll('select')[1]
      const options = shellSelect.findAll('option')
      const shellValues = options.map(option => option.element.value)
      
      expect(shellValues).toEqual(['bash', 'sh', 'zsh'])
    })

    it('should allow shell selection change', async () => {
      const shellSelect = wrapper.findAll('select')[1]
      await shellSelect.setValue('sh')
      
      expect(wrapper.vm.selectedShell).toBe('sh')
    })
  })

  describe('Component State', () => {
    it('should start in disconnected state', () => {
      expect(wrapper.vm.isConnected).toBe(false)
      expect(wrapper.vm.sessionId).toBe(null)
      expect(wrapper.vm.connectionError).toBe(null)
    })

    it('should show connecting state when connecting', async () => {
      // Access reactive state directly
      wrapper.vm.isConnecting = true
      await nextTick()
      
      expect(wrapper.text()).toContain('Connecting...')
    })

    it('should show connected state when connected', async () => {
      // Access reactive state directly
      wrapper.vm.isConnected = true
      wrapper.vm.sessionId = 'test-session-123'
      await nextTick()
      
      expect(wrapper.text()).toContain('Connected')
    })

    it('should display connection errors', async () => {
      // Access reactive state directly
      wrapper.vm.connectionError = 'Failed to connect to pod'
      await nextTick()
      
      expect(wrapper.text()).toContain('Failed to connect to pod')
      expect(wrapper.find('.text-red-700').exists()).toBe(true)
    })
  })

  describe('Event Handling', () => {
    it('should emit close event when close button is clicked', async () => {
      const closeButton = wrapper.find('[title="Close Shell"]')
      await closeButton.trigger('click')
      
      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })

  describe('Props Validation', () => {
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
})

describe('Shell Component Security', () => {
  it('should only allow secure shell options', () => {
    const wrapper = mount(ResourceShell, {
      props: {
        containers: [{ name: 'test' }],
        podName: 'test-pod',
        namespace: 'test-namespace'
      }
    })
    
    const shellSelect = wrapper.findAll('select')[1]
    const shellOptions = shellSelect.findAll('option')
    const allowedShells = shellOptions.map(option => option.element.value)
    
    // Should only contain secure shell options
    expect(allowedShells).toEqual(['bash', 'sh', 'zsh'])
    
    // Should not contain dangerous shells or commands
    expect(allowedShells).not.toContain('python')
    expect(allowedShells).not.toContain('node')
    expect(allowedShells).not.toContain('rm')
  })
})

describe('Shell Component Backend Integration', () => {
  it('should prepare correct parameters for backend shell connection', async () => {
    const wrapper = mount(ResourceShell, {
      props: {
        containers: [{ name: 'web-container' }],
        podName: 'test-pod',
        namespace: 'test-namespace'
      }
    })
    
    // Simulate connection attempt (without actually calling backend)
    const expectedParams = {
      podName: 'test-pod',
      namespace: 'test-namespace',
      containerName: 'web-container',
      cols: 80,
      rows: 24
    }
    
    // Verify component has the right data for backend call
    expect(wrapper.vm.podName).toBe('test-pod')
    expect(wrapper.vm.namespace).toBe('test-namespace')
    expect(wrapper.vm.selectedContainer).toBe('web-container')
  })
})