<template>
  <div class="relative">
    <div class="relative">
      <div
        @click="toggleDropdown"
        class="relative w-full pl-3 pr-10 py-2 text-left input-background rounded-md cursor-pointer focus:outline-none sm:text-sm"
        :class="{ 'input-focus': isOpen }"
      >
        <span class="block truncate">
          <span v-if="selectedNamespaces.length === 0" class="text-text-muted">
            Select namespaces
          </span>
          <span v-else-if="isAllNamespacesSelected" class="font-medium text-accent-primary">
            All namespaces
          </span>
          <span v-else-if="selectedNamespaces.length === 1" class="font-medium text-text-primary">
            {{ selectedNamespaces[0] }}
          </span>
          <span v-else class="font-medium text-text-primary">
            {{ selectedNamespaces.length }} namespaces selected
          </span>
        </span>
        <span class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
          <svg
            class="w-5 h-5 text-text-muted transition-transform duration-200"
            :class="{ 'rotate-180': isOpen }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </span>
      </div>

      <!-- Dropdown panel -->
      <div
        v-show="isOpen"
        ref="dropdownPanel"
        data-testid="dropdown-panel"
        role="listbox"
        aria-label="Namespace selection"
        class="absolute z-50 mt-1 dropdown-background max-h-60 py-1 text-base focus:outline-none sm:text-sm"
        :style="{ minWidth: dropdownWidth, left: dropdownLeft, right: dropdownRight }"
      >
        <!-- Search input -->
        <div class="px-3 py-2 border-b border-border-primary">
          <input
            ref="searchInput"
            v-model="searchQuery"
            @keydown="handleKeydown"
            type="text"
            placeholder="Search namespaces..."
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            role="combobox"
            aria-autocomplete="list"
            aria-expanded="true"
            :aria-activedescendant="highlightedIndex >= 0 ? `namespace-option-${filteredNamespaces[highlightedIndex]}` : (highlightedIndex === -1 && !searchQuery ? 'all-namespaces-option' : '')"
            class="w-full px-2 py-1 text-sm input-background rounded focus:input-focus text-text-primary placeholder:text-text-muted"
          >
        </div>

        <!-- All namespaces special option -->
        <div v-if="!searchQuery" class="border-b border-border-primary">
          <div
            @click="selectAllNamespaces"
            :ref="(el) => setOptionRef(el as HTMLElement, -1)"
            id="all-namespaces-option"
            data-testid="all-namespaces-option"
            role="option"
            :aria-selected="isAllNamespacesSelected"
            class="cursor-pointer select-none relative py-2 pl-3 pr-9 hover:bg-surface-secondary border-b border-border-primary"
            :class="{ 'selected-state': highlightedIndex === -1 }"
          >
            <div class="flex items-center">
              <input
                type="radio"
                :checked="isAllNamespacesSelected"
                @click.stop
                @change="selectAllNamespaces"
                class="h-4 w-4 text-accent-primary focus:ring-accent-primary input-background"
                name="namespace-selection"
              >
              <span class="ml-3 block font-medium text-accent-primary">
                All namespaces
              </span>
              <svg v-if="isAllNamespacesSelected" class="ml-auto w-4 h-4 text-accent-primary" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>


        <!-- Namespace options -->
        <div class="max-h-40 overflow-auto">
          <div
            v-for="(namespace, index) in filteredNamespaces"
            :key="namespace"
            @click="selectNamespace(namespace)"
            :ref="(el) => setOptionRef(el as HTMLElement, index)"
            :id="`namespace-option-${namespace}`"
            :data-testid="`namespace-option-${namespace}`"
            role="option"
            :aria-selected="selectedNamespaces.includes(namespace)"
            class="cursor-pointer select-none relative py-2 pl-3 pr-9 hover:bg-surface-secondary"
            :class="{ 'selected-state': highlightedIndex === index }"
          >
            <div class="flex items-center">
              <input
                type="radio"
                :checked="selectedNamespaces.includes(namespace)"
                @click.stop
                @change="selectNamespace(namespace)"
                class="h-4 w-4 text-accent-primary focus:ring-accent-primary input-background"
                name="namespace-selection"
              >
              <span class="ml-3 block font-normal truncate text-text-primary">
                {{ namespace }}
              </span>
              <svg v-if="selectedNamespaces.includes(namespace)" class="ml-auto w-4 h-4 text-accent-primary" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>

        <!-- No namespaces message -->
        <div v-if="filteredNamespaces.length === 0 && searchQuery" class="px-3 py-2 text-text-muted text-sm">
          No namespaces found matching "{{ searchQuery }}"
        </div>
        <div v-else-if="namespaces.length === 0" class="px-3 py-2 text-text-muted text-sm">
          No namespaces available
        </div>
      </div>
    </div>

    <!-- Click outside to close -->
    <div v-if="isOpen" @click="closeDropdown" class="fixed inset-0 z-40"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, type Ref } from 'vue'

