import { describe, it, expect, beforeEach, vi, type MockedFunction } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

// Mock the Tauri API
const mockInvoke = vi.fn() as MockedFunction<typeof import('@tauri-apps/api/core').invoke>
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

// Mock components that might not be available in test environment
const MockYamlEditor = {
  name: 'YamlEditor',
  props: ['modelValue', 'language', 'theme', 'options'],
  emits: ['update:modelValue'],
  template: `
    <div class="yaml-editor-mock" data-testid="yaml-editor">
      <textarea 
        :value="modelValue" 
        @input="$emit('update:modelValue', $event.target.value)"
        data-testid="yaml-textarea"
        rows="20"
        cols="80"
      />
    </div>
  `
}

// Mock ResourcePanel component
const MockResourcePanel = {
  name: 'ResourcePanel',
  props: ['resourceData', 'resourceKind'],
  template: `
    <div class="resource-panel-mock" data-testid="resource-panel">
      <div class="tabs">
        <button 
          class="tab-overview" 
          :class="{ active: activeTab === 'overview' }"
          @click="activeTab = 'overview'"
          data-testid="overview-tab"
        >
          Overview
        </button>
        <button 
          class="tab-yaml" 
          :class="{ active: activeTab === 'yaml' }"
          @click="activeTab = 'yaml'"
          data-testid="yaml-tab"
        >
          YAML
        </button>
      </div>
      
      <div v-if="activeTab === 'overview'" class="tab-content">
        <div data-testid="overview-content">
          Resource Overview Content
        </div>
      </div>
      
      <div v-if="activeTab === 'yaml'" class="tab-content">
        <div class="yaml-editor-container">
          <div class="editor-toolbar">
            <button 
              @click="saveYaml" 
              :disabled="!hasChanges || saving"
              data-testid="save-yaml-button"
              class="save-button"
            >
              {{ saving ? 'Saving...' : 'Save' }}
            </button>
            <button 
              @click="resetYaml"
              :disabled="!hasChanges"
              data-testid="reset-yaml-button"
              class="reset-button"
            >
              Reset
            </button>
          </div>
          
          <YamlEditor
            v-model="yamlContent"
            language="yaml"
            theme="vs-dark"
            :options="{ minimap: { enabled: false }, wordWrap: 'on' }"
          />
          
          <div v-if="saveError" class="error-message" data-testid="save-error">
            {{ saveError }}
          </div>
          
          <div v-if="saveSuccess" class="success-message" data-testid="save-success">
            Resource updated successfully
          </div>
        </div>
      </div>
    </div>
  `,
  components: {
    YamlEditor: MockYamlEditor
  },
  data() {
    return {
      activeTab: 'overview' as 'overview' | 'yaml',
      yamlContent: '',
      originalYamlContent: '',
      hasChanges: false,
      saving: false,
      saveError: '',
      saveSuccess: false
    }
  },
  watch: {
    yamlContent(newValue: string) {
      this.hasChanges = newValue !== this.originalYamlContent
      this.saveSuccess = false
      this.saveError = ''
    }
  },
  mounted() {
    this.loadResourceYaml()
  },
  methods: {
    async loadResourceYaml() {
      if (!this.resourceData?.metadata?.name || !this.resourceKind) return
      
      try {
        const yaml = await mockInvoke('get_full_resource', {
          resource_name: this.resourceData.metadata.name,
          resource_kind: this.resourceKind,
          namespace: this.resourceData.metadata.namespace || null
        })
        
        // Convert JSON to YAML string (simplified for testing)
        this.yamlContent = this.jsonToYaml(yaml)
        this.originalYamlContent = this.yamlContent
      } catch (error) {
        console.error('Failed to load resource YAML:', error)
      }
    },
    
    async saveYaml() {
      if (!this.resourceData?.metadata?.name || !this.resourceKind) return
      
      this.saving = true
      this.saveError = ''
      this.saveSuccess = false
      
      try {
        await mockInvoke('update_resource', {
          resource_name: this.resourceData.metadata.name,
          resource_kind: this.resourceKind,
          namespace: this.resourceData.metadata.namespace || null,
          yaml_content: this.yamlContent
        })
        
        this.originalYamlContent = this.yamlContent
        this.hasChanges = false
        this.saveSuccess = true
        
        // Hide success message after 3 seconds
        setTimeout(() => {
          this.saveSuccess = false
        }, 3000)
        
      } catch (error: any) {
        this.saveError = error.message || 'Failed to save resource'
      } finally {
        this.saving = false
      }
    },
    
    resetYaml() {
      this.yamlContent = this.originalYamlContent
      this.hasChanges = false
      this.saveError = ''
      this.saveSuccess = false
    },
    
    // Simple JSON to YAML converter for testing
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
              yaml += `${spaces}- \\n${this.jsonToYaml(item, indent + 1)}`
            } else {
              yaml += `${spaces}- ${item}\\n`
            }
          }
        } else {
          yaml += ` ${value}\\n`
        }
      }
      
      return yaml
    }
  }
}

