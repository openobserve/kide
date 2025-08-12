<template>
  <div class="flex-1 overflow-auto resource-table table-background" 
       @mouseenter="() => $emit('setMouseOverTable', true)" 
       @mouseleave="() => { $emit('setMouseOverTable', false); forceClearHoveredRow('table-exit'); }"
       @scroll="() => forceClearHoveredRow('scroll')">
    <table class="min-w-full table-background" style="table-layout: fixed;" :style="{ width: table.getTotalSize() + 'px' }">
      <!-- Table Header -->
      <thead class="table-header-background border-b border-border-primary sticky top-0 z-10 backdrop-blur-sm">
        <tr v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
          <th
            v-for="header in headerGroup.headers"
            :key="header.id"
            :colSpan="header.colSpan"
            :style="{ width: header.getSize() + 'px' }"
            class="px-3 py-2 text-left text-xs font-medium table-header-text uppercase tracking-wider relative select-none"
            :class="[
              header.column.getCanSort() ? 'cursor-pointer group' : '',
            ]"
            @click="header.column.getToggleSortingHandler()?.($event)"
            @mouseenter="$emit('clearHoveredRow', 'header-hover')"
          >
            <div class="flex items-center space-x-1">
              <FlexRender
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
              <!-- Sort indicator -->
              <div v-if="header.column.getCanSort()" class="w-5 h-5">
                <svg v-if="header.column.getIsSorted() === 'asc'" 
                     class="w-5 h-5 text-text-secondary" 
                     fill="currentColor" 
                     viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z" clip-rule="evenodd"/>
                </svg>
                <svg v-else-if="header.column.getIsSorted() === 'desc'" 
                     class="w-5 h-5 text-text-secondary" 
                     fill="currentColor" 
                     viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
                </svg>
                <svg v-else 
                     class="w-5 h-5 text-text-muted opacity-0 group-hover:opacity-100" 
                     fill="currentColor" 
                     viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
                </svg>
              </div>
            </div>

            <!-- Column resize handle -->
            <div
              v-if="header.column.getCanResize()"
              @mousedown="header.getResizeHandler()?.($event)"
              @touchstart="header.getResizeHandler()?.($event)"
              class="absolute right-0 top-0 h-full w-1 cursor-col-resize hover:bg-blue-500 transition-colors border-r-2 border-transparent hover:border-blue-500 active:bg-blue-600"
              :class="[
                header.column.getIsResizing() ? 'bg-blue-500 border-blue-500' : 'bg-transparent hover:bg-blue-300 dark:hover:bg-blue-600'
              ]"
            ></div>
          </th>
        </tr>
      </thead>

      <!-- Table Body -->
      <tbody class="table-background divide-y divide-border-primary">
        <tr
          v-for="row in table.getRowModel().rows"
          :key="row.id"
          data-testid="resource-row"
          @click="$emit('selectItem', row.original)"
          @mouseenter="$emit('setHoveredRow', row.original.metadata?.uid, `table-row-${row.index}`)"
          @mouseleave="$emit('clearHoveredRow', `table-row-${row.index}`)"
          :class="[
            'table-row-background cursor-pointer',
            selectedItem?.metadata?.uid === row.original.metadata?.uid ? 'selected-state' : '',
            selectedItems.has(row.original.metadata?.uid || '') ? 'bg-blue-900/20' : ''
          ]"
        >
          <td
            v-for="cell in row.getVisibleCells()"
            :key="cell.id"
            :style="{ width: cell.column.getSize() + 'px' }"
            class="px-3 py-1 whitespace-nowrap text-sm overflow-hidden text-ellipsis table-cell-text"
            :data-column-id="cell.column.id"
          >
            <FlexRender
              :render="cell.column.columnDef.cell"
              :props="cell.getContext()"
            />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
  
  <!-- Shadcn-style tooltip -->
  <Teleport to="body">
    <div
      v-if="tooltipState.show"
      class="fixed z-[9999] px-2 py-1 text-xs text-white bg-surface-secondary rounded border shadow-lg pointer-events-none"
      :style="{
        left: tooltipState.x + 'px',
        top: tooltipState.y + 'px',
        transform: 'translateY(-50%)'
      }"
    >
      <div v-for="(line, index) in tooltipState.content" :key="index" class="whitespace-nowrap" :class="{ 'font-medium': index === 0, 'text-gray-300': index > 0 }">
        {{ line }}
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, h, Teleport } from 'vue'
import {
  getCoreRowModel,
  getSortedRowModel,
  useVueTable,
  FlexRender,
  type ColumnDef,
  type SortingState,
  type ColumnResizeMode
} from '@tanstack/vue-table'
import type { K8sListItem, K8sResource, ContainerStatus } from '@/types'
import { useResourceStatus } from '@/composables/useResourceStatus'

interface Props {
  resource: K8sResource | null
  items: K8sListItem[]
  selectedItems: Set<string>
  selectedItem: K8sListItem | null
  hoveredRowId: string | null
  isMouseOverTable: boolean
  getStatusText: (item: K8sListItem) => string
  getStatusClass: (item: K8sListItem) => string
  getAge: (timestamp?: string) => string
  getTotalRestartCount: (item: K8sListItem) => number
  getControlledBy: (item: K8sListItem) => string | null
  getQoSClass: (item: K8sListItem) => string
  getContainerStatusColor: (container: ContainerStatus) => string
  getContainerStatusText: (container: ContainerStatus) => string
}

