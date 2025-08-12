<template>
  <!-- Overlay background -->
  <div 
    v-if="isOpen"
    @click="handleOverlayClick" 
    class="fixed inset-0 bg-black/50 z-40 transition-opacity"
  ></div>

  <!-- Panel -->
  <div class="fixed inset-y-0 right-0 bg-surface-primary shadow-2xl border-l border-border-primary z-50 flex flex-col transform transition-transform duration-300"
       style="width: 60%"
       :class="{ 'translate-x-full': !isOpen, 'translate-x-0': isOpen }">
    
    <!-- Header -->
    <ResourceHeader
      :resourceData="resourceData"
      :resourceKind="resourceKind"
      @close="$emit('close')"
    />

    <!-- Tab Navigation -->
    <TabNavigation
      v-model="activeTab"
      :tabs="availableTabs"
    />

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden">
      <!-- Overview Tab -->
      <ResourceOverview
        v-show="activeTab === 'overview'"
        :resourceData="resourceData"
        :resourceKind="resourceKind"
      />

      <!-- Pods Tab (Workload resources) -->
      <WorkloadPods
        v-show="activeTab === 'pods'"
        :resourceData="resourceData"
        :resourceKind="resourceKind"
        @viewPod="handleViewPod"
      />

      <!-- Node Pods Tab (Node resources) -->
      <NodePods
        v-show="activeTab === 'node'"
        :resourceData="resourceData"
        :resourceKind="resourceKind"
        @viewPod="handleViewPod"
      />

      <!-- Data Tab (Secret and ConfigMap resources) -->
      <div v-show="activeTab === 'data'" class="h-full overflow-y-auto p-6 space-y-6">
        <div v-if="(resourceKind === 'Secret' || resourceKind === 'ConfigMap') && getResourceData()" class="elevated-surface rounded-lg p-4">
          <h3 class="text-sm font-semibold text-text-primary mb-3">{{ resourceKind === 'Secret' ? 'Secret Data' : 'ConfigMap Data' }}</h3>
          <div class="space-y-2">
            <div v-for="(value, key) in getResourceData()" :key="key"
                 :class="[
                   'bg-surface-secondary rounded border p-3',
                   resourceKind === 'Secret' 
                     ? 'border-green-700' 
                     : 'border-blue-700'
                 ]">
              <div class="flex items-start justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span :class="[
                      'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
                      resourceKind === 'Secret'
                        ? 'status-badge-success'
                        : 'status-badge-info'
                    ]">
                      {{ key }}
                    </span>
                    <button
                      v-if="resourceKind === 'Secret'"
                      @click="toggleDataVisibility(key)"
                      :class="[
                        'text-xs px-2 py-0.5 rounded transition-colors',
                        visibleData[key] 
                          ? 'status-badge-error hover:opacity-80' 
                          : 'status-badge-warning hover:opacity-80'
                      ]"
                    >
                      {{ visibleData[key] ? 'Hide' : 'Show' }}
                    </button>
                    <button
                      v-if="(visibleData[key] || resourceKind === 'ConfigMap') && isLargeData(getDisplayValue(value))"
                      @click="toggleDataExpansion(key)"
                      class="text-xs text-text-secondary hover:text-text-primary transition-colors"
                    >
                      {{ expandedData.has(key) ? 'Collapse' : 'Expand' }}
                    </button>
                  </div>
                  <div class="text-xs text-text-primary font-mono">
                    <!-- For Secrets: Show based on visibility -->
                    <div v-if="resourceKind === 'Secret'">
                      <span v-if="!visibleData[key]" class="italic text-text-secondary">••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••</span>
                      <div v-else>
                        <div v-if="!isLargeData(getDisplayValue(value))" class="break-all">
                          {{ getDisplayValue(value) }}
                        </div>
                        <div v-else>
                          <div v-if="expandedData.has(key)" class="break-all whitespace-pre-wrap bg-surface-tertiary p-2 rounded border max-h-60 overflow-y-auto">
                            {{ formatDataValue(getDisplayValue(value)) }}
                          </div>
                          <div v-else class="text-text-secondary">
                            {{ getTruncatedDataValue(getDisplayValue(value)) }}
                            <button
                              @click="toggleDataExpansion(key)"
                              class="ml-1 text-accent-primary hover:underline"
                            >
                              Show more
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                    <!-- For ConfigMaps: Always show, with expand/collapse -->
                    <div v-else>
                      <div v-if="!isLargeData(getDisplayValue(value))" class="break-all">
                        {{ getDisplayValue(value) }}
                      </div>
                      <div v-else>
                        <div v-if="expandedData.has(key)" class="break-all whitespace-pre-wrap bg-surface-tertiary p-2 rounded border max-h-60 overflow-y-auto">
                          {{ formatDataValue(getDisplayValue(value)) }}
                        </div>
                        <div v-else class="text-text-secondary">
                          {{ getTruncatedDataValue(getDisplayValue(value)) }}
                          <button
                            @click="toggleDataExpansion(key)"
                            class="ml-1 text-accent-primary hover:underline"
                          >
                            Show more
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <button
                    @click="copyToClipboard(getDisplayValue(value), key)"
                    class="p-1 text-gray-400 hover:text-text-secondary transition-colors"
                    :title="`Copy ${key}`"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Fallback if no data -->
        <div v-else-if="resourceKind === 'Secret' || resourceKind === 'ConfigMap'" class="flex items-center justify-center h-32">
          <p class="text-text-secondary">No data available for this {{ resourceKind.toLowerCase() }}</p>
        </div>

        <!-- Not a Secret or ConfigMap Resource -->
        <div v-else class="flex items-center justify-center h-32">
          <div class="text-center">
            <div class="text-text-muted mb-2">
              <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </div>
            <p class="text-sm text-text-secondary">Data view is only available for Secret and ConfigMap resources</p>
          </div>
        </div>
      </div>

      <!-- YAML Tab -->
      <div v-show="activeTab === 'yaml'" class="h-full">
        <div v-if="yamlLoading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-2"></div>
            <p class="text-sm text-text-secondary">Loading full resource data...</p>
          </div>
        </div>
        <YamlEditor
          v-else-if="resourceData && (resourceData.complete_object || fullResourceData)"
          :yamlContent="yamlContent"
          :resourceName="resourceData.metadata?.name || ''"
          :resourceKind="resourceKind"
          :namespace="resourceData.metadata?.namespace"
          @close="$emit('close')"
          @saved="handleYamlSaved"
        />
        <div v-else-if="resourceData && !resourceData.complete_object && !fullResourceData" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="text-red-500 mb-2">
              <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </div>
            <p class="text-sm text-status-error">Failed to load full resource data</p>
            <button 
              @click="fetchFullResource" 
              class="mt-2 px-3 py-1 bg-blue-500 text-white text-sm rounded hover:bg-blue-600"
            >
              Retry
            </button>
          </div>
        </div>
      </div>

      <!-- Containers Tab (Pod and Job) -->
      <div v-show="activeTab === 'containers'" class="h-full overflow-y-auto p-6 space-y-6">
        <!-- Pod-specific components -->
        <div 
          v-if="resourceKind === 'Pod' && getGenericSpec(resourceData)?.containers"
          class="elevated-surface rounded-lg p-4"
        >
          <h3 class="text-sm font-semibold text-text-primary mb-3">Containers</h3>
          <div class="space-y-3">
            <div v-for="container in getGenericSpec(resourceData).containers" :key="container.name"
                 class="bg-surface-secondary rounded border border-border-primary p-3">
              <div class="flex items-center justify-between mb-2">
                <h4 class="text-sm font-medium text-text-primary">{{ container.name }}</h4>
                <span :class="[
                  'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
                  getOverviewContainerStatusColor(container.name)
                ]">
                  {{ getOverviewContainerStatus(container.name) }}
                </span>
              </div>
              <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2 text-xs">
                <div>
                  <dt class="text-text-secondary">Image</dt>
                  <dd class="text-text-primary font-mono break-all">{{ container.image }}</dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Pull Policy</dt>
                  <dd class="text-text-primary">{{ container.imagePullPolicy || 'Always' }}</dd>
                </div>
                <div v-if="container.ports?.length">
                  <dt class="text-text-secondary">Ports</dt>
                  <dd class="text-text-primary font-mono">
                    {{ container.ports.map((p: any) => `${p.containerPort}/${p.protocol || 'TCP'}`).join(', ') }}
                  </dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Restart Count</dt>
                  <dd class="text-text-primary">{{ getOverviewContainerRestartCount(container.name) }}</dd>
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

        <!-- Job-specific components -->
        <div 
          v-if="resourceKind === 'Job' && getGenericSpec(resourceData)?.template?.spec?.containers"
          class="elevated-surface rounded-lg p-4"
        >
          <h3 class="text-sm font-semibold text-text-primary mb-3">Job Template Containers</h3>
          <div class="space-y-3">
            <div v-for="container in getGenericSpec(resourceData).template.spec.containers" :key="container.name"
                 class="bg-surface-secondary rounded border border-border-primary p-3">
              <div class="flex items-center justify-between mb-2">
                <h4 class="text-sm font-medium text-text-primary">{{ container.name }}</h4>
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium status-badge-info">
                  Template
                </span>
              </div>
              <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2 text-xs">
                <div>
                  <dt class="text-text-secondary">Image</dt>
                  <dd class="text-text-primary font-mono break-all">{{ container.image }}</dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Pull Policy</dt>
                  <dd class="text-text-primary">{{ container.imagePullPolicy || 'Always' }}</dd>
                </div>
                <div v-if="container.ports?.length">
                  <dt class="text-text-secondary">Ports</dt>
                  <dd class="text-text-primary font-mono">
                    {{ container.ports.map((p: any) => `${p.containerPort}/${p.protocol || 'TCP'}`).join(', ') }}
                  </dd>
                </div>
                <div v-if="container.restartPolicy">
                  <dt class="text-text-secondary">Restart Policy</dt>
                  <dd class="text-text-primary">{{ container.restartPolicy }}</dd>
                </div>
                <div v-if="container.command?.length">
                  <dt class="text-text-secondary">Command</dt>
                  <dd class="text-text-primary font-mono text-xs">
                    {{ container.command.join(' ') }}
                  </dd>
                </div>
                <div v-if="container.args?.length">
                  <dt class="text-text-secondary">Args</dt>
                  <dd class="text-text-primary font-mono text-xs">
                    {{ container.args.join(' ') }}
                  </dd>
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

        <!-- Job Init Containers -->
        <div 
          v-if="resourceKind === 'Job' && getGenericSpec(resourceData)?.template?.spec?.initContainers"
          class="elevated-surface rounded-lg p-4"
        >
          <h3 class="text-sm font-semibold text-text-primary mb-3">Job Template Init Containers</h3>
          <div class="space-y-3">
            <div v-for="container in getGenericSpec(resourceData).template.spec.initContainers" :key="container.name"
                 class="bg-surface-secondary rounded border border-border-primary p-3">
              <div class="flex items-center justify-between mb-2">
                <h4 class="text-sm font-medium text-text-primary">{{ container.name }}</h4>
                <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium status-badge-secondary">
                  Init Template
                </span>
              </div>
              <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2 text-xs">
                <div>
                  <dt class="text-text-secondary">Image</dt>
                  <dd class="text-text-primary font-mono break-all">{{ container.image }}</dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Pull Policy</dt>
                  <dd class="text-text-primary">{{ container.imagePullPolicy || 'Always' }}</dd>
                </div>
                <div v-if="container.ports?.length">
                  <dt class="text-text-secondary">Ports</dt>
                  <dd class="text-text-primary font-mono">
                    {{ container.ports.map((p: any) => `${p.containerPort}/${p.protocol || 'TCP'}`).join(', ') }}
                  </dd>
                </div>
                <div v-if="container.command?.length">
                  <dt class="text-text-secondary">Command</dt>
                  <dd class="text-text-primary font-mono text-xs">
                    {{ container.command.join(' ') }}
                  </dd>
                </div>
                <div v-if="container.args?.length">
                  <dt class="text-text-secondary">Args</dt>
                  <dd class="text-text-primary font-mono text-xs">
                    {{ container.args.join(' ') }}
                  </dd>
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
        
        <!-- Pod Init Containers -->
        <div 
          v-if="resourceKind === 'Pod' && getGenericSpec(resourceData)?.initContainers"
          class="elevated-surface rounded-lg p-4"
        >
          <h3 class="text-sm font-semibold text-text-primary mb-3">Init Containers</h3>
          <div class="space-y-3">
            <div v-for="container in getGenericSpec(resourceData).initContainers" :key="container.name"
                 class="bg-surface-secondary rounded border border-border-primary p-3">
              <div class="flex items-center justify-between mb-2">
                <h4 class="text-sm font-medium text-text-primary">{{ container.name }}</h4>
                <span :class="[
                  'inline-flex items-center px-2 py-1 rounded-full text-xs font-medium',
                  getOverviewInitContainerStatusColor(container.name)
                ]">
                  {{ getOverviewInitContainerStatus(container.name) }}
                </span>
              </div>
              <dl class="grid grid-cols-1 gap-2 sm:grid-cols-2 text-xs">
                <div>
                  <dt class="text-text-secondary">Image</dt>
                  <dd class="text-text-primary font-mono break-all">{{ container.image }}</dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Pull Policy</dt>
                  <dd class="text-text-primary">{{ container.imagePullPolicy || 'Always' }}</dd>
                </div>
                <div v-if="container.ports?.length">
                  <dt class="text-text-secondary">Ports</dt>
                  <dd class="text-text-primary font-mono">
                    {{ container.ports.map((p: any) => `${p.containerPort}/${p.protocol || 'TCP'}`).join(', ') }}
                  </dd>
                </div>
                <div>
                  <dt class="text-text-secondary">Restart Count</dt>
                  <dd class="text-text-primary">{{ getOverviewInitContainerRestartCount(container.name) }}</dd>
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
        
        <!-- Fallback if no container data -->
        <div v-if="(resourceKind === 'Pod' && !getGenericSpec(resourceData)?.containers && !getGenericSpec(resourceData)?.initContainers) || (resourceKind === 'Job' && !getGenericSpec(resourceData)?.template?.spec?.containers && !getGenericSpec(resourceData)?.template?.spec?.initContainers) || (resourceKind !== 'Pod' && resourceKind !== 'Job')" class="flex items-center justify-center h-32">
          <p class="text-text-secondary">No container information available</p>
        </div>
      </div>

      <!-- Events Tab -->
      <ResourceEvents
        v-show="activeTab === 'events'"
        :events="events"
        :resourceKind="resourceKind"
      />

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue'
import { useResourceStatus } from '@/composables/useResourceStatus'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import * as yaml from 'js-yaml'

