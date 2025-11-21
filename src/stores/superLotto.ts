import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

// TypeScript types for Super Lotto data
export interface SuperLottoDraw {
  id: number
  draw_date: string
  draw_number?: string
  front_zone: number[]
  back_zone: number[]
  jackpot_amount?: number
  winners_count?: number
  sum_front: number
  odd_count_front: number
  even_count_front: number
  has_consecutive_front: boolean
  created_at: string
}

export interface NumberFrequency {
  number: number
  zone: 'FRONT' | 'BACK'
  frequency: number
  last_seen?: string
  hot_score: number
  cold_score: number
  average_gap: number
  current_gap: number
  period_days: number
  updated_at: string
}

export interface PatternAnalysis {
  id: number
  pattern_type: string
  analysis_data: any
  confidence_score: number
  sample_size: number
  period_days: number
  created_at: string
}

export interface PredictionResult {
  id: number
  algorithm: string
  front_numbers: number[]
  back_numbers: number[]
  confidence_score: number
  reasoning: any
  analysis_period_days: number
  sample_size: number
  created_at: string
  is_validated: boolean
}

export interface HistoryParams {
  limit?: number
  offset?: number
  start_date?: string
  end_date?: string
  draw_number?: string
}

export interface AnalysisParams {
  days: number
  zone?: 'FRONT' | 'BACK' | 'BOTH'
  limit?: number
  min_threshold?: number
}

export interface PredictionParams {
  algorithm: string
  analysis_period_days?: number
  custom_parameters?: any
  include_reasoning?: boolean
}

export const useSuperLottoStore = defineStore('superLotto', () => {
  // State
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Historical data
  const draws = ref<SuperLottoDraw[]>([])
  const totalDraws = ref(0)

  // Analysis results
  const hotNumbers = ref<NumberFrequency[]>([])
  const coldNumbers = ref<NumberFrequency[]>([])
  const patterns = ref<PatternAnalysis[]>([])
  const predictions = ref<PredictionResult[]>([])

  // Pagination
  const currentPage = ref(1)
  const pageSize = ref(100)
  const totalPages = ref(0)

  // Computed properties
  const filteredDraws = computed(() => {
    return draws.value // Add filtering logic here when needed
  })

  const isDataLoaded = computed(() => {
    return draws.value.length > 0
  })

  const hasAnalysisData = computed(() => {
    return hotNumbers.value.length > 0 || coldNumbers.value.length > 0
  })

  // Actions
  async function fetchDraws(params: HistoryParams = {}) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('get_super_lotto_draws', params)
      draws.value = result.draws || []
      totalDraws.value = result.total || 0
      currentPage.value = Math.floor((params.offset || 0) / (params.limit || 100)) + 1
      totalPages.value = Math.ceil(totalDraws.value / (params.limit || 100))
    } catch (err) {
      error.value = `Failed to fetch draws: ${err}`
      console.error('Error fetching draws:', err)
    } finally {
      loading.value = false
    }
  }

  async function importDraws(drawData: any[]) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('import_super_lotto_draws', { draws: drawData })
      return result
    } catch (err) {
      error.value = `Failed to import draws: ${err}`
      console.error('Error importing draws:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function analyzeHotNumbers(params: AnalysisParams) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('analyze_hot_numbers', params)
      hotNumbers.value = result.numbers || []
      return result
    } catch (err) {
      error.value = `Failed to analyze hot numbers: ${err}`
      console.error('Error analyzing hot numbers:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function analyzeColdNumbers(params: AnalysisParams) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('analyze_cold_numbers', params)
      coldNumbers.value = result.numbers || []
      return result
    } catch (err) {
      error.value = `Failed to analyze cold numbers: ${err}`
      console.error('Error analyzing cold numbers:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function getPatternAnalysis(params: any) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('get_pattern_analysis', params)
      patterns.value = result.patterns || []
      return result
    } catch (err) {
      error.value = `Failed to get pattern analysis: ${err}`
      console.error('Error getting pattern analysis:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function generatePrediction(params: PredictionParams) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('generate_prediction', params)
      predictions.value.unshift(result)
      return result
    } catch (err) {
      error.value = `Failed to generate prediction: ${err}`
      console.error('Error generating prediction:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function fetchPredictions(params: any = {}) {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<any>('get_predictions', params)
      predictions.value = result.predictions || []
      return result
    } catch (err) {
      error.value = `Failed to fetch predictions: ${err}`
      console.error('Error fetching predictions:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  function clearError() {
    error.value = null
  }

  function resetStore() {
    draws.value = []
    hotNumbers.value = []
    coldNumbers.value = []
    patterns.value = []
    predictions.value = []
    error.value = null
    loading.value = false
    currentPage.value = 1
    totalPages.value = 0
  }

  return {
    // State
    loading,
    error,
    draws,
    totalDraws,
    hotNumbers,
    coldNumbers,
    patterns,
    predictions,
    currentPage,
    pageSize,
    totalPages,

    // Computed
    filteredDraws,
    isDataLoaded,
    hasAnalysisData,

    // Actions
    fetchDraws,
    importDraws,
    analyzeHotNumbers,
    analyzeColdNumbers,
    getPatternAnalysis,
    generatePrediction,
    fetchPredictions,
    clearError,
    resetStore
  }
})