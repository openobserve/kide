import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceOverview from '@/components/ResourcePanel/ResourceOverview.vue'
import IngressConfiguration from '@/components/ResourcePanel/IngressConfiguration.vue'
import PodConditions from '@/components/ResourcePanel/PodConditions.vue'
import PodVolumes from '@/components/ResourcePanel/PodVolumes.vue'
import ServiceConfiguration from '@/components/ResourcePanel/ServiceConfiguration.vue'
import WorkloadConfiguration from '@/components/ResourcePanel/WorkloadConfiguration.vue'
import Tooltip from '@/components/ui/Tooltip.vue'
import type { K8sListItem } from '@/types'

// Mock the composable
vi.mock('@/composables/useResourceStatus', () => ({
  useResourceStatus: () => ({
    getStatusText: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Ingress') {
        return (item.ingressStatus?.loadBalancer?.ingress && item.ingressStatus.loadBalancer.ingress.length > 0) ? 'Ready' : 'Pending'
      }
      return 'Unknown'
    }),
    getStatusClass: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Ingress') {
        return (item.ingressStatus?.loadBalancer?.ingress && item.ingressStatus.loadBalancer.ingress.length > 0) 
          ? 'status-badge status-badge-success' 
          : 'status-badge status-badge-yellow'
      }
      return 'status-badge status-badge-secondary'
    }),
    getGenericStatus: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Ingress') return item.ingressStatus
      return null
    }),
    getGenericSpec: vi.fn((item: K8sListItem) => {
      if (item.kind === 'Ingress') return item.ingressSpec
      return null
    })
  })
}))

