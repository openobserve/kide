import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../../components/EmptyState.vue'

describe('EmptyState Error Display', () => {
  it('should show connection failed message when connectionStatus is failed and no selectedContext', () => {
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: 'auth exec command error: token expired',
        hadSelectedResource: false
      }
    })

    // Should show the failed connection indicator
    expect(wrapper.find('.text-red-600')).toBeTruthy()
    expect(wrapper.text()).toContain('Failed to connect to cluster')
    expect(wrapper.find('.bg-red-500')).toBeTruthy() // Red indicator dot
  })

  it('should show connection error details when connectionError is provided', () => {
    const errorMessage = 'auth exec command error: token expired for AWS'
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: errorMessage,
        hadSelectedResource: false
      }
    })

    // Should show the error message in a formatted container
    const errorContainer = wrapper.find('.bg-red-50')
    expect(errorContainer.exists()).toBe(true)
    expect(errorContainer.text()).toContain(errorMessage)
    expect(wrapper.find('pre').text()).toBe(errorMessage)
  })

  it('should show connection failed message with context name when selectedContext is provided', () => {
    const mockContext = { name: 'prod-cluster', cluster: 'prod', user: 'admin' }
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed',
        connectionError: 'connection refused',
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Failed to connect to prod-cluster')
    expect(wrapper.find('.text-red-600')).toBeTruthy()
  })

  it('should show connecting message when connectionStatus is connecting', () => {
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'connecting',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Connecting to cluster...')
    expect(wrapper.find('.text-yellow-600')).toBeTruthy()
    expect(wrapper.find('.animate-pulse')).toBeTruthy() // Pulsing indicator
  })

  it('should show connecting message with context name when selectedContext is provided', () => {
    const mockContext = { name: 'dev-cluster', cluster: 'dev', user: 'user' }
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'connecting',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Connecting to dev-cluster...')
    expect(wrapper.find('.text-yellow-600')).toBeTruthy()
  })

  it('should show connected message when connectionStatus is connected', () => {
    const mockContext = { name: 'local-cluster', cluster: 'local', user: 'user' }
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'connected',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Connected to local-cluster')
    expect(wrapper.find('.text-green-600')).toBeTruthy()
    expect(wrapper.find('.bg-green-500')).toBeTruthy() // Green indicator dot
  })

  it('should show appropriate message based on hadSelectedResource when connection failed', () => {
    // When selectedContext is null, it shows default message regardless of hadSelectedResource
    const wrapper1 = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: 'connection error',
        hadSelectedResource: false
      }
    })

    expect(wrapper1.text()).toContain('Select a cluster context to get started')

    const wrapper2 = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: 'connection error',
        hadSelectedResource: true
      }
    })

    expect(wrapper2.text()).toContain('Select a cluster context to get started')

    // When selectedContext exists, hadSelectedResource affects the message
    const mockContext = { name: 'test-cluster', cluster: 'test', user: 'test' }
    
    const wrapper3 = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed',
        connectionError: 'connection error',
        hadSelectedResource: false
      }
    })

    expect(wrapper3.text()).toContain('Unable to connect to cluster')

    const wrapper4 = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed', 
        connectionError: 'connection error',
        hadSelectedResource: true
      }
    })

    expect(wrapper4.text()).toContain('Lost connection to cluster. Unable to load resources')
  })

  it('should not show connection error details when connectionError is null or empty', () => {
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.find('.bg-red-50').exists()).toBe(false)
    expect(wrapper.find('pre').exists()).toBe(false)
  })

  it('should show default message when no context is selected and connected', () => {
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'connected',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Select a cluster context to get started')
  })

  it('should preserve whitespace and formatting in error messages', () => {
    const multiLineError = `Authentication failed - Credentials are invalid or expired. Please check:
• User credentials in kubeconfig
• Token expiration
• Service account permissions

Original error: auth exec command error`
    
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        connectionStatus: 'failed',
        connectionError: multiLineError,
        hadSelectedResource: false
      }
    })

    const preElement = wrapper.find('pre')
    expect(preElement.exists()).toBe(true)
    expect(preElement.classes()).toContain('whitespace-pre-wrap')
    expect(preElement.text()).toBe(multiLineError)
  })

  it('should show attempted context name when connection fails during context switching', () => {
    const attemptedContext = { name: 'prod-cluster', cluster: 'prod', user: 'admin' }
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        attemptedContext: attemptedContext,
        connectionStatus: 'failed',
        connectionError: 'auth exec command error: token expired',
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Failed to connect to prod-cluster')
    expect(wrapper.find('.text-red-600')).toBeTruthy()
  })

  it('should show attempted context name when connecting', () => {
    const attemptedContext = { name: 'dev-cluster', cluster: 'dev', user: 'user' }
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: null,
        attemptedContext: attemptedContext,
        connectionStatus: 'connecting',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    expect(wrapper.text()).toContain('Connecting to dev-cluster...')
    expect(wrapper.find('.text-yellow-600')).toBeTruthy()
  })

  it('should prefer selectedContext over attemptedContext when both are available', () => {
    const selectedContext = { name: 'current-cluster', cluster: 'current', user: 'user1' }
    const attemptedContext = { name: 'attempted-cluster', cluster: 'attempted', user: 'user2' }
    
    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: selectedContext,
        attemptedContext: attemptedContext,
        connectionStatus: 'failed',
        connectionError: 'connection error',
        hadSelectedResource: false
      }
    })

    // Should use selectedContext name, not attemptedContext
    expect(wrapper.text()).toContain('Failed to connect to current-cluster')
    expect(wrapper.text()).not.toContain('attempted-cluster')
  })
})