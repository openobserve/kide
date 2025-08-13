import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceOverview from '@/components/ResourcePanel/ResourceOverview.vue'
import ServiceConfiguration from '@/components/ResourcePanel/ServiceConfiguration.vue'
import PodConditions from '@/components/ResourcePanel/PodConditions.vue'
import PodVolumes from '@/components/ResourcePanel/PodVolumes.vue'
import WorkloadConfiguration from '@/components/ResourcePanel/WorkloadConfiguration.vue'
import Tooltip from '@/components/ui/Tooltip.vue'
import type { K8sListItem } from '@/types'

// Mock the composable
vi.mock('@/composables/useResourceStatus', () => ({
  useResourceStatus: () => ({
    getStatusText: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Service') {
        if (item.serviceSpec?.type === 'LoadBalancer') {
          return (item.serviceStatus?.loadBalancer?.ingress && item.serviceStatus.loadBalancer.ingress.length > 0) ? 'Ready' : 'Pending'
        }
        return 'Ready'
      }
      return 'Unknown'
    }),
    getStatusClass: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Service') {
        if (item.serviceSpec?.type === 'LoadBalancer') {
          return (item.serviceStatus?.loadBalancer?.ingress && item.serviceStatus.loadBalancer.ingress.length > 0) 
            ? 'status-badge status-badge-success' 
            : 'status-badge status-badge-yellow'
        }
        return 'status-badge status-badge-success'
      }
      return 'status-badge status-badge-secondary'
    }),
    getGenericStatus: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Service') return item.serviceStatus
      return null
    }),
    getGenericSpec: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Service') return item.serviceSpec
      return null
    })
  })
}))

