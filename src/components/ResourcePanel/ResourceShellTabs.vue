<template>
  <div class="h-full flex flex-col">
    <!-- Resize Handle -->
    <div 
      v-if="!isMaximized"
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
        <!-- Left side: Shell Tabs -->
        <div class="flex items-center overflow-x-auto flex-1">
          <div
            v-for="tab in shellTabs"
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
              :title="`Shell: ${tab.podName}`"
            >
              <!-- Connection status indicator -->
              <div 
                class="w-2 h-2 rounded-full mr-2 flex-shrink-0"
                :class="[
                  tab.isConnected 
                    ? 'bg-green-500 animate-pulse' 
                    : tab.isConnecting 
                      ? 'bg-yellow-500 animate-pulse' 
                      : 'bg-gray-400'
                ]"
              ></div>
              
              <!-- Tab label -->
              <span class="truncate">
                {{ tab.podName }}
              </span>
            </button>
            
            <!-- Close button -->
            <button
              @click.stop="closeTab(tab.id)"
              class="p-1 mr-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-all"
              :title="'Close shell'"
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
            {{ shellTabs.length }} active
          </span>
          
          <!-- Minimize Button -->
          <button
            @click="$emit('minimize')"
            class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            title="Minimize panel (Ctrl+Shift+D)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"/>
            </svg>
          </button>
          
          <!-- Maximize/Restore Button -->
          <button
            @click="$emit('toggle-maximize')"
            class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            :title="(isMaximized ? 'Restore panel (Esc)' : 'Maximize panel (Ctrl+Shift+M)')"
          >
            <svg v-if="!isMaximized" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"/>
            </svg>
            <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.5 3.5M15 9v-4.5M15 9h4.5M15 9l5.5-5.5M9 15v4.5M9 15H4.5M9 15l-5.5 5.5M15 15v4.5M15 15h4.5m0 0l5.5 5.5"/>
            </svg>
          </button>
          
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
        v-for="tab in shellTabs"
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
        <ResourceShell
          :ref="(el) => setShellRef(tab.id, el)"
          :containers="tab.containers"
          :podName="tab.podName"
          :namespace="tab.namespace"
          :initialContainer="tab.containerName"
          :autoConnect="true"
          :isVisible="activeTabId === tab.id"
          @close="closeTab(tab.id)"
          @connection-state-changed="updateTabConnectionState(tab.id, $event)"
        />
      </div>
    </div>
    
    <!-- Empty State -->
    <div
      v-if="shellTabs.length === 0"
      class="flex-1 flex items-center justify-center bg-gray-50 dark:bg-gray-900"
    >
      <div class="text-center">
        <div class="text-gray-400 mb-4">
          <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"/>
          </svg>
        </div>
        <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">No Shell Sessions</h3>
        <p class="text-gray-600 dark:text-gray-400">Click "Open shell" on any pod to start a terminal session</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import ResourceShell from './ResourceShell.vue'

interface ShellTab {
  id: string
  podName: string
  namespace: string
  containerName: string
  containers: any[]
  isConnected: boolean
  isConnecting: boolean
}

interface Props {
  maxTabs?: number
  isMaximized?: boolean
  isResizing?: boolean
}

withDefaults(defineProps<Props>(), {
  maxTabs: 10,
  isMaximized: false,
  isResizing: false
})

const emit = defineEmits<{
  'close': []
  'tab-changed': [tabId: string]
  'minimize': []
  'toggle-maximize': []
  'start-resize': [event: MouseEvent]
}>()

// State
const shellTabs = ref<ShellTab[]>([])
const activeTabId = ref<string | null>(null)


