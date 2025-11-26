import { ref, computed, watch } from 'vue'
import { useUIStore } from '../stores/ui'

export interface ErrorInfo {
  id: string
  type: 'validation' | 'network' | 'runtime' | 'business' | 'system'
  severity: 'low' | 'medium' | 'high' | 'critical'
  code?: string
  message: string
  details?: any
  stack?: string
  timestamp: Date
  context?: Record<string, any>
  retryable: boolean
  retried: number
  maxRetries: number
  resolved: boolean
}

export interface ErrorMetrics {
  total: number
  byType: Record<string, number>
  bySeverity: Record<string, number>
  resolved: number
  unresolved: number
  avgResolutionTime: number
  criticalRate: number
}

export function useErrorHandler() {
  const uiStore = useUIStore()

  // Error management state
  const errors = ref<ErrorInfo[]>([])
  const criticalErrors = ref<ErrorInfo[]>([])
  const suppressedErrors = ref<string[]>([])

  // Error metrics
  const metrics = ref<ErrorMetrics>({
    total: 0,
    byType: {},
    bySeverity: {},
    resolved: 0,
    unresolved: 0,
    avgResolutionTime: 0,
    criticalRate: 0
  })

  // Unhandled error handling
  const setupErrorHandlers = () => {
    if (typeof window !== 'undefined') {
      // Unhandled promise rejections
      window.addEventListener('unhandledrejection', (event) => {
        handleError({
          type: 'runtime',
          severity: 'high',
          message: 'Unhandled Promise Rejection',
          details: { reason: event.reason, promise: event.promise },
          retryable: false
        })
      })

      // JavaScript errors
      window.addEventListener('error', (event) => {
        handleError({
          type: 'runtime',
          severity: 'high',
          message: event.message,
          details: { filename: event.filename, lineno: event.lineno, colno: event.colno },
          stack: event.error?.stack,
          retryable: false
        })
      })

      // Tauri specific errors
      if (window.__TAURI__) {
        window.__TAURI__.listen('tauri://error', (event) => {
          handleError({
            type: 'system',
            severity: 'high',
            message: 'Tauri System Error',
            details: event.payload,
            retryable: true
          })
        })
      }
    }
  }

  // Error handling
  const handleError = (error: Partial<ErrorInfo>) => {
    const errorId = `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const timestamp = new Date()

    const errorInfo: ErrorInfo = {
      id: errorId,
      type: error.type || 'runtime',
      severity: error.severity || 'medium',
      code: error.code,
      message: error.message || 'Unknown error',
      details: error.details,
      stack: error.stack,
      timestamp,
      context: error.context,
      retryable: error.retryable ?? true,
      retried: error.retried || 0,
      maxRetries: error.maxRetries || 3,
      resolved: false
    }

    // Check if error should be suppressed
    if (suppressedErrors.value.includes(errorId) ||
        (errorInfo.type === 'validation' && errorInfo.severity === 'low')) {
      return errorId
    }

    // Add to errors list
    errors.value.unshift(errorInfo)

    // Add to critical errors if needed
    if (errorInfo.severity === 'critical') {
      criticalErrors.value.push(errorInfo)
    }

    // Update metrics
    updateMetrics(errorInfo)

    // Show user notification
    showErrorNotification(errorInfo)

    // Auto-resolve retryable errors
    if (errorInfo.retryable && errorInfo.retried < errorInfo.maxRetries) {
      setTimeout(() => {
        retryError(errorId)
      }, 1000 * Math.pow(2, errorInfo.retried)) // Exponential backoff
    }

    return errorId
  }

  // Error retry logic
  const retryError = async (errorId: string) => {
    const error = errors.value.find(e => e.id === errorId)
    if (!error || error.resolved || !error.retryable || error.retried >= error.maxRetries) {
      return false
    }

    error.retried++

    try {
      // Retry logic would depend on the error context
      // This is a placeholder for actual retry implementation
      const shouldRetry = await determineRetryStrategy(error)

      if (shouldRetry) {
        // Simulate retry success
        resolveError(errorId, 'Auto-retry successful')
        return true
      } else {
        if (error.retried >= error.maxRetries) {
          error.retryable = false
          error.severity = error.severity === 'critical' ? 'critical' : 'high'
          uiStore.showError('Retry Failed', `Failed to resolve error: ${error.message}`)
        }
        return false
      }
    } catch (retryError) {
      error.details = { ...error.details, retryError: retryError.message }
      return false
    }
  }

  // Error resolution
  const resolveError = (errorId: string, resolution?: string) => {
    const error = errors.value.find(e => e.id === errorId)
    if (!error) return false

    error.resolved = true
    error.context = { ...error.context, resolution }

    // Remove from critical errors
    const criticalIndex = criticalErrors.value.findIndex(e => e.id === errorId)
    if (criticalIndex !== -1) {
      criticalErrors.value.splice(criticalIndex, 1)
    }

    // Update metrics
    metrics.value.resolved++
    metrics.value.unresolved = Math.max(0, metrics.value.unresolved - 1)

    // Update average resolution time
    const resolutionTime = Date.now() - error.timestamp.getTime()
    metrics.value.avgResolutionTime =
      (metrics.value.avgResolutionTime * (metrics.value.resolved - 1) + resolutionTime) / metrics.value.resolved

    return true
  }

  // Error suppression
  const suppressError = (errorId: string) => {
    const index = suppressedErrors.value.indexOf(errorId)
    if (index === -1) {
      suppressedErrors.value.push(errorId)
    }
  }

  const unsuppressError = (errorId: string) => {
    const index = suppressedErrors.value.indexOf(errorId)
    if (index !== -1) {
      suppressedErrors.value.splice(index, 1)
    }
  }

  // Clear operations
  const clearErrors = (type?: string, severity?: string) => {
    if (type && severity) {
      errors.value = errors.value.filter(e =>
        e.type !== type || e.severity !== severity
      )
    } else if (type) {
      errors.value = errors.value.filter(e => e.type !== type)
    } else if (severity) {
      errors.value = errors.value.filter(e => e.severity !== severity)
    } else {
      errors.value = []
    }

    criticalErrors.value = criticalErrors.value.filter(e =>
      errors.value.some(mainError => mainError.id === e.id)
    )
  }

  // Error analysis
  const getErrorTrends = (hours: number = 24) => {
    const cutoff = Date.now() - (hours * 60 * 60 * 1000)
    const recentErrors = errors.value.filter(e => e.timestamp.getTime() > cutoff)

    const trends = {
      total: recentErrors.length,
      byHour: {} as Record<number, number>,
      byType: {} as Record<string, number>,
      bySeverity: {} as Record<string, number>,
      topMessages: {} as Record<string, number>
    }

    recentErrors.forEach(error => {
      const hour = error.timestamp.getHours()
      trends.byHour[hour] = (trends.byHour[hour] || 0) + 1
      trends.byType[error.type] = (trends.byType[error.type] || 0) + 1
      trends.bySeverity[error.severity] = (trends.bySeverity[error.severity] || 0) + 1
      trends.topMessages[error.message] = (trends.topMessages[error.message] || 0) + 1
    })

    return trends
  }

  // Metrics computation
  const updateMetrics = (error: ErrorInfo) => {
    metrics.value.total++
    metrics.value.byType[error.type] = (metrics.value.byType[error.type] || 0) + 1
    metrics.value.bySeverity[error.severity] = (metrics.value.bySeverity[error.severity] || 0) + 1
    metrics.value.unresolved++

    if (error.severity === 'critical') {
      metrics.value.criticalRate = criticalErrors.value.length / metrics.value.total
    }
  }

  // Notification handling
  const showErrorNotification = (error: ErrorInfo) => {
    switch (error.severity) {
      case 'critical':
        uiStore.showError('Critical Error', error.message, true)
        break
      case 'high':
        uiStore.showError('Error', error.message)
        break
      case 'medium':
        uiStore.showWarning('Warning', error.message)
        break
      case 'low':
        uiStore.showInfo('Info', error.message, 3000)
        break
    }
  }

  // Retry strategy determination
  const determineRetryStrategy = async (error: ErrorInfo): Promise<boolean> => {
    switch (error.type) {
      case 'network':
        return error.retried < 3
      case 'validation':
        return false // Validation errors are not retryable
      case 'runtime':
        return error.retried < 2
      case 'business':
        return error.retried < 1
      case 'system':
        return error.retried < 5
      default:
        return error.retried < 3
    }
  }

  // Computed properties
  const hasCriticalErrors = computed(() => criticalErrors.value.length > 0)
  const hasUnresolvedErrors = computed(() =>
    errors.value.some(e => !e.resolved)
  )
  const errorRate = computed(() =>
    metrics.value.total > 0 ? metrics.value.unresolved / metrics.value.total : 0
  )
  const recentErrors = computed(() =>
    errors.value.filter(e =>
      Date.now() - e.timestamp.getTime() < 60 * 60 * 1000 // Last hour
    )
  )

  return {
    // State
    errors,
    criticalErrors,
    suppressedErrors,
    metrics,

    // Computed
    hasCriticalErrors,
    hasUnresolvedErrors,
    errorRate,
    recentErrors,

    // Methods
    setupErrorHandlers,
    handleError,
    retryError,
    resolveError,
    suppressError,
    unsuppressError,
    clearErrors,
    getErrorTrends
  }
}