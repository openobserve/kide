<template>
  <div class="h-full flex flex-col">
    <!-- Resize Handle -->
    <div 
      @mousedown="startResize"
      class="h-2 bg-gray-300 dark:bg-gray-600 hover:bg-blue-500 dark:hover:bg-blue-400 cursor-row-resize transition-all relative group"
      :class="{ 'bg-blue-500 dark:bg-blue-400': isResizing }"
      title="Drag to resize panel"
    >
      <!-- Resize indicator -->
      <div class="absolute inset-x-0 top-0 h-full bg-blue-500 dark:bg-blue-400 opacity-0 group-hover:opacity-100 transition-opacity"></div>
      <!-- Drag dots indicator -->
      <div class="absolute left-1/2 top-1/2 transform -translate-x-1/2 -translate-y-1/2 opacity-60 group-hover:opacity-100 transition-opacity">
        <div class="flex space-x-1">
          <div class="w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full"></div>
          <div class="w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full"></div>
          <div class="w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full"></div>
          <div class="w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full"></div>
          <div class="w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full"></div>
        </div>
      </div>
    </div>
    
    <!-- Tab Bar with Controls -->
    <div class="flex-none bg-gray-100 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
      <div class="flex items-center justify-between">
        <!-- Left side: Tabs -->
        <div class="flex items-center overflow-x-auto flex-1">
          <div
            v-for="tab in terminalTabs"
            :key="tab.id"
            class="flex items-center min-w-0 border-r border-gray-200 dark:border-gray-600 group"
            :class="[
              activeTabId === tab.id 
                ? 'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100' 
                : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
            ]"
          >
            <button
              @click="setActiveTab(tab.id)"
              class="flex items-center px-3 py-2 text-sm font-medium min-w-0 flex-1 text-left"
              :title="getTabTooltip(tab)"
            >
              <!-- Tab type icon -->
              <div class="mr-2 flex-shrink-0">
                <!-- Shell icon -->
                <svg v-if="tab.type === 'shell'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
                </svg>
                <!-- Logs icon -->
                <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                </svg>
              </div>
              
              <!-- Connection status indicator for shells -->
              <div 
                v-if="tab.type === 'shell'"
                class="w-2 h-2 rounded-full mr-2 flex-shrink-0"
                :class="[
                  tab.isConnected 
                    ? 'bg-green-500 animate-pulse' 
                    : tab.isConnecting 
                      ? 'bg-yellow-500 animate-pulse' 
                      : 'bg-gray-400'
                ]"
              ></div>
              
              <!-- Live logging indicator for logs -->
              <div 
                v-else-if="tab.type === 'logs' && tab.isLiveLogging"
                class="w-2 h-2 bg-green-500 rounded-full mr-2 flex-shrink-0 animate-pulse"
              ></div>
              <div v-else-if="tab.type === 'logs'" class="w-2 h-2 bg-gray-400 rounded-full mr-2 flex-shrink-0"></div>
              
              <!-- Tab label -->
              <span class="truncate">
                {{ tab.podName }}
              </span>
            </button>
            
            <!-- Close button -->
            <button
              @click.stop="closeTab(tab.id)"
              class="p-1 mr-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-all"
              :title="`Close ${tab.type}`"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
        </div>
        
        <!-- Right side: Panel Controls -->
        <div class="flex items-center space-x-1 px-2">
          <!-- Session count -->
          <span class="text-xs text-gray-500 dark:text-gray-400 mr-2">
            {{ terminalTabs.length }} active
          </span>
          
          <!-- Close Button -->
          <button
            @click="$emit('close')"
            class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            title="Close panel"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Tab Content -->
    <div class="flex-1 relative">
      <div
        v-for="tab in terminalTabs"
        :key="tab.id"
        class="absolute inset-0"
        :class="{ 
          'opacity-0 pointer-events-none': activeTabId !== tab.id,
          'opacity-100': activeTabId === tab.id
        }"
        :style="{ 
          zIndex: activeTabId === tab.id ? 10 : 0,
          visibility: activeTabId === tab.id ? 'visible' : 'hidden'
        }"
      >
        <!-- Shell Tab Content -->
        <KideTerminal
          v-if="tab.type === 'shell'"
          :ref="(el) => setShellRef(tab.id, el)"
          :containers="tab.containers"
          :podName="tab.podName"
          :namespace="tab.namespace"
          :initialContainer="tab.containerName"
          :autoConnect="true"
          @close="closeTab(tab.id)"
          @connection-state-changed="updateTabConnectionState(tab.id, $event)"
        />
        
        <!-- Logs Tab Content -->
        <div v-else-if="tab.type === 'logs'" class="h-full">
          <ResourceLogs
            :ref="(el) => setLogRef(tab.id, el)"
            :containers="tab.containers"
            :initContainers="tab.initContainers"
            :podName="tab.podName"
            :namespace="tab.namespace"
            :initialContainer="tab.containerName"
            :logLines="tab.logLines || []"
            :isLiveLogging="tab.isLiveLogging || false"
            @refresh-logs="handleRefreshLogs(tab)"
            @toggle-live-logging="handleToggleLiveLogging(tab)"
            @container-changed="updateTabContainer(tab.id, $event)"
            @close="closeTab(tab.id)"
          />
        </div>
      </div>
    </div>
    
    <!-- Empty State -->
    <div
      v-if="terminalTabs.length === 0"
      class="flex-1 flex items-center justify-center bg-gray-50 dark:bg-gray-900"
    >
      <div class="text-center">
        <div class="text-gray-400 mb-4">
          <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"/>
          </svg>
        </div>
        <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">No Terminal Sessions</h3>
        <p class="text-gray-600 dark:text-gray-400">Click "Open shell" or "View logs" on any pod to start a session</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import KideTerminal from './KideTerminal.vue'