import ResourceHeader from './ResourceHeader.vue'
import TabNavigation from './TabNavigation.vue'
import ResourceOverview from './ResourceOverview.vue'
import ResourceEvents from './ResourceEvents.vue'
import ContainerEnvironment from './ContainerEnvironment.vue'
import WorkloadPods from './WorkloadPods.vue'
import NodePods from './NodePods.vue'
import YamlEditor from '../YamlEditor.vue'
import type { ContainerStatus } from '@/types'

interface Props {
  isOpen: boolean
  resourceData: any | null
  resourceKind: string
}

const props = defineProps<Props>()

// Helper functions for accessing resource-specific fields
const { getGenericStatus, getGenericSpec } = useResourceStatus()

const emit = defineEmits<{
  close: []
  viewPod: [pod: any]
}>()


// State
const activeTab = ref('overview')
const events = ref<any[]>([])

// Tab configuration - dynamically based on resource type
const availableTabs = computed(() => {
  const baseTabs = [
    { id: 'overview', label: 'Overview', icon: 'InfoIcon' },
    { id: 'yaml', label: 'YAML', icon: 'CodeIcon' },
    { id: 'events', label: 'Events', icon: 'EventIcon' }
  ]
  
  // Add containers tab for pods and jobs (before YAML tab)
  if (props.resourceKind === 'Pod' || props.resourceKind === 'Job') {
    baseTabs.splice(1, 0, { id: 'containers', label: 'Containers', icon: 'ContainerIcon' })
  }
  
  // Add pods tab for workload resources (before YAML tab)
  if (['Deployment', 'StatefulSet', 'DaemonSet', 'ReplicaSet'].includes(props.resourceKind)) {
    baseTabs.splice(1, 0, { id: 'pods', label: 'Pods', icon: 'PodIcon' })
  }
  
  // Add node tab for Node resources (before YAML tab)
  if (props.resourceKind === 'Node') {
    baseTabs.splice(1, 0, { id: 'node', label: 'Pods', icon: 'PodIcon' })
  }
  
  // Add data tab for Secrets and ConfigMaps (before YAML tab)
  if (props.resourceKind === 'Secret' || props.resourceKind === 'ConfigMap') {
    baseTabs.splice(1, 0, { id: 'data', label: 'Data', icon: 'DataIcon' })
  }
  
  return baseTabs
})

