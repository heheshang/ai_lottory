// Centralized Error Handling System for Super Lotto Application
// Provides consistent error handling, logging, and user feedback

import type {
  ErrorInfo,
  ErrorSeverity,
  ApiResponse,
  createErrorResponse,
  createApiResponse
} from '@/types/superLotto'

// =============================================================================
// Error Categories and Codes
// =============================================================================

export enum ErrorCategory {
  NETWORK = 'NETWORK',
  API = 'API',
  VALIDATION = 'VALIDATION',
  PERMISSION = 'PERMISSION',
  SYSTEM = 'SYSTEM',
  USER_INPUT = 'USER_INPUT',
  DATA_INTEGRITY = 'DATA_INTEGRITY',
  ALGORITHM = 'ALGORITHM',
  PREDICTION = 'PREDICTION'
}

export enum ErrorCode {
  // Network Errors
  NETWORK_OFFLINE = 'NETWORK_OFFLINE',
  NETWORK_TIMEOUT = 'NETWORK_TIMEOUT',
  NETWORK_CONNECTION_FAILED = 'NETWORK_CONNECTION_FAILED',

  // API Errors
  API_SERVER_ERROR = 'API_SERVER_ERROR',
  API_RATE_LIMIT = 'API_RATE_LIMIT',
  API_UNAUTHORIZED = 'API_UNAUTHORIZED',
  API_NOT_FOUND = 'API_NOT_FOUND',
  API_INVALID_RESPONSE = 'API_INVALID_RESPONSE',

  // Validation Errors
  VALIDATION_REQUIRED = 'VALIDATION_REQUIRED',
  VALIDATION_INVALID_FORMAT = 'VALIDATION_INVALID_FORMAT',
  VALIDATION_OUT_OF_RANGE = 'VALIDATION_OUT_OF_RANGE',
  VALIDATION_INVALID_COMBINATION = 'VALIDATION_INVALID_COMBINATION',

  // Permission Errors
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  PERMISSION_EXPIRED = 'PERMISSION_EXPIRED',
  PERMISSION_INSUFFICIENT = 'PERMISSION_INSUFFICIENT',

  // System Errors
  SYSTEM_MEMORY = 'SYSTEM_MEMORY',
  SYSTEM_STORAGE = 'SYSTEM_STORAGE',
  SYSTEM_PROCESSING = 'SYSTEM_PROCESSING',
  SYSTEM_CONFIGURATION = 'SYSTEM_CONFIGURATION',

  // User Input Errors
  USER_INPUT_INVALID = 'USER_INPUT_INVALID',
  USER_INPUT_MISSING = 'USER_INPUT_MISSING',
  USER_INPUT_OUT_OF_RANGE = 'USER_INPUT_OUT_OF_RANGE',

  // Data Integrity Errors
  DATA_CORRUPTION = 'DATA_CORRUPTION',
  DATA_INCONSISTENT = 'DATA_INCONSISTENT',
  DATA_MISSING = 'DATA_MISSING',

  // Algorithm Errors
  ALGORITHM_NOT_AVAILABLE = 'ALGORITHM_NOT_AVAILABLE',
  ALGORITHM_CONFIGURATION = 'ALGORITHM_CONFIGURATION',
  ALGORITHM_EXECUTION = 'ALGORITHM_EXECUTION',

  // Prediction Errors
  PREDICTION_FAILED = 'PREDICTION_FAILED',
  PREDICTION_INVALID_PARAMS = 'PREDICTION_INVALID_PARAMS',
  PREDICTION_INSUFFICIENT_DATA = 'PREDICTION_INSUFFICIENT_DATA'
}

// =============================================================================
// Error Configuration
// =============================================================================

interface ErrorConfig {
  category: ErrorCategory
  severity: ErrorSeverity
  recoverable: boolean
  userMessage: string
  technicalMessage: string
  suggestions: string[]
  retryStrategy?: RetryStrategy
  notificationLevel: NotificationLevel
}

interface RetryStrategy {
  maxRetries: number
  baseDelay: number
  maxDelay: number
  backoffMultiplier: number
  retryableErrors: ErrorCode[]
}

