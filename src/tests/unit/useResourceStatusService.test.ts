import { describe, it, expect } from 'vitest'
import { useResourceStatus } from '@/composables/useResourceStatus'
import type { K8sListItem } from '@/types'

describe('useResourceStatus - Service Integration Tests', () => {
  const { getStatusText, getStatusClass, getGenericSpec, getGenericStatus } = useResourceStatus()

  describe('Service Status Logic', () => {
    it('returns Ready for ClusterIP service', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-service', uid: 'service-123' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'ClusterIP',
          clusterIP: '10.0.0.1'
        }
      }

      expect(getStatusText(serviceResource)).toBe('Ready')
      expect(getStatusClass(serviceResource)).toBe('status-badge status-badge-success')
    })

    it('returns Ready for LoadBalancer service with external endpoints', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-lb-service', uid: 'service-456' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'LoadBalancer',
          clusterIP: '10.0.0.1'
        },
        serviceStatus: {
          loadBalancer: {
            ingress: [
              { ip: '203.0.113.1' }
            ]
          }
        }
      }

      expect(getStatusText(serviceResource)).toBe('Ready')
      expect(getStatusClass(serviceResource)).toBe('status-badge status-badge-success')
    })

    it('returns Pending for LoadBalancer service without external endpoints', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-lb-service', uid: 'service-789' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'LoadBalancer',
          clusterIP: '10.0.0.1'
        },
        serviceStatus: {
          loadBalancer: {}
        }
      }

      expect(getStatusText(serviceResource)).toBe('Pending')
      expect(getStatusClass(serviceResource)).toBe('status-badge status-badge-yellow')
    })

    it('returns Ready for NodePort service', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-nodeport-service', uid: 'service-101' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'NodePort',
          clusterIP: '10.0.0.1',
          ports: [
            {
              port: 80,
              targetPort: 8080,
              nodePort: 30080,
              protocol: 'TCP'
            }
          ]
        }
      }

      expect(getStatusText(serviceResource)).toBe('Ready')
      expect(getStatusClass(serviceResource)).toBe('status-badge status-badge-success')
    })

    it('returns Ready for ExternalName service', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-external-service', uid: 'service-202' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'ExternalName',
          externalName: 'example.com'
        }
      }

      expect(getStatusText(serviceResource)).toBe('Ready')
      expect(getStatusClass(serviceResource)).toBe('status-badge status-badge-success')
    })
  })

  describe('Service Spec and Status Extraction', () => {
    it('extracts serviceSpec correctly', () => {
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

      const serviceResource: K8sListItem = {
        metadata: { name: 'test-service', uid: 'service-303' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec
      }

      const extractedSpec = getGenericSpec(serviceResource)
      expect(extractedSpec).toEqual(serviceSpec)
      expect(extractedSpec.type).toBe('ClusterIP')
      expect(extractedSpec.clusterIP).toBe('172.20.159.8')
      expect(extractedSpec.sessionAffinity).toBe('None')
      expect(extractedSpec.internalTrafficPolicy).toBe('Cluster')
      expect(extractedSpec.ipFamilies).toEqual(['IPv4'])
      expect(extractedSpec.ipFamilyPolicy).toBe('SingleStack')
      expect(extractedSpec.ports).toHaveLength(1)
      expect(extractedSpec.ports[0].name).toBe('nats')
    })

    it('extracts serviceStatus correctly', () => {
      const serviceStatus = {
        loadBalancer: {
          ingress: [
            { ip: '203.0.113.1' },
            { hostname: 'example-lb-123456.us-west-2.elb.amazonaws.com' }
          ]
        }
      }

      const serviceResource: K8sListItem = {
        metadata: { name: 'test-service', uid: 'service-404' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'LoadBalancer'
        },
        serviceStatus
      }

      const extractedStatus = getGenericStatus(serviceResource)
      expect(extractedStatus).toEqual(serviceStatus)
      expect(extractedStatus.loadBalancer.ingress).toHaveLength(2)
      expect(extractedStatus.loadBalancer.ingress[0].ip).toBe('203.0.113.1')
    })

    it('returns null for non-Service resources', () => {
      const podResource: K8sListItem = {
        metadata: { name: 'test-pod', uid: 'pod-123' },
        kind: 'Pod',
        apiVersion: 'v1',
        podSpec: {
          containers: [{ name: 'test', image: 'nginx' }]
        }
      }

      // For Pod, getGenericSpec should return podSpec, not service-like spec
      expect(getGenericSpec(podResource)).toEqual({
        containers: [{ name: 'test', image: 'nginx' }]
      })
      // For Pod without podStatus, getGenericStatus should return undefined  
      expect(getGenericStatus(podResource)).toBeUndefined()
    })

    it('handles service with missing spec gracefully', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'test-service', uid: 'service-505' },
        kind: 'Service',
        apiVersion: 'v1'
        // No serviceSpec provided
      }

      expect(getGenericSpec(serviceResource)).toBeUndefined()
      expect(getGenericStatus(serviceResource)).toBeUndefined()
    })
  })

  describe('Complex Service Configurations', () => {
    it('handles dual-stack service configuration', () => {
      const dualStackSpec = {
        type: 'ClusterIP',
        clusterIP: '172.20.159.8',
        clusterIPs: ['172.20.159.8', '2001:db8::1'],
        ipFamilies: ['IPv4', 'IPv6'],
        ipFamilyPolicy: 'PreferDualStack',
        internalTrafficPolicy: 'Cluster'
      }

      const serviceResource: K8sListItem = {
        metadata: { name: 'dual-stack-service', uid: 'service-606' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: dualStackSpec
      }

      const extractedSpec = getGenericSpec(serviceResource)
      expect(extractedSpec.clusterIPs).toEqual(['172.20.159.8', '2001:db8::1'])
      expect(extractedSpec.ipFamilies).toEqual(['IPv4', 'IPv6'])
      expect(extractedSpec.ipFamilyPolicy).toBe('PreferDualStack')
    })

    it('handles service with ClientIP session affinity', () => {
      const sessionAffinitySpec = {
        type: 'ClusterIP',
        clusterIP: '10.0.0.1',
        sessionAffinity: 'ClientIP',
        sessionAffinityConfig: {
          clientIP: {
            timeoutSeconds: 10800
          }
        }
      }

      const serviceResource: K8sListItem = {
        metadata: { name: 'sticky-service', uid: 'service-707' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: sessionAffinitySpec
      }

      const extractedSpec = getGenericSpec(serviceResource)
      expect(extractedSpec.sessionAffinity).toBe('ClientIP')
      expect(extractedSpec.sessionAffinityConfig.clientIP.timeoutSeconds).toBe(10800)
    })

    it('handles LoadBalancer service with comprehensive configuration', () => {
      const loadBalancerSpec = {
        type: 'LoadBalancer',
        clusterIP: '10.0.0.1',
        externalTrafficPolicy: 'Local',
        internalTrafficPolicy: 'Cluster',
        allocateLoadBalancerNodePorts: true,
        loadBalancerClass: 'aws-load-balancer-controller',
        loadBalancerSourceRanges: ['10.0.0.0/8', '192.168.0.0/16'],
        healthCheckNodePort: 32000,
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
            name: 'https'
          }
        ]
      }

      const serviceResource: K8sListItem = {
        metadata: { name: 'advanced-lb-service', uid: 'service-808' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: loadBalancerSpec
      }

      const extractedSpec = getGenericSpec(serviceResource)
      expect(extractedSpec.type).toBe('LoadBalancer')
      expect(extractedSpec.externalTrafficPolicy).toBe('Local')
      expect(extractedSpec.allocateLoadBalancerNodePorts).toBe(true)
      expect(extractedSpec.loadBalancerClass).toBe('aws-load-balancer-controller')
      expect(extractedSpec.loadBalancerSourceRanges).toHaveLength(2)
      expect(extractedSpec.healthCheckNodePort).toBe(32000)
      expect(extractedSpec.ports).toHaveLength(2)
    })
  })

  describe('Edge Cases', () => {
    it('handles service with empty object gracefully', () => {
      const serviceResource: K8sListItem = {
        metadata: {},
        kind: 'Service',
        apiVersion: 'v1'
      }

      expect(() => getStatusText(serviceResource)).not.toThrow()
      expect(getStatusText(serviceResource)).toBe('Ready') // Default for services
    })

    it('handles malformed service spec gracefully', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'malformed-service', uid: 'service-999' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: null as any
      }

      expect(getGenericSpec(serviceResource)).toBeNull()
      expect(() => getStatusText(serviceResource)).not.toThrow()
    })

    it('handles service with partial LoadBalancer status', () => {
      const serviceResource: K8sListItem = {
        metadata: { name: 'partial-lb-service', uid: 'service-1000' },
        kind: 'Service',
        apiVersion: 'v1',
        serviceSpec: {
          type: 'LoadBalancer'
        },
        serviceStatus: {
          loadBalancer: {
            ingress: []
          }
        }
      }

      expect(getStatusText(serviceResource)).toBe('Pending')
    })
  })
})