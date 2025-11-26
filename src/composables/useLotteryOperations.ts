import { computed, ref, watch } from 'vue'
import { useAuthStore } from '../stores/auth'
import { useLotteryDataStore } from '../stores/lottery-data'
import { usePredictionsStore } from '../stores/predictions'
import { useUIStore } from '../stores/ui'
import type { SuperLottoDraw, PredictionResult } from '../types'

/**
 * Advanced Lottery Operations Composable
 *
 * Provides high-level, cross-store operations for lottery functionality.
 * Handles complex workflows that involve multiple stores and coordinated actions.
 *
 * Features:
 * - Intelligent workflow orchestration
 * - Error handling and retry logic
 * - Performance monitoring
 * - Cache coordination
 * - Batch operations
 * - State synchronization
 */
export function useLotteryOperations() {
  const authStore = useAuthStore()
  const dataStore = useLotteryDataStore()
  const predictionsStore = usePredictionsStore()
  const uiStore = useUIStore()

  // =============================================================================
  // Reactive State
  // =============================================================================

  const operationInProgress = ref(false)
  const currentOperation = ref<string | null>(null)
  const operationQueue = ref<Array<{
    id: string
    type: string
    description: string
    execute: () => Promise<any>
    priority: 'low' | 'medium' | 'high' | 'critical'
    createdAt: Date
    retryCount?: number
    maxRetries?: number
  }>>([])

  const operationHistory = ref<Array<{
    id: string
    type: string
    description: string
    status: 'success' | 'error' | 'cancelled'
    startTime: Date
    endTime?: Date
    duration?: number
    error?: string
    result?: any
  }>>([])

  const performanceMetrics = ref({
    totalOperations: 0,
    successfulOperations: 0,
    failedOperations: 0,
    averageExecutionTime: 0,
    cacheHitRate: 0,
    errorRate: 0,
    operationsByType: {} as Record<string, number>
  })

  // =============================================================================
  // Computed Properties
  // =============================================================================

  const isUserAuthenticated = computed(() => authStore.isAuthenticated)
  const hasDataLoaded = computed(() => dataStore.draws.length > 0)
  const hasPredictions = computed(() => predictionsStore.hasPredictions)
  const systemReady = computed(() => isUserAuthenticated.value && hasDataLoaded.value)

  const queueLength = computed(() => operationQueue.value.length)
  const hasActiveOperations = computed(() => operationInProgress.value || queueLength.value > 0)

  const operationSuccessRate = computed(() => {
    const total = performanceMetrics.value.totalOperations
    return total > 0 ? performanceMetrics.value.successfulOperations / total : 0
  })

  const recentOperations = computed(() =>
    operationHistory.value
      .sort((a, b) => b.startTime.getTime() - a.startTime.getTime())
      .slice(0, 10)
  )

  // =============================================================================
  // High-Level Workflows
  // =============================================================================

  /**
   * Complete lottery analysis workflow
   * - Fetches latest data
   * - Analyzes patterns
   * - Generates predictions using multiple algorithms
   * - Updates all relevant stores
   */
  const performCompleteAnalysis = async (
    options: {
      forceRefresh?: boolean
      algorithms?: string[]
      analysisPeriod?: number
      batchSize?: number
      enableCaching?: boolean
    } = {}
  ) => {
    const operation = createOperation(
      'complete_analysis',
      'Perform complete lottery analysis',
      async () => {
        const startTime = Date.now()

        try {
          uiStore.setGlobalLoading(true)
          uiStore.showLoading('Analysis', 'Starting comprehensive lottery analysis...')

          // Step 1: Fetch latest data
          const dataFetch = await dataStore.fetchDraws({
            limit: options.batchSize || 100,
            force: options.forceRefresh
          })

          // Step 2: Analyze hot/cold numbers
          const hotColdAnalysis = await predictionsStore.analyzeHotCold(
            options.analysisPeriod || 90,
            0.7,
            0.3
          )

          // Step 3: Generate batch predictions
          const batchRequest = {
            algorithms: options.algorithms || [
              'weighted_frequency',
              'pattern_based',
              'markov_chain'
            ],
            analysis_period_days: options.analysisPeriod || 90,
            sample_size: options.batchSize || 1000,
            include_validation: true
          }

          const predictions = await predictionsStore.generateBatchPredictions(batchRequest)

          // Step 4: Update UI with results
          uiStore.showSuccess(
            'Analysis Complete',
            `Generated ${predictions.total_predictions} predictions with ${predictions.successful_predictions} successful validations`
          )

          return {
            dataFetched: dataFetch.length,
            hotColdAnalysis,
            predictions,
            duration: Date.now() - startTime
          }
        } finally {
          uiStore.setGlobalLoading(false)
        }
      },
      'high'
    )

    return await executeOperation(operation)
  }

  /**
   * Smart prediction workflow
   * - Uses best performing algorithms
   * - Incorporates recent performance data
   * - Optimizes for accuracy and confidence
   */
  const generateSmartPrediction = async (
    customConfig?: {
      confidenceThreshold?: number
      maxPredictions?: number
      useEnsemble?: boolean
      considerHistorical?: boolean
    }
  ) => {
    const operation = createOperation(
      'smart_prediction',
      'Generate intelligent prediction',
      async () => {
        // Get best performing algorithms
        const algorithmStats = predictionsStore.algorithmStats
        const bestAlgorithms = Object.entries(algorithmStats)
          .filter(([_, stats]) => stats.enabled && stats.accuracy > 0.6)
          .sort(([, a], [, b]) => b.avgConfidence - a.avgConfidence)
          .slice(0, 3)
          .map(([name]) => name as any)

        if (bestAlgorithms.length === 0) {
          throw new Error('No suitable algorithms available for prediction')
        }

        // Generate predictions using best algorithms
        const predictions = []
        const threshold = customConfig?.confidenceThreshold || 0.7

        for (const algorithm of bestAlgorithms) {
          const prediction = await predictionsStore.generatePrediction(
            algorithm,
            customConfig
          )

          if (prediction.confidence_score >= threshold) {
            predictions.push(prediction)
          }
        }

        // Sort by confidence score
        predictions.sort((a, b) => b.confidence_score - a.confidence_score)

        return {
          predictions: predictions.slice(0, customConfig?.maxPredictions || 5),
          algorithms: bestAlgorithms,
          threshold,
          bestPrediction: predictions[0] || null
        }
      },
      'high'
    )

    return await executeOperation(operation)
  }

  /**
   * Data synchronization workflow
   * - Validates existing data integrity
   * - Fetches missing or outdated data
   * - Cleans up corrupted entries
   * - Optimizes storage
   */
  const synchronizeData = async () => {
    const operation = createOperation(
      'data_sync',
      'Synchronize lottery data',
      async () => {
        const startTime = Date.now()

        try {
          uiStore.setLoadingOperation('data_sync', true)
          uiStore.showLoading('Data Sync', 'Validating and synchronizing lottery data...')

          // Step 1: Validate existing data
          const validationResults = []
          for (const draw of dataStore.draws) {
            const validation = dataStore.validateDraw(draw)
            if (!validation.valid) {
              validationResults.push({
                draw,
                errors: validation.errors
              })
            }
          }

          // Step 2: Fetch missing data if needed
          if (validationResults.length > dataStore.draws.length * 0.1) {
            // More than 10% invalid data, fetch fresh data
            await dataStore.fetchDraws({ force: true })
          }

          // Step 3: Clear corrupted entries
          for (const { draw } of validationResults) {
            const index = dataStore.draws.findIndex(d => d.id === draw.id)
            if (index !== -1) {
              dataStore.draws.splice(index, 1)
            }
          }

          // Step 4: Optimize caches
          dataStore.clearCache()
          predictionsStore.clearCache()

          return {
            totalDraws: dataStore.draws.length,
            invalidEntries: validationResults.length,
            duration: Date.now() - startTime,
            dataIntegrity: validationResults.length === 0 ? 'excellent' : 'good'
          }
        } finally {
          uiStore.setLoadingOperation('data_sync', false)
        }
      },
      'medium'
    )

    return await executeOperation(operation)
  }

  /**
   * Batch import workflow
   * - Validates import data
   * - Processes in batches to avoid blocking
   * - Provides progress feedback
   * - Handles errors gracefully
   */
  const batchImportData = async (
    filePath: string,
    options: {
      batchSize?: number
      validateOnly?: boolean
      skipDuplicates?: boolean
      progressCallback?: (progress: number, total: number) => void
    } = {}
  ) => {
    const operation = createOperation(
      'batch_import',
      `Import lottery data from ${filePath}`,
      async () => {
        const startTime = Date.now()
        const batchSize = options.batchSize || 50

        uiStore.showLoading('Import', 'Starting batch import operation...')

        try {
          // Step 1: Read and validate file structure
          const importedData = await dataStore.importData(filePath)

          if (!Array.isArray(importedData) || importedData.length === 0) {
            throw new Error('Invalid data format or empty file')
          }

          // Step 2: Process in batches
          const total = importedData.length
          let processed = 0
          let imported = 0
          const errors = []

          for (let i = 0; i < total; i += batchSize) {
            const batch = importedData.slice(i, i + batchSize)

            uiStore.showInfo(
              'Import Progress',
              `Processing batch ${Math.floor(i / batchSize) + 1} of ${Math.ceil(total / batchSize)}`
            )

            // Process batch
            for (const item of batch) {
              processed++

              try {
                // Validate draw
                const validation = dataStore.validateDraw(item)
                if (!validation.valid) {
                  errors.push({ item, errors: validation.errors })
                  continue
                }

                // Check for duplicates
                if (options.skipDuplicates) {
                  const exists = dataStore.draws.some(d =>
                    d.draw_date === item.draw_date &&
                    JSON.stringify(d.winning_numbers) === JSON.stringify(item.winning_numbers)
                  )
                  if (exists) continue
                }

                // Import item
                if (!options.validateOnly) {
                  await dataStore.addDraw(item)
                  imported++
                }

                // Update progress
                options.progressCallback?.(processed, total)
                uiStore.updateProgressIndicator('import', processed / total * 100)
              } catch (error) {
                errors.push({ item, errors: [error instanceof Error ? error.message : 'Unknown error'] })
              }
            }

            // Small delay to prevent UI blocking
            await new Promise(resolve => setTimeout(resolve, 10))
          }

          uiStore.removeProgressIndicator('import')

          return {
            total,
            processed,
            imported,
            errors: errors.length,
            validationErrors: errors,
            duration: Date.now() - startTime,
            successRate: processed > 0 ? imported / processed : 0
          }
        } catch (error) {
          uiStore.removeProgressIndicator('import')
          throw error
        }
      },
      'medium'
    )

    return await executeOperation(operation)
  }

  // =============================================================================
  // Operation Management
  // =============================================================================

  const createOperation = (
    type: string,
    description: string,
    execute: () => Promise<any>,
    priority: 'low' | 'medium' | 'high' | 'critical' = 'medium',
    maxRetries: number = 3
  ) => {
    return {
      id: `operation_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type,
      description,
      execute,
      priority,
      createdAt: new Date(),
      retryCount: 0,
      maxRetries
    }
  }

  const executeOperation = async (operation: ReturnType<typeof createOperation>) => {
    if (operationInProgress.value && operation.priority !== 'critical') {
      // Add to queue
      operationQueue.value.push(operation)
      operationQueue.value.sort((a, b) => {
        const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 }
        return priorityOrder[a.priority] - priorityOrder[b.priority]
      })
      return null
    }

    const startTime = Date.now()
    operationInProgress.value = true
    currentOperation.value = operation.description

    let historyEntry = {
      id: operation.id,
      type: operation.type,
      description: operation.description,
      status: 'success' as const,
      startTime: new Date()
    }

    operationHistory.value.unshift(historyEntry)

    try {
      // Update operation type stats
      performanceMetrics.value.operationsByType[operation.type] =
        (performanceMetrics.value.operationsByType[operation.type] || 0) + 1

      // Execute operation
      const result = await operation.execute()

      // Update history
      historyEntry.status = 'success'
      historyEntry.endTime = new Date()
      historyEntry.duration = Date.now() - startTime
      historyEntry.result = result

      // Update performance metrics
      performanceMetrics.value.totalOperations++
      performanceMetrics.value.successfulOperations++
      performanceMetrics.value.averageExecutionTime =
        (performanceMetrics.value.averageExecutionTime * (performanceMetrics.value.totalOperations - 1) +
         (Date.now() - startTime)) / performanceMetrics.value.totalOperations

      uiStore.showSuccess('Operation Complete', operation.description)

      return result
    } catch (error) {
      // Update history
      historyEntry.status = 'error'
      historyEntry.endTime = new Date()
      historyEntry.duration = Date.now() - startTime
      historyEntry.error = error instanceof Error ? error.message : 'Unknown error'

      // Update performance metrics
      performanceMetrics.value.totalOperations++
      performanceMetrics.value.failedOperations++
      performanceMetrics.value.errorRate =
        performanceMetrics.value.failedOperations / performanceMetrics.value.totalOperations

      // Retry logic
      if (operation.retryCount < operation.maxRetries) {
        operation.retryCount++
        uiStore.showWarning(
          'Operation Retry',
          `Retrying ${operation.description} (attempt ${operation.retryCount})`
        )
        return await executeOperation(operation)
      }

      uiStore.showError('Operation Failed', `${operation.description}: ${historyEntry.error}`)
      throw error
    } finally {
      operationInProgress.value = false
      currentOperation.value = null

      // Process next operation in queue
      if (operationQueue.value.length > 0) {
        const nextOperation = operationQueue.value.shift()!
        setTimeout(() => executeOperation(nextOperation), 100)
      }
    }
  }

  const cancelOperation = (operationId?: string) => {
    if (operationId) {
      const index = operationQueue.value.findIndex(op => op.id === operationId)
      if (index !== -1) {
        operationQueue.value.splice(index, 1)
        return true
      }
    } else {
      // Cancel current operation
      operationInProgress.value = false
      currentOperation.value = null
      operationQueue.value = []
      return true
    }
    return false
  }

  const clearOperationHistory = () => {
    operationHistory.value = []
    performanceMetrics.value = {
      totalOperations: 0,
      successfulOperations: 0,
      failedOperations: 0,
      averageExecutionTime: 0,
      cacheHitRate: 0,
      errorRate: 0,
      operationsByType: {}
    }
  }

  // =============================================================================
  // Smart Workflows
  // =============================================================================

  /**
   * Intelligent refresh workflow
   * - Checks data freshness
   * - Refreshes only if needed
   * - Optimizes cache usage
   */
  const intelligentRefresh = async () => {
    const lastDataFetch = dataStore.draws.length > 0 ?
      Math.max(...dataStore.draws.map(d => new Date(d.draw_date).getTime())) : 0
    const now = Date.now()
    const daysSinceLastFetch = (now - lastDataFetch) / (1000 * 60 * 60 * 24)

    // Refresh if data is older than 7 days
    if (daysSinceLastFetch > 7) {
      await synchronizeData()
      return true
    }

    // Check if we have recent predictions
    const recentPredictions = predictionsStore.recentPredictions
    const hasRecentPredictions = recentPredictions.length > 0 &&
      new Date(recentPredictions[0].created_at).getTime() > (now - 24 * 60 * 60 * 1000)

    if (!hasRecentPredictions) {
      await generateSmartPrediction({ maxPredictions: 3 })
      return true
    }

    return false
  }

  /**
   * Performance optimization workflow
   * - Analyzes cache performance
   * - Optimizes data structures
   * - Cleans up unused resources
   */
  const optimizePerformance = async () => {
    const operation = createOperation(
      'performance_optimization',
      'Optimize application performance',
      async () => {
        const startTime = Date.now()

        // Clear old caches
        dataStore.clearCache()
        predictionsStore.clearCache()

        // Optimize UI store
        uiStore.optimize()

        // Compress operation history (keep last 50)
        if (operationHistory.value.length > 50) {
          operationHistory.value = operationHistory.value.slice(0, 50)
        }

        return {
          duration: Date.now() - startTime,
          optimizations: [
            'Cache cleared',
            'UI optimized',
            'History compressed'
          ]
        }
      },
      'low'
    )

    return await executeOperation(operation)
  }

  // =============================================================================
  // Reactive Effects
  // =============================================================================

  // Watch for data changes and trigger smart refresh
  watch(
    [() => dataStore.draws.length, () => predictionsStore.predictions.length],
    ([dataCount, predCount], [oldDataCount, oldPredCount]) => {
      if (dataCount !== oldDataCount || predCount !== oldPredCount) {
        // Update cache hit rates
        const cacheStats = dataStore.getCache?.() || predictionsStore.getStats()
        performanceMetrics.value.cacheHitRate = cacheStats?.hitRate || 0
      }
    },
    { deep: true }
  )

  // Auto-optimize every 10 minutes
  setInterval(() => {
    if (!operationInProgress.value) {
      optimizePerformance()
    }
  }, 10 * 60 * 1000)

  return {
    // State
    operationInProgress,
    currentOperation,
    operationQueue,
    operationHistory,
    performanceMetrics,

    // Computed
    isUserAuthenticated,
    hasDataLoaded,
    hasPredictions,
    systemReady,
    queueLength,
    hasActiveOperations,
    operationSuccessRate,
    recentOperations,

    // High-Level Workflows
    performCompleteAnalysis,
    generateSmartPrediction,
    synchronizeData,
    batchImportData,

    // Operation Management
    createOperation,
    executeOperation,
    cancelOperation,
    clearOperationHistory,

    // Smart Workflows
    intelligentRefresh,
    optimizePerformance
  }
}