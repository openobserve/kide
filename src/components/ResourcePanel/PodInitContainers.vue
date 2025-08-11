<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">Init Containers</h3>
    <div class="space-y-3">
      <div v-for="container in initContainers" :key="container.name"
           class="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-600 p-3 border-l-4 border-l-purple-500">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center space-x-2">
            <h4 class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ container.name }}</h4>
            <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300">
              INIT
            </span>
          </div>
          <span :class="[
            'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
            getInitContainerStatusColor(container.name)
          ]">
            {{ getInitContainerStatus(container.name) }}
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
            <dd class="text-gray-900 dark:text-gray-100">{{ getInitContainerRestartCount(container.name) }}</dd>
          </div>
          <div v-if="container.command?.length">
            <dt class="text-gray-500 dark:text-gray-400">Command</dt>
            <dd class="text-gray-900 dark:text-gray-100 font-mono text-xs">{{ container.command.join(' ') }}</dd>
          </div>
          <div v-if="container.args?.length">
            <dt class="text-gray-500 dark:text-gray-400">Args</dt>
            <dd class="text-gray-900 dark:text-gray-100 font-mono text-xs">{{ container.args.join(' ') }}</dd>
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
  initContainers: any[]
  initContainerStatuses?: any[]
}

const props = defineProps<Props>()

function getInitContainerStatusColor(containerName: string): string {
  const status = getInitContainerStatus(containerName)
  switch (status) {
    case 'Completed': return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
    case 'Running': return 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300'
    case 'Waiting': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300'
    case 'Terminated': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
    case 'Failed': return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
    default: return 'bg-gray-100 dark:bg-gray-900/30 text-gray-800 dark:text-gray-300'
  }
}

function getInitContainerStatus(containerName: string): string {
  const containerStatus = props.initContainerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  
  if (containerStatus?.state?.terminated) {
    // Check if it terminated successfully (exit code 0)
    if (containerStatus.state.terminated.exitCode === 0) {
      return 'Completed'
    } else {
      return 'Failed'
    }
  }
  if (containerStatus?.state?.running) return 'Running'
  if (containerStatus?.state?.waiting) return 'Waiting'
  return 'Unknown'
}

function getInitContainerRestartCount(containerName: string): number {
  const containerStatus = props.initContainerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  return containerStatus?.restartCount || 0
}
</script>