// State for full resource data
const fullResourceData = ref<any>(null)
const yamlLoading = ref(false)

// State for secret data visibility and expansion
const visibleData = ref<Record<string, boolean>>({})
const expandedData = ref<Set<string>>(new Set())

// Computed
const yamlContent = computed(() => {
  // Prefer complete_object for clean Kubernetes YAML, fallback to full resource data
  let dataToUse = props.resourceData?.complete_object || fullResourceData.value
  
  if (!dataToUse) return ''
  
  try {
    // Create a clean copy without internal wrapper fields
    const cleanData = { ...dataToUse }
    
    // Remove internal fields that shouldn't appear in YAML
    delete cleanData.completeObject
    
    // Remove resource-specific spec wrapper fields that duplicate the main resource
    delete cleanData.storageClassSpec
    delete cleanData.podDisruptionBudgetSpec
    delete cleanData.networkPolicySpec
    delete cleanData.ingressSpec
    delete cleanData.jobSpec
    delete cleanData.cronJobSpec
    delete cleanData.horizontalPodAutoscalerSpec
    delete cleanData.roleSpec
    delete cleanData.roleBindingSpec
    delete cleanData.clusterRoleSpec
    delete cleanData.clusterRoleBindingSpec
    delete cleanData.endpointSliceSpec
    
    // Clean up metadata if it exists
    if (cleanData.metadata) {
      const cleanMetadata = { ...cleanData.metadata }
      
      // Remove fields that are often null or internal
      if (cleanMetadata.labels === null) delete cleanMetadata.labels
      if (cleanMetadata.annotations === null) delete cleanMetadata.annotations
      if (cleanMetadata.ownerReferences === null) delete cleanMetadata.ownerReferences
      if (cleanMetadata.namespace === null) delete cleanMetadata.namespace
      
      // Remove managedFields as it's usually too verbose for viewing
      delete cleanMetadata.managedFields
      
      cleanData.metadata = cleanMetadata
    }
    
    // Remove null fields at root level
    Object.keys(cleanData).forEach(key => {
      if (cleanData[key] === null) {
        delete cleanData[key]
      }
    })
    
    return yaml.dump(cleanData, {
      indent: 2,
      noRefs: true,
      sortKeys: false,
      lineWidth: -1 // Prevent line wrapping
    })
  } catch (error) {
    console.error('Error converting to YAML:', error)
    return JSON.stringify(dataToUse, null, 2)
  }
})

