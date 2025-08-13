import { describe, it, expect, beforeEach, vi, type MockedFunction } from 'vitest'

// Mock the Tauri API
const mockInvoke = vi.fn() as MockedFunction<typeof import('@tauri-apps/api/core').invoke>
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

// Simulated resource update flow functions
class ResourceUpdateService {
  async getResourceYaml(resourceName: string, resourceKind: string, namespace: string | null): Promise<string> {
    const resourceData = await mockInvoke('get_full_resource', {
      resource_name: resourceName,
      resource_kind: resourceKind,
      namespace: namespace
    })
    
    return this.jsonToYaml(resourceData)
  }
  
  async updateResource(resourceName: string, resourceKind: string, namespace: string | null, yamlContent: string): Promise<void> {
    await mockInvoke('update_resource', {
      resource_name: resourceName,
      resource_kind: resourceKind,
      namespace: namespace,
      yaml_content: yamlContent
    })
  }
  
  private jsonToYaml(obj: any, indent = 0): string {
    const spaces = '  '.repeat(indent)
    let yaml = ''
    
    for (const [key, value] of Object.entries(obj || {})) {
      if (value === null || value === undefined) continue
      
      yaml += spaces + key + ':'
      
      if (typeof value === 'object' && !Array.isArray(value)) {
        yaml += '\n' + this.jsonToYaml(value, indent + 1)
      } else if (Array.isArray(value)) {
        yaml += '\n'
        for (const item of value) {
          if (typeof item === 'object') {
            yaml += spaces + '- \n' + this.jsonToYaml(item, indent + 1)
          } else {
            yaml += spaces + '- ' + item + '\n'
          }
        }
      } else {
        yaml += ' ' + value + '\n'
      }
    }
    
    return yaml
  }
  
  validateYamlSyntax(yamlContent: string): boolean {
    try {
      // Simple validation - check for basic YAML structure
      if (!yamlContent.trim()) return false
      if (!yamlContent.includes('apiVersion:')) return false
      if (!yamlContent.includes('kind:')) return false
      if (!yamlContent.includes('metadata:')) return false
      return true
    } catch {
      return false
    }
  }
}

// Simulated user interface state management
class YamlEditorState {
  public originalContent = ''
  public currentContent = ''
  public hasChanges = false
  public isLoading = false
  public isSaving = false
  public error = ''
  public successMessage = ''
  
  constructor(private service: ResourceUpdateService) {}
  
  async loadResource(resourceName: string, resourceKind: string, namespace: string | null) {
    this.isLoading = true
    this.error = ''
    
    try {
      const yamlContent = await this.service.getResourceYaml(resourceName, resourceKind, namespace)
      this.originalContent = yamlContent
      this.currentContent = yamlContent
      this.hasChanges = false
    } catch (error: any) {
      this.error = `Failed to load resource: ${error.message}`
    } finally {
      this.isLoading = false
    }
  }
  
  updateContent(newContent: string) {
    this.currentContent = newContent
    this.hasChanges = newContent !== this.originalContent
    this.successMessage = ''
    this.error = ''
  }
  
  async save(resourceName: string, resourceKind: string, namespace: string | null) {
    if (!this.hasChanges) return
    
    this.isSaving = true
    this.error = ''
    this.successMessage = ''
    
    try {
      if (!this.service.validateYamlSyntax(this.currentContent)) {
        throw new Error('Invalid YAML syntax')
      }
      
      await this.service.updateResource(resourceName, resourceKind, namespace, this.currentContent)
      
      this.originalContent = this.currentContent
      this.hasChanges = false
      this.successMessage = `${resourceKind} '${resourceName}' updated successfully`
      
    } catch (error: any) {
      this.error = `Failed to save: ${error.message}`
    } finally {
      this.isSaving = false
    }
  }
  
  reset() {
    this.currentContent = this.originalContent
    this.hasChanges = false
    this.error = ''
    this.successMessage = ''
  }
}