interface Emits {
  'toggleSelectAll': []
  'toggleItemSelection': [item: K8sListItem]
  'selectItem': [item: K8sListItem]
  'setHoveredRow': [itemId: string | undefined, source: string]
  'clearHoveredRow': [source: string]
  'setMouseOverTable': [value: boolean]
  'openPodLogs': [pod: K8sListItem]
  'openPodShell': [pod: K8sListItem]
  'deleteResource': [item: K8sListItem]
  'selectNamespace': [namespace: string]
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Import helper functions for accessing resource-specific fields
const { getGenericStatus, getGenericSpec } = useResourceStatus()

// State
const sorting = ref<SortingState>([])
const columnSizing = ref<Record<string, number>>({})
const columnResizeMode = ref<ColumnResizeMode>('onChange')

// Tooltip state
const tooltipState = ref({
  show: false,
  content: [] as string[],
  x: 0,
  y: 0
})

// Load column sizes from localStorage
const storageKey = computed(() => `kide-column-sizes-${props.resource?.kind || 'unknown'}`)

function loadColumnSizes(): Record<string, number> {
  try {
    const saved = localStorage.getItem(storageKey.value)
    return saved ? JSON.parse(saved) : {}
  } catch {
    return {}
  }
}

function saveColumnSizes(columnSizing: Record<string, number>) {
  try {
    localStorage.setItem(storageKey.value, JSON.stringify(columnSizing))
  } catch (error) {
    console.warn('Failed to save column sizes:', error)
  }
}

// Tooltip functions
function showTooltip(event: MouseEvent, content: string[]) {
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  tooltipState.value = {
    show: true,
    content,
    x: rect.right + 8,
    y: rect.top + rect.height / 2
  }
}

function hideTooltip() {
  tooltipState.value.show = false
}

// Generate enhanced tooltip content for containers
function getContainerTooltipLines(container: any, isInit = false): string[] {
  const lines = []
  
  // Container name
  lines.push(isInit ? `Init: ${container.name || 'Unknown'}` : container.name || 'Unknown')
  
  // Status
  lines.push(`Status: ${props.getContainerStatusText(container) || 'Unknown'}`)
  
  // Container ID if available
  if (container.containerID) {
    const shortId = container.containerID.replace(/^docker:\/\/|^containerd:\/\//, '').substring(0, 12)
    lines.push(`ID: ${shortId}`)
  }
  
  // State-specific information
  if (container.state?.running?.startedAt) {
    lines.push(`Started: ${props.getAge(container.state.running.startedAt)} ago`)
  } else if (container.state?.terminated) {
    const terminated = container.state.terminated
    
    // Exit code
    if (terminated.exitCode !== undefined) {
      lines.push(`Exit Code: ${terminated.exitCode}`)
    }
    
    // Reason
    if (terminated.reason) {
      lines.push(`Reason: ${terminated.reason}`)
    }
    
    // Started at
    if (terminated.startedAt) {
      lines.push(`Started: ${props.getAge(terminated.startedAt)} ago`)
    }
    
    // Finished at
    if (terminated.finishedAt) {
      lines.push(`Finished: ${props.getAge(terminated.finishedAt)} ago`)
    }
    
    // Container ID
    if (terminated.containerID) {
      const shortId = terminated.containerID.replace(/^docker:\/\/|^containerd:\/\//, '').substring(0, 12)
      lines.push(`Container ID: ${shortId}`)
    }
  } else if (container.state?.waiting) {
    const waiting = container.state.waiting
    if (waiting.reason) {
      lines.push(`Reason: ${waiting.reason}`)
    }
    if (waiting.message) {
      lines.push(`Message: ${waiting.message}`)
    }
  }
  
  return lines
}


// Define columns
const columns = computed((): ColumnDef<K8sListItem>[] => {
  const savedSizes = loadColumnSizes()
  
  const baseColumns: ColumnDef<K8sListItem>[] = [
    {
      id: 'select',
      header: () => h('input', {
        type: 'checkbox',
        checked: props.selectedItems.size > 0 && props.selectedItems.size === props.items.length,
        indeterminate: props.selectedItems.size > 0 && props.selectedItems.size < props.items.length,
        onChange: () => emit('toggleSelectAll'),
        class: 'rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:bg-gray-700'
      }),
      cell: ({ row }) => h('input', {
        type: 'checkbox',
        checked: props.selectedItems.has(row.original.metadata?.uid || ''),
        onChange: (e: Event) => {
          e.stopPropagation()
          emit('toggleItemSelection', row.original)
        },
        onClick: (e: Event) => e.stopPropagation(),
        class: 'rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 dark:bg-gray-700'
      }),
      size: savedSizes.select || 60,
      minSize: 50,
      enableSorting: false,
      enableResizing: true
    },
    {
      accessorKey: 'metadata.name',
      id: 'name',
      header: 'Name',
      cell: ({ row, getValue }) => {
        const name = getValue() as string || 'Unknown'
        return h('div', {
          class: 'relative w-full',
          'data-resource-name': name
        }, [
          h('div', { 
            class: 'truncate min-w-0 table-cell-text',
            title: name
          }, name),
          // Hover buttons positioned floating on the right side above text
          h('div', {
            class: [
              'absolute right-2 top-0 bottom-0 flex items-center space-x-1 transition-opacity duration-150 z-10',
              props.hoveredRowId === row.original.metadata?.uid ? 'opacity-100' : 'opacity-0'
            ],
            style: { pointerEvents: props.hoveredRowId === row.original.metadata?.uid ? 'auto' : 'none' },
            onMouseenter: () => emit('setHoveredRow', row.original.metadata?.uid, 'button-container'),
            onClick: (e: Event) => e.stopPropagation()
          }, createHoverButtons(row.original))
        ])
      },
      size: savedSizes.name || 200,
      minSize: 50,
      enableSorting: true,
      enableResizing: true
    }
  ]

  // Add namespace column if resource is namespaced
  if (props.resource?.namespaced) {
    baseColumns.push({
      accessorKey: 'metadata.namespace',
      id: 'namespace',
      header: 'Namespace',
      cell: ({ getValue }) => {
        const namespace = getValue() as string
        if (namespace) {
          return h('button', {
            class: 'px-1.5 py-0 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-xs rounded-full truncate max-w-full inline-block hover:bg-blue-100 dark:hover:bg-blue-800 hover:text-blue-700 dark:hover:text-blue-300 transition-colors cursor-pointer',
            title: `Filter by namespace: ${namespace}`,
            onClick: (e: Event) => {
              e.stopPropagation()
              emit('selectNamespace', namespace)
            }
          }, namespace)
        }
        return h('span', { class: 'text-text-muted text-xs' }, '-')
      },
      size: savedSizes.namespace || 120,
      minSize: 50,
      enableSorting: true,
      enableResizing: true
    })
  }

  // Add Job-specific columns
  if (props.resource?.kind === 'Job') {
    baseColumns.push(
      {
        id: 'succeeded',
        header: 'Succeeded',
        accessorFn: (row) => getGenericStatus(row)?.succeeded || 0,
        cell: ({ row }) => {
          const succeeded = getGenericStatus(row.original)?.succeeded || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              succeeded > 0 
                ? 'text-green-600 dark:text-green-400' 
                : 'text-text-muted'
            ]
          }, succeeded.toString())
        },
        size: savedSizes.succeeded || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'completions',
        header: 'Completions',
        accessorFn: (row) => getGenericSpec(row)?.completions || 0,
        cell: ({ row }) => {
          const completions = getGenericSpec(row.original)?.completions
          const succeeded = getGenericStatus(row.original)?.succeeded || 0
          if (completions) {
            return h('div', {
              class: 'text-xs table-cell-text',
              title: `${succeeded} of ${completions} completions`
            }, `${succeeded}/${completions}`)
          }
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.completions || 90,
        minSize: 70,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'parallelism',
        header: 'Parallelism',
        accessorFn: (row) => getGenericSpec(row)?.parallelism || 1,
        cell: ({ row }) => {
          const parallelism = getGenericSpec(row.original)?.parallelism
          if (parallelism !== undefined) {
            return h('div', {
              class: 'text-xs table-cell-text'
            }, parallelism.toString())
          }
          return h('span', { class: 'text-gray-400 dark:text-gray-500 text-xs' }, '1')
        },
        size: savedSizes.parallelism || 80,
        minSize: 60,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'duration',
        header: 'Duration',
        accessorFn: (row) => {
          const startTime = getGenericStatus(row)?.startTime
          const completionTime = getGenericStatus(row)?.completionTime
          if (startTime) {
            const start = new Date(startTime).getTime()
            const end = completionTime ? new Date(completionTime).getTime() : Date.now()
            return end - start // Return duration in milliseconds for sorting
          }
          return 0
        },
        cell: ({ row }) => {
          const startTime = getGenericStatus(row.original)?.startTime
          const completionTime = getGenericStatus(row.original)?.completionTime
          
          if (startTime) {
            const start = new Date(startTime).getTime()
            const end = completionTime ? new Date(completionTime).getTime() : Date.now()
            const durationMs = end - start
            const duration = getJobDuration(durationMs)
            
            return h('div', {
              class: 'text-xs table-cell-text',
              title: completionTime ? 'Completed' : 'Running'
            }, duration)
          }
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.duration || 80,
        minSize: 60,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'suspend',
        header: 'Suspend',
        accessorFn: (row) => getGenericSpec(row)?.suspend || false,
        cell: ({ row }) => {
          const suspend = getGenericSpec(row.original)?.suspend
          return h('div', {
            class: [
              'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium',
              suspend 
                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
                : 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
            ]
          }, suspend ? 'Yes' : 'No')
        },
        size: savedSizes.suspend || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      }
    )
  }

  // Add CronJob-specific columns
  if (props.resource?.kind === 'CronJob') {
    baseColumns.push(
      {
        id: 'schedule',
        header: 'Schedule',
        accessorFn: (row) => getGenericSpec(row)?.schedule || '',
        cell: ({ row }) => {
          const schedule = getGenericSpec(row.original)?.schedule
          if (schedule) {
            return h('div', {
              class: 'text-xs table-cell-text font-mono',
              title: `Schedule: ${schedule}`
            }, schedule)
          }
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.schedule || 120,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'timezone',
        header: 'Timezone',
        accessorFn: (row) => getGenericSpec(row)?.timeZone || 'UTC',
        cell: ({ row }) => {
          const timezone = getGenericSpec(row.original)?.timeZone
          if (timezone) {
            return h('div', {
              class: 'text-xs table-cell-text',
              title: `Timezone: ${timezone}`
            }, timezone)
          }
          return h('span', { class: 'text-gray-400 dark:text-gray-500 text-xs' }, 'UTC')
        },
        size: savedSizes.timezone || 100,
        minSize: 70,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'suspend',
        header: 'Suspend',
        accessorFn: (row) => getGenericSpec(row)?.suspend || false,
        cell: ({ row }) => {
          const suspend = getGenericSpec(row.original)?.suspend
          return h('div', {
            class: [
              'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium',
              suspend 
                ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
                : 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
            ]
          }, suspend ? 'Yes' : 'No')
        },
        size: savedSizes.suspend || 80,
        minSize: 60,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'active',
        header: 'Active',
        accessorFn: (row) => getGenericStatus(row)?.active?.length || 0,
        cell: ({ row }) => {
          const activeJobs = getGenericStatus(row.original)?.active?.length || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              activeJobs > 0 
                ? 'text-green-600 dark:text-green-400' 
                : 'text-text-muted'
            ]
          }, activeJobs.toString())
        },
        size: savedSizes.active || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'lastSchedule',
        header: 'Last Schedule',
        accessorFn: (row) => getGenericStatus(row)?.lastScheduleTime || '',
        cell: ({ row }) => {
          const lastScheduleTime = getGenericStatus(row.original)?.lastScheduleTime
          if (lastScheduleTime) {
            const age = props.getAge(lastScheduleTime)
            return h('div', {
              class: 'text-xs table-cell-text',
              title: new Date(lastScheduleTime).toLocaleString()
            }, `${age} ago`)
          }
          return h('span', { class: 'text-gray-400 dark:text-gray-500 text-xs' }, 'Never')
        },
        size: savedSizes.lastSchedule || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      }
    )
  }

  // Add Pod-specific columns
  if (props.resource?.kind === 'Pod') {
    baseColumns.push(
      {
        id: 'containers',
        header: 'Containers',
        accessorFn: (row) => {
          const status = getGenericStatus(row)
          const containerStatuses = status?.containerStatuses || []
          const initContainerStatuses = status?.initContainerStatuses || []
          return containerStatuses.length + initContainerStatuses.length
        },
        cell: ({ row }) => createContainersCell(row.original),
        size: savedSizes.containers || 100,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'restarts',
        header: 'Restarts',
        accessorFn: (row) => props.getTotalRestartCount(row),
        cell: ({ row }) => h('span', {
          class: 'text-xs table-cell-text'
        }, props.getTotalRestartCount(row.original).toString()),
        size: savedSizes.restarts || 80,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'controlled_by',
        header: 'Controlled By',
        accessorFn: (row) => props.getControlledBy(row) || '',
        cell: ({ row }) => {
          const controlledBy = props.getControlledBy(row.original)
          if (controlledBy) {
            return h('div', {
              class: 'text-xs table-cell-text truncate max-w-full',
              title: controlledBy
            }, controlledBy)
          }
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.controlled_by || 140,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'node',
        header: 'Node',
        accessorFn: (row) => (getGenericSpec(row) as any)?.nodeName || '',
        cell: ({ row }) => {
          // Backend sends camelCase field names as per k8s-openapi serialization
          const nodeName = (getGenericSpec(row.original) as any)?.nodeName
          if (nodeName) {
            return h('div', {
              class: 'text-xs table-cell-text truncate max-w-full',
              title: nodeName
            }, nodeName)
          }
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.node || 120,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'qos',
        header: 'QoS',
        accessorFn: (row) => props.getQoSClass(row),
        cell: ({ row }) => {
          const qosClass = props.getQoSClass(row.original)
          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              qosClass === 'Guaranteed' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
              qosClass === 'Burstable' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300' :
              qosClass === 'BestEffort' ? 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300' :
              'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ],
            title: qosClass
          }, qosClass)
        },
        size: savedSizes.qos || 80,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      }
    )
  }

  // Add NetworkPolicy-specific columns
  if (props.resource?.kind === 'NetworkPolicy') {
    baseColumns.push(
      {
        id: 'policyType',
        header: 'Policy Type',
        accessorFn: (row) => {
          const policyTypes = getGenericSpec(row)?.policyTypes
          if (policyTypes) {
            if (policyTypes.includes('Ingress') && policyTypes.includes('Egress')) {
              return 'Both'
            } else if (policyTypes.includes('Ingress')) {
              return 'Ingress'
            } else if (policyTypes.includes('Egress')) {
              return 'Egress'
            }
          }
          const hasEgress = getGenericSpec(row)?.egress != null
          return hasEgress ? 'Both' : 'Ingress'
        },
        cell: ({ row }) => {
          const policyTypes = getGenericSpec(row.original)?.policyTypes
          let displayValue = 'Ingress' // Default

          if (policyTypes) {
            if (policyTypes.includes('Ingress') && policyTypes.includes('Egress')) {
              displayValue = 'Both'
            } else if (policyTypes.includes('Ingress')) {
              displayValue = 'Ingress'
            } else if (policyTypes.includes('Egress')) {
              displayValue = 'Egress'
            }
          } else {
            // Default behavior when policyTypes not specified
            const hasEgress = getGenericSpec(row.original)?.egress != null
            if (hasEgress) {
              displayValue = 'Both'
            }
          }

          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              displayValue === 'Both' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300' :
              displayValue === 'Ingress' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300' :
              displayValue === 'Egress' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
              'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ],
            title: `Policy handles ${displayValue.toLowerCase()} traffic`
          }, displayValue)
        },
        size: savedSizes.policyType || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      }
    )
  }

  // Add Deployment-specific columns (replace Status)
  if (props.resource?.kind === 'Deployment') {
    baseColumns.push(
      {
        id: 'ready',
        header: 'Ready',
        accessorFn: (row) => {
          const readyReplicas = getGenericStatus(row)?.readyReplicas || 0
          const replicas = getGenericSpec(row)?.replicas || 0
          return readyReplicas / Math.max(replicas, 1) // Sort by ready ratio
        },
        cell: ({ row }) => {
          const readyReplicas = getGenericStatus(row.original)?.readyReplicas || 0
          const replicas = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              readyReplicas === replicas && replicas > 0
                ? 'text-green-600 dark:text-green-400'
                : readyReplicas > 0
                  ? 'text-yellow-600 dark:text-yellow-400'
                  : 'text-red-600 dark:text-red-400'
            ],
            title: `${readyReplicas} of ${replicas} ready`
          }, `${readyReplicas}/${replicas}`)
        },
        size: savedSizes.ready || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'desired',
        header: 'Desired',
        cell: ({ row }) => {
          const desired = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: 'text-xs table-cell-text'
          }, desired.toString())
        },
        size: savedSizes.desired || 60,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'updated',
        header: 'Updated',
        cell: ({ row }) => {
          const updatedReplicas = getGenericStatus(row.original)?.updatedReplicas || 0
          const replicas = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              updatedReplicas === replicas
                ? 'text-green-600 dark:text-green-400'
                : 'text-yellow-600 dark:text-yellow-400'
            ],
            title: `${updatedReplicas} of ${replicas} updated`
          }, updatedReplicas.toString())
        },
        size: savedSizes.updated || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'available',
        header: 'Available',
        cell: ({ row }) => {
          const availableReplicas = getGenericStatus(row.original)?.availableReplicas || 0
          const replicas = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              availableReplicas === replicas
                ? 'text-green-600 dark:text-green-400'
                : availableReplicas > 0
                  ? 'text-yellow-600 dark:text-yellow-400'
                  : 'text-red-600 dark:text-red-400'
            ],
            title: `${availableReplicas} of ${replicas} available`
          }, availableReplicas.toString())
        },
        size: savedSizes.available || 80,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'conditions',
        header: 'Conditions',
        cell: ({ row }) => {
          const conditions = getGenericStatus(row.original)?.conditions || []
          const availableCondition = conditions.find((c: any) => c.type === 'Available')
          const progressingCondition = conditions.find((c: any) => c.type === 'Progressing')
          
          if (availableCondition?.status === 'True' && progressingCondition?.status === 'True') {
            return h('div', {
              class: 'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
              title: 'Deployment is available and progressing normally'
            }, 'Healthy')
          }
          
          if (availableCondition?.status === 'False') {
            return h('div', {
              class: 'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300',
              title: availableCondition.reason || 'Not available'
            }, 'Unavailable')
          }
          
          if (progressingCondition?.status === 'False') {
            return h('div', {
              class: 'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300',
              title: progressingCondition.reason || 'Not progressing'
            }, 'Stalled')
          }
          
          if (progressingCondition?.status === 'True') {
            return h('div', {
              class: 'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
              title: 'Deployment is progressing'
            }, 'Progressing')
          }
          
          return h('div', {
            class: 'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
          }, 'Unknown')
        },
        sortingFn: (rowA, rowB) => {
          // Define sort priority: Healthy (4) > Progressing (3) > Stalled (2) > Unavailable (1) > Unknown (0)
          const getConditionPriority = (row: any) => {
            const conditions = getGenericStatus(row.original)?.conditions || []
            const availableCondition = conditions.find((c: any) => c.type === 'Available')
            const progressingCondition = conditions.find((c: any) => c.type === 'Progressing')
            
            if (availableCondition?.status === 'True' && progressingCondition?.status === 'True') return 4 // Healthy
            if (progressingCondition?.status === 'True') return 3 // Progressing
            if (progressingCondition?.status === 'False') return 2 // Stalled
            if (availableCondition?.status === 'False') return 1 // Unavailable
            return 0 // Unknown
          }
          
          return getConditionPriority(rowA) - getConditionPriority(rowB)
        },
        size: savedSizes.conditions || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'StatefulSet') {
    // Add StatefulSet-specific columns (replace Status)
    baseColumns.push(
      {
        id: 'ready',
        header: 'Ready',
        cell: ({ row }) => {
          const readyReplicas = getGenericStatus(row.original)?.readyReplicas || 0
          const replicas = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: [
              'text-xs font-medium',
              readyReplicas === replicas && replicas > 0
                ? 'text-green-600 dark:text-green-400'
                : readyReplicas > 0
                  ? 'text-yellow-600 dark:text-yellow-400'
                  : 'text-red-600 dark:text-red-400'
            ],
            title: `${readyReplicas} of ${replicas} ready`
          }, `${readyReplicas}/${replicas}`)
        },
        size: savedSizes.ready || 70,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'desired',
        header: 'Desired',
        cell: ({ row }) => {
          const desired = getGenericSpec(row.original)?.replicas || 0
          return h('div', {
            class: 'text-xs table-cell-text'
          }, desired.toString())
        },
        size: savedSizes.desired || 60,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'Service') {
    // Add Service-specific columns (replace Status)
    baseColumns.push(
      {
        id: 'type',
        header: 'Type',
        accessorFn: (row) => getGenericSpec(row)?.type || 'ClusterIP',
        cell: ({ row }) => {
          const type = getGenericSpec(row.original)?.type || 'ClusterIP'
          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              type === 'LoadBalancer' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300' :
              type === 'NodePort' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300' :
              type === 'ClusterIP' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300' :
              type === 'ExternalName' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300' :
              'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ],
            title: type
          }, type)
        },
        size: savedSizes.type || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'clusterIP',
        header: 'Cluster IP',
        accessorFn: (row) => getGenericSpec(row)?.clusterIP || '',
        cell: ({ row }) => {
          const clusterIP = getGenericSpec(row.original)?.clusterIP
          if (!clusterIP || clusterIP === 'None') {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-xs font-mono table-cell-text truncate max-w-full',
            title: clusterIP
          }, clusterIP)
        },
        size: savedSizes.clusterIP || 120,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'externalIP',
        header: 'External IP',
        accessorFn: (row) => {
          const externalIPs = getGenericSpec(row)?.externalIPs || []
          const loadBalancerIPs = getGenericStatus(row)?.loadBalancer?.ingress || []
          
          if (externalIPs.length > 0) {
            return externalIPs[0]
          }
          
          if (loadBalancerIPs.length > 0) {
            const ips = loadBalancerIPs.map((ing: any) => ing.ip || ing.hostname).filter(Boolean)
            if (ips.length > 0) {
              return ips[0]
            }
          }
          
          return ''
        },
        cell: ({ row }) => {
          const externalIPs = getGenericSpec(row.original)?.externalIPs || []
          const loadBalancerIPs = getGenericStatus(row.original)?.loadBalancer?.ingress || []
          
          // Show external IPs if specified
          if (externalIPs.length > 0) {
            return h('div', {
              class: 'text-xs font-mono table-cell-text truncate max-w-full',
              title: externalIPs.join(', ')
            }, externalIPs.length > 1 ? `${externalIPs[0]} +${externalIPs.length - 1}` : externalIPs[0])
          }
          
          // Show load balancer IPs if available
          if (loadBalancerIPs.length > 0) {
            const ips = loadBalancerIPs.map((ing: any) => ing.ip || ing.hostname).filter(Boolean)
            if (ips.length > 0) {
              return h('div', {
                class: 'text-xs font-mono table-cell-text truncate max-w-full',
                title: ips.join(', ')
              }, ips.length > 1 ? `${ips[0]} +${ips.length - 1}` : ips[0])
            }
          }
          
          const type = getGenericSpec(row.original)?.type || 'ClusterIP'
          if (type === 'LoadBalancer') {
            return h('span', { 
              class: 'text-yellow-500 dark:text-yellow-400 text-xs',
              title: 'Waiting for external IP assignment'
            }, 'Pending')
          }
          
          return h('span', { class: 'text-text-muted text-xs' }, '-')
        },
        size: savedSizes.externalIP || 120,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'ports',
        header: 'Ports',
        accessorFn: (row) => {
          const ports = getGenericSpec(row)?.ports || []
          if (ports.length === 0) return ''
          // Sort by first port number
          return ports[0]?.port || ports[0]?.targetPort || 0
        },
        cell: ({ row }) => {
          const ports = getGenericSpec(row.original)?.ports || []
          if (ports.length === 0) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          
          const portStrings = ports.map((port: any) => {
            const portNum = port.port || port.targetPort
            const protocol = port.protocol || 'TCP'
            let result = `${portNum}/${protocol}`
            
            // Add node port if it's a NodePort or LoadBalancer service
            if ((getGenericSpec(row.original)?.type === 'NodePort' || getGenericSpec(row.original)?.type === 'LoadBalancer') && port.nodePort) {
              result += `:${port.nodePort}`
            }
            
            return result
          })
          
          const displayText = portStrings.length > 2 
            ? `${portStrings.slice(0, 2).join(', ')} +${portStrings.length - 2}` 
            : portStrings.join(', ')
          
          return h('div', {
            class: 'text-xs font-mono table-cell-text truncate max-w-full',
            title: portStrings.join(', ')
          }, displayText)
        },
        size: savedSizes.ports || 140,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'status',
        header: 'Status',
        accessorFn: (row) => props.getStatusText(row),
        cell: ({ row }) => {
          const status = props.getStatusText(row.original)
          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              props.getStatusClass(row.original)
            ],
            title: status
          }, status)
        },
        size: savedSizes.status || 90,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'EndpointSlice') {
    // Add EndpointSlice-specific columns
    baseColumns.push(
      {
        id: 'addressType',
        header: 'Address Type',
        accessorFn: (row) => row.endpoint_slice?.addressType || 'IPv4',
        cell: ({ row }) => {
          const addressType = row.original.endpoint_slice?.addressType || 'IPv4'
          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              addressType === 'IPv4' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300' :
              addressType === 'IPv6' ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300' :
              'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300'
            ],
            title: addressType
          }, addressType)
        },
        size: savedSizes.addressType || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'ports',
        header: 'Ports',
        accessorFn: (row) => {
          const ports = row.endpoint_slice?.ports || []
          if (ports.length === 0) return 0
          // Sort by first port number
          return ports[0]?.port || 0
        },
        cell: ({ row }) => {
          // EndpointSlice ports are at the top level
          const ports = row.original.endpoint_slice?.ports || []
          if (ports.length === 0) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          
          // Show just the port numbers, similar to kubectl output
          const portStrings = ports.map((port: any) => {
            return port.port?.toString() || '?'
          })
          
          const displayText = portStrings.length > 3 
            ? `${portStrings.slice(0, 3).join(',')} +${portStrings.length - 3}`
            : portStrings.join(',')
          
          return h('div', {
            class: 'text-xs font-mono table-cell-text truncate max-w-full',
            title: `Ports: ${portStrings.join(', ')}`
          }, displayText)
        },
        size: savedSizes.ports || 120,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'endpoints',
        header: 'Endpoints',
        accessorFn: (row) => {
          const endpoints = row.endpoint_slice?.endpoints || []
          // Count total number of addresses across all endpoints
          let totalAddresses = 0
          endpoints.forEach((endpoint: any) => {
            if (endpoint.addresses && Array.isArray(endpoint.addresses)) {
              totalAddresses += endpoint.addresses.length
            }
          })
          return totalAddresses
        },
        cell: ({ row }) => {
          const endpoints = row.original.endpoint_slice?.endpoints || []
          if (endpoints.length === 0) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          
          // Extract all IP addresses from endpoints, similar to kubectl output
          const allAddresses: string[] = []
          endpoints.forEach((endpoint: any) => {
            if (endpoint.addresses && Array.isArray(endpoint.addresses)) {
              allAddresses.push(...endpoint.addresses)
            }
          })
          
          if (allAddresses.length === 0) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          
          // Display addresses like kubectl: "10.2.119.219,10.2.136.69"
          const displayText = allAddresses.length > 2 
            ? `${allAddresses.slice(0, 2).join(',')} +${allAddresses.length - 2}`
            : allAddresses.join(',')
          
          return h('div', {
            class: 'text-xs font-mono table-cell-text truncate max-w-full',
            title: `Endpoints: ${allAddresses.join(', ')}`
          }, displayText)
        },
        size: savedSizes.endpoints || 140,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'PodDisruptionBudget') {
    // Add PodDisruptionBudget-specific columns
    baseColumns.push(
      {
        id: 'currentHealthy',
        header: 'Current Healthy',
        accessorFn: (row) => getGenericStatus(row)?.currentHealthy || 0,
        cell: ({ row }) => {
          const currentHealthy = getGenericStatus(row.original)?.currentHealthy || 0
          return h('div', {
            class: 'text-sm table-cell-text'
          }, currentHealthy.toString())
        },
        size: savedSizes.currentHealthy || 90,
        minSize: 70,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'desiredHealthy',
        header: 'Desired Healthy',
        accessorFn: (row) => getGenericStatus(row)?.desiredHealthy || 0,
        cell: ({ row }) => {
          const desiredHealthy = getGenericStatus(row.original)?.desiredHealthy || 0
          return h('div', {
            class: 'text-sm table-cell-text'
          }, desiredHealthy.toString())
        },
        size: savedSizes.desiredHealthy || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'disruptionsAllowed',
        header: 'Disruptions Allowed',
        accessorFn: (row) => getGenericStatus(row)?.disruptionsAllowed || 0,
        cell: ({ row }) => {
          const disruptionsAllowed = getGenericStatus(row.original)?.disruptionsAllowed || 0
          return h('div', {
            class: 'text-sm table-cell-text'
          }, disruptionsAllowed.toString())
        },
        size: savedSizes.disruptionsAllowed || 120,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'expectedPods',
        header: 'Expected Pods',
        accessorFn: (row) => getGenericStatus(row)?.expectedPods || 0,
        cell: ({ row }) => {
          const expectedPods = getGenericStatus(row.original)?.expectedPods || 0
          return h('div', {
            class: 'text-sm table-cell-text'
          }, expectedPods.toString())
        },
        size: savedSizes.expectedPods || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'PersistentVolumeClaim') {
    // Add PersistentVolumeClaim-specific columns
    baseColumns.push(
      {
        id: 'capacity',
        header: 'Capacity',
        accessorFn: (row) => getGenericStatus(row)?.capacity?.storage || '-',
        cell: ({ row }) => {
          const storage = getGenericStatus(row.original)?.capacity?.storage
          if (!storage) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, storage)
        },
        size: savedSizes.capacity || 80,
        minSize: 60,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'storageClassName',
        header: 'Storage Class',
        accessorFn: (row) => getGenericSpec(row)?.storageClassName || 'default',
        cell: ({ row }) => {
          const storageClass = getGenericSpec(row.original)?.storageClassName || 'default'
          return h('div', {
            class: 'text-sm table-cell-text',
            title: storageClass
          }, storageClass)
        },
        size: savedSizes.storageClassName || 120,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'Namespace') {
    // Add Namespace-specific columns
    baseColumns.push(
      {
        id: 'labels',
        header: 'Labels',
        accessorFn: (row) => {
          const labels = row.metadata?.labels
          if (!labels || Object.keys(labels).length === 0) return 0
          return Object.keys(labels).length
        },
        cell: ({ row }) => {
          const labels = row.original.metadata?.labels
          if (!labels || Object.keys(labels).length === 0) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          
          const labelCount = Object.keys(labels).length
          const labelText = Object.entries(labels)
            .slice(0, 2)
            .map(([key, value]) => `${key}=${value}`)
            .join(', ')
          
          const displayText = labelCount > 2 
            ? `${labelText} +${labelCount - 2}`
            : labelText
          
          return h('div', {
            class: 'text-xs table-cell-text truncate max-w-full',
            title: `Labels (${labelCount}): ${Object.entries(labels).map(([k, v]) => `${k}=${v}`).join(', ')}`
          }, displayText)
        },
        size: savedSizes.labels || 200,
        minSize: 150,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'Node') {
    // Add Node-specific columns
    baseColumns.push(
      {
        id: 'cpu',
        header: 'CPU',
        accessorFn: (row) => getGenericStatus(row)?.capacity?.cpu || '0',
        cell: ({ row }) => {
          const cpu = getGenericStatus(row.original)?.capacity?.cpu
          if (!cpu) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, cpu)
        },
        size: savedSizes.cpu || 80,
        minSize: 60,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'memory',
        header: 'Memory',
        accessorFn: (row) => {
          const memory = getGenericStatus(row)?.capacity?.memory
          if (!memory) return 0
          // Convert memory to bytes for sorting
          const match = memory.match(/^(\d+)(.*)$/)
          if (match) {
            const value = parseInt(match[1])
            const unit = match[2]
            switch (unit) {
              case 'Ki': return value * 1024
              case 'Mi': return value * 1024 * 1024
              case 'Gi': return value * 1024 * 1024 * 1024
              case 'Ti': return value * 1024 * 1024 * 1024 * 1024
              default: return value
            }
          }
          return 0
        },
        cell: ({ row }) => {
          const memory = getGenericStatus(row.original)?.capacity?.memory
          if (!memory) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          const formattedMemory = formatMemory(memory)
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, formattedMemory)
        },
        size: savedSizes.memory || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'allocatable-cpu',
        header: 'Allocatable CPU',
        accessorFn: (row) => getGenericStatus(row)?.allocatable?.cpu || '0',
        cell: ({ row }) => {
          const allocatableCpu = getGenericStatus(row.original)?.allocatable?.cpu
          if (!allocatableCpu) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, allocatableCpu)
        },
        size: savedSizes['allocatable-cpu'] || 90,
        minSize: 70,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'allocatable-memory',
        header: 'Allocatable Memory',
        accessorFn: (row) => {
          const memory = getGenericStatus(row)?.allocatable?.memory
          if (!memory) return 0
          // Convert memory to bytes for sorting
          const match = memory.match(/^(\d+)(.*)$/)
          if (match) {
            const value = parseInt(match[1])
            const unit = match[2]
            switch (unit) {
              case 'Ki': return value * 1024
              case 'Mi': return value * 1024 * 1024
              case 'Gi': return value * 1024 * 1024 * 1024
              case 'Ti': return value * 1024 * 1024 * 1024 * 1024
              default: return value
            }
          }
          return 0
        },
        cell: ({ row }) => {
          const memory = getGenericStatus(row.original)?.allocatable?.memory
          if (!memory) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          const formattedMemory = formatMemory(memory)
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, formattedMemory)
        },
        size: savedSizes['allocatable-memory'] || 120,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'allocatable-pods',
        header: 'Allocatable Pods',
        accessorFn: (row) => {
          const pods = getGenericStatus(row)?.allocatable?.pods
          return pods ? parseInt(pods) : 0
        },
        cell: ({ row }) => {
          const pods = getGenericStatus(row.original)?.allocatable?.pods
          if (!pods) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, pods)
        },
        size: savedSizes['allocatable-pods'] || 100,
        minSize: 80,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'architecture',
        header: 'Architecture',
        accessorFn: (row) => getGenericStatus(row)?.nodeInfo?.architecture || 'unknown',
        cell: ({ row }) => {
          const architecture = getGenericStatus(row.original)?.nodeInfo?.architecture
          if (!architecture) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text'
          }, architecture)
        },
        size: savedSizes.architecture || 90,
        minSize: 70,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'osImage',
        header: 'OS Image',
        accessorFn: (row) => getGenericStatus(row)?.nodeInfo?.osImage || 'unknown',
        cell: ({ row }) => {
          const osImage = getGenericStatus(row.original)?.nodeInfo?.osImage
          if (!osImage) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text truncate',
            title: osImage
          }, osImage)
        },
        size: savedSizes.osImage || 180,
        minSize: 120,
        enableSorting: true,
        enableResizing: true
      },
      {
        id: 'kubeletVersion',
        header: 'Kubelet Version',
        accessorFn: (row) => getGenericStatus(row)?.nodeInfo?.kubeletVersion || 'unknown',
        cell: ({ row }) => {
          const kubeletVersion = getGenericStatus(row.original)?.nodeInfo?.kubeletVersion
          if (!kubeletVersion) {
            return h('span', { class: 'text-text-muted text-xs' }, '-')
          }
          return h('div', {
            class: 'text-sm table-cell-text font-mono'
          }, kubeletVersion)
        },
        size: savedSizes.kubeletVersion || 120,
        minSize: 100,
        enableSorting: true,
        enableResizing: true
      }
    )
  } else if (props.resource?.kind === 'Job' || props.resource?.kind === 'CronJob' || props.resource?.kind === 'NetworkPolicy' || props.resource?.kind === 'IngressClass' || props.resource?.kind === 'Endpoints' || props.resource?.kind === 'ConfigMap' || props.resource?.kind === 'Secret' || props.resource?.kind === 'CSINode' || props.resource?.kind === 'CSIDriver') {
    // Jobs, CronJobs, NetworkPolicies, IngressClasses, Endpoints, ConfigMaps, Secrets, CSINodes, and CSIDrivers have their own custom columns or no Status column needed
  } else {
    // Add status column for resources without custom columns
    baseColumns.push(
      {
        id: 'status',
        header: 'Status',
        accessorFn: (row) => props.getStatusText(row),
        cell: ({ row }) => {
          const status = props.getStatusText(row.original)
          return h('div', {
            class: [
              'px-1.5 py-0 text-xs font-medium rounded-full truncate max-w-full inline-block',
              props.getStatusClass(row.original)
            ],
            title: status
          }, status)
        },
        size: savedSizes.status || 90,
        minSize: 50,
        enableSorting: true,
        enableResizing: true
      }
    )
  }

  baseColumns.push(
    {
      accessorKey: 'metadata.creationTimestamp',
      id: 'age',
      header: 'Age',
      cell: ({ getValue }) => {
        const timestamp = getValue() as string
        const age = props.getAge(timestamp)
        return h('div', {
          class: 'truncate max-w-full text-text-muted',
          title: timestamp ? new Date(timestamp).toLocaleString() : undefined
        }, age)
      },
      size: savedSizes.age || 80,
      minSize: 50,
      enableSorting: true,
      enableResizing: true
    }
  )

  return baseColumns
})

