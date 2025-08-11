<template>
  <div 
    ref="navigationRef"
    class="bg-white dark:bg-gray-800 shadow-lg h-screen flex flex-col border-r border-gray-200 dark:border-gray-700 relative"
    :style="{ width: navigationWidth + 'px' }"
  >

    <!-- Navigation Tree -->
    <div class="flex-1 overflow-y-scroll bg-white dark:bg-gray-800">
      <div class="py-0.5">
        <div v-for="category in categories" :key="category.name" class="mb-0">
          <!-- Category Header -->
          <div :class="[
            'px-2 py-1 flex items-center justify-between group cursor-pointer transition-colors duration-150',
            getCategoryColorClasses(category.name)
          ]"
               @click="toggleCategory(category.name)"
               data-testid="category-header">
            <div class="flex items-center space-x-2">
              <svg class="w-4 h-4 transition-transform duration-200"
                   :class="[
                     { 'rotate-90': expandedCategories.has(category.name) },
                     getCategoryIconColor(category.name)
                   ]"
                   fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
              </svg>
              <h3 :class="[
                'text-xs font-semibold uppercase tracking-wide select-none',
                getCategoryTextColor(category.name)
              ]">
                {{ category.name }}
              </h3>
            </div>
            <span :class="['text-xs font-medium', getCategoryCountColor(category.name)]">{{ category.resources.length }}</span>
          </div>
          
          <!-- Category Resources -->
          <div v-show="expandedCategories.has(category.name)" class="ml-1 border-l border-gray-100 dark:border-gray-700">
            <div v-for="resource in category.resources" :key="resource.name" class="relative">
              <button
                @click="$emit('select-resource', resource)"
                data-testid="resource-item"
                :class="[
                  'w-full text-left pl-3 pr-2 py-1 text-sm transition-all duration-150 hover:bg-blue-50 dark:hover:bg-blue-900/30 border-l-2 border-transparent hover:border-blue-200 dark:hover:border-blue-700 group',
                  selectedResource?.name === resource.name
                    ? 'bg-blue-100 dark:bg-blue-900/50 border-blue-500 dark:border-blue-400 text-blue-900 dark:text-blue-200 shadow-sm'
                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
                ]"
                :title="resource.description"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center space-x-1">
                    <!-- Resource Icon -->
                    <div class="flex-shrink-0">
                      <ResourceIcons :kind="resource.kind" 
                                   class="w-3 h-3"
                                   :class="selectedResource?.name === resource.name ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400 dark:text-gray-500'" />
                    </div>
                    <span>{{ resource.name }}</span>
                  </div>
                  
                  <div class="flex items-center space-x-0.5">
                    <!-- Scope Badge -->
                    <span :class="[
                      'inline-flex items-center px-0.5 py-0 rounded text-xs',
                      resource.namespaced 
                        ? 'bg-blue-100 text-blue-700 border border-blue-200' 
                        : 'bg-purple-100 text-purple-700 border border-purple-200'
                    ]">
                      {{ resource.namespaced ? 'NS' : 'CL' }}
                    </span>
                    
                    <!-- API Version -->
                    <span class="text-xs text-gray-400 font-mono">
                      {{ getApiVersionShort(resource.apiVersion) }}
                    </span>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="border-t border-gray-200 bg-gray-50 px-4 py-3">
      <div class="flex items-center justify-between text-xs text-gray-500">
        <span>{{ getTotalResourcesCount() }} resources</span>
        <span>{{ getCurrentTime() }}</span>
      </div>
    </div>

    <!-- Resize Handle -->
    <div 
      ref="resizeHandleRef"
      class="absolute right-0 top-0 h-full w-1 cursor-col-resize hover:bg-blue-500 transition-colors border-r-2 border-transparent hover:border-blue-500 active:bg-blue-600 z-10"
      @mousedown="startResize"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useTimeouts } from '@/composables/useTimeouts'
import { TIMEOUTS } from '@/constants/timeouts'
import ResourceIcons from './ResourceIcons.vue'
import type { K8sResourceCategory, K8sResource, ConnectionStatus } from '@/types'

interface Props {
  categories: K8sResourceCategory[]
  selectedResource: K8sResource | null
  connected: boolean
  connectionStatus: ConnectionStatus
  currentContextName?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'select-resource': [resource: K8sResource]
}>()

