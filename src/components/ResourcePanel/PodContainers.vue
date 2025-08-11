<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">Containers</h3>
    <div class="space-y-3">
      <div v-for="container in containers" :key="container.name"
           class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-600 p-3">
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ container.name }}</h4>
          <span :class="[
            'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
            getContainerStatusColor(container.name)
          ]">
            {{ getContainerStatus(container.name) }}
          </span>
        </div>
        <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2 text-xs">
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Image</dt>
            <dd class="text-gray-900 dark:text-gray-100 font-mono break-all">{{ container.image }}</dd>
          </div>
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Pull Policy</dt>
            <dd class="text-gray-900 dark:text-gray-100">{{ container.imagePullPolicy || 'Always' }}</dd>
          </div>
          <div v-if="container.ports?.length">
            <dt class="text-gray-500 dark:text-gray-400">Ports</dt>
            <dd class="text-gray-900 dark:text-gray-100 font-mono">
              {{ container.ports.map((p: any) => `${p.containerPort}/${p.protocol || 'TCP'}`).join(', ') }}
            </dd>
          </div>
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Restart Count</dt>
            <dd class="text-gray-900 dark:text-gray-100">{{ getContainerRestartCount(container.name) }}</dd>
          </div>
        </dl>
        
        <!-- Environment Variables -->
        <ContainerEnvironment
          v-if="container.env?.length || container.envFrom?.length"
          :env="container.env"
          :envFrom="container.envFrom"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ContainerEnvironment from './ContainerEnvironment.vue'

interface Props {
  containers: any[]
  containerStatuses?: any[]
}

const props = defineProps<Props>()

function getContainerStatusColor(containerName: string): string {
  const status = getContainerStatus(containerName)
  switch (status) {
    case 'Running': return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
    case 'Waiting': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300'
    case 'Terminated': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
    default: return 'bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-300'
  }
}

function getContainerStatus(containerName: string): string {
  const containerStatus = props.containerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  if (containerStatus?.state?.running) return 'Running'
  if (containerStatus?.state?.waiting) return 'Waiting'
  if (containerStatus?.state?.terminated) return 'Terminated'
  return 'Unknown'
}

function getContainerRestartCount(containerName: string): number {
  const containerStatus = props.containerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  return containerStatus?.restartCount || 0
}
</script>