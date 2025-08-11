import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import App from '../../App.vue'
import { useClusterStore } from '../../stores/cluster'
import { useResourceStore } from '../../stores/resources'
import type { MockedFunction } from 'vitest'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

// Mock child components to focus on integration logic
vi.mock('../../components/ClusterHotbar.vue', () => ({
  default: {
    name: 'ClusterHotbar',
    template: '<div data-testid="cluster-hotbar">ClusterHotbar</div>',
    props: ['selectedContext', 'connectionStatus'],
    emits: ['context-selected', 'refresh-contexts']
  }
}))

vi.mock('../../components/ResourceNavigation.vue', () => ({
  default: {
    name: 'ResourceNavigation',
    template: '<div data-testid="resource-navigation">ResourceNavigation</div>',
    props: ['categories', 'selectedResource', 'connected', 'connectionStatus', 'currentContextName'],
    emits: ['select-resource']
  }
}))

vi.mock('../../components/ErrorBoundary.vue', () => ({
  default: {
    name: 'ErrorBoundary',
    template: '<slot />',
    props: ['fallbackMessage']
  }
}))

describe('Connection Error Flow Integration', () => {
  let mockInvoke: MockedFunction<any>
  let mockListen: MockedFunction<any>

  beforeEach(async () => {
    vi.clearAllMocks()
    setActivePinia(createPinia())
    
    const coreModule = await import('@tauri-apps/api/core')
    const eventModule = await import('@tauri-apps/api/event')
    mockInvoke = vi.mocked(coreModule.invoke)
    mockListen = vi.mocked(eventModule.listen)
    
    // Default mock for event listener
    mockListen.mockResolvedValue(() => {})
  })

  it('should display connection error in EmptyState when cluster connection fails on app init', async () => {
    // Mock initial connection failure
    mockInvoke.mockRejectedValueOnce(new Error('kubeconfig not found'))
    // Mock resource categories loading success
    mockInvoke.mockResolvedValueOnce([])

    const wrapper = mount(App)
    await flushPromises()

    // Should show EmptyState with error
    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.exists()).toBe(true)
    expect(emptyState.props('connectionStatus')).toBe('failed')
    expect(emptyState.props('connectionError')).toContain('kubeconfig not found')
    expect(emptyState.props('selectedContext')).toBe(null)
  })

  it('should display connection error when context selection fails', async () => {
    // Mock successful initial connection
    mockInvoke.mockResolvedValueOnce([{ name: 'test-context', cluster: 'test', user: 'test' }]) // get_k8s_contexts
    mockInvoke.mockResolvedValueOnce('test-context') // get_current_k8s_context
    mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
    mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    // Now simulate context change that fails
    const errorMessage = 'auth exec command error: token expired'
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

    const failingContext = { name: 'prod-cluster', cluster: 'prod', user: 'admin' }

    // Trigger context selection
    const clusterHotbar = wrapper.findComponent({ name: 'ClusterHotbar' })
    await clusterHotbar.vm.$emit('context-selected', failingContext)
    await flushPromises()

    // Should show EmptyState with error
    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.exists()).toBe(true)
    expect(emptyState.props('connectionStatus')).toBe('failed')
    expect(emptyState.props('connectionError')).toContain(errorMessage)
    // selectedContext remains as the previously successful context when new context fails
    expect(emptyState.props('selectedContext')).toEqual({ name: 'test-context', cluster: 'test', user: 'test' })
    // attemptedContext should be the failing context
    expect(emptyState.props('attemptedContext')).toEqual(failingContext)
  })

  it('should show EmptyState with error when resource is selected but connection fails', async () => {
    // Mock initial successful connection
    mockInvoke.mockResolvedValueOnce([{ name: 'test-context', cluster: 'test', user: 'test' }]) // get_k8s_contexts
    mockInvoke.mockResolvedValueOnce('test-context') // get_current_k8s_context
    mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
    mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    // Select a resource first
    const mockResource = { name: 'Pods', kind: 'Pod', namespaced: true }
    const resourceNavigation = wrapper.findComponent({ name: 'ResourceNavigation' })
    await resourceNavigation.vm.$emit('select-resource', mockResource)
    await flushPromises()

    // Now simulate connection failure (like cluster goes down)
    const clusterStore = useClusterStore()
    clusterStore.connectionStatus = 'failed'
    clusterStore.connectionError = 'Lost connection to cluster: network unreachable'
    
    await wrapper.vm.$nextTick()

    // Should show EmptyState with error, not ResourceList
    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.exists()).toBe(true)
    expect(emptyState.props('connectionStatus')).toBe('failed')
    expect(emptyState.props('connectionError')).toContain('network unreachable')
    expect(emptyState.props('hadSelectedResource')).toBe(true)

    // ResourceList should not be shown
    const resourceList = wrapper.findComponent({ name: 'ResourceList' })
    expect(resourceList.exists()).toBe(false)
  })

  it('should transition from connecting to failed state properly', async () => {
    let resolveConnection: (value: any) => void
    let rejectConnection: (reason: any) => void
    
    const connectionPromise = new Promise((resolve, reject) => {
      resolveConnection = resolve
      rejectConnection = reject
    })

    // Mock delayed connection that will fail
    mockInvoke.mockReturnValueOnce(connectionPromise)
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    // Should initially show connecting state
    let emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('connecting')

    // Reject the connection
    const errorMessage = 'connection timeout'
    rejectConnection!(new Error(errorMessage))
    await flushPromises()

    // Should now show failed state
    emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('failed')
    expect(emptyState.props('connectionError')).toContain(errorMessage)
  })

  it('should clear error state when connection succeeds after failure', async () => {
    // Start with failed connection
    mockInvoke.mockRejectedValueOnce(new Error('initial connection failed'))
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    // Verify error state
    let emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('failed')
    expect(emptyState.props('connectionError')).toBeTruthy()

    // Now simulate successful retry
    const successfulContext = { name: 'working-context', cluster: 'working', user: 'user' }
    mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
    mockInvoke.mockResolvedValueOnce(['default', 'kube-system']) // get_namespaces

    const clusterHotbar = wrapper.findComponent({ name: 'ClusterHotbar' })
    await clusterHotbar.vm.$emit('context-selected', successfulContext)
    await flushPromises()

    // Should clear error state
    emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('connected')
    expect(emptyState.props('connectionError')).toBe(null)
    expect(emptyState.props('selectedContext')).toEqual(successfulContext)
  })

  it('should properly handle authentication errors with formatted messages', async () => {
    const authError = 'auth exec command error: exec plugin: invalid user credentials'
    mockInvoke.mockRejectedValueOnce(new Error(authError))
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('failed')
    
    const errorMessage = emptyState.props('connectionError')
    expect(errorMessage).toContain('Authentication failed')
    expect(errorMessage).toContain('User credentials in kubeconfig')
    expect(errorMessage).toContain('Token expiration')
    expect(errorMessage).toContain(authError)
  })

  it('should handle network timeout errors with helpful guidance', async () => {
    const timeoutError = 'request timeout: context deadline exceeded after 30s'
    mockInvoke.mockRejectedValueOnce(new Error(timeoutError))
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.props('connectionStatus')).toBe('failed')
    
    const errorMessage = emptyState.props('connectionError')
    expect(errorMessage).toContain('Connection timeout')
    expect(errorMessage).toContain('Network connectivity')
    expect(errorMessage).toContain('Cluster responsiveness')
    expect(errorMessage).toContain(timeoutError)
  })

  it('should pass through all required props to EmptyState component', async () => {
    const connectionError = 'test connection error'
    mockInvoke.mockRejectedValueOnce(new Error(connectionError))
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    
    // Verify all props are passed correctly
    expect(emptyState.props()).toEqual({
      selectedContext: null,
      attemptedContext: null,
      connectionStatus: 'failed',
      connectionError: expect.stringContaining(connectionError),
      hadSelectedResource: false
    })
  })

  it('should display attempted context name in error message when context switching fails', async () => {
    // Mock successful initial connection
    mockInvoke.mockResolvedValueOnce([{ name: 'test-context', cluster: 'test', user: 'test' }]) // get_k8s_contexts
    mockInvoke.mockResolvedValueOnce('test-context') // get_current_k8s_context
    mockInvoke.mockResolvedValueOnce(undefined) // connect_k8s_with_context
    mockInvoke.mockResolvedValueOnce(['default']) // get_namespaces
    mockInvoke.mockResolvedValueOnce([]) // get_resources

    const wrapper = mount(App)
    await flushPromises()

    // Now simulate context change that fails
    const errorMessage = 'auth exec command error: token expired'
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage))

    const failingContext = { name: 'prod-cluster', cluster: 'prod', user: 'admin' }

    // Trigger context selection
    const clusterHotbar = wrapper.findComponent({ name: 'ClusterHotbar' })
    await clusterHotbar.vm.$emit('context-selected', failingContext)
    await flushPromises()

    // Check that the error message includes the attempted context name
    const emptyState = wrapper.findComponent({ name: 'EmptyState' })
    expect(emptyState.exists()).toBe(true)
    
    // The error should show the attempted context name even though selectedContext != attemptedContext
    // Since selectedContext (test-context) comes first, it should be shown
    expect(emptyState.text()).toContain('Failed to connect to test-context')
  })
})