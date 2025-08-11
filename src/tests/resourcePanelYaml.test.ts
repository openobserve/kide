import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed } from 'vue'
import * as yaml from 'js-yaml'

describe('ResourcePanel YAML Generation', () => {
  it('should generate YAML with apiVersion field', () => {
    // Simulate the yamlContent computed property from ResourcePanel
    const resourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default',
        uid: '123-456',
        creationTimestamp: '2025-01-01T00:00:00Z'
      },
      kind: 'Deployment',
      apiVersion: 'apps/v1',
      spec: {
        replicas: 3,
        selector: {
          matchLabels: { app: 'test' }
        }
      },
      status: {
        readyReplicas: 3
      }
    }

    // This mimics the yamlContent computed property
    const yamlContent = computed(() => {
      if (!resourceData) return ''
      
      try {
        return yaml.dump(resourceData, {
          indent: 2,
          noRefs: true,
          sortKeys: false
        })
      } catch (error) {
        console.error('Error converting to YAML:', error)
        return JSON.stringify(resourceData, null, 2)
      }
    })

    const generatedYaml = yamlContent.value
    
    // Verify the YAML contains apiVersion
    expect(generatedYaml).toContain('apiVersion: apps/v1')
    expect(generatedYaml).toContain('kind: Deployment')
    
    // Parse it back to verify structure
    const parsed = yaml.load(generatedYaml) as any
    expect(parsed.apiVersion).toBe('apps/v1')
    expect(parsed.kind).toBe('Deployment')
    expect(parsed.metadata.name).toBe('test-deployment')
  })

  it('should show what happens when apiVersion is missing', () => {
    const resourceDataWithoutApiVersion = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default',
        uid: '123-456',
        creationTimestamp: '2025-01-01T00:00:00Z'
      },
      kind: 'Deployment',
      // apiVersion is missing!
      spec: {
        replicas: 3
      }
    }

    const yamlContent = yaml.dump(resourceDataWithoutApiVersion, {
      indent: 2,
      noRefs: true,
      sortKeys: false
    })

    // The YAML should NOT contain apiVersion if it's not in the source
    expect(yamlContent).not.toContain('apiVersion:')
    
    // This would cause the Kubernetes API error we're seeing
    const parsed = yaml.load(yamlContent) as any
    expect(parsed.apiVersion).toBeUndefined()
  })

  it('should generate valid Kubernetes YAML for all resource types', () => {
    const testCases = [
      {
        resource: {
          apiVersion: 'v1',
          kind: 'Pod',
          metadata: { name: 'test-pod', namespace: 'default' },
          spec: { containers: [{ name: 'test', image: 'nginx' }] }
        },
        expectedApiVersion: 'v1'
      },
      {
        resource: {
          apiVersion: 'apps/v1',
          kind: 'Deployment',
          metadata: { name: 'test-deployment', namespace: 'default' },
          spec: { replicas: 1 }
        },
        expectedApiVersion: 'apps/v1'
      },
      {
        resource: {
          apiVersion: 'v1',
          kind: 'Service',
          metadata: { name: 'test-service', namespace: 'default' },
          spec: { type: 'ClusterIP' }
        },
        expectedApiVersion: 'v1'
      },
      {
        resource: {
          apiVersion: 'batch/v1',
          kind: 'Job',
          metadata: { name: 'test-job', namespace: 'default' },
          spec: { template: {} }
        },
        expectedApiVersion: 'batch/v1'
      }
    ]

    testCases.forEach(({ resource, expectedApiVersion }) => {
      const yamlContent = yaml.dump(resource, {
        indent: 2,
        noRefs: true,
        sortKeys: false
      })

      // Verify YAML contains correct apiVersion
      expect(yamlContent).toContain(`apiVersion: ${expectedApiVersion}`)
      expect(yamlContent).toContain(`kind: ${resource.kind}`)
      
      // Verify it parses back correctly
      const parsed = yaml.load(yamlContent) as any
      expect(parsed.apiVersion).toBe(expectedApiVersion)
      expect(parsed.kind).toBe(resource.kind)
    })
  })
})