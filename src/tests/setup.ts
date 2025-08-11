import { vi } from 'vitest'
import { config } from '@vue/test-utils'

// Global test setup

// Mock window.reportError
Object.defineProperty(window, 'reportError', {
  value: vi.fn(),
  writable: true
})

// Mock matchMedia for XTerm compatibility
Object.defineProperty(window, 'matchMedia', {
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
  writable: true,
})

Object.defineProperty(window, 'reportApiError', {
  value: vi.fn(),
  writable: true
})

// Mock IntersectionObserver for components that might use it
global.IntersectionObserver = vi.fn().mockImplementation((callback, options) => ({
  root: null,
  rootMargin: '0px',
  thresholds: [0],
  disconnect: vi.fn(),
  observe: vi.fn(),
  unobserve: vi.fn(),
  takeRecords: vi.fn(() => []),
}))

// Mock ResizeObserver
global.ResizeObserver = vi.fn(() => ({
  disconnect: vi.fn(),
  observe: vi.fn(),
  unobserve: vi.fn(),
}))

// Mock HTMLCanvasElement.getContext for XTerm compatibility
HTMLCanvasElement.prototype.getContext = vi.fn().mockImplementation((contextType) => {
  if (contextType === '2d') {
    return {
      fillStyle: '',
      strokeStyle: '',
      lineWidth: 1,
      font: '10px sans-serif',
      textAlign: 'start',
      textBaseline: 'alphabetic',
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      clearRect: vi.fn(),
      fillText: vi.fn(),
      strokeText: vi.fn(),
      measureText: vi.fn(() => ({ width: 0 })),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      arc: vi.fn(),
      rect: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      translate: vi.fn(),
      rotate: vi.fn(),
      scale: vi.fn(),
      setTransform: vi.fn(),
      createImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4), width: 1, height: 1 })),
      getImageData: vi.fn(() => ({ data: new Uint8ClampedArray(4), width: 1, height: 1 })),
      putImageData: vi.fn(),
      createLinearGradient: vi.fn(() => ({
        addColorStop: vi.fn()
      })),
      createRadialGradient: vi.fn(() => ({
        addColorStop: vi.fn()
      })),
      createPattern: vi.fn(() => null),
    }
  }
  return null
})

// Set up Vue Test Utils global config
config.global.stubs = {
  // Stub out components that might cause issues in tests
  transition: false,
  'transition-group': false
}

// Mock CSS modules and other asset imports
vi.mock('*.css', () => ({}))
vi.mock('*.scss', () => ({}))
vi.mock('*.svg', () => 'svg-mock')
vi.mock('*.png', () => 'png-mock')
vi.mock('*.jpg', () => 'jpg-mock')

// Mock Monaco Editor and our custom setup
vi.mock('@/utils/monaco-setup', async () => {
  const monaco = {
    editor: {
      create: vi.fn(() => ({
        dispose: vi.fn(),
        setValue: vi.fn(),
        getValue: vi.fn(() => ''),
        onDidChangeModelContent: vi.fn(() => ({ dispose: vi.fn() })),
        layout: vi.fn(),
        focus: vi.fn(),
        getModel: vi.fn(() => ({
          pushEditOperations: vi.fn(),
          setValue: vi.fn(),
          getValue: vi.fn(() => ''),
        })),
      })),
      setTheme: vi.fn(),
      defineTheme: vi.fn(),
      getModels: vi.fn(() => []),
      createModel: vi.fn(() => ({
        dispose: vi.fn(),
        setValue: vi.fn(),
        getValue: vi.fn(() => ''),
      })),
    },
    languages: {
      yaml: {},
      json: {},
      register: vi.fn(),
      setLanguageConfiguration: vi.fn(),
      setMonarchTokensProvider: vi.fn(),
    },
    Range: vi.fn(),
    Selection: vi.fn(),
  }
  return { monaco, default: monaco }
})

vi.mock('monaco-editor', () => ({
  editor: {
    create: vi.fn(() => ({
      dispose: vi.fn(),
      setValue: vi.fn(),
      getValue: vi.fn(() => ''),
      onDidChangeModelContent: vi.fn(() => ({ dispose: vi.fn() })),
      layout: vi.fn(),
      focus: vi.fn(),
      getModel: vi.fn(() => ({
        pushEditOperations: vi.fn(),
        setValue: vi.fn(),
        getValue: vi.fn(() => ''),
      })),
    })),
    setTheme: vi.fn(),
    defineTheme: vi.fn(),
    getModels: vi.fn(() => []),
    createModel: vi.fn(() => ({
      dispose: vi.fn(),
      setValue: vi.fn(),
      getValue: vi.fn(() => ''),
    })),
  },
  languages: {
    yaml: {},
    register: vi.fn(),
    setLanguageConfiguration: vi.fn(),
    setMonarchTokensProvider: vi.fn(),
  },
  Range: vi.fn(),
  Selection: vi.fn(),
}))

// Suppress console warnings during tests unless explicitly testing them
const originalWarn = console.warn
console.warn = vi.fn((...args) => {
  // Only show Vue warnings that we care about
  const message = args[0]
  if (typeof message === 'string' && !message.includes('[Vue warn]')) {
    originalWarn.apply(console, args)
  }
})

// Mock Date.now for consistent timestamp testing (disabled by default to avoid timer conflicts)
// Individual tests can enable this if needed with vi.setSystemTime()
// const mockDate = new Date('2024-01-01T00:00:00.000Z')
// vi.setSystemTime(mockDate)