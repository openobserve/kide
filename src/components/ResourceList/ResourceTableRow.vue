<template>
  <tr
    @mouseenter="$emit('setHoveredRow', item.metadata?.uid, `table-row-${index}`)"
    @mouseleave="$emit('clearHoveredRow', `table-row-${index}`)"
    :class="[
      'hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors',
      selectedItem?.metadata?.uid === item.metadata?.uid ? 'bg-blue-50 dark:bg-blue-900/30' : '',
      selectedItems.has(item.metadata?.uid || '') ? 'bg-blue-50 dark:bg-blue-900/20' : ''
    ]"
  >
    <!-- Checkbox -->
    <td class="px-3 py-1 whitespace-nowrap">
      <input
        type="checkbox"
        :checked="selectedItems.has(item.metadata?.uid || '')"
        @change="$emit('toggleItemSelection', item)"
        @click.stop
        class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:bg-gray-700"
      >
    </td>
    
    <!-- Name -->
    <td class="px-3 py-1 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100 cursor-pointer relative" @click="$emit('selectItem', item)">
      <div class="flex items-center justify-between w-full">
        <span class="truncate">{{ item.metadata?.name || 'Unknown' }}</span>
        <!-- Hover buttons with expanded hover area -->
        <div 
          class="flex items-center space-x-1 transition-opacity duration-150 ml-2 relative z-10 p-1 -m-1"
          :class="hoveredRowId === item.metadata?.uid ? 'opacity-100' : 'opacity-0'"
          :style="{ pointerEvents: hoveredRowId === item.metadata?.uid ? 'auto' : 'none' }"
          @mouseenter="$emit('setHoveredRow', item.metadata?.uid, 'button-container')"
        >
          <!-- Pod-specific buttons -->
          <template v-if="resource?.kind === 'Pod'">
            <button
              @click.stop="$emit('openPodLogs', item)"
              class="btn-icon-primary w-6 h-6"
              title="View Pod Logs"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
              </svg>
            </button>
            <button
              @click.stop="$emit('openPodShell', item)"
              class="btn-icon-success w-6 h-6"
              title="Open Shell"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
              </svg>
            </button>
          </template>
          <!-- Delete button -->
          <button
            @click.stop="$emit('deleteResource', item)"
            class="btn-icon-danger w-6 h-6"
            :title="`Delete ${resource?.kind || 'Resource'}`"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
            </svg>
          </button>
        </div>
      </div>
    </td>
    
    <!-- Namespace (conditional) -->
    <td v-if="resource?.namespaced" class="px-3 py-1 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 cursor-pointer" @click="$emit('selectItem', item)">
      <span v-if="item.metadata?.namespace" class="px-1.5 py-0 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-xs rounded-full">
        {{ item.metadata.namespace }}
      </span>
      <span v-else class="text-gray-400 dark:text-gray-500 text-xs">-</span>
    </td>
    
    <!-- Pod-specific columns -->
    <template v-if="resource?.kind === 'Pod'">
      <!-- Containers -->
      <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
        <div class="flex gap-1 items-center">
          <!-- Init containers -->
          <div v-if="getGenericStatus(item)?.initContainerStatuses && getGenericStatus(item).initContainerStatuses.length > 0" 
               class="flex gap-0.5 pr-1 border-r border-gray-300">
            <div 
              v-for="(container, containerIndex) in getGenericStatus(item).initContainerStatuses" 
              :key="`init-${containerIndex}`"
              class="relative group"
            >
              <div 
                :class="[
                  'w-2.5 h-2.5 rounded-sm border border-gray-400 cursor-help',
                  getContainerStatusColor(container)
                ]"
              ></div>
              <!-- Tooltip -->
              <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-1 bg-gray-900 text-white text-xs rounded px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-0 pointer-events-none z-50">
                <div class="font-medium">Init: {{ container.name }}</div>
                <div class="text-gray-300">{{ getContainerStatusText(container) }}</div>
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 border-l-2 border-r-2 border-t-2 border-transparent border-t-gray-900"></div>
              </div>
            </div>
          </div>
          <!-- Regular containers -->
          <div class="flex gap-0.5">
            <div 
              v-for="(container, containerIndex) in (getGenericStatus(item)?.containerStatuses || [])" 
              :key="`container-${containerIndex}`"
              class="relative group"
            >
              <div 
                :class="[
                  'w-2.5 h-2.5 rounded-sm border border-gray-400 cursor-help',
                  getContainerStatusColor(container)
                ]"
              ></div>
              <!-- Tooltip -->
              <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-1 bg-gray-900 text-white text-xs rounded px-2 py-1 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-0 pointer-events-none z-50">
                <div class="font-medium">{{ container.name }}</div>
                <div class="text-gray-300">{{ getContainerStatusText(container) }}</div>
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 border-l-2 border-r-2 border-t-2 border-transparent border-t-gray-900"></div>
              </div>
            </div>
          </div>
          <!-- Fallback -->
          <span v-if="!getGenericStatus(item)?.containerStatuses && !getGenericStatus(item)?.initContainerStatuses" 
                class="text-gray-400 text-xs">-</span>
        </div>
      </td>
      
      <!-- Restarts -->
      <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
        <span class="text-xs text-gray-700 dark:text-gray-300">
          {{ getTotalRestartCount(item) }}
        </span>
      </td>
      
      <!-- Controlled By -->
      <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
        <span v-if="getControlledBy(item)" class="text-xs text-gray-700 dark:text-gray-300">
          {{ getControlledBy(item) }}
        </span>
        <span v-else class="text-gray-400 dark:text-gray-500 text-xs">-</span>
      </td>
      
      <!-- Node -->
      <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
        <span v-if="getGenericSpec(item)?.nodeName" class="text-xs text-gray-700 dark:text-gray-300">
          {{ getGenericSpec(item).nodeName }}
        </span>
        <span v-else class="text-gray-400 dark:text-gray-500 text-xs">-</span>
      </td>
      
      <!-- QoS -->
      <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
        <span :class="[
          'px-1.5 py-0 text-xs font-medium rounded-full',
          getQoSClass(item) === 'Guaranteed' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
          getQoSClass(item) === 'Burstable' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300' :
          getQoSClass(item) === 'BestEffort' ? 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300' :
          'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
        ]">
          {{ getQoSClass(item) }}
        </span>
      </td>
    </template>
    
    <!-- Status -->
    <td class="px-3 py-1 whitespace-nowrap text-sm cursor-pointer" @click="$emit('selectItem', item)">
      <span :class="[
        'px-1.5 py-0 text-xs font-medium rounded-full',
        getStatusClass(item)
      ]">
        {{ getStatusText(item) }}
      </span>
    </td>
    
    <!-- Age -->
    <td class="px-3 py-1 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400 cursor-pointer" @click="$emit('selectItem', item)">
      {{ getAge(item.metadata?.creationTimestamp) }}
    </td>
  </tr>
</template>

<script setup lang="ts">
import { useResourceStatus } from '@/composables/useResourceStatus'

// Helper functions for accessing resource-specific fields
const { getGenericStatus, getGenericSpec } = useResourceStatus()
interface Props {
  item: any
  index: number
  resource?: {
    kind: string
    namespaced: boolean
  } | null
  selectedItems: Set<string>
  selectedItem?: any | null
  hoveredRowId?: string | null
  getContainerStatusColor: (container: any) => string
  getContainerStatusText: (container: any) => string
  getTotalRestartCount: (item: any) => number
  getControlledBy: (item: any) => string | null
  getQoSClass: (item: any) => string
  getStatusClass: (item: any) => string
  getStatusText: (item: any) => string
  getAge: (timestamp?: string) => string
}

defineProps<Props>()

defineEmits<{
  setHoveredRow: [uid: string | undefined, source: string]
  clearHoveredRow: [source: string]
  toggleItemSelection: [item: any]
  selectItem: [item: any]
  openPodLogs: [item: any]
  openPodShell: [item: any]
  deleteResource: [item: any]
}>()
</script>