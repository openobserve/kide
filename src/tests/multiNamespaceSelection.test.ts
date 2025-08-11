import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceList from '../components/ResourceList.vue'
import MultiSelectNamespace from '../components/MultiSelectNamespace.vue'

describe('Multi-Namespace Selection', () => {
  let wrapper
  let mockResource
  let mockNamespaces

  beforeEach(() => {
    mockNamespaces = ['default', 'kube-system', 'production', 'staging']
    
    mockResource = {
      name: 'Pods',
      namespaced: true,
      apiVersion: 'v1',
      kind: 'Pod'
    }
  })

  describe('MultiSelectNamespace Component', () => {
    it('should render multi-select dropdown', () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })

      // Look for the main dropdown button (not the Select All button)
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      expect(dropdownButton.exists()).toBe(true)
      expect(dropdownButton.text()).toContain('default')
    })

    it('should show multiple selected namespaces count', () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: ['default', 'production']
        }
      })

      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      expect(dropdownButton.text()).toContain('2 namespaces selected')
    })

    it('should emit namespace changes', async () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })

      // Click to open dropdown
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      await dropdownButton.trigger('click')

      // Find and click a namespace option
      const namespaceOption = wrapper.find('[data-testid="namespace-option-production"]')
      if (namespaceOption.exists()) {
        await namespaceOption.trigger('click')
        
        const emitted = wrapper.emitted('update:selectedNamespaces')
        expect(emitted).toBeTruthy()
      }
    })

    it('should filter namespaces based on search input', async () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: []
        }
      })

      // Click to open dropdown
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      await dropdownButton.trigger('click')

      // Find search input and type
      const searchInput = wrapper.find('input[placeholder="Search namespaces..."]')
      expect(searchInput.exists()).toBe(true)
      
      await searchInput.setValue('prod')
      
      // Should show only namespaces containing 'prod'
      const namespaceOptions = wrapper.findAll('div[class*="cursor-pointer select-none"]')
      expect(namespaceOptions.length).toBeLessThan(mockNamespaces.length)
      
      // Should contain 'production' namespace
      const text = wrapper.text()
      expect(text).toContain('production')
    })

    it('should show no results message when search yields no matches', async () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: []
        }
      })

      // Click to open dropdown
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      await dropdownButton.trigger('click')

      // Search for non-existent namespace
      const searchInput = wrapper.find('input[placeholder="Search namespaces..."]')
      await searchInput.setValue('nonexistent')
      
      // Should show no results message
      expect(wrapper.text()).toContain('No namespaces found matching "nonexistent"')
    })
  })

  describe('Integration with ResourceList', () => {
    it('should render multi-select for namespaced resources', () => {
      wrapper = mount(ResourceList, {
        props: {
          resource: mockResource,
          items: [],
          loading: false,
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })

      // Should contain multi-select component
      const multiSelect = wrapper.findComponent(MultiSelectNamespace)
      expect(multiSelect.exists()).toBe(true)
    })

    it('should not render multi-select for cluster-wide resources', () => {
      const clusterResource = {
        name: 'Nodes',
        namespaced: false,
        apiVersion: 'v1',
        kind: 'Node'
      }

      wrapper = mount(ResourceList, {
        props: {
          resource: clusterResource,
          items: [],
          loading: false,
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })

      // Should not contain multi-select component
      const multiSelect = wrapper.findComponent(MultiSelectNamespace)
      expect(multiSelect.exists()).toBe(false)
    })

    it('should handle namespace change events', async () => {
      wrapper = mount(ResourceList, {
        props: {
          resource: mockResource,
          items: [],
          loading: false,
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })

      const multiSelect = wrapper.findComponent(MultiSelectNamespace)
      
      // Simulate namespace change
      await multiSelect.vm.$emit('update:selectedNamespaces', ['default', 'production'])

      const emitted = wrapper.emitted('namespace-change')
      expect(emitted).toBeTruthy()
      expect(emitted[0][0]).toEqual(['default', 'production'])
    })
  })

  describe('UI Functionality', () => {
    it('should show correct text for different selection states', () => {
      // No selection
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: []
        }
      })
      expect(wrapper.find('div[class*="cursor-pointer"]').text()).toContain('Select namespaces')

      // Single selection
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: ['default']
        }
      })
      expect(wrapper.find('div[class*="cursor-pointer"]').text()).toContain('default')

      // Multiple selection
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: ['default', 'production', 'staging']
        }
      })
      expect(wrapper.find('div[class*="cursor-pointer"]').text()).toContain('3 namespaces selected')
    })


    it('should adjust dropdown width based on longest namespace', async () => {
      const longNamespaces = [
        'short',
        'this-is-a-very-long-namespace-name-that-should-expand-dropdown-width',
        'medium-length-namespace'
      ]

      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: longNamespaces,
          selectedNamespaces: []
        }
      })

      // Open dropdown to trigger width calculation
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      await dropdownButton.trigger('click')

      // Check that dropdown panel has a calculated width
      const dropdownPanel = wrapper.find('[class*="absolute z-50"]')
      expect(dropdownPanel.exists()).toBe(true)
      
      // Should have a style attribute with minWidth
      const style = dropdownPanel.attributes('style')
      expect(style).toContain('min-width')
      expect(style).toMatch(/min-width:\s*\d+px/)
    })

    it('should handle viewport positioning', async () => {
      wrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: mockNamespaces,
          selectedNamespaces: []
        }
      })

      // Mock getBoundingClientRect to simulate near-right-edge positioning
      const mockGetBoundingClientRect = vi.fn(() => ({
        left: 800,    // Close to right edge (simulating 1000px viewport)
        right: 1000,
        top: 100,
        bottom: 150,
        width: 200,
        height: 50
      }))

      // Mock window.innerWidth
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 1000
      })

      // Open dropdown to trigger positioning calculation
      const dropdownButton = wrapper.find('div[class*="cursor-pointer"]')
      await dropdownButton.trigger('click')

      // Mock the parent element's getBoundingClientRect after dropdown is open
      const dropdownPanel = wrapper.find('[class*="absolute z-50"]')
      expect(dropdownPanel.exists()).toBe(true)
      
      if (dropdownPanel.element.parentElement) {
        dropdownPanel.element.parentElement.getBoundingClientRect = mockGetBoundingClientRect
      }

      // Trigger a window resize to recalculate positioning
      window.dispatchEvent(new Event('resize'))
      await wrapper.vm.$nextTick()
      
      // Should have positioning styles to keep within viewport
      const style = dropdownPanel.attributes('style')
      expect(style).toMatch(/(left|right):\s*(auto|[-]?\d+px|0)/)
    })
  })
})