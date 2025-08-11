/**
 * Timeout constants used throughout the application
 * Centralized to avoid magic numbers and ensure consistency
 */
export const TIMEOUTS = {
  // Event batching - how long to wait before processing batched events
  EVENT_BATCH_WINDOW: 50,
  
  // Namespace changes - debounce delay to prevent rapid successive API calls
  NAMESPACE_CHANGE_DEBOUNCE: 300,
  
  // Resource cleanup - small delay to allow backend cleanup
  RESOURCE_CLEANUP_DELAY: 100,
  
  // Hover effects - delays for UI hover states
  HOVER_DELAY: 150,
  HOVER_MAX_DURATION: 5000,
  
  // Clock updates - interval for time displays
  CLOCK_UPDATE_INTERVAL: 1000,
  
  // Retry delays - exponential backoff for retries
  RETRY_BASE_DELAY: 1000,
  RETRY_MAX_DELAY: 30000,
  
  // Scaling timeouts - for resource scaling operations
  SCALING_FALLBACK_TIMEOUT: 15000,
} as const

/**
 * Helper function for exponential backoff
 */
export function getRetryDelay(attempt: number): number {
  const delay = TIMEOUTS.RETRY_BASE_DELAY * Math.pow(2, attempt)
  return Math.min(delay, TIMEOUTS.RETRY_MAX_DELAY)
}