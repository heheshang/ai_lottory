/**
 * Performance Monitoring System
 *
 * Provides comprehensive performance monitoring capabilities including:
 * - Real-time performance metrics
 * - Component performance tracking
 * - API response monitoring
 * - Memory usage tracking
 * - User experience metrics
 * - Performance alerting
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useLogger } from './useLogger'
import { useUIStore } from '../stores/ui'
import { useOrchestratorStore } from '../stores/orchestrator'

export interface PerformanceMetrics {
  // System performance
  memory: {
    used: number
    total: number
    percentage: number
    trend: 'increasing' | 'decreasing' | 'stable'
  }
  cpu: {
    usage: number
    trend: 'increasing' | 'decreasing' | 'stable'
    cores: number
  }
  // Application performance
  api: {
    averageResponseTime: number
    slowRequests: number
    failedRequests: number
    requestsPerSecond: number
    cacheHitRate: number
    endpoints: Record<string, {
      count: number
      averageTime: number
      errorRate: number
    }>
  }
  // Component performance
  components: {
    renderTime: Record<string, number>
    reRenderCount: Record<string, number>
    memoryUsage: Record<string, number>
    slowComponents: string[]
  }
  // User experience
  userExperience: {
    averageLoadTime: number
    averageInteractionTime: number
    errorRate: number
    crashRate: number
    satisfaction: number
  }
  // Timing metrics
  timing: {
    firstContentfulPaint: number
    largestContentfulPaint: number
    firstInputDelay: number
    cumulativeLayoutShift: number
    timeToInteractive: number
  }
}

export interface PerformanceThreshold {
  name: string
  value: number
  operator: 'gt' | 'lt' | 'eq'
  severity: 'warning' | 'error' | 'critical'
  enabled: boolean
}

export interface PerformanceAlert {
  id: string
  timestamp: Date
  threshold: PerformanceThreshold
  currentValue: number
  message: string
  severity: 'warning' | 'error' | 'critical'
  acknowledged: boolean
  resolved: boolean
}

export interface ComponentPerformanceTracker {
  component: string
  renderCount: number
  totalRenderTime: number
  averageRenderTime: number
  maxRenderTime: number
  minRenderTime: number
  memoryUsage: number
  slowRenderCount: number
  reRenderReasons: Record<string, number>
}

export function usePerformanceMonitoring() {
  const logger = useLogger('performance')
  const uiStore = useUIStore()
  const orchestratorStore = useOrchestratorStore()

  // Reactive state
  const isMonitoring = ref(false)
  const metrics = ref<PerformanceMetrics>({
    memory: {
      used: 0,
      total: 0,
      percentage: 0,
      trend: 'stable'
    },
    cpu: {
      usage: 0,
      trend: 'stable',
      cores: navigator.hardwareConcurrency || 4
    },
    api: {
      averageResponseTime: 0,
      slowRequests: 0,
      failedRequests: 0,
      requestsPerSecond: 0,
      cacheHitRate: 0,
      endpoints: {}
    },
    components: {
      renderTime: {},
      reRenderCount: {},
      memoryUsage: {},
      slowComponents: []
    },
    userExperience: {
      averageLoadTime: 0,
      averageInteractionTime: 0,
      errorRate: 0,
      crashRate: 0,
      satisfaction: 0
    },
    timing: {
      firstContentfulPaint: 0,
      largestContentfulPaint: 0,
      firstInputDelay: 0,
      cumulativeLayoutShift: 0,
      timeToInteractive: 0
    }
  })

  const thresholds = ref<PerformanceThreshold[]>([
    {
      name: 'memory_usage',
      value: 80,
      operator: 'gt',
      severity: 'warning',
      enabled: true
    },
    {
      name: 'memory_usage_critical',
      value: 95,
      operator: 'gt',
      severity: 'critical',
      enabled: true
    },
    {
      name: 'api_response_time',
      value: 2000,
      operator: 'gt',
      severity: 'warning',
      enabled: true
    },
    {
      name: 'api_response_time_critical',
      value: 5000,
      operator: 'gt',
      severity: 'critical',
      enabled: true
    },
    {
      name: 'error_rate',
      value: 5,
      operator: 'gt',
      severity: 'warning',
      enabled: true
    },
    {
      name: 'error_rate_critical',
      value: 10,
      operator: 'gt',
      severity: 'critical',
      enabled: true
    },
    {
      name: 'component_render_time',
      value: 100,
      operator: 'gt',
      severity: 'warning',
      enabled: true
    },
    {
      name: 'component_render_time_critical',
      value: 300,
      operator: 'gt',
      severity: 'critical',
      enabled: true
    }
  ])

  const alerts = ref<PerformanceAlert[]>([])
  const componentTrackers = ref<Map<string, ComponentPerformanceTracker>>(new Map())
  const apiRequests = ref<Array<{
    url: string
    method: string
    startTime: number
    endTime?: number
    status?: number
    cached: boolean
    component?: string
  }>>([])

  // Computed properties
  const performanceScore = computed(() => {
    const weights = {
      memory: 0.2,
      api: 0.3,
      components: 0.25,
      userExperience: 0.25
    }

    const memoryScore = Math.max(0, 100 - metrics.value.memory.percentage)
    const apiScore = Math.max(0, 100 - (metrics.value.api.averageResponseTime / 50))
    const componentScore = calculateComponentScore()
    const userExperienceScore = calculateUserExperienceScore()

    return Math.round(
      weights.memory * memoryScore +
      weights.api * apiScore +
      weights.components * componentScore +
      weights.userExperience * userExperienceScore
    )
  })

  const activeAlerts = computed(() =>
    alerts.value.filter(alert => !alert.acknowledged && !alert.resolved)
  )

  const criticalAlerts = computed(() =>
    activeAlerts.value.filter(alert => alert.severity === 'critical')
  )

  const performanceStatus = computed(() => {
    if (criticalAlerts.value.length > 0) return 'critical'
    if (activeAlerts.value.filter(a => a.severity === 'error').length > 0) return 'poor'
    if (activeAlerts.value.length > 0) return 'degraded'
    if (performanceScore.value >= 90) return 'excellent'
    if (performanceScore.value >= 75) return 'good'
    return 'fair'
  })

  // Performance monitoring methods
  const startMonitoring = () => {
    if (isMonitoring.value) return

    isMonitoring.value = true
    logger.info('Performance monitoring started', {
      thresholds: thresholds.value.filter(t => t.enabled)
    }, { category: 'performance', action: 'monitoring_start' })

    // Start system metrics collection
    startSystemMetricsCollection()

    // Start web vitals monitoring
    startWebVitalsMonitoring()

    // Start API monitoring
    startAPIMonitoring()
  }

  const stopMonitoring = () => {
    isMonitoring.value = false
    logger.info('Performance monitoring stopped', {}, { category: 'performance', action: 'monitoring_stop' })
  }

  const startSystemMetricsCollection = () => {
    if (!('memory' in performance)) return

    const collectMetrics = () => {
      if (!isMonitoring.value) return

      try {
        // Memory metrics (if available)
        if ('memory' in performance && (performance as any).memory) {
          const memoryInfo = (performance as any).memory
          const usedMB = memoryInfo.usedJSHeapSize / (1024 * 1024)
          const totalMB = memoryInfo.totalJSHeapSize / (1024 * 1024)

          metrics.value.memory = {
            used: usedMB,
            total: totalMB,
            percentage: (usedMB / totalMB) * 100,
            trend: calculateTrend(metrics.value.memory.used, usedMB)
          }
        }

        // Check thresholds
        checkThresholds()
      } catch (error) {
        logger.error('Failed to collect system metrics', error as Error, {
          category: 'performance',
          action: 'metrics_collection_error'
        })
      }
    }

    // Collect metrics every 5 seconds
    const interval = setInterval(collectMetrics, 5000)

    // Store interval for cleanup
    return () => clearInterval(interval)
  }

  const startWebVitalsMonitoring = () => {
    if (typeof window === 'undefined') return

    // First Contentful Paint
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      entries.forEach((entry) => {
        if (entry.name === 'first-contentful-paint') {
          metrics.value.timing.firstContentfulPaint = entry.startTime
          logger.debug('First Contentful Paint', {
            time: entry.startTime
          }, { category: 'performance', action: 'web_vitals' })
        }
      })
    })
    observer.observe({ entryTypes: ['paint'] })

    // Largest Contentful Paint
    const lcpObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      const lastEntry = entries[entries.length - 1]
      if (lastEntry) {
        metrics.value.timing.largestContentfulPaint = lastEntry.startTime
      }
    })
    lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] })

    // First Input Delay
    const fidObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      entries.forEach((entry) => {
        if (entry.name === 'first-input') {
          metrics.value.timing.firstInputDelay = (entry as any).processingStart - entry.startTime
        }
      })
    })
    fidObserver.observe({ entryTypes: ['first-input'] })

    // Cumulative Layout Shift
    let clsValue = 0
    const clsObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries()
      entries.forEach((entry) => {
        if (!(entry as any).hadRecentInput) {
          clsValue += (entry as any).value
          metrics.value.timing.cumulativeLayoutShift = clsValue
        }
      })
    })
    clsObserver.observe({ entryTypes: ['layout-shift'] })

    // Cleanup function
    return () => {
      observer.disconnect()
      lcpObserver.disconnect()
      fidObserver.disconnect()
      clsObserver.disconnect()
    }
  }

  const startAPIMonitoring = () => {
    // Intercept fetch calls
    const originalFetch = window.fetch

    window.fetch = async (...args) => {
      const startTime = performance.now()
      const [url, options] = args

      const request = {
        url: typeof url === 'string' ? url : url.toString(),
        method: options?.method || 'GET',
        startTime,
        cached: false,
        component: getCurrentComponent()
      }

      apiRequests.value.push(request)

      try {
        const response = await originalFetch(...args)
        const endTime = performance.now()
        const duration = endTime - startTime

        // Update request
        request.endTime = endTime
        request.status = response.status

        // Update metrics
        updateAPIMetrics(request, duration, response.status, false)

        return response
      } catch (error) {
        const endTime = performance.now()
        const duration = endTime - startTime

        // Update request
        request.endTime = endTime
        request.status = 0

        // Update metrics
        updateAPIMetrics(request, duration, 0, true)

        throw error
      } finally {
        // Clean up old requests
        const cutoff = Date.now() - (5 * 60 * 1000) // Keep 5 minutes
        apiRequests.value = apiRequests.value.filter(req => req.startTime > cutoff)
      }
    }
  }

  const updateAPIMetrics = (
    request: any,
    duration: number,
    status: number,
    failed: boolean
  ) => {
    const url = new URL(request.url).pathname

    // Update endpoint metrics
    if (!metrics.value.api.endpoints[url]) {
      metrics.value.api.endpoints[url] = {
        count: 0,
        averageTime: 0,
        errorRate: 0
      }
    }

    const endpoint = metrics.value.api.endpoints[url]
    endpoint.count++
    endpoint.averageTime = (endpoint.averageTime * (endpoint.count - 1) + duration) / endpoint.count
    if (failed) endpoint.errorRate = (endpoint.errorRate * (endpoint.count - 1) + 1) / endpoint.count

    // Update overall metrics
    const totalRequests = Object.values(metrics.value.api.endpoints).reduce((sum, ep) => sum + ep.count, 0)

    if (failed) {
      metrics.value.api.failedRequests++
    }

    metrics.value.api.averageResponseTime =
      (metrics.value.api.averageResponseTime * (totalRequests - 1) + duration) / totalRequests

    if (duration > 2000) {
      metrics.value.api.slowRequests++
    }

    // Calculate requests per second (last minute)
    const oneMinuteAgo = Date.now() - 60000
    const recentRequests = apiRequests.value.filter(req => req.startTime > oneMinuteAgo)
    metrics.value.api.requestsPerSecond = recentRequests.length / 60

    // Log slow requests
    if (duration > 2000) {
      logger.warn('Slow API request', {
        url: request.url,
        method: request.method,
        duration,
        status
      }, { category: 'performance', component: request.component })
    }
  }

  const trackComponentPerformance = (
    componentName: string,
    renderTime: number,
    memoryUsage: number = 0,
    reason: string = 'update'
  ) => {
    let tracker = componentTrackers.value.get(componentName)

    if (!tracker) {
      tracker = {
        component: componentName,
        renderCount: 0,
        totalRenderTime: 0,
        averageRenderTime: 0,
        maxRenderTime: 0,
        minRenderTime: Infinity,
        memoryUsage: 0,
        slowRenderCount: 0,
        reRenderReasons: {}
      }
      componentTrackers.value.set(componentName, tracker)
    }

    // Update tracker
    tracker.renderCount++
    tracker.totalRenderTime += renderTime
    tracker.averageRenderTime = tracker.totalRenderTime / tracker.renderCount
    tracker.maxRenderTime = Math.max(tracker.maxRenderTime, renderTime)
    tracker.minRenderTime = Math.min(tracker.minRenderTime, renderTime)
    tracker.memoryUsage += memoryUsage

    // Track re-render reasons
    tracker.reRenderReasons[reason] = (tracker.reRenderReasons[reason] || 0) + 1

    // Count slow renders
    if (renderTime > 100) {
      tracker.slowRenderCount++

      logger.warn('Slow component render', {
        component: componentName,
        renderTime,
        reason
      }, { category: 'performance', component: componentName })
    }

    // Update component metrics
    metrics.value.components.renderTime[componentName] = tracker.averageRenderTime
    metrics.value.components.reRenderCount[componentName] = tracker.renderCount
    metrics.value.components.memoryUsage[componentName] = tracker.memoryUsage

    // Update slow components list
    metrics.value.components.slowComponents = Array.from(componentTrackers.value.values())
      .filter(tracker => tracker.averageRenderTime > 100)
      .sort((a, b) => b.averageRenderTime - a.averageRenderTime)
      .slice(0, 10)
      .map(tracker => tracker.component)

    // Check thresholds
    if (renderTime > 300) {
      createAlert({
        name: 'component_render_time_critical',
        value: renderTime,
        operator: 'gt',
        severity: 'critical',
        enabled: true
      }, renderTime, `Critical slow render in component: ${componentName}`)
    }
  }

  const trackUserInteraction = (
    interactionType: string,
    startTime: number,
    endTime?: number,
    success: boolean = true
  ) => {
    const duration = endTime ? endTime - startTime : 0

    logger.info('User interaction', {
      type: interactionType,
      duration,
      success
    }, { category: 'user_experience', action: 'interaction' })

    // Update user experience metrics
    if (duration > 0) {
      const totalInteractions = metrics.value.userExperience.averageInteractionTime || 0
      metrics.value.userExperience.averageInteractionTime =
        (totalInteractions + duration) / 2 // Simple average, could be more sophisticated
    }
  }

  const calculateComponentScore = () => {
    if (componentTrackers.value.size === 0) return 100

    const trackers = Array.from(componentTrackers.value.values())
    const averageRenderTime = trackers.reduce((sum, tracker) => sum + tracker.averageRenderTime, 0) / trackers.length
    const slowRenderCount = trackers.reduce((sum, tracker) => sum + tracker.slowRenderCount, 0)
    const totalRenders = trackers.reduce((sum, tracker) => sum + tracker.renderCount, 0)

    const renderTimeScore = Math.max(0, 100 - (averageRenderTime / 2))
    const slowRenderScore = Math.max(0, 100 - ((slowRenderCount / Math.max(1, totalRenders)) * 100))

    return (renderTimeScore + slowRenderScore) / 2
  }

  const calculateUserExperienceScore = () => {
    const { userExperience } = metrics.value

    const loadTimeScore = Math.max(0, 100 - (userExperience.averageLoadTime / 100))
    const interactionTimeScore = Math.max(0, 100 - (userExperience.averageInteractionTime / 500))
    const errorRateScore = Math.max(0, 100 - (userExperience.errorRate * 10))

    return (loadTimeScore + interactionTimeScore + errorRateScore) / 3
  }

  const calculateTrend = (previousValue: number, currentValue: number): 'increasing' | 'decreasing' | 'stable' => {
    const change = (currentValue - previousValue) / Math.max(1, previousValue)

    if (Math.abs(change) < 0.05) return 'stable'
    return change > 0 ? 'increasing' : 'decreasing'
  }

  const checkThresholds = () => {
    thresholds.value.forEach(threshold => {
      if (!threshold.enabled) return

      let currentValue: number

      switch (threshold.name) {
        case 'memory_usage':
        case 'memory_usage_critical':
          currentValue = metrics.value.memory.percentage
          break
        case 'api_response_time':
        case 'api_response_time_critical':
          currentValue = metrics.value.api.averageResponseTime
          break
        case 'error_rate':
        case 'error_rate_critical':
          currentValue = metrics.value.userExperience.errorRate
          break
        default:
          return
      }

      const thresholdMet = evaluateThreshold(currentValue, threshold)

      if (thresholdMet) {
        createAlert(threshold, currentValue, generateAlertMessage(threshold, currentValue))
      }
    })
  }

  const evaluateThreshold = (value: number, threshold: PerformanceThreshold): boolean => {
    switch (threshold.operator) {
      case 'gt': return value > threshold.value
      case 'lt': return value < threshold.value
      case 'eq': return value === threshold.value
      default: return false
    }
  }

  const createAlert = (
    threshold: PerformanceThreshold,
    currentValue: number,
    message: string
  ) => {
    const existingAlert = alerts.value.find(alert =>
      alert.threshold.name === threshold.name &&
      !alert.acknowledged &&
      alert.timestamp > new Date(Date.now() - 60000) // Within last minute
    )

    if (existingAlert) return

    const alert: PerformanceAlert = {
      id: `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: new Date(),
      threshold,
      currentValue,
      message,
      severity: threshold.severity,
      acknowledged: false,
      resolved: false
    }

    alerts.value.push(alert)

    logger.error('Performance threshold breached', {
      threshold: threshold.name,
      value: currentValue,
      limit: threshold.value,
      severity: threshold.severity
    }, { category: 'performance', action: 'threshold_breach' })

    // Show notification
    if (threshold.severity === 'critical') {
      uiStore.showError('Performance Alert', message, true)
    } else if (threshold.severity === 'error') {
      uiStore.showWarning('Performance Warning', message)
    }
  }

  const generateAlertMessage = (threshold: PerformanceThreshold, currentValue: number): string => {
    const operator = threshold.operator === 'gt' ? 'exceeded' : 'fell below'
    return `${threshold.name.replace(/_/g, ' ').toUpperCase()} ${operator} limit: ${currentValue.toFixed(2)} ${threshold.operator} ${threshold.value}`
  }

  const acknowledgeAlert = (alertId: string) => {
    const alert = alerts.value.find(a => a.id === alertId)
    if (alert) {
      alert.acknowledged = true
      logger.info('Performance alert acknowledged', {
        alertId,
        threshold: alert.threshold.name
      }, { category: 'performance', action: 'alert_acknowledged' })
    }
  }

  const resolveAlert = (alertId: string) => {
    const alert = alerts.value.find(a => a.id === alertId)
    if (alert) {
      alert.resolved = true
      logger.info('Performance alert resolved', {
        alertId,
        threshold: alert.threshold.name
      }, { category: 'performance', action: 'alert_resolved' })
    }
  }

  const getCurrentComponent = (): string | undefined => {
    // This would need to be implemented based on your component structure
    // For now, return undefined
    return undefined
  }

  // Auto-cleanup old alerts
  const cleanupOldAlerts = () => {
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000)
    alerts.value = alerts.value.filter(alert => alert.timestamp > oneHourAgo || !alert.acknowledged)
  }

  // Lifecycle
  let cleanupSystemMetrics: (() => void) | undefined
  let cleanupWebVitals: (() => void) | undefined

  onMounted(() => {
    startMonitoring()

    // Cleanup old alerts every 10 minutes
    const cleanupInterval = setInterval(cleanupOldAlerts, 10 * 60 * 1000)

    onUnmounted(() => {
      stopMonitoring()
      cleanupSystemMetrics?.()
      cleanupWebVitals?.()
      clearInterval(cleanupInterval)
    })
  })

  return {
    // State
    isMonitoring,
    metrics,
    thresholds,
    alerts,
    componentTrackers,

    // Computed
    performanceScore,
    activeAlerts,
    criticalAlerts,
    performanceStatus,

    // Methods
    startMonitoring,
    stopMonitoring,
    trackComponentPerformance,
    trackUserInteraction,
    acknowledgeAlert,
    resolveAlert,

    // Configuration
    updateThresholds: (newThresholds: PerformanceThreshold[]) => {
      thresholds.value = newThresholds
      logger.info('Performance thresholds updated', { thresholds: newThresholds })
    },

    // Utilities
    exportMetrics: () => JSON.stringify(metrics.value, null, 2),
    exportAlerts: () => JSON.stringify(alerts.value, null, 2)
  }
}