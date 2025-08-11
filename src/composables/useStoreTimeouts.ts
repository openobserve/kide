/**
 * Global timeout management for stores (Pinia)
 * Since stores don't have component lifecycle, we need manual cleanup
 */
class StoreTimeoutManager {
  private timeouts = new Set<NodeJS.Timeout>()
  private intervals = new Set<NodeJS.Timeout>()

  createTimeout(callback: () => void, delay: number): NodeJS.Timeout {
    const timeout = setTimeout(() => {
      this.timeouts.delete(timeout)
      callback()
    }, delay)
    
    this.timeouts.add(timeout)
    return timeout
  }

  createInterval(callback: () => void, delay: number): NodeJS.Timeout {
    const interval = setInterval(callback, delay)
    this.intervals.add(interval)
    return interval
  }

  clearTimeout(timeout: NodeJS.Timeout | null): void {
    if (timeout) {
      clearTimeout(timeout)
      this.timeouts.delete(timeout)
    }
  }

  clearInterval(interval: NodeJS.Timeout | null): void {
    if (interval) {
      clearInterval(interval)
      this.intervals.delete(interval)
    }
  }

  clearAll(): void {
    this.timeouts.forEach(timeout => {
      clearTimeout(timeout)
    })
    this.timeouts.clear()

    this.intervals.forEach(interval => {
      clearInterval(interval)
    })
    this.intervals.clear()
  }

  getActiveCount() {
    return {
      timeouts: this.timeouts.size,
      intervals: this.intervals.size
    }
  }
}

// Global instance for stores
const storeTimeoutManager = new StoreTimeoutManager()

export function useStoreTimeouts() {
  return {
    createTimeout: storeTimeoutManager.createTimeout.bind(storeTimeoutManager),
    createInterval: storeTimeoutManager.createInterval.bind(storeTimeoutManager),
    clearTimeout: storeTimeoutManager.clearTimeout.bind(storeTimeoutManager),
    clearInterval: storeTimeoutManager.clearInterval.bind(storeTimeoutManager),
    clearAll: storeTimeoutManager.clearAll.bind(storeTimeoutManager),
    getActiveCount: storeTimeoutManager.getActiveCount.bind(storeTimeoutManager)
  }
}

// Cleanup function for when app shuts down
export function cleanupStoreTimeouts() {
  storeTimeoutManager.clearAll()
}