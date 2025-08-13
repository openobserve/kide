import { describe, it, expect, beforeEach, vi, type MockedFunction } from 'vitest'

// Mock the Tauri API
const mockInvoke = vi.fn() as MockedFunction<typeof import('@tauri-apps/api/core').invoke>
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

// Simplified UI component simulation for testing the integration
interface ResourceUpdateUIProps {
  resourceName: string
  resourceKind: string
  namespace: string | null
}

class ResourceUpdateUI {
  public yamlContent = ''
  public originalYamlContent = ''
  public hasChanges = false
  public isLoading = false
  public isSaving = false
  public error = ''
  public successMessage = ''
  
  constructor(private props: ResourceUpdateUIProps) {}
  
  async loadResourceYaml() {
    this.isLoading = true
    this.error = ''
    this.successMessage = ''
    
    try {
      const resourceData = await mockInvoke('get_full_resource', {
        resource_name: this.props.resourceName,
        resource_kind: this.props.resourceKind,
        namespace: this.props.namespace
      })
      
      this.yamlContent = JSON.stringify(resourceData, null, 2) // Simplified YAML conversion
      this.originalYamlContent = this.yamlContent
      this.hasChanges = false
      
    } catch (error: any) {
      this.error = 'Failed to load resource: ' + error.message
    } finally {
      this.isLoading = false
    }
  }
  
  updateYamlContent(newContent: string) {
    this.yamlContent = newContent
    this.hasChanges = newContent !== this.originalYamlContent
    this.error = ''
    this.successMessage = ''
  }
  
  async saveChanges() {
    if (!this.hasChanges || this.isSaving) return
    
    this.isSaving = true
    this.error = ''
    this.successMessage = ''
    
    try {
      await mockInvoke('update_resource', {
        resource_name: this.props.resourceName,
        resource_kind: this.props.resourceKind,
        namespace: this.props.namespace,
        yaml_content: this.yamlContent
      })
      
      this.originalYamlContent = this.yamlContent
      this.hasChanges = false
      this.successMessage = `${this.props.resourceKind} '${this.props.resourceName}' updated successfully`
      
    } catch (error: any) {
      this.error = 'Failed to save changes: ' + error.message
    } finally {
      this.isSaving = false
    }
  }
  
  resetChanges() {
    this.yamlContent = this.originalYamlContent
    this.hasChanges = false
    this.error = ''
    this.successMessage = ''
  }
  
  // Simulate user interactions
  simulateUserEdit(searchText: string, replaceText: string) {
    const modifiedContent = this.yamlContent.replace(searchText, replaceText)
    this.updateYamlContent(modifiedContent)
  }
  
  // UI state helpers
  get canSave(): boolean {
    return this.hasChanges && !this.isSaving && !this.isLoading
  }
  
  get canReset(): boolean {
    return this.hasChanges && !this.isSaving
  }
  
  get saveButtonText(): string {
    return this.isSaving ? 'Saving...' : 'Save'
  }
}

