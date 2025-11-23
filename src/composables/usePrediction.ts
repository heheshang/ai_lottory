/**
 * Composable for prediction-related logic
 * Provides reusable prediction functionality across components
 */

import { ref, computed, type Ref } from 'vue'
import { useSuperLottoStore } from '@/stores/superLotto'
import type { PredictionResult, PredictionParams } from '@/types/superLotto'

export interface UsePredictionOptions {
  autoLoad?: boolean
  defaultAlgorithm?: string
  defaultPeriod?: number
}

export function usePrediction(options: UsePredictionOptions = {}) {
  const {
    autoLoad = false,
    defaultAlgorithm = 'WEIGHTED_FREQUENCY',
    defaultPeriod = 90
  } = options

  const store = useSuperLottoStore()

  // Local state
  const selectedAlgorithm = ref(defaultAlgorithm)
  const analysisPeriod = ref(defaultPeriod)
  const isGenerating = ref(false)

  // Computed from store
  const predictions = computed(() => store.predictions)
  const loading = computed(() => store.isLoading)
  const error = computed(() => store.errorMessage)

  // Computed predictions
  const latestPrediction = computed(() => 
    predictions.value.length > 0 ? predictions.value[0] : null
  )

  const validatedPredictions = computed(() =>
    predictions.value.filter((p: PredictionResult) => p.is_validated)
  )

  const predictionsByAlgorithm = computed(() => {
    const grouped = new Map<string, PredictionResult[]>()
    predictions.value.forEach((p: PredictionResult) => {
      const key = p.algorithm_id
      if (!grouped.has(key)) {
        grouped.set(key, [])
      }
      grouped.get(key)!.push(p)
    })
    return grouped
  })

  // Statistics
  const averageAccuracy = computed(() => {
    if (validatedPredictions.value.length === 0) return 0
    const sum = validatedPredictions.value.reduce((acc: number, p: PredictionResult) => 
      acc + (p.accuracy || 0), 0
    )
    return sum / validatedPredictions.value.length
  })

  const bestPrediction = computed(() => {
    if (predictions.value.length === 0) return null
    return predictions.value.reduce((best: PredictionResult, current: PredictionResult) =>
      (current.confidence_score > best.confidence_score) ? current : best
    )
  })

  // Methods
  const generatePrediction = async (params?: Partial<PredictionParams>) => {
    isGenerating.value = true
    try {
      await store.generatePrediction({
        algorithm: selectedAlgorithm.value,
        analysis_period_days: analysisPeriod.value,
        include_reasoning: true,
        ...params
      })
    } finally {
      isGenerating.value = false
    }
  }

  const loadPredictions = async (params?: any) => {
    await store.fetchPredictions(params)
  }

  const clearError = () => {
    store.clearError()
  }

  // Auto-load if enabled
  if (autoLoad) {
    loadPredictions()
  }

  return {
    // State
    selectedAlgorithm,
    analysisPeriod,
    isGenerating,
    
    // Computed
    predictions,
    loading,
    error,
    latestPrediction,
    validatedPredictions,
    predictionsByAlgorithm,
    averageAccuracy,
    bestPrediction,
    
    // Methods
    generatePrediction,
    loadPredictions,
    clearError
  }
}