interface Props {
  namespaces: string[]
  selectedNamespaces: string[]
}

const props = withDefaults(defineProps<Props>(), {
  namespaces: () => [],
  selectedNamespaces: () => []
})

const emit = defineEmits<{
  'update:selectedNamespaces': [namespaces: string[]]
}>()

const isOpen = ref(false)
const searchQuery = ref('')
const highlightedIndex = ref(-1)
const searchInput: Ref<HTMLInputElement | null> = ref(null)
const dropdownPanel: Ref<HTMLDivElement | null> = ref(null)
const optionRefs: Ref<HTMLElement[]> = ref([])
const dropdownWidth = ref('100%')
const dropdownLeft = ref('0')
const dropdownRight = ref('auto')

// Computed property to filter namespaces based on search query
const filteredNamespaces = computed(() => {
  if (!searchQuery.value) {
    return props.namespaces
  }
  return props.namespaces.filter(namespace =>
    namespace.toLowerCase().includes(searchQuery.value.toLowerCase())
  )
})

// Computed property to check if all namespaces are selected
const isAllNamespacesSelected = computed(() => {
  if (props.namespaces.length === 0) return false
  return props.selectedNamespaces.length === props.namespaces.length &&
         props.namespaces.every(ns => props.selectedNamespaces.includes(ns))
})

// Calculate optimal dropdown width and positioning
function calculateDropdownDimensions(): void {
  if (!props.namespaces.length) {
    dropdownWidth.value = '100%'
    dropdownLeft.value = '0'
    dropdownRight.value = 'auto'
    return
  }

  // Find the longest namespace name
  const longestNamespace = props.namespaces.reduce((longest, current) => 
    current.length > longest.length ? current : longest, ''
  )

  // Estimate character width (approximately 8px per character for 14px font)
  const estimatedTextWidth = longestNamespace.length * 8
  
  // Add padding for checkbox (20px) + spacing (12px) + padding (24px) + scrollbar (20px) + buffer (20px)
  const totalWidth = estimatedTextWidth + 96
  
  // Ensure minimum width matches the trigger button and reasonable maximum
  const triggerWidth = 220 // Approximate trigger button width
  const optimalWidth = Math.max(triggerWidth, Math.min(totalWidth, 500))
  
  dropdownWidth.value = `${optimalWidth}px`
  
  // Calculate positioning to keep dropdown in viewport
  calculateDropdownPosition(optimalWidth)
}

// Calculate dropdown position to keep it within viewport
function calculateDropdownPosition(dropdownWidthPx: number): void {
  // Get the trigger element (the parent container)
  const triggerElement = dropdownPanel.value?.parentElement
  if (!triggerElement) {
    dropdownLeft.value = '0'
    dropdownRight.value = 'auto'
    return
  }

  // Get trigger element's position relative to viewport
  const triggerRect = triggerElement.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const margin = 20 // Margin from viewport edge
  
  // Calculate if dropdown would overflow on the right
  const dropdownEndPosition = triggerRect.left + dropdownWidthPx
  const wouldOverflow = dropdownEndPosition > (viewportWidth - margin)
  
  if (wouldOverflow) {
    // Calculate how much we need to shift left to fit in viewport
    const overflowAmount = dropdownEndPosition - (viewportWidth - margin)
    
    // Check if we can shift left without going past the left edge
    const maxLeftShift = triggerRect.left - margin
    
    if (maxLeftShift >= overflowAmount) {
      // We can shift left enough to fit within viewport
      dropdownLeft.value = `-${overflowAmount}px`
      dropdownRight.value = 'auto'
    } else {
      // Can't shift left enough, align to right edge of viewport
      dropdownLeft.value = 'auto'
      dropdownRight.value = `${margin}px`
    }
  } else {
    // No overflow - use default left positioning
    dropdownLeft.value = '0'
    dropdownRight.value = 'auto'
  }
}

function selectNamespace(namespace: string): void {
  // Single selection - replace current selection
  emit('update:selectedNamespaces', [namespace])
  closeDropdown()
}

function removeNamespace(namespace: string): void {
  const newSelection = props.selectedNamespaces.filter(ns => ns !== namespace)
  emit('update:selectedNamespaces', newSelection)
}

function selectAllNamespaces(): void {
  // Select all namespaces
  emit('update:selectedNamespaces', [...props.namespaces])
  closeDropdown()
}