describe('Resource Update UI Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockClear()
  })

  describe('Deployment Update Workflow', () => {
    it('simulates complete user workflow for updating deployment replicas', async () => {
      const mockDeploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: {
          name: 'o2-openobserve-router',
          namespace: 'default'
        },
        spec: {
          replicas: 3,
          selector: {
            matchLabels: { app: 'openobserve-router' }
          },
          template: {
            metadata: { labels: { app: 'openobserve-router' } },
            spec: {
              containers: [{
                name: 'router',
                image: 'openobserve/router:v0.1.0'
              }]
            }
          }
        }
      }

      // Setup mocks for successful workflow
      mockInvoke
        .mockResolvedValueOnce(mockDeploymentData) // get_full_resource
        .mockResolvedValueOnce(undefined) // update_resource success

      // Create UI component
      const ui = new ResourceUpdateUI({
        resourceName: 'o2-openobserve-router',
        resourceKind: 'Deployment', 
        namespace: 'default'
      })

      // Step 1: User opens YAML tab and loads resource
      expect(ui.isLoading).toBe(false)
      await ui.loadResourceYaml()

      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: 'o2-openobserve-router',
        resource_kind: 'Deployment',
        namespace: 'default'
      })

      expect(ui.isLoading).toBe(false)
      expect(ui.error).toBe('')
      expect(ui.yamlContent).toContain('"replicas": 3')
      expect(ui.hasChanges).toBe(false)
      expect(ui.canSave).toBe(false)

      // Step 2: User edits YAML to change replicas from 3 to 5
      ui.simulateUserEdit('"replicas": 3', '"replicas": 5')

      expect(ui.hasChanges).toBe(true)
      expect(ui.canSave).toBe(true)
      expect(ui.yamlContent).toContain('"replicas": 5')

      // Step 3: User clicks save button
      expect(ui.saveButtonText).toBe('Save')
      const savePromise = ui.saveChanges()
      
      expect(ui.isSaving).toBe(true)
      expect(ui.saveButtonText).toBe('Saving...')
      expect(ui.canSave).toBe(false)
      
      await savePromise

      expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
        resource_name: 'o2-openobserve-router',
        resource_kind: 'Deployment',
        namespace: 'default',
        yaml_content: ui.yamlContent
      })

      expect(ui.isSaving).toBe(false)
      expect(ui.hasChanges).toBe(false)
      expect(ui.error).toBe('')
      expect(ui.successMessage).toContain('updated successfully')
      expect(ui.canSave).toBe(false)
    })

    it('handles user cancellation with reset functionality', async () => {
      const mockServiceData = {
        apiVersion: 'v1',
        kind: 'Service',
        metadata: { name: 'test-service', namespace: 'default' },
        spec: {
          type: 'ClusterIP',
          ports: [{ port: 80, targetPort: 8080 }]
        }
      }

      mockInvoke.mockResolvedValueOnce(mockServiceData)

      const ui = new ResourceUpdateUI({
        resourceName: 'test-service',
        resourceKind: 'Service',
        namespace: 'default'
      })

      await ui.loadResourceYaml()
      const originalContent = ui.yamlContent

      // User makes changes
      ui.simulateUserEdit('"port": 80', '"port": 8080')
      
      expect(ui.hasChanges).toBe(true)
      expect(ui.canReset).toBe(true)
      expect(ui.yamlContent).toContain('"port": 8080')

      // User decides to cancel changes
      ui.resetChanges()

      expect(ui.hasChanges).toBe(false)
      expect(ui.canReset).toBe(false)
      expect(ui.yamlContent).toBe(originalContent)
      expect(ui.yamlContent).toContain('"port": 80')
      expect(ui.error).toBe('')
      expect(ui.successMessage).toBe('')
    })
  })

  describe('Error Handling in UI', () => {
    it('displays user-friendly error when resource fails to load', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Resource "missing-deployment" not found in namespace "default"'))

      const ui = new ResourceUpdateUI({
        resourceName: 'missing-deployment',
        resourceKind: 'Deployment',
        namespace: 'default'
      })

      await ui.loadResourceYaml()

      expect(ui.error).toContain('Failed to load resource')
      expect(ui.error).toContain('not found')
      expect(ui.isLoading).toBe(false)
      expect(ui.yamlContent).toBe('')
      expect(ui.canSave).toBe(false)
    })

    it('shows error when save fails due to validation issues', async () => {
      const configMapData = {
        apiVersion: 'v1',
        kind: 'ConfigMap',
        metadata: { name: 'test-config' },
        data: { 'app.properties': 'setting=value' }
      }

      mockInvoke
        .mockResolvedValueOnce(configMapData)
        .mockRejectedValueOnce(new Error('kubectl apply failed: validation error - invalid configuration'))

      const ui = new ResourceUpdateUI({
        resourceName: 'test-config',
        resourceKind: 'ConfigMap',
        namespace: 'default'
      })

      await ui.loadResourceYaml()
      
      // User makes invalid changes
      ui.simulateUserEdit('"setting=value"', '"invalid-syntax')

      await ui.saveChanges()

      expect(ui.error).toContain('Failed to save changes')
      expect(ui.error).toContain('validation error')
      expect(ui.isSaving).toBe(false)
      expect(ui.hasChanges).toBe(true) // Changes preserved for user to fix
      expect(ui.successMessage).toBe('')
    })

    it('handles network errors gracefully', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 2 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockRejectedValueOnce(new Error('Failed to execute kubectl apply: command not found'))

      const ui = new ResourceUpdateUI({
        resourceName: 'test-deployment',
        resourceKind: 'Deployment',
        namespace: 'default'
      })

      await ui.loadResourceYaml()
      ui.simulateUserEdit('"replicas": 2', '"replicas": 4')

      await ui.saveChanges()

      expect(ui.error).toContain('command not found')
      expect(ui.isSaving).toBe(false)
      expect(ui.successMessage).toBe('')
    })
  })

  describe('Multiple Resource Types Support', () => {
    const testResourceTypes = [
      {
        kind: 'Service',
        name: 'web-service',
        namespace: 'frontend',
        data: {
          apiVersion: 'v1',
          kind: 'Service',
          metadata: { name: 'web-service', namespace: 'frontend' },
          spec: { type: 'LoadBalancer', ports: [{ port: 443 }] }
        },
        editFrom: '"type": "LoadBalancer"',
        editTo: '"type": "NodePort"'
      },
      {
        kind: 'ConfigMap',
        name: 'app-settings',
        namespace: 'backend',
        data: {
          apiVersion: 'v1',
          kind: 'ConfigMap',
          metadata: { name: 'app-settings', namespace: 'backend' },
          data: { 'debug.enabled': 'false' }
        },
        editFrom: '"debug.enabled": "false"',
        editTo: '"debug.enabled": "true"'
      },
      {
        kind: 'Secret',
        name: 'database-creds',
        namespace: 'data',
        data: {
          apiVersion: 'v1',
          kind: 'Secret',
          metadata: { name: 'database-creds', namespace: 'data' },
          type: 'Opaque',
          data: { username: 'YWRtaW4=' }
        },
        editFrom: '"username": "YWRtaW4="',
        editTo: '"username": "dXNlcg=="'
      },
      {
        kind: 'Ingress',
        name: 'api-ingress',
        namespace: 'api',
        data: {
          apiVersion: 'networking.k8s.io/v1',
          kind: 'Ingress',
          metadata: { name: 'api-ingress', namespace: 'api' },
          spec: {
            ingressClassName: 'nginx',
            rules: [{
              host: 'api.example.com',
              http: {
                paths: [{
                  path: '/v1',
                  pathType: 'Prefix',
                  backend: { service: { name: 'api-v1', port: { number: 8080 } } }
                }]
              }
            }]
          }
        },
        editFrom: '"/v1"',
        editTo: '"/v2"'
      }
    ]

    testResourceTypes.forEach((resourceType) => {
      it(`handles ${resourceType.kind} resources correctly`, async () => {
        mockInvoke
          .mockResolvedValueOnce(resourceType.data)
          .mockResolvedValueOnce(undefined)

        const ui = new ResourceUpdateUI({
          resourceName: resourceType.name,
          resourceKind: resourceType.kind,
          namespace: resourceType.namespace
        })

        // Load resource
        await ui.loadResourceYaml()

        expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
          resource_name: resourceType.name,
          resource_kind: resourceType.kind,
          namespace: resourceType.namespace
        })

        expect(ui.yamlContent).toContain(resourceType.editFrom)
        expect(ui.hasChanges).toBe(false)

        // Edit resource
        ui.simulateUserEdit(resourceType.editFrom, resourceType.editTo)

        expect(ui.hasChanges).toBe(true)
        expect(ui.yamlContent).toContain(resourceType.editTo)

        // Save changes
        await ui.saveChanges()

        expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
          resource_name: resourceType.name,
          resource_kind: resourceType.kind,
          namespace: resourceType.namespace,
          yaml_content: ui.yamlContent
        })

        expect(ui.hasChanges).toBe(false)
        expect(ui.successMessage).toContain(`${resourceType.kind} '${resourceType.name}' updated successfully`)
      })
    })
  })

  describe('Cluster-scoped Resources', () => {
    it('handles Node resources without namespace parameter', async () => {
      const nodeData = {
        apiVersion: 'v1',
        kind: 'Node',
        metadata: {
          name: 'worker-01',
          labels: { 'node-type': 'worker' }
        },
        spec: { taints: [] }
      }

      mockInvoke
        .mockResolvedValueOnce(nodeData)
        .mockResolvedValueOnce(undefined)

      const ui = new ResourceUpdateUI({
        resourceName: 'worker-01',
        resourceKind: 'Node',
        namespace: null // Cluster-scoped
      })

      await ui.loadResourceYaml()

      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: 'worker-01',
        resource_kind: 'Node',
        namespace: null
      })

      // Edit node labels
      ui.simulateUserEdit('"node-type": "worker"', '"node-type": "worker", "updated": "true"')

      await ui.saveChanges()

      expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
        resource_name: 'worker-01',
        resource_kind: 'Node',
        namespace: null,
        yaml_content: ui.yamlContent
      })
    })
  })

  describe('UI State Management', () => {
    it('correctly manages button states during operations', async () => {
      const deploymentData = {
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-app' },
        spec: { replicas: 1 }
      }

      mockInvoke
        .mockResolvedValueOnce(deploymentData)
        .mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 50)))

      const ui = new ResourceUpdateUI({
        resourceName: 'test-app',
        resourceKind: 'Deployment',
        namespace: 'default'
      })

      // Initial state
      expect(ui.canSave).toBe(false)
      expect(ui.canReset).toBe(false)
      expect(ui.saveButtonText).toBe('Save')

      await ui.loadResourceYaml()

      // After load, still no changes
      expect(ui.canSave).toBe(false)
      expect(ui.canReset).toBe(false)

      // After making changes
      ui.simulateUserEdit('"replicas": 1', '"replicas": 3')
      expect(ui.canSave).toBe(true)
      expect(ui.canReset).toBe(true)

      // During save operation
      const savePromise = ui.saveChanges()
      expect(ui.canSave).toBe(false) // Disabled during save
      expect(ui.canReset).toBe(false) // Also disabled during save
      expect(ui.saveButtonText).toBe('Saving...')

      await savePromise

      // After successful save
      expect(ui.canSave).toBe(false) // No changes to save
      expect(ui.canReset).toBe(false) // No changes to reset
      expect(ui.saveButtonText).toBe('Save')
    })

    it('prevents double-clicking save button', async () => {
      const configMapData = {
        apiVersion: 'v1',
        kind: 'ConfigMap',
        metadata: { name: 'test' },
        data: { key: 'value' }
      }

      let saveCallCount = 0
      mockInvoke
        .mockResolvedValueOnce(configMapData)
        .mockImplementation(() => {
          saveCallCount++
          return Promise.resolve()
        })

      const ui = new ResourceUpdateUI({
        resourceName: 'test',
        resourceKind: 'ConfigMap',
        namespace: 'default'
      })

      await ui.loadResourceYaml()
      ui.simulateUserEdit('value', 'modified')

      // Simulate rapid clicking
      const promise1 = ui.saveChanges()
      const promise2 = ui.saveChanges() // Should not trigger another save
      const promise3 = ui.saveChanges() // Should not trigger another save

      await Promise.all([promise1, promise2, promise3])

      // Only one actual save should have occurred
      expect(saveCallCount).toBe(1)
      expect(mockInvoke).toHaveBeenCalledTimes(2) // 1 load + 1 save
    })
  })
})