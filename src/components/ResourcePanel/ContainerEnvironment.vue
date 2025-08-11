<template>
  <div class="mt-3">
    <!-- Environment Variables -->
    <div v-if="env?.length">
      <dt class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">Environment Variables</dt>
      <div class="bg-gray-50 dark:bg-gray-700 rounded p-2">
        <div v-for="envVar in env" :key="envVar.name" 
             class="flex items-center justify-between py-1 text-xs border-b border-gray-200 dark:border-gray-600 last:border-b-0">
          <div class="font-medium text-blue-700 dark:text-blue-300 font-mono break-all flex-shrink-0 pr-2">{{ envVar.name }}</div>
          <div class="text-gray-600 dark:text-gray-300 break-all text-right flex-1 min-w-0">
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
            <span v-else class="italic text-gray-400 dark:text-gray-500">
              (from reference)
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Environment From (EnvFrom) -->
    <div v-if="envFrom?.length" class="mt-3">
      <dt class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">Environment From</dt>
      <div class="bg-gray-50 dark:bg-gray-700 rounded p-2">
        <div v-for="envFromItem in envFrom" :key="envFromItem.configMapRef?.name || envFromItem.secretRef?.name" 
             class="text-xs py-1">
          <span v-if="envFromItem.configMapRef" class="font-mono text-blue-700 dark:text-blue-300">
            ConfigMap: {{ envFromItem.configMapRef.name }}
            <span v-if="envFromItem.prefix" class="text-gray-500 dark:text-gray-400">(prefix: {{ envFromItem.prefix }})</span>
          </span>
          <span v-else-if="envFromItem.secretRef" class="font-mono text-purple-700 dark:text-purple-300">
            Secret: {{ envFromItem.secretRef.name }}
            <span v-if="envFromItem.prefix" class="text-gray-500 dark:text-gray-400">(prefix: {{ envFromItem.prefix }})</span>
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