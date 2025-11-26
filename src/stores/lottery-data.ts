import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SuperLottoDraw, LotterySearchQuery, DrawStatistics } from '../types/lottery'
import { CacheManager } from '../utils/cache'

/**
 * Lottery Data Store - Raw lottery draw data management
 *
 * Responsibilities:
 * - Historical lottery draw storage and retrieval
 * - Data filtering and search operations
 * - Pagination and sorting
 * - Data import/export
 * - Draw validation
 */
export const useLotteryDataStore = defineStore('lottery-data', () => {
  // =============================================================================
  // Reactive State
  // =============================================================================

  // Data storage
  const draws = ref<SuperLottoDraw[]>([])
  const totalDraws = ref(0)

  // Search and filter state
  const searchQuery = ref('')
  const activeFilters = ref<LotterySearchQuery>({
    draw_date_range: null,
    number_range: null,
    jackpot_min: null,
    jackpot_max: null
  })

  // Sort state
  const sortBy = ref<'draw_date' | 'jackpot_amount' | 'numbers'>('draw_date')
  const sortOrder = ref<'asc' | 'desc'>('desc')

  // Pagination state
  const pagination = ref({
    page: 1,
    limit: 50,
    total: 0,
    totalPages: 0
  })

  // Loading states
  const loading = ref({
    draws: false,
    import: false,
    export: false,
    validation: false
  })

  // Error handling
  const error = ref<string | null>(null)
  const lastError = ref<{ timestamp: Date; message: string } | null>(null)

  // Cache management
  const cacheManager = new CacheManager('lottery-data')

  // =============================================================================
  // Computed Properties
  // =============================================================================

  const isLoading = computed(() =>
    Object.values(loading.value).some(state => state)
  )

  const hasError = computed(() => error.value !== null)
  const errorMessage = computed(() => error.value || 'Unknown error')

  // Filter and search functionality
  const filteredDraws = computed(() => {
    let filtered = [...draws.value]

    // Apply search query
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      filtered = filtered.filter(draw =>
        draw.winning_numbers.some(num =>
          num.toString().includes(query)
        ) ||
        draw.bonus_number?.toString().includes(query) ||
        draw.draw_date.toString().includes(query)
      )
    }

    // Apply date range filter
    if (activeFilters.value.draw_date_range) {
      const [start, end] = activeFilters.value.draw_date_range
      filtered = filtered.filter(draw => {
        const drawDate = new Date(draw.draw_date)
        return drawDate >= new Date(start) && drawDate <= new Date(end)
      })
    }

    // Apply number range filter
    if (activeFilters.value.number_range) {
      const [min, max] = activeFilters.value.number_range
      filtered = filtered.filter(draw =>
        draw.winning_numbers.some(num => num >= min && num <= max) ||
        (draw.bonus_number && draw.bonus_number >= min && draw.bonus_number <= max)
      )
    }

    // Apply jackpot filters
    if (activeFilters.value.jackpot_min !== null) {
      filtered = filtered.filter(draw =>
        draw.jackpot_amount && draw.jackpot_amount >= activeFilters.value.jackpot_min!
      )
    }

    if (activeFilters.value.jackpot_max !== null) {
      filtered = filtered.filter(draw =>
        draw.jackpot_amount && draw.jackpot_amount <= activeFilters.value.jackpot_max!
      )
    }

    return filtered
  })

  // Sorting functionality
  const sortedDraws = computed(() => {
    const filtered = filteredDraws.value

    return [...filtered].sort((a, b) => {
      let comparison = 0

      switch (sortBy.value) {
        case 'draw_date':
          comparison = new Date(a.draw_date).getTime() - new Date(b.draw_date).getTime()
          break
        case 'jackpot_amount':
          const aAmount = a.jackpot_amount || 0
          const bAmount = b.jackpot_amount || 0
          comparison = aAmount - bAmount
          break
        case 'numbers':
          comparison = JSON.stringify(a.winning_numbers).localeCompare(
            JSON.stringify(b.winning_numbers)
          )
          break
      }

      return sortOrder.value === 'asc' ? comparison : -comparison
    })
  })

  // Pagination
  const paginatedDraws = computed(() => {
    const { page, limit } = pagination.value
    const startIndex = (page - 1) * limit
    const endIndex = startIndex + limit

    return sortedDraws.value.slice(startIndex, endIndex)
  })

  // Statistics
  const drawStatistics = computed((): DrawStatistics => {
    if (draws.value.length === 0) {
      return {
        total_draws: 0,
        average_jackpot: 0,
        highest_jackpot: 0,
        lowest_jackpot: 0,
        draw_frequency: {}
      }
    }

    const jackpots = draws.value
      .map(draw => draw.jackpot_amount || 0)
      .filter(amount => amount > 0)

    const drawFrequency: Record<number, number> = {}

    draws.value.forEach(draw => {
      draw.winning_numbers.forEach(num => {
        drawFrequency[num] = (drawFrequency[num] || 0) + 1
      })

      if (draw.bonus_number) {
        drawFrequency[draw.bonus_number] = (drawFrequency[draw.bonus_number] || 0) + 1
      }
    })

    return {
      total_draws: draws.value.length,
      average_jackpot: jackpots.length > 0 ? jackpots.reduce((a, b) => a + b, 0) / jackpots.length : 0,
      highest_jackpot: Math.max(...jackpots),
      lowest_jackpot: Math.min(...jackpots),
      draw_frequency: drawFrequency
    }
  })

  // =============================================================================
  // Core Actions
  // =============================================================================

  // Fetch lottery draws from backend
  const fetchDraws = async (options: {
    limit?: number
    offset?: number
    force?: boolean
  } = {}) => {
    const { limit = 100, offset = 0, force = false } = options

    const cacheKey = `draws_${limit}_${offset}`

    // Try cache first
    if (!force) {
      const cached = cacheManager.get<SuperLottoDraw[]>(cacheKey)
      if (cached) {
        draws.value = cached
        totalDraws.value = cached.length
        updatePagination()
        return cached
      }
    }

    try {
      loading.value.draws = true
      error.value = null

      const fetchedDraws = await invoke<SuperLottoDraw[]>('get_lottery_history', {
        limit,
        offset
      })

      draws.value = fetchedDraws
      totalDraws.value = fetchedDraws.length

      // Cache the results
      cacheManager.set(cacheKey, fetchedDraws, 5 * 60 * 1000) // 5 minutes

      updatePagination()

      return fetchedDraws
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to fetch lottery draws'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.draws = false
    }
  }

  // Add a new lottery draw
  const addDraw = async (draw: Omit<SuperLottoDraw, 'id'>) => {
    try {
      loading.value.import = true

      const newDraw = await invoke<SuperLottoDraw>('add_lottery_draw', { draw })

      draws.value.unshift(newDraw)
      totalDraws.value += 1

      // Clear relevant cache
      cacheManager.clear('draws_')

      updatePagination()

      return newDraw
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to add lottery draw'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.import = false
    }
  }

  // Search lottery draws
  const searchDraws = async (query: LotterySearchQuery) => {
    try {
      loading.value.draws = true
      error.value = null

      const results = await invoke<SuperLottoDraw[]>('search_lottery_draws', { query })

      return results
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to search lottery draws'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.draws = false
    }
  }

  // Import lottery data from file
  const importData = async (filePath: string) => {
    try {
      loading.value.import = true
      error.value = null

      const importedDraws = await invoke<SuperLottoDraw[]>('import_lottery_data', {
        file_path: filePath
      })

      draws.value = [...importedDraws, ...draws.value]
      totalDraws.value += importedDraws.length

      // Clear cache
      cacheManager.clear()

      updatePagination()

      return importedDraws
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to import lottery data'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.import = false
    }
  }

  // Export lottery data
  const exportData = async (format: 'json' | 'csv' = 'json', filters?: LotterySearchQuery) => {
    try {
      loading.value.export = true
      error.value = null

      const exportPath = await invoke<string>('export_lottery_data', {
        format,
        filters
      })

      return exportPath
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to export lottery data'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.export = false
    }
  }

  // Validate lottery draw data
  const validateDraw = (draw: SuperLottoDraw): { valid: boolean; errors: string[] } => {
    const errors: string[] = []

    // Validate winning numbers
    if (!draw.winning_numbers || draw.winning_numbers.length !== 5) {
      errors.push('Must have exactly 5 winning numbers')
    }

    if (draw.winning_numbers.some(num => num < 1 || num > 35)) {
      errors.push('Winning numbers must be between 1 and 35')
    }

    // Check for duplicates
    const uniqueNumbers = new Set(draw.winning_numbers)
    if (uniqueNumbers.size !== draw.winning_numbers.length) {
      errors.push('Winning numbers must be unique')
    }

    // Validate bonus number
    if (draw.bonus_number && (draw.bonus_number < 1 || draw.bonus_number > 12)) {
      errors.push('Bonus number must be between 1 and 12')
    }

    // Validate draw date
    if (!draw.draw_date) {
      errors.push('Draw date is required')
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }

  // =============================================================================
  // Filter and Search Management
  // =============================================================================

  const setSearchQuery = (query: string) => {
    searchQuery.value = query
    pagination.value.page = 1 // Reset to first page
    updatePagination()
  }

  const setActiveFilters = (filters: Partial<LotterySearchQuery>) => {
    activeFilters.value = { ...activeFilters.value, ...filters }
    pagination.value.page = 1
    updatePagination()
  }

  const clearFilters = () => {
    activeFilters.value = {
      draw_date_range: null,
      number_range: null,
      jackpot_min: null,
      jackpot_max: null
    }
    searchQuery.value = ''
    pagination.value.page = 1
    updatePagination()
  }

  const setSorting = (field: typeof sortBy.value, order: typeof sortOrder.value) => {
    sortBy.value = field
    sortOrder.value = order
  }

  // =============================================================================
  // Pagination Management
  // =============================================================================

  const setPage = (page: number) => {
    pagination.value.page = page
  }

  const setLimit = (limit: number) => {
    pagination.value.limit = limit
    pagination.value.page = 1
    updatePagination()
  }

  const nextPage = () => {
    if (pagination.value.page < pagination.value.totalPages) {
      pagination.value.page += 1
    }
  }

  const prevPage = () => {
    if (pagination.value.page > 1) {
      pagination.value.page -= 1
    }
  }

  const updatePagination = () => {
    const total = filteredDraws.value.length
    const limit = pagination.value.limit

    pagination.value.total = total
    pagination.value.totalPages = Math.ceil(total / limit)

    // Ensure current page is valid
    if (pagination.value.page > pagination.value.totalPages) {
      pagination.value.page = Math.max(1, pagination.value.totalPages)
    }
  }

  // =============================================================================
  // Error Management
  // =============================================================================

  const setError = (message: string) => {
    error.value = message
    lastError.value = {
      timestamp: new Date(),
      message
    }
  }

  const clearError = () => {
    error.value = null
  }

  // =============================================================================
  // Cache Management
  // =============================================================================

  const clearCache = () => {
    cacheManager.clear()
  }

  // =============================================================================
  // Data Persistence
  // =============================================================================

  const saveState = () => {
    try {
      const state = {
        searchQuery: searchQuery.value,
        activeFilters: activeFilters.value,
        sortBy: sortBy.value,
        sortOrder: sortOrder.value,
        pagination: pagination.value
      }
      localStorage.setItem('lottery-data-store', JSON.stringify(state))
    } catch (error) {
      console.warn('Failed to save lottery data store state:', error)
    }
  }

  const loadState = () => {
    try {
      const savedState = localStorage.getItem('lottery-data-store')
      if (savedState) {
        const state = JSON.parse(savedState)

        if (state.searchQuery) searchQuery.value = state.searchQuery
        if (state.activeFilters) activeFilters.value = state.activeFilters
        if (state.sortBy) sortBy.value = state.sortBy
        if (state.sortOrder) sortOrder.value = state.sortOrder
        if (state.pagination) pagination.value = { ...pagination.value, ...state.pagination }
      }
    } catch (error) {
      console.warn('Failed to load lottery data store state:', error)
    }
  }

  const resetStore = () => {
    draws.value = []
    totalDraws.value = 0
    searchQuery.value = ''
    activeFilters.value = {
      draw_date_range: null,
      number_range: null,
      jackpot_min: null,
      jackpot_max: null
    }
    sortBy.value = 'draw_date'
    sortOrder.value = 'desc'
    pagination.value = {
      page: 1,
      limit: 50,
      total: 0,
      totalPages: 0
    }
    error.value = null
    lastError.value = null

    // Clear cache and persisted state
    clearCache()
    localStorage.removeItem('lottery-data-store')
  }

  // =============================================================================
  // Watchers for persistence
  // =============================================================================

  // Auto-save state changes
  const watchStateChanges = () => {
    // Note: In a real implementation, you'd use Vue's watchEffect or watch
    // to automatically save state changes to localStorage
  }

  return {
    // State
    draws,
    totalDraws,
    searchQuery,
    activeFilters,
    sortBy,
    sortOrder,
    pagination,
    loading,
    error,
    lastError,

    // Computed
    isLoading,
    hasError,
    errorMessage,
    filteredDraws,
    sortedDraws,
    paginatedDraws,
    drawStatistics,

    // Actions
    fetchDraws,
    addDraw,
    searchDraws,
    importData,
    exportData,
    validateDraw,

    // Filter & Search
    setSearchQuery,
    setActiveFilters,
    clearFilters,
    setSorting,

    // Pagination
    setPage,
    setLimit,
    nextPage,
    prevPage,

    // Error Management
    setError,
    clearError,

    // Cache & Persistence
    clearCache,
    saveState,
    loadState,
    resetStore
  }
})