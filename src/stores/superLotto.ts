// Enhanced Super Lotto Pinia Store
// Implements advanced state management patterns with proper typing, error handling, and performance optimization

import { defineStore } from 'pinia'
import { ref, computed, reactive, watch } from 'vue'
import type {
  SuperLottoDraw,
  PredictionResult,
  HotNumberAnalysis,
  ColdNumberAnalysis,
  PatternAnalysis,
  BatchPredictionRequest,
  BatchPredictionResult,
  AlgorithmId,
  AlgorithmConfig,
  LoadingState,
  ErrorState,
  SelectionState,
  FilterState,
  PaginationParams,
  SearchParams,
  ErrorInfo
} from '@/types/superLotto'
import { VALIDATION_RULES } from '@/constants/lottery'

import { useSuperLottoApi } from '@/api/superLotto'
import { useErrorHandler } from '@/utils/errorHandler'

// =============================================================================
// Store Configuration
// =============================================================================

interface SuperLottoStoreConfig {
  enableAutoRefresh: boolean
  autoRefreshInterval: number
  maxCacheSize: number
  enablePersistence: boolean
  debugMode: boolean
}

const DEFAULT_CONFIG: SuperLottoStoreConfig = {
  enableAutoRefresh: false,
  autoRefreshInterval: 300000, // 5 minutes
  maxCacheSize: 1000,
  enablePersistence: true,
  debugMode: false
}

interface AlgorithmStats {
  algorithm_id: AlgorithmId
  algorithm_name: string
  count: number
  total_confidence: number
  average_confidence: number
  max_confidence: number
  validated_count: number
  average_accuracy: number
}

// =============================================================================
// Store Definition
// =============================================================================

