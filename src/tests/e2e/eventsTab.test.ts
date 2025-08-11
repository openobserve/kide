import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceEvents from '../../components/ResourcePanel/ResourceEvents.vue'

// Mock Tauri APIs
const mockInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

const mockEvents = [
  {
    uid: 'event-1',
    type: 'Normal',
    reason: 'Created',
    message: 'Pod created successfully',
    source: { component: 'kubelet' },
    firstTimestamp: '2025-08-05T04:51:15.136893673Z'
  },
  {
    uid: 'event-2',
    type: 'Normal',
    reason: 'Started',
    message: 'Container started',
    source: { component: 'kubelet' },
    firstTimestamp: '2025-08-05T04:51:16.245781234Z'
  },
  {
    uid: 'event-3',
    type: 'Warning',
    reason: 'BackOff',
    message: 'Back-off restarting failed container',
    source: { component: 'kubelet' },
    firstTimestamp: '2025-08-05T04:51:20.345678901Z'
  }
]

describe('Events Tab E2E', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockResolvedValue(mockEvents)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('should display events correctly', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: mockEvents,
        resourceKind: 'Pod'
      }
    })

    // Should display all events
    expect(wrapper.text()).toContain('Created')
    expect(wrapper.text()).toContain('Started')
    expect(wrapper.text()).toContain('BackOff')
    
    // Should display event messages
    expect(wrapper.text()).toContain('Pod created successfully')
    expect(wrapper.text()).toContain('Container started')
    expect(wrapper.text()).toContain('Back-off restarting failed container')
  })

  it('should display different event types with correct styling', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: mockEvents,
        resourceKind: 'Pod'
      }
    })

    // Should have Normal events with blue styling
    const normalEvents = wrapper.findAll('.bg-blue-100')
    expect(normalEvents.length).toBe(2)

    // Should have Warning events with yellow styling
    const warningEvents = wrapper.findAll('.bg-yellow-100')
    expect(warningEvents.length).toBe(1)
  })

  it('should display event components and timestamps', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: mockEvents,
        resourceKind: 'Pod'
      }
    })

    // Should display component sources
    expect(wrapper.text()).toContain('kubelet')
    
    // Should display formatted timestamps
    mockEvents.forEach(event => {
      if (event.firstTimestamp) {
        const expectedTime = new Date(event.firstTimestamp).toLocaleString()
        expect(wrapper.text()).toContain(expectedTime)
      }
    })
  })

  it('should show empty state when no events', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: [],
        resourceKind: 'Deployment'
      }
    })

    expect(wrapper.text()).toContain('No events found for this deployment')
  })

  it('should display event indicators with correct colors', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: mockEvents,
        resourceKind: 'Pod'
      }
    })

    // Should have blue indicators for Normal events
    const blueIndicators = wrapper.findAll('.bg-blue-500')
    expect(blueIndicators.length).toBe(2)

    // Should have yellow indicators for Warning events  
    const yellowIndicators = wrapper.findAll('.bg-yellow-500')
    expect(yellowIndicators.length).toBe(1)
  })

  it('should handle events with missing data gracefully', () => {
    const incompleteEvents = [
      {
        uid: 'event-incomplete',
        type: 'Normal',
        reason: 'Test',
        message: 'Test message',
        // Missing source and timestamp
      }
    ]

    const wrapper = mount(ResourceEvents, {
      props: {
        events: incompleteEvents,
        resourceKind: 'Pod'
      }
    })

    expect(wrapper.text()).toContain('Test')
    expect(wrapper.text()).toContain('Test message')
    expect(wrapper.text()).toContain('Unknown') // Should show "Unknown" for missing timestamp
  })

  it('should display event reasons as headers', () => {
    const wrapper = mount(ResourceEvents, {
      props: {
        events: mockEvents,
        resourceKind: 'Pod'
      }
    })

    // Event reasons should be displayed prominently  
    mockEvents.forEach(event => {
      expect(wrapper.text()).toContain(event.reason)
    })
    
    // Should have the correct number of event containers
    const eventContainers = wrapper.findAll('.border-gray-200')
    expect(eventContainers.length).toBe(mockEvents.length)
  })
})

describe('Events Integration Tests', () => {
  it('should handle events with long messages', () => {
    const longMessageEvents = [
      {
        uid: 'long-event',
        type: 'Warning',
        reason: 'FailedMount',
        message: 'MountVolume.SetUp failed for volume "data-volume" : mount failed: exit status 32\nMounting command: systemd-run\nMounting arguments: --description=Kubernetes transient mount for /var/lib/kubelet/pods/123/volumes/kubernetes.io~nfs/data-volume --scope --slice=system.slice --property=DefaultDependencies=false --property=RemainAfterExit=true --property=ExecStart=/bin/mount -t nfs server:/path /var/lib/kubelet/pods/123/volumes/kubernetes.io~nfs/data-volume --property=Type=oneshot /bin/mount -t nfs server:/path /var/lib/kubelet/pods/123/volumes/kubernetes.io~nfs/data-volume',
        source: { component: 'kubelet' },
        firstTimestamp: '2025-08-05T04:51:20.345678901Z'
      }
    ]

    const wrapper = mount(ResourceEvents, {
      props: {
        events: longMessageEvents,
        resourceKind: 'Pod'
      }
    })

    // Should display the long message without breaking layout
    expect(wrapper.text()).toContain('MountVolume.SetUp failed')
    expect(wrapper.text()).toContain('exit status 32')
  })

  it('should handle events with special characters', () => {
    const specialCharEvents = [
      {
        uid: 'special-event',
        type: 'Normal',
        reason: 'Pulled',
        message: 'Successfully pulled image "nginx:latest" (digest: sha256:abc123...)',
        source: { component: 'kubelet' },
        firstTimestamp: '2025-08-05T04:51:15.136893673Z'
      }
    ]

    const wrapper = mount(ResourceEvents, {
      props: {
        events: specialCharEvents,
        resourceKind: 'Pod'
      }
    })

    expect(wrapper.text()).toContain('nginx:latest')
    expect(wrapper.text()).toContain('sha256:abc123...')
  })
})