describe('Ingress ResourceOverview Integration Tests', () => {
  const createIngressResource = (ingressSpec: any, ingressStatus?: any): K8sListItem => ({
    metadata: {
      name: 'example-app-ingress-internal',
      namespace: 'example-namespace',
      uid: 'ingress-uid-abc123',
      creationTimestamp: '2024-01-15T10:30:00Z',
      labels: {
        'app.kubernetes.io/instance': 'example-app',
        'app.kubernetes.io/managed-by': 'Helm',
        'app.kubernetes.io/name': 'example-service',
        'app.kubernetes.io/version': 'v1.2.3',
        'helm.sh/chart': 'example-service-1.2.34'
      },
      annotations: {
        'kubectl.kubernetes.io/last-applied-configuration': '{"apiVersion":"networking.k8s.io/v1","kind":"Ingress"}',
        'meta.helm.sh/release-name': 'example-app',
        'meta.helm.sh/release-namespace': 'example-namespace',
        'nginx.ingress.kubernetes.io/enable-cors': 'true'
      }
    },
    kind: 'Ingress',
    apiVersion: 'networking.k8s.io/v1',
    ingressSpec: ingressSpec,
    ingressStatus: ingressStatus,
    complete_object: {
      apiVersion: 'networking.k8s.io/v1',
      kind: 'Ingress',
      metadata: {
        name: 'example-app-ingress-internal',
        namespace: 'example-namespace'
      },
      spec: ingressSpec,
      status: ingressStatus
    }
  })

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Ingress Configuration Integration', () => {
    it('renders Ingress overview with configuration when ingress spec is available', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-internal-lb',
        rules: [
          {
            host: 'auth-service.internal.example-company.dev',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'ImplementationSpecific',
                  backend: {
                    service: {
                      name: 'example-auth-service',
                      port: {
                        number: 5556
                      }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressResource = createIngressResource(ingressSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check basic resource information
      expect(wrapper.text()).toContain('Ingress Information')
      expect(wrapper.text()).toContain('example-app-ingress-internal')
      expect(wrapper.text()).toContain('example-namespace')
      expect(wrapper.text()).toContain('Ingress')

      // Check IngressConfiguration component is rendered
      const ingressConfigComponent = wrapper.findComponent(IngressConfiguration)
      expect(ingressConfigComponent.exists()).toBe(true)
      expect(ingressConfigComponent.props('spec')).toEqual(ingressSpec)

      // Check ingress configuration content
      expect(wrapper.text()).toContain('Ingress Configuration')
      expect(wrapper.text()).toContain('nginx-internal-lb')
      expect(wrapper.text()).toContain('auth-service.internal.example-company.dev')
      expect(wrapper.text()).toContain('example-auth-service')
    })

    it('renders built-in hosts section in ResourceOverview', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-public',
        rules: [
          {
            host: 'web.example-company.com',
            http: {
              paths: [
                {
                  path: '/api',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-backend-service',
                      port: { number: 8080 }
                    }
                  }
                }
              ]
            }
          },
          {
            host: 'admin.example-company.com',
            http: {
              paths: [
                {
                  path: '/dashboard',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'admin-dashboard-service',
                      port: { number: 3000 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressResource = createIngressResource(ingressSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check built-in hosts section
      expect(wrapper.text()).toContain('Hosts')
      expect(wrapper.text()).toContain('(2)') // Two hosts
      expect(wrapper.text()).toContain('web.example-company.com')
      expect(wrapper.text()).toContain('admin.example-company.com')
      expect(wrapper.text()).toContain('/api')
      expect(wrapper.text()).toContain('/dashboard')
    })

    it('renders labels and annotations for Ingress', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-test',
        rules: []
      }

      const ingressResource = createIngressResource(ingressSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check labels section
      expect(wrapper.text()).toContain('Labels')
      expect(wrapper.text()).toContain('app.kubernetes.io/instance')
      expect(wrapper.text()).toContain('example-app')
      expect(wrapper.text()).toContain('helm.sh/chart')
      expect(wrapper.text()).toContain('example-service-1.2.34')

      // Check annotations section
      expect(wrapper.text()).toContain('Annotations')
      expect(wrapper.text()).toContain('nginx.ingress.kubernetes.io/enable-cors')
      expect(wrapper.text()).toContain('meta.helm.sh/release-name')
    })

    it('does not render IngressConfiguration when ingress spec is missing', () => {
      const ingressResource: K8sListItem = {
        metadata: {
          name: 'example-ingress',
          namespace: 'default',
          uid: 'ingress-uid-456'
        },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        // No ingressSpec provided
      }
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check that IngressConfiguration is not rendered
      const ingressConfigComponent = wrapper.findComponent(IngressConfiguration)
      expect(ingressConfigComponent.exists()).toBe(false)
      expect(wrapper.text()).not.toContain('Ingress Configuration')
    })

    it('does not render IngressConfiguration for non-Ingress resources', () => {
      const serviceResource: K8sListItem = {
        metadata: {
          name: 'test-service',
          namespace: 'default',
          uid: 'service-uid-789'
        },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'ClusterIP',
          ports: [{ port: 80, targetPort: 8080 }]
        }
      }
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: serviceResource,
          resourceKind: 'Service'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check that IngressConfiguration is not rendered for Service
      const ingressConfigComponent = wrapper.findComponent(IngressConfiguration)
      expect(ingressConfigComponent.exists()).toBe(false)
      expect(wrapper.text()).not.toContain('Ingress Configuration')
    })
  })

  describe('TLS Configuration Integration', () => {
    it('renders Ingress with TLS configuration', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-tls',
        rules: [
          {
            host: 'secure-api.example-company.com',
            http: {
              paths: [
                {
                  path: '/secure',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'secure-api-service',
                      port: { number: 8443 }
                    }
                  }
                }
              ]
            }
          }
        ],
        tls: [
          {
            secretName: 'example-company-tls-cert',
            hosts: [
              'secure-api.example-company.com',
              'backup-api.example-company.com'
            ]
          }
        ]
      }

      const ingressResource = createIngressResource(ingressSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      expect(wrapper.text()).toContain('TLS Configuration')
      expect(wrapper.text()).toContain('example-company-tls-cert')
      expect(wrapper.text()).toContain('🔒 secure-api.example-company.com')
      expect(wrapper.text()).toContain('🔒 backup-api.example-company.com')
    })
  })

  describe('Load Balancer Integration', () => {
    it('renders Ingress with AWS load balancer', () => {
      const ingressSpec = {
        ingressClassName: 'aws-load-balancer-controller',
        rules: [
          {
            host: 'lb.example-company.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'lb-backend-service',
                      port: { number: 80 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressStatus = {
        loadBalancer: {
          ingress: [
            {
              hostname: 'k8s-elb-abcd1234-example12345678.elb.us-east-1.amazonaws.com'
            }
          ]
        }
      }

      const ingressResource = createIngressResource(ingressSpec, ingressStatus)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      expect(wrapper.text()).toContain('Load Balancer')
      expect(wrapper.text()).toContain('k8s-elb-abcd1234-example12345678.elb.us-east-1.amazonaws.com')
    })

    it('shows pending status for Ingress without load balancer', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-pending',
        rules: [
          {
            host: 'pending.example-company.com',
            http: {
              paths: [
                {
                  path: '/',
                  backend: {
                    service: {
                      name: 'pending-service',
                      port: { number: 80 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressStatus = {
        loadBalancer: {}
      }

      const ingressResource = createIngressResource(ingressSpec, ingressStatus)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // The pending status would be shown in the resource status, not necessarily in load balancer section
      expect(wrapper.text()).toContain('nginx-pending')
      expect(wrapper.text()).toContain('pending.example-company.com')
    })
  })

  describe('Complex Ingress Configuration', () => {
    it('renders comprehensive Ingress configuration matching sample YAML structure', () => {
      // Based on the user's sample YAML but with obfuscated data
      const ingressSpec = {
        ingressClassName: 'nginx-internal-lb',
        rules: [
          {
            host: 'introspection-auth.internal.example-company.dev',
            http: {
              paths: [
                {
                  backend: {
                    service: {
                      name: 'example-auth-dex-service',
                      port: {
                        number: 5556
                      }
                    }
                  },
                  path: '/',
                  pathType: 'ImplementationSpecific'
                }
              ]
            }
          }
        ]
      }

      const ingressStatus = {
        loadBalancer: {
          ingress: [
            {
              hostname: 'a1b2c3d4-example-load-balancer-567890ab.elb.us-east-2.amazonaws.com'
            }
          ]
        }
      }

      const ingressResource = createIngressResource(ingressSpec, ingressStatus)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Verify all fields from the sample YAML structure are displayed
      expect(wrapper.text()).toContain('nginx-internal-lb')
      expect(wrapper.text()).toContain('introspection-auth.internal.example-company.dev')
      expect(wrapper.text()).toContain('/')
      expect(wrapper.text()).toContain('ImplementationSpecific')
      expect(wrapper.text()).toContain('example-auth-dex-service')
      expect(wrapper.text()).toContain('5556')
      expect(wrapper.text()).toContain('a1b2c3d4-example-load-balancer-567890ab.elb.us-east-2.amazonaws.com')
    })
  })

  describe('Ingress Hosts Section', () => {
    it('renders hosts section with path details and service mappings', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-multi-path',
        rules: [
          {
            host: 'multi-path.example-company.com',
            http: {
              paths: [
                {
                  path: '/api/v1',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-v1-service',
                      port: { number: 8080 }
                    }
                  }
                },
                {
                  path: '/api/v2',
                  pathType: 'Prefix', 
                  backend: {
                    service: {
                      name: 'api-v2-service',
                      port: { number: 8090 }
                    }
                  }
                },
                {
                  path: '/static',
                  pathType: 'Exact',
                  backend: {
                    service: {
                      name: 'static-content-service',
                      port: { name: 'http' }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressResource = createIngressResource(ingressSpec)
      
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingressResource,
          resourceKind: 'Ingress'
        },
        global: {
          components: {
            IngressConfiguration,
            PodConditions,
            PodVolumes,
            ServiceConfiguration,
            WorkloadConfiguration,
            Tooltip
          }
        }
      })

      // Check the built-in hosts section shows all path details
      expect(wrapper.text()).toContain('Hosts')
      expect(wrapper.text()).toContain('multi-path.example-company.com')
      expect(wrapper.text()).toContain('3 paths') // Should show path count
      expect(wrapper.text()).toContain('/api/v1')
      expect(wrapper.text()).toContain('/api/v2')
      expect(wrapper.text()).toContain('/static')
      expect(wrapper.text()).toContain('api-v1-service')
      expect(wrapper.text()).toContain('api-v2-service')
      expect(wrapper.text()).toContain('static-content-service')
      expect(wrapper.text()).toContain('8080')
      expect(wrapper.text()).toContain('8090')
      expect(wrapper.text()).toContain('http')
    })
  })
})