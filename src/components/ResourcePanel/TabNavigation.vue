<template>
  <div class="flex-none border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
    <nav class="flex space-x-0">
      <button v-for="tab in tabs" :key="tab.id"
              @click="$emit('update:modelValue', tab.id)"
              :class="[
                'px-6 py-3 text-sm font-medium border-b-2 transition-colors',
                modelValue === tab.id
                  ? 'border-blue-500 dark:border-blue-400 text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/30'
                  : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700'
              ]">
        <div class="flex items-center space-x-2">
          <component :is="getIconComponent(tab.icon)" class="w-4 h-4" />
          <span>{{ tab.label }}</span>
        </div>
      </button>
    </nav>
  </div>
</template>

<script setup lang="ts">
interface Tab {
  id: string
  label: string
  icon: string
}

interface Props {
  modelValue: string
  tabs: Tab[]
}

defineProps<Props>()

defineEmits<{
  'update:modelValue': [value: string]
}>()

function getIconComponent(iconName: string) {
  const icons: Record<string, any> = {
    InfoIcon: {
      template: `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4"/>
          <path d="M12 8h.01"/>
        </svg>
      `
    },
    CodeIcon: {
      template: `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M16 18l6-6-6-6"/>
          <path d="M8 6l-6 6 6 6"/>
        </svg>
      `
    },
    LogIcon: {
      template: `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
          <path d="M14 2v6h6"/>
          <path d="M16 13H8"/>
          <path d="M16 17H8"/>
          <path d="M10 9H8"/>
        </svg>
      `
    },
    EventIcon: {
      template: `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
        </svg>
      `
    }
  }
  
  return icons[iconName] || icons.InfoIcon
}
</script>