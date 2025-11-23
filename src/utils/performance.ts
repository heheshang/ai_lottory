// Performance Optimization Utilities for Super Lotto Application
// Implements lazy loading, virtual scrolling, memoization, and other performance enhancements

import { ref, onMounted, onUnmounted, nextTick, computed, type Ref } from 'vue'
import { debounce, throttle } from 'lodash-es'

// =============================================================================
// Performance Configuration
// =============================================================================

interface PerformanceConfig {
  enableVirtualScrolling: boolean
  virtualScrollItemHeight: number
  virtualScrollBufferSize: number
  enableImageOptimization: boolean
  enableComponentLazyLoading: boolean
  enableDataPrefetching: boolean
  enableMemoryOptimization: boolean
  maxCacheSize: number
  enablePerformanceMonitoring: boolean
}

const DEFAULT_PERFORMANCE_CONFIG: PerformanceConfig = {
  enableVirtualScrolling: true,
  virtualScrollItemHeight: 50,
  virtualScrollBufferSize: 10,
  enableImageOptimization: true,
  enableComponentLazyLoading: true,
  enableDataPrefetching: true,
  enableMemoryOptimization: true,
  maxCacheSize: 100,
  enablePerformanceMonitoring: process.env.NODE_ENV === 'development'
}

// =============================================================================
// Virtual Scrolling
// =============================================================================

export interface VirtualScrollOptions {
  itemHeight: number
  containerHeight: number
  bufferSize?: number
  overscan?: number
}

export function useVirtualScroll<T>(
  items: Ref<T[]>,
  options: VirtualScrollOptions
) {
  const { itemHeight, containerHeight, bufferSize = 10, overscan = 5 } = options

  const scrollTop = ref(0)
  const containerRef = ref<HTMLElement>()

  const visibleCount = Math.ceil(containerHeight / itemHeight)
  const totalCount = items.value.length
  const totalHeight = totalCount * itemHeight

  const startIndex = computed(() => {
    const index = Math.floor(scrollTop.value / itemHeight)
    return Math.max(0, index - overscan)
  })

  const endIndex = computed(() => {
    const index = startIndex.value + visibleCount + overscan * 2
    return Math.min(totalCount, index)
  })

  const visibleItems = computed(() => {
    return items.value.slice(startIndex.value, endIndex.value).map((item, index) => ({
      item,
      index: startIndex.value + index,
      offset: (startIndex.value + index) * itemHeight
    }))
  })

  const offsetY = computed(() => startIndex.value * itemHeight)

  const handleScroll = throttle((event: Event) => {
    const target = event.target as HTMLElement
    scrollTop.value = target.scrollTop
  }, 16) // ~60fps

  const scrollToItem = (index: number) => {
    const targetScrollTop = index * itemHeight
    if (containerRef.value) {
      containerRef.value.scrollTop = targetScrollTop
      scrollTop.value = targetScrollTop
    }
  }

  const scrollToTop = () => scrollToItem(0)
  const scrollToBottom = () => scrollToItem(totalCount - 1)

  return {
    containerRef,
    visibleItems,
    offsetY,
    totalHeight,
    scrollTop,
    handleScroll,
    scrollToItem,
    scrollToTop,
    scrollToBottom,
    startIndex,
    endIndex,
    visibleCount
  }
}

// =============================================================================
// Lazy Loading
// =============================================================================

export interface LazyLoadOptions {
  rootMargin?: string
  threshold?: number
  once?: boolean
}

export function useLazyLoad(
  callback: (entries: IntersectionObserverEntry[]) => void,
  options: LazyLoadOptions = {}
) {
  const { rootMargin = '50px', threshold = 0.1, once = true } = options

  const observer = ref<IntersectionObserver | null>(null)
  const elements = ref<HTMLElement[]>([])

  const observe = (element: HTMLElement) => {
    if (!observer.value) return

    elements.value.push(element)
    observer.value.observe(element)
  }

  const unobserve = (element: HTMLElement) => {
    if (!observer.value) return

    observer.value.unobserve(element)
    elements.value = elements.value.filter(el => el !== element)
  }

  const disconnect = () => {
    if (!observer.value) return

    observer.value.disconnect()
    observer.value = null
    elements.value = []
  }

  onMounted(() => {
    if (typeof IntersectionObserver === 'undefined') {
      console.warn('IntersectionObserver is not supported')
      return
    }

    observer.value = new IntersectionObserver((entries) => {
      callback(entries)

      if (once) {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            unobserve(entry.target as HTMLElement)
          }
        })
      }
    }, {
      rootMargin,
      threshold
    })
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    observe,
    unobserve,
    disconnect
  }
}

