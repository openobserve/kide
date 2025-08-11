import { onBeforeUnmount, onUnmounted } from 'vue'

/**
 * Composable for managing timeouts with automatic cleanup
 * Prevents memory leaks by tracking and clearing all timeouts on component unmount
 */
export function useTimeouts() {
  const timeouts = new Set<NodeJS.Timeout>()
  const intervals = new Set<NodeJS.Timeout>()

  /**
   * Create a timeout with automatic cleanup tracking
   */
  const createTimeout = (callback: () => void, delay: number): NodeJS.Timeout => {
    const timeout = setTimeout(() => {
      timeouts.delete(timeout)
      callback()
    }, delay)
    
    timeouts.add(timeout)
    return timeout
  }

  /**
   * Create an interval with automatic cleanup tracking
   */
  const createInterval = (callback: () => void, delay: number): NodeJS.Timeout => {
    const interval = setInterval(callback, delay)
    intervals.add(interval)
    return interval
  }

  /**
   * Clear a specific timeout
   */
  const clearTimeoutSafe = (timeout: NodeJS.Timeout | null): void => {
    if (timeout) {
      clearTimeout(timeout)
      timeouts.delete(timeout)
    }
  }

  /**
   * Clear a specific interval
   */
  const clearIntervalSafe = (interval: NodeJS.Timeout | null): void => {
    if (interval) {
      clearInterval(interval)
      intervals.delete(interval)
    }
  }

  /**
   * Clear all timeouts and intervals
   */
  const clearAll = (): void => {
    timeouts.forEach(timeout => {
      clearTimeout(timeout)
    })
    timeouts.clear()

    intervals.forEach(interval => {
      clearInterval(interval)
    })
    intervals.clear()
  }

  /**
   * Get count of active timeouts/intervals (for debugging)
   */
  const getActiveCount = () => ({
    timeouts: timeouts.size,
    intervals: intervals.size
  })

  // Cleanup on component unmount
  onBeforeUnmount(clearAll)
  onUnmounted(clearAll)

  return {
    createTimeout,
    createInterval,
    clearTimeoutSafe,
    clearIntervalSafe,
    clearAll,
    getActiveCount
  }
}