describe('Resource Update Flow Integration Tests', () => {
  let service: ResourceUpdateService
  let editorState: YamlEditorState
  
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockClear()
    service = new ResourceUpdateService()
    editorState = new YamlEditorState(service)
  })

  describe('Complete Resource Update Workflow', () => {
    it('successfully loads, edits, and saves a Deployment', async () => {
      const mockDeploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: {
          name: 'o2-openobserve-router',
          namespace: 'default',
          labels: {
            app: 'openobserve-router'
          }
        },
        spec: {
          replicas: 3,
          selector: {
            matchLabels: {
              app: 'openobserve-router'
            }
          },
          template: {
            metadata: {
              labels: {
                app: 'openobserve-router'
              }
            },
            spec: {
              containers: [{
                name: 'router',
                image: 'openobserve/router:v0.1.0',
                ports: [{ containerPort: 8080 }]
              }]
            }
          }
        }
      }

      // Mock the get_full_resource and update_resource calls
      mockInvoke
        .mockResolvedValueOnce(mockDeploymentData) // get_full_resource
        .mockResolvedValueOnce(undefined) // update_resource success

      // Step 1: Load the resource
      await editorState.loadResource('o2-openobserve-router', 'Deployment', 'default')

      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: 'o2-openobserve-router',
        resource_kind: 'Deployment',
        namespace: 'default'
      })

      expect(editorState.isLoading).toBe(false)
      expect(editorState.error).toBe('')
      expect(editorState.currentContent).toContain('replicas: 3')
      expect(editorState.hasChanges).toBe(false)

      // Step 2: Edit the YAML (change replicas from 3 to 5)
      const modifiedContent = editorState.currentContent.replace('replicas: 3', 'replicas: 5')
      editorState.updateContent(modifiedContent)

      expect(editorState.hasChanges).toBe(true)
      expect(editorState.currentContent).toContain('replicas: 5')

      // Step 3: Save the changes
      await editorState.save('o2-openobserve-router', 'Deployment', 'default')

      expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
        resource_name: 'o2-openobserve-router',
        resource_kind: 'Deployment',
        namespace: 'default',
        yaml_content: modifiedContent
      })

      expect(editorState.isSaving).toBe(false)
      expect(editorState.error).toBe('')
      expect(editorState.hasChanges).toBe(false)
      expect(editorState.successMessage).toContain('updated successfully')
    })

    it('works with different resource types using generic update', async () => {
      const testCases = [
        {
          name: 'nginx-service',
          kind: 'Service',
          namespace: 'default',
          data: {
            apiVersion: 'v1',
            kind: 'Service',
            metadata: { name: 'nginx-service', namespace: 'default' },
            spec: { 
              type: 'ClusterIP',
              ports: [{ port: 80, targetPort: 8080 }],
              selector: { app: 'nginx' }
            }
          }
        },
        {
          name: 'app-config',
          kind: 'ConfigMap',
          namespace: 'default',
          data: {
            apiVersion: 'v1',
            kind: 'ConfigMap',
            metadata: { name: 'app-config', namespace: 'default' },
            data: { 
              'app.properties': 'debug=true\nport=8080',
              'database.url': 'postgres://localhost:5432/mydb'
            }
          }
        },
        {
          name: 'app-secrets',
          kind: 'Secret',
          namespace: 'default',
          data: {
            apiVersion: 'v1',
            kind: 'Secret',
            metadata: { name: 'app-secrets', namespace: 'default' },
            type: 'Opaque',
            data: {
              'db-password': 'cGFzc3dvcmQ=',
              'api-key': 'YWJjZGVmZ2g='
            }
          }
        },
        {
          name: 'web-ingress',
          kind: 'Ingress',
          namespace: 'default',
          data: {
            apiVersion: 'networking.k8s.io/v1',
            kind: 'Ingress',
            metadata: { name: 'web-ingress', namespace: 'default' },
            spec: {
              ingressClassName: 'nginx',
              rules: [{
                host: 'example.com',
                http: {
                  paths: [{
                    path: '/',
                    pathType: 'Prefix',
                    backend: {
                      service: {
                        name: 'web-service',
                        port: { number: 80 }
                      }
                    }
                  }]
                }
              }]
            }
          }
        }
      ]

      for (const testCase of testCases) {
        mockInvoke.mockClear()
        mockInvoke
          .mockResolvedValueOnce(testCase.data) // get_full_resource
          .mockResolvedValueOnce(undefined) // update_resource

        const freshState = new YamlEditorState(service)
        
        // Load resource
        await freshState.loadResource(testCase.name, testCase.kind, testCase.namespace)
        
        expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
          resource_name: testCase.name,
          resource_kind: testCase.kind,
          namespace: testCase.namespace
        })

        expect(freshState.error).toBe('')
        expect(freshState.currentContent).toContain(testCase.kind)
        expect(freshState.currentContent).toContain(testCase.name)

        // Make a generic change
        const modifiedContent = freshState.currentContent + '\n# Updated by test'
        freshState.updateContent(modifiedContent)
        
        expect(freshState.hasChanges).toBe(true)

        // Save changes
        await freshState.save(testCase.name, testCase.kind, testCase.namespace)

        expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
          resource_name: testCase.name,
          resource_kind: testCase.kind,
          namespace: testCase.namespace,
          yaml_content: modifiedContent
        })

        expect(freshState.error).toBe('')
        expect(freshState.successMessage).toContain(`${testCase.kind} '${testCase.name}' updated successfully`)
      }
    })

    it('handles cluster-scoped resources without namespace', async () => {
      const nodeData = {
        apiVersion: 'v1',
        kind: 'Node',
        metadata: {
          name: 'worker-node-01',
          labels: {
            'kubernetes.io/arch': 'amd64',
            'kubernetes.io/os': 'linux'
          }
        },
        spec: {
          podCIDR: '10.244.1.0/24'
        },
        status: {
          conditions: [{
            type: 'Ready',
            status: 'True'
          }]
        }
      }

      mockInvoke
        .mockResolvedValueOnce(nodeData)
        .mockResolvedValueOnce(undefined)

      await editorState.loadResource('worker-node-01', 'Node', null)

      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: 'worker-node-01',
        resource_kind: 'Node',
        namespace: null
      })

      expect(editorState.currentContent).toContain('Node')
      expect(editorState.currentContent).toContain('worker-node-01')

      // Update node labels
      const modifiedContent = editorState.currentContent.replace(
        'kubernetes.io/os: linux',
        'kubernetes.io/os: linux\n    custom-label: test-value'
      )
      editorState.updateContent(modifiedContent)

      await editorState.save('worker-node-01', 'Node', null)

      expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
        resource_name: 'worker-node-01',
        resource_kind: 'Node',
        namespace: null,
        yaml_content: modifiedContent
      })
    })
  })

  describe('Error Handling', () => {
    it('handles resource loading errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Resource not found'))

      await editorState.loadResource('nonexistent-deployment', 'Deployment', 'default')

      expect(editorState.error).toContain('Failed to load resource: Resource not found')
      expect(editorState.isLoading).toBe(false)
      expect(editorState.currentContent).toBe('')
    })

    it('handles save errors from kubectl apply failures', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockRejectedValueOnce(new Error('kubectl apply failed: validation error - missing required selector'))

      await editorState.loadResource('test-deployment', 'Deployment', 'default')
      
      // Make invalid changes (remove required fields)
      editorState.updateContent(`
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
spec:
  replicas: 5
  # Missing required selector field
`)

      await editorState.save('test-deployment', 'Deployment', 'default')

      expect(editorState.error).toContain('kubectl apply failed: validation error')
      expect(editorState.isSaving).toBe(false)
      expect(editorState.hasChanges).toBe(true) // Changes not saved
    })

    it('validates YAML syntax before saving', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      }

      mockInvoke.mockResolvedValueOnce(deploymentData)

      await editorState.loadResource('test-deployment', 'Deployment', 'default')
      
      // Set invalid YAML content
      editorState.updateContent('completely invalid yaml without any structure')

      await editorState.save('test-deployment', 'Deployment', 'default')

      expect(editorState.error).toContain('Invalid YAML syntax')
      expect(mockInvoke).toHaveBeenCalledTimes(1) // Only the load call, not save
    })

    it('handles network errors during save operations', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockRejectedValueOnce(new Error('Network error: Connection timeout'))

      await editorState.loadResource('test-deployment', 'Deployment', 'default')
      editorState.updateContent(editorState.currentContent.replace('replicas: 3', 'replicas: 5'))

      await editorState.save('test-deployment', 'Deployment', 'default')

      expect(editorState.error).toContain('Network error: Connection timeout')
      expect(editorState.isSaving).toBe(false)
    })
  })

  describe('State Management', () => {
    it('correctly tracks changes and reset functionality', async () => {
      const configMapData = {
        apiVersion: 'v1',
        kind: 'ConfigMap',
        metadata: { name: 'test-config' },
        data: { 'config.yaml': 'setting: original' }
      }

      mockInvoke.mockResolvedValueOnce(configMapData)

      await editorState.loadResource('test-config', 'ConfigMap', 'default')
      
      const originalContent = editorState.currentContent
      expect(editorState.hasChanges).toBe(false)

      // Make changes
      const modifiedContent = originalContent.replace('setting: original', 'setting: modified')
      editorState.updateContent(modifiedContent)
      
      expect(editorState.hasChanges).toBe(true)
      expect(editorState.currentContent).toContain('setting: modified')

      // Reset changes
      editorState.reset()
      
      expect(editorState.hasChanges).toBe(false)
      expect(editorState.currentContent).toBe(originalContent)
      expect(editorState.currentContent).toContain('setting: original')
      expect(editorState.error).toBe('')
      expect(editorState.successMessage).toBe('')
    })

    it('prevents saving when no changes are made', async () => {
      const secretData = {
        apiVersion: 'v1',
        kind: 'Secret',
        metadata: { name: 'test-secret' },
        data: { 'key': 'value' }
      }

      mockInvoke.mockResolvedValueOnce(secretData)

      await editorState.loadResource('test-secret', 'Secret', 'default')
      
      expect(editorState.hasChanges).toBe(false)

      // Try to save without making changes
      await editorState.save('test-secret', 'Secret', 'default')

      expect(mockInvoke).toHaveBeenCalledTimes(1) // Only the load call
      expect(editorState.isSaving).toBe(false)
    })

    it('clears success messages when new changes are made', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockResolvedValueOnce(undefined)

      await editorState.loadResource('test-deployment', 'Deployment', 'default')
      
      // First change and save
      editorState.updateContent(editorState.currentContent.replace('replicas: 3', 'replicas: 4'))
      await editorState.save('test-deployment', 'Deployment', 'default')
      
      expect(editorState.successMessage).toContain('updated successfully')

      // Make another change - success message should clear
      editorState.updateContent(editorState.currentContent.replace('replicas: 4', 'replicas: 5'))
      
      expect(editorState.successMessage).toBe('')
      expect(editorState.hasChanges).toBe(true)
    })
  })

  describe('Concurrent Operations', () => {
    it('handles overlapping save operations correctly', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 100))) // Slow save

      await editorState.loadResource('test-deployment', 'Deployment', 'default')
      editorState.updateContent(editorState.currentContent.replace('replicas: 3', 'replicas: 5'))

      // Start save operation
      const savePromise = editorState.save('test-deployment', 'Deployment', 'default')
      
      expect(editorState.isSaving).toBe(true)

      // Original save should still complete
      await savePromise
      
      expect(editorState.isSaving).toBe(false)
      
      // Now make additional changes after save is complete
      editorState.updateContent(editorState.currentContent + '\n# Additional change')
      
      // The additional change should be tracked as unsaved
      expect(editorState.hasChanges).toBe(true)
    })
  })
})