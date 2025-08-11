import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceList from '../../components/ResourceList.vue'

describe('NetworkPolicy Policy Type Column', () => {
  const networkPolicyResource = {
    name: 'NetworkPolicies',
    namespaced: true,
    apiVersion: 'networking.k8s.io/v1',
    kind: 'NetworkPolicy'
  }

  it('should show Ingress for ingress-only policy', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'ingress-only-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        policyTypes: ['Ingress'],
        ingress: [{ from: [{}] }]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Ingress')
  })

  it('should show Egress for egress-only policy', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'egress-only-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        policyTypes: ['Egress'],
        egress: [{ to: [{}] }]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Egress')
  })

  it('should show Both for policies with both ingress and egress', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'both-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        policyTypes: ['Ingress', 'Egress'],
        ingress: [{ from: [{}] }],
        egress: [{ to: [{}] }]
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Both')
  })

  it('should show Ingress for default behavior (no policyTypes, no egress)', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'default-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        ingress: [{ from: [{}] }]
        // No policyTypes specified, no egress rules
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Ingress')
  })

  it('should show Both for default behavior with egress rules', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'default-with-egress-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        ingress: [{ from: [{}] }],
        egress: [{ to: [{}] }]
        // No policyTypes specified, but has egress rules
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Both')
  })

  it('should display Policy Type column header', () => {
    const networkPolicyItems = [{
      kind: 'NetworkPolicy',
      metadata: {
        name: 'test-policy',
        uid: 'netpol-123',
        namespace: 'default',
        creationTimestamp: '2025-08-04T10:00:00Z'
      },
      apiVersion: 'networking.k8s.io/v1',
      networkPolicySpec: {
        podSelector: { matchLabels: { app: 'web' } },
        policyTypes: ['Ingress']
      }
    }]

    const wrapper = mount(ResourceList, {
      props: {
        resource: networkPolicyResource,
        items: networkPolicyItems,
        loading: false
      }
    })

    expect(wrapper.text()).toContain('Policy Type')
  })
})