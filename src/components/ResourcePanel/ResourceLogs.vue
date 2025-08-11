<template>
  <div class="h-full flex flex-col">
    <div class="flex-none px-6 py-2 border-b border-gray-200 bg-gray-50 dark:bg-gray-700 dark:border-gray-600">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="relative">
            <select v-model="selectedContainer" class="flat-select text-sm border border-gray-300 dark:border-gray-600 rounded px-2 py-1 pr-6 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:bg-gray-50 dark:focus:bg-gray-700">
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
                  class="text-sm px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors">
            Refresh
          </button>
          <button @click="$emit('toggle-live-logging')" 
                  :class="[
                    'text-sm px-3 py-1 rounded transition-colors',
                    isLiveLogging 
                      ? 'bg-red-600 text-white hover:bg-red-700' 
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
                class="text-sm border border-gray-300 dark:border-gray-600 rounded px-2 py-1 w-40 pr-8 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400"
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
            <div v-if="searchQuery && searchMatches.length > 0" class="flex items-center space-x-1 text-xs text-gray-600 dark:text-gray-400">
              <span>{{ currentMatchIndex + 1 }} of {{ searchMatches.length }}</span>
              <button 
                @click="searchPrevious"
                :disabled="searchMatches.length === 0"
                class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded disabled:opacity-50"
                title="Previous match (Shift+Enter)"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/>
                </svg>
              </button>
              <button 
                @click="searchNext"
                :disabled="searchMatches.length === 0"
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
          <div class="w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse"></div>
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
const searchMatches = ref<{ lineIndex: number; matchIndex: number }[]>([])
const currentMatchIndex = ref(0)
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

// Function to extract timestamp from log line
const extractTimestamp = (line: string): string | null => {
  // Match ISO 8601 timestamp at the beginning of the line
  const timestampRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\s+/
  const match = line.match(timestampRegex)
  return match ? match[0] : null
}

// Function to extract log content after timestamp
const extractLogContent = (line: string): string => {
  const timestampRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\s+/
  return line.replace(timestampRegex, '')
}

// Search functionality
function setLineRef(index: number, el: Element | null): void {
  if (el) {
    lineRefs.value.set(index, el as HTMLElement)
  } else {
    lineRefs.value.delete(index)
  }
}

function updateSearchMatches(): void {
  searchMatches.value = []
  currentMatchIndex.value = 0
  
  if (!searchQuery.value.trim()) return
  
  const query = searchQuery.value.toLowerCase()
  
  props.logLines.forEach((line, lineIndex) => {
    const searchText = extractTimestamp(line) ? extractLogContent(line) : line
    const lowerLine = searchText.toLowerCase()
    let matchIndex = 0
    let startIndex = 0
    
    while ((startIndex = lowerLine.indexOf(query, startIndex)) !== -1) {
      searchMatches.value.push({ lineIndex, matchIndex })
      startIndex += query.length
      matchIndex++
    }
  })
}

function highlightSearchInText(text: string, lineIndex: number): string {
  if (!searchQuery.value.trim()) return escapeHtml(text)
  
  const query = searchQuery.value
  const regex = new RegExp(`(${escapeRegExp(query)})`, 'gi')
  const currentMatch = searchMatches.value[currentMatchIndex.value]
  
  let matchIndex = 0
  return escapeHtml(text).replace(regex, (match) => {
    const isCurrentMatch = currentMatch && 
      currentMatch.lineIndex === lineIndex && 
      currentMatch.matchIndex === matchIndex
    matchIndex++
    return isCurrentMatch 
      ? `<span style="background-color: #fbbf24; color: #000; font-weight: bold; padding: 0 2px; border-radius: 2px;">${match}</span>`
      : `<span style="background-color: #f59e0b; color: #000; padding: 0 2px; border-radius: 2px;">${match}</span>`
  })
}

function isMatchingLine(lineIndex: number): boolean {
  if (!searchQuery.value.trim()) return false
  return searchMatches.value.some(match => match.lineIndex === lineIndex)
}

function searchNext(): void {
  if (searchMatches.value.length === 0) return
  
  currentMatchIndex.value = (currentMatchIndex.value + 1) % searchMatches.value.length
  scrollToCurrentMatch()
}

function searchPrevious(): void {
  if (searchMatches.value.length === 0) return
  
  currentMatchIndex.value = currentMatchIndex.value === 0 
    ? searchMatches.value.length - 1 
    : currentMatchIndex.value - 1
  scrollToCurrentMatch()
}

function scrollToCurrentMatch(): void {
  const currentMatch = searchMatches.value[currentMatchIndex.value]
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
  searchMatches.value = []
  currentMatchIndex.value = 0
}

function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

function escapeRegExp(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

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