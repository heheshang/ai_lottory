/**
 * Vue composable for debounced operations with performance tracking
 */

import { ref, onUnmounted, computed, watch, readonly, type Ref } from 'vue'
import { debounce, throttle, DebounceManager, DEBOUNCE_SETTINGS } from '@/utils/debounce'
import type { DebounceConfig, DebounceFunction } from '@/utils/debounce'
import { usePerformanceStore } from '@/stores/performance'

export interface DebounceTrackingConfig {
  id: string
  delay: number
  debounceOptions?: DebounceConfig
  trackPerformance?: boolean
  maxConcurrent?: number
}

export interface DebounceStats {
  id: string
  calls: number
  executions: number
  cancelled: number
  averageWaitTime: number
  maxWaitTime: number
  currentPending: boolean
  executionTimes: number[]
}

export function useDebounceWithTracking<T extends (...args: any[]) => any>(
  func: T,
  config: DebounceTrackingConfig
) {
  const performanceStore = usePerformanceStore()

  // Tracking state
  const stats = ref<DebounceStats>({
    id: config.id,
    calls: 0,
    executions: 0,
    cancelled: 0,
    averageWaitTime: 0,
    maxWaitTime: 0,
    currentPending: false,
    executionTimes: []
  })

  // Internal tracking variables
  let lastCallTime = 0
  let pendingCallTime = 0
  let concurrentCount = 0
  const executionTimes: number[] = []

  // Create debounced function with tracking
  const debouncedFunc = debounce((...args: Parameters<T>) => {
    const now = Date.now()
    const waitTime = now - lastCallTime
    lastCallTime = now

    // Update stats
    stats.value.calls++
    stats.value.currentPending = true
    pendingCallTime = now

    // Track concurrent calls if limit is set
    if (config.maxConcurrent) {
      concurrentCount++
      if (concurrentCount > config.maxConcurrent) {
        stats.value.cancelled++
        stats.value.currentPending = false
        concurrentCount--
        return
      }
    }

    // Performance tracking
    if (config.trackPerformance) {
      performanceStore.recordInteraction(`${config.id}_debounced_call`)
    }

    const startTime = performance.now()

    try {
      // Execute the original function
      const result = func(...args)

      // Handle both sync and async functions
      if (result && typeof result.then === 'function') {
        return result
          .then((value: any) => {
            onExecutionComplete(startTime)
            return value
          })
          .catch((error: any) => {
            onExecutionComplete(startTime)
            throw error
          })
      } else {
        onExecutionComplete(startTime)
        return result
      }
    } catch (error) {
      onExecutionComplete(startTime)
      throw error
    }
  }, {
    delay: config.delay,
    ...config.debounceOptions
  })

  const onExecutionComplete = (startTime: number) => {
    const executionTime = performance.now() - startTime
    const totalWaitTime = Date.now() - pendingCallTime

    // Update stats
    stats.value.executions++
    stats.value.currentPending = false
    stats.value.averageWaitTime =
      (stats.value.averageWaitTime * (stats.value.executions - 1) + totalWaitTime) / stats.value.executions
    stats.value.maxWaitTime = Math.max(stats.value.maxWaitTime, totalWaitTime)

    // Track execution times
    executionTimes.push(executionTime)
    stats.value.executionTimes = [...executionTimes.slice(-100)] // Keep last 100 execution times

    // Reset concurrent count
    if (config.maxConcurrent) {
      concurrentCount--
    }

    // Performance tracking
    if (config.trackPerformance) {
      performanceStore.recordRenderTime(config.id, executionTime)
      performanceStore.recordApiTime(`${config.id}_execution`, executionTime)
    }
  }

  const cancel = () => {
    if (stats.value.currentPending) {
      stats.value.cancelled++
      stats.value.currentPending = false

      if (config.maxConcurrent && concurrentCount > 0) {
        concurrentCount--
      }
    }
    debouncedFunc.cancel()
  }

  const flush = () => {
    if (stats.value.currentPending) {
      stats.value.currentPending = false
    }
    return debouncedFunc.flush()
  }

  const reset = () => {
    cancel()
    stats.value = {
      id: config.id,
      calls: 0,
      executions: 0,
      cancelled: 0,
      averageWaitTime: 0,
      maxWaitTime: 0,
      currentPending: false,
      executionTimes: []
    }
    executionTimes.length = 0
    concurrentCount = 0
  }

  // Computed properties
  const successRate = computed(() => {
    return stats.value.calls > 0 ? (stats.value.executions / stats.value.calls) * 100 : 0
  })

  const cancellationRate = computed(() => {
    return stats.value.calls > 0 ? (stats.value.cancelled / stats.value.calls) * 100 : 0
  })

  const averageExecutionTime = computed(() => {
    const times = stats.value.executionTimes
    return times.length > 0 ? times.reduce((sum, time) => sum + time, 0) / times.length : 0
  })

  const isOverloaded = computed(() => {
    return config.maxConcurrent ? concurrentCount >= config.maxConcurrent : false
  })

  // Cleanup on unmount
  onUnmounted(() => {
    cancel()
  })

  return {
    // Debounced function
    debounced: debouncedFunc as DebounceFunction<T>,

    // Control methods
    cancel,
    flush,
    reset,

    // Reactive stats
    stats: readonly(stats),
    successRate,
    cancellationRate,
    averageExecutionTime,
    isOverloaded,

    // Utilities
    isPending: computed(() => stats.value.currentPending)
  }
}

