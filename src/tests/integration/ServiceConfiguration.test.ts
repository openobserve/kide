import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ServiceConfiguration from '@/components/ResourcePanel/ServiceConfiguration.vue'
import type { K8sListItem } from '@/types'

describe('ServiceConfiguration Integration Tests', () => {
  describe('Component Rendering', () => {
    it('renders basic ClusterIP service configuration', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        sessionAffinity: 'None',
        internalTrafficPolicy: 'Cluster',
        ipFamilies: ['IPv4'],
        ipFamilyPolicy: 'SingleStack',
        ports: [
          {
            port: 4222,
            targetPort: 'nats',
            protocol: 'TCP',
            name: 'nats',
            appProtocol: 'tcp'
          }
        ],
        selector: {
          'app.kubernetes.io/component': 'nats',
          'app.kubernetes.io/instance': 'o2',
          'app.kubernetes.io/name': 'nats'
        }
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      // Check main title
      expect(wrapper.text()).toContain('Service Configuration')

      // Check service type
      expect(wrapper.text()).toContain('Type')
      expect(wrapper.text()).toContain('ClusterIP')

      // Check cluster IP
      expect(wrapper.text()).toContain('Cluster IP')
      expect(wrapper.text()).toContain('172.20.159.8')

      // Check session affinity
      expect(wrapper.text()).toContain('Session Affinity')
      expect(wrapper.text()).toContain('None')

      // Check internal traffic policy
      expect(wrapper.text()).toContain('Internal Traffic Policy')
      expect(wrapper.text()).toContain('Cluster')

      // Check IP families
      expect(wrapper.text()).toContain('IP Families')
      expect(wrapper.text()).toContain('IPv4')

      // Check IP family policy
      expect(wrapper.text()).toContain('IP Family Policy')
      expect(wrapper.text()).toContain('SingleStack')
    })

    it('renders ports configuration correctly', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        ports: [
          {
            port: 80,
            targetPort: 8080,
            protocol: 'TCP',
            name: 'http'
          },
          {
            port: 443,
            targetPort: 8443,
            protocol: 'TCP',
            name: 'https',
            appProtocol: 'https'
          }
        ]
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      // Check ports section
      expect(wrapper.text()).toContain('Ports')
      expect(wrapper.text()).toContain('80:8080/TCP')
      expect(wrapper.text()).toContain('443:8443/TCP')
      expect(wrapper.text()).toContain('(http)')
      expect(wrapper.text()).toContain('(https)')
      expect(wrapper.text()).toContain('[https]')
    })

    it('renders selector as key-value badges', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        selector: {
          'app': 'web-server',
          'version': 'v1.0',
          'environment': 'production'
        }
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      // Check selector section
      expect(wrapper.text()).toContain('Selector')
      expect(wrapper.text()).toContain('app')
      expect(wrapper.text()).toContain('web-server')
      expect(wrapper.text()).toContain('version')
      expect(wrapper.text()).toContain('v1.0')
      expect(wrapper.text()).toContain('environment')
      expect(wrapper.text()).toContain('production')
    })
  })

  describe('Service Types', () => {
    it('renders LoadBalancer service with external configuration', () => {
      const serviceSpec = {
        type: 'LoadBalancer',
        clusterIP: '10.0.0.1',
        allocateLoadBalancerNodePorts: true,
        externalTrafficPolicy: 'Local',
        loadBalancerClass: 'aws-load-balancer-controller'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('LoadBalancer')
      expect(wrapper.text()).toContain('Allocate LoadBalancer NodePorts')
      expect(wrapper.text()).toContain('Yes')
      expect(wrapper.text()).toContain('External Traffic Policy')
      expect(wrapper.text()).toContain('Local')
    })

    it('renders NodePort service correctly', () => {
      const serviceSpec = {
        type: 'NodePort',
        clusterIP: '10.0.0.2',
        ports: [
          {
            port: 80,
            targetPort: 8080,
            nodePort: 30080,
            protocol: 'TCP'
          }
        ]
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('NodePort')
      expect(wrapper.text()).toContain('80:8080/TCP')
    })

    it('renders ExternalName service', () => {
      const serviceSpec = {
        type: 'ExternalName',
        externalName: 'example.com'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('ExternalName')
    })
  })

  describe('IP Configuration', () => {
    it('renders multiple cluster IPs correctly', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8', '2001:db8::1'],
        ipFamilies: ['IPv4', 'IPv6'],
        ipFamilyPolicy: 'PreferDualStack'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Cluster IPs')
      expect(wrapper.text()).toContain('172.20.159.8')
      expect(wrapper.text()).toContain('2001:db8::1')
      expect(wrapper.text()).toContain('IPv4')
      expect(wrapper.text()).toContain('IPv6')
      expect(wrapper.text()).toContain('PreferDualStack')
    })

    it('renders IPv6-only service configuration', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '2001:db8::1',
        clusterIPs: ['2001:db8::1'],
        ipFamilies: ['IPv6'],
        ipFamilyPolicy: 'SingleStack'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('2001:db8::1')
      expect(wrapper.text()).toContain('IPv6')
      expect(wrapper.text()).toContain('SingleStack')
    })
  })

  describe('Session Affinity', () => {
    it('renders None session affinity', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        sessionAffinity: 'None'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Session Affinity')
      expect(wrapper.text()).toContain('None')
    })

    it('renders ClientIP session affinity with timeout', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        sessionAffinity: 'ClientIP',
        sessionAffinityConfig: {
          clientIP: {
            timeoutSeconds: 10800
          }
        }
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Session Affinity')
      expect(wrapper.text()).toContain('ClientIP')
      expect(wrapper.text()).toContain('Timeout: 10800s')
    })

    it('renders ClientIP session affinity without timeout config', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        sessionAffinity: 'ClientIP'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Session Affinity')
      expect(wrapper.text()).toContain('ClientIP')
      expect(wrapper.text()).not.toContain('Timeout:')
    })
  })

  describe('Edge Cases and Missing Fields', () => {
    it('renders service with minimal configuration', () => {
      const serviceSpec = {
        type: 'ClusterIP'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Service Configuration')
      expect(wrapper.text()).toContain('Type')
      expect(wrapper.text()).toContain('ClusterIP')
    })

    it('handles missing type field gracefully', () => {
      const serviceSpec = {
        clusterIP: '10.0.0.1'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Service Configuration')
      expect(wrapper.text()).toContain('ClusterIP') // Default type
    })

    it('handles empty ports array', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        ports: []
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('Service Configuration')
      expect(wrapper.text()).not.toContain('Ports')
    })

    it('handles missing optional fields gracefully', () => {
      const serviceSpec = {
        type: 'LoadBalancer',
        clusterIP: '10.0.0.1'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.text()).toContain('LoadBalancer')
      expect(wrapper.text()).toContain('10.0.0.1')
      // Should not show optional fields that are not present
      expect(wrapper.text()).not.toContain('Session Affinity')
      expect(wrapper.text()).not.toContain('IP Families')
    })
  })

  describe('Visual Elements and Styling', () => {
    it('applies correct CSS classes for status badges', () => {
      const serviceSpec = {
        type: 'LoadBalancer',
        sessionAffinity: 'ClientIP',
        internalTrafficPolicy: 'Local',
        ipFamilyPolicy: 'SingleStack',
        ipFamilies: ['IPv4']
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      // Check for status badge classes
      expect(wrapper.html()).toContain('status-badge')
      expect(wrapper.html()).toContain('status-badge-info')
      expect(wrapper.html()).toContain('status-badge-warning')
      expect(wrapper.html()).toContain('status-badge-secondary')
      expect(wrapper.html()).toContain('status-badge-success')
    })

    it('applies correct container styling', () => {
      const serviceSpec = {
        type: 'ClusterIP'
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      // Check for main container styling
      expect(wrapper.html()).toContain('elevated-surface')
      expect(wrapper.html()).toContain('rounded-lg')
      expect(wrapper.html()).toContain('p-4')
    })

    it('applies monospace font to IP addresses', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8']
      }

      const wrapper = mount(ServiceConfiguration, {
        props: {
          spec: serviceSpec
        }
      })

      expect(wrapper.html()).toContain('font-mono')
    })
  })
})