<template>
  <div class="mt-3">
    <!-- Environment Variables -->
    <div v-if="env?.length">
      <dt class="text-xs font-medium text-text-secondary mb-2">Environment Variables</dt>
      <div class="bg-surface-tertiary rounded p-2">
        <div v-for="envVar in env" :key="envVar.name" 
             class="flex items-center justify-between py-1 text-xs border-b border-border-primary last:border-b-0">
          <div class="font-medium text-status-success font-mono break-all flex-shrink-0 pr-2">{{ envVar.name }}</div>
          <div class="text-text-primary break-all text-right flex-1 min-w-0">
            <span v-if="envVar.value">{{ envVar.value }}</span>
            <span v-else-if="envVar.valueFrom?.fieldRef" class="italic">
              fieldRef: {{ envVar.valueFrom.fieldRef.fieldPath }}
            </span>
            <span v-else-if="envVar.valueFrom?.secretKeyRef" class="italic">
              secret: {{ envVar.valueFrom.secretKeyRef.name }}.{{ envVar.valueFrom.secretKeyRef.key }}
            </span>
            <span v-else-if="envVar.valueFrom?.configMapKeyRef" class="italic">
              configMap: {{ envVar.valueFrom.configMapKeyRef.name }}.{{ envVar.valueFrom.configMapKeyRef.key }}
            </span>
            <span v-else-if="envVar.valueFrom?.resourceFieldRef" class="italic">
              resource: {{ envVar.valueFrom.resourceFieldRef.resource }}
            </span>
            <span v-else class="italic text-text-muted">
              (from reference)
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Environment From (EnvFrom) -->
    <div v-if="envFrom?.length" class="mt-3">
      <dt class="text-xs font-medium text-text-secondary mb-2">Environment From</dt>
      <div class="bg-surface-tertiary rounded p-2">
        <div v-for="envFromItem in envFrom" :key="envFromItem.configMapRef?.name || envFromItem.secretRef?.name" 
             class="text-xs py-1">
          <span v-if="envFromItem.configMapRef" class="font-mono text-accent-primary">
            ConfigMap: {{ envFromItem.configMapRef.name }}
            <span v-if="envFromItem.prefix" class="text-text-secondary">(prefix: {{ envFromItem.prefix }})</span>
          </span>
          <span v-else-if="envFromItem.secretRef" class="font-mono text-status-info">
            Secret: {{ envFromItem.secretRef.name }}
            <span v-if="envFromItem.prefix" class="text-text-secondary">(prefix: {{ envFromItem.prefix }})</span>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  env?: Array<{
    name: string
    value?: string
    valueFrom?: {
      fieldRef?: { fieldPath: string }
      secretKeyRef?: { name: string; key: string }
      configMapKeyRef?: { name: string; key: string }
      resourceFieldRef?: { resource: string }
    }
  }>
  envFrom?: Array<{
    prefix?: string
    configMapRef?: { name: string }
    secretRef?: { name: string }
  }>
}

defineProps<Props>()
</script>