describe('Service ResourceOverview Integration Tests', () => {
  const createServiceResource = (serviceSpec: any, serviceStatus?: any): K8sListItem => ({
    metadata: {
      name: 'test-service',
      namespace: 'default',
      uid: 'service-uid-123',
      creationTimestamp: '2024-01-01T00:00:00Z',
      labels: {
        'app': 'test-app',
        'version': 'v1.0'
      },
      annotations: {
        'kubectl.kubernetes.io/last-applied-configuration': '{"apiVersion":"v1","kind":"Service"}',
        'service.beta.kubernetes.io/aws-load-balancer-type': 'nlb'
      }
    },
    kind: 'Service',
    apiVersion: 'v1',
    serviceSpec: serviceSpec,
    serviceStatus: serviceStatus,
    complete_object: {
      apiVersion: 'v1',
      kind: 'Service',
      metadata: {
        name: 'test-service',
        namespace: 'default'
      },
      spec: serviceSpec,
      status: serviceStatus
    }
  })

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Service Configuration Integration', () => {
    it('renders Service overview with configuration when service spec is available', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8'],
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
          'app.kubernetes.io/name': 'nats'
        }
      }

      const serviceResource = createServiceResource(serviceSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check basic resource information
      expect(wrapper.text()).toContain('Service Information')
      expect(wrapper.text()).toContain('test-service')
      expect(wrapper.text()).toContain('default')
      expect(wrapper.text()).toContain('Service')

      // Check ServiceConfiguration component is rendered
      const serviceConfigComponent = wrapper.findComponent(ServiceConfiguration)
      expect(serviceConfigComponent.exists()).toBe(true)
      expect(serviceConfigComponent.props('spec')).toEqual(serviceSpec)

      // Check service configuration content
      expect(wrapper.text()).toContain('Service Configuration')
      expect(wrapper.text()).toContain('ClusterIP')
      expect(wrapper.text()).toContain('172.20.159.8')
      expect(wrapper.text()).toContain('None') // Session affinity
      expect(wrapper.text()).toContain('Cluster') // Internal traffic policy
    })

    it('includes sessionAffinity in basic resource information fields', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '10.0.0.1',
        sessionAffinity: 'ClientIP'
      }

      const serviceResource = createServiceResource(serviceSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check that sessionAffinity appears in the basic resource information
      expect(wrapper.text()).toContain('Session Affinity')
      expect(wrapper.text()).toContain('ClientIP')
    })

    it('renders labels and annotations for Service', () => {
      const serviceSpec = {
        type: 'LoadBalancer',
        clusterIP: '10.0.0.1'
      }

      const serviceResource = createServiceResource(serviceSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check labels section
      expect(wrapper.text()).toContain('Labels')
      expect(wrapper.text()).toContain('app')
      expect(wrapper.text()).toContain('test-app')
      expect(wrapper.text()).toContain('version')
      expect(wrapper.text()).toContain('v1.0')

      // Check annotations section
      expect(wrapper.text()).toContain('Annotations')
      expect(wrapper.text()).toContain('kubectl.kubernetes.io/last-applied-configuration')
      expect(wrapper.text()).toContain('service.beta.kubernetes.io/aws-load-balancer-type')
    })

    it('does not render ServiceConfiguration when service spec is missing', () => {
      const serviceResource: K8sListItem = {
        metadata: {
          name: 'test-service',
          namespace: 'default',
          uid: 'service-uid-123'
        },
        kind: 'Service',
        apiVersion: 'v1',
        // No serviceSpec provided
      }
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check that ServiceConfiguration is not rendered
      const serviceConfigComponent = wrapper.findComponent(ServiceConfiguration)
      expect(serviceConfigComponent.exists()).toBe(false)
      expect(wrapper.text()).not.toContain('Service Configuration')
    })

    it('does not render ServiceConfiguration for non-Service resources', () => {
      const podResource: K8sListItem = {
        metadata: {
          name: 'test-pod',
          namespace: 'default',
          uid: 'pod-uid-123'
        },
        kind: 'Pod',
        apiVersion: 'v1',
        podSpec: {
          containers: [
            {
              name: 'test-container',
              image: 'nginx:latest'
            }
          ]
        }
      }
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: podResource,
          resourceKind: 'Pod'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check that ServiceConfiguration is not rendered for Pod
      const serviceConfigComponent = wrapper.findComponent(ServiceConfiguration)
      expect(serviceConfigComponent.exists()).toBe(false)
      expect(wrapper.text()).not.toContain('Service Configuration')
    })
  })

  describe('LoadBalancer Service Integration', () => {
    it('renders LoadBalancer service configuration correctly', () => {
      const serviceSpec = {
        type: 'LoadBalancer',
        clusterIP: '10.0.0.1',
        externalTrafficPolicy: 'Local',
        allocateLoadBalancerNodePorts: true,
        ports: [
          {
            port: 80,
            targetPort: 8080,
            protocol: 'TCP'
          }
        ]
      }

      const serviceStatus = {
        loadBalancer: {
          ingress: [
            { ip: '203.0.113.1' },
            { hostname: 'example-lb-123456.us-west-2.elb.amazonaws.com' }
          ]
        }
      }

      const serviceResource = createServiceResource(serviceSpec, serviceStatus)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Test core LoadBalancer service configuration
      expect(wrapper.text()).toContain('LoadBalancer')
      expect(wrapper.text()).toContain('10.0.0.1')
      expect(wrapper.text()).toContain('Local') // External traffic policy
      expect(wrapper.text()).toContain('80:8080/TCP')
      expect(wrapper.text()).toContain('Yes') // Allocate LoadBalancer NodePorts
    })
  })

  describe('Dual-Stack Service Integration', () => {
    it('renders dual-stack IPv4/IPv6 service configuration', () => {
      const serviceSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8', '2001:db8::1'],
        ipFamilies: ['IPv4', 'IPv6'],
        ipFamilyPolicy: 'PreferDualStack',
        internalTrafficPolicy: 'Cluster'
      }

      const serviceResource = createServiceResource(serviceSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      expect(wrapper.text()).toContain('172.20.159.8')
      expect(wrapper.text()).toContain('2001:db8::1')
      expect(wrapper.text()).toContain('IPv4')
      expect(wrapper.text()).toContain('IPv6')
      expect(wrapper.text()).toContain('PreferDualStack')
    })
  })

  describe('Complex Service Configuration', () => {
    it('renders service with comprehensive configuration matching sample YAML', () => {
      // Based on the user's sample YAML
      const serviceSpec = {
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8'],
        internalTrafficPolicy: 'Cluster',
        ipFamilies: ['IPv4'],
        ipFamilyPolicy: 'SingleStack',
        ports: [
          {
            appProtocol: 'tcp',
            name: 'nats',
            port: 4222,
            protocol: 'TCP',
            targetPort: 'nats'
          }
        ],
        selector: {
          'app.kubernetes.io/component': 'nats',
          'app.kubernetes.io/instance': 'o2',
          'app.kubernetes.io/name': 'nats'
        },
        sessionAffinity: 'None',
        type: 'ClusterIP'
      }

      const serviceResource = createServiceResource(serviceSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            ServiceConfiguration,
            PodConditions,
            PodVolumes,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Verify all fields from the sample YAML are displayed
      expect(wrapper.text()).toContain('ClusterIP')
      expect(wrapper.text()).toContain('172.20.159.8')
      expect(wrapper.text()).toContain('Cluster')
      expect(wrapper.text()).toContain('IPv4')
      expect(wrapper.text()).toContain('SingleStack')
      expect(wrapper.text()).toContain('4222:nats/TCP')
      expect(wrapper.text()).toContain('(nats)')
      expect(wrapper.text()).toContain('[tcp]')
      expect(wrapper.text()).toContain('None')
      expect(wrapper.text()).toContain('app.kubernetes.io/component')
      expect(wrapper.text()).toContain('nats')
      expect(wrapper.text()).toContain('app.kubernetes.io/instance')
      expect(wrapper.text()).toContain('o2')
    })
  })
})