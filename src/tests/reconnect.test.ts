import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyState from '../components/EmptyState.vue'
import type { K8sContext } from '../types'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Reconnect Functionality', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should show reconnect button when connection fails', async () => {
    const mockContext: K8sContext = {
      name: 'test-cluster',
      cluster: 'test-cluster',
      user: 'test-user',
      namespace: 'default'
    }

    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed',
        connectionError: 'Connection refused by cluster',
        hadSelectedResource: false
      }
    })

    // Should show connection failed status
    expect(wrapper.text()).toContain('Failed to connect to test-cluster')
    expect(wrapper.text()).toContain('Connection refused by cluster')

    // Should show reconnect button
    const buttons = wrapper.findAll('button')
    const reconnectButton = buttons.find(button => button.text().includes('Reconnect'))
    expect(reconnectButton).toBeDefined()
    expect(reconnectButton?.text()).toContain('Reconnect')
  })

  it('should emit reconnect event when button is clicked', async () => {
    const mockContext: K8sContext = {
      name: 'test-cluster',
      cluster: 'test-cluster', 
      user: 'test-user',
      namespace: 'default'
    }

    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed',
        connectionError: 'Connection timed out',
        hadSelectedResource: true
      }
    })

    const buttons = wrapper.findAll('button')
    const reconnectButton = buttons.find(button => button.text().includes('Reconnect'))
    expect(reconnectButton).toBeDefined()
    
    await reconnectButton?.trigger('click')

    // Should emit reconnect event
    expect(wrapper.emitted('reconnect')).toBeTruthy()
    expect(wrapper.emitted('reconnect')).toHaveLength(1)
  })

  it('should show connecting state when reconnecting', async () => {
    const mockContext: K8sContext = {
      name: 'prod-cluster',
      cluster: 'prod-cluster',
      user: 'admin-user', 
      namespace: 'kube-system'
    }

    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'failed',
        connectionError: 'Network error',
        hadSelectedResource: false
      }
    })

    const buttons = wrapper.findAll('button')
    const reconnectButton = buttons.find(button => button.text().includes('Reconnect'))
    expect(reconnectButton).toBeDefined()
    
    // Click reconnect button
    await reconnectButton?.trigger('click')

    // Button should show reconnecting state
    expect(reconnectButton?.text()).toContain('Reconnecting...')
    expect(reconnectButton?.attributes('disabled')).toBeDefined()
  })

  it('should not show reconnect button when connected', () => {
    const mockContext: K8sContext = {
      name: 'working-cluster',
      cluster: 'working-cluster',
      user: 'user',
      namespace: 'default'
    }

    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'connected',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    // Should show connected status
    expect(wrapper.text()).toContain('Connected to working-cluster')

    // Should not show reconnect button
    const buttons = wrapper.findAll('button')
    const reconnectButton = buttons.find(button => button.text().includes('Reconnect'))
    expect(reconnectButton).toBeUndefined()
  })

  it('should not show reconnect button when connecting', () => {
    const mockContext: K8sContext = {
      name: 'connecting-cluster',
      cluster: 'connecting-cluster',
      user: 'user',
      namespace: 'default'
    }

    const wrapper = mount(EmptyState, {
      props: {
        selectedContext: mockContext,
        connectionStatus: 'connecting',
        connectionError: null,
        hadSelectedResource: false
      }
    })

    // Should show connecting status
    expect(wrapper.text()).toContain('Connecting to connecting-cluster')

    // Should not show reconnect button
    const buttons = wrapper.findAll('button')
    const reconnectButton = buttons.find(button => button.text().includes('Reconnect'))
    expect(reconnectButton).toBeUndefined()
  })
})