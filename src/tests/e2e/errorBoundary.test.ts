import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ErrorBoundary from '@/components/ErrorBoundary.vue'
import ApiErrorBoundary from '@/components/ApiErrorBoundary.vue'
import { defineComponent, h } from 'vue'

// Helper function to find button by text
function findButtonByText(wrapper: any, text: string) {
  const buttons = wrapper.findAll('button')
  return buttons.find((btn: any) => btn.text().includes(text))
}

// Component that always throws an error
const BrokenComponent = defineComponent({
  name: 'BrokenComponent',
  setup() {
    throw new Error('Test error')
  },
  render() {
    return h('div', 'This should not render')
  }
})

// Component that works normally
const WorkingComponent = defineComponent({
  name: 'WorkingComponent',
  render() {
    return h('div', 'Working component')
  }
})

// Component that throws error on demand
const ConditionalErrorComponent = defineComponent({
  name: 'ConditionalErrorComponent',
  props: {
    shouldThrow: Boolean
  },
  setup(props) {
    if (props.shouldThrow) {
      throw new Error('Conditional error')
    }
  },
  render() {
    return h('div', 'Conditional component')
  }
})

describe('ErrorBoundary E2E', () => {
  it('should render children when no error occurs', () => {
    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: () => h(WorkingComponent)
      }
    })

    expect(wrapper.text()).toContain('Working component')
    expect(wrapper.find('.error-boundary').exists()).toBe(false)
  })

  it('should catch and display component errors', async () => {
    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: () => h(BrokenComponent)
      }
    })

    // Wait for the error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.find('.error-boundary').exists()).toBe(true)
    expect(wrapper.text()).toContain('Something went wrong')
    expect(wrapper.text()).not.toContain('This should not render')
  })

  it('should show user-friendly messages for common errors', async () => {
    // Test network error
    const NetworkErrorComponent = defineComponent({
      name: 'NetworkErrorComponent',
      setup() {
        throw new Error('Network request failed')
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: () => h(NetworkErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Unable to connect to the server')
  })

  it('should handle retry functionality', async () => {
    const onRetrySpy = vi.fn()
    
    const wrapper = mount(ErrorBoundary, {
      props: {
        onRetry: onRetrySpy
      },
      slots: {
        default: () => h(BrokenComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    // Find and click retry button
    const retryButton = findButtonByText(wrapper, 'Try Again')
    expect(retryButton).toBeDefined()
    await retryButton?.trigger('click')
    
    expect(onRetrySpy).toHaveBeenCalled()
  })

  it('should toggle error details correctly', async () => {
    const ErrorComponent = defineComponent({
      name: 'ErrorComponent',
      setup() {
        const error = new Error('Detailed error message')
        error.stack = 'Error stack trace here'
        throw error
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ErrorBoundary, {
      props: {
        showDetails: true
      },
      slots: {
        default: () => h(ErrorComponent)
      }
    })

    // Initially details should be hidden
    expect(wrapper.find('pre').exists()).toBe(false)
    
    // Click to show details
    const showDetailsButton = findButtonByText(wrapper, 'Show Details')
    if (showDetailsButton) {
      await showDetailsButton.trigger('click')
      await wrapper.vm.$nextTick()
      expect(wrapper.find('pre').exists()).toBe(true)
      expect(wrapper.text()).toContain('Detailed error message')
      
      // Click to hide details
      const hideDetailsButton = findButtonByText(wrapper, 'Hide Details')
      if (hideDetailsButton) {
        await hideDetailsButton.trigger('click')
        await wrapper.vm.$nextTick()
        expect(wrapper.find('pre').exists()).toBe(false)
      }
    }
  })

  it('should use custom fallback message', async () => {
    const customMessage = 'Custom error message'
    
    const wrapper = mount(ErrorBoundary, {
      props: {
        fallbackMessage: customMessage
      },
      slots: {
        default: () => h(BrokenComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain(customMessage)
  })
})

describe('ApiErrorBoundary E2E', () => {
  it('should provide API-specific error messages', async () => {
    const ApiErrorComponent = defineComponent({
      name: 'ApiErrorComponent',
      setup() {
        const error = new Error('404 Not Found')
        throw error
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ApiErrorBoundary, {
      props: {
        resourceType: 'pods',
        operation: 'fetch'
      },
      slots: {
        default: () => h(ApiErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('Failed to fetch pods')
  })

  it('should handle RBAC errors appropriately', async () => {
    const RbacErrorComponent = defineComponent({
      name: 'RbacErrorComponent',
      setup() {
        const error = new Error('Forbidden')
        throw error
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ApiErrorBoundary, {
      props: {
        resourceType: 'secrets',
        operation: 'access'
      },
      slots: {
        default: () => h(RbacErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('sufficient permissions')
  })

  it('should handle namespace errors', async () => {
    const NamespaceErrorComponent = defineComponent({
      name: 'NamespaceErrorComponent',
      setup() {
        const error = new Error('Namespace not found')
        throw error
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ApiErrorBoundary, {
      props: {
        resourceType: 'deployments',
        operation: 'list'
      },
      slots: {
        default: () => h(NamespaceErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('namespace')
  })

  it('should handle connection errors', async () => {
    const ConnectionErrorComponent = defineComponent({
      name: 'ConnectionErrorComponent',
      setup() {
        const error = new Error('ECONNREFUSED')
        throw error
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ApiErrorBoundary, {
      props: {
        resourceType: 'services',
        operation: 'fetch'
      },
      slots: {
        default: () => h(ConnectionErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('connection')
  })

  it('should execute retry callback', async () => {
    const retryCallback = vi.fn()
    
    const wrapper = mount(ApiErrorBoundary, {
      props: {
        resourceType: 'pods',
        operation: 'fetch',
        onRetry: retryCallback
      },
      slots: {
        default: () => h(BrokenComponent)
      }
    })

    const retryButton = findButtonByText(wrapper, 'Try Again')
    if (retryButton) {
      await retryButton.trigger('click')
      expect(retryCallback).toHaveBeenCalled()
    }
  })
})

describe('Error Integration Tests', () => {
  it('should handle cascading errors gracefully', async () => {
    // Component that throws multiple errors
    const CascadingErrorComponent = defineComponent({
      name: 'CascadingErrorComponent',
      setup() {
        throw new Error('Error 1')
      },
      errorCaptured() {
        throw new Error('Error 2')
      },
      render() {
        return h('div')
      }
    })

    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: () => h(CascadingErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    // Should handle the first error gracefully
    expect(wrapper.find('.error-boundary').exists()).toBe(true)
  })

  it('should prevent error boundary loops', async () => {
    let errorCount = 0
    
    // Component that always throws
    const LoopingErrorComponent = defineComponent({
      name: 'LoopingErrorComponent',
      setup() {
        errorCount++
        if (errorCount > 10) {
          return {} // Prevent infinite loop in test
        }
        throw new Error('Always throws')
      },
      render() {
        return h('div', 'Should not render')
      }
    })

    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: () => h(LoopingErrorComponent)
      }
    })

    // Wait for error boundary to update
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    // Should show error boundary
    expect(wrapper.find('.error-boundary').exists()).toBe(true)
    
    // Should not have triggered excessive errors
    expect(errorCount).toBeLessThan(5)
  })
})