// Methods
function addShellTab(podData: {
  podName: string
  namespace: string
  containerName: string
  containers: any[]
}): string {
  // Check if tab already exists for this pod/container
  const existingTab = shellTabs.value.find(
    tab => tab.podName === podData.podName && 
           tab.namespace === podData.namespace && 
           tab.containerName === podData.containerName
  )
  
  if (existingTab) {
    // Switch to existing tab
    setActiveTab(existingTab.id)
    return existingTab.id
  }
  
  // Create new tab
  const tabId = `shell-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`
  const newTab: ShellTab = {
    id: tabId,
    podName: podData.podName,
    namespace: podData.namespace,
    containerName: podData.containerName,
    containers: podData.containers,
    isConnected: false,
    isConnecting: false
  }
  
  shellTabs.value.push(newTab)
  setActiveTab(tabId)
  
  // Focus the new terminal after creation
  nextTick(() => {
    setTimeout(() => {
      focusActiveTerminal()
    }, 100) // Small delay to ensure terminal is fully initialized
  })
  
  return tabId
}

function setActiveTab(tabId: string): void {
  if (shellTabs.value.find(tab => tab.id === tabId)) {
    activeTabId.value = tabId
    emit('tab-changed', tabId)
    
    // Focus the active terminal after tab switch
    nextTick(() => {
      focusActiveTerminal()
    })
  }
}

function closeTab(tabId: string): void {
  const tabIndex = shellTabs.value.findIndex(tab => tab.id === tabId)
  if (tabIndex === -1) return
  
  shellTabs.value.splice(tabIndex, 1)
  
  // If we closed the active tab, switch to another one
  if (activeTabId.value === tabId) {
    if (shellTabs.value.length > 0) {
      // Switch to the previous tab or the first one
      const newActiveIndex = Math.max(0, tabIndex - 1)
      setActiveTab(shellTabs.value[newActiveIndex].id)
    } else {
      // No tabs left - emit close to indicate empty state
      activeTabId.value = null
      emit('close')
    }
  }
}

function updateTabConnectionState(tabId: string, state: { isConnected: boolean, isConnecting: boolean }): void {
  const tab = shellTabs.value.find(t => t.id === tabId)
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

function closeAllTabs(): void {
  shellTabs.value = []
  activeTabId.value = null
  // Don't emit 'close' here as it can cause infinite loops
  // The panel close should be initiated from the parent
}

// Resize functionality
function startResize(event: MouseEvent): void {
  emit('start-resize', event)
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent): void {
  // Ctrl/Cmd + W to close current tab
  if ((event.ctrlKey || event.metaKey) && event.key === 'w' && activeTabId.value) {
    event.preventDefault()
    closeTab(activeTabId.value)
  }
  
  // Ctrl/Cmd + T removed - users should open shells from main interface
  
  // Ctrl/Cmd + Tab to cycle through tabs
  if ((event.ctrlKey || event.metaKey) && event.key === 'Tab') {
    event.preventDefault()
    cycleTabs(event.shiftKey ? -1 : 1)
  }
  
  // Ctrl/Cmd + 1-9 to switch to specific tab
  if ((event.ctrlKey || event.metaKey) && /^[1-9]$/.test(event.key)) {
    event.preventDefault()
    const tabIndex = parseInt(event.key) - 1
    if (tabIndex < shellTabs.value.length) {
      setActiveTab(shellTabs.value[tabIndex].id)
    }
  }
}

function cycleTabs(direction: number): void {
  if (shellTabs.value.length === 0) return
  
  const currentIndex = shellTabs.value.findIndex(tab => tab.id === activeTabId.value)
  if (currentIndex === -1) return
  
  const newIndex = (currentIndex + direction + shellTabs.value.length) % shellTabs.value.length
  setActiveTab(shellTabs.value[newIndex].id)
}

// Shell component refs tracking
const shellRefs = ref<Map<string, any>>(new Map())

function setShellRef(tabId: string, el: any): void {
  if (el) {
    shellRefs.value.set(tabId, el)
  } else {
    shellRefs.value.delete(tabId)
  }
}

// Helper function to focus the active terminal
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

// Expose public methods
defineExpose({
  addShellTab,
  setActiveTab,
  closeTab,
  closeAllTabs,
  shellTabs: shellTabs,
  activeTabId: activeTabId
})

// Set up keyboard shortcuts on mount
import { onMounted, onUnmounted } from 'vue'

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>