// Watch for resource changes - handled at the bottom with logs cleanup

// Methods

async function refreshEvents(): Promise<void> {
  if (!props.resourceData) return
  
  try {
    const resourceName = props.resourceData.metadata?.name
    const namespace = props.resourceData.metadata?.namespace
    
    if (!resourceName) {
      events.value = []
      return
    }
    
    const eventList = await invoke<any[]>('get_resource_events', {
      resourceName,
      resourceKind: props.resourceKind,
      namespace: namespace || undefined
    })
    
    events.value = eventList
  } catch (error) {
    console.error('Error fetching events:', error)
    events.value = []
  }
}

// Handle YAML saved event
function handleYamlSaved(newYamlContent: string): void {
  // Optionally refresh the resource data or update the parent component
  // For now, we'll just log the success
}

// Handle view pod event
function handleViewPod(pod: any): void {
  emit('viewPod', pod)
}

// Handle overlay click - allow table row clicks to pass through
function handleOverlayClick(event: MouseEvent): void {
  // Check if the click is on a table row or table-related element
  const target = event.target as Element
  
  // Find the actual clickable table row element
  let tableRow = target.closest('tr')
  
  if (tableRow) {
    // If we clicked on a table row, simulate the click directly on the row
    // This allows the table's click handler to execute
    event.preventDefault()
    event.stopPropagation()
    
    // Dispatch a new click event on the table row
    const rowClickEvent = new MouseEvent('click', {
      bubbles: true,
      cancelable: true,
      clientX: event.clientX,
      clientY: event.clientY,
    })
    
    tableRow.dispatchEvent(rowClickEvent)
    return
  }
  
  // Check if we clicked on any table-related element
  const tableContainer = target.closest('.resource-table') || 
                         target.closest('[role="table"]') ||
                         target.closest('table')
  
  if (tableContainer) {
    // If we're in the table area but not on a specific row,
    // don't close the panel - just do nothing
    return
  }
  
  // Only close if we clicked outside the table area entirely
  emit('close')
}