/**
 * Composable for managing multiple debounced operations
 */
export function useDebounceManager() {
  const performanceStore = usePerformanceStore()
  const manager = new DebounceManager()

  // Global tracking for all managed debounces
  const allStats = ref<Record<string, DebounceStats>>({})

  const createTrackedDebounce = <T extends (...args: any[]) => any>(
    id: string,
    func: T,
    config: Omit<DebounceTrackingConfig, 'id'>
  ) => {
    const { debounced, stats, ...rest } = useDebounceWithTracking(func, { ...config, id })

    // Track stats in global manager
    allStats.value[id] = stats.value
    watch(stats, (newStats) => {
      allStats.value[id] = newStats
    }, { deep: true })

    return { debounced, stats, ...rest }
  }

  const getStats = (id: string) => {
    return allStats.value[id]
  }

  const getAllStats = () => {
    return allStats.value
  }

  const clearStats = (id?: string) => {
    if (id) {
      delete allStats.value[id]
      manager.remove(id)
    } else {
      allStats.value = {}
      manager.clear()
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    manager.clear()
  })

  return {
    manager,
    createTrackedDebounce,
    getStats,
    getAllStats,
    clearStats,
    allStats: readonly(allStats)
  }
}

/**
 * Specialized composable for search with debouncing and caching
 */
