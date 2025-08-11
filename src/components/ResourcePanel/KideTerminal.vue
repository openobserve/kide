<template>
  <div class="kide-terminal">
    <!-- Terminal Container -->
    <div 
      ref="terminalContainer" 
      class="terminal-container"
      :style="{ height: '100%', width: '100%' }"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
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

// Kide terminal implementation
class KideTerminal {
  private terminal: Terminal | null = null
  private fitAddon: FitAddon | null = null
  private resizeObserver: ResizeObserver | null = null
  private container: HTMLElement | null = null
  private sessionId: string | null = null
  private isConnected = false
  private isConnecting = false
  private shellEventUnlistener: UnlistenFn | null = null
  
  constructor(container: HTMLElement) {
    this.container = container
    this.initialize()
  }
  
  private initialize() {
    if (!this.container) return
    
    console.log('🚀 KideTerminal: Initializing terminal')
    
    // Create terminal with optimal settings
    this.terminal = new Terminal({
      theme: {
        background: '#1e1e1e',  // Dark theme
        foreground: '#cccccc',
        cursor: '#ffffff',
      },
      fontFamily: '"Cascadia Code", "Fira Code", Monaco, Menlo, "Ubuntu Mono", monospace',
      fontSize: 14,
      lineHeight: 1.2,
      letterSpacing: 0,
      cursorBlink: true,
      cursorStyle: 'block',
      scrollback: 1000,
      convertEol: true,
      allowTransparency: false,
    })
    
    // Create and load addons
    this.fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()
    
    this.terminal.loadAddon(this.fitAddon)
    this.terminal.loadAddon(webLinksAddon)
    
    // Open terminal in container
    this.terminal.open(this.container)
    
    // Initial fit
    this.fitAddon.fit()
    
    // Set up resize handling
    this.setupResizeHandling()
    
    // Set up terminal event handlers
    this.setupEventHandlers()
    
    console.log('✅ KideTerminal: Terminal initialized')
  }
  
  private setupResizeHandling() {
    if (!this.container || !this.terminal || !this.fitAddon) return
    
    console.log('🔧 KideTerminal: Setting up resize handling')
    
    // Use ResizeObserver for dynamic container resizing
    this.resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        this.handleResize(entry.contentRect.width, entry.contentRect.height)
      }
    })
    
    this.resizeObserver.observe(this.container)
    
    // Also listen for window resize
    window.addEventListener('resize', this.handleWindowResize)
  }
  
  private handleResize = (width: number, height: number) => {
    if (!this.terminal || !this.fitAddon || width <= 0 || height <= 0) return
    
    console.log('📏 KideTerminal: Handling resize', { width, height })
    
    // Optimal approach: Fit to container, then ensure visibility
    try {
      // Fit terminal to new dimensions
      this.fitAddon.fit()
      
      // Key insight: Keep the current cursor/prompt position visible
      // If we were at the bottom, stay at the bottom
      // If user was scrolled up, maintain that scroll position
      
      const buffer = this.terminal.buffer.active
      const wasAtBottom = this.isAtBottom()
      
      console.log('📍 KideTerminal: Resize - was at bottom:', wasAtBottom)
      
      if (wasAtBottom) {
        // If we were at bottom, scroll to bottom after resize
        this.terminal.scrollToBottom()
        console.log('📜 KideTerminal: Scrolled to bottom after resize')
      }
      // If user was scrolled up, let them stay there (Kide behavior)
      
      // Update backend size if connected
      if (this.isConnected && this.sessionId) {
        this.updateTerminalSize()
      }
      
    } catch (error) {
      console.error('❌ KideTerminal: Resize error', error)
    }
  }
  
  private handleWindowResize = () => {
    // Debounce window resize events
    if (this.resizeTimeout) {
      clearTimeout(this.resizeTimeout)
    }
    
    this.resizeTimeout = setTimeout(() => {
      if (this.container) {
        this.handleResize(this.container.clientWidth, this.container.clientHeight)
      }
    }, 100)
  }
  
  private resizeTimeout: NodeJS.Timeout | null = null
  
  private isAtBottom(): boolean {
    if (!this.terminal) return true
    
    const buffer = this.terminal.buffer.active
    const viewportHeight = this.terminal.rows
    const currentViewportTop = buffer.viewportY || 0
    const bufferHeight = buffer.length
    
    // Calculate if we're at the bottom (within 2 lines)
    const viewportBottom = currentViewportTop + viewportHeight
    return viewportBottom >= bufferHeight - 2
  }
  
  private setupEventHandlers() {
    if (!this.terminal) return
    
    // Handle user input
    this.terminal.onData((data) => {
      if (this.isConnected && this.sessionId) {
        this.sendInput(data)
      }
    })
    
    // Handle terminal resize events
    this.terminal.onResize(({ cols, rows }) => {
      console.log('📏 KideTerminal: Terminal resized to', { cols, rows })
      if (this.isConnected && this.sessionId) {
        this.updateTerminalSize()
      }
    })
  }
  
  // Public methods
  public async connect(podName: string, namespace: string, containerName: string): Promise<void> {
    if (this.isConnecting || this.isConnected) return
    
    console.log('🔗 KideTerminal: Connecting to', { podName, namespace, containerName })
    
    this.isConnecting = true
    
    try {
      if (this.terminal) {
        this.terminal.clear()
        this.terminal.writeln('\r\n🔗 Connecting to shell...\r\n')
      }
      
      const cols = this.terminal?.cols || 80
      const rows = this.terminal?.rows || 24
      
      const response = await invoke<string>('start_pod_shell', {
        podName,
        namespace,
        containerName,
        cols,
        rows
      })
      
      this.sessionId = response
      this.isConnected = true
      
      await this.setupShellEventListener()
      
      if (this.terminal) {
        this.terminal.writeln('✅ Connected to shell\r\n')
        this.terminal.focus()
      }
      
      console.log('✅ KideTerminal: Connected successfully')
      
    } catch (error) {
      console.error('❌ KideTerminal: Connection failed', error)
      if (this.terminal) {
        this.terminal.writeln(`\r\n❌ Connection failed: ${error}\r\n`)
      }
    } finally {
      this.isConnecting = false
    }
  }
  
  private async setupShellEventListener(): Promise<void> {
    try {
      this.shellEventUnlistener = await listen<any>('shell-output', (event) => {
        if (this.sessionId && event.payload.session_id === this.sessionId) {
          if (this.terminal) {
            this.terminal.write(event.payload.data)
            
            // Optimal behavior: Only auto-scroll if user is at bottom
            if (this.isAtBottom()) {
              this.terminal.scrollToBottom()
            }
          }
        }
      })
    } catch (error) {
      console.error('❌ KideTerminal: Failed to setup shell listener', error)
    }
  }
  
  private async sendInput(data: string): Promise<void> {
    if (!this.isConnected || !this.sessionId) return
    
    try {
      await invoke('send_shell_input', {
        sessionId: this.sessionId,
        data: data
      })
    } catch (error) {
      console.error('❌ KideTerminal: Failed to send input', error)
    }
  }
  
  private async updateTerminalSize(): Promise<void> {
    if (!this.isConnected || !this.sessionId || !this.terminal) return
    
    try {
      await invoke('resize_shell', {
        sessionId: this.sessionId,
        cols: this.terminal.cols,
        rows: this.terminal.rows
      })
    } catch (error) {
      console.error('❌ KideTerminal: Failed to update size', error)
    }
  }
  
  public focus() {
    if (this.terminal) {
      this.terminal.focus()
    }
  }
  
  public dispose() {
    console.log('🗑️ KideTerminal: Disposing terminal')
    
    // Clean up event listeners
    if (this.shellEventUnlistener) {
      this.shellEventUnlistener()
      this.shellEventUnlistener = null
    }
    
    if (this.resizeObserver) {
      this.resizeObserver.disconnect()
      this.resizeObserver = null
    }
    
    window.removeEventListener('resize', this.handleWindowResize)
    
    if (this.resizeTimeout) {
      clearTimeout(this.resizeTimeout)
      this.resizeTimeout = null
    }
    
    // Disconnect if connected
    if (this.isConnected && this.sessionId) {
      invoke('stop_pod_shell', { sessionId: this.sessionId }).catch(console.error)
    }
    
    // Dispose terminal
    if (this.terminal) {
      this.terminal.dispose()
      this.terminal = null
    }
    
    this.fitAddon = null
    this.container = null
  }
}