type NotificationLevel = 'silent' | 'toast' | 'modal' | 'banner'

const ERROR_CONFIGS: Record<ErrorCode, ErrorConfig> = {
  // Network Errors
  [ErrorCode.NETWORK_OFFLINE]: {
    category: ErrorCategory.NETWORK,
    severity: 'high',
    recoverable: true,
    userMessage: '网络连接不可用，请检查网络设置',
    technicalMessage: 'Network connection is unavailable',
    suggestions: ['检查网络连接', '尝试刷新页面', '稍后重试'],
    retryStrategy: {
      maxRetries: 3,
      baseDelay: 1000,
      maxDelay: 5000,
      backoffMultiplier: 2,
      retryableErrors: [ErrorCode.NETWORK_OFFLINE, ErrorCode.NETWORK_TIMEOUT]
    },
    notificationLevel: 'banner'
  },

  [ErrorCode.NETWORK_TIMEOUT]: {
    category: ErrorCategory.NETWORK,
    severity: 'medium',
    recoverable: true,
    userMessage: '请求超时，请稍后重试',
    technicalMessage: 'Network request timed out',
    suggestions: ['检查网络连接', '稍后重试', '减少数据量'],
    retryStrategy: {
      maxRetries: 2,
      baseDelay: 500,
      maxDelay: 3000,
      backoffMultiplier: 1.5,
      retryableErrors: [ErrorCode.NETWORK_TIMEOUT]
    },
    notificationLevel: 'toast'
  },

  // API Errors
  [ErrorCode.API_SERVER_ERROR]: {
    category: ErrorCategory.API,
    severity: 'high',
    recoverable: true,
    userMessage: '服务器暂时不可用，请稍后重试',
    technicalMessage: 'API server returned an error',
    suggestions: ['稍后重试', '联系技术支持'],
    retryStrategy: {
      maxRetries: 2,
      baseDelay: 1000,
      maxDelay: 4000,
      backoffMultiplier: 2,
      retryableErrors: [ErrorCode.API_SERVER_ERROR]
    },
    notificationLevel: 'toast'
  },

  [ErrorCode.API_RATE_LIMIT]: {
    category: ErrorCategory.API,
    severity: 'medium',
    recoverable: true,
    userMessage: '请求过于频繁，请稍后重试',
    technicalMessage: 'API rate limit exceeded',
    suggestions: ['降低请求频率', '稍后重试'],
    retryStrategy: {
      maxRetries: 1,
      baseDelay: 5000,
      maxDelay: 10000,
      backoffMultiplier: 1,
      retryableErrors: [ErrorCode.API_RATE_LIMIT]
    },
    notificationLevel: 'toast'
  },

  // Validation Errors
  [ErrorCode.VALIDATION_REQUIRED]: {
    category: ErrorCategory.VALIDATION,
    severity: 'low',
    recoverable: true,
    userMessage: '请填写必填字段',
    technicalMessage: 'Required field validation failed',
    suggestions: ['检查所有必填字段'],
    notificationLevel: 'toast'
  },

  [ErrorCode.VALIDATION_INVALID_FORMAT]: {
    category: ErrorCategory.VALIDATION,
    severity: 'low',
    recoverable: true,
    userMessage: '输入格式不正确',
    technicalMessage: 'Input format validation failed',
    suggestions: ['检查输入格式', '参考示例格式'],
    notificationLevel: 'toast'
  },

  // Prediction Errors
  [ErrorCode.PREDICTION_FAILED]: {
    category: ErrorCategory.PREDICTION,
    severity: 'medium',
    recoverable: true,
    userMessage: '预测生成失败，请重试',
    technicalMessage: 'Prediction algorithm execution failed',
    suggestions: ['检查预测参数', '尝试其他算法', '稍后重试'],
    retryStrategy: {
      maxRetries: 1,
      baseDelay: 1000,
      maxDelay: 3000,
      backoffMultiplier: 1,
      retryableErrors: [ErrorCode.PREDICTION_FAILED]
    },
    notificationLevel: 'toast'
  },

  [ErrorCode.PREDICTION_INSUFFICIENT_DATA]: {
    category: ErrorCategory.PREDICTION,
    severity: 'medium',
    recoverable: true,
    userMessage: '数据不足，无法生成预测',
    technicalMessage: 'Insufficient data for prediction',
    suggestions: ['导入更多历史数据', '扩大分析范围', '选择需要数据较少的算法'],
    notificationLevel: 'modal'
  },

  // ... other error configurations would be defined here
} as const

