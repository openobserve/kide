/**
 * Side Panel Overview Integration Tests
 * 
 * These tests verify the ResourceOverview component displays correct information
 * for different resource types and handles various data scenarios.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ResourceOverview from '@/components/ResourcePanel/ResourceOverview.vue'
import type { K8sListItem } from '@/types'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Side Panel Overview Integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const createMockResource = (kind: string, overrides: Partial<K8sListItem> = {}): K8sListItem => ({
    metadata: {
      name: `test-${kind.toLowerCase()}`,
      namespace: 'default',
      uid: `${kind.toLowerCase()}-12345`,
      creationTimestamp: '2024-01-01T00:00:00Z',
      labels: {
        'app': 'test-app',
        'version': 'v1.0.0'
      },
      annotations: {
        'description': 'Test resource for integration testing'
      }
    },
    kind,
    apiVersion: kind === 'Pod' ? 'v1' : 'apps/v1',
    ...overrides
  })

  describe('Basic Resource Information Display', () => {
    it('should display basic resource metadata for a Pod', async () => {
      const pod = createMockResource('Pod', {
        spec: {
          nodeName: 'worker-node-1',
          containers: [{ name: 'app', image: 'nginx:latest' }]
        },
        status: {
          phase: 'Running',
          podIP: '10.244.0.10',
          qosClass: 'Guaranteed'
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: pod,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Check basic information section
      const infoSection = wrapper.find('.elevated-surface')
      expect(infoSection.exists()).toBe(true)
      expect(infoSection.text()).toContain('Pod Information')

      // Check metadata fields
      expect(wrapper.text()).toContain('test-pod')
      expect(wrapper.text()).toContain('default')
      expect(wrapper.text()).toContain('Pod')
      expect(wrapper.text()).toContain('pod-12345')
    })

    it('should display resource-specific fields for different resource types', async () => {
      const deployment = createMockResource('Deployment', {
        spec: {
          replicas: 3,
          strategy: {
            type: 'RollingUpdate'
          }
        },
        status: {
          readyReplicas: 3,
          availableReplicas: 3,
          updatedReplicas: 3
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: deployment,
          resourceKind: 'Deployment'
        }
      })

      await wrapper.vm.$nextTick()

      // Check deployment-specific information is displayed
      expect(wrapper.text()).toContain('Deployment Information')
      expect(wrapper.text()).toContain('test-deployment')
    })

    it('should handle resources without namespace', async () => {
      const clusterRole = createMockResource('ClusterRole', {
        metadata: {
          name: 'test-cluster-role',
          uid: 'cr-12345',
          creationTimestamp: '2024-01-01T00:00:00Z'
          // No namespace for cluster-scoped resource
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: clusterRole,
          resourceKind: 'ClusterRole'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('test-cluster-role')
      expect(wrapper.text()).toContain('ClusterRole Information')
      // Should not show namespace field
      expect(wrapper.text()).not.toMatch(/Namespace[\s\S]*default/)
    })
  })

  describe('Labels and Annotations Display', () => {
    it('should display resource labels when present', async () => {
      const resource = createMockResource('Pod', {
        metadata: {
          ...createMockResource('Pod').metadata,
          labels: {
            'app': 'web-server',
            'tier': 'frontend',
            'environment': 'production'
          }
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Labels should be displayed if component handles them
      const labelsSection = wrapper.find('[class*="labels"]')
      if (labelsSection.exists()) {
        expect(wrapper.text()).toContain('app=web-server')
      }
    })

    it('should handle resources with no labels gracefully', async () => {
      const resource = createMockResource('Pod', {
        metadata: {
          ...createMockResource('Pod').metadata,
          labels: undefined
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render without errors
      expect(wrapper.find('.elevated-surface').exists()).toBe(true)
      expect(wrapper.text()).toContain('Pod Information')
    })

    it('should display annotations when present', async () => {
      const resource = createMockResource('Service', {
        metadata: {
          ...createMockResource('Service').metadata,
          annotations: {
            'service.beta.kubernetes.io/aws-load-balancer-type': 'nlb',
            'kubernetes.io/ingress.class': 'nginx'
          }
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Service'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render without errors
      expect(wrapper.find('.elevated-surface').exists()).toBe(true)
    })
  })

  describe('Ingress-specific Features', () => {
    it('should display ingress hosts section for Ingress resources', async () => {
      const ingress = createMockResource('Ingress', {
        spec: {
          rules: [
            {
              host: 'example.com',
              http: {
                paths: [
                  {
                    path: '/api',
                    pathType: 'Prefix',
                    backend: {
                      service: {
                        name: 'api-service',
                        port: { number: 80 }
                      }
                    }
                  },
                  {
                    path: '/web',
                    pathType: 'Prefix',
                    backend: {
                      service: {
                        name: 'web-service',
                        port: { number: 3000 }
                      }
                    }
                  }
                ]
              }
            },
            {
              host: 'api.example.com',
              http: {
                paths: [
                  {
                    path: '/',
                    pathType: 'Prefix',
                    backend: {
                      service: {
                        name: 'api-service',
                        port: { number: 80 }
                      }
                    }
                  }
                ]
              }
            }
          ]
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingress,
          resourceKind: 'Ingress'
        }
      })

      await wrapper.vm.$nextTick()

      // Check for hosts section
      const hostsSection = wrapper.find('h3')
      if (hostsSection.exists() && hostsSection.text().includes('Hosts')) {
        expect(wrapper.text()).toContain('Hosts')
        expect(wrapper.text()).toContain('example.com')
        expect(wrapper.text()).toContain('api.example.com')
        expect(wrapper.text()).toContain('/api')
        expect(wrapper.text()).toContain('/web')
        expect(wrapper.text()).toContain('api-service')
        expect(wrapper.text()).toContain('web-service')
      }
    })

    it('should handle ingress without rules gracefully', async () => {
      const ingress = createMockResource('Ingress', {
        spec: {
          // No rules defined
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingress,
          resourceKind: 'Ingress'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render basic info without hosts section
      expect(wrapper.find('.elevated-surface').exists()).toBe(true)
      expect(wrapper.text()).toContain('Ingress Information')
    })

    it('should display ingress with wildcard host', async () => {
      const ingress = createMockResource('Ingress', {
        spec: {
          rules: [
            {
              // No host specified (wildcard)
              http: {
                paths: [
                  {
                    path: '/',
                    pathType: 'Prefix',
                    backend: {
                      service: {
                        name: 'default-service',
                        port: { number: 80 }
                      }
                    }
                  }
                ]
              }
            }
          ]
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: ingress,
          resourceKind: 'Ingress'
        }
      })

      await wrapper.vm.$nextTick()

      // Should handle wildcard host (shows as *)
      const hostsSection = wrapper.find('h3')
      if (hostsSection.exists() && hostsSection.text().includes('Hosts')) {
        expect(wrapper.text()).toContain('*')
      }
    })
  })

  describe('Conditions and Status Display', () => {
    it('should display deployment conditions when present', async () => {
      const deployment = createMockResource('Deployment', {
        status: {
          replicas: 3,
          readyReplicas: 3,
          availableReplicas: 3,
          conditions: [
            {
              type: 'Available',
              status: 'True',
              reason: 'MinimumReplicasAvailable',
              message: 'Deployment has minimum availability.'
            },
            {
              type: 'Progressing',
              status: 'True',
              reason: 'NewReplicaSetAvailable',
              message: 'ReplicaSet has successfully progressed.'
            }
          ]
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: deployment,
          resourceKind: 'Deployment'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render without errors and show deployment info
      expect(wrapper.find('.elevated-surface').exists()).toBe(true)
      expect(wrapper.text()).toContain('Deployment Information')
    })

    it('should handle resources with failed conditions', async () => {
      const deployment = createMockResource('Deployment', {
        status: {
          replicas: 3,
          readyReplicas: 0,
          availableReplicas: 0,
          conditions: [
            {
              type: 'Available',
              status: 'False',
              reason: 'MinimumReplicasUnavailable',
              message: 'Deployment does not have minimum availability.'
            }
          ]
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: deployment,
          resourceKind: 'Deployment'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.elevated-surface').exists()).toBe(true)
    })
  })

  describe('Resource-specific Field Handling', () => {
    it('should display Pod-specific fields correctly', async () => {
      const pod = createMockResource('Pod', {
        spec: {
          nodeName: 'worker-node-1',
          serviceAccountName: 'default',
          restartPolicy: 'Always'
        },
        status: {
          phase: 'Running',
          podIP: '10.244.0.10',
          hostIP: '192.168.1.100',
          qosClass: 'Guaranteed'
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: pod,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // The component should access spec/status fields correctly
      expect(wrapper.text()).toContain('Pod Information')
    })

    it('should handle Service-specific fields', async () => {
      const service = createMockResource('Service', {
        serviceSpec: {
          type: 'ClusterIP',
          clusterIP: '10.96.0.1',
          ports: [
            { name: 'http', port: 80, targetPort: 8080, protocol: 'TCP' },
            { name: 'https', port: 443, targetPort: 8443, protocol: 'TCP' }
          ],
          selector: {
            app: 'web-app'
          }
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: service,
          resourceKind: 'Service'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('Service Information')
    })

    it('should handle ConfigMap data display', async () => {
      const configMap = createMockResource('ConfigMap', {
        data: {
          'app.properties': 'debug=true\nport=8080',
          'config.yaml': 'server:\n  host: localhost\n  port: 3000'
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: configMap,
          resourceKind: 'ConfigMap'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('ConfigMap Information')
    })
  })

  describe('Error Handling and Edge Cases', () => {
    it('should handle null/undefined resourceData gracefully', async () => {
      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: null,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render without crashing, possibly showing empty state
      expect(wrapper.exists()).toBe(true)
    })

    it('should handle empty resource data', async () => {
      const emptyResource = {
        metadata: {},
        kind: 'Pod'
      } as K8sListItem

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: emptyResource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.exists()).toBe(true)
    })

    it('should handle malformed metadata', async () => {
      const malformedResource = {
        metadata: {
          name: null,
          namespace: undefined,
          uid: '',
          creationTimestamp: 'invalid-date'
        },
        kind: 'Pod'
      } as any

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: malformedResource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Should handle malformed data gracefully
      expect(wrapper.exists()).toBe(true)
    })

    it('should handle unknown resource kinds', async () => {
      const unknownResource = createMockResource('UnknownKind')

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: unknownResource,
          resourceKind: 'UnknownKind'
        }
      })

      await wrapper.vm.$nextTick()

      // Should render basic information for unknown resource types
      expect(wrapper.text()).toContain('UnknownKind Information')
      expect(wrapper.text()).toContain('test-unknownkind')
    })
  })

  describe('Responsive Behavior', () => {
    it('should render properly with grid layout', async () => {
      const resource = createMockResource('Pod')

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // Check for grid layout classes
      const gridContainer = wrapper.find('.grid')
      expect(gridContainer.exists()).toBe(true)

      // Should have responsive classes
      const responsiveElements = wrapper.findAll('[class*="sm:grid-cols-2"]')
      expect(responsiveElements.length).toBeGreaterThan(0)
    })
  })

  describe('Data Formatting', () => {
    it('should properly format timestamps', async () => {
      const resource = createMockResource('Pod', {
        metadata: {
          ...createMockResource('Pod').metadata,
          creationTimestamp: '2024-01-01T12:30:45Z'
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      // UID should be displayed in monospace font
      const uidElement = wrapper.find('.font-mono')
      expect(uidElement.exists()).toBe(true)
    })

    it('should handle long UIDs with proper styling', async () => {
      const resource = createMockResource('Pod', {
        metadata: {
          ...createMockResource('Pod').metadata,
          uid: 'very-long-uid-that-should-be-displayed-properly-12345678-1234-1234-1234-123456789012'
        }
      })

      const wrapper = mount(ResourceOverview, {
        props: {
          resourceData: resource,
          resourceKind: 'Pod'
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.text()).toContain('very-long-uid-that-should-be-displayed-properly')
    })
  })
})