export function useSearchWithDebounce<T>(
  searchFn: (query: string) => Promise<T[]>,
  config: {
    delay?: number
    minQueryLength?: number
    maxResults?: number
    cacheResults?: boolean
    cacheTimeout?: number
  } = {}
) {
  const {
    delay = 300,
    minQueryLength = 2,
    maxResults = 50,
    cacheResults = true,
    cacheTimeout = 60000 // 1 minute
  } = config

  const performanceStore = usePerformanceStore()

  // State
  const results = ref<T[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const query = ref('')
  const lastSearchedQuery = ref('')

  // Cache for search results
  const cache = ref<Map<string, { data: T[]; timestamp: number }>>(new Map())

  const performSearch = async (searchQuery: string) => {
    if (searchQuery.length < minQueryLength && searchQuery.length > 0) {
      results.value = []
      return
    }

    // Check cache first
    if (cacheResults && cache.value.has(searchQuery)) {
      const cached = cache.value.get(searchQuery)!
      const now = Date.now()

      if (now - cached.timestamp < cacheTimeout) {
        results.value = cached.data.slice(0, maxResults)
        performanceStore.recordCacheHit('search')
        return
      } else {
        cache.value.delete(searchQuery)
      }
    }

    loading.value = true
    error.value = null
    lastSearchedQuery.value = searchQuery

    const startTime = performance.now()

    try {
      const searchResults = await searchFn(searchQuery)
      const limitedResults = searchResults.slice(0, maxResults)

      results.value = limitedResults

      // Cache results
      if (cacheResults) {
        cache.value.set(searchQuery, {
          data: searchResults,
          timestamp: Date.now()
        })
      }

      // Performance tracking
      const searchTime = performance.now() - startTime
      performanceStore.recordApiTime('search', searchTime)
      performanceStore.recordInteraction('search_performed')

    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Search failed'
      results.value = []
      performanceStore.recordInteraction('search_error')
    } finally {
      loading.value = false
    }
  }

  const { debounced, cancel, stats, successRate } = useDebounceWithTracking(performSearch, {
    id: 'search',
    delay,
    trackPerformance: true,
    debounceOptions: {
      leading: false,
      trailing: true
    }
  })

  // Debounced search function
  const search = (searchQuery: string) => {
    query.value = searchQuery
    debounced(searchQuery)
  }

  // Clear search and cache
  const clearSearch = () => {
    cancel()
    query.value = ''
    lastSearchedQuery.value = ''
    results.value = []
    error.value = null
    loading.value = false
  }

  // Clear cache
  const clearCache = () => {
    cache.value.clear()
  }

  // Get cached queries count
  const cacheSize = computed(() => cache.value.size)

  return {
    // State
    results,
    loading,
    error,
    query,
    lastSearchedQuery,

    // Methods
    search,
    clearSearch,
    clearCache,
    cancel,

    // Computed
    cacheSize,
    hasResults: computed(() => results.value.length > 0),
    isActive: computed(() => query.value.length >= minQueryLength),
    successRate,

    // Stats
    stats
  }
}

/**
 * Specialized composable for form input debouncing
 */
export function useFormInputDebounce<T>(
  value: Ref<T>,
  validator?: (value: T) => Promise<boolean>,
  config: {
    delay?: number
    validateOnChange?: boolean
    validateOnBlur?: boolean
  } = {}
) {
  const {
    delay = 500,
    validateOnChange = true,
    validateOnBlur = true
  } = config

  const performanceStore = usePerformanceStore()

  // State
  const debouncedValue = ref<T>(value.value)
  const isValidating = ref(false)
  const isValid = ref<boolean | null>(null)
  const validationError = ref<string | null>(null)

  // Debounced validation function
  const validateValue = async (val: T) => {
    if (!validator) return

    isValidating.value = true
    validationError.value = null

    const startTime = performance.now()

    try {
      const valid = await validator(val)
      isValid.value = valid

      const validationTime = performance.now() - startTime
      performanceStore.recordApiTime('form_validation', validationTime)

    } catch (err) {
      isValid.value = false
      validationError.value = err instanceof Error ? err.message : 'Validation failed'
    } finally {
      isValidating.value = false
    }
  }

  const { debounced: debouncedValidate, cancel } = useDebounceWithTracking(validateValue, {
    id: 'form_validation',
    delay,
    trackPerformance: true
  })

  // Watch for value changes
  watch(value, (newValue) => {
    debouncedValue.value = newValue

    if (validateOnChange) {
      debouncedValidate(newValue)
    }
  })

  const onBlur = () => {
    if (validateOnBlur) {
      validateValue(value.value)
    }
  }

  const reset = () => {
    cancel()
    isValid.value = null
    validationError.value = null
    isValidating.value = false
  }

  // Cleanup
  onUnmounted(() => {
    cancel()
  })

  return {
    // State
    debouncedValue,
    isValidating,
    isValid,
    validationError,

    // Methods
    onBlur,
    reset,
    validate: () => validateValue(value.value),

    // Computed
    hasError: computed(() => isValid.value === false),
    isPending: computed(() => isValidating.value)
  }
}