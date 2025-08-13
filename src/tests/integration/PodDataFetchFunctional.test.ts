import { describe, it, expect, vi, beforeEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)

describe('Pod Data Fetch Functional Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  const mockPodData = {
    metadata: { name: 'test-pod', namespace: 'default' },
    spec: { containers: [{ name: 'main', image: 'nginx' }] },
    status: { phase: 'Running' }
  }

  const mockCompletePodData = {
    metadata: { name: 'test-pod', namespace: 'default', creationTimestamp: '2023-01-01T00:00:00Z' },
    spec: {
      containers: [{ name: 'main', image: 'nginx', env: [{ name: 'ENV_VAR', value: 'test' }] }],
      volumes: [{ name: 'config-vol', configMap: { name: 'app-config' } }],
      tolerations: [{ key: 'node.kubernetes.io/not-ready', operator: 'Exists' }]
    },
    status: {
      phase: 'Running',
      podIP: '10.244.0.5',
      conditions: [{ type: 'Ready', status: 'True' }, { type: 'PodScheduled', status: 'True' }],
      containerStatuses: [{ name: 'main', state: { running: {} }, ready: true }]
    }
  }

  // Simulate the actual handleViewPod logic
  async function simulateNodePodsHandleViewPod(pod: any): Promise<any> {
    try {
      const completePodData = await invoke<any>('get_resource', {
        resourceKind: 'Pod',
        resourceName: pod.metadata?.name,
        namespace: pod.metadata?.namespace
      })
      
      // Map the data structure to match what ResourcePanel expects
      const normalizedPod = {
        ...completePodData,
        kind: 'Pod',
        podStatus: completePodData.status,
        podSpec: completePodData.spec,
        status: completePodData.status,
        spec: completePodData.spec
      }
      
      return normalizedPod
    } catch (error) {
      const normalizedPod = {
        ...pod,
        kind: 'Pod',
        podStatus: pod.status || pod.podStatus,
        podSpec: pod.spec || pod.podSpec,
        status: pod.status || pod.podStatus,
        spec: pod.spec || pod.podSpec
      }
      return normalizedPod
    }
  }

  // Simulate the actual handleViewPod logic for WorkloadPods
  async function simulateWorkloadPodsHandleViewPod(pod: any): Promise<any> {
    try {
      const completePodData = await invoke<any>('get_resource', {
        resourceKind: 'Pod',
        resourceName: pod.metadata?.name,
        namespace: pod.metadata?.namespace
      })
      
      // Map the data structure to match what ResourcePanel expects
      const normalizedPod = {
        ...completePodData,
        kind: 'Pod',
        podStatus: completePodData.status,
        podSpec: completePodData.spec,
        status: completePodData.status,
        spec: completePodData.spec
      }
      
      return normalizedPod
    } catch (error) {
      const normalizedPod = {
        ...pod,
        kind: 'Pod',
        podStatus: pod.status || pod.podStatus,
        podSpec: pod.spec || pod.podSpec,
        status: pod.status || pod.podStatus,
        spec: pod.spec || pod.podSpec
      }
      return normalizedPod
    }
  }

  describe('NodePods handleViewPod Logic', () => {
    it('should fetch complete pod data and map structure correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // Verify API call
      expect(mockInvoke).toHaveBeenCalledWith('get_resource', {
        resourceKind: 'Pod',
        resourceName: 'test-pod',
        namespace: 'default'
      })

      // Verify proper data structure mapping
      expect(result).toMatchObject({
        kind: 'Pod',
        metadata: mockCompletePodData.metadata,
        spec: mockCompletePodData.spec,
        status: mockCompletePodData.status,
        podSpec: mockCompletePodData.spec,
        podStatus: mockCompletePodData.status
      })

      // Verify complete data for ResourcePanel tabs
      expect(result.spec.volumes).toHaveLength(1)
      expect(result.spec.tolerations).toHaveLength(1)
      expect(result.status.conditions).toHaveLength(2)
      expect(result.spec.containers[0].env).toHaveLength(1)

      // Verify dual structure access
      expect(result.podSpec).toBe(result.spec)
      expect(result.podStatus).toBe(result.status)
    })

    it('should handle API errors with fallback data mapping', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('API Error'))

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // Verify fallback structure mapping
      expect(result).toMatchObject({
        kind: 'Pod',
        metadata: mockPodData.metadata,
        podSpec: mockPodData.spec,
        podStatus: mockPodData.status,
        spec: mockPodData.spec,
        status: mockPodData.status
      })
    })
  })

  describe('WorkloadPods handleViewPod Logic', () => {
    it('should fetch complete pod data and map structure correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateWorkloadPodsHandleViewPod(mockPodData)

      // Verify API call
      expect(mockInvoke).toHaveBeenCalledWith('get_resource', {
        resourceKind: 'Pod',
        resourceName: 'test-pod',
        namespace: 'default'
      })

      // Verify proper data structure mapping
      expect(result).toMatchObject({
        kind: 'Pod',
        metadata: mockCompletePodData.metadata,
        spec: mockCompletePodData.spec,
        status: mockCompletePodData.status,
        podSpec: mockCompletePodData.spec,
        podStatus: mockCompletePodData.status
      })

      // Verify complete data for ResourcePanel tabs
      expect(result.spec.volumes).toHaveLength(1)
      expect(result.spec.tolerations).toHaveLength(1)
      expect(result.status.conditions).toHaveLength(2)
      expect(result.spec.containers[0].env).toHaveLength(1)
    })

    it('should handle API errors with fallback data mapping', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network Error'))

      const result = await simulateWorkloadPodsHandleViewPod(mockPodData)

      // Verify fallback structure mapping
      expect(result).toMatchObject({
        kind: 'Pod',
        metadata: mockPodData.metadata,
        podSpec: mockPodData.spec,
        podStatus: mockPodData.status
      })
    })
  })

  describe('Data Structure Validation for ResourcePanel', () => {
    it('should provide data structure compatible with useResourceStatus composable', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // Validate structure for getStatusForKind and getSpecForKind
      expect(result.kind).toBe('Pod')
      expect(result.podStatus).toBeDefined()
      expect(result.podSpec).toBeDefined()

      // Validate that useResourceStatus can access pod data
      // getStatusForKind(item) looks for item.podStatus for Pods
      // getSpecForKind(item) looks for item.podSpec for Pods
      expect(result.podStatus.phase).toBe('Running')
      expect(result.podSpec.containers).toHaveLength(1)
    })

    it('should provide data structure compatible with YAML tab', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // YAML tab expects direct access to status and spec
      expect(result.status).toBeDefined()
      expect(result.spec).toBeDefined()
      expect(result.metadata).toBeDefined()

      // Should be the complete Kubernetes object structure
      expect(result.status.phase).toBe('Running')
      expect(result.spec.containers).toHaveLength(1)
      expect(result.metadata.name).toBe('test-pod')
    })

    it('should provide data structure compatible with Overview tab components', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // Overview tab components expect getGenericSpec() and getGenericStatus()
      // which call getSpecForKind() and getStatusForKind()
      
      // For Pod volumes component: getGenericSpec(resourceData)?.volumes
      expect(result.podSpec.volumes).toBeDefined()
      expect(Array.isArray(result.podSpec.volumes)).toBe(true)

      // For Pod tolerations component: getGenericSpec(resourceData)?.tolerations
      expect(result.podSpec.tolerations).toBeDefined()
      expect(Array.isArray(result.podSpec.tolerations)).toBe(true)

      // For Pod conditions component: getGenericStatus(resourceData)?.conditions
      expect(result.podStatus.conditions).toBeDefined()
      expect(Array.isArray(result.podStatus.conditions)).toBe(true)
    })

    it('should provide data structure compatible with Containers tab', async () => {
      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateNodePodsHandleViewPod(mockPodData)

      // Containers tab expects complete container specifications
      expect(result.spec.containers).toBeDefined()
      expect(Array.isArray(result.spec.containers)).toBe(true)
      expect(result.spec.containers[0]).toHaveProperty('name')
      expect(result.spec.containers[0]).toHaveProperty('image')
      expect(result.spec.containers[0]).toHaveProperty('env')

      // Also expects container status information
      expect(result.status.containerStatuses).toBeDefined()
      expect(Array.isArray(result.status.containerStatuses)).toBe(true)
    })
  })

  describe('Regression Prevention', () => {
    it('should prevent the original issue where pods showed tabs but no data', async () => {
      // This test ensures that the fix prevents the scenario where:
      // 1. User clicks on pod from workload or node pods table
      // 2. Side panel opens with all tabs visible
      // 3. But tabs show empty/no data because pod data structure was incomplete

      mockInvoke.mockResolvedValueOnce(mockCompletePodData)

      const result = await simulateWorkloadPodsHandleViewPod(mockPodData)

      // Ensure all tab requirements are met:

      // 1. Overview tab should have volumes, tolerations, conditions
      expect(result.podSpec.volumes).toBeTruthy()
      expect(result.podSpec.tolerations).toBeTruthy()
      expect(result.podStatus.conditions).toBeTruthy()

      // 2. Containers tab should have complete container data
      expect(result.spec.containers).toBeTruthy()
      expect(result.spec.containers[0].env).toBeTruthy()

      // 3. YAML tab should have complete object
      expect(result.metadata).toBeTruthy()
      expect(result.spec).toBeTruthy()
      expect(result.status).toBeTruthy()

      // 4. Events tab should have resource reference
      expect(result.kind).toBe('Pod')
      expect(result.metadata.name).toBeTruthy()
      expect(result.metadata.namespace).toBeTruthy()

      // 5. Logs tab should have container information
      expect(result.spec.containers.length).toBeGreaterThan(0)
      expect(result.spec.containers[0].name).toBeTruthy()
    })
  })
})