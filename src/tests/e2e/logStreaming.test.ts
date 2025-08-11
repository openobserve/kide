import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceLogs from '../../components/ResourcePanel/ResourceLogs.vue'

// Mock Tauri APIs
const mockInvoke = vi.fn()
const mockListen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen
}))

const mockContainers = [
  {
    name: 'main-container',
    image: 'nginx:latest'
  },
  {
    name: 'sidecar-container',
    image: 'fluent-bit:latest'
  }
]

const mockInitContainers = [
  {
    name: 'init-setup',
    image: 'busybox:latest'
  }
]

describe('Log Streaming E2E', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockResolvedValue('test-stream-id')
    mockListen.mockResolvedValue(() => {}) // Mock unlisten function
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('should display container selection dropdown', () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        initContainers: mockInitContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    const select = wrapper.find('select')
    expect(select.exists()).toBe(true)

    // Should have optgroups for init and regular containers
    const optgroups = wrapper.findAll('optgroup')
    expect(optgroups.length).toBeGreaterThan(0)
    
    // Check for optgroup labels
    const labels = optgroups.map(group => group.attributes('label'))
    expect(labels).toContain('Init Containers')
    expect(labels).toContain('Containers')
    
    // Should list all containers
    expect(wrapper.text()).toContain('init-setup (init)')
    expect(wrapper.text()).toContain('main-container')
    expect(wrapper.text()).toContain('sidecar-container')
  })

  it('should handle refresh logs button', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    await wrapper.find('button').trigger('click')
    
    expect(wrapper.emitted('refresh-logs')).toBeTruthy()
  })

  it('should toggle live logging', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    // Initially should show "Start Live"
    expect(wrapper.text()).toContain('Start Live')
    
    const buttons = wrapper.findAll('button')
    const liveButton = buttons.find(button => button.text().includes('Start Live'))
    await liveButton!.trigger('click')
    
    expect(wrapper.emitted('toggle-live-logging')).toBeTruthy()
  })

  it('should show live indicator when streaming', () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: true
      }
    })

    expect(wrapper.text()).toContain('Stop Live')
    expect(wrapper.text()).toContain('Live')
    expect(wrapper.find('.animate-pulse').exists()).toBe(true)
  })

  it('should display log lines correctly', () => {
    const logLines = [
      '2024-01-01T10:00:00Z Starting application...',
      '2024-01-01T10:00:01Z Application ready',
      '2024-01-01T10:00:02Z Listening on port 8080'
    ]

    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines,
        isLiveLogging: false
      }
    })

    logLines.forEach(line => {
      expect(wrapper.text()).toContain(line)
    })
  })

  it('should parse and highlight timestamps correctly', () => {
    const logLines = [
      '2025-08-05T04:51:15.136893673Z Starting container...',
      '2025-08-05T04:51:16.245781234Z Container ready',
      'Log without timestamp'
    ]

    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines,
        isLiveLogging: false
      }
    })

    // Should contain timestamps with different styling
    const timestampSpans = wrapper.findAll('.text-gray-400')
    expect(timestampSpans.length).toBe(3)
    
    // Should contain log content
    expect(wrapper.text()).toContain('Starting container...')
    expect(wrapper.text()).toContain('Container ready') 
    expect(wrapper.text()).toContain('Log without timestamp')
  })

  it('should show empty state when no logs', () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    expect(wrapper.text()).toContain('No logs available')
  })

  it('should emit container change event', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    const select = wrapper.find('select')
    await select.setValue('sidecar-container')
    
    expect(wrapper.emitted('container-changed')).toBeTruthy()
    expect(wrapper.emitted('container-changed')[0]).toEqual(['sidecar-container'])
  })

  it('should not have follow logs or timestamps checkboxes', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    const checkboxes = wrapper.findAll('input[type="checkbox"]')
    expect(checkboxes).toHaveLength(0)
    
    // Should not contain the text for these checkboxes
    expect(wrapper.text()).not.toContain('Follow')
    expect(wrapper.text()).not.toContain('Timestamps')
  })


  it('should auto-select first container on mount', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        initContainers: mockInitContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    // Wait for watchers to trigger
    await wrapper.vm.$nextTick()
    
    // Check that the select element has the correct value
    const select = wrapper.find('select')
    expect(select.element.value).toBe('main-container')
  })

  it('should select init container if no regular containers', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: [],
        initContainers: mockInitContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    // Wait for watchers to trigger
    await wrapper.vm.$nextTick()
    
    // Check that the select element has the correct value for init containers
    const select = wrapper.find('select')
    expect(select.element.value).toBe('init-init-setup')
  })
})

