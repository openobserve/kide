import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceList from '../../components/ResourceList.vue'
import type { K8sResource, K8sListItem } from '../../types'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('ResourceList Sorting', () => {
  const mockResource: K8sResource = {
    name: 'Pods',
    kind: 'Pod',
    apiVersion: 'v1',
    namespaced: true
  }

  const mockPods: K8sListItem[] = [
    {
      kind: 'Pod',
      apiVersion: 'v1',
      metadata: {
        name: 'pod-b',
        namespace: 'default',
        creationTimestamp: '2023-01-02T00:00:00Z',
        uid: 'pod-b-uid'
      },
      podSpec: {
        nodeName: 'node-2',
        containers: [{ name: 'container1' }]
      },
      podStatus: {
        phase: 'Running',
        containerStatuses: [{ restartCount: 2 }]
      }
    },
    {
      kind: 'Pod',
      apiVersion: 'v1',
      metadata: {
        name: 'pod-a',
        namespace: 'default',
        creationTimestamp: '2023-01-01T00:00:00Z',
        uid: 'pod-a-uid'
      },
      podSpec: {
        nodeName: 'node-1',
        containers: [{ name: 'container1' }]
      },
      podStatus: {
        phase: 'Pending',
        containerStatuses: [{ restartCount: 1 }]
      }
    },
    {
      kind: 'Pod',
      apiVersion: 'v1',
      metadata: {
        name: 'pod-c',
        namespace: 'kube-system',
        creationTimestamp: '2023-01-03T00:00:00Z',
        uid: 'pod-c-uid'
      },
      podSpec: {
        nodeName: 'node-3',
        containers: [{ name: 'container1' }]
      },
      podStatus: {
        phase: 'Failed',
        containerStatuses: [{ restartCount: 0 }]
      }
    }
  ]

  it('should sort by name in ascending order by default', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Check initial sort (should be by name ascending)
    const sortedItems = wrapper.vm.filteredItems
    expect(sortedItems[0].metadata?.name).toBe('pod-a')
    expect(sortedItems[1].metadata?.name).toBe('pod-b')
    expect(sortedItems[2].metadata?.name).toBe('pod-c')
  })

  it('should change sort direction when clicking the same column', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Initial state should be ascending by name
    expect(wrapper.vm.sortColumn).toBe('name')
    expect(wrapper.vm.sortDirection).toBe('asc')

    // Click name header again to reverse sort
    wrapper.vm.handleSort('name')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.sortDirection).toBe('desc')
    
    // Check that items are now in descending order
    const sortedItems = wrapper.vm.filteredItems
    expect(sortedItems[0].metadata?.name).toBe('pod-c')
    expect(sortedItems[1].metadata?.name).toBe('pod-b')
    expect(sortedItems[2].metadata?.name).toBe('pod-a')
  })

  it('should sort by different columns', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Sort by namespace
    wrapper.vm.handleSort('namespace')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.sortColumn).toBe('namespace')
    expect(wrapper.vm.sortDirection).toBe('asc')

    const sortedByNamespace = wrapper.vm.filteredItems
    // 'default' comes before 'kube-system' alphabetically
    expect(sortedByNamespace[0].metadata?.namespace).toBe('default')
    expect(sortedByNamespace[2].metadata?.namespace).toBe('kube-system')
  })

  it('should sort by age (creation timestamp)', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Sort by age (ascending = oldest first)
    wrapper.vm.handleSort('age')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.sortColumn).toBe('age')
    const sortedByAge = wrapper.vm.filteredItems
    // pod-a was created first (2023-01-01)
    expect(sortedByAge[0].metadata?.name).toBe('pod-a')
    // pod-c was created last (2023-01-03)
    expect(sortedByAge[2].metadata?.name).toBe('pod-c')
  })

  it('should sort by containers column (Pod-specific)', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Sort by restarts (Pod-specific column)
    wrapper.vm.handleSort('restarts')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.sortColumn).toBe('restarts')
    const sortedByRestarts = wrapper.vm.filteredItems
    // Sorted by restart count: 0, 1, 2
    expect(wrapper.vm.getTotalRestartCount(sortedByRestarts[0])).toBe(0)
    expect(wrapper.vm.getTotalRestartCount(sortedByRestarts[1])).toBe(1)
    expect(wrapper.vm.getTotalRestartCount(sortedByRestarts[2])).toBe(2)
  })

  it('should maintain sort when filtering', async () => {
    const wrapper = mount(ResourceList, {
      props: {
        resource: mockResource,
        items: mockPods,
        loading: false,
        namespaces: ['default', 'kube-system'],
        selectedNamespaces: ['default', 'kube-system']
      }
    })

    // Initial state: sortColumn='name', sortDirection='asc'
    expect(wrapper.vm.sortColumn).toBe('name')
    expect(wrapper.vm.sortDirection).toBe('asc')

    // First click on 'name' should toggle to desc (since it's already the current column)
    wrapper.vm.handleSort('name')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.sortColumn).toBe('name')
    expect(wrapper.vm.sortDirection).toBe('desc')

    // Apply filter that matches all pods
    wrapper.vm.filterText = 'pod-'
    await wrapper.vm.$nextTick()

    // Should still be sorted desc by name even with filter
    const filteredAndSorted = wrapper.vm.filteredItems
    // When sorted desc by name: pod-c, pod-b, pod-a
    expect(filteredAndSorted[0].metadata?.name).toBe('pod-c')
    expect(filteredAndSorted[1].metadata?.name).toBe('pod-b')  
    expect(filteredAndSorted[2].metadata?.name).toBe('pod-a')
  })
})