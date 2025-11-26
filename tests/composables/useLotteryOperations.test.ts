/**
 * Lottery Operations Composable Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useLotteryOperations } from '../../src/composables/useLotteryOperations'
import { useAuthStore } from '../../src/stores/auth'
import { useLotteryDataStore } from '../../src/stores/lottery-data'
import { usePredictionsStore } from '../../src/stores/predictions'
import { useUIStore } from '../../src/stores/ui'
import { createPinia, setActivePinia } from 'pinia'
import { createMockUser, generateMockDraws, generateMockPredictions } from '../setup'

describe('useLotteryOperations Composable', () => {
  let lotteryOps: ReturnType<typeof useLotteryOperations>
  let authStore: ReturnType<typeof useAuthStore>
  let dataStore: ReturnType<typeof useLotteryDataStore>
  let predictionsStore: ReturnType<typeof usePredictionsStore>
  let uiStore: ReturnType<typeof useUIStore>

  beforeEach(() => {
    const pinia = createPinia()
    setActivePinia(pinia)

    authStore = useAuthStore()
    dataStore = useLotteryDataStore()
    predictionsStore = usePredictionsStore()
    uiStore = useUIStore()
    lotteryOps = useLotteryOperations()

    // Setup authenticated user
    authStore.$patch({
      user: createMockUser(),
      isAuthenticated: true
    })

    // Mock data
    dataStore.$patch({
      draws: generateMockDraws(10)
    })

    predictionsStore.$patch({
      predictions: generateMockPredictions(5)
    })
  })

  describe('Computed Properties', () => {
    it('should compute user authentication status', () => {
      expect(lotteryOps.isUserAuthenticated.value).toBe(true)

      authStore.$patch({
        isAuthenticated: false
      })

      expect(lotteryOps.isUserAuthenticated.value).toBe(false)
    })

    it('should compute data loaded status', () => {
      expect(lotteryOps.hasDataLoaded.value).toBe(true)

      dataStore.$patch({
        draws: []
      })

      expect(lotteryOps.hasDataLoaded.value).toBe(false)
    })

    it('should compute predictions available status', () => {
      expect(lotteryOps.hasPredictions.value).toBe(true)

      predictionsStore.$patch({
        predictions: []
      })

      expect(lotteryOps.hasPredictions.value).toBe(false)
    })

    it('should compute system ready status', () => {
      expect(lotteryOps.systemReady.value).toBe(true)

      authStore.$patch({
        isAuthenticated: false
      })

      expect(lotteryOps.systemReady.value).toBe(false)
    })

    it('should compute operation success rate', () => {
      // Mock successful operations
      lotteryOps.operationHistory.value = [
        {
          id: '1',
          type: 'test',
          description: 'Test operation',
          status: 'success',
          startTime: new Date(Date.now() - 1000),
          endTime: new Date(),
          duration: 1000
        },
        {
          id: '2',
          type: 'test',
          description: 'Test operation',
          status: 'success',
          startTime: new Date(Date.now() - 1000),
          endTime: new Date(),
          duration: 1000
        },
        {
          id: '3',
          type: 'test',
          description: 'Test operation',
          status: 'error',
          startTime: new Date(Date.now() - 1000),
          endTime: new Date(),
          duration: 1000,
          error: 'Test error'
        }
      ]

      expect(lotteryOps.operationSuccessRate.value).toBe(2/3)
    })

    it('should compute recent operations', () => {
      const now = new Date()
      const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000)
      const twoHoursAgo = new Date(now.getTime() - 2 * 60 * 60 * 1000)

      lotteryOps.operationHistory.value = [
        {
          id: '1',
          type: 'test',
          description: 'Most recent',
          status: 'success',
          startTime: now,
          endTime: now,
          duration: 100
        },
        {
          id: '2',
          type: 'test',
          description: 'One hour ago',
          status: 'success',
          startTime: oneHourAgo,
          endTime: oneHourAgo,
          duration: 200
        },
        {
          id: '3',
          type: 'test',
          description: 'Two hours ago',
          status: 'success',
          startTime: twoHoursAgo,
          endTime: twoHoursAgo,
          duration: 300
        }
      ]

      const recent = lotteryOps.recentOperations.value
      expect(recent).toHaveLength(3)
      expect(recent[0].description).toBe('Most recent')
      expect(recent[1].description).toBe('One hour ago')
    })
  })

  describe('Operation Management', () => {
    it('should create operation correctly', () => {
      const operation = lotteryOps.createOperation(
        'test_operation',
        'Test operation description',
        vi.fn().mockResolvedValue('test_result'),
        'medium'
      )

      expect(operation.id).toMatch(/^operation_\d+_[a-z0-9]+$/)
      expect(operation.type).toBe('test_operation')
      expect(operation.description).toBe('Test operation description')
      expect(operation.priority).toBe('medium')
      expect(operation.retried).toBe(0)
      expect(operation.maxRetries).toBe(3)
      expect(operation.createdAt).toBeInstanceOf(Date)
    })

    it('should execute operation successfully', async () => {
      const mockFn = vi.fn().mockResolvedValue('test_result')
      const operation = lotteryOps.createOperation(
        'test_operation',
        'Test operation',
        mockFn
      )

      const result = await lotteryOps.executeOperation(operation)

      expect(result).toBe('test_result')
      expect(mockFn).toHaveBeenCalledTimes(1)
      expect(lotteryOps.operationHistory.value).toHaveLength(1)
      expect(lotteryOps.operationHistory.value[0].status).toBe('success')
      expect(lotteryOps.performanceMetrics.value.totalOperations).toBe(1)
      expect(lotteryOps.performanceMetrics.value.successfulOperations).toBe(1)
    })

    it('should handle operation failure with retry', async () => {
      const mockFn = vi.fn()
        .mockRejectedValueOnce(new Error('Test error'))
        .mockResolvedValueOnce('retry_result')

      const operation = lotteryOps.createOperation(
        'test_operation',
        'Test operation',
        mockFn,
        'medium'
      )

      const result = await lotteryOps.executeOperation(operation)

      expect(result).toBe('retry_result')
      expect(mockFn).toHaveBeenCalledTimes(2)
      expect(lotteryOps.operationHistory.value[0].status).toBe('success')
    })

    it('should handle operation failure after max retries', async () => {
      const mockFn = vi.fn().mockRejectedValue(new Error('Persistent error'))
      const operation = lotteryOps.createOperation(
        'test_operation',
        'Test operation',
        mockFn,
        'medium'
      )

      await expect(lotteryOps.executeOperation(operation)).rejects.toThrow('Persistent error')

      expect(mockFn).toHaveBeenCalledTimes(4) // 1 initial + 3 retries
      expect(lotteryOps.operationHistory.value[0].status).toBe('error')
    })

    it('should cancel operation', () => {
      const operation = lotteryOps.createOperation(
        'test_operation',
        'Test operation',
        vi.fn()
      )

      lotteryOps.operationQueue.value.push(operation)
      expect(lotteryOps.queueLength.value).toBe(1)

      const cancelled = lotteryOps.cancelOperation(operation.id)

      expect(cancelled).toBe(true)
      expect(lotteryOps.queueLength.value).toBe(0)
    })

    it('should clear operation history', () => {
      lotteryOps.operationHistory.value = [
        { id: '1', type: 'test', description: 'Test', status: 'success', startTime: new Date(), endTime: new Date(), duration: 100 },
        { id: '2', type: 'test', description: 'Test', status: 'success', startTime: new Date(), endTime: new Date(), duration: 200 }
      ]

      lotteryOps.clearOperationHistory()

      expect(lotteryOps.operationHistory.value).toHaveLength(0)
      expect(lotteryOps.performanceMetrics.value.totalOperations).toBe(0)
    })
  })

  describe('High-Level Workflows', () => {
    it('should perform complete analysis', async () => {
      // Mock the underlying operations
      const mockDataFetch = vi.fn().mockResolvedValue(generateMockDraws(50))
      const mockHotColdAnalysis = vi.fn().mockResolvedValue({
        hot_numbers: [1, 2, 3],
        cold_numbers: [35, 34, 33]
      })
      const mockBatchPredictions = vi.fn().mockResolvedValue({
        id: 'batch_1',
        predictions: generateMockPredictions(5),
        total_predictions: 5,
        successful_predictions: 4,
        processing_time_ms: 2000,
        created_at: new Date().toISOString()
      })

      vi.spyOn(dataStore, 'fetchDraws').mockImplementation(mockDataFetch)
      vi.spyOn(predictionsStore, 'analyzeHotCold').mockImplementation(mockHotColdAnalysis)
      vi.spyOn(predictionsStore, 'generateBatchPredictions').mockImplementation(mockBatchPredictions)

      const result = await lotteryOps.performCompleteAnalysis({
        analysisPeriod: 90,
        batchSize: 100
      })

      expect(result).toBeDefined()
      expect(mockDataFetch).toHaveBeenCalledWith({ limit: 100, force: false })
      expect(mockHotColdAnalysis).toHaveBeenCalledWith(90, 0.7, 0.3)
      expect(mockBatchPredictions).toHaveBeenCalledWith({
        algorithms: ['weighted_frequency', 'pattern_based', 'markov_chain'],
        analysis_period_days: 90,
        sample_size: 100,
        include_validation: true
      })
    })

    it('should generate smart prediction', async () => {
      const mockPrediction = {
        id: 'prediction_1',
        algorithm: 'markov_chain',
        front_numbers: [1, 2, 3, 4, 5],
        back_numbers: [1],
        confidence_score: 0.85,
        reasoning: 'Smart prediction reasoning',
        created_at: new Date().toISOString()
      }

      const mockGeneratePrediction = vi.fn().mockResolvedValue(mockPrediction)
      vi.spyOn(predictionsStore, 'generatePrediction').mockImplementation(mockGeneratePrediction)

      const result = await lotteryOps.generateSmartPrediction({
        confidenceThreshold: 0.8,
        maxPredictions: 3,
        useEnsemble: true
      })

      expect(result).toBeDefined()
      expect(result.bestPrediction).toEqual(mockPrediction)
      expect(result.predictions).toHaveLength(1)
      expect(result.threshold).toBe(0.8)
    })

    it('should synchronize data', async () => {
      const mockValidateDraw = vi.fn().mockReturnValue({
        valid: true,
        errors: []
      })
      const mockFetchDraws = vi.fn().mockResolvedValue(generateMockDraws(100))

      vi.spyOn(dataStore, 'validateDraw').mockImplementation(mockValidateDraw)
      vi.spyOn(dataStore, 'fetchDraws').mockImplementation(mockFetchDraws)

      const result = await lotteryOps.synchronizeData()

      expect(result).toBeDefined()
      expect(result.dataIntegrity).toBe('excellent')
      expect(mockFetchDraws).toHaveBeenCalled()
    })

    it('should batch import data', async () => {
      const mockImportData = vi.fn().mockResolvedValue(generateMockDraws(100))
      const mockValidateDraw = vi.fn().mockReturnValue({
        valid: true,
        errors: []
      })
      const mockAddDraw = vi.fn().mockResolvedValue({ success: true })

      vi.spyOn(dataStore, 'importData').mockImplementation(mockImportData)
      vi.spyOn(dataStore, 'validateDraw').mockImplementation(mockValidateDraw)
      vi.spyOn(dataStore, 'addDraw').mockImplementation(mockAddDraw)

      const result = await lotteryOps.batchImportData('/test/path.csv', {
        batchSize: 20,
        progressCallback: vi.fn()
      })

      expect(result).toBeDefined()
      expect(result.total).toBe(100)
      expect(result.imported).toBe(100)
      expect(result.successRate).toBe(1)
    })
  })

  describe('Smart Workflows', () => {
    it('should perform intelligent refresh when needed', async () => {
      // Mock old data
      const oldDate = new Date(Date.now() - 10 * 24 * 60 * 60 * 1000)
      dataStore.$patch({
        draws: [{
          ...generateMockDraws(1)[0],
          draw_date: oldDate.toISOString()
        }]
      })

      const mockSynchronizeData = vi.fn().mockResolvedValue({
        dataIntegrity: 'excellent',
        duration: 1000
      })
      const mockSmartPrediction = vi.fn().mockResolvedValue({
        predictions: generateMockPredictions(3)
      })

      vi.spyOn(lotteryOps, 'synchronizeData').mockImplementation(mockSynchronizeData)
      vi.spyOn(lotteryOps, 'generateSmartPrediction').mockImplementation(mockSmartPrediction)

      const result = await lotteryOps.intelligentRefresh()

      expect(result).toBe(true)
      expect(mockSynchronizeData).toHaveBeenCalled()
    })

    it('should skip refresh when data is fresh', async () => {
      // Mock recent data
      const recentDate = new Date(Date.now() - 2 * 60 * 60 * 1000)
      dataStore.$patch({
        draws: [{
          ...generateMockDraws(1)[0],
          draw_date: recentDate.toISOString()
        }]
      })

      const result = await lotteryOps.intelligentRefresh()

      expect(result).toBe(false)
    })

    it('should optimize performance', async () => {
      const mockClearCache = vi.fn()
      const mockOptimize = vi.fn()

      vi.spyOn(dataStore, 'clearCache').mockImplementation(mockClearCache)
      vi.spyOn(predictionsStore, 'clearCache').mockImplementation(mockClearCache)
      vi.spyOn(uiStore, 'optimize').mockImplementation(mockOptimize)

      const result = await lotteryOps.optimizePerformance()

      expect(result).toBeDefined()
      expect(mockClearCache).toHaveBeenCalledTimes(2)
      expect(mockOptimize).toHaveBeenCalled()
    })
  })

  describe('Error Handling', () => {
    it('should handle validation errors gracefully', async () => {
      vi.spyOn(predictionsStore, 'analyzeHotCold').mockRejectedValue(new Error('Validation failed'))

      await expect(
        lotteryOps.performCompleteAnalysis()
      ).rejects.toThrow('Validation failed')

      expect(uiStore.lastError).not.toBeNull()
    })

    it('should handle API errors with retry', async () => {
      let attempts = 0
      const mockFn = vi.fn().mockImplementation(() => {
        attempts++
        if (attempts < 3) {
          throw new Error('API error')
        }
        return Promise.resolve({ success: true })
      })

      const operation = lotteryOps.createOperation('api_call', 'Test API call', mockFn)

      const result = await lotteryOps.executeOperation(operation)

      expect(result).toEqual({ success: true })
      expect(attempts).toBe(3)
    })

    it('should handle critical errors without retry', async () => {
      const mockFn = vi.fn().mockRejectedValue(new Error('Critical error'))
      const operation = lotteryOps.createOperation('critical_operation', 'Critical operation', mockFn, 'critical')

      await expect(lotteryOps.executeOperation(operation)).rejects.toThrow('Critical error')

      expect(mockFn).toHaveBeenCalledTimes(1) // Should not retry critical errors
    })
  })
})