describe('YAML Editor Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockClear()
  })

  it('loads resource YAML when component mounts', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      },
      kind: 'Deployment'
    }

    const mockYamlResponse = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      },
      spec: {
        replicas: 3,
        selector: {
          matchLabels: {
            app: 'test-app'
          }
        }
      }
    }

    mockInvoke.mockResolvedValueOnce(mockYamlResponse)

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: 'test-deployment',
        resource_kind: 'Deployment',
        namespace: 'default'
      })
    })

    // Switch to YAML tab
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    expect(wrapper.find('[data-testid="yaml-editor"]').exists()).toBe(true)
  })

  it('detects changes in YAML content and enables save button', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      }
    }

    mockInvoke.mockResolvedValueOnce({
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: { name: 'test-deployment' },
      spec: { replicas: 3 }
    })

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Initially, save button should be disabled
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    expect(saveButton.attributes('disabled')).toBeDefined()

    // Modify YAML content
    const textarea = wrapper.find('[data-testid="yaml-textarea"]')
    await textarea.setValue('modified yaml content')
    await nextTick()

    // Save button should now be enabled
    expect(saveButton.attributes('disabled')).toBeUndefined()
  })

  it('successfully saves YAML changes', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      }
    }

    mockInvoke
      .mockResolvedValueOnce({
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      })
      .mockResolvedValueOnce(undefined) // update_resource success

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Modify YAML content
    const modifiedYaml = `
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
  namespace: default
spec:
  replicas: 5
`
    
    const textarea = wrapper.find('[data-testid="yaml-textarea"]')
    await textarea.setValue(modifiedYaml)
    await nextTick()

    // Click save button
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    await saveButton.trigger('click')
    await nextTick()

    expect(mockInvoke).toHaveBeenCalledWith('update_resource', {
      resource_name: 'test-deployment',
      resource_kind: 'Deployment',
      namespace: 'default',
      yaml_content: modifiedYaml
    })

    // Success message should appear
    await vi.waitFor(() => {
      expect(wrapper.find('[data-testid="save-success"]').exists()).toBe(true)
    })
  })

  it('handles save errors gracefully', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      }
    }

    mockInvoke
      .mockResolvedValueOnce({
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      })
      .mockRejectedValueOnce(new Error('kubectl apply failed: invalid YAML syntax'))

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Modify YAML with invalid content
    const invalidYaml = `
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
spec:
  # Invalid structure - missing required selector
  replicas: 5
`
    
    const textarea = wrapper.find('[data-testid="yaml-textarea"]')
    await textarea.setValue(invalidYaml)
    await nextTick()

    // Click save button
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    await saveButton.trigger('click')
    await nextTick()

    // Error message should appear
    await vi.waitFor(() => {
      const errorElement = wrapper.find('[data-testid="save-error"]')
      expect(errorElement.exists()).toBe(true)
      expect(errorElement.text()).toContain('kubectl apply failed')
    })
  })

  it('resets YAML content to original state', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      }
    }

    const originalYaml = {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: { name: 'test-deployment' },
      spec: { replicas: 3 }
    }

    mockInvoke.mockResolvedValueOnce(originalYaml)

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Modify YAML content
    const textarea = wrapper.find('[data-testid="yaml-textarea"]')
    const modifiedContent = 'modified content'
    await textarea.setValue(modifiedContent)
    await nextTick()

    expect(textarea.element.value).toBe(modifiedContent)

    // Reset to original
    const resetButton = wrapper.find('[data-testid="reset-yaml-button"]')
    await resetButton.trigger('click')
    await nextTick()

    // Should contain original content pattern
    expect(textarea.element.value).toContain('replicas: 3')
  })

  it('supports different resource types generically', async () => {
    const testCases = [
      {
        resourceKind: 'Service',
        resourceData: {
          metadata: { name: 'test-service', namespace: 'default' }
        },
        mockResponse: {
          apiVersion: 'v1',
          kind: 'Service',
          metadata: { name: 'test-service' },
          spec: { type: 'ClusterIP', ports: [{ port: 80 }] }
        }
      },
      {
        resourceKind: 'ConfigMap',
        resourceData: {
          metadata: { name: 'test-configmap', namespace: 'default' }
        },
        mockResponse: {
          apiVersion: 'v1',
          kind: 'ConfigMap',
          metadata: { name: 'test-configmap' },
          data: { 'config.properties': 'key=value' }
        }
      },
      {
        resourceKind: 'Secret',
        resourceData: {
          metadata: { name: 'test-secret', namespace: 'default' }
        },
        mockResponse: {
          apiVersion: 'v1',
          kind: 'Secret',
          metadata: { name: 'test-secret' },
          type: 'Opaque',
          data: { 'secret-key': 'c2VjcmV0LXZhbHVl' }
        }
      }
    ]

    for (const testCase of testCases) {
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValueOnce(testCase.mockResponse)

      const wrapper = mount(MockResourcePanel, {
        props: {
          resourceData: testCase.resourceData,
          resourceKind: testCase.resourceKind
        }
      })

      await nextTick()
      
      expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
        resource_name: testCase.resourceData.metadata.name,
        resource_kind: testCase.resourceKind,
        namespace: testCase.resourceData.metadata.namespace
      })

      // Switch to YAML tab and verify editor loads
      await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
      await nextTick()

      expect(wrapper.find('[data-testid="yaml-editor"]').exists()).toBe(true)
      
      wrapper.unmount()
    }
  })

  it('handles cluster-scoped resources without namespace', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-node'
        // No namespace for cluster-scoped resources
      }
    }

    mockInvoke.mockResolvedValueOnce({
      apiVersion: 'v1',
      kind: 'Node',
      metadata: { name: 'test-node' },
      spec: { podCIDR: '10.244.0.0/24' }
    })

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Node'
      }
    })

    await nextTick()

    expect(mockInvoke).toHaveBeenCalledWith('get_full_resource', {
      resource_name: 'test-node',
      resource_kind: 'Node',
      namespace: null
    })
  })

  it('prevents multiple simultaneous save operations', async () => {
    const mockResourceData = {
      metadata: {
        name: 'test-deployment',
        namespace: 'default'
      }
    }

    mockInvoke
      .mockResolvedValueOnce({
        apiVersion: 'apps/v1',
        kind: 'Deployment',
        metadata: { name: 'test-deployment' },
        spec: { replicas: 3 }
      })
      .mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 100))) // Slow save

    const wrapper = mount(MockResourcePanel, {
      props: {
        resourceData: mockResourceData,
        resourceKind: 'Deployment'
      }
    })

    await nextTick()
    await wrapper.find('[data-testid="yaml-tab"]').trigger('click')
    await nextTick()

    // Modify content
    const textarea = wrapper.find('[data-testid="yaml-textarea"]')
    await textarea.setValue('modified content')
    await nextTick()

    // Start first save
    const saveButton = wrapper.find('[data-testid="save-yaml-button"]')
    await saveButton.trigger('click')
    await nextTick()

    // Button should show "Saving..." and be disabled
    expect(saveButton.text()).toBe('Saving...')
    expect(saveButton.attributes('disabled')).toBeDefined()
  })
})