// Composables
const { createInterval } = useTimeouts()

// Navigation resizing state
const DEFAULT_WIDTH = 256 // 80 * 0.8 = 320 * 0.8 = 256px (20% reduction)
const MIN_WIDTH = 200
const MAX_WIDTH = 500
const navigationWidth = ref(DEFAULT_WIDTH)
const navigationRef = ref<HTMLElement | null>(null)
const resizeHandleRef = ref<HTMLElement | null>(null)
const isResizing = ref(false)

// State for expanded categories (only "Commonly used" expanded by default)
const expandedCategories = ref(new Set<string>())
const currentTime = ref('')

// Initialize with only "Commonly used" expanded
onMounted(() => {
  expandCommonlyUsedOnly()
  updateTime()
  createInterval(updateTime, TIMEOUTS.CLOCK_UPDATE_INTERVAL)
  loadNavigationWidth()
})

// Watch for changes in categories and expand only "Commonly used"
watch(() => props.categories, (newCategories) => {
  if (newCategories && newCategories.length > 0) {
    expandCommonlyUsedOnly()
  }
}, { immediate: true })

function expandCommonlyUsedOnly() {
  // Clear all expanded categories first
  expandedCategories.value.clear()
  
  // Only expand "Commonly used" category
  props.categories.forEach(category => {
    if (category.name === 'Commonly used') {
      expandedCategories.value.add(category.name)
    }
  })
}

function expandAllCategories() {
  props.categories.forEach(category => {
    expandedCategories.value.add(category.name)
  })
}

function toggleCategory(categoryName: string) {
  if (expandedCategories.value.has(categoryName)) {
    expandedCategories.value.delete(categoryName)
  } else {
    expandedCategories.value.add(categoryName)
  }
}

function getTotalResourcesCount(): number {
  return props.categories.reduce((total, category) => total + category.resources.length, 0)
}

function updateTime() {
  currentTime.value = new Date().toLocaleTimeString('en-US', { 
    hour12: false, 
    hour: '2-digit', 
    minute: '2-digit'
  })
}

function getCurrentTime(): string {
  return currentTime.value
}

function getApiVersionShort(apiVersion: string): string {
  // Extract just the version part (e.g., "apps/v1" -> "v1")
  if (!apiVersion) {
    return 'v1' // Default fallback
  }
  const parts = apiVersion.split('/')
  return parts[parts.length - 1]
}

// Color mapping for different categories
const categoryColors = {
  'Commonly used': {
    background: 'bg-amber-50 dark:bg-amber-900/20 hover:bg-amber-100 dark:hover:bg-amber-900/30',
    text: 'text-amber-800 dark:text-amber-200',
    icon: 'text-amber-600 dark:text-amber-400',
    count: 'text-amber-700 dark:text-amber-300'
  },
  'Workloads': {
    background: 'bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/30',
    text: 'text-blue-700 dark:text-blue-300',
    icon: 'text-blue-500 dark:text-blue-400',
    count: 'text-blue-600 dark:text-blue-400'
  },
  'Services & Networking': {
    background: 'bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30',
    text: 'text-green-700 dark:text-green-300',
    icon: 'text-green-500 dark:text-green-400',
    count: 'text-green-600 dark:text-green-400'
  },
  'Configuration': {
    background: 'bg-purple-50 dark:bg-purple-900/20 hover:bg-purple-100 dark:hover:bg-purple-900/30',
    text: 'text-purple-700 dark:text-purple-300',
    icon: 'text-purple-500 dark:text-purple-400',
    count: 'text-purple-600 dark:text-purple-400'
  },
  'Storage': {
    background: 'bg-cyan-50 dark:bg-cyan-900/20 hover:bg-cyan-100 dark:hover:bg-cyan-900/30',
    text: 'text-cyan-700 dark:text-cyan-300',
    icon: 'text-cyan-500 dark:text-cyan-400',
    count: 'text-cyan-600 dark:text-cyan-400'
  },
  'Cluster Administration': {
    background: 'bg-orange-50 dark:bg-orange-900/20 hover:bg-orange-100 dark:hover:bg-orange-900/30',
    text: 'text-orange-700 dark:text-orange-300',
    icon: 'text-orange-500 dark:text-orange-400',
    count: 'text-orange-600 dark:text-orange-400'
  },
  'Security & Access Control': {
    background: 'bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/30',
    text: 'text-red-700 dark:text-red-300',
    icon: 'text-red-500 dark:text-red-400',
    count: 'text-red-600 dark:text-red-400'
  },
  'Scaling': {
    background: 'bg-indigo-50 dark:bg-indigo-900/20 hover:bg-indigo-100 dark:hover:bg-indigo-900/30',
    text: 'text-indigo-700 dark:text-indigo-300',
    icon: 'text-indigo-500 dark:text-indigo-400',
    count: 'text-indigo-600 dark:text-indigo-400'
  },
  'Custom Resources': {
    background: 'bg-teal-50 dark:bg-teal-900/20 hover:bg-teal-100 dark:hover:bg-teal-900/30',
    text: 'text-teal-700 dark:text-teal-300',
    icon: 'text-teal-500 dark:text-teal-400',
    count: 'text-teal-600 dark:text-teal-400'
  }
} as const

