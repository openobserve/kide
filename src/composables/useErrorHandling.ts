import { ref } from 'vue'
import type { Ref } from 'vue'

export interface AppError {
  message: string
  code?: string | number
  type: 'api' | 'validation' | 'network' | 'unknown'
  timestamp: Date
  context?: Record<string, any>
}

export function useErrorHandling() {
  const errors: Ref<AppError[]> = ref([])
  const isLoading = ref(false)

  function createError(
    message: string, 
    type: AppError['type'] = 'unknown',
    code?: string | number,
    context?: Record<string, any>
  ): AppError {
    return {
      message,
      type,
      code,
      timestamp: new Date(),
      context
    }
  }

  function addError(error: AppError | string, type?: AppError['type']): void {
    const errorObj = typeof error === 'string' 
      ? createError(error, type) 
      : error
    
    errors.value.push(errorObj)
    console.error(`[${errorObj.type.toUpperCase()}] ${errorObj.message}`, errorObj.context)
  }

  function clearErrors(): void {
    errors.value = []
  }

  function removeError(index: number): void {
    if (index >= 0 && index < errors.value.length) {
      errors.value.splice(index, 1)
    }
  }

  async function handleAsyncOperation<T>(
    operation: () => Promise<T>,
    errorMessage = 'Operation failed'
  ): Promise<T | null> {
    isLoading.value = true
    try {
      const result = await operation()
      return result
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      addError(`${errorMessage}: ${message}`, 'api')
      return null
    } finally {
      isLoading.value = false
    }
  }

  function formatApiError(error: any): string {
    if (typeof error === 'string') return error
    if (error?.message) return error.message
    if (error?.error) return error.error
    return 'An unexpected error occurred'
  }

  return {
    errors,
    isLoading,
    addError,
    clearErrors,
    removeError,
    handleAsyncOperation,
    formatApiError,
    createError
  }
}