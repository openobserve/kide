<template>
  <div 
    ref="triggerRef"
    class="relative inline-block group"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- Trigger slot -->
    <slot />
    
    <!-- Tooltip content with Teleport to body -->
    <Teleport to="body">
      <div
        v-if="show"
        ref="tooltipRef"
        class="fixed z-[9999] px-2 py-1 text-xs text-white bg-gray-900 dark:bg-gray-700 rounded border shadow-lg whitespace-nowrap pointer-events-none"
        :style="{ left: position.x + 'px', top: position.y + 'px', transform: position.transform }"
      >
        <slot name="content">{{ content }}</slot>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'

interface Props {
  content?: string
  side?: 'top' | 'bottom' | 'left' | 'right'
  offset?: number
}

const props = withDefaults(defineProps<Props>(), {
  side: 'right',
  offset: 8
})

const show = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const tooltipRef = ref<HTMLElement | null>(null)
const position = ref({ x: 0, y: 0, transform: '' })

const updatePosition = async () => {
  if (!triggerRef.value || !show.value) return
  
  await nextTick()
  
  const triggerRect = triggerRef.value.getBoundingClientRect()
  const { side, offset } = props
  
  let x = 0
  let y = 0
  let transform = ''
  
  switch (side) {
    case 'top':
      x = triggerRect.left + triggerRect.width / 2
      y = triggerRect.top - offset
      transform = 'translateX(-50%) translateY(-100%)'
      break
    case 'bottom':
      x = triggerRect.left + triggerRect.width / 2
      y = triggerRect.bottom + offset
      transform = 'translateX(-50%)'
      break
    case 'left':
      x = triggerRect.left - offset
      y = triggerRect.top + triggerRect.height / 2
      transform = 'translateX(-100%) translateY(-50%)'
      break
    case 'right':
      x = triggerRect.right + offset
      y = triggerRect.top + triggerRect.height / 2
      transform = 'translateY(-50%)'
      break
  }
  
  position.value = { x, y, transform }
}

const handleMouseEnter = () => {
  show.value = true
  updatePosition()
}

const handleMouseLeave = () => {
  show.value = false
}

onMounted(() => {
  window.addEventListener('scroll', updatePosition)
  window.addEventListener('resize', updatePosition)
})

onBeforeUnmount(() => {
  window.removeEventListener('scroll', updatePosition)
  window.removeEventListener('resize', updatePosition)
})
</script>