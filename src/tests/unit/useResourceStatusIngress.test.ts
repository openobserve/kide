import { describe, it, expect } from 'vitest'
import { useResourceStatus } from '@/composables/useResourceStatus'
import type { K8sListItem } from '@/types'

describe('useResourceStatus - Ingress Integration Tests', () => {
  const { getStatusText, getStatusClass, getGenericSpec, getGenericStatus } = useResourceStatus()

  describe('Ingress Status Logic', () => {
    it('returns Ready for Ingress with load balancer', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'test-ingress', uid: 'ingress-123' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'nginx',
          rules: [
            {
              host: 'test.example.com',
              http: {
                paths: [
                  {
                    path: '/',
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
        },
        ingressStatus: {
          loadBalancer: {
            ingress: [
              { hostname: 'lb.example.com' }
            ]
          }
        }
      }

      expect(getStatusText(ingressResource)).toBe('Ready')
      expect(getStatusClass(ingressResource)).toBe('status-badge status-badge-success')
    })

    it('returns Pending for Ingress without load balancer', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'pending-ingress', uid: 'ingress-456' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'nginx',
          rules: [
            {
              host: 'pending.example.com',
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
        },
        ingressStatus: {
          loadBalancer: {}
        }
      }

      expect(getStatusText(ingressResource)).toBe('Pending')
      expect(getStatusClass(ingressResource)).toBe('status-badge status-badge-yellow')
    })

    it('returns Ready for Ingress with IP address', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'ip-ingress', uid: 'ingress-789' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'traefik',
          rules: []
        },
        ingressStatus: {
          loadBalancer: {
            ingress: [
              { ip: '203.0.113.42' }
            ]
          }
        }
      }

      expect(getStatusText(ingressResource)).toBe('Ready')
      expect(getStatusClass(ingressResource)).toBe('status-badge status-badge-success')
    })

    it('returns Ready for Ingress with both IP and hostname', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'multi-endpoint-ingress', uid: 'ingress-101' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'aws-load-balancer',
          rules: []
        },
        ingressStatus: {
          loadBalancer: {
            ingress: [
              { 
                ip: '198.51.100.10',
                hostname: 'multi.example.com'
              }
            ]
          }
        }
      }

      expect(getStatusText(ingressResource)).toBe('Ready')
      expect(getStatusClass(ingressResource)).toBe('status-badge status-badge-success')
    })

    it('handles Ingress without status gracefully', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'no-status-ingress', uid: 'ingress-202' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'nginx',
          rules: []
        }
        // No ingressStatus provided
      }

      expect(getStatusText(ingressResource)).toBe('Pending')
      expect(getStatusClass(ingressResource)).toBe('status-badge status-badge-yellow')
    })
  })

  describe('Ingress Spec and Status Extraction', () => {
    it('extracts ingressSpec correctly', () => {
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
                      name: 'auth-dex-service',
                      port: {
                        number: 5556
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
            secretName: 'auth-tls-cert',
            hosts: [
              'auth-service.internal.example-company.dev'
            ]
          }
        ]
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'test-ingress', uid: 'ingress-303' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec
      }

      const extractedSpec = getGenericSpec(ingressResource)
      expect(extractedSpec).toEqual(ingressSpec)
      expect(extractedSpec.ingressClassName).toBe('nginx-internal-lb')
      expect(extractedSpec.rules).toHaveLength(1)
      expect(extractedSpec.rules[0].host).toBe('auth-service.internal.example-company.dev')
      expect(extractedSpec.rules[0].http.paths).toHaveLength(1)
      expect(extractedSpec.rules[0].http.paths[0].path).toBe('/')
      expect(extractedSpec.rules[0].http.paths[0].pathType).toBe('ImplementationSpecific')
      expect(extractedSpec.rules[0].http.paths[0].backend.service.name).toBe('auth-dex-service')
      expect(extractedSpec.rules[0].http.paths[0].backend.service.port.number).toBe(5556)
      expect(extractedSpec.tls).toHaveLength(1)
      expect(extractedSpec.tls[0].secretName).toBe('auth-tls-cert')
      expect(extractedSpec.tls[0].hosts).toEqual(['auth-service.internal.example-company.dev'])
    })

    it('extracts ingressStatus correctly', () => {
      const ingressStatus = {
        loadBalancer: {
          ingress: [
            {
              hostname: 'a1b2c3d4-example-load-balancer-567890ab.elb.us-east-2.amazonaws.com'
            },
            {
              ip: '203.0.113.100'
            }
          ]
        }
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'test-ingress', uid: 'ingress-404' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'aws-lb',
          rules: []
        },
        ingressStatus
      }

      const extractedStatus = getGenericStatus(ingressResource)
      expect(extractedStatus).toEqual(ingressStatus)
      expect(extractedStatus.loadBalancer.ingress).toHaveLength(2)
      expect(extractedStatus.loadBalancer.ingress[0].hostname).toBe('a1b2c3d4-example-load-balancer-567890ab.elb.us-east-2.amazonaws.com')
      expect(extractedStatus.loadBalancer.ingress[1].ip).toBe('203.0.113.100')
    })

    it('returns undefined for non-Ingress resources', () => {
      const podResource: K8sListItem = {
        metadata: { name: 'test-pod', uid: 'pod-123' },
        kind: 'Pod',
        apiVersion: 'v1',
        podSpec: {
          containers: [{ name: 'test', image: 'nginx' }]
        }
      }

      expect(getGenericSpec(podResource)).not.toEqual(expect.objectContaining({
        ingressClassName: expect.any(String),
        rules: expect.any(Array)
      }))
      expect(getGenericStatus(podResource)).toBeUndefined()
    })

    it('handles ingress with missing spec gracefully', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'test-ingress', uid: 'ingress-505' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1'
        // No ingressSpec provided
      }

      expect(getGenericSpec(ingressResource)).toBeUndefined()
      expect(getGenericStatus(ingressResource)).toBeUndefined()
    })
  })

  describe('Complex Ingress Configurations', () => {
    it('handles multi-host ingress configuration', () => {
      const multiHostSpec = {
        ingressClassName: 'nginx-multi',
        rules: [
          {
            host: 'api.example-company.com',
            http: {
              paths: [
                {
                  path: '/v1',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-v1-service',
                      port: { number: 8080 }
                    }
                  }
                },
                {
                  path: '/v2',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'api-v2-service',
                      port: { number: 8090 }
                    }
                  }
                }
              ]
            }
          },
          {
            host: 'web.example-company.com',
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'frontend-service',
                      port: { number: 80 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'multi-host-ingress', uid: 'ingress-606' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: multiHostSpec
      }

      const extractedSpec = getGenericSpec(ingressResource)
      expect(extractedSpec.rules).toHaveLength(2)
      expect(extractedSpec.rules[0].host).toBe('api.example-company.com')
      expect(extractedSpec.rules[0].http.paths).toHaveLength(2)
      expect(extractedSpec.rules[1].host).toBe('web.example-company.com')
      expect(extractedSpec.rules[1].http.paths).toHaveLength(1)
    })

    it('handles ingress with default backend', () => {
      const defaultBackendSpec = {
        ingressClassName: 'nginx-default',
        defaultBackend: {
          service: {
            name: 'default-backend-service',
            port: {
              number: 8080
            }
          }
        },
        rules: []
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'default-backend-ingress', uid: 'ingress-707' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: defaultBackendSpec
      }

      const extractedSpec = getGenericSpec(ingressResource)
      expect(extractedSpec.defaultBackend.service.name).toBe('default-backend-service')
      expect(extractedSpec.defaultBackend.service.port.number).toBe(8080)
    })

    it('handles ingress with comprehensive TLS configuration', () => {
      const tlsSpec = {
        ingressClassName: 'nginx-tls',
        rules: [
          {
            host: 'secure1.example-company.com',
            http: {
              paths: [
                {
                  path: '/',
                  backend: {
                    service: {
                      name: 'secure-service-1',
                      port: { number: 8443 }
                    }
                  }
                }
              ]
            }
          },
          {
            host: 'secure2.example-company.com',
            http: {
              paths: [
                {
                  path: '/',
                  backend: {
                    service: {
                      name: 'secure-service-2',
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
            secretName: 'tls-cert-1',
            hosts: ['secure1.example-company.com']
          },
          {
            secretName: 'tls-cert-2',
            hosts: ['secure2.example-company.com']
          },
          {
            secretName: 'wildcard-cert',
            hosts: ['*.example-company.com']
          }
        ]
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'comprehensive-tls-ingress', uid: 'ingress-808' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: tlsSpec
      }

      const extractedSpec = getGenericSpec(ingressResource)
      expect(extractedSpec.tls).toHaveLength(3)
      expect(extractedSpec.tls[0].secretName).toBe('tls-cert-1')
      expect(extractedSpec.tls[1].secretName).toBe('tls-cert-2')
      expect(extractedSpec.tls[2].secretName).toBe('wildcard-cert')
      expect(extractedSpec.tls[2].hosts).toEqual(['*.example-company.com'])
    })
  })

  describe('Edge Cases', () => {
    it('handles ingress with empty object gracefully', () => {
      const ingressResource: K8sListItem = {
        metadata: {},
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1'
      }

      expect(() => getStatusText(ingressResource)).not.toThrow()
      expect(getStatusText(ingressResource)).toBe('Pending')
    })

    it('handles malformed ingress spec gracefully', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'malformed-ingress', uid: 'ingress-999' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: null as any
      }

      expect(getGenericSpec(ingressResource)).toBeNull()
      expect(() => getStatusText(ingressResource)).not.toThrow()
    })

    it('handles ingress with empty load balancer ingress array', () => {
      const ingressResource: K8sListItem = {
        metadata: { name: 'empty-lb-ingress', uid: 'ingress-1000' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: {
          ingressClassName: 'nginx'
        },
        ingressStatus: {
          loadBalancer: {
            ingress: []
          }
        }
      }

      expect(getStatusText(ingressResource)).toBe('Pending')
    })

    it('handles ingress with catch-all host rules', () => {
      const catchAllSpec = {
        ingressClassName: 'nginx-catch-all',
        rules: [
          {
            // No host specified - catch-all rule
            http: {
              paths: [
                {
                  path: '/',
                  pathType: 'Prefix',
                  backend: {
                    service: {
                      name: 'catch-all-service',
                      port: { number: 80 }
                    }
                  }
                }
              ]
            }
          }
        ]
      }

      const ingressResource: K8sListItem = {
        metadata: { name: 'catch-all-ingress', uid: 'ingress-1001' },
        kind: 'Ingress',
        apiVersion: 'networking.k8s.io/v1',
        ingressSpec: catchAllSpec
      }

      const extractedSpec = getGenericSpec(ingressResource)
      expect(extractedSpec.rules).toHaveLength(1)
      expect(extractedSpec.rules[0].host).toBeUndefined()
      expect(extractedSpec.rules[0].http.paths[0].backend.service.name).toBe('catch-all-service')
    })
  })
})