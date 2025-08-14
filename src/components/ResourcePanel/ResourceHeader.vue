<template>
  <div class="flex-none px-6 py-4 border-b border-border-primary bg-surface-secondary">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <ResourceIcons :kind="resourceKind" class="w-6 h-6 text-accent-primary" />
        <div>
          <h2 class="text-lg font-semibold text-text-primary">{{ resourceData?.metadata?.name || `${resourceKind} Details` }}</h2>
          <p class="text-sm text-text-secondary">{{ getResourceSubtitle() }}</p>
        </div>
      </div>
      <button @click="$emit('close')" 
              class="p-2 rounded-md hover:bg-surface-tertiary transition-colors">
        <svg class="w-5 h-5 text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <!-- Status Badge -->
    <div class="mt-3 flex items-center space-x-2">
      <span :class="[
        'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium',
        getStatusColor()
      ]">
        <div :class="[
          'w-2 h-2 rounded-full mr-1.5',
          getStatusDotColor()
        ]"></div>
        {{ getResourceStatus() }}
      </span>
      <span class="text-xs text-text-muted">
        Created {{ formatTime(resourceData?.metadata?.creationTimestamp) }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import ResourceIcons from '../ResourceIcons.vue'
import { useResourceStatus } from '@/composables/useResourceStatus'

interface Props {
  resourceData: any | null
  resourceKind: string
}

const props = defineProps<Props>()

// Helper functions for accessing resource-specific fields
const { getGenericStatus, getGenericSpec, getStatusText } = useResourceStatus()

defineEmits<{
  close: []
}>()

function getResourceSubtitle(): string {
  if (props.resourceData?.metadata?.namespace) {
    return `${props.resourceData.metadata.namespace} namespace`
  }
  return `${props.resourceKind} resource`
}

function getResourceStatus(): string {
  if (!props.resourceData) return 'Unknown'
  
  // Use the centralized status logic for PersistentVolumeClaim and other resources
  if (props.resourceKind === 'PersistentVolumeClaim' || props.resourceKind === 'PersistentVolume' || props.resourceKind === 'Namespace') {
    return getStatusText(props.resourceData)
  }
  
  // Pod status
  if (props.resourceKind === 'Pod' && getGenericStatus(props.resourceData)?.phase) {
    return getGenericStatus(props.resourceData)?.phase
  }
  
  // Service status
  if (props.resourceKind === 'Service') {
    return 'Active'
  }
  
  // Deployment/ReplicaSet status
  if (['Deployment', 'ReplicaSet', 'StatefulSet'].includes(props.resourceKind)) {
    const desired = getGenericSpec(props.resourceData)?.replicas || 0
    const ready = getGenericStatus(props.resourceData)?.readyReplicas || 0
    return `${ready}/${desired} Ready`
  }
  
  // Default status check
  if (getGenericStatus(props.resourceData)?.conditions) {
    const readyCondition = getGenericStatus(props.resourceData).conditions.find((c: any) => c.type === 'Ready')
    if (readyCondition) {
      return readyCondition.status === 'True' ? 'Ready' : 'Not Ready'
    }
  }
  
  return 'Active'
}

function getStatusColor(): string {
  const status = getResourceStatus()
  
  if (props.resourceKind === 'Pod') {
    switch (status) {
      case 'Running': return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 border border-green-200 dark:border-green-700'
      case 'Pending': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300 border border-yellow-200 dark:border-yellow-700'
      case 'Failed': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300 border border-red-200 dark:border-red-700'
      case 'Succeeded': return 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 border border-blue-200 dark:border-blue-700'
      default: return 'bg-gray-100 dark:bg-gray-700/30 text-gray-800 dark:text-gray-300 border border-gray-200 dark:border-gray-600'
    }
  }
  
  if (status.includes('Ready') || status === 'Active' || status === 'Bound') {
    return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 border border-green-200 dark:border-green-700'
  }
  
  return 'bg-gray-100 dark:bg-gray-700/30 text-gray-800 dark:text-gray-300 border border-gray-200 dark:border-gray-600'
}

function getStatusDotColor(): string {
  const status = getResourceStatus()
  
  if (props.resourceKind === 'Pod') {
    switch (status) {
      case 'Running': return 'bg-status-success'
      case 'Pending': return 'bg-status-warning'
      case 'Failed': return 'bg-status-error'
      case 'Succeeded': return 'bg-status-info'
      default: return 'bg-status-secondary'
    }
  }
  
  if (status.includes('Ready') || status === 'Active' || status === 'Bound') {
    return 'bg-status-success'
  }
  
  return 'bg-gray-500'
}

function formatTime(timestamp: string): string {
  if (!timestamp) return 'Unknown'
  try {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    const seconds = Math.floor(diff / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)
    
    if (days > 0) return `${days}d ago`
    if (hours > 0) return `${hours}h ago`
    if (minutes > 0) return `${minutes}m ago`
    return `${seconds}s ago`
  } catch {
    return 'Invalid date'
  }
}
</script>