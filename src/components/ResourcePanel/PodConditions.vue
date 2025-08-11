<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">Conditions</h3>
    <div class="space-y-2">
      <div v-for="condition in sortedConditions" :key="condition.type"
           class="flex items-center justify-between py-2 px-3 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-600">
        <div class="flex items-center space-x-2">
          <div :class="[
            'w-2 h-2 rounded-full',
            condition.status === 'True' ? 'bg-green-500' : 'bg-red-500'
          ]"></div>
          <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ condition.type }}</span>
        </div>
        <span class="text-xs text-gray-500 dark:text-gray-400">{{ formatTime(condition.lastTransitionTime) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  conditions: Array<{
    type: string
    status: string
    lastTransitionTime?: string
  }>
}

const props = defineProps<Props>()

// Sort conditions by lastTransitionTime in reverse chronological order (newest first)
const sortedConditions = computed(() => {
  return [...props.conditions].sort((a, b) => {
    // Handle missing timestamps (put them at the end)
    if (!a.lastTransitionTime && !b.lastTransitionTime) return 0
    if (!a.lastTransitionTime) return 1
    if (!b.lastTransitionTime) return -1
    
    // Sort by timestamp (newest first)
    const timeA = new Date(a.lastTransitionTime).getTime()
    const timeB = new Date(b.lastTransitionTime).getTime()
    return timeB - timeA
  })
})

function formatTime(timestamp?: string): string {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return 'Invalid date'
  }
}
</script>