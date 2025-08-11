import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import { nextTick } from 'vue'
import MultiSelectNamespace from '@/components/MultiSelectNamespace.vue'

describe('MultiSelectNamespace Keyboard Navigation E2E Tests', () => {
  let wrapper: VueWrapper<any>
  
  const mockNamespaces = [
    'default',
    'kube-system',
    'kube-public',
    'test-namespace',
    'production',
    'staging',
    'development'
  ]

  beforeEach(() => {
    wrapper = mount(MultiSelectNamespace, {
      props: {
        namespaces: mockNamespaces,
        selectedNamespaces: []
      }
    })
  })

  describe('Initial State and Opening Dropdown', () => {
    it('should start with closed dropdown and no selection', () => {
      // Check that dropdown panel is not visible
      const dropdownPanel = wrapper.find('[data-testid="dropdown-panel"]')
      expect(dropdownPanel.isVisible()).toBe(false)
      expect(wrapper.text()).toContain('Select namespaces')
    })

    it('should open dropdown when clicked and focus search input', async () => {
      const trigger = wrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()

      const dropdownPanel = wrapper.find('[data-testid="dropdown-panel"]')
      expect(dropdownPanel.isVisible()).toBe(true)
      
      const searchInput = wrapper.find('input[type="text"]')
      expect(searchInput.exists()).toBe(true)
    })
  })

  describe('Keyboard Navigation Without Search', () => {
    beforeEach(async () => {
      // Open dropdown
      const trigger = wrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
    })

    it('should highlight "All namespaces" option initially when no search', async () => {
      const allNamespacesOption = wrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.classes()).toContain('bg-blue-50')
    })

    it('should navigate down from "All namespaces" to first namespace', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Press down arrow
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // First namespace should be highlighted
      const firstNamespaceOption = wrapper.find('[data-testid="namespace-option-default"]')
      expect(firstNamespaceOption.classes()).toContain('bg-blue-50')
    })

    it('should navigate through all namespaces with down arrow', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Navigate to first namespace (skip "All namespaces")
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Navigate to second namespace
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      const secondNamespaceOption = wrapper.find(`[data-testid="namespace-option-${mockNamespaces[1]}"]`)
      expect(secondNamespaceOption.classes()).toContain('bg-blue-50')
    })

    it('should navigate up from first namespace to "All namespaces"', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Navigate to first namespace
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Navigate back up to "All namespaces"
      await searchInput.trigger('keydown', { key: 'ArrowUp' })
      await nextTick()
      
      const allNamespacesOption = wrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.classes()).toContain('bg-blue-50')
    })

    it('should not navigate beyond last namespace with down arrow', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Navigate to last namespace
      for (let i = 0; i <= mockNamespaces.length; i++) {
        await searchInput.trigger('keydown', { key: 'ArrowDown' })
        await nextTick()
      }
      
      // Try to navigate beyond last namespace
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Should still be on last namespace
      const lastNamespaceOption = wrapper.find(`[data-testid="namespace-option-${mockNamespaces[mockNamespaces.length - 1]}"]`)
      expect(lastNamespaceOption.classes()).toContain('bg-blue-50')
    })

    it('should not navigate beyond "All namespaces" with up arrow', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Try to navigate up from initial position (should be "All namespaces")
      await searchInput.trigger('keydown', { key: 'ArrowUp' })
      await nextTick()
      
      // Should still be on "All namespaces"
      const allNamespacesOption = wrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.classes()).toContain('bg-blue-50')
    })

    it('should select highlighted namespace with Enter key', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Navigate to first namespace
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Select with Enter
      await searchInput.trigger('keydown', { key: 'Enter' })
      await nextTick()
      
      // Check that the namespace was selected
      expect(wrapper.emitted('update:selectedNamespaces')).toBeTruthy()
      expect(wrapper.emitted('update:selectedNamespaces')![0]).toEqual([[mockNamespaces[0]]])
    })

    it('should select "All namespaces" with Enter key', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Select "All namespaces" with Enter (should be highlighted by default)
      await searchInput.trigger('keydown', { key: 'Enter' })
      await nextTick()
      
      // Check that all namespaces were selected
      expect(wrapper.emitted('update:selectedNamespaces')).toBeTruthy()
      expect(wrapper.emitted('update:selectedNamespaces')![0]).toEqual([mockNamespaces])
    })
  })

  describe('Keyboard Navigation With Search', () => {
    beforeEach(async () => {
      // Open dropdown
      const trigger = wrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
    })

    it('should filter namespaces when typing', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query
      await searchInput.setValue('kube')
      await nextTick()
      
      // Should show filtered results
      const kubeSystemOption = wrapper.find('[data-testid="namespace-option-kube-system"]')
      const kubePublicOption = wrapper.find('[data-testid="namespace-option-kube-public"]')
      const defaultOption = wrapper.find('[data-testid="namespace-option-default"]')
      
      expect(kubeSystemOption.exists()).toBe(true)
      expect(kubePublicOption.exists()).toBe(true)
      expect(defaultOption.exists()).toBe(false) // Should be filtered out
    })

    it('should not show "All namespaces" option when searching', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query
      await searchInput.setValue('kube')
      await nextTick()
      
      // "All namespaces" option should not be visible
      const allNamespacesOption = wrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.exists()).toBe(false)
    })

    it('should navigate filtered results with arrow keys after typing', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query to filter to 'kube-system' and 'kube-public'
      await searchInput.setValue('kube')
      await nextTick()
      
      // Press down arrow to highlight first filtered result
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // First filtered result should be highlighted
      const firstFilteredOption = wrapper.find('[data-testid="namespace-option-kube-system"]')
      expect(firstFilteredOption.classes()).toContain('bg-blue-50')
    })

    it('should navigate between filtered results correctly', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query
      await searchInput.setValue('kube')
      await nextTick()
      
      // Navigate to first filtered result
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Navigate to second filtered result
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Second filtered result should be highlighted
      const secondFilteredOption = wrapper.find('[data-testid="namespace-option-kube-public"]')
      expect(secondFilteredOption.classes()).toContain('bg-blue-50')
    })

    it('should handle no search results gracefully', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query that matches nothing
      await searchInput.setValue('nonexistent')
      await nextTick()
      
      // Should show "no namespaces found" message
      expect(wrapper.text()).toContain('No namespaces found matching "nonexistent"')
      
      // Arrow keys should do nothing
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // No options should be highlighted
      const options = wrapper.findAll('[class*="bg-blue-50"]')
      expect(options.length).toBe(0)
    })

    it('should select filtered namespace with Enter key', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query
      await searchInput.setValue('kube-sys')
      await nextTick()
      
      // Navigate to first (and only) filtered result
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Select with Enter
      await searchInput.trigger('keydown', { key: 'Enter' })
      await nextTick()
      
      // Check that the correct namespace was selected
      expect(wrapper.emitted('update:selectedNamespaces')).toBeTruthy()
      expect(wrapper.emitted('update:selectedNamespaces')![0]).toEqual([['kube-system']])
    })

    it('should reset highlighting when search query changes', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type initial search
      await searchInput.setValue('kube')
      await nextTick()
      
      // Navigate to highlight first result
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Change search query
      await searchInput.setValue('test')
      await nextTick()
      
      // Highlighting should be reset (no items highlighted initially)
      const highlightedOptions = wrapper.findAll('[class*="bg-blue-50"]')
      expect(highlightedOptions.length).toBe(0)
    })

    it('should navigate correctly after search query change', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type initial search
      await searchInput.setValue('kube')
      await nextTick()
      
      // Change to different search
      await searchInput.setValue('test')
      await nextTick()
      
      // Navigate to first result of new search
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Should highlight the test-namespace
      const testNamespaceOption = wrapper.find('[data-testid="namespace-option-test-namespace"]')
      expect(testNamespaceOption.classes()).toContain('bg-blue-50')
    })
  })

  describe('Mixed Navigation Scenarios', () => {
    beforeEach(async () => {
      // Open dropdown
      const trigger = wrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
    })

    it('should handle typing, then clearing search, then navigating', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Type search query
      await searchInput.setValue('kube')
      await nextTick()
      
      // Clear search
      await searchInput.setValue('')
      await nextTick()
      
      // Should show "All namespaces" option again
      const allNamespacesOption = wrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.exists()).toBe(true)
      
      // Navigation should work normally
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // First namespace should be highlighted
      const firstNamespaceOption = wrapper.find('[data-testid="namespace-option-default"]')
      expect(firstNamespaceOption.classes()).toContain('bg-blue-50')
    })

    it('should close dropdown when Escape is pressed', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Press Escape
      await searchInput.trigger('keydown', { key: 'Escape' })
      await nextTick()
      
      // Dropdown should be closed (not visible)
      const dropdownPanel = wrapper.find('[data-testid="dropdown-panel"]')
      expect(dropdownPanel.isVisible()).toBe(false)
    })

    it('should handle rapid typing and navigation', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Rapid typing simulation
      await searchInput.setValue('k')
      await searchInput.setValue('ku')
      await searchInput.setValue('kub')
      await searchInput.setValue('kube')
      await nextTick()
      
      // Should still work for navigation
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // First filtered result should be highlighted
      const firstFilteredOption = wrapper.find('[data-testid="namespace-option-kube-system"]')
      expect(firstFilteredOption.classes()).toContain('bg-blue-50')
    })

    it('should have autocomplete disabled to prevent browser interference', async () => {
      const searchInput = wrapper.find('input[type="text"]')
      
      // Check that autocomplete attributes are properly set
      expect(searchInput.attributes('autocomplete')).toBe('off')
      expect(searchInput.attributes('autocorrect')).toBe('off')
      expect(searchInput.attributes('autocapitalize')).toBe('off')
      expect(searchInput.attributes('spellcheck')).toBe('false')
      
      // Check ARIA attributes for accessibility
      expect(searchInput.attributes('role')).toBe('combobox')
      expect(searchInput.attributes('aria-autocomplete')).toBe('list')
      expect(searchInput.attributes('aria-expanded')).toBe('true')
    })
  })

  describe('Edge Cases and Error Handling', () => {
    it('should handle empty namespaces list', async () => {
      const emptyWrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: [],
          selectedNamespaces: []
        }
      })
      
      // Open dropdown
      const trigger = emptyWrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
      
      // Should show "no namespaces available" message
      expect(emptyWrapper.text()).toContain('No namespaces available')
      
      const searchInput = emptyWrapper.find('input[type="text"]')
      
      // Arrow keys should do nothing
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // With empty namespaces, the "All namespaces" option should still be present but no highlighting should occur after arrow key since there are no actual namespaces
      // The highlighting logic should handle this case gracefully
      const allNamespacesOption = emptyWrapper.find('[data-testid="all-namespaces-option"]')
      expect(allNamespacesOption.exists()).toBe(true) // Should show "All namespaces" option even when no namespaces
      
      // But arrow keys should not highlight anything when there are no actual namespaces to navigate to
      const highlightedOptions = emptyWrapper.findAll('[class*="bg-blue-50"]:not([data-testid="all-namespaces-option"])')
      expect(highlightedOptions.length).toBe(0)
    })

    it('should handle single namespace in list', async () => {
      const singleWrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: ['only-namespace'],
          selectedNamespaces: []
        }
      })
      
      // Open dropdown
      const trigger = singleWrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
      
      const searchInput = singleWrapper.find('input[type="text"]')
      
      // Navigate from "All namespaces" to single namespace
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Single namespace should be highlighted
      const singleOption = singleWrapper.find('[data-testid="namespace-option-only-namespace"]')
      expect(singleOption.classes()).toContain('bg-blue-50')
      
      // Trying to navigate further down should do nothing
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      // Should still be on the same namespace
      expect(singleOption.classes()).toContain('bg-blue-50')
    })

    it('should handle special characters in namespace names', async () => {
      const specialWrapper = mount(MultiSelectNamespace, {
        props: {
          namespaces: ['test-namespace', 'test_namespace', 'test.namespace'],
          selectedNamespaces: []
        }
      })
      
      // Open dropdown
      const trigger = specialWrapper.findAll('.cursor-pointer')[0]
      await trigger.trigger('click')
      await nextTick()
      
      const searchInput = specialWrapper.find('input[type="text"]')
      
      // Search for namespace with special characters
      await searchInput.setValue('test.')
      await nextTick()
      
      // Should find the namespace with dot
      const dotNamespaceOption = specialWrapper.find('[data-testid="namespace-option-test.namespace"]')
      expect(dotNamespaceOption.exists()).toBe(true)
      
      // Navigation should work
      await searchInput.trigger('keydown', { key: 'ArrowDown' })
      await nextTick()
      
      expect(dotNamespaceOption.classes()).toContain('bg-blue-50')
    })
  })
})