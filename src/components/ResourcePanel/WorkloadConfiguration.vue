<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">{{ resourceKind }} Configuration</h3>
    <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div v-if="spec?.replicas !== undefined">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Desired Replicas</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 flex items-center space-x-2">
          <span>{{ currentReplicas }}</span>
          <span v-if="isScaling" class="text-xs text-blue-600 dark:text-blue-400 animate-pulse">Scaling...</span>
          <!-- Scale controls for scalable resources -->
          <div v-if="isScalable" class="flex items-center space-x-1">
            <button
              @click="scaleDown"
              :disabled="currentReplicas <= 0 || isScaling"
              class="w-6 h-6 rounded bg-gray-200 dark:bg-gray-600 hover:bg-gray-300 dark:hover:bg-gray-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center text-xs font-bold transition-colors"
              title="Scale down"
            >
              −
            </button>
            <button
              @click="scaleUp"
              :disabled="isScaling"
              class="w-6 h-6 rounded bg-gray-200 dark:bg-gray-600 hover:bg-gray-300 dark:hover:bg-gray-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center text-xs font-bold transition-colors"
              title="Scale up"
            >
              +
            </button>
          </div>
        </dd>
      </div>
      <div v-if="status?.readyReplicas !== undefined">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Ready Replicas</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 flex items-center space-x-2">
          <span>{{ optimisticReadyReplicas }}</span>
          <span v-if="isScaling && optimisticReadyReplicas !== currentReplicas" class="text-xs text-orange-600 dark:text-orange-400">
            → {{ currentReplicas }}
          </span>
        </dd>
      </div>
      <div v-if="status?.availableReplicas !== undefined">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Available Replicas</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 flex items-center space-x-2">
          <span>{{ optimisticAvailableReplicas }}</span>
          <span v-if="isScaling && optimisticAvailableReplicas !== currentReplicas" class="text-xs text-orange-600 dark:text-orange-400">
            → {{ currentReplicas }}
          </span>
        </dd>
      </div>
      <div v-if="status?.updatedReplicas !== undefined">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Updated Replicas</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 flex items-center space-x-2">
          <span>{{ status.updatedReplicas }}</span>
          <span v-if="isScaling && status.updatedReplicas !== currentReplicas" class="text-xs text-orange-600 dark:text-orange-400">
            → {{ currentReplicas }}
          </span>
        </dd>
      </div>
      <div v-if="spec?.selector?.matchLabels">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Selector</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <div v-for="(value, key) in spec?.selector?.matchLabels" :key="key" class="text-xs">
            <span class="font-mono">{{ key }}: {{ value }}</span>
          </div>
        </dd>
      </div>
      <div v-if="spec?.updateStrategy || spec?.strategy">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Update Strategy</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          {{ (spec?.updateStrategy || spec?.strategy)?.type || 'RollingUpdate' }}
        </dd>
      </div>
    </dl>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTimeouts } from '@/composables/useTimeouts'
import { TIMEOUTS } from '@/constants/timeouts'

interface Props {
  resourceKind: string
  resourceName: string
  namespace?: string
  spec: {
    replicas?: number
    selector?: {
      matchLabels?: Record<string, string>
    }
    updateStrategy?: { type?: string }
    strategy?: { type?: string }
  }
  status?: {
    readyReplicas?: number
    availableReplicas?: number
    updatedReplicas?: number
  }
}

const props = defineProps<Props>()

// No need for helper functions since we receive spec/status directly

const emit = defineEmits<{
  scaled: [newReplicas: number]
}>()

// Composables
const { createTimeout } = useTimeouts()

const isScaling = ref(false)
let scalingTimeout: NodeJS.Timeout | null = null

// Local reactive replicas count for immediate UI feedback
const currentReplicas = ref(props.spec?.replicas || 0)

// Track the last scaling operation for optimistic updates
const lastScaledTo = ref<number | null>(null)
const scalingStartTime = ref<number | null>(null)

// Watch for changes in props.spec.replicas to sync with backend updates
watch(() => props.spec?.replicas, (newReplicas, oldReplicas) => {
  if (newReplicas !== undefined) {
    if (oldReplicas !== newReplicas) {
    }
    currentReplicas.value = newReplicas
  }
}, { immediate: true })

// Debug watcher for status changes
watch(() => props.status, (newStatus, oldStatus) => {
  if (newStatus && oldStatus) {
    const changes = []
    if (oldStatus.readyReplicas !== newStatus.readyReplicas) {
      changes.push(`ready: ${oldStatus.readyReplicas} → ${newStatus.readyReplicas}`)
    }
    if (oldStatus.availableReplicas !== newStatus.availableReplicas) {
      changes.push(`available: ${oldStatus.availableReplicas} → ${newStatus.availableReplicas}`)
    }
    if (oldStatus.updatedReplicas !== newStatus.updatedReplicas) {
      changes.push(`updated: ${oldStatus.updatedReplicas} → ${newStatus.updatedReplicas}`)
    }
    
    if (changes.length > 0) {
      
      // Check if scaling operation completed based on status changes
      if (isScaling.value && lastScaledTo.value !== null) {
        const target = lastScaledTo.value
        const isReadyComplete = newStatus.readyReplicas === target
        const isAvailableComplete = newStatus.availableReplicas === target
        
        
        // Complete scaling if both ready and available replicas match target
        if (isReadyComplete && isAvailableComplete) {
          isScaling.value = false
          lastScaledTo.value = null
          scalingStartTime.value = null
          
          // Clear any pending timeout since we detected completion
          if (scalingTimeout) {
            clearTimeout(scalingTimeout)
            scalingTimeout = null
          }
        }
      }
    }
  }
}, { deep: true })

