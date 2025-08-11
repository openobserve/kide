import { describe, it, expect } from 'vitest'
import { useResourceStatus } from '../../composables/useResourceStatus'
import type { K8sListItem } from '@/types'

describe('useResourceStatus - Endpoints and EndpointSlices', () => {
  const { getStatusText, getStatusClass } = useResourceStatus()

  describe('Endpoints Status', () => {
    it('should return Ready when endpoints has subsets with addresses', () => {
      const endpoints: K8sListItem = {
        kind: 'Endpoints',
        metadata: { name: 'test-endpoints', namespace: 'default' },
        subsets: [
          {
            addresses: [
              { ip: '10.1.1.1' },
              { ip: '10.1.1.2' }
            ],
            ports: [{ port: 80 }]
          }
        ]
      }

      expect(getStatusText(endpoints)).toBe('Ready')
      expect(getStatusClass(endpoints)).toContain('green')
    })

    it('should return NotReady when endpoints has subsets but no addresses', () => {
      const endpoints: K8sListItem = {
        kind: 'Endpoints',
        metadata: { name: 'test-endpoints', namespace: 'default' },
        subsets: [
          {
            addresses: [],
            ports: [{ port: 80 }]
          }
        ]
      }

      expect(getStatusText(endpoints)).toBe('NotReady')
      expect(getStatusClass(endpoints)).toContain('red')
    })

    it('should return NotReady when endpoints has no subsets', () => {
      const endpoints: K8sListItem = {
        kind: 'Endpoints',
        metadata: { name: 'test-endpoints', namespace: 'default' },
        subsets: []
      }

      expect(getStatusText(endpoints)).toBe('NotReady')
      expect(getStatusClass(endpoints)).toContain('red')
    })

    it('should return NotReady when endpoints has undefined subsets', () => {
      const endpoints: K8sListItem = {
        kind: 'Endpoints',
        metadata: { name: 'test-endpoints', namespace: 'default' }
      }

      expect(getStatusText(endpoints)).toBe('NotReady')
      expect(getStatusClass(endpoints)).toContain('red')
    })

    it('should return Ready when at least one subset has addresses', () => {
      const endpoints: K8sListItem = {
        kind: 'Endpoints',
        metadata: { name: 'test-endpoints', namespace: 'default' },
        subsets: [
          {
            addresses: [], // No addresses
            ports: [{ port: 80 }]
          },
          {
            addresses: [{ ip: '10.1.1.1' }], // Has addresses
            ports: [{ port: 8080 }]
          }
        ]
      }

      expect(getStatusText(endpoints)).toBe('Ready')
      expect(getStatusClass(endpoints)).toContain('green')
    })
  })

  describe('EndpointSlice Status', () => {
    it('should return Ready when endpointslice has ready endpoints', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' },
        endpoint_slice: {
          endpoints: [
            {
              addresses: ['10.1.1.1'],
              conditions: { ready: true },
              targetRef: { kind: 'Pod', name: 'pod-1' }
            },
            {
              addresses: ['10.1.1.2'],
              conditions: { ready: true },
              targetRef: { kind: 'Pod', name: 'pod-2' }
            }
          ]
        }
      }

      expect(getStatusText(endpointSlice)).toBe('Ready')
      expect(getStatusClass(endpointSlice)).toContain('green')
    })

    it('should return Ready when at least one endpoint is ready', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' },
        endpoint_slice: {
          endpoints: [
            {
              addresses: ['10.1.1.1'],
              conditions: { ready: false },
              targetRef: { kind: 'Pod', name: 'pod-1' }
            },
            {
              addresses: ['10.1.1.2'],
              conditions: { ready: true },
              targetRef: { kind: 'Pod', name: 'pod-2' }
            }
          ]
        }
      }

      expect(getStatusText(endpointSlice)).toBe('Ready')
      expect(getStatusClass(endpointSlice)).toContain('green')
    })

    it('should return NotReady when all endpoints are not ready', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' },
        endpoint_slice: {
          endpoints: [
            {
              addresses: ['10.1.1.1'],
              conditions: { ready: false },
              targetRef: { kind: 'Pod', name: 'pod-1' }
            },
            {
              addresses: ['10.1.1.2'],
              conditions: { ready: false },
              targetRef: { kind: 'Pod', name: 'pod-2' }
            }
          ]
        }
      }

      expect(getStatusText(endpointSlice)).toBe('NotReady')
      expect(getStatusClass(endpointSlice)).toContain('red')
    })

    it('should return Ready when endpoint has no ready condition (defaults to ready)', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' },
        endpoint_slice: {
          endpoints: [
            {
              addresses: ['10.1.1.1'],
              // No conditions - should be considered ready
              targetRef: { kind: 'Pod', name: 'pod-1' }
            }
          ]
        }
      }

      expect(getStatusText(endpointSlice)).toBe('Ready')
      expect(getStatusClass(endpointSlice)).toContain('green')
    })

    it('should return NotReady when endpointslice has no endpoints', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' },
        endpoint_slice: {
          endpoints: []
        }
      }

      expect(getStatusText(endpointSlice)).toBe('NotReady')
      expect(getStatusClass(endpointSlice)).toContain('red')
    })

    it('should return NotReady when endpointslice has undefined endpoints', () => {
      const endpointSlice: K8sListItem = {
        kind: 'EndpointSlice',
        metadata: { name: 'test-endpointslice', namespace: 'default' }
      }

      expect(getStatusText(endpointSlice)).toBe('NotReady')
      expect(getStatusClass(endpointSlice)).toContain('red')
    })
  })
})