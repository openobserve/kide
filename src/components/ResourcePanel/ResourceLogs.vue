<template>
  <div class="h-full flex flex-col">
    <div class="flex-none px-6 py-2 border-b border-gray-200 bg-gray-50 dark:bg-gray-700 dark:border-gray-600">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="relative">
            <select v-model="selectedContainer" class="form-select flat-select text-sm pr-6">
            <!-- Init Containers -->
            <optgroup v-if="initContainers?.length" label="Init Containers">
              <option v-for="container in initContainers" :key="`init-${container.name}`" :value="`init-${container.name}`">
                {{ container.name }} (init)
              </option>
            </optgroup>
            <!-- Regular Containers -->
            <optgroup v-if="containers?.length" label="Containers">
              <option v-for="container in containers" :key="container.name" :value="container.name">
                {{ container.name }}
              </option>
            </optgroup>
            </select>
            <!-- Custom dropdown arrow -->
            <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
              <svg class="w-3 h-3 text-gray-400 dark:text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
              </svg>
            </div>
          </div>
          <button @click="$emit('refresh-logs')" 
                  class="btn-primary text-sm px-3 py-1">
            Refresh
          </button>
          <button @click="$emit('toggle-live-logging')" 
                  :class="[
                    'text-sm px-3 py-1 rounded transition-colors',
                    isLiveLogging 
                      ? 'btn-danger' 
                      : 'bg-green-600 text-white hover:bg-green-700'
                  ]">
            {{ isLiveLogging ? 'Stop Live' : 'Start Live' }}
          </button>
          
          <!-- Search functionality -->
          <div class="flex items-center space-x-2 border-l pl-3 border-gray-300 dark:border-gray-600">
            <div class="relative">
              <input
                v-model="searchQuery"
                type="text"
                placeholder="Search logs..."
                class="form-input text-sm w-64 pr-8"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                @keydown.enter.exact="searchNext"
                @keydown.enter.shift="searchPrevious"
                @keydown.escape="clearSearch"
              >
              <div class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400 dark:text-gray-500">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                </svg>
              </div>
            </div>
            <div v-if="searchQuery && searchManager.hasMatches" class="flex items-center space-x-1 text-xs text-gray-600 dark:text-gray-400">
              <span>{{ searchManager.currentIndex + 1 }} of {{ searchManager.matches.length }}</span>
              <button 
                @click="searchPrevious"
                :disabled="!searchManager.hasMatches"
                class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded disabled:opacity-50"
                title="Previous match (Shift+Enter)"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/>
                </svg>
              </button>
              <button 
                @click="searchNext"
                :disabled="!searchManager.hasMatches"
                class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded disabled:opacity-50"
                title="Next match (Enter)"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
        <div v-if="isLiveLogging" class="flex items-center text-sm text-green-500">
          <div class="status-indicator-success mr-2 animate-pulse"></div>
          Live
        </div>
      </div>
    </div>
    <div ref="logContainer" class="flex-1 overflow-auto bg-black text-green-400 font-mono p-4 leading-relaxed log-lines" style="font-size: 11px;">
      <div 
        v-for="(line, index) in logLines" 
        :key="index" 
        :ref="(el) => setLineRef(index, el as Element)"
        class="whitespace-pre-wrap"
        :class="{ 'bg-yellow-900/10': isMatchingLine(index) }"
      >
        <span v-if="extractTimestamp(line)" class="text-gray-400">{{ extractTimestamp(line) }}</span>
        <span v-if="extractTimestamp(line)" v-html="highlightSearchInText(extractLogContent(line), index)"></span>
        <span v-else v-html="highlightSearchInText(line, index)"></span>
      </div>
      <div v-if="logLines.length === 0" class="text-gray-500 italic">No logs available</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { LogSearchManager, extractTimestamp, extractLogContent } from '@/utils/logUtils'

interface Props {
  containers?: any[]
  initContainers?: any[]
  logLines: string[]
  isLiveLogging: boolean
  podName?: string
  namespace?: string
  initialContainer?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'refresh-logs': []
  'toggle-live-logging': []
  'container-changed': [container: string]
}>()

const selectedContainer = ref('')

// Search functionality
const searchQuery = ref('')
const searchManager = new LogSearchManager()
const logContainer = ref<HTMLElement | null>(null)
const lineRefs = ref<Map<number, HTMLElement>>(new Map())


// Initialize selected container
watch([() => props.containers, () => props.initContainers, () => props.initialContainer], ([containers, initContainers, initialContainer]) => {
  // Use initialContainer if provided, otherwise default selection
  if (initialContainer) {
    selectedContainer.value = initialContainer
  } else if (containers?.length) {
    selectedContainer.value = containers[0].name
  } else if (initContainers?.length) {
    selectedContainer.value = `init-${initContainers[0].name}`
  } else {
    selectedContainer.value = ''
  }
}, { immediate: true })

watch(selectedContainer, (value) => {
  emit('container-changed', value)
})

// Using utility functions for timestamp extraction
// (extractTimestamp and extractLogContent imported from utils)

// Search functionality
function setLineRef(index: number, el: Element | null): void {
  if (el) {
    lineRefs.value.set(index, el as HTMLElement)
  } else {
    lineRefs.value.delete(index)
  }
}

function updateSearchMatches(): void {
  searchManager.updateSearchMatches(props.logLines, searchQuery.value)
}

function highlightSearchInText(text: string, lineIndex: number): string {
  return searchManager.highlightSearchInText(text, lineIndex, searchQuery.value)
}

function isMatchingLine(lineIndex: number): boolean {
  return searchManager.isMatchingLine(lineIndex)
}

function searchNext(): void {
  if (!searchManager.hasMatches) return
  
  searchManager.nextMatch()
  scrollToCurrentMatch()
}

function searchPrevious(): void {
  if (!searchManager.hasMatches) return
  
  searchManager.previousMatch()
  scrollToCurrentMatch()
}

function scrollToCurrentMatch(): void {
  const currentMatch = searchManager.getCurrentMatch()
  if (!currentMatch) return
  
  const lineElement = lineRefs.value.get(currentMatch.lineIndex)
  if (lineElement && logContainer.value) {
    lineElement.scrollIntoView({ 
      behavior: 'smooth', 
      block: 'center' 
    })
  }
}

function clearSearch(): void {
  searchQuery.value = ''
  searchManager.clearMatches()
}

// Utility functions moved to logUtils
// (escapeHtml and escapeRegExp now in LogSearchManager)

// Watch for search query changes
watch(searchQuery, () => {
  updateSearchMatches()
}, { immediate: true })

// Watch for log lines changes to update search
watch(() => props.logLines, () => {
  updateSearchMatches()
}, { immediate: true })
</script>

<style scoped>
.flat-select {
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  background-image: none;
  box-shadow: none;
}

.flat-select::-ms-expand {
  display: none;
}
</style>