import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceList from '../../components/ResourceList.vue'
import type { K8sResource, K8sListItem } from '@/types'

describe('ResourceList - EndpointSlice Columns', () => {
  const endpointSliceResource: K8sResource = {
    name: 'EndpointSlices',
    namespaced: true,
    apiVersion: 'discovery.k8s.io/v1',
    kind: 'EndpointSlice',
    description: 'EndpointSlice resources'
  }

  it('should display EndpointSlice specific columns', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'test-endpointslice',
        uid: 'endpointslice-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        addressType: 'IPv4',
        ports: [
          { port: 80, protocol: 'TCP', name: 'http' },
          { port: 443, protocol: 'TCP', name: 'https' }
        ],
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
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    // Should display custom column headers
    expect(wrapper.text()).toContain('Address Type')
    expect(wrapper.text()).toContain('Ports')
    expect(wrapper.text()).toContain('Endpoints')

    // Should display address type
    expect(wrapper.text()).toContain('IPv4')

    // Should display ports information (just port numbers)
    expect(wrapper.text()).toContain('80,443')

    // Should display endpoints as IP addresses
    expect(wrapper.text()).toContain('10.1.1.1,10.1.1.2')
  })

  it('should handle IPv6 address type', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'test-endpointslice-ipv6',
        uid: 'endpointslice-456',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        addressType: 'IPv6',
        ports: [{ port: 8080, protocol: 'TCP' }],
        endpoints: [
          {
            addresses: ['2001:db8::1'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-ipv6' }
          }
        ]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('IPv6')
  })

  it('should handle EndpointSlice with no ports', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'no-ports-endpointslice',
        uid: 'endpointslice-789',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        addressType: 'IPv4',
        ports: [],
        endpoints: [
          {
            addresses: ['10.1.1.1'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-1' }
          }
        ]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    // Should show dash for no ports
    expect(wrapper.html()).toContain('-')
  })

  it('should handle mixed ready/not ready endpoints', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'mixed-endpoints',
        uid: 'endpointslice-mixed',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        addressType: 'IPv4',
        ports: [{ port: 80, protocol: 'TCP' }],
        endpoints: [
          {
            addresses: ['10.1.1.1'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-1' }
          },
          {
            addresses: ['10.1.1.2'],
            conditions: { ready: false },
            targetRef: { kind: 'Pod', name: 'pod-2' }
          },
          {
            addresses: ['10.1.1.3'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-3' }
          }
        ]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    // Should show all IP addresses (regardless of ready state)
    expect(wrapper.text()).toContain('10.1.1.1,10.1.1.2')
  })

  it('should handle EndpointSlice with many ports (truncation)', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'many-ports-endpointslice',
        uid: 'endpointslice-ports',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        addressType: 'IPv4',
        ports: [
          { port: 80, protocol: 'TCP', name: 'http' },
          { port: 443, protocol: 'TCP', name: 'https' },
          { port: 8080, protocol: 'TCP', name: 'alt-http' },
          { port: 9090, protocol: 'TCP', name: 'metrics' }
        ],
        endpoints: [
          {
            addresses: ['10.1.1.1'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-1' }
          }
        ]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    // Should show first three ports and +1 more
    expect(wrapper.text()).toContain('80,443,8080')
    expect(wrapper.text()).toContain('+1')
  })

  it('should handle EndpointSlice with default address type', () => {
    const endpointSliceItems: K8sListItem[] = [{
      kind: 'EndpointSlice',
      apiVersion: 'discovery.k8s.io/v1',
      metadata: {
        name: 'default-type-endpointslice',
        uid: 'endpointslice-default',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      endpoint_slice: {
        // No addressType specified - should default to IPv4
        ports: [{ port: 80, protocol: 'TCP' }],
        endpoints: [
          {
            addresses: ['10.1.1.1'],
            conditions: { ready: true },
            targetRef: { kind: 'Pod', name: 'pod-1' }
          }
        ]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: endpointSliceResource,
        items: endpointSliceItems,
        loading: false
      }
    })

    // Should default to IPv4
    expect(wrapper.text()).toContain('IPv4')
  })
})