// =============================================================================
// Error Handler Class
// =============================================================================

export class ErrorHandler {
  private static instance: ErrorHandler
  private errorLog: ErrorInfo[] = []
  private maxLogSize = 1000
  private errorCallbacks: Map<ErrorCategory, ((error: ErrorInfo) => void)[]> = new Map()

  static getInstance(): ErrorHandler {
    if (!ErrorHandler.instance) {
      ErrorHandler.instance = new ErrorHandler()
    }
    return ErrorHandler.instance
  }

  private constructor() {
    this.initializeErrorCallbacks()
  }

  // Main error handling method
  public handleError(
    error: Error | string | ErrorCode,
    context?: Record<string, unknown>,
    requestId?: string
  ): ErrorInfo {
    const errorInfo = this.createErrorInfo(error, context, requestId)

    // Log error
    this.logError(errorInfo)

    // Trigger category-specific callbacks
    this.triggerErrorCallbacks(errorInfo)

    // Show user notification
    this.showErrorNotification(errorInfo)

    return errorInfo
  }

  // Create standardized ErrorInfo
  private createErrorInfo(
    error: Error | string | ErrorCode,
    context?: Record<string, unknown>,
    requestId?: string
  ): ErrorInfo {
    let errorCode: ErrorCode
    let originalError: Error | string

    if (error instanceof Error) {
      // Try to map error message to known error codes
      errorCode = this.mapErrorToCode(error.message) || ErrorCode.SYSTEM_PROCESSING
      originalError = error
    } else if (Object.values(ErrorCode).includes(error as ErrorCode)) {
      errorCode = error as ErrorCode
      originalError = error
    } else {
      errorCode = ErrorCode.USER_INPUT_INVALID
      originalError = error
    }

    const config = ERROR_CONFIGS[errorCode]

    return {
      code: errorCode,
      message: config.userMessage,
      details: {
        technicalMessage: config.technicalMessage,
        originalError: originalError instanceof Error ? originalError.message : originalError,
        stack: originalError instanceof Error ? originalError.stack : undefined,
        context,
        userAgent: navigator.userAgent,
        timestamp: new Date().toISOString(),
        url: window.location.href
      },
      timestamp: new Date().toISOString(),
      request_id: requestId,
      severity: config.severity,
      recoverable: config.recoverable,
      suggestions: config.suggestions
    }
  }

  // Map error messages to error codes
  private mapErrorToCode(message: string): ErrorCode | null {
    const lowerMessage = message.toLowerCase()

    if (lowerMessage.includes('network') || lowerMessage.includes('fetch')) {
      if (lowerMessage.includes('offline')) return ErrorCode.NETWORK_OFFLINE
      if (lowerMessage.includes('timeout')) return ErrorCode.NETWORK_TIMEOUT
      return ErrorCode.NETWORK_CONNECTION_FAILED
    }

    if (lowerMessage.includes('unauthorized')) return ErrorCode.API_UNAUTHORIZED
    if (lowerMessage.includes('not found')) return ErrorCode.API_NOT_FOUND
    if (lowerMessage.includes('rate limit')) return ErrorCode.API_RATE_LIMIT

    if (lowerMessage.includes('validation')) return ErrorCode.VALIDATION_INVALID_FORMAT
    if (lowerMessage.includes('required')) return ErrorCode.VALIDATION_REQUIRED

    if (lowerMessage.includes('prediction')) return ErrorCode.PREDICTION_FAILED
    if (lowerMessage.includes('algorithm')) return ErrorCode.ALGORITHM_EXECUTION

    return null
  }

