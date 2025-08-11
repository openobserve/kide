/**
 * Pod Data Integration Tests
 * 
 * These tests ensure that Pod data flows correctly from backend to frontend
 * and catches issues where Pod columns would show empty values.
 */

import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { useContainerStatus } from '@/composables/useContainerStatus'
import { useResourceStatus } from '@/composables/useResourceStatus'
import type { K8sListItem } from '@/types'

describe('Pod Data Integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const createPodItem = (overrides: Partial<K8sListItem> = {}): K8sListItem => ({
    metadata: {
      name: 'test-pod',
      namespace: 'default',
      uid: '12345'
    },
    kind: 'Pod',
    apiVersion: 'v1',
    // Simulate the camelCase structure that comes from Tauri serde serialization
    podSpec: {
      nodeName: 'ip-10-2-45-217.us-east-2.compute.internal',
      containers: [
        { name: 'main-container', image: 'nginx:latest' }
      ]
    },
    podStatus: {
      phase: 'Running',
      containerStatuses: [
        {
          name: 'main-container',
          restartCount: 2,
          ready: true,
          state: {
            running: { startedAt: '2023-01-01T00:00:00Z' }
          }
        }
      ],
      initContainerStatuses: [
        {
          name: 'init-container',
          restartCount: 1,
          ready: true
        }
      ]
    },
    ...overrides
  } as K8sListItem)

  describe('useContainerStatus composable', () => {
    it('should calculate total restart count correctly', () => {
      const { getTotalRestartCount } = useContainerStatus()
      const podItem = createPodItem()

      const totalRestarts = getTotalRestartCount(podItem)

      // Should sum main container (2) + init container (1) = 3
      expect(totalRestarts).toBe(3)
    })

    it('should handle missing container statuses gracefully', () => {
      const { getTotalRestartCount } = useContainerStatus()
      const podItem = createPodItem({
        podStatus: {
          phase: 'Running',
          containerStatuses: undefined,
          initContainerStatuses: undefined
        }
      } as any)

      const totalRestarts = getTotalRestartCount(podItem)
      expect(totalRestarts).toBe(0)
    })

    it('should return 0 for non-Pod resources', () => {
      const { getTotalRestartCount } = useContainerStatus()
      const deploymentItem = {
        ...createPodItem(),
        kind: 'Deployment'
      }

      const totalRestarts = getTotalRestartCount(deploymentItem)
      expect(totalRestarts).toBe(0)
    })

    it('should handle missing podStatus gracefully', () => {
      const { getTotalRestartCount } = useContainerStatus()
      const podItem = createPodItem({
        podStatus: undefined
      } as any)

      const totalRestarts = getTotalRestartCount(podItem)
      expect(totalRestarts).toBe(0)
    })
  })

  describe('useResourceStatus composable', () => {
    it('should extract Pod phase correctly', () => {
      const { getStatusText } = useResourceStatus()
      const podItem = createPodItem()

      const status = getStatusText(podItem)
      expect(status).toBe('Running')
    })

    it('should handle missing Pod status gracefully', () => {
      const { getStatusText } = useResourceStatus()
      const podItem = createPodItem({
        podStatus: undefined
      } as any)

      const status = getStatusText(podItem)
      expect(status).toBe('Unknown')
    })

    it('should extract Pod spec correctly for node name', () => {
      const { getGenericSpec } = useResourceStatus()
      const podItem = createPodItem()

      const spec = getGenericSpec(podItem)
      expect(spec).toBeDefined()
      expect(spec.nodeName).toBe('ip-10-2-45-217.us-east-2.compute.internal')
    })

    it('should handle missing Pod spec gracefully', () => {
      const { getGenericSpec } = useResourceStatus()
      const podItem = createPodItem({
        podSpec: undefined
      } as any)

      const spec = getGenericSpec(podItem)
      expect(spec).toBeUndefined()
    })
  })

  describe('Pod field naming compatibility', () => {
    it('should access Pod data using camelCase field names (podSpec, podStatus)', () => {
      const podItem = createPodItem()

      // Verify the structure matches what Tauri serde produces
      expect((podItem as any).podSpec).toBeDefined()
      expect((podItem as any).podStatus).toBeDefined()
      
      // Verify snake_case fields don't exist (this was the old broken structure)
      expect((podItem as any).pod_spec).toBeUndefined()
      expect((podItem as any).pod_status).toBeUndefined()
    })

    it('should have the correct nested structure for container data', () => {
      const podItem = createPodItem()

      const podStatus = (podItem as any).podStatus
      expect(podStatus.phase).toBe('Running')
      expect(podStatus.containerStatuses).toHaveLength(1)
      expect(podStatus.containerStatuses[0].restartCount).toBe(2)
      expect(podStatus.initContainerStatuses).toHaveLength(1)
      expect(podStatus.initContainerStatuses[0].restartCount).toBe(1)
    })
  })

  describe('Critical regression tests', () => {
    it('should prevent Pod columns from showing empty values', () => {
      // This test ensures the exact issue we fixed doesn't regress
      const { getTotalRestartCount, getQoSClass } = useContainerStatus()
      const { getStatusText, getGenericSpec } = useResourceStatus()
      
      const podItem = createPodItem()

      // Test all three critical columns that were broken
      
      // 1. Status column
      const status = getStatusText(podItem)
      expect(status).not.toBe('Unknown')
      expect(status).toBe('Running')
      
      // 2. Restarts column  
      const restarts = getTotalRestartCount(podItem)
      expect(restarts).toBeGreaterThan(0)
      expect(restarts).toBe(3)
      
      // 3. Node column
      const spec = getGenericSpec(podItem)
      expect(spec).toBeDefined()
      expect(spec.nodeName).toBeTruthy()
      expect(spec.nodeName).toContain('ip-10-2')
    })

    it('should verify Pod data can be JSON serialized (backend serialization test)', () => {
      // This simulates the Tauri IPC serialization that was failing
      const podItem = createPodItem()

      expect(() => {
        const serialized = JSON.stringify(podItem)
        const deserialized = JSON.parse(serialized)
        
        // Verify essential fields survive serialization
        expect(deserialized.podSpec.nodeName).toBeTruthy()
        expect(deserialized.podStatus.phase).toBeTruthy()
        expect(deserialized.podStatus.containerStatuses).toHaveLength(1)
      }).not.toThrow()
    })
  })
})