function toggleDropdown(): void {
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    nextTick(() => {
      // Calculate optimal width and position after DOM is updated
      calculateDropdownDimensions()
      searchInput.value?.focus()
      resetHighlight()
    })
  } else {
    resetSearch()
  }
}

function closeDropdown(): void {
  isOpen.value = false
  resetSearch()
}

function resetSearch(): void {
  searchQuery.value = ''
  highlightedIndex.value = -1
}

function resetHighlight(): void {
  // Start with "All namespaces" if available, otherwise first namespace
  const hasAllOption = !searchQuery.value
  highlightedIndex.value = hasAllOption ? -1 : 0
}

// Reset highlight when search query changes
function resetHighlightForSearch(): void {
  // When searching, reset to -1 so next arrow key press starts at 0
  // This prevents auto-highlighting during typing
  highlightedIndex.value = -1
}

function setOptionRef(el: HTMLElement | null, index: number): void {
  if (el) {
    optionRefs.value[index] = el
  }
}

function handleKeydown(event: KeyboardEvent): void {
  // Handle all keyboard navigation and prevent browser interference
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      event.stopPropagation()
      navigateDown()
      break
    case 'ArrowUp':
      event.preventDefault()
      event.stopPropagation()
      navigateUp()
      break
    case 'Enter':
      event.preventDefault()
      event.stopPropagation()
      selectHighlighted()
      break
    case 'Tab':
      event.preventDefault()
      event.stopPropagation()
      selectHighlighted()
      break
    case 'Escape':
      event.preventDefault()
      event.stopPropagation()
      closeDropdown()
      break
    default:
      // Allow normal typing for search, but prevent any autocomplete interference
      if (event.key.length === 1) {
        // This is a character key, allow it for search
        // No preventDefault here as we want the character to be typed
        return
      }
      break
  }
}

function navigateDown(): void {
  const maxIndex = filteredNamespaces.value.length - 1
  const hasAllOption = !searchQuery.value
  const minIndex = hasAllOption ? -1 : 0
  
  // If no filtered results, do nothing
  if (filteredNamespaces.value.length === 0) {
    return
  }
  
  // If we're at the end, don't navigate further
  if (highlightedIndex.value >= maxIndex) {
    return
  }
  
  // If no item is highlighted yet, start at the beginning
  if (highlightedIndex.value < minIndex) {
    highlightedIndex.value = minIndex
  } else {
    highlightedIndex.value++
  }
  
  scrollToOption()
}

function navigateUp(): void {
  const hasAllOption = !searchQuery.value
  const minIndex = hasAllOption ? -1 : 0
  const maxIndex = filteredNamespaces.value.length - 1
  
  // If no filtered results, do nothing
  if (filteredNamespaces.value.length === 0) {
    return
  }
  
  // If we're at the beginning, don't navigate further
  if (highlightedIndex.value <= minIndex) {
    return
  }
  
  // If no item is highlighted yet, start at the end
  if (highlightedIndex.value > maxIndex || highlightedIndex.value === -1) {
    highlightedIndex.value = maxIndex
  } else {
    highlightedIndex.value--
  }
  
  scrollToOption()
}

function scrollToOption(): void {
  const option = optionRefs.value[highlightedIndex.value]
  if (option && typeof option.scrollIntoView === 'function') {
    option.scrollIntoView({ block: 'nearest' })
  }
}

function selectHighlighted(): void {
  if (highlightedIndex.value === -1 && !searchQuery.value) {
    // Select "All namespaces" option
    selectAllNamespaces()
  } else if (highlightedIndex.value >= 0 && highlightedIndex.value < filteredNamespaces.value.length) {
    // Select individual namespace
    const namespace = filteredNamespaces.value[highlightedIndex.value]
    selectNamespace(namespace)
  }
}

// Watch for changes in namespaces to recalculate dimensions
watch(() => props.namespaces, () => {
  if (isOpen.value) {
    nextTick(() => {
      calculateDropdownDimensions()
    })
  }
}, { deep: true })

// Watch for search query changes to reset highlighting
watch(searchQuery, () => {
  resetHighlightForSearch()
})

// Close dropdown when clicking outside
function handleClickOutside(event: Event): void {
  if (!(event.target as Element)?.closest('.relative')) {
    closeDropdown()
  }
}

// Handle window resize to recalculate dropdown position
function handleWindowResize(): void {
  if (isOpen.value) {
    nextTick(() => {
      calculateDropdownDimensions()
    })
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  window.addEventListener('resize', handleWindowResize)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  window.removeEventListener('resize', handleWindowResize)
})
</script>