/**
 * Structured Logging System
 *
 * Provides comprehensive logging capabilities with multiple levels,
 * structured output, performance monitoring, and integration with
 * the orchestrator store for centralized error handling.
 *
 * Features:
 * - Multiple log levels (trace, debug, info, warn, error, fatal)
 * - Structured logging with context and metadata
 * - Performance tracking and timing
 * - Log aggregation and filtering
 * - Export capabilities
 * - Integration with Tauri backend
 */

export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
  FATAL = 5
}

export interface LogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  message: string
  context?: Record<string, any>
  category?: string
  source?: string
  userId?: string
  sessionId?: string
  requestId?: string
  duration?: number
  stack?: string
  tags?: string[]
  component?: string
  action?: string
  metrics?: Record<string, number>
}

export interface LoggerConfig {
  level: LogLevel
  enableConsole: boolean
  enableFile: boolean
  enableRemote: boolean
  enableStructuredOutput: boolean
  maxLogEntries: number
  retentionDays: number
  enablePerformanceLogging: boolean
  enableUserTracking: boolean
  enableErrorTracking: boolean
  bufferSize: number
  flushInterval: number
  remoteEndpoint?: string
  formats: {
    console: 'simple' | 'structured' | 'colored'
    file: 'json' | 'text'
    remote: 'json'
  }
  filters: {
    categories: string[]
    levels: LogLevel[]
    components: string[]
  }
}

export interface LogMetrics {
  totalEntries: number
  entriesByLevel: Record<LogLevel, number>
  entriesByCategory: Record<string, number>
  entriesByComponent: Record<string, number>
  averageEntrySize: number
  memoryUsage: number
  errorRate: number
  performanceMetrics: {
    averageDuration: number
    slowOperations: number
    criticalOperations: number
  }
  userMetrics: {
    activeSessions: number
    errorsByUser: Record<string, number>
    actionsByUser: Record<string, number>
  }
}

export interface PerformanceTimer {
  id: string
  name: string
  startTime: number
  category: string
  component?: string
  context?: Record<string, any>
  threshold?: number
  onComplete?: (duration: number) => void
  onThreshold?: (duration: number) => void
}

/**
 * Structured Logger Composable
 */
