<template>
  <!-- Bottom Unified Terminal Panel -->
  <div 
    v-if="isOpen" 
    class="flex-none bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 shadow-xl"
    :style="{ height: isMaximized ? '100vh' : panelHeight + 'px' }"
  >
    <UnifiedTerminalTabs
      ref="terminalTabsRef"
      :maxTabs="10"
      :isMaximized="isMaximized"
      :isResizing="isResizing"
      @close="handleClose"
      @minimize="handleMinimize"
      @toggle-maximize="handleToggleMaximize"
      @start-resize="handleStartResize"
      @refresh-logs="handleRefreshLogs"
      @toggle-live-logging="handleToggleLiveLogging"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import UnifiedTerminalTabs from './ResourcePanel/UnifiedTerminalTabs.vue'
import type { K8sListItem } from '@/types'

interface Props {
  isOpen?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isOpen: false
})

const emit = defineEmits<{
  'close': []
  'update:isOpen': [value: boolean]
}>()

// Template refs
const terminalTabsRef = ref<any>(null)

// Panel state
const panelHeight = ref(400)
const minPanelHeight = 200
const maxPanelHeight = ref(800)
const isMaximized = ref(false)
const isResizing = ref(false)
const currentStreamId = ref<string | null>(null)

// Panel preferences storage keys
const PANEL_HEIGHT_KEY = 'kide-panel-height'
const PANEL_MAXIMIZED_KEY = 'kide-panel-maximized'

// Public methods for adding tabs
function openPodLogs(pod: K8sListItem): void {
  
  if (!props.isOpen) {
    emit('update:isOpen', true)
    loadPanelPreferences()
    
    // Wait for the next tick to ensure the terminal tabs component is mounted
    nextTick(() => {
      addLogTabForPod(pod)
    })
  } else {
    // Panel is already open, add tab immediately
    addLogTabForPod(pod)
  }
}

function openPodShell(pod: K8sListItem): void {
  
  if (!props.isOpen) {
    emit('update:isOpen', true)
    loadPanelPreferences()
    
    // Wait for the next tick to ensure the terminal tabs component is mounted
    nextTick(() => {
      addShellTabForPod(pod)
    })
  } else {
    // Panel is already open, add tab immediately
    addShellTabForPod(pod)
  }
}

function addLogTabForPod(pod: K8sListItem, retryCount = 0): void {
  // Add log tab for this pod
  if (terminalTabsRef.value && pod.metadata?.name && pod.metadata?.namespace) {
    const containers = (pod as any).podSpec?.containers || []
    const initContainers = (pod as any).podSpec?.initContainers || []
    const containerName = containers[0]?.name || initContainers[0]?.name || 'main'
    
    terminalTabsRef.value.addLogTab({
      podName: pod.metadata.name,
      namespace: pod.metadata.namespace,
      containerName: containerName,
      containers: containers,
      initContainers: initContainers
    })
  } else if (retryCount < 10) { // Limit retries to prevent infinite loop
    console.warn(`Terminal tabs ref not available yet, retrying... (attempt ${retryCount + 1})`)
    // Retry after a short delay if ref isn't available
    setTimeout(() => {
      addLogTabForPod(pod, retryCount + 1)
    }, 100)
  } else {
    console.error('Failed to add log tab after multiple retries')
  }
}

function addShellTabForPod(pod: K8sListItem, retryCount = 0): void {
  // Add shell tab for this pod - will auto-connect
  if (terminalTabsRef.value && pod.metadata?.name && pod.metadata?.namespace) {
    const containers = (pod as any).podSpec?.containers || []
    const containerName = containers[0]?.name || 'main'
    
    terminalTabsRef.value.addShellTab({
      podName: pod.metadata.name,
      namespace: pod.metadata.namespace,
      containerName: containerName,
      containers: containers
    })
  } else if (retryCount < 10) { // Limit retries to prevent infinite loop
    console.warn(`Terminal tabs ref not available yet, retrying... (attempt ${retryCount + 1})`)
    // Retry after a short delay if ref isn't available
    setTimeout(() => {
      addShellTabForPod(pod, retryCount + 1)
    }, 100)
  } else {
    console.error('Failed to add shell tab after multiple retries')
  }
}

// Event handlers
function handleClose(): void {
  
  // Stop any active log streaming
  if (currentStreamId.value) {
    stopLogStreaming()
  }
  
  // Close all tabs in the unified terminal
  if (terminalTabsRef.value) {
    terminalTabsRef.value.closeAllTabs()
  }
  
  emit('update:isOpen', false)
  emit('close')
}

