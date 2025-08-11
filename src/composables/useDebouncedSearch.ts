import { ref, computed, watch } from 'vue'
import type { Ref } from 'vue'

export function useDebouncedSearch(delay = 300) {
  const searchQuery = ref('')
  const debouncedQuery = ref('')
  let timeoutId: NodeJS.Timeout | null = null

  watch(searchQuery, (newValue) => {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
    
    timeoutId = setTimeout(() => {
      debouncedQuery.value = newValue
      timeoutId = null
    }, delay)
  })

  const clearSearch = () => {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
    }
    searchQuery.value = ''
    debouncedQuery.value = ''
  }

  return {
    searchQuery,
    debouncedQuery: computed(() => debouncedQuery.value),
    clearSearch
  }
}