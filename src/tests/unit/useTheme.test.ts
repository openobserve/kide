import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useTheme } from '../../composables/useTheme'

// Mock localStorage
const mockLocalStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  clear: vi.fn()
}

// Mock document
const mockDocument = {
  documentElement: {
    classList: {
      add: vi.fn(),
      remove: vi.fn()
    }
  }
}

Object.defineProperty(global, 'localStorage', {
  value: mockLocalStorage,
  writable: true
})

Object.defineProperty(global, 'document', {
  value: mockDocument,
  writable: true
})

describe('useTheme', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should initialize with dark theme as default', () => {
    mockLocalStorage.getItem.mockReturnValue(null)
    
    const { theme, initTheme } = useTheme()
    initTheme()
    
    expect(theme.value).toBe('dark')
    expect(mockDocument.documentElement.classList.add).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'dark')
  })

  it('should load saved theme from localStorage', () => {
    mockLocalStorage.getItem.mockReturnValue('light')
    
    const { theme, initTheme } = useTheme()
    initTheme()
    
    expect(theme.value).toBe('light')
    expect(mockDocument.documentElement.classList.remove).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'light')
  })

  it('should toggle theme from dark to light', () => {
    mockLocalStorage.getItem.mockReturnValue('dark')
    
    const { theme, toggleTheme, initTheme } = useTheme()
    initTheme()
    
    expect(theme.value).toBe('dark')
    
    toggleTheme()
    
    expect(theme.value).toBe('light')
    expect(mockDocument.documentElement.classList.remove).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'light')
  })

  it('should toggle theme from light to dark', () => {
    mockLocalStorage.getItem.mockReturnValue('light')
    
    const { theme, toggleTheme, initTheme } = useTheme()
    initTheme()
    
    expect(theme.value).toBe('light')
    
    toggleTheme()
    
    expect(theme.value).toBe('dark')
    expect(mockDocument.documentElement.classList.add).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'dark')
  })

  it('should set theme directly', () => {
    const { theme, setTheme } = useTheme()
    
    setTheme('light')
    
    expect(theme.value).toBe('light')
    expect(mockDocument.documentElement.classList.remove).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'light')
  })

  it('should apply dark class for dark theme', () => {
    const { setTheme } = useTheme()
    
    setTheme('dark')
    
    expect(mockDocument.documentElement.classList.add).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'dark')
  })

  it('should remove dark class for light theme', () => {
    const { setTheme } = useTheme()
    
    setTheme('light')
    
    expect(mockDocument.documentElement.classList.remove).toHaveBeenCalledWith('dark')
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('kide-theme', 'light')
  })
})