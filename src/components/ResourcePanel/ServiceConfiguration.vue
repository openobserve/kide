<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">Service Configuration</h3>
    <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div>
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Type</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">{{ spec.type || 'ClusterIP' }}</dd>
      </div>
      <div v-if="spec.clusterIP">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Cluster IP</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 font-mono">{{ spec.clusterIP }}</dd>
      </div>
      <div v-if="spec.ports?.length">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Ports</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <div v-for="port in spec.ports" :key="port.port" class="font-mono text-xs">
            {{ port.port }}:{{ port.targetPort }}/{{ port.protocol || 'TCP' }}
            <span v-if="port.name" class="text-gray-500 dark:text-gray-400">({{ port.name }})</span>
          </div>
        </dd>
      </div>
      <div v-if="spec.selector">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Selector</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <div v-for="(value, key) in spec.selector" :key="key" class="text-xs">
            <span class="font-mono">{{ key }}: {{ value }}</span>
          </div>
        </dd>
      </div>
      <div v-if="spec.clusterIPs?.length">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Cluster IPs</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <div v-for="ip in spec.clusterIPs" :key="ip" class="font-mono text-xs">
            {{ ip }}
          </div>
        </dd>
      </div>
      <div v-if="spec.ipFamilies?.length">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">IP Families</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <div class="flex flex-wrap gap-1">
            <span v-for="family in spec.ipFamilies" :key="family" 
                  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300">
              {{ family }}
            </span>
          </div>
        </dd>
      </div>
      <div v-if="spec.ipFamilyPolicy">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">IP Family Policy</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
            {{ spec.ipFamilyPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.externalTrafficPolicy">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">External Traffic Policy</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <span :class="[
            'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
            spec.externalTrafficPolicy === 'Local' 
              ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300'
              : 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'
          ]">
            {{ spec.externalTrafficPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.internalTrafficPolicy">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Internal Traffic Policy</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300">
            {{ spec.internalTrafficPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.allocateLoadBalancerNodePorts !== undefined">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Allocate LoadBalancer NodePorts</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100">
          <span :class="[
            'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
            spec.allocateLoadBalancerNodePorts 
              ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
              : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
          ]">
            {{ spec.allocateLoadBalancerNodePorts ? 'Yes' : 'No' }}
          </span>
        </dd>
      </div>
      <div v-if="spec.healthCheckNodePort">
        <dt class="text-xs font-medium text-gray-500 dark:text-gray-400">Health Check NodePort</dt>
        <dd class="text-sm text-gray-900 dark:text-gray-100 font-mono">{{ spec.healthCheckNodePort }}</dd>
      </div>
    </dl>
  </div>
</template>

<script setup lang="ts">
interface Props {
  spec: {
    type?: string
    clusterIP?: string
    clusterIPs?: string[]
    ports?: Array<{
      port: number
      targetPort: number | string
      protocol?: string
      name?: string
    }>
    selector?: Record<string, string>
    ipFamilies?: string[]
    ipFamilyPolicy?: string
    externalTrafficPolicy?: string
    internalTrafficPolicy?: string
    allocateLoadBalancerNodePorts?: boolean
    healthCheckNodePort?: number
    loadBalancerClass?: string
    loadBalancerSourceRanges?: string[]
  }
}

defineProps<Props>()
</script>