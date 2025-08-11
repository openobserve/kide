<template>
  <thead class="bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600 sticky top-0 z-10 backdrop-blur-sm"
         @mouseenter="$emit('clearHoveredRow', 'header-hover')">
    <tr>
      <th class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-10">
        <input
          type="checkbox"
          :checked="selectedItems.size > 0 && selectedItems.size === filteredItemsCount"
          :indeterminate="selectedItems.size > 0 && selectedItems.size < filteredItemsCount"
          @change="$emit('toggleSelectAll')"
          class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:bg-gray-700"
        >
      </th>
      <th class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'name')">
        <div class="flex items-center space-x-1">
          <span>Name</span>
          <SortIcon :column="'name'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th v-if="resource?.namespaced" 
          class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'namespace')">
        <div class="flex items-center space-x-1">
          <span>Namespace</span>
          <SortIcon :column="'namespace'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th v-if="resource?.kind === 'Pod'" class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Containers</th>
      <th v-if="resource?.kind === 'Pod'" 
          class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'restarts')">
        <div class="flex items-center space-x-1">
          <span>Restarts</span>
          <SortIcon :column="'restarts'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th v-if="resource?.kind === 'Pod'" 
          class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'controlled_by')">
        <div class="flex items-center space-x-1">
          <span>Controlled By</span>
          <SortIcon :column="'controlled_by'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th v-if="resource?.kind === 'Pod'" 
          class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'node')">
        <div class="flex items-center space-x-1">
          <span>Node</span>
          <SortIcon :column="'node'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th v-if="resource?.kind === 'Pod'" 
          class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'qos')">
        <div class="flex items-center space-x-1">
          <span>QoS</span>
          <SortIcon :column="'qos'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'status')">
        <div class="flex items-center space-x-1">
          <span>Status</span>
          <SortIcon :column="'status'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
      <th class="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 select-none group"
          @click="$emit('sort', 'age')">
        <div class="flex items-center space-x-1">
          <span>Age</span>
          <SortIcon :column="'age'" :sortColumn="sortColumn" :sortDirection="sortDirection" />
        </div>
      </th>
    </tr>
  </thead>
</template>

<script setup lang="ts">
import { defineComponent } from 'vue'

interface Props {
  resource?: {
    kind: string
    namespaced: boolean
  } | null
  selectedItems: Set<string>
  filteredItemsCount: number
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  toggleSelectAll: []
  clearHoveredRow: [source: string]
  sort: [column: string]
}>()

// Sort Icon Component
const SortIcon = defineComponent({
  props: {
    column: { type: String, required: true },
    sortColumn: { type: String, required: true },
    sortDirection: { type: String as () => 'asc' | 'desc', required: true }
  },
  template: `
    <div class="w-3 h-3">
      <svg v-if="column === sortColumn && sortDirection === 'asc'" 
           class="w-3 h-3 text-gray-500" 
           fill="currentColor" 
           viewBox="0 0 20 20">
        <path fill-rule="evenodd" d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z" clip-rule="evenodd"/>
      </svg>
      <svg v-else-if="column === sortColumn && sortDirection === 'desc'" 
           class="w-3 h-3 text-gray-500" 
           fill="currentColor" 
           viewBox="0 0 20 20">
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
      </svg>
      <svg v-else 
           class="w-3 h-3 text-gray-300 opacity-0 group-hover:opacity-100" 
           fill="currentColor" 
           viewBox="0 0 20 20">
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
      </svg>
    </div>
  `
})
</script>