// =============================================================================
// Component Lazy Loading
// =============================================================================

export function useComponentLazyLoad() {
  const loadedComponents = ref(new Set<string>())
  const loadingComponents = ref(new Set<string>())

  const loadComponent = async <T>(
    componentLoader: () => Promise<T>,
    componentId: string
  ): Promise<T | null> => {
    if (loadedComponents.value.has(componentId)) {
      return null // Component already loaded
    }

    if (loadingComponents.value.has(componentId)) {
      return null // Component is currently loading
    }

    loadingComponents.value.add(componentId)

    try {
      const component = await componentLoader()
      loadedComponents.value.add(componentId)
      return component
    } catch (error) {
      console.error(`Failed to load component ${componentId}:`, error)
      return null
    } finally {
      loadingComponents.value.delete(componentId)
    }
  }

  const preloadComponent = async <T>(
    componentLoader: () => Promise<T>,
    componentId: string
  ) => {
    // Only preload if not already loaded
    if (!loadedComponents.value.has(componentId)) {
      await loadComponent(componentLoader, componentId)
    }
  }

  const isComponentLoaded = (componentId: string): boolean => {
    return loadedComponents.value.has(componentId)
  }

  const isComponentLoading = (componentId: string): boolean => {
    return loadingComponents.value.has(componentId)
  }

  return {
    loadComponent,
    preloadComponent,
    isComponentLoaded,
    isComponentLoading
  }
}

// =============================================================================
// Data Prefetching
// =============================================================================

export interface PrefetchOptions {
  priority?: 'high' | 'normal' | 'low'
  timeout?: number
  retryCount?: number
  retryDelay?: number
}

export function useDataPrefetch() {
  const prefetchQueue = ref<Array<{
    id: string
    fetcher: () => Promise<any>
    options: PrefetchOptions
    resolve: (value: any) => void
    reject: (error: Error) => void
  }>>([])

  const prefetchCache = ref(new Map<string, any>())
  const isProcessing = ref(false)

  const prefetch = async <T>(
    id: string,
    fetcher: () => Promise<T>,
    options: PrefetchOptions = {}
  ): Promise<T> => {
    // Return cached result if available
    if (prefetchCache.value.has(id)) {
      return prefetchCache.value.get(id)
    }

    return new Promise((resolve, reject) => {
      prefetchQueue.value.push({
        id,
        fetcher,
        options: {
          priority: 'normal',
          timeout: 10000,
          retryCount: 2,
          retryDelay: 1000,
          ...options
        },
        resolve,
        reject
      })

      processQueue()
    })
  }

  const processQueue = async () => {
    if (isProcessing.value || prefetchQueue.value.length === 0) {
      return
    }

    isProcessing.value = true

    // Sort by priority
    prefetchQueue.value.sort((a, b) => {
      const priorityOrder = { high: 3, normal: 2, low: 1 }
      return (priorityOrder[b.options.priority!] || 2) - (priorityOrder[a.options.priority!] || 2)
    })

    const tasks = [...prefetchQueue.value]
    prefetchQueue.value = []

    await Promise.allSettled(
      tasks.map(task => executePrefetchTask(task))
    )

    isProcessing.value = false

    // Process next batch if queue has items
    if (prefetchQueue.value.length > 0) {
      nextTick(() => processQueue())
    }
  }

  const executePrefetchTask = async (task: any) => {
    const { id, fetcher, options, resolve, reject } = task
    let retryCount = 0

    while (retryCount <= (options.retryCount || 2)) {
      try {
        const timeoutPromise = new Promise<never>((_, timeoutReject) => {
          setTimeout(() => timeoutReject(new Error('Prefetch timeout')), options.timeout)
        })

        const result = await Promise.race([fetcher(), timeoutPromise])

        prefetchCache.value.set(id, result)
        resolve(result)
        return

      } catch (error) {
        retryCount++

        if (retryCount > (options.retryCount || 2)) {
          reject(error as Error)
          return
        }

        // Wait before retry
        await new Promise(resolve => setTimeout(resolve, options.retryDelay))
      }
    }
  }

  const clearCache = (pattern?: string) => {
    if (pattern) {
      const regex = new RegExp(pattern)
      for (const key of prefetchCache.value.keys()) {
        if (regex.test(key)) {
          prefetchCache.value.delete(key)
        }
      }
    } else {
      prefetchCache.value.clear()
    }
  }

  const getCacheStats = () => {
    return {
      size: prefetchCache.value.size,
      queueLength: prefetchQueue.value.length,
      isProcessing: isProcessing.value
    }
  }

  return {
    prefetch,
    clearCache,
    getCacheStats
  }
}