// Enhanced scaling completion detection
// This watcher is a backup in case the deep status watcher above doesn't catch changes
watch(() => [props.status?.readyReplicas, props.status?.availableReplicas, currentReplicas.value], 
  ([readyReplicas, availableReplicas, desired]) => {
    if (isScaling.value && lastScaledTo.value !== null) {
      const target = lastScaledTo.value
      
      // Use the actual scaling target instead of current desired replicas
      // since currentReplicas is updated optimistically
      if (readyReplicas === target && availableReplicas === target) {
        isScaling.value = false
        lastScaledTo.value = null
        scalingStartTime.value = null
        
        // Clear any pending timeout since we detected completion
        if (scalingTimeout) {
          clearTimeout(scalingTimeout)
          scalingTimeout = null
        }
      }
    }
  }
)

// Check if the resource type is scalable (DaemonSets don't have replicas to scale)
const isScalable = computed(() => {
  return ['Deployment', 'StatefulSet', 'ReplicaSet'].includes(props.resourceKind)
})

// Optimistic status values - show expected values during scaling
const optimisticReadyReplicas = computed(() => {
  if (isScaling.value && lastScaledTo.value !== null && scalingStartTime.value) {
    const target = lastScaledTo.value
    const current = props.status?.readyReplicas || 0
    const elapsed = Date.now() - scalingStartTime.value
    
    // Handle both scaling up and down
    if (target !== current && elapsed > 1000) { // After 1 second, show movement toward target
      if (target > current) {
        // Scaling up - show gradual increase
        const progress = Math.min(1, elapsed / 5000) // Complete progress over 5 seconds
        return Math.floor(current + (target - current) * progress)
      } else if (target < current) {
        // Scaling down - show gradual decrease
        const progress = Math.min(1, elapsed / 3000) // Complete progress over 3 seconds  
        return Math.ceil(current - (current - target) * progress)
      }
    }
  }
  return props.status?.readyReplicas
})

const optimisticAvailableReplicas = computed(() => {
  if (isScaling.value && lastScaledTo.value !== null && scalingStartTime.value) {
    const target = lastScaledTo.value
    const current = props.status?.availableReplicas || 0
    const elapsed = Date.now() - scalingStartTime.value
    
    // Handle both scaling up and down
    if (target !== current && elapsed > 1500) { // Available updates a bit later
      if (target > current) {
        // Scaling up - show gradual increase
        const progress = Math.min(1, elapsed / 6000) // Complete progress over 6 seconds
        return Math.floor(current + (target - current) * progress)
      } else if (target < current) {
        // Scaling down - show gradual decrease
        const progress = Math.min(1, elapsed / 4000) // Complete progress over 4 seconds
        return Math.ceil(current - (current - target) * progress)
      }
    }
  }
  return props.status?.availableReplicas
})

async function scaleUp(): Promise<void> {
  if (isScaling.value) return
  
  const newReplicas = currentReplicas.value + 1
  await scaleResource(newReplicas)
}

async function scaleDown(): Promise<void> {
  if (currentReplicas.value <= 0 || isScaling.value) return
  
  const newReplicas = currentReplicas.value - 1
  await scaleResource(newReplicas)
}

async function scaleResource(newReplicas: number): Promise<void> {
  isScaling.value = true
  lastScaledTo.value = newReplicas
  scalingStartTime.value = Date.now()
  
  
  try {
    await invoke('scale_resource', {
      resourceName: props.resourceName,
      resourceKind: props.resourceKind,
      namespace: props.namespace || undefined,
      replicas: newReplicas
    })
    
    // Immediately update the local replicas count for instant UI feedback
    currentReplicas.value = newReplicas
    
    emit('scaled', newReplicas)
    
    // Keep the scaling indicator active while waiting for status updates
    // The status watchers above will detect completion when backend updates arrive
    // This timeout is a safety fallback in case status updates never arrive
    scalingTimeout = createTimeout(() => {
      console.warn('📊 Status fields may not have updated properly from backend watch events')
      isScaling.value = false
      lastScaledTo.value = null
      scalingStartTime.value = null
      scalingTimeout = null
    }, TIMEOUTS.SCALING_FALLBACK_TIMEOUT)
    
  } catch (error: any) {
    console.error(`❌ Failed to scale ${props.resourceKind}:`, error)
    // You might want to show a toast notification here
    alert(`Failed to scale ${props.resourceKind}: ${error.message || error}`)
    isScaling.value = false
    lastScaledTo.value = null
    scalingStartTime.value = null
  }
}
</script>