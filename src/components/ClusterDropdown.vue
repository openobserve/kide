<!-- ClusterDropdown.vue - Compact cluster dropdown for title bar -->
<template>
  <div class="relative" data-dropdown>
    <!-- Dropdown button -->
    <button
      @click="toggleDropdown"
      :class="[
        'flex items-center space-x-2 px-3 py-1 rounded text-xs font-medium transition-colors min-w-0',
        'bg-surface-secondary border border-border-primary hover:bg-surface-tertiary focus:outline-none focus:ring-1 focus:ring-blue-500'
      ]"
      :title="selectedContext?.name"
    >
      <!-- Connection status indicator -->
      <div 
        :class="[
          'w-1.5 h-1.5 rounded-full flex-shrink-0',
          selectedContext ? (
            getContextStatus(selectedContext) === 'connected' ? 'bg-status-success' :
            getContextStatus(selectedContext) === 'connecting' ? 'bg-status-warning' :
            getContextStatus(selectedContext) === 'failed' ? 'bg-red-500' : 'bg-text-muted'
          ) : 'bg-text-muted'
        ]"
      />
      
      <!-- Selected context name -->
      <span class="truncate max-w-48 text-text-primary text-xs" :title="selectedContext?.name">
        {{ selectedContext ? truncateContextName(selectedContext.name) : 'Select Cluster' }}
      </span>
      
      <!-- Dropdown arrow -->
      <svg 
        :class="['w-3 h-3 text-text-muted transition-transform', isDropdownOpen ? 'rotate-180' : '']" 
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
      </svg>
    </button>

    <!-- Dropdown menu -->
    <div 
      v-show="isDropdownOpen"
      class="absolute top-full left-0 mt-1 w-72 bg-surface-primary border border-border-primary rounded-lg shadow-lg z-50 max-h-80 overflow-y-auto"
    >
      <!-- Search/Filter input -->
      <div class="p-2 border-b border-border-primary">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search clusters..."
          class="w-full px-2 py-1 text-xs bg-surface-secondary border border-border-primary rounded focus:outline-none focus:ring-1 focus:ring-blue-500 text-text-primary placeholder-text-muted"
        />
      </div>
      
      <!-- Cluster list -->
      <div class="py-1">
        <div
          v-for="context in filteredContexts"
          :key="context.name"
          class="relative"
        >
          <button
            @click="selectContextFromDropdown(context)"
            :class="[
              'w-full flex items-center space-x-2 px-3 py-2 text-xs transition-colors text-left',
              context.name === selectedContext?.name 
                ? 'bg-blue-900/20 text-blue-300 border-r-2 border-blue-500' 
                : 'text-text-primary hover:bg-surface-secondary'
            ]"
            :title="context.name"
          >
            <!-- Connection status indicator -->
            <div 
              :class="[
                'w-1.5 h-1.5 rounded-full flex-shrink-0',
                getContextStatus(context) === 'connected' ? 'bg-status-success' :
                getContextStatus(context) === 'connecting' ? 'bg-status-warning animate-pulse' :
                getContextStatus(context) === 'failed' ? 'bg-red-500' : 'bg-text-muted'
              ]"
            />
            
            <!-- Context details -->
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate text-xs">
                {{ context.name }}
              </div>
              <div v-if="context.cluster" class="text-xs text-text-secondary truncate">
                {{ context.cluster }}
              </div>
            </div>
            
            <!-- Selected indicator -->
            <svg 
              v-if="context.name === selectedContext?.name"
              class="w-3 h-3 text-blue-400 flex-shrink-0" 
              fill="currentColor" 
              viewBox="0 0 20 20"
            >
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
            </svg>
          </button>
        </div>
        
        <!-- No clusters found -->
        <div v-if="filteredContexts.length === 0 && searchQuery" class="px-3 py-2 text-xs text-text-secondary text-center">
          No clusters found matching "{{ searchQuery }}"
        </div>
        
        <!-- No clusters available -->
        <div v-if="contexts.length === 0" class="px-3 py-2 text-xs text-text-secondary text-center">
          No clusters available
        </div>
      </div>
      
      <!-- Refresh button at bottom -->
      <div class="border-t border-border-primary p-2">
        <button
          @click="refreshContexts"
          :disabled="refreshing"
          class="w-full flex items-center justify-center space-x-1 px-2 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-surface-secondary rounded transition-colors disabled:opacity-50"
          title="Refresh contexts"
        >
          <svg 
            :class="['w-3 h-3', refreshing ? 'animate-spin' : '']" 
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          <span>{{ refreshing ? 'Refreshing...' : 'Refresh' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { K8sContext, ContextStatus } from '@/types'

interface Props {
  selectedContext?: K8sContext | null
  connectionStatus?: Record<string, ContextStatus>
}

interface Emits {
  (e: 'context-selected', context: K8sContext): void
  (e: 'refresh-contexts'): void
}

const props = withDefaults(defineProps<Props>(), {
  selectedContext: null,
  connectionStatus: () => ({})
})

const emit = defineEmits<Emits>()

const contexts: Ref<K8sContext[]> = ref([])
const refreshing = ref(false)
const isDropdownOpen = ref(false)
const searchQuery = ref('')

// Filtered contexts based on search query
const filteredContexts = computed(() => {
  if (!searchQuery.value) {
    return contexts.value
  }
  
  const query = searchQuery.value.toLowerCase()
  return contexts.value.filter(context => 
    context.name.toLowerCase().includes(query) ||
    (context.cluster && context.cluster.toLowerCase().includes(query))
  )
})

onMounted(async () => {
  await loadContexts()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

async function loadContexts(): Promise<void> {
  try {
    refreshing.value = true
    const fetchedContexts = await invoke<K8sContext[]>('get_k8s_contexts')
    contexts.value = fetchedContexts
  } catch (error) {
    console.error('Failed to load contexts:', error)
    contexts.value = []
  } finally {
    refreshing.value = false
  }
}

async function refreshContexts(): Promise<void> {
  emit('refresh-contexts')
  await loadContexts()
  closeDropdown()
}

function selectContext(context: K8sContext): void {
  emit('context-selected', context)
}

function selectContextFromDropdown(context: K8sContext): void {
  selectContext(context)
  closeDropdown()
}

function toggleDropdown(): void {
  isDropdownOpen.value = !isDropdownOpen.value
  if (isDropdownOpen.value) {
    // Clear search when opening
    searchQuery.value = ''
  }
}

function closeDropdown(): void {
  isDropdownOpen.value = false
  searchQuery.value = ''
}

function handleClickOutside(event: Event): void {
  const target = event.target as HTMLElement
  const dropdownElement = target.closest('[data-dropdown]')
  
  if (!dropdownElement && isDropdownOpen.value) {
    closeDropdown()
  }
}

function getContextStatus(context: K8sContext): ContextStatus {
  return props.connectionStatus[context.name] || 'disconnected'
}

function truncateContextName(name: string): string {
  if (name.length <= 25) {
    return name
  }
  return name.substring(0, 25) + '...'
}
</script>

<style scoped>
/* Custom scrollbar for dropdown overflow */
.max-h-80::-webkit-scrollbar {
  width: 4px;
}

.max-h-80::-webkit-scrollbar-track {
  background: transparent;
}

.max-h-80::-webkit-scrollbar-thumb {
  background: var(--color-border-muted);
  border-radius: 2px;
}

.max-h-80::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-secondary);
}

/* Firefox scrollbar */
.max-h-80 {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.3) transparent;
}
</style>