export function useLogger(
  category: string = 'app',
  defaultContext: Record<string, any> = {}
) {
  // Configuration
  const config = ref<LoggerConfig>({
    level: LogLevel.INFO,
    enableConsole: true,
    enableFile: true,
    enableRemote: false,
    enableStructuredOutput: true,
    maxLogEntries: 10000,
    retentionDays: 30,
    enablePerformanceLogging: true,
    enableUserTracking: true,
    enableErrorTracking: true,
    bufferSize: 100,
    flushInterval: 5000,
    formats: {
      console: 'structured',
      file: 'json',
      remote: 'json'
    },
    filters: {
      categories: [],
      levels: [],
      components: []
    }
  })

  // State
  const logs = ref<LogEntry[]>([])
  const buffer = ref<LogEntry[]>([])
  const timers = ref<Map<string, PerformanceTimer>>(new Map())
  const metrics = ref<LogMetrics>({
    totalEntries: 0,
    entriesByLevel: {} as Record<LogLevel, number>,
    entriesByCategory: {} as Record<string, number>,
    entriesByComponent: {} as Record<string, number>,
    averageEntrySize: 0,
    memoryUsage: 0,
    errorRate: 0,
    performanceMetrics: {
      averageDuration: 0,
      slowOperations: 0,
      criticalOperations: 0
    },
    userMetrics: {
      activeSessions: 1,
      errorsByUser: {} as Record<string, number>,
      actionsByUser: {} as Record<string, number>
    }
  })

  const sessionId = ref(`session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`)
  const userId = ref<string | null>(null)

  // Computed
  const logLevelName = (level: LogLevel): string => {
    return LogLevel[level]
  }

  const shouldLog = (level: LogLevel, entryCategory?: string, component?: string): boolean => {
    // Check level
    if (level < config.value.level) return false

    // Check category filters
    if (config.value.filters.categories.length > 0 && entryCategory) {
      if (!config.value.filters.categories.includes(entryCategory)) return false
    }

    // Check level filters
    if (config.value.filters.levels.length > 0) {
      if (!config.value.filters.levels.includes(level)) return false
    }

    // Check component filters
    if (config.value.filters.components.length > 0 && component) {
      if (!config.value.filters.components.includes(component)) return false
    }

    return true
  }

  const formatLogEntry = (entry: LogEntry, format: 'simple' | 'structured' | 'colored' = 'structured'): string => {
    const timestamp = entry.timestamp.toISOString()
    const level = logLevelName(entry.level)
    const contextStr = entry.context ? ` ${JSON.stringify(entry.context)}` : ''
    const durationStr = entry.duration ? ` ${entry.duration}ms` : ''

    switch (format) {
      case 'simple':
        return `${timestamp} [${level}] ${entry.message}`

      case 'structured':
        return JSON.stringify({
          timestamp,
          level,
          message: entry.message,
          category: entry.category,
          context: entry.context,
          duration: entry.duration,
          component: entry.component,
          tags: entry.tags
        })

      case 'colored':
        const colors = {
          [LogLevel.TRACE]: '\x1b[90m', // Gray
          [LogLevel.DEBUG]: '\x1b[36m', // Cyan
          [LogLevel.INFO]: '\x1b[32m',  // Green
          [LogLevel.WARN]: '\x1b[33m',  // Yellow
          [LogLevel.ERROR]: '\x1b[31m', // Red
          [LogLevel.FATAL]: '\x1b[35m'  // Magenta
        }
        const reset = '\x1b[0m'
        const color = colors[entry.level] || ''

        return `${timestamp} ${color}[${level}]${reset} ${entry.message}${contextStr}${durationStr}`

      default:
        return entry.message
    }
  }

  // Core logging methods
  const log = (
    level: LogLevel,
    message: string,
    context: Record<string, any> = {},
    options: {
      component?: string
      action?: string
      tags?: string[]
      stack?: string
      metrics?: Record<string, number>
      requestId?: string
    } = {}
  ): void => {
    if (!shouldLog(level, category, options.component)) return

    const entry: LogEntry = {
      id: `log_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: new Date(),
      level,
      message,
      context: { ...defaultContext, ...context },
      category,
      source: options.component || 'unknown',
      userId: userId.value || undefined,
      sessionId: sessionId.value,
      requestId: options.requestId,
      component: options.component,
      action: options.action,
      tags: options.tags,
      stack: options.stack,
      metrics: options.metrics
    }

    // Add to buffer
    buffer.value.push(entry)

    // Update metrics
    updateMetrics(entry)

    // Process immediately for high-level logs
    if (level >= LogLevel.ERROR) {
      flushBuffer()
    }
  }

  const trace = (message: string, context: Record<string, any> = {}, options?: any): void => {
    log(LogLevel.TRACE, message, context, options)
  }

  const debug = (message: string, context: Record<string, any> = {}, options?: any): void => {
    log(LogLevel.DEBUG, message, context, options)
  }

  const info = (message: string, context: Record<string, any> = {}, options?: any): void => {
    log(LogLevel.INFO, message, context, options)
  }

  const warn = (message: string, context: Record<string, any> = {}, options?: any): void => {
    log(LogLevel.WARN, message, context, options)
  }

  const error = (message: string, error?: Error | string, context: Record<string, any> = {}, options?: any): void => {
    const errorContext = {
      ...context,
      error: error instanceof Error ? error.message : error,
      stack: error instanceof Error ? error.stack : undefined
    }
    log(LogLevel.ERROR, message, errorContext, {
      ...options,
      stack: error instanceof Error ? error.stack : undefined
    })
  }

  const fatal = (message: string, error?: Error | string, context: Record<string, any> = {}, options?: any): void => {
    const errorContext = {
      ...context,
      error: error instanceof Error ? error.message : error,
      stack: error instanceof Error ? error.stack : undefined
    }
    log(LogLevel.FATAL, message, errorContext, {
      ...options,
      stack: error instanceof Error ? error.stack : undefined
    })

    // Flush immediately for fatal errors
    flushBuffer()
  }

  // Performance logging
  const startTimer = (
    name: string,
    category: string = 'performance',
    options: {
      component?: string
      context?: Record<string, any>
      threshold?: number
      onComplete?: (duration: number) => void
      onThreshold?: (duration: number) => void
    } = {}
  ): string => {
    const timerId = `timer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const timer: PerformanceTimer = {
      id: timerId,
      name,
      startTime: performance.now(),
      category,
      component: options.component,
      context: options.context,
      threshold: options.threshold,
      onComplete: options.onComplete,
      onThreshold: options.onThreshold
    }

    timers.value.set(timerId, timer)

    debug(`Timer started: ${name}`, {
      timerId,
      category,
      threshold: options.threshold
    }, { component: options.component, action: 'timer_start' })

    return timerId
  }

  const endTimer = (timerId: string, additionalContext: Record<string, any> = {}): number | null => {
    const timer = timers.value.get(timerId)
    if (!timer) {
      warn(`Timer not found: ${timerId}`, { timerId }, { action: 'timer_end' })
      return null
    }

    const duration = performance.now() - timer.startTime
    const context = {
      ...timer.context,
      ...additionalContext,
      timerId,
      duration: Math.round(duration)
    }

    // Log performance entry
    const level = timer.threshold && duration > timer.threshold ? LogLevel.WARN : LogLevel.DEBUG
    const message = timer.threshold && duration > timer.threshold
      ? `Slow operation: ${timer.name} (${Math.round(duration)}ms)`
      : `Operation completed: ${timer.name} (${Math.round(duration)}ms)`

    log(level, message, context, {
      component: timer.component,
      action: 'timer_end',
      tags: ['performance', timer.category],
      metrics: { duration: Math.round(duration) }
    })

    // Update performance metrics
    updatePerformanceMetrics(duration, timer.threshold)

    // Call callbacks
    timer.onComplete?.(duration)
    if (timer.threshold && duration > timer.threshold) {
      timer.onThreshold?.(duration)
    }

    // Clean up
    timers.value.delete(timerId)

    return duration
  }

  const timeAsync = async <T>(
    name: string,
    fn: () => Promise<T>,
    category: string = 'async',
    options: {
      component?: string
      context?: Record<string, any>
      threshold?: number
      onError?: (error: Error) => void
    } = {}
  ): Promise<T> => {
    const timerId = startTimer(name, category, options)

    try {
      const result = await fn()
      endTimer(timerId, { success: true })
      return result
    } catch (error) {
      const errorContext = {
        ...options.context,
        timerId,
        success: false
      }

      error(`Async operation failed: ${name}`, error instanceof Error ? error : String(error),
        errorContext,
        { component: options.component, action: 'async_error' }
      )

      options.onError?.(error instanceof Error ? error : new Error(String(error)))
      endTimer(timerId, { success: false, error: String(error) })
      throw error
    }
  }

  // Metrics update
  const updateMetrics = (entry: LogEntry): void => {
    metrics.value.totalEntries++

    // Update level counts
    metrics.value.entriesByLevel[entry.level] =
      (metrics.value.entriesByLevel[entry.level] || 0) + 1

    // Update category counts
    if (entry.category) {
      metrics.value.entriesByCategory[entry.category] =
        (metrics.value.entriesByCategory[entry.category] || 0) + 1
    }

    // Update component counts
    if (entry.component) {
      metrics.value.entriesByComponent[entry.component] =
        (metrics.value.entriesByComponent[entry.component] || 0) + 1
    }

    // Update error rate
    if (entry.level >= LogLevel.ERROR) {
      metrics.value.errorRate =
        Object.values(metrics.value.entriesByLevel)
          .slice(LogLevel.ERROR)
          .reduce((sum, count) => sum + count, 0) / metrics.value.totalEntries
    }

    // Update user metrics
    if (entry.userId) {
      if (entry.level >= LogLevel.ERROR) {
        metrics.value.userMetrics.errorsByUser[entry.userId] =
          (metrics.value.userMetrics.errorsByUser[entry.userId] || 0) + 1
      }

      if (entry.action) {
        metrics.value.userMetrics.actionsByUser[entry.userId] =
          (metrics.value.userMetrics.actionsByUser[entry.userId] || 0) + 1
      }
    }

    // Calculate average entry size
    const entrySize = JSON.stringify(entry).length
    metrics.value.averageEntrySize =
      (metrics.value.averageEntrySize * (metrics.value.totalEntries - 1) + entrySize) /
      metrics.value.totalEntries

    // Update memory usage
    metrics.value.memoryUsage =
      metrics.value.totalEntries * metrics.value.averageEntrySize
  }

  const updatePerformanceMetrics = (duration: number, threshold?: number): void => {
    const { performanceMetrics } = metrics.value

    // Update average duration
    performanceMetrics.averageDuration =
      (performanceMetrics.averageDuration * (metrics.value.totalEntries - 1) + duration) /
      metrics.value.totalEntries

    // Count slow operations
    if (threshold && duration > threshold) {
      performanceMetrics.slowOperations++
    }

    // Count critical operations (> 5 seconds)
    if (duration > 5000) {
      performanceMetrics.criticalOperations++
    }
  }

  // Buffer management
  const flushBuffer = (): void => {
    if (buffer.value.length === 0) return

    const entries = [...buffer.value]
    buffer.value = []

    // Add to main logs
    logs.value.push(...entries)

    // Maintain max entries
    if (logs.value.length > config.value.maxLogEntries) {
      const excess = logs.value.length - config.value.maxLogEntries
      logs.value.splice(0, excess)
    }

    // Output to configured destinations
    outputLogs(entries)

    // Clean up old entries based on retention
    cleanupOldEntries()
  }

  const outputLogs = (entries: LogEntry[]): void => {
    entries.forEach(entry => {
      // Console output
      if (config.value.enableConsole) {
        const format = config.value.formats.console
        console.log(formatLogEntry(entry, format))
      }

      // File output (would need Tauri integration)
      if (config.value.enableFile) {
        // Implementation would use Tauri file system APIs
        // For now, we'll store in localStorage
        try {
          const existingLogs = localStorage.getItem(`logs_${entry.category || 'default'}`) || '[]'
          const logsArray = JSON.parse(existingLogs)
          logsArray.push(entry)

          // Keep only last N entries per category
          if (logsArray.length > 1000) {
            logsArray.splice(0, logsArray.length - 1000)
          }

          localStorage.setItem(`logs_${entry.category || 'default'}`, JSON.stringify(logsArray))
        } catch (error) {
          console.error('Failed to write logs to file:', error)
        }
      }

      // Remote output
      if (config.value.enableRemote && config.value.remoteEndpoint) {
        // Implementation would send logs to remote endpoint
        // For now, we'll just log the intent
        debug(`Would send log to remote endpoint: ${entry.id}`, {
          endpoint: config.value.remoteEndpoint,
          entry: entry.id
        })
      }
    })
  }

  const cleanupOldEntries = (): void => {
    if (!config.value.retentionDays) return

    const cutoffDate = new Date()
    cutoffDate.setDate(cutoffDate.getDate() - config.value.retentionDays)

    const initialCount = logs.value.length
    logs.value = logs.value.filter(entry => entry.timestamp >= cutoffDate)
    const removedCount = initialCount - logs.value.length

    if (removedCount > 0) {
      info(`Cleaned up ${removedCount} old log entries`, {
        retentionDays: config.value.retentionDays,
        cutoffDate: cutoffDate.toISOString()
      })
    }
  }

  // Log filtering and searching
  const getLogs = (filters: {
    level?: LogLevel
    category?: string
    component?: string
    startDate?: Date
    endDate?: Date
    search?: string
    limit?: number
  } = {}): LogEntry[] => {
    let filtered = [...logs.value]

    if (filters.level !== undefined) {
      filtered = filtered.filter(entry => entry.level >= filters.level!)
    }

    if (filters.category) {
      filtered = filtered.filter(entry => entry.category === filters.category)
    }

    if (filters.component) {
      filtered = filtered.filter(entry => entry.component === filters.component)
    }

    if (filters.startDate) {
      filtered = filtered.filter(entry => entry.timestamp >= filters.startDate!)
    }

    if (filters.endDate) {
      filtered = filtered.filter(entry => entry.timestamp <= filters.endDate!)
    }

    if (filters.search) {
      const searchLower = filters.search.toLowerCase()
      filtered = filtered.filter(entry =>
        entry.message.toLowerCase().includes(searchLower) ||
        entry.category?.toLowerCase().includes(searchLower) ||
        entry.component?.toLowerCase().includes(searchLower)
      )
    }

    if (filters.limit) {
      filtered = filtered.slice(-filters.limit)
    }

    return filtered
  }

  const exportLogs = (filters: any = {}, format: 'json' | 'csv' = 'json'): string => {
    const filtered = getLogs(filters)

    if (format === 'json') {
      return JSON.stringify(filtered, null, 2)
    } else if (format === 'csv') {
      const headers = ['timestamp', 'level', 'message', 'category', 'component', 'duration']
      const rows = filtered.map(entry => [
        entry.timestamp.toISOString(),
        logLevelName(entry.level),
        entry.message,
        entry.category || '',
        entry.component || '',
        entry.duration || ''
      ])

      return [headers, ...rows].map(row => row.join(',')).join('\n')
    }

    return ''
  }

  // Configuration management
  const updateConfig = (newConfig: Partial<LoggerConfig>): void => {
    config.value = { ...config.value, ...newConfig }

    // Log the configuration change
    info('Logger configuration updated', {
      oldConfig: config.value,
      newConfig
    }, { category: 'logger' })
  }

  const setUserId = (id: string): void => {
    userId.value = id
    info(`User ID set: ${id}`, { userId: id }, { category: 'auth' })
  }

  const clearLogs = (): void => {
    logs.value = []
    buffer.value = []

    info('Logs cleared', {}, { category: 'logger' })
  }

  // Auto-flush timer
  const flushTimer = setInterval(() => {
    flushBuffer()
  }, config.value.flushInterval)

  // Cleanup on unmount
  const cleanup = (): void => {
    clearInterval(flushTimer)
    flushBuffer()
  }

  return {
    // State
    logs,
    metrics,
    config,
    sessionId,
    userId,

    // Logging methods
    trace,
    debug,
    info,
    warn,
    error,
    fatal,

    // Performance tracking
    startTimer,
    endTimer,
    timeAsync,

    // Log management
    getLogs,
    exportLogs,
    clearLogs,

    // Configuration
    updateConfig,
    setUserId,

    // Utilities
    flushBuffer,
    cleanup,

    // Computed helpers
    logLevelName,
    formatLogEntry
  }
}