<!-- TitleBar.vue - Custom title bar with cluster dropdown -->
<template>
  <div class="title-bar-container bg-surface-primary border-b border-border-primary" data-tauri-drag-region>
    <div class="flex items-center justify-between h-8 pl-20 pr-4">
      <!-- Left side: Only cluster dropdown -->
      <div class="flex items-center flex-1">
        <!-- Cluster dropdown -->
        <div class="flex-1 max-w-sm">
          <ClusterDropdown 
            :selectedContext="selectedContext"
            :connectionStatus="connectionStatus"
            @context-selected="$emit('context-selected', $event)"
            @refresh-contexts="$emit('refresh-contexts')"
          />
        </div>
      </div>

      <!-- Right side: Window controls placeholder -->
      <div class="flex-shrink-0 w-20">
        <!-- Space reserved for native window controls -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ClusterDropdown from './ClusterDropdown.vue'
import type { K8sContext, ContextStatus } from '@/types'

interface Props {
  selectedContext?: K8sContext | null
  connectionStatus?: Record<string, ContextStatus>
}

interface Emits {
  (e: 'context-selected', context: K8sContext): void
  (e: 'refresh-contexts'): void
}

const props = withDefaults(defineProps<Props>(), {
  selectedContext: null,
  connectionStatus: () => ({})
})

defineEmits<Emits>()
</script>

<style scoped>
.title-bar-container {
  /* Ensure title bar is above other content */
  z-index: 1000;
  
  /* Make the entire title bar draggable except for interactive elements */
  -webkit-app-region: drag;
  app-region: drag;
}

/* Make interactive elements non-draggable */
.title-bar-container button,
.title-bar-container input,
.title-bar-container select,
.title-bar-container [role="button"] {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

/* Ensure dropdown menus are also non-draggable */
.title-bar-container [data-dropdown],
.title-bar-container [data-dropdown] * {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
</style>