function handleMinimize(): void {
  handleClose()
}

function handleToggleMaximize(): void {
  isMaximized.value = !isMaximized.value
  savePanelPreferences()
}

function handleStartResize(event: MouseEvent): void {
  startResize(event)
}

function handleRefreshLogs(tab: any): void {
  // This will be handled by the UnifiedTerminalTabs component internally
}

function handleToggleLiveLogging(tab: any): void {
  // This will be handled by the UnifiedTerminalTabs component internally
}

// Legacy streaming support functions
async function stopLogStreaming(): Promise<void> {
  if (currentStreamId.value) {
    try {
      await invoke('stop_pod_logs_stream', {
        streamId: currentStreamId.value
      })
    } catch (error) {
      console.error('Error stopping log stream:', error)
    } finally {
      currentStreamId.value = null
    }
  }
}

// Resize functionality
function startResize(event: MouseEvent): void {
  if (isMaximized.value) return
  
  isResizing.value = true
  const startY = event.clientY
  const startHeight = panelHeight.value
  
  const handleMouseMove = (e: MouseEvent) => {
    const deltaY = startY - e.clientY // Inverted because we're resizing from bottom up
    let newHeight = startHeight + deltaY
    
    // Constrain height
    newHeight = Math.max(minPanelHeight, Math.min(newHeight, maxPanelHeight.value))
    panelHeight.value = newHeight
  }
  
  const handleMouseUp = () => {
    isResizing.value = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
    savePanelPreferences()
  }
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
  
  // Prevent text selection during resize
  event.preventDefault()
}

function stopResize(): void {
  isResizing.value = false
  // Remove any lingering event listeners
  document.removeEventListener('mousemove', () => {})
  document.removeEventListener('mouseup', () => {})
}

// Panel preferences functions
function savePanelPreferences(): void {
  try {
    localStorage.setItem(PANEL_HEIGHT_KEY, panelHeight.value.toString())
    localStorage.setItem(PANEL_MAXIMIZED_KEY, isMaximized.value.toString())
  } catch (error) {
    console.warn('Failed to save panel preferences:', error)
  }
}

function loadPanelPreferences(): void {
  try {
    const savedHeight = localStorage.getItem(PANEL_HEIGHT_KEY)
    if (savedHeight) {
      const height = parseInt(savedHeight, 10)
      if (height >= minPanelHeight && height <= maxPanelHeight.value) {
        panelHeight.value = height
      }
    }
    
    const savedMaximized = localStorage.getItem(PANEL_MAXIMIZED_KEY)
    if (savedMaximized) {
      isMaximized.value = savedMaximized === 'true'
    }
  } catch (error) {
    console.warn('Failed to load panel preferences:', error)
  }
}

// Keyboard shortcuts for panel controls
function handleKeydown(event: KeyboardEvent): void {
  // Only handle shortcuts when panel is open
  if (!props.isOpen) return
  
  // Ctrl/Cmd + Shift + M: Toggle maximize
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'M') {
    event.preventDefault()
    handleToggleMaximize()
  }
  
  // Ctrl/Cmd + Shift + D: Minimize/close panel
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'D') {
    event.preventDefault()
    handleMinimize()
  }
  
  // Escape: Close panel when maximized
  if (event.key === 'Escape' && isMaximized.value) {
    event.preventDefault()
    isMaximized.value = false
    savePanelPreferences()
  }
}

// Setup and cleanup
onMounted(() => {
  // Calculate initial max height
  maxPanelHeight.value = Math.floor(window.innerHeight * 0.8)
  
  // Load saved preferences (this will override default if saved values exist)
  loadPanelPreferences()
  
  // Set up keyboard shortcuts
  document.addEventListener('keydown', handleKeydown)
  
  // Handle window resize
  const handleWindowResize = () => {
    maxPanelHeight.value = Math.floor(window.innerHeight * 0.8)
    // Constrain current height if needed
    if (panelHeight.value > maxPanelHeight.value) {
      panelHeight.value = maxPanelHeight.value
      savePanelPreferences()
    }
  }
  
  window.addEventListener('resize', handleWindowResize)
  
  // Cleanup on unmount
  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('resize', handleWindowResize)
    // Clean up resize listeners if active
    if (isResizing.value) {
      stopResize()
    }
  })
})

// Expose public methods
defineExpose({
  openPodLogs,
  openPodShell,
  terminalTabsRef
})
</script>