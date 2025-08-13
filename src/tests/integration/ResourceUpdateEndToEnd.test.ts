import { describe, it, expect, beforeEach, vi, type MockedFunction } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

// Mock the Tauri API
const mockInvoke = vi.fn() as MockedFunction<typeof import('@tauri-apps/api/core').invoke>
const mockListen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen
}))

// Mock a full application component that includes resource list and detail view
const MockKubernetesApp = {
  name: 'KubernetesApp',
  template: `
    <div class="kubernetes-app" data-testid="kubernetes-app">
      <div class="sidebar">
        <div class="resource-list" data-testid="resource-list">
          <div class="resource-category">
            <h3>Workloads</h3>
            <div class="resource-items">
              <div 
                v-for="resource in resources" 
                :key="resource.metadata.uid"
                :class="['resource-item', { selected: selectedResource?.metadata.uid === resource.metadata.uid }]"
                @click="selectResource(resource)"
                :data-testid="'resource-item-' + resource.metadata.name"
              >
                <span class="resource-name">{{ resource.metadata.name }}</span>
                <span class="resource-namespace" v-if="resource.metadata.namespace">{{ resource.metadata.namespace }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <div class="main-content">
        <div v-if="selectedResource" class="resource-detail" data-testid="resource-detail">
          <div class="resource-header">
            <h2>{{ selectedResource.kind }}/{{ selectedResource.metadata.name }}</h2>
            <div class="resource-actions">
              <button 
                @click="deleteResource" 
                class="delete-button"
                data-testid="delete-resource-button"
              >
                Delete
              </button>
            </div>
          </div>
          
          <div class="tabs">
            <button 
              :class="['tab', { active: activeTab === 'overview' }]"
              @click="activeTab = 'overview'"
              data-testid="overview-tab"
            >
              Overview
            </button>
            <button 
              :class="['tab', { active: activeTab === 'yaml' }]"
              @click="activeTab = 'yaml'"
              data-testid="yaml-tab"
            >
              YAML
            </button>
          </div>
          
          <div class="tab-content">
            <div v-if="activeTab === 'overview'" class="overview-content">
              <div class="resource-info">
                <p>Name: {{ selectedResource.metadata.name }}</p>
                <p>Namespace: {{ selectedResource.metadata.namespace }}</p>
                <p>Kind: {{ selectedResource.kind }}</p>
                <p v-if="selectedResource.spec?.replicas">Replicas: {{ selectedResource.spec.replicas }}</p>
              </div>
            </div>
            
            <div v-if="activeTab === 'yaml'" class="yaml-content">
              <div class="yaml-editor-toolbar">
                <button 
                  @click="loadYaml"
                  :disabled="loadingYaml"
                  data-testid="load-yaml-button"
                >
                  {{ loadingYaml ? 'Loading...' : 'Refresh YAML' }}
                </button>
                <button 
                  @click="saveYaml"
                  :disabled="!hasYamlChanges || savingYaml"
                  data-testid="save-yaml-button"
                  class="save-button"
                >
                  {{ savingYaml ? 'Saving...' : 'Save Changes' }}
                </button>
                <button 
                  @click="resetYaml"
                  :disabled="!hasYamlChanges || savingYaml"
                  data-testid="reset-yaml-button"
                >
                  Reset
                </button>
              </div>
              
              <div class="yaml-editor-container">
                <textarea
                  v-model="yamlContent"
                  data-testid="yaml-editor-textarea"
                  rows="25"
                  cols="100"
                  class="yaml-editor"
                  spellcheck="false"
                ></textarea>
              </div>
              
              <div v-if="yamlError" class="error-message" data-testid="yaml-error">
                {{ yamlError }}
              </div>
              
              <div v-if="yamlSuccess" class="success-message" data-testid="yaml-success">
                {{ yamlSuccess }}
              </div>
            </div>
          </div>
        </div>
        
        <div v-else class="no-selection" data-testid="no-selection">
          Select a resource to view details
        </div>
      </div>
    </div>
  `,
  data() {
    return {
      resources: [] as any[],
      selectedResource: null as any,
      activeTab: 'overview' as 'overview' | 'yaml',
      yamlContent: '',
      originalYamlContent: '',
      hasYamlChanges: false,
      loadingYaml: false,
      savingYaml: false,
      yamlError: '',
      yamlSuccess: ''
    }
  },
  watch: {
    yamlContent(newValue: string) {
      this.hasYamlChanges = newValue !== this.originalYamlContent
      this.yamlSuccess = ''
      this.yamlError = ''
    },
    selectedResource(newResource) {
      if (newResource && this.activeTab === 'yaml') {
        this.loadYaml()
      }
    }
  },
  async mounted() {
    await this.loadResources()
  },
  methods: {
    async loadResources() {
      try {
        // Load sample resources for testing
        this.resources = [
          {
            metadata: {
              name: 'o2-openobserve-router',
              namespace: 'default',
              uid: 'deployment-1'
            },
            kind: 'Deployment',
            spec: {
              replicas: 3
            }
          },
          {
            metadata: {
              name: 'nginx-service',
              namespace: 'default',
              uid: 'service-1'
            },
            kind: 'Service',
            spec: {
              type: 'ClusterIP',
              ports: [{ port: 80, targetPort: 8080 }]
            }
          },
          {
            metadata: {
              name: 'app-config',
              namespace: 'default',
              uid: 'configmap-1'
            },
            kind: 'ConfigMap',
            data: {
              'app.properties': 'debug=true'
            }
          }
        ]
      } catch (error) {
        console.error('Failed to load resources:', error)
      }
    },
    
    selectResource(resource: any) {
      this.selectedResource = resource
      this.activeTab = 'overview'
      this.yamlContent = ''
      this.originalYamlContent = ''
      this.hasYamlChanges = false
      this.yamlError = ''
      this.yamlSuccess = ''
    },
    
    async loadYaml() {
      if (!this.selectedResource) return
      
      this.loadingYaml = true
      this.yamlError = ''
      
      try {
        const resourceData = await mockInvoke('get_full_resource', {
          resource_name: this.selectedResource.metadata.name,
          resource_kind: this.selectedResource.kind,
          namespace: this.selectedResource.metadata.namespace || null
        })
        
        // Convert JSON to YAML for display (simplified)
        this.yamlContent = this.jsonToYaml(resourceData)
        this.originalYamlContent = this.yamlContent
        this.hasYamlChanges = false
        
      } catch (error: any) {
        this.yamlError = `Failed to load YAML: ${error.message}`
      } finally {
        this.loadingYaml = false
      }
    },
    
    async saveYaml() {
      if (!this.selectedResource || !this.hasYamlChanges) return
      
      this.savingYaml = true
      this.yamlError = ''
      this.yamlSuccess = ''
      
      try {
        await mockInvoke('update_resource', {
          resource_name: this.selectedResource.metadata.name,
          resource_kind: this.selectedResource.kind,
          namespace: this.selectedResource.metadata.namespace || null,
          yaml_content: this.yamlContent
        })
        
        this.originalYamlContent = this.yamlContent
        this.hasYamlChanges = false
        this.yamlSuccess = `${this.selectedResource.kind} '${this.selectedResource.metadata.name}' updated successfully`
        
        // Update the local resource data if it was a replica change
        if (this.yamlContent.includes('replicas:')) {
          const replicasMatch = this.yamlContent.match(/replicas:\\s*(\\d+)/)
          if (replicasMatch && this.selectedResource.spec) {
            this.selectedResource.spec.replicas = parseInt(replicasMatch[1])
          }
        }
        
        // Hide success message after 5 seconds
        setTimeout(() => {
          this.yamlSuccess = ''
        }, 5000)
        
      } catch (error: any) {
        this.yamlError = `Failed to save changes: ${error.message}`
      } finally {
        this.savingYaml = false
      }
    },
    
    resetYaml() {
      this.yamlContent = this.originalYamlContent
      this.hasYamlChanges = false
      this.yamlError = ''
      this.yamlSuccess = ''
    },
    
    async deleteResource() {
      if (!this.selectedResource) return
      
      if (!confirm(`Delete ${this.selectedResource.kind} '${this.selectedResource.metadata.name}'?`)) {
        return
      }
      
      try {
        await mockInvoke('delete_resource', {
          resource_name: this.selectedResource.metadata.name,
          resource_kind: this.selectedResource.kind,
          namespace: this.selectedResource.metadata.namespace || null
        })
        
        // Remove from local list
        this.resources = this.resources.filter(r => r.metadata.uid !== this.selectedResource.metadata.uid)
        this.selectedResource = null
        
      } catch (error: any) {
        alert(`Failed to delete resource: ${error.message}`)
      }
    },
    
    jsonToYaml(obj: any, indent = 0): string {
      const spaces = '  '.repeat(indent)
      let yaml = ''
      
      for (const [key, value] of Object.entries(obj)) {
        if (value === null || value === undefined) continue
        
        yaml += `${spaces}${key}:`
        
        if (typeof value === 'object' && !Array.isArray(value)) {
          yaml += '\\n' + this.jsonToYaml(value, indent + 1)
        } else if (Array.isArray(value)) {
          yaml += '\\n'
          for (const item of value) {
            if (typeof item === 'object') {
              yaml += `${spaces}- \n${this.jsonToYaml(item, indent + 1)}`
            } else {
              yaml += `${spaces}- ${item}\n`
            }
          }
        } else {
          yaml += ` ${value}\n`
        }
      }
      
      return yaml
    }
  }
}

