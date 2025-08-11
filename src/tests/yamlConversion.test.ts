import { describe, it, expect } from 'vitest'
import * as yaml from 'js-yaml'

describe('YAML Conversion Tests', () => {
  it('should preserve apiVersion field when converting from object to YAML', () => {
    const resourceData = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      },
      spec: {
        replicas: 3
      }
    }

    const yamlContent = yaml.dump(resourceData, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    // Parse it back to verify
    const parsed = yaml.load(yamlContent) as any

    expect(parsed.apiVersion).toBe('apps/v1')
    expect(parsed.kind).toBe('Deployment')
    expect(parsed.metadata.name).toBe('test-deployment')
    
    // Check that the YAML string contains apiVersion
    expect(yamlContent).toContain('apiVersion: apps/v1')
    expect(yamlContent).toContain('kind: Deployment')
  })

  it('should handle missing apiVersion field', () => {
    const resourceData = {
      // Missing apiVersion!
      kind: 'Deployment',
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      },
      spec: {
        replicas: 3
      }
    }

    const yamlContent = yaml.dump(resourceData, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    const parsed = yaml.load(yamlContent) as any

    // apiVersion should be undefined if not provided
    expect(parsed.apiVersion).toBeUndefined()
    expect(yamlContent).not.toContain('apiVersion:')
  })

  it('should preserve all fields when converting complex deployment', () => {
    const complexDeployment = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'httptester',
        namespace: 'default',
        uid: 'fee38dd1-e826-4da3-af27-f31b4720fe2f',
        creationTimestamp: '2025-02-07T12:20:44+00:00',
        labels: null,
        annotations: {
          'deployment.kubernetes.io/revision': '1'
        }
      },
      spec: {
        replicas: 1,
        selector: {
          matchLabels: {
            app: 'httptester'
          }
        },
        template: {
          metadata: {
            labels: {
              app: 'httptester'
            }
          },
          spec: {
            containers: [{
              name: 'httptester',
              image: 'public.ecr.aws/zinclabs/httptester:v0.0.1-87113f4',
              ports: [{
                containerPort: 6080,
                protocol: 'TCP'
              }]
            }]
          }
        }
      },
      status: {
        readyReplicas: 1,
        replicas: 1
      }
    }

    const yamlContent = yaml.dump(complexDeployment, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    const parsed = yaml.load(yamlContent) as any

    // Verify critical fields are preserved
    expect(parsed.apiVersion).toBe('apps/v1')
    expect(parsed.kind).toBe('Deployment')
    expect(parsed.metadata.name).toBe('httptester')
    expect(parsed.spec.replicas).toBe(1)
    expect(parsed.spec.selector.matchLabels.app).toBe('httptester')
    
    // Verify YAML content includes apiVersion
    expect(yamlContent).toMatch(/^apiVersion: apps\/v1$/m)
    expect(yamlContent).toMatch(/^kind: Deployment$/m)
  })

  it('should handle resource data with camelCase apiVersion field', () => {
    // Test if the data has apiVersion in camelCase (from backend)
    const resourceData = {
      apiVersion: 'apps/v1',  // This should remain as apiVersion
      kind: 'Deployment',
      metadata: {
        name: 'test'
      }
    }

    const yamlContent = yaml.dump(resourceData, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    // Should preserve apiVersion (not convert to api_version)
    expect(yamlContent).toContain('apiVersion: apps/v1')
    expect(yamlContent).not.toContain('api_version')
  })

  it('should verify the exact structure expected by Kubernetes API', () => {
    const deployment = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      },
      spec: {
        replicas: 1,
        selector: {
          matchLabels: {
            app: 'test'
          }
        },
        template: {
          metadata: {
            labels: {
              app: 'test'
            }
          },
          spec: {
            containers: [{
              name: 'test',
              image: 'nginx:latest'
            }]
          }
        }
      }
    }

    const yamlContent = yaml.dump(deployment, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    // Verify the YAML starts with apiVersion and kind
    const lines = yamlContent.split('\n')
    expect(lines[0]).toBe('apiVersion: apps/v1')
    expect(lines[1]).toBe('kind: Deployment')
    
    // Parse and verify structure
    const parsed = yaml.load(yamlContent) as any
    expect(parsed).toHaveProperty('apiVersion')
    expect(parsed).toHaveProperty('kind')
    expect(parsed).toHaveProperty('metadata')
    expect(parsed).toHaveProperty('spec')
  })

  it('should preserve complete object data with all Kubernetes fields', () => {
    // Simulate a complete Kubernetes object with all fields
    const completeRoleBinding = {
      apiVersion: 'rbac.authorization.k8s.io/v1',
      kind: 'RoleBinding',
      metadata: {
        name: 'monitor-openobserve-actions',
        namespace: 'monitor',
        uid: '29a74884-9ec8-4b02-8153-c81839dd4652',
        resourceVersion: '326618717',
        creationTimestamp: '2025-05-27T10:47:31Z',
        labels: {
          'app.kubernetes.io/instance': 'monitor',
          'app.kubernetes.io/managed-by': 'Helm',
          'app.kubernetes.io/name': 'openobserve',
          'app.kubernetes.io/version': 'v0.14.7',
          'helm.sh/chart': 'openobserve-0.14.78'
        },
        annotations: {
          'kubectl.kubernetes.io/last-applied-configuration': '{"apiVersion":"rbac.authorization.k8s.io/v1","kind":"RoleBinding","metadata":{"annotations":{},"labels":{"app.kubernetes.io/instance":"monitor","app.kubernetes.io/managed-by":"Helm","app.kubernetes.io/name":"openobserve","app.kubernetes.io/version":"v0.14.7","helm.sh/chart":"openobserve-0.14.78"},"name":"monitor-openobserve-actions","namespace":"monitor"},"roleRef":{"apiGroup":"rbac.authorization.k8s.io","kind":"Role","name":"monitor-openobserve-actions"},"subjects":[{"kind":"ServiceAccount","name":"monitor-openobserve-actions","namespace":"monitor"}]}'
        },
        managedFields: [
          {
            manager: 'argocd-application-controller',
            operation: 'Update',
            apiVersion: 'rbac.authorization.k8s.io/v1',
            time: '2025-05-27T10:47:31Z',
            fieldsType: 'FieldsV1',
            fieldsV1: {
              'f:metadata': {
                'f:labels': {
                  '.': {},
                  'f:app.kubernetes.io/instance': {},
                  'f:app.kubernetes.io/managed-by': {},
                  'f:app.kubernetes.io/name': {},
                  'f:app.kubernetes.io/version': {}
                }
              },
              'f:roleRef': {},
              'f:subjects': {}
            }
          }
        ],
        selfLink: '/apis/rbac.authorization.k8s.io/v1/namespaces/monitor/rolebindings/monitor-openobserve-actions'
      },
      subjects: [
        {
          kind: 'ServiceAccount',
          name: 'monitor-openobserve-actions',
          namespace: 'monitor'
        }
      ],
      roleRef: {
        apiGroup: 'rbac.authorization.k8s.io',
        kind: 'Role',
        name: 'monitor-openobserve-actions'
      }
    }

    const yamlContent = yaml.dump(completeRoleBinding, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    const parsed = yaml.load(yamlContent) as any

    // Verify all critical fields are preserved
    expect(parsed.apiVersion).toBe('rbac.authorization.k8s.io/v1')
    expect(parsed.kind).toBe('RoleBinding')
    expect(parsed.metadata.name).toBe('monitor-openobserve-actions')
    expect(parsed.metadata.resourceVersion).toBe('326618717')
    expect(parsed.metadata.selfLink).toBe('/apis/rbac.authorization.k8s.io/v1/namespaces/monitor/rolebindings/monitor-openobserve-actions')
    expect(parsed.metadata.managedFields).toBeDefined()
    expect(parsed.subjects).toBeDefined()
    expect(parsed.roleRef).toBeDefined()
    
    // Verify YAML contains all the important fields
    expect(yamlContent).toContain('resourceVersion:')
    expect(yamlContent).toContain('selfLink:')
    expect(yamlContent).toContain('managedFields:')
    expect(yamlContent).toContain('subjects:')
    expect(yamlContent).toContain('roleRef:')
  })
})