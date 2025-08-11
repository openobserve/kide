<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex-none px-6 py-2 border-b border-gray-200 bg-gray-50 dark:bg-gray-700 dark:border-gray-600">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <!-- Pod Information -->
          <div v-if="podName || namespace" class="mr-2">
            <div class="text-sm font-semibold text-gray-700 dark:text-gray-300">
              <span v-if="podName">{{ podName }}</span>
              <span v-if="namespace" class="text-xs text-gray-500 dark:text-gray-400 ml-1">({{ namespace }})</span>
            </div>
          </div>
          
          <!-- Container Selection -->
          <select v-model="selectedContainer" class="text-sm border border-gray-300 rounded px-2 py-1">
            <!-- Regular Containers -->
            <optgroup v-if="containers?.length" label="Containers">
              <option v-for="container in containers" :key="container.name" :value="container.name">
                {{ container.name }}
              </option>
            </optgroup>
          </select>
          
          <!-- Shell Selection -->
          <select v-model="selectedShell" class="text-sm border border-gray-300 rounded px-2 py-1">
            <option value="bash">bash</option>
            <option value="sh">sh</option>
            <option value="zsh">zsh</option>
          </select>
          
          <!-- Connect Button -->
          <button 
            v-if="!isConnected"
            @click="connect" 
            :disabled="isConnecting"
            :class="[
              'text-sm px-3 py-1 rounded transition-colors bg-green-600 text-white hover:bg-green-700',
              isConnecting ? 'opacity-50 cursor-not-allowed' : ''
            ]"
          >
            {{ isConnecting ? 'Connecting...' : 'Connect' }}
          </button>
        </div>
        
        <!-- Connection Status and Close -->
        <div class="flex items-center space-x-2">
          <!-- Connection status indicator -->
          <div v-if="isConnected" class="flex items-center text-sm text-green-500">
            <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse"></div>
            Connected
          </div>
          
          <div v-else-if="connectionError" class="flex items-center text-sm text-red-500">
            <div class="w-2 h-2 bg-red-500 rounded-full mr-2"></div>
            Error
          </div>
          
          <!-- Close button -->
          <button 
            @click="$emit('close')"
            class="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 transition-colors p-1 rounded"
            title="Close Shell"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Terminal Container -->
    <div class="flex-1 bg-black">
      <div ref="terminalContainer" class="h-full w-full"></div>
    </div>
    
    <!-- Error Display -->
    <div v-if="connectionError" class="flex-none bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-800 px-4 py-2">
      <div class="flex items-center text-sm text-red-700 dark:text-red-300">
        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        {{ connectionError }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'

interface Props {
  containers?: any[]
  podName?: string
  namespace?: string
  initialContainer?: string
  autoConnect?: boolean
  isVisible?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'close': []
  'container-changed': [container: string]
  'connection-state-changed': [state: { isConnected: boolean, isConnecting: boolean }]
}>()

// Focus management
function focusTerminal(): void {
  if (terminal.value) {
    // Focus the terminal element
    terminal.value.focus()
  }
}

// Check if terminal is scrolled to bottom
function isAtBottom(): boolean {
  if (!terminal.value) return true
  
  // Check if viewport is at the bottom of the buffer
  const buffer = terminal.value.buffer.active
  const viewport = terminal.value.rows
  const scrollTop = buffer.viewportY || 0
  
  // Consider "at bottom" if within 1 line of the actual bottom
  return scrollTop >= buffer.length - viewport - 1
}

// Refresh terminal display
function refreshTerminal(): void {
  if (terminal.value && fitAddon) {
    try {
      // Check if terminal container has valid dimensions
      const container = terminalContainer.value
      if (!container || container.offsetWidth === 0 || container.offsetHeight === 0) {
        console.warn('Terminal container has invalid dimensions, skipping refresh')
        return
      }
      
      fitAddon.fit()
      
      // Always scroll to bottom on refresh (user-initiated action)
      nextTick(() => {
        if (terminal.value) {
          terminal.value.scrollToBottom()
          
          // Update terminal size if connected
          if (isConnected.value && sessionId.value) {
            updateTerminalSize()
          }
        }
      })
    } catch (error) {
      console.warn('Error refreshing terminal:', error)
    }
  }
}

// Expose focus and refresh methods for parent component
defineExpose({
  focusTerminal,
  refreshTerminal
})

// State
const terminalContainer = ref<HTMLElement | null>(null)
const selectedContainer = ref('')
const selectedShell = ref('bash')
const isConnected = ref(false)
const isConnecting = ref(false)
const connectionError = ref<string | null>(null)
const sessionId = ref<string | null>(null)

// Terminal instances
const terminal = ref<Terminal | null>(null)
let fitAddon: FitAddon | null = null
let shellEventUnlistener: UnlistenFn | null = null
let isCleaningUp = false

// Initialize selected container
watch([() => props.containers, () => props.initialContainer], ([containers, initialContainer]) => {
  if (initialContainer) {
    selectedContainer.value = initialContainer
  } else if (containers?.length && !selectedContainer.value) {
    selectedContainer.value = containers[0].name
  }
}, { immediate: true })

// Watch for container changes
watch(selectedContainer, (value) => {
  emit('container-changed', value)
  // Disconnect if connected when container changes
  if (isConnected.value) {
    disconnect()
  }
})

// Watch for connection state changes and emit them
watch([isConnected, isConnecting], ([connected, connecting]) => {
  emit('connection-state-changed', { isConnected: connected, isConnecting: connecting })
})

// Watch for visibility changes to refresh terminal
watch(() => props.isVisible, (isVisible) => {
  if (isVisible && terminal.value) {
    // Simple visibility handling
    nextTick(() => {
      setTimeout(() => {
        refreshTerminal()
        if (isConnected.value) {
          focusTerminal()
        }
      }, 100) // Small delay for DOM stability
    })
  }
})

// Initialize terminal
onMounted(async () => {
  await initializeTerminal()
})

// Auto-connect when conditions are met
watch([() => props.autoConnect, selectedContainer, terminal], async ([autoConnect, container, terminalInstance]) => {
  if (autoConnect && props.podName && props.namespace && container && terminalInstance) {
    // Small delay to ensure everything is initialized
    setTimeout(() => {
      if (!isConnected.value && !isConnecting.value) {
        connect()
      }
    }, 500) // Increased delay to ensure full initialization
  }
}, { immediate: true })

// Cleanup on unmount
onUnmounted(() => {
  cleanup()
})

async function initializeTerminal(): Promise<void> {
  if (!terminalContainer.value) return
  
  try {
    // Create terminal instance with proper terminal capabilities
    terminal.value = new Terminal({
      theme: {
        background: '#000000',
        foreground: '#ffffff',
        cursor: '#ffffff',
      },
      fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
      fontSize: 11,
      cursorBlink: true,
      cursorStyle: 'block',
      scrollback: 1000,
      tabStopWidth: 4,
      // Enable terminal features for proper shell interaction
      allowProposedApi: true,
      convertEol: true,
      disableStdin: false,
      // Properly handle escape sequences and special keys
      macOptionIsMeta: true,
      rightClickSelectsWord: true,
      screenReaderMode: false,
    })
    
    // Add addons
    fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()
    
    terminal.value.loadAddon(fitAddon)
    terminal.value.loadAddon(webLinksAddon)
    
    // Open terminal
    terminal.value.open(terminalContainer.value)
    
    // Fit terminal to container
    fitAddon.fit()
    
    // Handle terminal resize
    const resizeObserver = new ResizeObserver(() => {
      if (fitAddon && terminal.value) {
        fitAddon.fit()
        // Update terminal size on backend if connected
        if (isConnected.value && sessionId.value) {
          updateTerminalSize()
        }
      }
    })
    resizeObserver.observe(terminalContainer.value)
    
    // Handle user input
    terminal.value.onData((data) => {
      if (isConnected.value && sessionId.value) {
        sendInput(data)
        // Always scroll to bottom on user input
        terminal.value?.scrollToBottom()
      }
    })
    
    // Welcome message
    terminal.value.writeln('\r\n\x1b[1;32m✨ Kide Terminal\x1b[0m')
    if (props.autoConnect) {
      terminal.value.writeln('\r\n\x1b[1;33m⏳ Connecting to shell...\x1b[0m\r\n')
    } else {
      terminal.value.writeln('\r\nSelect a container and click "Connect" to start a shell session.\r\n')
    }
    
  } catch (error) {
    console.error('Failed to initialize terminal:', error)
    connectionError.value = 'Failed to initialize terminal'
  }
}

async function connect(): Promise<void> {
  if (!props.podName || !props.namespace || !selectedContainer.value) {
    connectionError.value = 'Missing pod name, namespace, or container'
    return
  }
  
  if (isConnecting.value || isConnected.value) {
    return
  }
  
  isConnecting.value = true
  connectionError.value = null
  
  try {
    terminal.value?.clear()
    terminal.value?.writeln('\r\n\x1b[1;33mConnecting to shell...\x1b[0m\r\n')
    
    // Get terminal dimensions
    const cols = terminal.value?.cols || 80
    const rows = terminal.value?.rows || 24
    
    // Start shell session
    const response = await invoke<string>('start_pod_shell', {
      podName: props.podName,
      namespace: props.namespace,
      containerName: selectedContainer.value,
      cols,
      rows
    })
    
    sessionId.value = response
    isConnected.value = true
    
    // Set up event listener for shell output
    await setupShellEventListener()
    
    terminal.value?.writeln('\x1b[1;32m✅ Connected to shell\x1b[0m\r\n')
    
    // Focus the terminal for immediate typing
    setTimeout(() => {
      if (isConnected.value) {
        focusTerminal()
      }
    }, 500)
    
  } catch (error) {
    console.error('Failed to connect to shell:', error)
    connectionError.value = `Failed to connect: ${error}`
    terminal.value?.writeln(`\r\n\x1b[1;31m❌ Connection failed: ${error}\x1b[0m\r\n`)
  } finally {
    isConnecting.value = false
  }
}

async function disconnect(): Promise<void> {
  if (!isConnected.value || !sessionId.value) return
  
  try {
    // Stop shell session
    await invoke('stop_pod_shell', {
      sessionId: sessionId.value
    })
    
    // Only write to terminal if it's still available and we're not cleaning up
    if (terminal.value && !isCleaningUp) {
      try {
        terminal.value.writeln('\r\n\x1b[1;33m🔌 Disconnected from shell\x1b[0m\r\n')
      } catch (error) {
        console.warn('Could not write disconnect message to terminal:', error)
      }
    }
    
  } catch (error) {
    console.error('Error disconnecting from shell:', error)
    if (terminal.value && !isCleaningUp) {
      try {
        terminal.value.writeln(`\r\n\x1b[1;31m❌ Disconnect error: ${error}\x1b[0m\r\n`)
      } catch (terminalError) {
        console.warn('Could not write error message to terminal:', terminalError)
      }
    }
  } finally {
    isConnected.value = false
    sessionId.value = null
    
    // Clean up event listener
    if (shellEventUnlistener) {
      try {
        shellEventUnlistener()
      } catch (error) {
        console.warn('Error cleaning up shell event listener during disconnect:', error)
      } finally {
        shellEventUnlistener = null
      }
    }
  }
}

async function setupShellEventListener(): Promise<void> {
  try {
    // Listen for shell output
    shellEventUnlistener = await listen<any>('shell-output', (event) => {
      if (sessionId.value && event.payload.session_id === sessionId.value) {
        terminal.value?.write(event.payload.data)
        // Only auto-scroll if user hasn't manually scrolled up
        // This respects user intent when reviewing terminal history
        if (terminal.value && isAtBottom()) {
          terminal.value.scrollToBottom()
        }
      }
    })
    
    // Also listen for shell exit events
    listen<any>('shell-exit', (event) => {
      if (sessionId.value && event.payload.session_id === sessionId.value) {
        terminal.value?.writeln('\r\n\x1b[1;31m📡 Shell session ended\x1b[0m\r\n')
        isConnected.value = false
        sessionId.value = null
      }
    })
  } catch (error) {
    console.error('Failed to setup shell event listener:', error)
  }
}

async function sendInput(data: string): Promise<void> {
  if (!isConnected.value || !sessionId.value) return
  
  try {
    await invoke('send_shell_input', {
      sessionId: sessionId.value,
      data: data
    })
  } catch (error) {
    console.error('Failed to send input:', error)
    connectionError.value = `Input error: ${error}`
  }
}

async function updateTerminalSize(): Promise<void> {
  if (!isConnected.value || !sessionId.value || !terminal.value) return
  
  try {
    await invoke('resize_shell', {
      sessionId: sessionId.value,
      cols: terminal.value.cols,
      rows: terminal.value.rows
    })
  } catch (error) {
    console.error('Failed to update terminal size:', error)
  }
}

function cleanup(): void {
  // Prevent multiple cleanup calls
  if (isCleaningUp) {
    return
  }
  isCleaningUp = true
  
  try {
    // First disconnect any active connections
    if (isConnected.value) {
      disconnect()
    }
    
    // Clean up event listeners
    if (shellEventUnlistener) {
      try {
        shellEventUnlistener()
      } catch (error) {
        console.warn('Error cleaning up shell event listener:', error)
      } finally {
        shellEventUnlistener = null
      }
    }
    
    // Clean up terminal with proper error handling
    if (terminal.value) {
      try {
        // Clear addon reference (FitAddon doesn't have dispose method)
        if (fitAddon) {
          fitAddon = null
        }
        
        // Finally dispose the terminal
        terminal.value.dispose()
      } catch (error) {
        console.warn('Error disposing terminal:', error)
      } finally {
        terminal.value = null
      }
    } else {
      // Still clean up fitAddon even if terminal is null
      if (fitAddon) {
        fitAddon = null
      }
    }
  } catch (error) {
    console.error('Error during cleanup:', error)
  } finally {
    isCleaningUp = false
  }
}
</script>