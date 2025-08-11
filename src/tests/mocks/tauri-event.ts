import { vi } from 'vitest'

// Mock event listeners
const eventListeners = new Map<string, Function[]>()

export const listen = vi.fn().mockImplementation((event: string, handler: Function) => {
  if (!eventListeners.has(event)) {
    eventListeners.set(event, [])
  }
  eventListeners.get(event)!.push(handler)
  
  // Return unlisten function
  return Promise.resolve(() => {
    const listeners = eventListeners.get(event)
    if (listeners) {
      const index = listeners.indexOf(handler)
      if (index > -1) {
        listeners.splice(index, 1)
      }
    }
  })
})

export const emit = vi.fn().mockImplementation((event: string, payload: any) => {
  const listeners = eventListeners.get(event)
  if (listeners) {
    listeners.forEach(listener => {
      listener({ payload })
    })
  }
  return Promise.resolve()
})

// Helper function to trigger events in tests
export const triggerEvent = (event: string, payload: any) => {
  const listeners = eventListeners.get(event)
  if (listeners) {
    listeners.forEach(listener => {
      listener({ payload })
    })
  }
}

// Helper to clear all listeners (useful for test cleanup)
export const clearAllListeners = () => {
  eventListeners.clear()
}