// Fetch full resource data for YAML display
async function fetchFullResource(): Promise<void> {
  if (!props.resourceData?.metadata?.name) return
  
  yamlLoading.value = true
  
  try {
    const fullResource = await invoke<any>('get_full_resource', {
      resourceName: props.resourceData.metadata.name,
      resourceKind: props.resourceKind,
      namespace: props.resourceData.metadata.namespace || undefined
    })
    
    fullResourceData.value = fullResource
  } catch (error) {
    console.error('Error fetching full resource:', error)
    // Fallback to the simplified data if full fetch fails
    fullResourceData.value = props.resourceData
  } finally {
    yamlLoading.value = false
  }
}

// Initialize
onMounted(() => {
  if (props.resourceData) {
    refreshEvents()
  }
})

// Overview-style container helper functions
function getOverviewContainerStatusColor(containerName: string): string {
  const status = getOverviewContainerStatus(containerName)
  switch (status) {
    case 'Running': return 'status-badge-success'
    case 'Waiting': return 'status-badge-yellow'
    case 'Terminated': return 'status-badge-error'
    default: return 'status-badge-secondary'
  }
}

function getOverviewContainerStatus(containerName: string): string {
  const containerStatus = getGenericStatus(props.resourceData)?.containerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  if (containerStatus?.state?.running) return 'Running'
  if (containerStatus?.state?.waiting) return 'Waiting'
  if (containerStatus?.state?.terminated) return 'Terminated'
  return 'Unknown'
}