import ResourceLogs from './ResourceLogs.vue'

interface TerminalTab {
  id: string
  type: 'shell' | 'logs'
  podName: string
  namespace: string
  containerName: string
  containers: any[]
  initContainers?: any[]
  // Shell-specific properties
  isConnected?: boolean
  isConnecting?: boolean
  // Logs-specific properties
  logLines?: string[]
  isLiveLogging?: boolean
  streamId?: string
}

interface Props {
  maxTabs?: number
  isResizing?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  maxTabs: 10,
  isResizing: false
})

const emit = defineEmits<{
  'close': []
  'tab-changed': [tabId: string]
  'start-resize': [event: MouseEvent]
  'refresh-logs': [tab: TerminalTab]
  'toggle-live-logging': [tab: TerminalTab]
}>()

// State
const terminalTabs = ref<TerminalTab[]>([])
const activeTabId = ref<string | null>(null)

// Computed
const hasActiveTabs = computed(() => terminalTabs.value.length > 0)

// Methods
function addShellTab(podData: {
  podName: string
  namespace: string
  containerName: string
  containers: any[]
}): string {
  // Check if shell tab already exists for this pod/container
  const existingTab = terminalTabs.value.find(
    tab => tab.type === 'shell' &&
           tab.podName === podData.podName && 
           tab.namespace === podData.namespace && 
           tab.containerName === podData.containerName
  )
  
  if (existingTab) {
    // Switch to existing tab
    setActiveTab(existingTab.id)
    return existingTab.id
  }
  
  // Create new shell tab
  const tabId = `shell-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  const newTab: TerminalTab = {
    id: tabId,
    type: 'shell',
    podName: podData.podName,
    namespace: podData.namespace,
    containerName: podData.containerName,
    containers: podData.containers,
    isConnected: false,
    isConnecting: false
  }
  
  terminalTabs.value.push(newTab)
  setActiveTab(tabId)
  
  // Focus the new terminal after creation
  nextTick(() => {
    setTimeout(() => {
      focusActiveTerminal()
    }, 100)
  })
  
  return tabId
}

function addLogTab(podData: {
  podName: string
  namespace: string
  containerName: string
  containers: any[]
  initContainers?: any[]
}): string {
  // Check if log tab already exists for this pod/container
  const existingTab = terminalTabs.value.find(
    tab => tab.type === 'logs' &&
           tab.podName === podData.podName && 
           tab.namespace === podData.namespace && 
           tab.containerName === podData.containerName
  )
  
  if (existingTab) {
    // Switch to existing tab
    setActiveTab(existingTab.id)
    return existingTab.id
  }
  
  // Create new log tab
  const tabId = `logs-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  const newTab: TerminalTab = {
    id: tabId,
    type: 'logs',
    podName: podData.podName,
    namespace: podData.namespace,
    containerName: podData.containerName,
    containers: podData.containers,
    initContainers: podData.initContainers,
    logLines: [],
    isLiveLogging: false
  }
  
  terminalTabs.value.push(newTab)
  setActiveTab(tabId)
  
  // Load initial logs for the new tab
  nextTick(() => {
    loadLogsForTab(tabId)
  })
  
  return tabId
}