  // Error logging
  private logError(error: ErrorInfo): void {
    // Add to in-memory log
    this.errorLog.push(error)

    // Maintain log size
    if (this.errorLog.length > this.maxLogSize) {
      this.errorLog = this.errorLog.slice(-this.maxLogSize)
    }

    // Console logging with appropriate level
    const logMessage = `Error [${error.code}]: ${error.message}`
    const logDetails = {
      requestId: error.request_id,
      severity: error.severity,
      details: error.details,
      suggestions: error.suggestions
    }

    switch (error.severity) {
      case 'critical':
        console.error(logMessage, logDetails)
        break
      case 'high':
        console.error(logMessage, logDetails)
        break
      case 'medium':
        console.warn(logMessage, logDetails)
        break
      case 'low':
        console.info(logMessage, logDetails)
        break
    }
  }

  // Error callbacks
  private initializeErrorCallbacks(): void {
    // Network error callbacks
    this.registerErrorCallback(ErrorCategory.NETWORK, (error) => {
      // Could implement offline detection, retry mechanisms, etc.
      console.log('Network error callback triggered:', error.code)
    })

    // API error callbacks
    this.registerErrorCallback(ErrorCategory.API, (error) => {
      // Could implement token refresh, re-authentication, etc.
      console.log('API error callback triggered:', error.code)
    })

    // Validation error callbacks
    this.registerErrorCallback(ErrorCategory.VALIDATION, (error) => {
      // Could implement form field highlighting, validation UI updates, etc.
      console.log('Validation error callback triggered:', error.code)
    })
  }

  public registerErrorCallback(
    category: ErrorCategory,
    callback: (error: ErrorInfo) => void
  ): void {
    if (!this.errorCallbacks.has(category)) {
      this.errorCallbacks.set(category, [])
    }
    this.errorCallbacks.get(category)!.push(callback)
  }

  private triggerErrorCallbacks(error: ErrorInfo): void {
    const config = ERROR_CONFIGS[error.code as ErrorCode]
    if (!config) return

    const callbacks = this.errorCallbacks.get(config.category)
    if (callbacks) {
      callbacks.forEach(callback => {
        try {
          callback(error)
        } catch (callbackError) {
          console.error('Error in error callback:', callbackError)
        }
      })
    }
  }

  // User notifications
  private showErrorNotification(error: ErrorInfo): void {
    const config = ERROR_CONFIGS[error.code as ErrorCode]
    if (!config) return

    // Import notification system dynamically to avoid circular dependencies
    this.showNotification(error.message, config.notificationLevel, error.suggestions)
  }

  private showNotification(
    message: string,
    level: NotificationLevel,
    suggestions: string[] = []
  ): void {
    // This would integrate with your UI notification system
    // For now, just console log the notification
    console.log(`Notification [${level}]: ${message}`)
    if (suggestions.length > 0) {
      console.log('Suggestions:', suggestions)
    }

    // TODO: Integrate with Element Plus notifications
    // Example:
    // if (level === 'toast') {
    //   ElMessage.error(message)
    // } else if (level === 'modal') {
    //   ElMessageBox.alert(message, '错误', {
    //     type: 'error',
    //     showCancelButton: false
    //   })
    // }
  }

  // Retry mechanism
  public async retryWithBackoff<T>(
    operation: () => Promise<T>,
    errorCode: ErrorCode,
    context?: Record<string, unknown>
  ): Promise<T> {
    const config = ERROR_CONFIGS[errorCode]
    if (!config?.retryStrategy) {
      throw this.handleError(errorCode, context)
    }

    const { maxRetries, baseDelay, maxDelay, backoffMultiplier } = config.retryStrategy
    let lastError: Error | unknown

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await operation()
      } catch (error) {
        lastError = error

        if (attempt === maxRetries) {
          break
        }

        const delay = Math.min(baseDelay * Math.pow(backoffMultiplier, attempt), maxDelay)
        await new Promise(resolve => setTimeout(resolve, delay))

        console.log(`Retrying operation after ${delay}ms (attempt ${attempt + 1}/${maxRetries})`)
      }
    }

    throw this.handleError(errorCode, context)
  }

  // Error analytics
  public getErrorStats(): ErrorStats {
    const categoryCounts = new Map<ErrorCategory, number>()
    const severityCounts = new Map<ErrorSeverity, number>()
    const recentErrors = this.errorLog.slice(-10)

    this.errorLog.forEach(error => {
      const config = ERROR_CONFIGS[error.code as ErrorCode]
      if (config) {
        categoryCounts.set(
          config.category,
          (categoryCounts.get(config.category) || 0) + 1
        )
        severityCounts.set(
          error.severity,
          (severityCounts.get(error.severity) || 0) + 1
        )
      }
    })

    return {
      totalErrors: this.errorLog.length,
      categoryCounts: Object.fromEntries(categoryCounts),
      severityCounts: Object.fromEntries(severityCounts),
      recentErrors: recentErrors.map(e => ({
        code: e.code,
        message: e.message,
        timestamp: e.timestamp,
        severity: e.severity
      }))
    }
  }

  public clearErrorLog(): void {
    this.errorLog = []
  }

  public getErrorLog(): readonly ErrorInfo[] {
    return [...this.errorLog]
  }
}