function getOverviewContainerRestartCount(containerName: string): number {
  const containerStatus = getGenericStatus(props.resourceData)?.containerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  return containerStatus?.restartCount || 0
}

function getOverviewInitContainerStatusColor(containerName: string): string {
  const status = getOverviewInitContainerStatus(containerName)
  switch (status) {
    case 'Running': return 'status-badge-success'
    case 'Waiting': return 'status-badge-yellow'
    case 'Terminated': return 'status-badge-error'
    default: return 'status-badge-secondary'
  }
}

function getOverviewInitContainerStatus(containerName: string): string {
  const containerStatus = getGenericStatus(props.resourceData)?.initContainerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  if (containerStatus?.state?.running) return 'Running'
  if (containerStatus?.state?.waiting) return 'Waiting'
  if (containerStatus?.state?.terminated) return 'Terminated'
  return 'Unknown'
}

function getOverviewInitContainerRestartCount(containerName: string): number {
  const containerStatus = getGenericStatus(props.resourceData)?.initContainerStatuses?.find(
    (cs: any) => cs.name === containerName
  )
  return containerStatus?.restartCount || 0
}

// Resource data management functions (Secret and ConfigMap)
function getResourceData(): Record<string, string> | null {
  // Try different possible locations for resource data
  const data = props.resourceData?.data || 
               props.resourceData?.complete_object?.data || 
               fullResourceData.value?.data
  
  return data && Object.keys(data).length > 0 ? data : null
}

