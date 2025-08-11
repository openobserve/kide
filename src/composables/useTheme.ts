import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark'

const theme = ref<Theme>('dark') // Default to dark theme

export function useTheme() {
  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme
    
    // Apply theme to document class
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
    
    // Save to localStorage
    localStorage.setItem('kide-theme', newTheme)
  }

  const toggleTheme = () => {
    setTheme(theme.value === 'dark' ? 'light' : 'dark')
  }

  const initTheme = () => {
    // Try to get theme from localStorage, fallback to dark
    const savedTheme = localStorage.getItem('kide-theme') as Theme
    const initialTheme = savedTheme || 'dark'
    setTheme(initialTheme)
  }

  // Watch for theme changes and apply to document
  watch(theme, (newTheme) => {
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, { immediate: true })

  return {
    theme: readonly(theme),
    setTheme,
    toggleTheme,
    initTheme
  }
}

// Make theme reactive globally
function readonly<T>(ref: { readonly value: T }) {
  return {
    get value() {
      return ref.value
    }
  }
}