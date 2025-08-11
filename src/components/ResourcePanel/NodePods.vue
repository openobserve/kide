<template>
  <div class="h-full overflow-y-auto p-6 space-y-6">
    <!-- Loading State -->
    <div v-if="loading" class="flex items-center justify-center h-32">
      <div class="text-center">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-2"></div>
        <p class="text-sm text-gray-600 dark:text-gray-400">Loading pods on node...</p>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="flex items-center justify-center h-32">
      <div class="text-center">
        <div class="text-red-500 mb-2">
          <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
        </div>
        <p class="text-sm text-red-600 dark:text-red-400">{{ error }}</p>
        <button 
          @click="fetchNodePods" 
          class="mt-2 px-3 py-1 bg-blue-500 text-white text-sm rounded hover:bg-blue-600"
        >
          Retry
        </button>
      </div>
    </div>

    <!-- Node Info Header -->
    <div v-else-if="nodeName" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
      <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">Node: {{ nodeName }}</h3>
      <p class="text-xs text-gray-600 dark:text-gray-400 mb-3">{{ pods.length }} pod{{ pods.length !== 1 ? 's' : '' }} running on this node</p>
    </div>

    <!-- Pods Table -->
    <div v-if="!loading && !error && pods.length > 0" class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-600 overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full bg-white dark:bg-gray-800" style="table-layout: fixed;" :style="{ width: table.getTotalSize() + 'px' }">
          <!-- Table Header -->
          <thead class="bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
            <tr v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
              <th
                v-for="header in headerGroup.headers"
                :key="header.id"
                :colSpan="header.colSpan"
                :style="{ width: header.getSize() + 'px' }"
                class="px-3 py-1 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider relative select-none"
                :class="[
                  header.column.getCanSort() ? 'cursor-pointer hover:text-gray-700 dark:hover:text-gray-300 group' : '',
                ]"
                @click="header.column.getToggleSortingHandler()?.($event)"
              >
                <div class="flex items-center space-x-1">
                  <FlexRender
                    :render="header.column.columnDef.header"
                    :props="header.getContext()"
                  />
                  <!-- Sort indicator -->
                  <div v-if="header.column.getCanSort()" class="w-3 h-3">
                    <svg v-if="header.column.getIsSorted() === 'asc'" 
                         class="w-3 h-3 text-gray-500" 
                         fill="currentColor" 
                         viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z" clip-rule="evenodd"/>
                    </svg>
                    <svg v-else-if="header.column.getIsSorted() === 'desc'" 
                         class="w-3 h-3 text-gray-500" 
                         fill="currentColor" 
                         viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
                    </svg>
                    <svg v-else 
                         class="w-3 h-3 text-gray-300 opacity-0 group-hover:opacity-100" 
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
          <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
            <tr
              v-for="row in table.getRowModel().rows"
              :key="row.id"
              class="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer transition-colors"
              @click="handleViewPod(row.original)"
            >
              <td
                v-for="cell in row.getVisibleCells()"
                :key="cell.id"
                :style="{ width: cell.column.getSize() + 'px' }"
                class="px-3 py-0.5 whitespace-nowrap text-sm overflow-hidden text-ellipsis text-gray-600 dark:text-gray-400"
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
    </div>

    <!-- Empty State -->
    <div v-else-if="!loading && !error && pods.length === 0" class="flex items-center justify-center h-32">
      <div class="text-center">
        <div class="text-gray-400 dark:text-gray-500 mb-2">
          <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2 2v-5m16 0h-2M4 13h2m0 0V9a2 2 0 012-2h2m0 0V6a2 2 0 012-2h2a2 2 0 012 2v1M6 7h.01M6 11h.01"/>
          </svg>
        </div>
        <p class="text-sm text-gray-500 dark:text-gray-400">No pods found on this node</p>
      </div>
    </div>

    <!-- Not a Node Resource -->
    <div v-else-if="!loading && !error && !nodeName" class="flex items-center justify-center h-32">
      <div class="text-center">
        <div class="text-gray-400 dark:text-gray-500 mb-2">
          <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
        </div>
        <p class="text-sm text-gray-500 dark:text-gray-400">Node details are only available for Node resources</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  getCoreRowModel,
  getSortedRowModel,
  useVueTable,
  FlexRender,
  type ColumnDef,
  type SortingState,
  type ColumnResizeMode
} from '@tanstack/vue-table'
import { useResourceStatus } from '@/composables/useResourceStatus'