// Component state
const terminalContainer = ref<HTMLElement | null>(null)
const kideTerminal = ref<KideTerminal | null>(null)

// Lifecycle
onMounted(async () => {
  await nextTick() // Ensure DOM is ready
  
  if (terminalContainer.value) {
    kideTerminal.value = new KideTerminal(terminalContainer.value)
    
    // Auto-connect if requested
    if (props.autoConnect && props.podName && props.namespace) {
      const containerName = props.initialContainer || props.containers?.[0]?.name || 'main'
      await kideTerminal.value.connect(props.podName, props.namespace, containerName)
    }
  }
})

onUnmounted(() => {
  if (kideTerminal.value) {
    kideTerminal.value.dispose()
    kideTerminal.value = null
  }
})

// Expose methods
defineExpose({
  focusTerminal: () => {
    kideTerminal.value?.focus()
  },
  refreshTerminal: () => {
    // ResizeObserver handles refresh automatically
    if (terminalContainer.value && kideTerminal.value) {
      const container = terminalContainer.value
      // Call the private method through a public interface
      ;(kideTerminal.value as any).handleResize(container.clientWidth, container.clientHeight)
    }
  }
})
</script>

<style scoped>
.kide-terminal {
  height: 100%;
  width: 100%;
  background-color: #1e1e1e; /* Dark theme */
}

.terminal-container {
  height: 100%;
  width: 100%;
}

/* Kide terminal styles */
:deep(.xterm) {
  height: 100%;
  width: 100%;
  padding: 8px; /* Padding around terminal */
}

:deep(.xterm-viewport) {
  overflow-y: auto;
  overflow-x: hidden;
  background-color: #1e1e1e;
}

:deep(.xterm-screen) {
  background-color: #1e1e1e;
}

/* Kide terminal scrollbar styling */
:deep(.xterm-viewport::-webkit-scrollbar) {
  width: 14px;
}

:deep(.xterm-viewport::-webkit-scrollbar-track) {
  background: #1e1e1e;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb) {
  background: #424242;
  border-radius: 7px;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb:hover) {
  background: #4f4f4f;
}
</style>