function setActiveTab(tabId: string): void {
  if (terminalTabs.value.find(tab => tab.id === tabId)) {
    activeTabId.value = tabId
    emit('tab-changed', tabId)
    
    // Focus the active terminal after tab switch (for shells)
    nextTick(() => {
      const activeTab = terminalTabs.value.find(tab => tab.id === tabId)
      if (activeTab?.type === 'shell') {
        focusActiveTerminal()
      }
    })
  }
}

function closeTab(tabId: string): void {
  const tabIndex = terminalTabs.value.findIndex(tab => tab.id === tabId)
  if (tabIndex === -1) return
  
  terminalTabs.value.splice(tabIndex, 1)
  
  // If we closed the active tab, switch to another one
  if (activeTabId.value === tabId) {
    if (terminalTabs.value.length > 0) {
      // Switch to the previous tab or the first one
      const newActiveIndex = Math.max(0, tabIndex - 1)
      setActiveTab(terminalTabs.value[newActiveIndex].id)
    } else {
      // No tabs left - emit close to indicate empty state
      activeTabId.value = null
      emit('close')
    }
  }
}

function updateTabConnectionState(tabId: string, state: { isConnected: boolean, isConnecting: boolean }): void {
  const tab = terminalTabs.value.find(t => t.id === tabId && t.type === 'shell')
  if (tab) {
    tab.isConnected = state.isConnected
    tab.isConnecting = state.isConnecting
    
    // Focus terminal when it becomes connected and is the active tab
    if (state.isConnected && !state.isConnecting && activeTabId.value === tabId) {
      nextTick(() => {
        focusActiveTerminal()
      })
    }
  }
}

function updateTabContainer(tabId: string, containerName: string): void {
  const tab = terminalTabs.value.find(t => t.id === tabId)
  if (tab) {
    tab.containerName = containerName
    
    // If this is a log tab, reload logs for the new container
    if (tab.type === 'logs') {
      tab.logLines = [] // Clear existing logs
      tab.isLiveLogging = false // Stop live logging if active
      loadLogsForTab(tabId) // Load logs for new container
    }
  }
}

function updateTabLogState(tabId: string, logState: { isLiveLogging: boolean, logLines?: string[] }): void {
  const tab = terminalTabs.value.find(t => t.id === tabId && t.type === 'logs')
  if (tab) {
    tab.isLiveLogging = logState.isLiveLogging
    if (logState.logLines) {
      tab.logLines = logState.logLines
    }
  }
}

function closeAllTabs(): void {
  terminalTabs.value = []
  activeTabId.value = null
}

function getTabTooltip(tab: TerminalTab): string {
  return `${tab.type.charAt(0).toUpperCase() + tab.type.slice(1)}: ${tab.podName}`
}

// Resize functionality
function startResize(event: MouseEvent): void {
  emit('start-resize', event)
}

// Component refs tracking
const shellRefs = ref<Map<string, any>>(new Map())
const logRefs = ref<Map<string, any>>(new Map())

function setShellRef(tabId: string, el: any): void {
  if (el) {
    shellRefs.value.set(tabId, el)
  } else {
    shellRefs.value.delete(tabId)
  }
}

function setLogRef(tabId: string, el: any): void {
  if (el) {
    logRefs.value.set(tabId, el)
  } else {
    logRefs.value.delete(tabId)
  }
}

// Helper function to focus the active terminal (shells only)
function focusActiveTerminal(): void {
  if (activeTabId.value) {
    const shellComponent = shellRefs.value.get(activeTabId.value)
    if (shellComponent) {
      // Use a more robust approach for terminal activation
      setTimeout(() => {
        // Refresh the terminal display first to ensure proper rendering
        if (typeof shellComponent.refreshTerminal === 'function') {
          shellComponent.refreshTerminal()
        }
        // Wait for refresh to complete before focusing
        setTimeout(() => {
          if (typeof shellComponent.focusTerminal === 'function') {
            shellComponent.focusTerminal()
          }
        }, 50)
      }, 50)
    }
  }
}

