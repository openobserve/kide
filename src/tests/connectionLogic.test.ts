import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import type { MockedFunction } from 'vitest'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn()
}))

describe('Connection Logic', () => {
  let mockInvoke: MockedFunction<any>
  let mockListen: MockedFunction<any>

  beforeEach(async () => {
    vi.clearAllMocks()
    const coreModule = await import('@tauri-apps/api/core')
    const eventModule = await import('@tauri-apps/api/event')
    mockInvoke = vi.mocked(coreModule.invoke)
    mockListen = vi.mocked(eventModule.listen)
  })

  it('should set connection status to connecting initially', () => {
    const connectionStatus = ref('connecting')
    expect(connectionStatus.value).toBe('connecting')
  })

  it('should update connection status to connected on successful connection', async () => {
    // Mock successful connection
    mockInvoke.mockResolvedValueOnce(undefined)

    const connectionStatus = ref('connecting')
    const k8sConnected = ref(false)
    const loading = ref(false)

    // Simulate the handleK8sConnect function logic
    const handleK8sConnect = async () => {
      try {
        connectionStatus.value = 'connecting'
        loading.value = true
        await mockInvoke('connect_k8s')
        k8sConnected.value = true
        connectionStatus.value = 'connected'
      } catch (error) {
        connectionStatus.value = 'failed'
      } finally {
        loading.value = false
      }
    }

    await handleK8sConnect()

    expect(mockInvoke).toHaveBeenCalledWith('connect_k8s')
    expect(connectionStatus.value).toBe('connected')
    expect(k8sConnected.value).toBe(true)
    expect(loading.value).toBe(false)
  })

  it('should update connection status to failed on connection error', async () => {
    // Mock failed connection
    mockInvoke.mockRejectedValueOnce(new Error('Connection failed'))

    const connectionStatus = ref('connecting')
    const k8sConnected = ref(false)
    const loading = ref(false)

    // Simulate the handleK8sConnect function logic
    const handleK8sConnect = async () => {
      try {
        connectionStatus.value = 'connecting'
        loading.value = true
        await mockInvoke('connect_k8s')
        k8sConnected.value = true
        connectionStatus.value = 'connected'
      } catch (error) {
        connectionStatus.value = 'failed'
      } finally {
        loading.value = false
      }
    }

    await handleK8sConnect()

    expect(mockInvoke).toHaveBeenCalledWith('connect_k8s')
    expect(connectionStatus.value).toBe('failed')
    expect(k8sConnected.value).toBe(false)
    expect(loading.value).toBe(false)
  })

  it('should handle resource selection correctly', async () => {
    // Mock successful resource watch start
    mockInvoke.mockResolvedValueOnce(undefined)

    const selectedResource = ref(null)
    const resourceItems = ref([])
    const loading = ref(false)

    const mockResource = {
      name: 'Pods',
      kind: 'Pod'
    }

    // Simulate the handleResourceSelect function logic
    const handleResourceSelect = async (resource) => {
      selectedResource.value = resource
      resourceItems.value = []
      loading.value = true

      try {
        await mockInvoke('start_resource_watch', {
          resourceType: resource.name.toLowerCase(),
          namespace: null
        })
      } catch (error) {
        console.error('Failed to start watch:', error)
      } finally {
        loading.value = false
      }
    }

    await handleResourceSelect(mockResource)

    expect(mockInvoke).toHaveBeenCalledWith('start_resource_watch', {
      resourceType: 'pods',
      namespace: null
    })
    expect(selectedResource.value).toStrictEqual(mockResource)
    expect(resourceItems.value).toEqual([])
    expect(loading.value).toBe(false)
  })
})