import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceNavigation from '@/components/ResourceNavigation.vue'
import type { K8sResourceCategory } from '@/types'

describe('ResourceNavigation', () => {
  const mockCategories: K8sResourceCategory[] = [
    {
      name: 'Test Category',
      resources: [
        {
          name: 'TestResource',
          apiVersion: 'v1',
          kind: 'TestResource',
          namespaced: true,
          description: 'Test description'
        }
      ]
    }
  ]

  it('displays resource count in footer', () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: null,
        connected: false,
        connectionStatus: 'connecting'
      }
    })

    expect(wrapper.text()).toContain('1 resources')
    expect(wrapper.find('.border-t').exists()).toBe(true)
  })

  it('displays current time in footer', async () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: null,
        connected: true,
        connectionStatus: 'connected'
      }
    })

    // Wait for component to initialize and update time
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    // Should contain time display (format: HH:MM)
    expect(wrapper.text()).toMatch(/\d{2}:\d{2}/)
  })

  it('shows proper namespace/cluster scope indicators', async () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: null,
        connected: false,
        connectionStatus: 'failed'
      }
    })

    // Expand the category first to show the resource
    await wrapper.find('[data-testid="category-header"]').trigger('click')
    
    // Should show NS for namespaced resources
    expect(wrapper.text()).toContain('NS')
  })

  it('renders categories and resources', () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: null,
        connected: true,
        connectionStatus: 'connected'
      }
    })

    expect(wrapper.text()).toContain('Test Category')
    expect(wrapper.text()).toContain('TestResource')
  })

  it('emits select-resource event when resource is clicked', async () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: null,
        connected: true,
        connectionStatus: 'connected'
      }
    })

    // Click on the category to expand it first
    await wrapper.find('[data-testid="category-header"]').trigger('click')
    
    // Then click on the resource
    await wrapper.find('[data-testid="resource-item"]').trigger('click')

    expect(wrapper.emitted('select-resource')).toBeTruthy()
    expect(wrapper.emitted('select-resource')?.[0]).toEqual([mockCategories[0].resources[0]])
  })

  it('highlights selected resource', async () => {
    const wrapper = mount(ResourceNavigation, {
      props: {
        categories: mockCategories,
        selectedResource: mockCategories[0].resources[0],
        connected: true,
        connectionStatus: 'connected'
      }
    })

    // Category needs to be expanded first to show the selected resource
    await wrapper.find('[data-testid="category-header"]').trigger('click')
    
    // The selected resource should have bg-blue-100 class according to the component
    expect(wrapper.find('.bg-blue-100').exists()).toBe(true)
  })
})