// Refresh all terminal displays (for shell tabs)
function refreshAllTerminals(): void {
  shellRefs.value.forEach((shellComponent) => {
    if (shellComponent && typeof shellComponent.refreshTerminal === 'function') {
      shellComponent.refreshTerminal()
    }
  })
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent): void {
  // Ctrl/Cmd + W to close current tab
  if ((event.ctrlKey || event.metaKey) && event.key === 'w' && activeTabId.value) {
    event.preventDefault()
    closeTab(activeTabId.value)
  }
  
  // Ctrl/Cmd + Tab to cycle through tabs
  if ((event.ctrlKey || event.metaKey) && event.key === 'Tab') {
    event.preventDefault()
    cycleTabs(event.shiftKey ? -1 : 1)
  }
  
  // Ctrl/Cmd + 1-9 to switch to specific tab
  if ((event.ctrlKey || event.metaKey) && /^[1-9]$/.test(event.key)) {
    event.preventDefault()
    const tabIndex = parseInt(event.key) - 1
    if (tabIndex < terminalTabs.value.length) {
      setActiveTab(terminalTabs.value[tabIndex].id)
    }
  }
}

function cycleTabs(direction: number): void {
  if (terminalTabs.value.length === 0) return
  
  const currentIndex = terminalTabs.value.findIndex(tab => tab.id === activeTabId.value)
  if (currentIndex === -1) return
  
  const newIndex = (currentIndex + direction + terminalTabs.value.length) % terminalTabs.value.length
  setActiveTab(terminalTabs.value[newIndex].id)
}

// Expose public methods
defineExpose({
  addShellTab,
  addLogTab,
  setActiveTab,
  closeTab,
  closeAllTabs,
  updateTabLogState,
  loadLogsForTab,
  toggleLiveLoggingForTab,
  refreshAllTerminals,
  terminalTabs: terminalTabs,
  activeTabId: activeTabId
})

// Log management functions
async function loadLogsForTab(tabId: string): Promise<void> {
  const tab = terminalTabs.value.find(t => t.id === tabId && t.type === 'logs')
  if (!tab) return
  
  try {
    const containerName = tab.containerName.startsWith('init-') 
      ? tab.containerName.replace('init-', '') 
      : tab.containerName
    
    const logs = await invoke<string>('get_pod_logs', {
      podName: tab.podName,
      namespace: tab.namespace,
      containerName: containerName,
      lines: 100
    })
    
    tab.logLines = logs.split('\n').filter(line => line.trim())
  } catch (error) {
    console.error('Error fetching logs:', error)
    tab.logLines = [`Error fetching logs: ${error}`]
  }
}

async function toggleLiveLoggingForTab(tabId: string): Promise<void> {
  const tab = terminalTabs.value.find(t => t.id === tabId && t.type === 'logs')
  if (!tab) return
  
  try {
    if (tab.isLiveLogging) {
      // Stop live logging
      await invoke('stop_pod_logs_stream', {
        podName: tab.podName,
        namespace: tab.namespace,
        containerName: tab.containerName.startsWith('init-') 
          ? tab.containerName.replace('init-', '') 
          : tab.containerName
      })
      tab.isLiveLogging = false
    } else {
      // Start live logging
      const containerName = tab.containerName.startsWith('init-') 
        ? tab.containerName.replace('init-', '') 
        : tab.containerName
      
      const streamId = await invoke<string>('start_pod_logs_stream', {
        podName: tab.podName,
        namespace: tab.namespace,
        containerName: containerName
      })
      
      tab.streamId = streamId
      tab.isLiveLogging = true
      
      // Set up log event listener if not already done
      setupLogEventListener()
    }
  } catch (error) {
    console.error('Error toggling live logging:', error)
    tab.isLiveLogging = false
  }
}

// Global log event listener (set up once)
let logListenerSetup = false
function setupLogEventListener(): void {
  if (logListenerSetup) return
  
  logListenerSetup = true
  listen<any>('pod-log-line', (event) => {
    // Find the tab that matches this stream ID
    const tab = terminalTabs.value.find(t => 
      t.type === 'logs' && t.streamId === event.payload.stream_id
    )
    
    if (tab) {
      if (!tab.logLines) tab.logLines = []
      tab.logLines.push(event.payload.line)
      
      // Keep only last 1000 lines to prevent memory issues
      if (tab.logLines.length > 1000) {
        tab.logLines = tab.logLines.slice(-1000)
      }
    }
  })
}

// Handle refresh logs event
function handleRefreshLogs(tab: TerminalTab): void {
  if (tab.type === 'logs') {
    loadLogsForTab(tab.id)
  }
}

// Handle toggle live logging event
function handleToggleLiveLogging(tab: TerminalTab): void {
  if (tab.type === 'logs') {
    toggleLiveLoggingForTab(tab.id)
  }
}

// Set up keyboard shortcuts on mount
import { onMounted, onUnmounted } from 'vue'

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  // Set up log event listener
  setupLogEventListener()
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>