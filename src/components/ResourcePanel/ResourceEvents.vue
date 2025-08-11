<template>
  <div class="h-full overflow-y-auto p-6">
    <div class="space-y-3">
      <div v-for="event in events" :key="event.uid" 
           class="border border-gray-200 dark:border-gray-700 rounded-lg p-3 bg-gray-50 dark:bg-gray-700">
        <div class="flex items-start justify-between">
          <div class="flex items-start space-x-3">
            <div :class="[
              'w-2 h-2 rounded-full mt-2',
              event.type === 'Warning' ? 'bg-yellow-500' : 'bg-blue-500'
            ]"></div>
            <div>
              <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ event.reason }}</div>
              <div class="text-sm text-gray-700 dark:text-gray-300 mt-1">{{ event.message }}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {{ event.source?.component }} • {{ formatTime(event.lastTimestamp || event.firstTimestamp) }}
              </div>
            </div>
          </div>
          <span :class="[
            'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
            event.type === 'Warning' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300' : 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300'
          ]">
            {{ event.type }}
          </span>
        </div>
      </div>
      <div v-if="events.length === 0" class="text-center text-gray-500 dark:text-gray-400 italic py-8">
        No events found for this {{ resourceKind.toLowerCase() }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  events: Array<{
    uid: string
    type: string
    reason: string
    message: string
    source?: { component?: string }
    firstTimestamp?: string
    lastTimestamp?: string
  }>
  resourceKind: string
}

defineProps<Props>()

function formatTime(timestamp?: string): string {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return 'Invalid date'
  }
}
</script>