interface Props {
  resourceData: any | null
  resourceKind: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  viewPod: [pod: any]
}>()

// Use centralized status logic
const { getStatusText, getStatusClass, getGenericStatus, getGenericSpec } = useResourceStatus()

// State
const pods = ref<any[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Table state
const sorting = ref<SortingState>([])
const columnSizing = ref<Record<string, number>>({})
const columnResizeMode = ref<ColumnResizeMode>('onChange')

// Computed
const nodeName = computed(() => {
  if (props.resourceKind === 'Node' && props.resourceData?.metadata?.name) {
    return props.resourceData.metadata.name
  }
  return null
})

// Column definitions
const columns = computed((): ColumnDef<any>[] => {
  const savedSizes = loadColumnSizes()
  
  return [
    {
      accessorKey: 'metadata.name',
      id: 'name',
      header: 'Name',
      cell: ({ getValue }) => {
        const name = getValue() as string || 'Unknown'
        return h('div', {
          class: 'text-sm text-gray-900 dark:text-gray-100 truncate',
          title: name
        }, name)
      },
      size: savedSizes.name || 200,
      minSize: 100,
      enableSorting: true,
      enableResizing: true
    },
    {
      accessorKey: 'metadata.namespace',
      id: 'namespace',
      header: 'Namespace',
      cell: ({ getValue }) => {
        const namespace = getValue() as string || 'default'
        return h('span', {
          class: 'inline-flex items-center px-1.5 py-0 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-xs rounded-full'
        }, namespace)
      },
      size: savedSizes.namespace || 120,
      minSize: 80,
      enableSorting: true,
      enableResizing: true
    },
    {
      id: 'status',
      header: 'Status',
      cell: ({ row }) => {
        const status = getStatusText(row.original)
        return h('span', {
          class: [
            'inline-flex items-center px-1.5 py-0 rounded-full text-xs font-medium',
            getStatusClass(row.original)
          ]
        }, status)
      },
      size: savedSizes.status || 90,
      minSize: 70,
      enableSorting: false,
      enableResizing: true
    },
    {
      id: 'containers',
      header: 'Containers',
      cell: ({ row }) => {
        const containerStatuses = getGenericStatus(row.original)?.containerStatuses || []
        return h('div', { class: 'flex items-center space-x-1' }, [
          ...containerStatuses.map((container: any, index: number) =>
            h('div', {
              key: `container-${index}`,
              class: 'flex items-center space-x-0.5',
              title: `${container.name}: ${getContainerStatusText(container)}`
            }, [
              h('div', {
                class: [
                  'w-2 h-2 rounded-full',
                  getContainerStatusColor(container)
                ]
              })
            ])
          ),
          h('span', {
            class: 'text-xs text-gray-500 dark:text-gray-400 ml-1'
          }, containerStatuses.length.toString())
        ])
      },
      size: savedSizes.containers || 100,
      minSize: 80,
      enableSorting: false,
      enableResizing: true
    },
    {
      id: 'restarts',
      header: 'Restarts',
      cell: ({ row }) => {
        const restarts = getTotalRestarts(row.original)
        return h('div', {
          class: 'text-sm text-gray-600 dark:text-gray-400'
        }, restarts.toString())
      },
      size: savedSizes.restarts || 80,
      minSize: 60,
      enableSorting: true,
      enableResizing: true
    },
    {
      id: 'cpu',
      header: 'CPU',
      cell: ({ row }) => {
        const cpu = getCpuRequests(row.original)
        return h('div', {
          class: 'text-sm text-gray-600 dark:text-gray-400'
        }, cpu)
      },
      size: savedSizes.cpu || 80,
      minSize: 60,
      enableSorting: false,
      enableResizing: true
    },
    {
      id: 'memory',
      header: 'Memory',
      cell: ({ row }) => {
        const memory = getMemoryRequests(row.original)
        return h('div', {
          class: 'text-sm text-gray-600 dark:text-gray-400'
        }, memory)
      },
      size: savedSizes.memory || 80,
      minSize: 60,
      enableSorting: false,
      enableResizing: true
    },
    {
      accessorKey: 'status.podIP',
      id: 'podIP',
      header: 'Pod IP',
      cell: ({ getValue }) => {
        const podIP = getValue() as string
        if (podIP) {
          return h('div', {
            class: 'text-sm text-gray-600 dark:text-gray-400 font-mono',
            title: podIP
          }, podIP)
        }
        return h('span', {
          class: 'text-gray-400 dark:text-gray-500 text-sm'
        }, '-')
      },
      size: savedSizes.podIP || 120,
      minSize: 100,
      enableSorting: true,
      enableResizing: true
    },
    {
      accessorKey: 'metadata.creationTimestamp',
      id: 'age',
      header: 'Age',
      cell: ({ getValue, row }) => {
        const timestamp = getValue() as string
        const age = getAge(timestamp)
        return h('div', {
          class: 'text-sm text-gray-500 dark:text-gray-400',
          title: timestamp ? new Date(timestamp).toLocaleString() : undefined
        }, age)
      },
      size: savedSizes.age || 80,
      minSize: 60,
      enableSorting: true,
      enableResizing: true
    }
  ]
})

// LocalStorage functions for column sizes
const storageKey = computed(() => `kide-node-pods-column-sizes`)

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

// Initialize column sizing from localStorage
columnSizing.value = loadColumnSizes()

// Create table instance
const table = useVueTable({
  get data() { return pods.value },
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

// Methods
async function fetchNodePods(): Promise<void> {
  if (!nodeName.value) return

  loading.value = true
  error.value = null

  try {
    const podList = await invoke<any>('get_node_pods', {
      nodeName: nodeName.value
    })
    
    pods.value = podList.items || []
  } catch (err: any) {
    console.error('Error fetching node pods:', err)
    error.value = err.message || 'Failed to fetch pods for this node'
    pods.value = []
  } finally {
    loading.value = false
  }
}

function handleViewPod(pod: any): void {
  emit('viewPod', pod)
}


function getContainerStatusColor(container: any): string {
  if (container.state?.running) return 'bg-green-500'
  if (container.state?.waiting) return 'bg-yellow-500'
  if (container.state?.terminated) return 'bg-red-500'
  return 'bg-gray-500'
}

function getContainerStatusText(container: any): string {
  if (container.state?.running) return 'Running'
  if (container.state?.waiting) return container.state.waiting.reason || 'Waiting'
  if (container.state?.terminated) return container.state.terminated.reason || 'Terminated'
  return 'Unknown'
}

function getTotalRestarts(pod: any): number {
  if (!getGenericStatus(pod)?.containerStatuses) return 0
  return getGenericStatus(pod).containerStatuses.reduce((total: number, container: any) => {
    return total + (container.restartCount || 0)
  }, 0)
}

function getAge(timestamp?: string): string {
  if (!timestamp) return '-'
  
  const now = new Date()
  const created = new Date(timestamp)
  const diffMs = now.getTime() - created.getTime()
  
  const days = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60))
  
  if (days > 0) return `${days}d`
  if (hours > 0) return `${hours}h`
  return `${minutes}m`
}

function getCpuRequests(pod: any): string {
  let totalCpu = 0
  const containers = getGenericSpec(pod)?.containers || []
  
  containers.forEach((container: any) => {
    const cpu = container.resources?.requests?.cpu
    if (cpu) {
      // Simple parsing for common CPU formats
      if (cpu.endsWith('m')) {
        totalCpu += parseInt(cpu.slice(0, -1))
      } else {
        totalCpu += parseInt(cpu) * 1000
      }
    }
  })
  
  return totalCpu > 0 ? `${totalCpu}m` : '-'
}

function getMemoryRequests(pod: any): string {
  let totalMemory = 0
  const containers = getGenericSpec(pod)?.containers || []
  
  containers.forEach((container: any) => {
    const memory = container.resources?.requests?.memory
    if (memory) {
      // Simple parsing for memory (assume Mi for now)
      if (memory.endsWith('Mi')) {
        totalMemory += parseInt(memory.slice(0, -2))
      } else if (memory.endsWith('Gi')) {
        totalMemory += parseInt(memory.slice(0, -2)) * 1024
      }
    }
  })
  
  return totalMemory > 0 ? `${totalMemory}Mi` : '-'
}

// Watch for resource changes
watch(() => props.resourceData, (newResource) => {
  if (newResource && props.resourceKind === 'Node') {
    fetchNodePods()
  }
}, { immediate: true })

// Initialize
onMounted(() => {
  if (props.resourceData && props.resourceKind === 'Node') {
    fetchNodePods()
  }
})
</script>