// =============================================================================
// Error Statistics
// =============================================================================

export interface ErrorStats {
  totalErrors: number
  categoryCounts: Record<string, number>
  severityCounts: Record<string, number>
  recentErrors: Array<{
    code: string
    message: string
    timestamp: string
    severity: ErrorSeverity
  }>
}

// =============================================================================
// Utility Functions
// =============================================================================

export const errorHandler = ErrorHandler.getInstance()

export const handleError = (
  error: Error | string | ErrorCode,
  context?: Record<string, unknown>,
  requestId?: string
): ErrorInfo => {
  return errorHandler.handleError(error, context, requestId)
}

export const createSafeAsyncFunction = <T extends (...args: any[]) => Promise<any>>(
  fn: T,
  context?: Record<string, unknown>
): T => {
  return (async (...args: Parameters<T>): Promise<ReturnType<T>> => {
    try {
      return await fn(...args)
    } catch (error) {
      handleError(error as Error, context)
      throw error
    }
  }) as T
}

export const withErrorHandling = <T extends (...args: any[]) => any>(
  fn: T,
  errorHandler?: (error: Error) => void
): T => {
  return ((...args: Parameters<T>) => {
    try {
      const result = fn(...args)

      // Handle async functions
      if (result && typeof result.catch === 'function') {
        return result.catch((error: Error) => {
          if (errorHandler) {
            errorHandler(error)
          } else {
            handleError(error)
          }
          throw error
        })
      }

      return result
    } catch (error) {
      if (errorHandler) {
        errorHandler(error as Error)
      } else {
        handleError(error as Error)
      }
      throw error
    }
  }) as T
}

// =============================================================================
// Vue Composable for Error Handling
// =============================================================================

import { ref, reactive } from 'vue'

export function useErrorHandler() {
  const currentError = ref<ErrorInfo | null>(null)
  const isLoading = ref(false)
  const retryCount = ref(0)

  const errorState = reactive({
    hasError: false,
    errorMessage: '',
    errorCode: '',
    canRetry: false,
    suggestions: [] as string[]
  })

  const setError = (error: ErrorInfo) => {
    currentError.value = error
    errorState.hasError = true
    errorState.errorMessage = error.message
    errorState.errorCode = error.code
    errorState.canRetry = error.recoverable
    errorState.suggestions = error.suggestions || []
  }

  const clearError = () => {
    currentError.value = null
    errorState.hasError = false
    errorState.errorMessage = ''
    errorState.errorCode = ''
    errorState.canRetry = false
    errorState.suggestions = []
    retryCount.value = 0
  }

  const executeWithErrorHandling = async <T>(
    operation: () => Promise<T>,
    context?: Record<string, unknown>
  ): Promise<T | null> => {
    clearError()
    isLoading.value = true

    try {
      const result = await operation()
      return result
    } catch (error) {
      const errorInfo = handleError(error as Error, context)
      setError(errorInfo)
      return null
    } finally {
      isLoading.value = false
    }
  }

  return {
    currentError,
    isLoading,
    retryCount,
    errorState,
    setError,
    clearError,
    executeWithErrorHandling,
    handleError
  }
}

console.log('🛡️ [Error Handler] Centralized error handling system initialized')