describe('Log Streaming Integration Tests', () => {
  it('should handle long log lines without breaking layout', () => {
    const longLogLines = [
      'A'.repeat(1000),
      'B'.repeat(500) + ' - This is a very long log line that should wrap properly',
      'C'.repeat(2000)
    ]

    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: longLogLines,
        isLiveLogging: false
      }
    })

    // Should contain the log lines
    longLogLines.forEach(line => {
      expect(wrapper.text()).toContain(line.substring(0, 100)) // Check first part
    })

    // Should have proper CSS classes for wrapping
    expect(wrapper.find('.whitespace-pre-wrap').exists()).toBe(true)
  })

  it('should handle special characters in logs', () => {
    const specialLogLines = [
      'JSON: {"key": "value", "number": 123}',
      'XML: <root><item id="1">test</item></root>',
      'Symbols: !@#$%^&*()_+-=[]{}|;:,.<>?',
      'Unicode: 🎉 ✅ ❌ 🚀 🔧',
      'Escape sequences: \\n \\t \\r \\"'
    ]

    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: specialLogLines,
        isLiveLogging: false
      }
    })

    specialLogLines.forEach(line => {
      expect(wrapper.text()).toContain(line)
    })
  })

  it('should handle rapid log updates during live streaming', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: ['Initial log'],
        isLiveLogging: true
      }
    })

    // Simulate rapid log updates
    const newLogLines = []
    for (let i = 0; i < 100; i++) {
      newLogLines.push(`Log line ${i}`)
    }

    await wrapper.setProps({ logLines: newLogLines })
    
    // Should handle the updates without crashing
    expect(wrapper.exists()).toBe(true)
    expect(wrapper.text()).toContain('Log line 99')
  })

  it('should handle many log lines without performance issues', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: Array.from({ length: 50 }, (_, i) => `Log line ${i}`),
        isLiveLogging: false
      }
    })

    // Should render all log lines
    expect(wrapper.text()).toContain('Log line 0')
    expect(wrapper.text()).toContain('Log line 49')
    
    // Add more log lines
    await wrapper.setProps({
      logLines: [...wrapper.props('logLines'), 'New log line']
    })
    
    // Should handle the update without issues
    expect(wrapper.text()).toContain('New log line')
  })

  it('should handle container switching during live logging', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: ['Container 1 logs'],
        isLiveLogging: true
      }
    })

    // Switch to different container
    const select = wrapper.find('select')
    await select.setValue('sidecar-container')
    
    expect(wrapper.emitted('container-changed')).toBeTruthy()
    expect(wrapper.emitted('container-changed')[0]).toEqual(['sidecar-container'])
  })

  it('should reset container selection when containers change', async () => {
    const wrapper = mount(ResourceLogs, {
      props: {
        containers: mockContainers,
        logLines: [],
        isLiveLogging: false
      }
    })

    // Initially should select first container
    await wrapper.vm.$nextTick()
    expect(wrapper.find('select').element.value).toBe('main-container')

    // Change to different containers (simulating pod change)
    const newContainers = [
      { name: 'web-server', image: 'nginx:alpine' },
      { name: 'db-sidecar', image: 'postgres:13' }
    ]

    await wrapper.setProps({ containers: newContainers })
    await wrapper.vm.$nextTick()

    // Should reset to first container of new pod
    expect(wrapper.find('select').element.value).toBe('web-server')
    expect(wrapper.emitted('container-changed')).toBeTruthy()
  })
})