// =============================================================================
// Memoization
// =============================================================================

export function useMemoize<TArgs extends any[], TReturn>(
  fn: (...args: TArgs) => TReturn,
  keyGenerator?: (...args: TArgs) => string
) {
  const cache = new Map<string, TReturn>()

  const memoized = (...args: TArgs): TReturn => {
    const key = keyGenerator ? keyGenerator(...args) : JSON.stringify(args)

    if (cache.has(key)) {
      return cache.get(key)!
    }

    const result = fn(...args)
    cache.set(key, result)

    // Limit cache size
    if (cache.size > 100) {
      const firstKey = cache.keys().next().value
      cache.delete(firstKey)
    }

    return result
  }

  const clear = () => {
    cache.clear()
  }

  return { memoized, clear }
}

// =============================================================================
// Performance Monitoring
// =============================================================================

export interface PerformanceMetrics {
  componentName: string
  renderTime: number
  updateTime: number
  memoryUsage?: number
  timestamp: number
}

export function usePerformanceMonitor(componentName: string) {
  const metrics = ref<PerformanceMetrics[]>([])
  const isMonitoring = ref(false)

  const startMonitoring = () => {
    isMonitoring.value = true
  }

  const stopMonitoring = () => {
    isMonitoring.value = false
  }

  const recordRender = (renderTime: number) => {
    if (!isMonitoring.value) return

    metrics.value.push({
      componentName,
      renderTime,
      updateTime: 0,
      timestamp: Date.now()
    })

    // Keep only last 50 metrics
    if (metrics.value.length > 50) {
      metrics.value = metrics.value.slice(-50)
    }
  }

  const recordUpdate = (updateTime: number) => {
    if (!isMonitoring.value) return

    const lastMetric = metrics.value[metrics.value.length - 1]
    if (lastMetric) {
      lastMetric.updateTime = updateTime
    }
  }

  const getAverageRenderTime = () => {
    if (metrics.value.length === 0) return 0
    return metrics.value.reduce((sum, metric) => sum + metric.renderTime, 0) / metrics.value.length
  }

  const getMetrics = () => metrics.value

  const clearMetrics = () => {
    metrics.value = []
  }

  return {
    isMonitoring,
    startMonitoring,
    stopMonitoring,
    recordRender,
    recordUpdate,
    getAverageRenderTime,
    getMetrics,
    clearMetrics
  }
}

// =============================================================================
// Memory Optimization
// =============================================================================