export const useSuperLottoStore = defineStore('superLotto', () => {
  // =============================================================================
  // Dependencies
  // =============================================================================
  const api = useSuperLottoApi()
  const errorHandler = useErrorHandler()

  // =============================================================================
  // Configuration
  // =============================================================================
  const config = reactive<SuperLottoStoreConfig>({ ...DEFAULT_CONFIG })

  // =============================================================================
  // Core State
  // =============================================================================

  // Loading state management
  const loadingState = reactive<LoadingState>({
    loading: false,
    loading_text: undefined,
    progress: undefined,
    cancellable: false
  })

  // Error state management
  const errorState = reactive<ErrorState>({
    has_error: false,
    error_message: undefined,
    error_code: undefined,
    retry_count: 0,
    can_retry: false
  })

  // Data state
  const draws = ref<SuperLottoDraw[]>([])
  const totalDraws = ref(0)
  const hotNumbers = ref<HotNumberAnalysis[]>([])
  const coldNumbers = ref<ColdNumberAnalysis[]>([])
  const patterns = ref<PatternAnalysis[]>([])
  const predictions = ref<PredictionResult[]>([])
  const algorithms = ref<AlgorithmConfig[]>([])

  // UI State
  const selectionState = reactive<SelectionState<SuperLottoDraw>>({
    selected_items: [],
    selected_ids: [],
    last_selected: undefined,
    selection_mode: 'single'
  })

  const filterState = reactive<FilterState>({
    active_filters: [],
    saved_filters: [],
    current_preset: undefined
  })

  // Pagination state
  const pagination = reactive({
    page: 1,
    limit: 100,
    total: 0,
    has_next: false,
    has_prev: false
  })

  // Search state
  const searchState = reactive({
    query: '',
    active_search: false,
    search_results: [] as SuperLottoDraw[],
    search_suggestions: [] as string[]
  })

  // =============================================================================
  // Cache Management
  // =============================================================================
  const cache = new Map<string, CacheEntry>()

  interface CacheEntry {
    data: any
    timestamp: number
    ttl: number
  }

  const setCache = (key: string, data: any, ttl: number = 300000) => {
    cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl
    })

    // Clean up old cache entries
    if (cache.size > config.maxCacheSize) {
      const oldestKey = Array.from(cache.keys())[0]
      cache.delete(oldestKey)
    }
  }

  const getCache = (key: string) => {
    const entry = cache.get(key)
    if (!entry) return null

    if (Date.now() - entry.timestamp > entry.ttl) {
      cache.delete(key)
      return null
    }

    return entry.data
  }

  const clearCache = (pattern?: string) => {
    if (pattern) {
      const regex = new RegExp(pattern)
      for (const key of cache.keys()) {
        if (regex.test(key)) {
          cache.delete(key)
        }
      }
    } else {
      cache.clear()
    }
  }

  // =============================================================================
  // Computed Properties
  // =============================================================================

  const isLoading = computed(() => loadingState.loading)
  const hasError = computed(() => errorState.has_error)
  const errorMessage = computed(() => errorState.error_message)
  const canRetry = computed(() => errorState.can_retry && errorState.retry_count < 3)

  const isDataLoaded = computed(() => draws.value.length > 0)
  const hasPredictions = computed(() => predictions.value.length > 0)
  const hasAnalysisData = computed(() =>
    hotNumbers.value.length > 0 ||
    coldNumbers.value.length > 0 ||
    patterns.value.length > 0
  )

  const filteredDraws = computed(() => {
    let filtered = [...draws.value]

    // Apply search filter
    if (searchState.query) {
      filtered = filtered.filter(draw =>
        draw.draw_number.toString().includes(searchState.query) ||
        draw.front_numbers.some((n: number) => n.toString().includes(searchState.query)) ||
        draw.back_numbers.some((n: number) => n.toString().includes(searchState.query))
      )
    }

    // Apply date filters
    filterState.active_filters.forEach(filter => {
      if (filter.date_range) {
        filtered = filtered.filter(draw => {
          const drawDate = new Date(draw.draw_date)
          const startDate = new Date(filter.date_range!.start_date)
          const endDate = new Date(filter.date_range!.end_date)
          return drawDate >= startDate && drawDate <= endDate
        })
      }
    })

    // Apply algorithm filters
    if (filterState.active_filters.some(f => f.algorithm_ids)) {
      const algorithmFilter = filterState.active_filters.find(f => f.algorithm_ids)
      if (algorithmFilter?.algorithm_ids) {
        // Filter based on predictions for these draws
        filtered = filtered.filter(draw => {
          return predictions.value.some(prediction =>
            prediction.front_numbers[0] === draw.front_numbers[0] && // Simple match using first number
            algorithmFilter.algorithm_ids!.includes(prediction.algorithm_id)
          )
        })
      }
    }

    return filtered
  })

  const paginatedDraws = computed(() => {
    const start = (pagination.page - 1) * pagination.limit
    const end = start + pagination.limit
    return filteredDraws.value.slice(start, end)
  })

  const totalPages = computed(() =>
    Math.ceil(filteredDraws.value.length / pagination.limit)
  )

  const validatedPredictions = computed(() =>
    predictions.value.filter(p => p.is_validated)
  )

  const averagePredictionAccuracy = computed(() => {
    const validated = validatedPredictions.value
    if (validated.length === 0) return 0

    const totalAccuracy = validated.reduce((sum, p) => sum + (p.accuracy || 0), 0)
    return totalAccuracy / validated.length
  })

  const bestPrediction = computed(() => {
    if (predictions.value.length === 0) return null
    return predictions.value.reduce((best, current) =>
      (current.confidence_score > best.confidence_score) ? current : best
    )
  })

  const algorithmStats = computed(() => {
    const stats = new Map<AlgorithmId, AlgorithmStats>()

    predictions.value.forEach(prediction => {
      if (!stats.has(prediction.algorithm_id)) {
        stats.set(prediction.algorithm_id, {
          algorithm_id: prediction.algorithm_id,
          algorithm_name: prediction.algorithm,
          count: 0,
          total_confidence: 0,
          average_confidence: 0,
          max_confidence: 0,
          validated_count: 0,
          average_accuracy: 0
        })
      }

      const stat = stats.get(prediction.algorithm_id)!
      stat.count++
      stat.total_confidence += prediction.confidence_score
      stat.max_confidence = Math.max(stat.max_confidence, prediction.confidence_score)
      stat.average_confidence = stat.total_confidence / stat.count

      if (prediction.is_validated && prediction.accuracy) {
        stat.validated_count++
        stat.average_accuracy = ((stat.average_accuracy * (stat.validated_count - 1)) + prediction.accuracy) / stat.validated_count
      }
    })

    return Array.from(stats.values())
  })

  // =============================================================================
  // Loading State Management
  // =============================================================================

  const setLoading = (loading: boolean, text?: string, progress?: number, cancellable?: boolean) => {
    loadingState.loading = loading
    loadingState.loading_text = text
    loadingState.progress = progress
    loadingState.cancellable = cancellable || false
  }

  const withLoading = async <T>(operation: () => Promise<T>, loadingText?: string): Promise<T> => {
    setLoading(true, loadingText)
    try {
      return await operation()
    } finally {
      setLoading(false)
    }
  }

  // =============================================================================
  // Error State Management
  // =============================================================================

  const setError = (error: ErrorInfo) => {
    errorState.has_error = true
    errorState.error_message = error.message
    errorState.error_code = error.code
    errorState.can_retry = error.recoverable
  }

  const clearError = () => {
    errorState.has_error = false
    errorState.error_message = undefined
    errorState.error_code = undefined
    errorState.retry_count = 0
    errorState.can_retry = false
  }

  const incrementRetryCount = () => {
    errorState.retry_count++
  }

  // =============================================================================
  // Data Operations
  // =============================================================================

  const fetchDraws = async (params: PaginationParams & SearchParams = {}) => {
    const cacheKey = `draws:${JSON.stringify(params)}`
    const cached = getCache(cacheKey)
    if (cached) {
      draws.value = cached.draws
      totalDraws.value = cached.total
      return cached
    }

    return withLoading(async () => {
      clearError()

      try {
        const result = await api.getDraws(params)
        if (result) {
          draws.value = result.draws
          totalDraws.value = result.total

          // Update pagination
          pagination.limit = params.limit || 100
          pagination.page = Math.floor(((params as any).offset || 0) / pagination.limit) + 1
          pagination.total = result.total
          pagination.has_next = pagination.page * pagination.limit < result.total
          pagination.has_prev = pagination.page > 1

          // Cache result
          setCache(cacheKey, result)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '加载开奖数据...')
  }

  const fetchPredictions = async (params: SearchParams = {}) => {
    return withLoading(async () => {
      clearError()

      try {
        const result = await api.getPredictions(params)
        if (result) {
          predictions.value = result.predictions
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '加载预测数据...')
  }

  const analyzeHotNumbers = async (params: { days: number; zone?: string }) => {
    const cacheKey = `hot_numbers:${JSON.stringify(params)}`
    const cached = getCache(cacheKey)
    if (cached) {
      hotNumbers.value = cached
      return cached
    }

    return withLoading(async () => {
      clearError()

      try {
        const result = await api.analyzeHotNumbers(params)
        if (result) {
          hotNumbers.value = result
          setCache(cacheKey, result)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '分析热号...')
  }

  const analyzeColdNumbers = async (params: { days: number; zone?: string }) => {
    const cacheKey = `cold_numbers:${JSON.stringify(params)}`
    const cached = getCache(cacheKey)
    if (cached) {
      coldNumbers.value = cached
      return cached
    }

    return withLoading(async () => {
      clearError()

      try {
        const result = await api.analyzeColdNumbers(params)
        if (result) {
          coldNumbers.value = result
          setCache(cacheKey, result)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '分析冷号...')
  }

  const analyzePatterns = async (params: { days: number; pattern_types?: string[] }) => {
    const cacheKey = `patterns:${JSON.stringify(params)}`
    const cached = getCache(cacheKey)
    if (cached) {
      patterns.value = cached
      return cached
    }

    return withLoading(async () => {
      clearError()

      try {
        const result = await api.analyzePatterns(params)
        if (result) {
          patterns.value = result
          setCache(cacheKey, result)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '分析模式...')
  }

  const generatePrediction = async (params: {
    algorithm: AlgorithmId
    analysis_period_days?: number
    custom_parameters?: Record<string, unknown>
    include_reasoning?: boolean
  }) => {
    return withLoading(async () => {
      clearError()

      try {
        const result = await api.generatePrediction(params)
        if (result) {
          predictions.value.unshift(result)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '生成预测...')
  }

  const generateBatchPredictions = async (request: BatchPredictionRequest) => {
    return withLoading(async () => {
      clearError()

      try {
        const result = await api.generateBatchPredictions(request)
        if (result) {
          predictions.value.unshift(...result.predictions)
        }
        return result
      } catch (error) {
        incrementRetryCount()
        throw error
      }
    }, '批量生成预测...')
  }

  // =============================================================================
  // Selection Management
  // =============================================================================

  const selectDraw = (draw: SuperLottoDraw, mode: 'single' | 'multiple' = 'single') => {
    selectionState.selection_mode = mode
    selectionState.last_selected = draw

    if (mode === 'single') {
      selectionState.selected_items = [draw]
      selectionState.selected_ids = [draw.id.toString()]
    } else {
      const id = draw.id.toString()
      const index = selectionState.selected_ids.indexOf(id)

      if (index > -1) {
        selectionState.selected_ids.splice(index, 1)
        selectionState.selected_items = selectionState.selected_items.filter(d => d.id !== draw.id)
      } else {
        selectionState.selected_ids.push(id)
        selectionState.selected_items.push(draw)
      }
    }
  }

  const clearSelection = () => {
    selectionState.selected_items = []
    selectionState.selected_ids = []
    selectionState.last_selected = undefined
  }

  const selectAll = () => {
    selectionState.selected_items = [...filteredDraws.value]
    selectionState.selected_ids = filteredDraws.value.map(d => d.id.toString())
  }

  // =============================================================================
  // Filter Management
  // =============================================================================

  const addFilter = (filter: any) => {
    filterState.active_filters.push(filter)
    clearCache() // Clear cache when filters change
  }

  const removeFilter = (index: number) => {
    filterState.active_filters.splice(index, 1)
    clearCache() // Clear cache when filters change
  }

  const clearFilters = () => {
    filterState.active_filters = []
    filterState.current_preset = undefined
    clearCache()
  }

  // =============================================================================
  // Search Management
  // =============================================================================

  const search = (query: string) => {
    searchState.query = query
    searchState.active_search = query.length > 0
  }

  const clearSearch = () => {
    searchState.query = ''
    searchState.active_search = false
    searchState.search_results = []
  }

  // =============================================================================
  // Pagination Management
  // =============================================================================

  const setPage = (page: number) => {
    pagination.page = Math.max(1, Math.min(page, totalPages.value))
  }

  const nextPage = () => {
    if (pagination.has_next) {
      setPage(pagination.page + 1)
    }
  }

  const prevPage = () => {
    if (pagination.has_prev) {
      setPage(pagination.page - 1)
    }
  }

  const setLimit = (limit: number) => {
    pagination.limit = limit
    pagination.page = 1
  }

  // =============================================================================
  // Data Management Utilities
  // =============================================================================

  const validateDrawNumbers = (redNumbers: number[], blueNumber: number) => {
    const errors: string[] = []

    // Validate red numbers
    if (redNumbers.length !== 5) {
      errors.push(`前区号码数量必须为5个`)
    }

    if (new Set(redNumbers).size !== redNumbers.length) {
      errors.push('前区号码不能重复')
    }

    for (const num of redNumbers) {
      if (num < VALIDATION_RULES.FRONT_NUMBER_RANGE.min ||
          num > VALIDATION_RULES.FRONT_NUMBER_RANGE.max) {
        errors.push(`前区号码必须在${VALIDATION_RULES.FRONT_NUMBER_RANGE.min}-${VALIDATION_RULES.FRONT_NUMBER_RANGE.max}范围内`)
      }
    }

    // Validate blue number
    if (blueNumber < VALIDATION_RULES.BACK_NUMBER_RANGE.min ||
        blueNumber > VALIDATION_RULES.BACK_NUMBER_RANGE.max) {
      errors.push(`后区号码必须在${VALIDATION_RULES.BACK_NUMBER_RANGE.min}-${VALIDATION_RULES.BACK_NUMBER_RANGE.max}范围内`)
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }

  const formatDrawDate = (date: string | Date, format: 'short' | 'long' = 'short') => {
    const d = typeof date === 'string' ? new Date(date) : date

    if (format === 'short') {
      return d.toLocaleDateString('zh-CN')
    } else {
      return d.toLocaleString('zh-CN')
    }
  }

  const calculateDrawStatistics = (draws: SuperLottoDraw[]) => {
    if (draws.length === 0) return null

    const stats = {
      totalDraws: draws.length,
      dateRange: {
        earliest: formatDrawDate(draws[draws.length - 1].draw_date),
        latest: formatDrawDate(draws[0].draw_date)
      },
      redNumberFrequency: new Map<number, number>(),
      blueNumberFrequency: new Map<number, number>(),
      averageJackpot: 0,
      totalJackpot: 0
    }

    let totalJackpot = 0
    let jackpotCount = 0

    draws.forEach(draw => {
      // Count red numbers
      draw.front_numbers.forEach((num: number) => {
        stats.redNumberFrequency.set(num, (stats.redNumberFrequency.get(num) || 0) + 1)
      })

      // Count blue numbers
      draw.back_numbers.forEach((num: number) => {
        stats.blueNumberFrequency.set(num, (stats.blueNumberFrequency.get(num) || 0) + 1)
      })

      // Sum jackpot (using prize_pool as equivalent)
      if (draw.prize_pool) {
        totalJackpot += draw.prize_pool
        jackpotCount++
      }
    })

    stats.averageJackpot = jackpotCount > 0 ? totalJackpot / jackpotCount : 0
    stats.totalJackpot = totalJackpot

    return stats
  }

  // =============================================================================
  // Store Reset
  // =============================================================================

  const resetStore = () => {
    draws.value = []
    totalDraws.value = 0
    hotNumbers.value = []
    coldNumbers.value = []
    patterns.value = []
    predictions.value = []
    algorithms.value = []

    clearError()
    clearSelection()
    clearFilters()
    clearSearch()
    clearCache()

    pagination.page = 1
    pagination.limit = 100
    pagination.total = 0
    pagination.has_next = false
    pagination.has_prev = false
  }

  // =============================================================================
  // Persistence
  // =============================================================================

  const saveState = () => {
    if (!config.enablePersistence) return

    const stateToSave = {
      selectionState: selectionState.selected_ids,
      filterState: filterState.active_filters,
      searchState: searchState.query,
      pagination: {
        page: pagination.page,
        limit: pagination.limit
      }
    }

    localStorage.setItem('superLottoStore', JSON.stringify(stateToSave))
  }

  const loadState = () => {
    if (!config.enablePersistence) return

    try {
      const savedState = localStorage.getItem('superLottoStore')
      if (savedState) {
        const state = JSON.parse(savedState)

        if (state.selectionState) {
          selectionState.selected_ids = state.selectionState
        }

        if (state.filterState) {
          filterState.active_filters = state.filterState
        }

        if (state.searchState) {
          searchState.query = state.searchState
        }

        if (state.pagination) {
          pagination.page = state.pagination.page || 1
          pagination.limit = state.pagination.limit || 100
        }
      }
    } catch (error) {
      console.warn('Failed to load saved state:', error)
    }
  }

  // =============================================================================
  // Auto-refresh
  // =============================================================================

  let autoRefreshTimer: number | null = null

  const startAutoRefresh = () => {
    if (!config.enableAutoRefresh) return

    stopAutoRefresh()
    autoRefreshTimer = setInterval(async () => {
      if (!isLoading.value) {
        try {
          await fetchDraws({ limit: pagination.limit })
        } catch (error) {
          console.warn('Auto-refresh failed:', error)
        }
      }
    }, config.autoRefreshInterval)
  }

  const stopAutoRefresh = () => {
    if (autoRefreshTimer) {
      clearInterval(autoRefreshTimer)
      autoRefreshTimer = null
    }
  }

  // =============================================================================
  // Watchers
  // =============================================================================

  watch(
    () => [selectionState.selected_ids, filterState.active_filters, searchState.query, pagination.page, pagination.limit],
    saveState,
    { deep: true }
  )

  // Auto-clear cache when data changes
  watch(
    () => [draws.value.length, predictions.value.length],
    () => {
      if (config.debugMode) {
        console.log('Data changed, clearing cache')
      }
      clearCache()
    }
  )

  // =============================================================================
  // Return Store Interface
  // =============================================================================

  return {
    // Configuration
    config,

    // State
    loadingState,
    errorState,
    draws,
    totalDraws,
    hotNumbers,
    coldNumbers,
    patterns,
    predictions,
    algorithms,
    selectionState,
    filterState,
    searchState,
    pagination,

    // Computed
    isLoading,
    hasError,
    errorMessage,
    canRetry,
    isDataLoaded,
    hasPredictions,
    hasAnalysisData,
    filteredDraws,
    paginatedDraws,
    totalPages,
    validatedPredictions,
    averagePredictionAccuracy,
    bestPrediction,
    algorithmStats,

    // Loading Management
    setLoading,
    withLoading,

    // Error Management
    setError,
    clearError,
    incrementRetryCount,

    // Data Operations
    fetchDraws,
    fetchPredictions,
    analyzeHotNumbers,
    analyzeColdNumbers,
    analyzePatterns,
    generatePrediction,
    generateBatchPredictions,

    // Selection Management
    selectDraw,
    clearSelection,
    selectAll,

    // Filter Management
    addFilter,
    removeFilter,
    clearFilters,

    // Search Management
    search,
    clearSearch,

    // Pagination Management
    setPage,
    nextPage,
    prevPage,
    setLimit,

    // Utilities
    validateDrawNumbers,
    formatDrawDate,
    calculateDrawStatistics,

    // Cache Management
    setCache,
    getCache,
    clearCache,

    // Auto-refresh
    startAutoRefresh,
    stopAutoRefresh,

    // Store Management
    resetStore,
    saveState,
    loadState
  }
})

// =============================================================================
// Store Initialization
// =============================================================================

export const initializeSuperLottoStore = async () => {
  const store = useSuperLottoStore()

  // Load persisted state
  store.loadState()

  // Start auto-refresh if enabled
  store.startAutoRefresh()

  // Initial data load
  try {
    await store.fetchDraws({ limit: 100 })
  } catch (error) {
    console.warn('Failed to load initial data:', error)
  }

  console.log('🎯 [Super Lotto Store] Enhanced store initialized successfully')
}
