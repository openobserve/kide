<template>
  <!-- Delete Confirmation Dialog -->
  <div 
    v-if="show"
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
    @click="$emit('cancel')"
  >
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 p-6"
      @click.stop
    >
      <div class="flex items-center mb-4">
        <div class="flex-shrink-0">
          <svg class="w-10 h-10 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <div class="ml-4">
          <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">
            Delete Resources
          </h3>
          <div class="mt-2 text-sm text-gray-500 dark:text-gray-400">
            Are you sure you want to delete {{ selectedCount }} selected resource{{ selectedCount > 1 ? 's' : '' }}? This action cannot be undone.
          </div>
        </div>
      </div>
      
      <!-- Selected resources list -->
      <div class="mb-4 max-h-32 overflow-y-auto bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
        <div class="space-y-1">
          <div 
            v-for="item in selectedResources"
            :key="item.metadata?.uid"
            class="text-sm font-mono text-gray-700 dark:text-gray-300"
          >
            {{ item.metadata?.namespace ? `${item.metadata.namespace}/` : '' }}{{ item.metadata?.name || 'Unknown' }}
          </div>
        </div>
      </div>
      
      <div class="flex justify-end space-x-3">
        <button
          @click="$emit('cancel')"
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-200 dark:bg-gray-600 hover:bg-gray-300 dark:hover:bg-gray-500 rounded-md transition-colors"
        >
          Cancel
        </button>
        <button
          @click="$emit('confirm')"
          class="btn-danger text-sm"
        >
          Delete {{ selectedCount }} Resource{{ selectedCount > 1 ? 's' : '' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  show: boolean
  selectedCount: number
  selectedResources: any[]
}

defineProps<Props>()

defineEmits<{
  cancel: []
  confirm: []
}>()
</script>