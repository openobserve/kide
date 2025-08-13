import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import IngressConfiguration from '@/components/ResourcePanel/IngressConfiguration.vue'
import type { K8sListItem } from '@/types'

describe('IngressConfiguration Integration Tests', () => {
  describe('Component Rendering', () => {
    it('renders basic Ingress configuration with single host', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-internal',
        rules: [
          {
            host: 'api.example-app.internal.company.dev',
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

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check main title
      expect(wrapper.text()).toContain('Ingress Configuration')

      // Check ingress class
      expect(wrapper.text()).toContain('Ingress Class')
      expect(wrapper.text()).toContain('nginx-internal')

      // Check hosts and rules section
      expect(wrapper.text()).toContain('Hosts & Rules')
      expect(wrapper.text()).toContain('api.example-app.internal.company.dev')
      expect(wrapper.text()).toContain('1 path')

      // Check path details
      expect(wrapper.text()).toContain('/')
      expect(wrapper.text()).toContain('ImplementationSpecific')
      expect(wrapper.text()).toContain('example-auth-service')
      expect(wrapper.text()).toContain('5556')
    })

    it('renders multiple hosts with multiple paths', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-public',
        rules: [
          {
            host: 'web.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/api',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-backend-service',
                      port: {
                        number: 8080
                      }
                    }
                  }
                },
                {
                  path: '/static',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'static-files-service',
                      port: {
                        name: 'http'
                      }
                    }
                  }
                }
              ]
            }
          },
          {
            host: 'admin.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Exact',
                  backend: {
                    service: {
                      name: 'admin-panel-service',
                      port: {
                        number: 3000
                      }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check multiple hosts
      expect(wrapper.text()).toContain('web.example-app.company.com')
      expect(wrapper.text()).toContain('admin.example-app.company.com')

      // Check path counts
      expect(wrapper.text()).toContain('2 paths')
      expect(wrapper.text()).toContain('1 path')

      // Check different path types
      expect(wrapper.text()).toContain('Prefix')
      expect(wrapper.text()).toContain('Exact')

      // Check service names and ports
      expect(wrapper.text()).toContain('api-backend-service')
      expect(wrapper.text()).toContain('8080')
      expect(wrapper.text()).toContain('static-files-service')
      expect(wrapper.text()).toContain('http')
      expect(wrapper.text()).toContain('admin-panel-service')
      expect(wrapper.text()).toContain('3000')
    })

    it('renders catch-all host rule', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-default',
        rules: [
          {
            // No host specified - catch-all
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'default-backend-service',
                      port: {
                        number: 80
                      }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check catch-all indicator
      expect(wrapper.text()).toContain('*')
      expect(wrapper.text()).toContain('(catch-all)')
      expect(wrapper.text()).toContain('default-backend-service')
    })

    it('renders default backend configuration', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-lb',
        defaultBackend: {
          service: {
            name: 'global-default-service',
            port: {
              number: 8080
            }
          }
        },
        rules: []
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('Default Backend')
      expect(wrapper.text()).toContain('global-default-service:8080')
    })
  })

  describe('TLS Configuration', () => {
    it('renders TLS configuration with certificates', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-tls',
        rules: [
          {
            host: 'secure.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'secure-app-service',
                      port: {
                        number: 8443
                      }
                    }
                  }
                }
              ]
            }
          }
        ],
        tls: [
          {
            secretName: 'example-app-tls-cert',
            hosts: [
              'secure.example-app.company.com',
              'api.example-app.company.com'
            ]
          },
          {
            secretName: 'wildcard-company-cert',
            hosts: [
              '*.company.com'
            ]
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check TLS section
      expect(wrapper.text()).toContain('TLS Configuration')
      expect(wrapper.text()).toContain('example-app-tls-cert')
      expect(wrapper.text()).toContain('wildcard-company-cert')

      // Check TLS hosts with lock icons
      expect(wrapper.text()).toContain('🔒 secure.example-app.company.com')
      expect(wrapper.text()).toContain('🔒 api.example-app.company.com')
      expect(wrapper.text()).toContain('🔒 *.company.com')
    })

    it('renders TLS configuration without specific hosts', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-tls',
        tls: [
          {
            secretName: 'default-tls-cert'
            // No hosts specified
          }
        ],
        rules: []
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('TLS Configuration')
      expect(wrapper.text()).toContain('default-tls-cert')
    })

    it('renders TLS configuration with default certificate', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-tls',
        tls: [
          {
            // No secretName specified - uses default certificate
            hosts: ['default.example-app.company.com']
          }
        ],
        rules: []
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('TLS Configuration')
      expect(wrapper.text()).toContain('Default Certificate')
      expect(wrapper.text()).toContain('🔒 default.example-app.company.com')
    })
  })

  describe('Load Balancer Status', () => {
    it('renders load balancer with IP address', () => {
      const ingressSpec = {
        ingressClassName: 'aws-lb',
        rules: [
          {
            host: 'lb-test.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'test-service',
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
              ip: '203.0.113.42'
            }
          ]
        }
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec,
          status: ingressStatus
        }
      })

      expect(wrapper.text()).toContain('Load Balancer')
      expect(wrapper.text()).toContain('203.0.113.42')
    })

    it('renders load balancer with hostname and ports', () => {
      const ingressSpec = {
        ingressClassName: 'aws-lb',
        rules: []
      }

      const ingressStatus = {
        loadBalancer: {
          ingress: [
            {
              hostname: 'a1b2c3d4-example-abcd1234.elb.us-west-2.amazonaws.com',
              ports: [
                { port: 80, protocol: 'TCP' },
                { port: 443, protocol: 'TCP' }
              ]
            }
          ]
        }
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec,
          status: ingressStatus
        }
      })

      expect(wrapper.text()).toContain('Load Balancer')
      expect(wrapper.text()).toContain('a1b2c3d4-example-abcd1234.elb.us-west-2.amazonaws.com')
      expect(wrapper.text()).toContain(':80/TCP')
      expect(wrapper.text()).toContain(':443/TCP')
    })

    it('renders multiple load balancer endpoints', () => {
      const ingressSpec = {
        ingressClassName: 'multi-lb',
        rules: []
      }

      const ingressStatus = {
        loadBalancer: {
          ingress: [
            {
              ip: '198.51.100.10'
            },
            {
              ip: '198.51.100.11'
            },
            {
              hostname: 'backup-lb.example-app.company.com'
            }
          ]
        }
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec,
          status: ingressStatus
        }
      })

      expect(wrapper.text()).toContain('Load Balancer')
      expect(wrapper.text()).toContain('198.51.100.10')
      expect(wrapper.text()).toContain('198.51.100.11')
      expect(wrapper.text()).toContain('backup-lb.example-app.company.com')
    })
  })

  describe('Edge Cases and Missing Fields', () => {
    it('renders ingress with minimal configuration', () => {
      const ingressSpec = {
        rules: [
          {
            host: 'minimal.example-app.company.com',
            http: {
              paths: [
                {
                  backend: {
                    service: {
                      name: 'minimal-service'
                      // No port specified
                    }
                  }
                  // No path or pathType specified
                }
              ]
            }
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('Ingress Configuration')
      expect(wrapper.text()).toContain('minimal.example-app.company.com')
      expect(wrapper.text()).toContain('minimal-service')
      expect(wrapper.text()).toContain('/') // Default path
      expect(wrapper.text()).toContain('?') // Unknown port
    })

    it('handles missing service backend gracefully', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-test',
        rules: [
          {
            host: 'broken.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/broken',
                  pathType: 'Exact',
                  backend: {
                    // No service specified
                  }
                }
              ]
            }
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('broken.example-app.company.com')
      expect(wrapper.text()).toContain('/broken')
      expect(wrapper.text()).toContain('Unknown Service')
    })

    it('handles empty rules array', () => {
      const ingressSpec = {
        ingressClassName: 'nginx-empty',
        rules: []
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('Ingress Configuration')
      expect(wrapper.text()).toContain('nginx-empty')
      expect(wrapper.text()).not.toContain('Hosts & Rules')
    })

    it('handles missing fields gracefully', () => {
      const ingressSpec = {}

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.text()).toContain('Ingress Configuration')
      expect(wrapper.text()).not.toContain('Ingress Class')
      expect(wrapper.text()).not.toContain('Hosts & Rules')
      expect(wrapper.text()).not.toContain('TLS Configuration')
    })
  })

  describe('Visual Elements and Styling', () => {
    it('applies correct CSS classes for status badges', () => {
      const ingressSpec = {
        ingressClassName: 'test-class',
        rules: [
          {
            host: 'styled.example-app.company.com',
            http: {
              paths: [
                {
                  path: '/test',
                  backend: {
                    service: {
                      name: 'test-service',
                      port: { number: 8080 }
                    }
                  }
                }
              ]
            }
          }
        ],
        tls: [
          {
            secretName: 'test-cert',
            hosts: ['styled.example-app.company.com']
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check for status badge classes
      expect(wrapper.html()).toContain('status-badge')
      expect(wrapper.html()).toContain('status-badge-info')
      expect(wrapper.html()).toContain('status-badge-success')
    })

    it('applies correct container styling', () => {
      const ingressSpec = {
        ingressClassName: 'container-test'
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      // Check for main container styling
      expect(wrapper.html()).toContain('elevated-surface')
      expect(wrapper.html()).toContain('rounded-lg')
      expect(wrapper.html()).toContain('p-4')
    })

    it('applies monospace font to service names and ports', () => {
      const ingressSpec = {
        rules: [
          {
            host: 'mono.example-app.company.com',
            http: {
              paths: [
                {
                  backend: {
                    service: {
                      name: 'mono-service',
                      port: { number: 9999 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const wrapper = mount(IngressConfiguration, {
        props: {
          spec: ingressSpec
        }
      })

      expect(wrapper.html()).toContain('font-mono')
    })
  })
})