import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  PredictionResult,
  PredictionAlgorithm,
  BatchPredictionRequest,
  BatchPredictionResult,
  HotColdAnalysis
} from '../types/prediction'
import type { SuperLottoDraw } from '../types/lottery'
import { CacheManager } from '../utils/cache'

/**
 * Predictions Store - Prediction algorithm management and results
 *
 * Responsibilities:
 * - Prediction algorithm configuration and execution
 * - Prediction result storage and analysis
 * - Algorithm performance tracking
 * - Batch prediction management
 * - Confidence score calculations
 */
export const usePredictionsStore = defineStore('predictions', () => {
  // =============================================================================
  // Reactive State
  // =============================================================================

  // Prediction results storage
  const predictions = ref<PredictionResult[]>([])
  const batchPredictions = ref<BatchPredictionResult[]>([])

  // Available algorithms
  const algorithms = ref<Record<PredictionAlgorithm, {
    name: string
    description: string
    confidence: number
    enabled: boolean
    parameters: Record<string, any>
  }>>({
    weighted_frequency: {
      name: 'Weighted Frequency',
      description: 'Based on historical frequency with time decay',
      confidence: 0.75,
      enabled: true,
      parameters: {
        time_decay_factor: 0.9,
        min_draws: 30
      }
    },
    pattern_based: {
      name: 'Pattern Based',
      description: 'Analyzes patterns in consecutive numbers, sum ranges, etc.',
      confidence: 0.80,
      enabled: true,
      parameters: {
        pattern_types: ['consecutive', 'odd_even', 'sum_ranges'],
        min_pattern_occurrences: 3
      }
    },
    hot_numbers: {
      name: 'Hot Numbers',
      description: 'Focuses on frequently drawn numbers',
      confidence: 0.70,
      enabled: true,
      parameters: {
        hot_threshold: 0.7,
        analysis_period_days: 90
      }
    },
    cold_numbers: {
      name: 'Cold Numbers',
      description: 'Identifies overdue numbers',
      confidence: 0.65,
      enabled: true,
      parameters: {
        cold_threshold: 0.3,
        analysis_period_days: 180
      }
    },
    markov_chain: {
      name: 'Markov Chain',
      description: 'Uses transition probability matrices',
      confidence: 0.85,
      enabled: true,
      parameters: {
        order: 2,
        time_decay_factor: 0.95,
        analysis_period_days: 365
      }
    },
    ensemble: {
      name: 'Ensemble',
      description: 'Combines multiple algorithms with weighted voting',
      confidence: 0.90,
      enabled: true,
      parameters: {
        algorithms: ['weighted_frequency', 'pattern_based', 'markov_chain'],
        voting_weights: {
          weighted_frequency: 0.3,
          pattern_based: 0.35,
          markov_chain: 0.35
        }
      }
    }
  })

  // Generation configuration
  const generationConfig = ref({
    sample_size: 1000,
    analysis_period_days: 180,
    max_confidence_threshold: 0.8,
    min_confidence_threshold: 0.6,
    algorithm_weights: {
      weighted_frequency: 0.25,
      pattern_based: 0.3,
      hot_numbers: 0.2,
      cold_numbers: 0.15,
      markov_chain: 0.3
    },
    diversification_factor: 0.7
  })

  // Loading states
  const loading = ref({
    prediction: false,
    batchPrediction: false,
    analysis: false,
    validation: false
  })

  // Error handling
  const error = ref<string | null>(null)
  const lastError = ref<{ timestamp: Date; message: string; algorithm?: string } | null>(null)

  // Cache management
  const cacheManager = new CacheManager('predictions')

  // Statistics tracking
  const statistics = ref({
    total_predictions: 0,
    accurate_predictions: 0,
    accuracy_rate: 0,
    algorithm_performance: {} as Record<PredictionAlgorithm, {
      total: number
      accurate: number
      accuracy: number
      average_confidence: number
    }>
  })

  // =============================================================================
  // Computed Properties
  // =============================================================================

  const isLoading = computed(() =>
    Object.values(loading.value).some(state => state)
  )

  const hasError = computed(() => error.value !== null)
  const errorMessage = computed(() => error.value || 'Unknown error')

  const hasPredictions = computed(() => predictions.value.length > 0)
  const hasBatchPredictions = computed(() => batchPredictions.value.length > 0)

  // Best prediction by confidence score
  const bestPrediction = computed(() => {
    if (predictions.value.length === 0) return null

    return predictions.value.reduce((best, current) =>
      current.confidence_score > best.confidence_score ? current : best
    )
  })

  // Recent predictions (last 10)
  const recentPredictions = computed(() =>
    predictions.value
      .slice()
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 10)
  )

  // Algorithm statistics
  const algorithmStats = computed(() => {
    const stats = {} as Record<PredictionAlgorithm, {
      usage: number
      accuracy: number
      avgConfidence: number
      enabled: boolean
    }>

    Object.keys(algorithms.value).forEach(algorithm => {
      const algo = algorithm as PredictionAlgorithm
      const algoPredictions = predictions.value.filter(p => p.algorithm === algo)

      let accuracy = 0
      let avgConfidence = 0

      if (algoPredictions.length > 0) {
        accuracy = algoPredictions.filter(p => p.is_accurate).length / algoPredictions.length
        avgConfidence = algoPredictions.reduce((sum, p) => sum + p.confidence_score, 0) / algoPredictions.length
      }

      stats[algo] = {
        usage: algoPredictions.length,
        accuracy,
        avgConfidence,
        enabled: algorithms.value[algo].enabled
      }
    })

    return stats
  })

  // Accuracy trends
  const accuracyTrends = computed(() => {
    const recent = recentPredictions.value
    if (recent.length === 0) return []

    return recent.map((prediction, index) => ({
      index: recent.length - index,
      accuracy: prediction.is_accurate ? 1 : 0,
      confidence: prediction.confidence_score,
      algorithm: prediction.algorithm
    }))
  })

  // Confidence distribution
  const confidenceDistribution = computed(() => {
    const ranges = [
      { min: 0.9, max: 1.0, label: 'Very High (90-100%)', count: 0 },
      { min: 0.8, max: 0.9, label: 'High (80-90%)', count: 0 },
      { min: 0.7, max: 0.8, label: 'Medium (70-80%)', count: 0 },
      { min: 0.6, max: 0.7, label: 'Low (60-70%)', count: 0 },
      { min: 0.0, max: 0.6, label: 'Very Low (0-60%)', count: 0 }
    ]

    predictions.value.forEach(prediction => {
      const range = ranges.find(r =>
        prediction.confidence_score >= r.min && prediction.confidence_score < r.max
      )
      if (range) range.count++
    })

    return ranges
  })

  // =============================================================================
  // Core Actions
  // =============================================================================

  // Generate single prediction
  const generatePrediction = async (
    algorithm: PredictionAlgorithm,
    customConfig?: Partial<typeof generationConfig.value>
  ): Promise<PredictionResult> => {
    const config = { ...generationConfig.value, ...customConfig }

    try {
      loading.value.prediction = true
      error.value = null

      const result = await invoke<PredictionResult>('generate_markov_chain_prediction', {
        order: algorithms.value.markov_chain.parameters.order || 2,
        period_days: config.analysis_period_days,
        time_decay_factor: algorithms.value.markov_chain.parameters.time_decay_factor || 0.95
      })

      // Update result with algorithm info
      result.algorithm = algorithm
      result.created_at = new Date().toISOString()
      result.analysis_period_days = config.analysis_period_days
      result.sample_size = config.sample_size

      // Store prediction
      predictions.value.unshift(result)
      updateStatistics()

      // Cache result
      const cacheKey = `prediction_${algorithm}_${Date.now()}`
      cacheManager.set(cacheKey, result, 30 * 60 * 1000) // 30 minutes

      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to generate prediction'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg,
        algorithm
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.prediction = false
    }
  }

  // Generate batch predictions
  const generateBatchPredictions = async (
    request: BatchPredictionRequest
  ): Promise<BatchPredictionResult> => {
    const cacheKey = `batch_${JSON.stringify(request)}`

    // Check cache first
    const cached = cacheManager.get<BatchPredictionResult>(cacheKey)
    if (cached) {
      batchPredictions.value.unshift(cached)
      return cached
    }

    try {
      loading.value.batchPrediction = true
      error.value = null

      const result = await invoke<BatchPredictionResult>('generate_batch_prediction', {
        request
      })

      // Store batch result
      batchPredictions.value.unshift(result)

      // Add individual predictions to main store
      result.predictions.forEach(prediction => {
        predictions.value.unshift(prediction)
      })

      updateStatistics()

      // Cache result
      cacheManager.set(cacheKey, result, 60 * 60 * 1000) // 1 hour

      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to generate batch predictions'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.batchPrediction = false
    }
  }

  // Analyze hot/cold numbers
  const analyzeHotCold = async (
    periodDays: number = 90,
    hotThreshold: number = 0.7,
    coldThreshold: number = 0.3
  ): Promise<HotColdAnalysis> => {
    const cacheKey = `hotcold_${periodDays}_${hotThreshold}_${coldThreshold}`

    try {
      loading.value.analysis = true
      error.value = null

      const result = await invoke<HotColdAnalysis>('generate_hot_cold_analysis', {
        period_days: periodDays,
        hot_threshold: hotThreshold,
        cold_threshold: coldThreshold
      })

      // Cache result
      cacheManager.set(cacheKey, result, 15 * 60 * 1000) // 15 minutes

      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to analyze hot/cold numbers'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.analysis = false
    }
  }

  // Validate prediction against actual draw
  const validatePrediction = async (
    predictionId: string,
    actualDraw: SuperLottoDraw
  ): Promise<{ accurate: boolean; accuracy: number }> => {
    try {
      loading.value.validation = true
      error.value = null

      const prediction = predictions.value.find(p => p.id === predictionId)
      if (!prediction) {
        throw new Error(`Prediction with ID ${predictionId} not found`)
      }

      // Calculate accuracy based on matching numbers
      const matchingFront = prediction.front_numbers.filter(num =>
        actualDraw.winning_numbers.includes(num)
      ).length

      const matchingBack = prediction.back_numbers.filter(num =>
        num === actualDraw.bonus_number
      ).length

      const accuracy = (matchingFront + matchingBack) / (prediction.front_numbers.length + prediction.back_numbers.length)
      const accurate = accuracy >= 0.6 // 60% accuracy threshold

      // Update prediction
      prediction.is_accurate = accurate
      prediction.actual_accuracy = accuracy

      updateStatistics()

      return { accurate, accuracy }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to validate prediction'
      error.value = errorMsg
      lastError.value = {
        timestamp: new Date(),
        message: errorMsg
      }
      throw new Error(errorMsg)
    } finally {
      loading.value.validation = false
    }
  }

  // =============================================================================
  // Algorithm Management
  // =============================================================================

  const updateAlgorithm = (
    algorithm: PredictionAlgorithm,
    updates: Partial<typeof algorithms.value[PredictionAlgorithm]>
  ) => {
    algorithms.value[algorithm] = { ...algorithms.value[algorithm], ...updates }
    saveState()
  }

  const enableAlgorithm = (algorithm: PredictionAlgorithm) => {
    updateAlgorithm(algorithm, { enabled: true })
  }

  const disableAlgorithm = (algorithm: PredictionAlgorithm) => {
    updateAlgorithm(algorithm, { enabled: false })
  }

  const updateAlgorithmParameters = (
    algorithm: PredictionAlgorithm,
    parameters: Record<string, any>
  ) => {
    updateAlgorithm(algorithm, { parameters: { ...algorithms.value[algorithm].parameters, ...parameters } })
  }

  // =============================================================================
  // Configuration Management
  // =============================================================================

  const updateConfig = (updates: Partial<typeof generationConfig.value>) => {
    generationConfig.value = { ...generationConfig.value, ...updates }
    saveState()
  }

  const resetConfig = () => {
    generationConfig.value = {
      sample_size: 1000,
      analysis_period_days: 180,
      max_confidence_threshold: 0.8,
      min_confidence_threshold: 0.6,
      algorithm_weights: {
        weighted_frequency: 0.25,
        pattern_based: 0.3,
        hot_numbers: 0.2,
        cold_numbers: 0.15,
        markov_chain: 0.3
      },
      diversification_factor: 0.7
    }
    saveState()
  }

  // =============================================================================
  // Prediction Management
  // =============================================================================

  const deletePrediction = (predictionId: string) => {
    const index = predictions.value.findIndex(p => p.id === predictionId)
    if (index !== -1) {
      predictions.value.splice(index, 1)
      updateStatistics()
    }
  }

  const clearPredictions = () => {
    predictions.value = []
    batchPredictions.value = []
    updateStatistics()
  }

  const getPredictionsByAlgorithm = (algorithm: PredictionAlgorithm) => {
    return predictions.value.filter(p => p.algorithm === algorithm)
  }

  const getPredictionsByConfidence = (minConfidence: number) => {
    return predictions.value.filter(p => p.confidence_score >= minConfidence)
  }

  // =============================================================================
  // Statistics and Analytics
  // =============================================================================

  const updateStatistics = () => {
    const total = predictions.value.length
    const accurate = predictions.value.filter(p => p.is_accurate).length

    statistics.value.total_predictions = total
    statistics.value.accurate_predictions = accurate
    statistics.value.accuracy_rate = total > 0 ? accurate / total : 0

    // Update algorithm-specific statistics
    Object.keys(algorithms.value).forEach(algorithm => {
      const algo = algorithm as PredictionAlgorithm
      const algoPredictions = predictions.value.filter(p => p.algorithm === algo)

      const total = algoPredictions.length
      const accurate = algoPredictions.filter(p => p.is_accurate).length
      const avgConfidence = total > 0
        ? algoPredictions.reduce((sum, p) => sum + p.confidence_score, 0) / total
        : 0

      statistics.value.algorithm_performance[algo] = {
        total,
        accurate,
        accuracy: total > 0 ? accurate / total : 0,
        average_confidence: avgConfidence
      }
    })
  }

  // =============================================================================
  // Error Management
  // =============================================================================

  const setError = (message: string, algorithm?: string) => {
    error.value = message
    lastError.value = {
      timestamp: new Date(),
      message,
      algorithm
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
        algorithms: algorithms.value,
        generationConfig: generationConfig.value,
        statistics: statistics.value
      }
      localStorage.setItem('predictions-store', JSON.stringify(state))
    } catch (error) {
      console.warn('Failed to save predictions store state:', error)
    }
  }

  const loadState = () => {
    try {
      const savedState = localStorage.getItem('predictions-store')
      if (savedState) {
        const state = JSON.parse(savedState)

        if (state.algorithms) algorithms.value = { ...algorithms.value, ...state.algorithms }
        if (state.generationConfig) generationConfig.value = { ...generationConfig.value, ...state.generationConfig }
        if (state.statistics) statistics.value = { ...statistics.value, ...state.statistics }
      }
    } catch (error) {
      console.warn('Failed to load predictions store state:', error)
    }
  }

  const resetStore = () => {
    predictions.value = []
    batchPredictions.value = []
    loading.value = {
      prediction: false,
      batchPrediction: false,
      analysis: false,
      validation: false
    }
    error.value = null
    lastError.value = null
    statistics.value = {
      total_predictions: 0,
      accurate_predictions: 0,
      accuracy_rate: 0,
      algorithm_performance: {}
    }

    // Clear cache and persisted state
    clearCache()
    localStorage.removeItem('predictions-store')
  }

  return {
    // State
    predictions,
    batchPredictions,
    algorithms,
    generationConfig,
    loading,
    error,
    lastError,
    statistics,

    // Computed
    isLoading,
    hasError,
    errorMessage,
    hasPredictions,
    hasBatchPredictions,
    bestPrediction,
    recentPredictions,
    algorithmStats,
    accuracyTrends,
    confidenceDistribution,

    // Actions
    generatePrediction,
    generateBatchPredictions,
    analyzeHotCold,
    validatePrediction,

    // Algorithm Management
    updateAlgorithm,
    enableAlgorithm,
    disableAlgorithm,
    updateAlgorithmParameters,

    // Configuration
    updateConfig,
    resetConfig,

    // Prediction Management
    deletePrediction,
    clearPredictions,
    getPredictionsByAlgorithm,
    getPredictionsByConfidence,

    // Statistics
    updateStatistics,

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