// Fallback colors for unknown categories
const fallbackColors = {
  background: 'bg-gray-50 dark:bg-gray-700/20 hover:bg-gray-100 dark:hover:bg-gray-700/30',
  text: 'text-gray-700 dark:text-gray-300',
  icon: 'text-gray-500 dark:text-gray-400',
  count: 'text-gray-600 dark:text-gray-400'
}

function getCategoryColorClasses(categoryName: string): string {
  const colors = categoryColors[categoryName as keyof typeof categoryColors] || fallbackColors
  return colors.background
}

function getCategoryTextColor(categoryName: string): string {
  const colors = categoryColors[categoryName as keyof typeof categoryColors] || fallbackColors
  return colors.text
}

function getCategoryIconColor(categoryName: string): string {
  const colors = categoryColors[categoryName as keyof typeof categoryColors] || fallbackColors
  return colors.icon
}

function getCategoryCountColor(categoryName: string): string {
  const colors = categoryColors[categoryName as keyof typeof categoryColors] || fallbackColors
  return colors.count
}

// Navigation width persistence
function loadNavigationWidth(): void {
  try {
    const saved = localStorage.getItem('kide-navigation-width')
    if (saved) {
      const width = parseInt(saved, 10)
      if (width >= MIN_WIDTH && width <= MAX_WIDTH) {
        navigationWidth.value = width
      }
    }
  } catch (error) {
    console.warn('Failed to load navigation width:', error)
  }
}

function saveNavigationWidth(): void {
  try {
    localStorage.setItem('kide-navigation-width', navigationWidth.value.toString())
  } catch (error) {
    console.warn('Failed to save navigation width:', error)
  }
}

// Resize functionality
function startResize(event: MouseEvent): void {
  event.preventDefault()
  isResizing.value = true
  
  const startX = event.clientX
  const startWidth = navigationWidth.value
  
  function handleMouseMove(e: MouseEvent): void {
    if (!isResizing.value) return
    
    const deltaX = e.clientX - startX
    const newWidth = startWidth + deltaX
    
    // Constrain width to min/max bounds
    navigationWidth.value = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, newWidth))
  }
  
  function handleMouseUp(): void {
    isResizing.value = false
    saveNavigationWidth()
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}
</script>

<style scoped>
/* Custom scrollbar*/
.overflow-y-scroll::-webkit-scrollbar {
  width: 8px;
}

/* Light mode scrollbar */
.overflow-y-scroll::-webkit-scrollbar-track {
  background: #f7f8fc;
}

.overflow-y-scroll::-webkit-scrollbar-thumb {
  background: #cbd5e0;
  border-radius: 4px;
}

.overflow-y-scroll::-webkit-scrollbar-thumb:hover {
  background: #a0aec0;
}

/* Dark mode scrollbar */
.dark .overflow-y-scroll::-webkit-scrollbar-track {
  background: #374151;
}

.dark .overflow-y-scroll::-webkit-scrollbar-thumb {
  background: #6b7280;
  border-radius: 4px;
}

.dark .overflow-y-scroll::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}

/* selection animation */
.group:hover .border-l-2 {
  transition: border-color 0.15s ease-in-out;
}

/* Default resource icon (temporary) */
.default-icon {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  background: currentColor;
  mask: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='currentColor'%3e%3cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10'/%3e%3c/svg%3e") no-repeat center;
  mask-size: contain;
}
</style>