describe('Resource Update End-to-End Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockClear()
    
    // Mock window.confirm for delete operations
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true))
    vi.stubGlobal('alert', vi.fn())
  })

  it('completes full workflow: select resource → load YAML → edit → save', async () => {
    const deploymentYaml = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'o2-openobserve-router',
        namespace: 'default'
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

    mockInvoke
      .mockResolvedValueOnce(deploymentYaml) // get_full_resource
      .mockResolvedValueOnce(undefined) // update_resource success

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    // Step 1: Select the deployment resource
    const deploymentItem = wrapper.find('[data-testid="resource-item-o2-openobserve-router"]')
    expect(deploymentItem.exists()).toBe(true)
    
    await deploymentItem.trigger('click')
    await nextTick()

    // Verify resource is selected
    expect(wrapper.find('[data-testid="resource-detail"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('Deployment/o2-openobserve-router')

    // Step 2: Switch to YAML tab
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Step 3: Load YAML content
    await wrapper.find('[data-testid="load-yaml-button"]').trigger('click')
    await nextTick()

    expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
      resource_name: 'o2-openobserve-router',
      resource_kind: 'Deployment',
      namespace: 'default'
    })

    // Step 4: Edit YAML content (change replicas from 3 to 5)
    const yamlTextarea = wrapper.find('[data-testid="yaml-editor-textarea"]')
    const originalContent = yamlTextarea.element.value
    expect(originalContent).toContain('replicas: 3')
    
    const modifiedContent = originalContent.replace('replicas: 3', 'replicas: 5')
    await yamlTextarea.setValue(modifiedContent)
    await nextTick()

    // Verify changes are detected
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    expect(saveButton.attributes('disabled')).toBeUndefined()

    // Step 5: Save changes
    await saveButton.trigger('click')
    await nextTick()

    expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
      resource_name: 'o2-openobserve-router',
      resource_kind: 'Deployment',
      namespace: 'default',
      yaml_content: modifiedContent
    })

    // Verify success message appears
    await vi.waitFor(() => {
      expect(wrapper.find('[data-testid="yaml-success"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="yaml-success"]').text()).toContain('updated successfully')
    })
  })

  it('handles different resource types with correct API calls', async () => {
    const testScenarios = [
      {
        resourceName: 'nginx-service',
        resourceKind: 'Service',
        mockResponse: {
          apiVersion: 'v1',
          kind: 'Service',
          metadata: { name: 'nginx-service', namespace: 'default' },
          spec: { type: 'ClusterIP', ports: [{ port: 80, targetPort: 8080 }] }
        }
      },
      {
        resourceName: 'app-config',
        resourceKind: 'ConfigMap',
        mockResponse: {
          apiVersion: 'v1',
          kind: 'ConfigMap',
          metadata: { name: 'app-config', namespace: 'default' },
          data: { 'app.properties': 'debug=true' }
        }
      }
    ]

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    for (const scenario of testScenarios) {
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValueOnce(scenario.mockResponse)

      // Select resource
      await wrapper.find(`[data-testid="resource-item-${scenario.resourceName}"]`).trigger('click')
      await nextTick()

      // Switch to YAML tab and load
      await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
      await nextTick()
      await wrapper.find('[data-testid="load-yaml-button"]').trigger('click')
      await nextTick()

      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: scenario.resourceName,
        resource_kind: scenario.resourceKind,
        namespace: 'default'
      })

      // Verify YAML content loads
      const yamlTextarea = wrapper.find('[data-testid="yaml-editor-textarea"]')
      expect(yamlTextarea.element.value).toContain(scenario.resourceKind)
      expect(yamlTextarea.element.value).toContain(scenario.resourceName)
    }
  })

  it('handles save errors and shows appropriate error messages', async () => {
    mockInvoke
      .mockResolvedValueOnce({
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'o2-openobserve-router', namespace: 'default' },
        spec: { replicas: 3 }
      })
      .mockRejectedValueOnce(new Error('kubectl apply failed: validation error - missing required field'))

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    // Select resource and go to YAML tab
    await wrapper.find('[data-testid="resource-item-o2-openobserve-router"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="load-yaml-button"]').trigger('click')
    await nextTick()

    // Make invalid changes
    const yamlTextarea = wrapper.find('[data-testid="yaml-editor-textarea"]')
    await yamlTextarea.setValue('invalid yaml content without required fields')
    await nextTick()

    // Try to save
    await wrapper.find('[data-testid="save-yaml-button"]').trigger('click')
    await nextTick()

    // Verify error message appears
    await vi.waitFor(() => {
      const errorElement = wrapper.find('[data-testid="yaml-error"]')
      expect(errorElement.exists()).toBe(true)
      expect(errorElement.text()).toContain('validation error')
      expect(errorElement.text()).toContain('missing required field')
    })
  })

  it('supports reset functionality to revert changes', async () => {
    const originalYaml = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: { name: 'o2-openobserve-router', namespace: 'default' },
      spec: { replicas: 3 }
    }

    mockInvoke.mockResolvedValueOnce(originalYaml)

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    // Setup: select resource, load YAML
    await wrapper.find('[data-testid="resource-item-o2-openobserve-router"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="load-yaml-button"]').trigger('click')
    await nextTick()

    const yamlTextarea = wrapper.find('[data-testid="yaml-editor-textarea"]')
    const originalContent = yamlTextarea.element.value
    
    // Make changes
    const modifiedContent = 'completely different content'
    await yamlTextarea.setValue(modifiedContent)
    await nextTick()

    expect(yamlTextarea.element.value).toBe(modifiedContent)

    // Reset changes
    const resetButton = wrapper.find('[data-testid="reset-yaml-button"]')
    expect(resetButton.attributes('disabled')).toBeUndefined() // Should be enabled
    
    await resetButton.trigger('click')
    await nextTick()

    // Verify content is restored
    expect(yamlTextarea.element.value).toBe(originalContent)
    
    // Reset button should be disabled again
    expect(resetButton.attributes('disabled')).toBeDefined()
  })

  it('prevents multiple simultaneous operations', async () => {
    mockInvoke
      .mockResolvedValueOnce({
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'o2-openobserve-router', namespace: 'default' },
        spec: { replicas: 3 }
      })
      .mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 100))) // Slow update

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    // Setup
    await wrapper.find('[data-testid="resource-item-o2-openobserve-router"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()
    await wrapper.find('[data-testid="load-yaml-button"]').trigger('click')
    await nextTick()

    // Make changes
    const yamlTextarea = wrapper.find('[data-testid="yaml-editor-textarea"]')
    await yamlTextarea.setValue('modified content')
    await nextTick()

    // Start save operation
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    await saveButton.trigger('click')
    await nextTick()

    // Button should show saving state and be disabled
    expect(saveButton.text()).toBe('Saving...')
    expect(saveButton.attributes('disabled')).toBeDefined()

    // Other buttons should also be disabled during save
    expect(wrapper.find('[data-testid="reset-yaml-button"]').attributes('disabled')).toBeDefined()
  })

  it('integrates with resource deletion workflow', async () => {
    mockInvoke.mockResolvedValueOnce(undefined) // delete_resource success

    const wrapper = mount(MockKubernetesApp)
    await nextTick()

    // Select a resource
    await wrapper.find('[data-testid="resource-item-o2-openobserve-router"]').trigger('click')
    await nextTick()

    // Verify resource is selected
    expect(wrapper.find('[data-testid="resource-detail"]').exists()).toBe(true)

    // Delete the resource
    await wrapper.find('[data-testid="delete-resource-button"]').trigger('click')
    await nextTick()

    expect(mockInvoke).toHaveBeenCalledWith('delete_resource', {
      resource_name: 'o2-openobserve-router',
      resource_kind: 'Deployment',
      namespace: 'default'
    })

    // Resource should be removed from the list
    expect(wrapper.find('[data-testid="resource-item-o2-openobserve-router"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="no-selection"]').exists()).toBe(true)
  })
})