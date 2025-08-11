import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ResourceOverview from '../../components/ResourcePanel/ResourceOverview.vue'

describe('ResourceOverview - Ingress Hosts Section', () => {
  it('should display hosts section for Ingress resources', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'test-ingress',
        namespace: 'default',
        uid: 'ingress-123'
      },
      ingressSpec: {
        rules: [
          {
            host: 'api.example.com',
            http: {
              paths: [
                {
                  path: '/api/v1',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-service',
                      port: {
                        number: 80
                      }
                    }
                  }
                },
                {
                  path: '/api/v2',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-v2-service',
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
            host: 'web.example.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'web-service',
                      port: {
                        number: 8080
                      }
                    }
                  }
                }
              ]
            }
          }
        ]
      }
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should display hosts section
    expect(wrapper.text()).toContain('Hosts')
    expect(wrapper.text()).toContain('(2)')

    // Should display host names
    expect(wrapper.text()).toContain('api.example.com')
    expect(wrapper.text()).toContain('web.example.com')

    // Should display path counts
    expect(wrapper.text()).toContain('2 paths')
    expect(wrapper.text()).toContain('1 path')

    // Should display actual paths
    expect(wrapper.text()).toContain('/api/v1')
    expect(wrapper.text()).toContain('/api/v2')
    expect(wrapper.text()).toContain('/')

    // Should display path types
    expect(wrapper.text()).toContain('Prefix')

    // Should display backend services
    expect(wrapper.text()).toContain('api-service')
    expect(wrapper.text()).toContain('api-v2-service')
    expect(wrapper.text()).toContain('web-service')

    // Should display ports
    expect(wrapper.text()).toContain(':80')
    expect(wrapper.text()).toContain(':http')
    expect(wrapper.text()).toContain(':8080')

    // Should display full URLs as clickable links
    expect(wrapper.text()).toContain('https://api.example.com/api/v1')
    expect(wrapper.text()).toContain('https://api.example.com/api/v2')
    expect(wrapper.text()).toContain('https://web.example.com/')
    
    // Should render URLs as hyperlinks
    const links = wrapper.findAll('a[href^="https://"]')
    expect(links.length).toBe(3) // Should have 3 clickable URL links
    
    // Verify link attributes
    const firstLink = links[0]
    expect(firstLink.attributes('href')).toBe('https://api.example.com/api/v1')
    expect(firstLink.attributes('target')).toBe('_blank')
    expect(firstLink.attributes('rel')).toBe('noopener noreferrer')
  })

  it('should handle ingress with wildcard host', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'wildcard-ingress',
        namespace: 'default'
      },
      ingressSpec: {
        rules: [
          {
            http: {
              paths: [
                {
                  path: '/app',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'app-service',
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
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should display wildcard host as *
    expect(wrapper.text()).toContain('*')
    
    // Should display path
    expect(wrapper.text()).toContain('/app')
    
    // Should display service
    expect(wrapper.text()).toContain('app-service')
    
    // Should show example URL with example.com fallback as clickable link
    expect(wrapper.text()).toContain('https://example.com/app')
    
    // Should render URL as hyperlink
    const links = wrapper.findAll('a[href^="https://"]')
    expect(links.length).toBe(1) // Should have 1 clickable URL link
    expect(links[0].attributes('href')).toBe('https://example.com/app')
  })

  it('should handle ingress rule with no HTTP paths', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'empty-paths-ingress',
        namespace: 'default'
      },
      ingressSpec: {
        rules: [
          {
            host: 'empty.example.com'
          }
        ]
      }
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should display host
    expect(wrapper.text()).toContain('empty.example.com')
    
    // Should show no paths message
    expect(wrapper.text()).toContain('No HTTP paths configured')
  })

  it('should not display hosts section for non-Ingress resources', () => {
    const serviceData = {
      kind: 'Service',
      apiVersion: 'v1',
      metadata: {
        name: 'test-service',
        namespace: 'default'
      },
      serviceSpec: {
        type: 'ClusterIP',
        ports: [
          {
            port: 80,
            targetPort: 8080
          }
        ]
      }
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: serviceData,
        resourceKind: 'Service'
      }
    })

    // Should NOT display hosts section
    expect(wrapper.text()).not.toContain('Hosts')
  })

  it('should not display hosts section for Ingress without rules', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'no-rules-ingress',
        namespace: 'default'
      },
      ingressSpec: {}
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should NOT display hosts section when no rules
    expect(wrapper.text()).not.toContain('Hosts')
  })

  it('should handle ingress with resource backend (not service)', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'resource-backend-ingress',
        namespace: 'default'
      },
      ingressSpec: {
        rules: [
          {
            host: 'resource.example.com',
            http: {
              paths: [
                {
                  path: '/storage',
                  pathType: 'Exact',
                  backend: {
                    resource: {
                      apiGroup: 'k8s.example.com',
                      kind: 'StorageOS',
                      name: 'storage-resource'
                    }
                  }
                }
              ]
            }
          }
        ]
      }
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should display resource backend name
    expect(wrapper.text()).toContain('storage-resource')
    
    // Should display path type
    expect(wrapper.text()).toContain('Exact')
    
    // Should display full URL as clickable link
    expect(wrapper.text()).toContain('https://resource.example.com/storage')
    
    // Should render URL as hyperlink
    const links = wrapper.findAll('a[href^="https://"]')
    expect(links.length).toBe(1) // Should have 1 clickable URL link
    expect(links[0].attributes('href')).toBe('https://resource.example.com/storage')
  })

  it('should handle missing or default path values', () => {
    const ingressData = {
      kind: 'Ingress',
      apiVersion: 'networking.k8s.io/v1',
      metadata: {
        name: 'minimal-ingress',
        namespace: 'default'
      },
      ingressSpec: {
        rules: [
          {
            host: 'minimal.example.com',
            http: {
              paths: [
                {
                  // No path specified - should default to '/'
                  // No pathType specified - should default to 'Prefix'
                  backend: {
                    service: {
                      name: 'minimal-service',
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
    }

    const wrapper = mount(ResourceOverview, {
      props: {
        resourceData: ingressData,
        resourceKind: 'Ingress'
      }
    })

    // Should show default path
    expect(wrapper.text()).toContain('/')
    
    // Should show default path type
    expect(wrapper.text()).toContain('Prefix')
    
    // Should display full URL with default path as clickable link
    expect(wrapper.text()).toContain('https://minimal.example.com/')
    
    // Should render URL as hyperlink
    const links = wrapper.findAll('a[href^="https://"]')
    expect(links.length).toBe(1) // Should have 1 clickable URL link
    expect(links[0].attributes('href')).toBe('https://minimal.example.com/')
  })
})