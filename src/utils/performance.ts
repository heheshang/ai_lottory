/**
 * Performance monitoring utilities
 */

interface PerformanceMetric {
  name: string
  duration: number
  timestamp: number
}

class PerformanceMonitor {
  private metrics: PerformanceMetric[] = []
  private marks: Map<string, number> = new Map()

  /**
   * Start measuring a performance metric
   */
  start(name: string): void {
    this.marks.set(name, performance.now())
  }

  /**
   * End measuring and record the metric
   */
  end(name: string): number {
    const startTime = this.marks.get(name)
    if (!startTime) {
      console.warn(`No start mark found for: ${name}`)
      return 0
    }

    const duration = performance.now() - startTime
    this.metrics.push({
      name,
      duration,
      timestamp: Date.now()
    })

    this.marks.delete(name)
    return duration
  }

  /**
   * Measure a function execution time
   */
  async measure<T>(name: string, fn: () => T | Promise<T>): Promise<T> {
    this.start(name)
    try {
      const result = await fn()
      const duration = this.end(name)
      if (import.meta.env.DEV) {
        console.log(`⚡ ${name}: ${duration.toFixed(2)}ms`)
      }
      return result
    } catch (error) {
      this.end(name)
      throw error
    }
  }

  /**
   * Get all metrics
   */
  getMetrics(): PerformanceMetric[] {
    return [...this.metrics]
  }

  /**
   * Get metrics by name
   */
  getMetricsByName(name: string): PerformanceMetric[] {
    return this.metrics.filter(m => m.name === name)
  }

  /**
   * Get average duration for a metric
   */
  getAverageDuration(name: string): number {
    const metrics = this.getMetricsByName(name)
    if (metrics.length === 0) return 0
    
    const total = metrics.reduce((sum, m) => sum + m.duration, 0)
    return total / metrics.length
  }

  /**
   * Clear all metrics
   */
  clear(): void {
    this.metrics = []
    this.marks.clear()
  }

  /**
   * Log performance summary
   */
  logSummary(): void {
    const summary = new Map<string, { count: number; total: number; avg: number; min: number; max: number }>()

    this.metrics.forEach(metric => {
      const existing = summary.get(metric.name)
      if (!existing) {
        summary.set(metric.name, {
          count: 1,
          total: metric.duration,
          avg: metric.duration,
          min: metric.duration,
          max: metric.duration
        })
      } else {
        existing.count++
        existing.total += metric.duration
        existing.avg = existing.total / existing.count
        existing.min = Math.min(existing.min, metric.duration)
        existing.max = Math.max(existing.max, metric.duration)
      }
    })

    console.group('⚡ Performance Summary')
    summary.forEach((stats, name) => {
      console.log(
        `${name}: avg=${stats.avg.toFixed(2)}ms, min=${stats.min.toFixed(2)}ms, max=${stats.max.toFixed(2)}ms, count=${stats.count}`
      )
    })
    console.groupEnd()
  }
}

// Singleton instance
export const performanceMonitor = new PerformanceMonitor()

/**
 * Debounce function for performance optimization
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null

  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null
      func(...args)
    }

    if (timeout) {
      clearTimeout(timeout)
    }
    timeout = setTimeout(later, wait)
  }
}

/**
 * Throttle function for performance optimization
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false

  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => {
        inThrottle = false
      }, limit)
    }
  }
}

/**
 * Memoize function results for performance
 */
export function memoize<T extends (...args: any[]) => any>(
  func: T,
  resolver?: (...args: Parameters<T>) => string
): T {
  const cache = new Map<string, ReturnType<T>>()

  return function memoized(...args: Parameters<T>): ReturnType<T> {
    const key = resolver ? resolver(...args) : JSON.stringify(args)
    
    if (cache.has(key)) {
      return cache.get(key)!
    }

    const result = func(...args)
    cache.set(key, result)
    return result
  } as T
}

/**
 * Lazy load image with intersection observer
 */
export function lazyLoadImage(
  img: HTMLImageElement,
  src: string,
  options?: IntersectionObserverInit
): void {
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        img.src = src
        observer.unobserve(img)
      }
    })
  }, options)

  observer.observe(img)
}

/**
 * Check if code is running in production
 */
export const isProduction = (): boolean => {
  return import.meta.env.PROD
}

/**
 * Safe console log that only runs in development
 */
export const devLog = (...args: any[]): void => {
  if (import.meta.env.DEV) {
    console.log(...args)
  }
}

/**
 * Measure component render time
 */
export function measureRender(componentName: string) {
  return {
    onBeforeMount() {
      performanceMonitor.start(`${componentName}-mount`)
    },
    onMounted() {
      const duration = performanceMonitor.end(`${componentName}-mount`)
      devLog(`📊 ${componentName} mounted in ${duration.toFixed(2)}ms`)
    },
    onBeforeUpdate() {
      performanceMonitor.start(`${componentName}-update`)
    },
    onUpdated() {
      const duration = performanceMonitor.end(`${componentName}-update`)
      devLog(`📊 ${componentName} updated in ${duration.toFixed(2)}ms`)
    }
  }
}