export function useMemoryOptimization() {
  const weakReferences = new WeakMap()
  const cache = new Map()

  const setWeakReference = (key: object, value: any) => {
    weakReferences.set(key, value)
  }

  const getWeakReference = (key: object): any => {
    return weakReferences.get(key)
  }

  const setCache = (key: string, value: any, ttl: number = 300000) => {
    cache.set(key, {
      value,
      expires: Date.now() + ttl
    })
  }

  const getCache = (key: string): any => {
    const item = cache.get(key)
    if (!item) return null

    if (Date.now() > item.expires) {
      cache.delete(key)
      return null
    }

    return item.value
  }

  const cleanExpiredCache = () => {
    const now = Date.now()
    for (const [key, item] of cache.entries()) {
      if (now > item.expires) {
        cache.delete(key)
      }
    }
  }

  const getMemoryUsage = () => {
    if ('memory' in performance) {
      const memory = (performance as any).memory
      return {
        used: memory.usedJSHeapSize,
        total: memory.totalJSHeapSize,
        limit: memory.jsHeapSizeLimit
      }
    }
    return null
  }

  // Auto-cleanup expired cache entries every 5 minutes
  setInterval(cleanExpiredCache, 300000)

  return {
    setWeakReference,
    getWeakReference,
    setCache,
    getCache,
    cleanExpiredCache,
    getMemoryUsage
  }
}

// =============================================================================
// Debounced and Throttled Utilities
// =============================================================================

export function useDebounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number = 300
): (...args: Parameters<T>) => void {
  return debounce(fn, delay)
}

export function useThrottle<T extends (...args: any[]) => any>(
  fn: T,
  delay: number = 300
): (...args: Parameters<T>) => void {
  return throttle(fn, delay)
}

// =============================================================================
// Image Optimization
// =============================================================================

export function useImageOptimization() {
  const imageCache = new Map<string, HTMLImageElement>()

  const preloadImage = (src: string): Promise<HTMLImageElement> => {
    return new Promise((resolve, reject) => {
      if (imageCache.has(src)) {
        resolve(imageCache.get(src)!)
        return
      }

      const img = new Image()
      img.onload = () => {
        imageCache.set(src, img)
        resolve(img)
      }
      img.onerror = reject
      img.src = src
    })
  }

  const preloadImages = async (sources: string[]): Promise<HTMLImageElement[]> => {
    const promises = sources.map(src => preloadImage(src))
    return Promise.all(promises)
  }

  const generateResponsiveSrc = (
    src: string,
    sizes: number[],
    format?: 'webp' | 'avif' | 'auto'
  ): string[] => {
    return sizes.map(size => {
      const extension = format === 'auto' ? src.split('.').pop() : format
      return `${src}?w=${size}&format=${extension} ${size}w`
    })
  }

  return {
    preloadImage,
    preloadImages,
    generateResponsiveSrc
  }
}

// =============================================================================
// Performance Optimization Composable
// =============================================================================

export function usePerformanceOptimization(config: Partial<PerformanceConfig> = {}) {
  const performanceConfig = { ...DEFAULT_PERFORMANCE_CONFIG, ...config }

  // Initialize all optimization utilities
  const memoryOptimization = useMemoryOptimization()
  const performanceMonitor = usePerformanceOptimization('App')
  const imageOptimization = useImageOptimization()
  const dataPrefetch = useDataPrefetch()

  // Auto-start monitoring in development
  if (performanceConfig.enablePerformanceMonitoring && process.env.NODE_ENV === 'development') {
    performanceMonitor.startMonitoring()
  }

  const optimizeComponent = (componentName: string) => {
    const monitor = usePerformanceMonitor(componentName)

    return {
      monitor,
      lazyLoad: useComponentLazyLoad(),
      virtualScroll: useVirtualScroll,
      debounce: useDebounce,
      throttle: useThrottle
    }
  }

  const getOptimizationStats = () => {
    return {
      memory: memoryOptimization.getMemoryUsage(),
      prefetch: dataPrefetch.getCacheStats(),
      performance: {
        averageRenderTime: performanceMonitor.getAverageRenderTime(),
        metricsCount: performanceMonitor.getMetrics().length
      }
    }
  }

  return {
    config: performanceConfig,
    memoryOptimization,
    performanceMonitor,
    imageOptimization,
    dataPrefetch,
    optimizeComponent,
    getOptimizationStats,
    useVirtualScroll,
    useLazyLoad,
    useComponentLazyLoad,
    useMemoize,
    useDebounce,
    useThrottle
  }
}

console.log('⚡ [Performance] Performance optimization utilities loaded')