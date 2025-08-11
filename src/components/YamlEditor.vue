<template>
  <div class="h-full flex flex-col">
    <!-- Editor Actions Bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center space-x-2">
        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">YAML Editor</span>
        <span v-if="isDirty" class="text-xs text-orange-600 dark:text-orange-400">• Modified</span>
        <span v-if="hasValidationError" class="text-xs text-red-600 dark:text-red-400">• Invalid YAML</span>
      </div>
      <div class="flex items-center space-x-2">
        <button
          @click="cancel"
          :disabled="!isDirty"
          class="px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          @click="save"
          :disabled="!isDirty || hasValidationError || isSaving"
          class="px-3 py-1 text-xs font-medium text-white bg-blue-600 border border-blue-600 rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isSaving ? 'Saving...' : 'Save' }}
        </button>
        <button
          @click="saveAndClose"
          :disabled="!isDirty || hasValidationError || isSaving"
          class="px-3 py-1 text-xs font-medium text-white bg-green-600 border border-green-600 rounded hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isSaving ? 'Saving...' : 'Save & Close' }}
        </button>
      </div>
    </div>

    <!-- Error/Success Messages -->
    <div v-if="errorMessage" class="px-4 py-2 bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800">
      <div class="flex items-center">
        <svg class="w-4 h-4 text-red-600 dark:text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <span class="text-sm text-red-800 dark:text-red-200">{{ errorMessage }}</span>
      </div>
    </div>

    <div v-if="successMessage" class="px-4 py-2 bg-green-50 dark:bg-green-900/20 border-b border-green-200 dark:border-green-800">
      <div class="flex items-center">
        <svg class="w-4 h-4 text-green-600 dark:text-green-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <span class="text-sm text-green-800 dark:text-green-200">{{ successMessage }}</span>
      </div>
    </div>

    <!-- Monaco Editor Container -->
    <div ref="editorContainer" class="flex-1 min-h-0"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
// Import Monaco editor with YAML and JSON language support
import * as monaco from 'monaco-editor'
import * as yaml from 'js-yaml'
import { invoke } from '@tauri-apps/api/core'
import { useTimeouts } from '@/composables/useTimeouts'

interface Props {
  yamlContent: string
  resourceName: string
  resourceKind: string
  namespace?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  saved: [yamlContent: string]
}>()

// Composables
const { createTimeout } = useTimeouts()

// Refs
const editorContainer = ref<HTMLDivElement>()
let editor: monaco.editor.IStandaloneCodeEditor | null = null

// State
const isDirty = ref(false)
const hasValidationError = ref(false)
const isSaving = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const originalContent = ref('')
const currentContent = ref('')

// Initialize Monaco Editor
onMounted(async () => {
  await nextTick()
  
  if (!editorContainer.value) return

  // Configure Monaco Editor worker environment
  if (typeof window !== 'undefined') {
    (window as any).MonacoEnvironment = {
      getWorkerUrl: () => {
        // In test environment, return a mock worker URL
        if (import.meta.env?.MODE === 'test') {
          return 'data:text/javascript;base64,Ly8gTW9jayB3b3JrZXI='
        }
        // For production, dynamically construct worker URL
        const workerPath = 'monaco-editor/esm/vs/editor/editor.worker.js'
        return new URL(workerPath, import.meta.url).href
      }
    }
  }

  // Create editor (YAML language already registered via contribution)
  editor = monaco.editor.create(editorContainer.value, {
    value: props.yamlContent,
    language: 'yaml',
    theme: 'vs-dark',
    automaticLayout: true,
    minimap: { enabled: true },
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    lineNumbers: 'on',
    folding: true,
    renderWhitespace: 'boundary',
    fontSize: 12,
    tabSize: 2,
    insertSpaces: true,
  })

  // Store original content
  originalContent.value = props.yamlContent
  currentContent.value = props.yamlContent

  // Listen for content changes
  editor.onDidChangeModelContent(() => {
    const value = editor?.getValue() || ''
    currentContent.value = value
    isDirty.value = value !== originalContent.value
    validateYaml(value)
    clearMessages()
  })

  // Initial validation
  validateYaml(props.yamlContent)
})

// Cleanup
onUnmounted(() => {
  if (editor) {
    editor.dispose()
  }
})

// Watch for external content changes
watch(() => props.yamlContent, (newContent) => {
  if (editor && !isDirty.value) {
    editor.setValue(newContent)
    originalContent.value = newContent
    currentContent.value = newContent
    validateYaml(newContent)
  }
})

// YAML validation
function validateYaml(content: string): void {
  try {
    yaml.load(content)
    hasValidationError.value = false
    
    // Clear Monaco editor markers
    if (editor) {
      monaco.editor.setModelMarkers(editor.getModel()!, 'yaml', [])
    }
  } catch (error: any) {
    hasValidationError.value = true
    
    // Add Monaco editor markers for YAML errors
    if (editor && error.mark) {
      const markers: monaco.editor.IMarkerData[] = [{
        severity: monaco.MarkerSeverity.Error,
        startLineNumber: error.mark.line + 1,
        startColumn: error.mark.column + 1,
        endLineNumber: error.mark.line + 1,
        endColumn: error.mark.column + 10,
        message: error.message || 'YAML syntax error'
      }]
      
      monaco.editor.setModelMarkers(editor.getModel()!, 'yaml', markers)
    }
  }
}

// Clear messages
function clearMessages(): void {
  errorMessage.value = ''
  successMessage.value = ''
}

// Cancel changes
function cancel(): void {
  if (editor) {
    editor.setValue(originalContent.value)
    currentContent.value = originalContent.value
    isDirty.value = false
    hasValidationError.value = false
    validateYaml(originalContent.value)
    clearMessages()
  }
}

// Save changes
async function save(): Promise<boolean> {
  if (!isDirty.value || hasValidationError.value || isSaving.value) {
    return false
  }

  isSaving.value = true
  clearMessages()

  try {
    // Parse YAML to validate it once more
    const parsedYaml = yaml.load(currentContent.value)
    
    // Call backend API to save the resource
    await invoke('update_resource', {
      resourceName: props.resourceName,
      resourceKind: props.resourceKind,
      namespace: props.namespace || undefined,
      yamlContent: currentContent.value
    })

    // Update original content and mark as clean
    originalContent.value = currentContent.value
    isDirty.value = false
    
    successMessage.value = `${props.resourceKind} "${props.resourceName}" saved successfully`
    
    // Clear success message after 3 seconds
    createTimeout(() => {
      successMessage.value = ''
    }, 3000)

    // Emit saved event
    emit('saved', currentContent.value)
    
    return true
  } catch (error: any) {
    errorMessage.value = error.message || `Failed to save ${props.resourceKind} "${props.resourceName}"`
    return false
  } finally {
    isSaving.value = false
  }
}

// Save and close
async function saveAndClose(): Promise<void> {
  const saved = await save()
  if (saved) {
    emit('close')
  }
}
</script>