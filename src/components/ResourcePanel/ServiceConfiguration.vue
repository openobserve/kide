<template>
  <div class="elevated-surface rounded-lg p-4">
    <h3 class="text-sm font-semibold text-text-primary mb-3">Service Configuration</h3>
    <dl class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <div>
        <dt class="text-xs font-medium text-text-secondary">Type</dt>
        <dd class="text-sm text-text-primary">
          <span class="status-badge status-badge-info">
            {{ spec.type || 'ClusterIP' }}
          </span>
        </dd>
      </div>
      <div v-if="spec.clusterIP">
        <dt class="text-xs font-medium text-text-secondary">Cluster IP</dt>
        <dd class="text-sm text-text-primary font-mono">{{ spec.clusterIP }}</dd>
      </div>
      <div v-if="spec.ports?.length">
        <dt class="text-xs font-medium text-text-secondary">Ports</dt>
        <dd class="text-sm text-text-primary">
          <div v-for="port in spec.ports" :key="port.port" class="mb-1">
            <span class="font-mono text-sm bg-surface-secondary px-2 py-1 rounded border border-border-primary">
              {{ port.port }}:{{ port.targetPort }}/{{ port.protocol || 'TCP' }}
            </span>
            <span v-if="port.name" class="text-text-secondary text-xs ml-2">({{ port.name }})</span>
            <span v-if="port.appProtocol" class="text-text-secondary text-xs ml-1">[{{ port.appProtocol }}]</span>
          </div>
        </dd>
      </div>
      <div v-if="spec.selector">
        <dt class="text-xs font-medium text-text-secondary">Selector</dt>
        <dd class="text-sm text-text-primary">
          <div class="flex flex-wrap gap-2">
            <div v-for="(value, key) in spec.selector" :key="key" class="inline-flex items-center group">
              <span class="status-badge status-badge-info rounded-l-full border-r-0">
                {{ key }}
              </span>
              <span class="status-badge status-badge-info rounded-r-full border-l-0 opacity-80">
                {{ value }}
              </span>
            </div>
          </div>
        </dd>
      </div>
      <div v-if="spec.clusterIPs?.length">
        <dt class="text-xs font-medium text-text-secondary">Cluster IPs</dt>
        <dd class="text-sm text-text-primary">
          <div class="space-y-1">
            <div v-for="ip in spec.clusterIPs" :key="ip" class="font-mono text-sm">
              {{ ip }}
            </div>
          </div>
        </dd>
      </div>
      <div v-if="spec.ipFamilies?.length">
        <dt class="text-xs font-medium text-text-secondary">IP Families</dt>
        <dd class="text-sm text-text-primary">
          <div class="flex flex-wrap gap-1">
            <span v-for="family in spec.ipFamilies" :key="family" 
                  class="status-badge status-badge-info">
              {{ family }}
            </span>
          </div>
        </dd>
      </div>
      <div v-if="spec.ipFamilyPolicy">
        <dt class="text-xs font-medium text-text-secondary">IP Family Policy</dt>
        <dd class="text-sm text-text-primary">
          <span class="status-badge status-badge-success">
            {{ spec.ipFamilyPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.externalTrafficPolicy">
        <dt class="text-xs font-medium text-text-secondary">External Traffic Policy</dt>
        <dd class="text-sm text-text-primary">
          <span :class="[
            'status-badge',
            spec.externalTrafficPolicy === 'Local' 
              ? 'status-badge-warning'
              : 'status-badge-info'
          ]">
            {{ spec.externalTrafficPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.internalTrafficPolicy">
        <dt class="text-xs font-medium text-text-secondary">Internal Traffic Policy</dt>
        <dd class="text-sm text-text-primary">
          <span class="status-badge status-badge-secondary">
            {{ spec.internalTrafficPolicy }}
          </span>
        </dd>
      </div>
      <div v-if="spec.allocateLoadBalancerNodePorts !== undefined">
        <dt class="text-xs font-medium text-text-secondary">Allocate LoadBalancer NodePorts</dt>
        <dd class="text-sm text-text-primary">
          <span :class="[
            'status-badge',
            spec.allocateLoadBalancerNodePorts 
              ? 'status-badge-success'
              : 'status-badge-error'
          ]">
            {{ spec.allocateLoadBalancerNodePorts ? 'Yes' : 'No' }}
          </span>
        </dd>
      </div>
      <div v-if="spec.healthCheckNodePort">
        <dt class="text-xs font-medium text-text-secondary">Health Check NodePort</dt>
        <dd class="text-sm text-text-primary font-mono">{{ spec.healthCheckNodePort }}</dd>
      </div>
      
      <!-- Session Affinity -->
      <div v-if="spec.sessionAffinity">
        <dt class="text-xs font-medium text-text-secondary">Session Affinity</dt>
        <dd class="text-sm text-text-primary">
          <span :class="[
            'status-badge',
            spec.sessionAffinity === 'ClientIP' 
              ? 'status-badge-warning'
              : 'status-badge-secondary'
          ]">
            {{ spec.sessionAffinity }}
          </span>
          <div v-if="spec.sessionAffinity === 'ClientIP' && spec.sessionAffinityConfig?.clientIP?.timeoutSeconds" class="text-xs text-text-secondary mt-1">
            Timeout: {{ spec.sessionAffinityConfig.clientIP.timeoutSeconds }}s
          </div>
        </dd>
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
      appProtocol?: string
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
    sessionAffinity?: string
    sessionAffinityConfig?: {
      clientIP?: {
        timeoutSeconds?: number
      }
    }
  }
}

defineProps<Props>()
</script>