function getDisplayValue(value: string): string {
  if (props.resourceKind === 'Secret') {
    return decodeSecretValue(value)
  } else {
    // ConfigMap data is already plain text
    return value
  }
}

function toggleDataVisibility(key: string): void {
  visibleData.value[key] = !visibleData.value[key]
}

// Data expansion management functions
function isLargeData(value: string): boolean {
  return value.length > 100 || value.includes('\n') || value.startsWith('{') || value.startsWith('[')
}

function getTruncatedDataValue(value: string): string {
  if (value.length <= 100) return value
  return value.substring(0, 100) + '...'
}

function formatDataValue(value: string): string {
  // Try to format JSON if it looks like JSON
  if (value.startsWith('{') || value.startsWith('[')) {
    try {
      const parsed = JSON.parse(value)
      return JSON.stringify(parsed, null, 2)
    } catch {
      // If not valid JSON, return as-is
      return value
    }
  }
  return value
}

function toggleDataExpansion(key: string): void {
  if (expandedData.value.has(key)) {
    expandedData.value.delete(key)
  } else {
    expandedData.value.add(key)
  }
}

function decodeSecretValue(base64Value: string): string {
  try {
    return atob(base64Value)
  } catch (error) {
    console.warn('Failed to decode base64 value:', error)
    return 'Failed to decode value'
  }
}

async function copyToClipboard(text: string, key: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
    // You could add a toast notification here if desired
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()
    try {
      document.execCommand('copy')
    } catch (fallbackError) {
      console.error('Fallback copy method also failed:', fallbackError)
    }
    document.body.removeChild(textArea)
  }
}

// Watch for resource changes - only reset tab for new resources
let previousResourceId: string | null = null

watch(() => props.resourceData, (newResource, oldResource) => {
  const currentResourceId = newResource?.metadata?.uid || null
  
  // Only reset to overview if this is a completely different resource
  // (not just an update to the same resource)
  if (currentResourceId !== previousResourceId) {
    activeTab.value = 'overview'
    events.value = []
    fullResourceData.value = null
    visibleData.value = {} // Reset secret data visibility
    expandedData.value.clear() // Reset data expansion state
  } else {
    // Same resource, just refresh events if we're on the events tab
    if (activeTab.value === 'events') {
      events.value = []
    }
  }
  
  previousResourceId = currentResourceId
})

// Watch for active tab changes
watch(() => activeTab.value, (newTab) => {
  if (newTab === 'events') {
    refreshEvents()
  } else if (newTab === 'yaml' && !props.resourceData?.complete_object) {
    fetchFullResource()
  } else if (newTab === 'data' && (props.resourceKind === 'Secret' || props.resourceKind === 'ConfigMap') && !getResourceData()) {
    // Fetch full resource data for data tab if data is not available
    fetchFullResource()
  }
})

// Handle keyboard events
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.isOpen) {
    emit('close')
  }
}

// Add/remove keyboard event listener
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
  }
})

// Cleanup on unmount
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>