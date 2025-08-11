import type { App } from 'vue'

interface ErrorDetails {
  error: any
  info?: string
  component?: string
  timestamp: Date
}

class GlobalErrorHandler {
  private errors: ErrorDetails[] = []
  private maxErrors = 100

  logError(error: any, info?: string, component?: string) {
    const errorDetail: ErrorDetails = {
      error,
      info,
      component,
      timestamp: new Date()
    }

    this.errors.unshift(errorDetail)
    
    // Keep only the last N errors
    if (this.errors.length > this.maxErrors) {
      this.errors = this.errors.slice(0, this.maxErrors)
    }

    // Log to console
    console.error('Global Error Handler:', errorDetail)

    // Report to external service if configured
    this.reportError(errorDetail)
  }

  private reportError(errorDetail: ErrorDetails) {
    // Integration point for error reporting services
    if (typeof window !== 'undefined') {
      if (window.reportError) {
        window.reportError(errorDetail.error, {
          info: errorDetail.info,
          component: errorDetail.component,
          timestamp: errorDetail.timestamp.toISOString()
        })
      }

      // Could also integrate with Sentry, LogRocket, or other services
      if (window.Sentry?.captureException) {
        window.Sentry.captureException(errorDetail.error, {
          tags: { component: errorDetail.component },
          extra: { info: errorDetail.info }
        })
      }
    }
  }

  getRecentErrors(count = 10): ErrorDetails[] {
    return this.errors.slice(0, count)
  }

  clearErrors() {
    this.errors = []
  }
}

const globalErrorHandler = new GlobalErrorHandler()

export default {
  install(app: App) {
    // Handle Vue errors
    app.config.errorHandler = (err, instance, info) => {
      globalErrorHandler.logError(err, info, instance?.$options.name || instance?.$options.__name)
    }

    // Handle unhandled promise rejections
    if (typeof window !== 'undefined') {
      window.addEventListener('unhandledrejection', (event) => {
        globalErrorHandler.logError(event.reason, 'Unhandled Promise Rejection')
        
        // Prevent the default browser behavior (logging to console)
        // event.preventDefault()
      })

      // Handle general JavaScript errors
      window.addEventListener('error', (event) => {
        globalErrorHandler.logError(event.error || event.message, 'JavaScript Error', event.filename)
      })
    }

    // Make error handler available globally
    app.config.globalProperties.$errorHandler = globalErrorHandler
    app.provide('errorHandler', globalErrorHandler)
  }
}

export { globalErrorHandler }

// TypeScript declaration for global properties
declare module '@vue/runtime-core' {
  interface ComponentCustomProperties {
    $errorHandler: GlobalErrorHandler
  }
}

// Window interface extension for error reporting
declare global {
  interface Window {
    reportError?: (error: any, context?: any) => void
    reportApiError?: (error: any, context?: any) => void
    Sentry?: {
      captureException: (error: any, context?: any) => void
    }
  }
}