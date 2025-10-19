<template>
  <div v-if="isVisible" class="loading-progress-bar">
    <div class="progress-container">
      <div class="progress-info">
        <span class="progress-text">
          {{ progressText }}
        </span>
        <span class="progress-percentage">
          {{ progress }}%
        </span>
      </div>
      <div class="progress-bar">
        <div 
          class="progress-fill" 
          :style="{ width: `${progress}%` }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// Progress state
const isVisible = ref(false)
const progress = ref(0)
const current = ref(0)
const total = ref(0)
const currentResource = ref('')
const status = ref('')

// Event listener cleanup
let unlisten: UnlistenFn | null = null

// Computed text for progress display
const progressText = computed(() => {
  if (status.value === 'loading' && currentResource.value) {
    return `Loading ${currentResource.value}... (${current.value}/${total.value})`
  } else if (status.value === 'completed' && currentResource.value) {
    return `Completed ${currentResource.value} (${current.value}/${total.value})`
  } else if (status.value === 'skipped') {
    return `Skipping (${current.value}/${total.value})`
  } else if (total.value > 0) {
    return `Background loading... (${current.value}/${total.value})`
  }
  return 'Initializing...'
})

// Auto-hide timer
let hideTimer: NodeJS.Timeout | null = null

const hideProgressBar = () => {
  if (hideTimer) {
    clearTimeout(hideTimer)
  }
  hideTimer = setTimeout(() => {
    isVisible.value = false
    // Reset state
    progress.value = 0
    current.value = 0
    total.value = 0
    currentResource.value = ''
    status.value = ''
  }, 2000) // Hide after 2 seconds
}

const showProgressBar = () => {
  isVisible.value = true
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }
}

onMounted(async () => {
  // Listen for background loading progress events
  unlisten = await listen('background-loading-progress', (event) => {
    const data = event.payload as {
      current: number
      total: number
      progress: number
      current_resource: string
      status: string
    }
    
    // Update progress state
    current.value = data.current
    total.value = data.total
    progress.value = data.progress
    currentResource.value = data.current_resource
    status.value = data.status
    
    // Show progress bar when we receive events
    showProgressBar()
    
    // If progress is complete, start hide timer
    if (data.progress >= 100) {
      hideProgressBar()
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (hideTimer) {
    clearTimeout(hideTimer)
  }
})
</script>

<style scoped>
.loading-progress-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(255, 255, 255, 0.95);
  border-top: 1px solid #e0e0e0;
  backdrop-filter: blur(8px);
  z-index: 1000;
  transition: all 0.3s ease;
}

.progress-container {
  padding: 12px 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 13px;
}

.progress-text {
  color: #555;
  font-weight: 500;
}

.progress-percentage {
  color: #2196F3;
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: #f0f0f0;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #2196F3, #21CBF3);
  border-radius: 2px;
  transition: width 0.3s ease;
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.3),
    transparent
  );
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .loading-progress-bar {
    background: rgba(30, 30, 30, 0.95);
    border-top-color: #404040;
  }
  
  .progress-text {
    color: #ccc;
  }
  
  .progress-bar {
    background: #333;
  }
}
</style>