// Helper functions
function getJobDuration(durationMs: number): string {
  const seconds = Math.floor(durationMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) return `${days}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}

// Helper function to convert memory to appropriate binary units (Mi, Gi, Ti)
function formatMemory(memory: string): string {
  const match = memory.match(/^(\d+)(.*)$/)
  if (!match) return memory
  
  const value = parseInt(match[1])
  const unit = match[2]
  
  // Convert to bytes first
  let bytes = value
  switch (unit) {
    case 'Ki': bytes = value * 1024; break
    case 'Mi': bytes = value * 1024 * 1024; break
    case 'Gi': bytes = value * 1024 * 1024 * 1024; break
    case 'Ti': bytes = value * 1024 * 1024 * 1024 * 1024; break
    default: bytes = value; break
  }
  
  // Convert to appropriate unit based on 1024 boundaries
  if (bytes >= 1024 * 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024 * 1024)).toFixed(1)}Ti`
  } else if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}Gi`
  } else if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)}Mi`
  } else if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)}Ki`
  } else {
    return `${bytes}B`
  }
}

function createHoverButtons(item: K8sListItem) {
  const buttons = []
  
  // Pod-specific buttons
  if (props.resource?.kind === 'Pod') {
    buttons.push(
      h('button', {
        onClick: (e: Event) => {
          e.stopPropagation()
          emit('openPodLogs', item)
        },
        class: 'w-6 h-6 rounded-full bg-blue-500 hover:bg-blue-600 text-white flex items-center justify-center transition-colors duration-200',
        title: 'View Pod Logs'
      }, [
        h('svg', {
          class: 'w-3 h-3',
          fill: 'none',
          stroke: 'currentColor',
          viewBox: '0 0 24 24'
        }, [
          h('path', {
            'stroke-linecap': 'round',
            'stroke-linejoin': 'round',
            'stroke-width': '2',
            d: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z'
          })
        ])
      ]),
      h('button', {
        onClick: (e: Event) => {
          e.stopPropagation()
          emit('openPodShell', item)
        },
        class: 'w-6 h-6 rounded-full bg-green-500 hover:bg-green-600 text-white flex items-center justify-center transition-colors duration-200',
        title: 'Open Shell'
      }, [
        h('svg', {
          class: 'w-3 h-3',
          fill: 'none',
          stroke: 'currentColor',
          viewBox: '0 0 24 24'
        }, [
          h('path', {
            'stroke-linecap': 'round',
            'stroke-linejoin': 'round',
            'stroke-width': '2',
            d: 'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z'
          })
        ])
      ])
    )
  }

  // Delete button
  buttons.push(
    h('button', {
      onClick: (e: Event) => {
        e.stopPropagation()
        emit('deleteResource', item)
      },
      class: 'w-6 h-6 rounded-full bg-red-500 hover:bg-red-600 text-white flex items-center justify-center transition-colors duration-200',
      title: `Delete ${props.resource?.kind || 'Resource'}`
    }, [
      h('svg', {
        class: 'w-3 h-3',
        fill: 'none',
        stroke: 'currentColor',
        viewBox: '0 0 24 24'
      }, [
        h('path', {
          'stroke-linecap': 'round',
          'stroke-linejoin': 'round',
          'stroke-width': '2',
          d: 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16'
        })
      ])
    ])
  )

  return buttons
}

function createContainersCell(item: K8sListItem) {
  // Backend sends camelCase field names as per k8s-openapi serialization
  const status = getGenericStatus(item) as any
  const initContainerStatuses = status?.initContainerStatuses || []
  const containerStatuses = status?.containerStatuses || []
  
  return h('div', { class: 'flex gap-1 items-center' }, [
    // Init containers
    ...(initContainerStatuses?.length ? [
      h('div', { class: 'flex gap-0.5 pr-1 border-r border-gray-300' }, 
        initContainerStatuses.map((container: any, index: number) =>
          h('div', {
            key: `init-${index}`,
            onMouseenter: (e: MouseEvent) => {
              const lines = getContainerTooltipLines(container, true)
              showTooltip(e, lines)
            },
            onMouseleave: hideTooltip,
            class: [
              'w-2.5 h-2.5 rounded-full border border-gray-400 cursor-pointer',
              props.getContainerStatusColor(container)
            ]
          })
        )
      )
    ] : []),
    // Regular containers
    h('div', { class: 'flex gap-0.5' },
      containerStatuses.map((container: any, index: number) =>
        h('div', {
          key: `container-${index}`,
          onMouseenter: (e: MouseEvent) => {
            const lines = getContainerTooltipLines(container, false)
            showTooltip(e, lines)
          },
          onMouseleave: hideTooltip,
          class: [
            'w-2.5 h-2.5 border border-gray-400 cursor-pointer',
            props.getContainerStatusColor(container)
          ]
        })
      )
    )
  ])
}

// Initialize column sizing from localStorage
columnSizing.value = loadColumnSizes()

// Create table instance
const table = useVueTable({
  get data() { return props.items },
  get columns() { return columns.value },
  state: {
    get sorting() { return sorting.value },
    get columnSizing() { return columnSizing.value }
  },
  onSortingChange: (updater) => {
    sorting.value = typeof updater === 'function' ? updater(sorting.value) : updater
  },
  onColumnSizingChange: (updater) => {
    columnSizing.value = typeof updater === 'function' ? updater(columnSizing.value) : updater
    saveColumnSizes(columnSizing.value)
  },
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  columnResizeMode: columnResizeMode.value,
  enableColumnResizing: true
})

function forceClearHoveredRow(reason: string) {
  emit('clearHoveredRow', reason)
}

</script>