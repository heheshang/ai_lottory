// Enhanced Super Lotto API Client
// Provides type-safe API calls with comprehensive error handling and caching

import { invoke } from '@tauri-apps/api/tauri'
import type {
  SuperLottoDraw,
  PredictionResult,
  BatchPredictionRequest,
  BatchPredictionResult,
  PredictionComparison,
  UnifiedTableData,
  ExportRequest,
  ExportResult,
  HotNumberAnalysis,
  ColdNumberAnalysis,
  PatternAnalysis,
  AlgorithmId,
  ApiResponse,
  ErrorInfo,
  SearchParams,
  PaginationParams,
  createApiResponse,
  VALIDATION_RULES
} from '@/types/superLotto'

import { handleError, ErrorCode, createSafeAsyncFunction } from '@/utils/errorHandler'

// =============================================================================
// API Configuration
// =============================================================================

interface ApiConfig {
  baseUrl?: string
  timeout: number
  retryAttempts: number
  enableCaching: boolean
  cacheTimeout: number
}

const DEFAULT_CONFIG: ApiConfig = {
  timeout: 30000,
  retryAttempts: 3,
  enableCaching: true,
  cacheTimeout: 300000 // 5 minutes
}

interface CacheEntry<T> {
  data: ApiResponse<T>;
  timestamp: number;
}

class SuperLottoApi {
  private config: ApiConfig
  private cache: Map<string, CacheEntry> = new Map()

  constructor(config: Partial<ApiConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config }
  }

  // =============================================================================
  // Core API Method
  // =============================================================================

  private async invokeCommand<T>(
    command: string,
    args?: Record<string, unknown>,
    options?: {
      timeout?: number
      retries?: number
      cache?: boolean
      cacheKey?: string
    }
  ): Promise<ApiResponse<T>> {
    const {
      timeout = this.config.timeout,
      retries = this.config.retryAttempts,
      cache = this.config.enableCaching,
      cacheKey
    } = options || {}

    // Check cache first
    if (cache && cacheKey) {
      const cached = this.getFromCache<T>(cacheKey)
      if (cached) {
        return cached
      }
    }

    const requestId = crypto.randomUUID()
    const startTime = Date.now()

    try {
      console.log(`🚀 [API] Invoking command: ${command}`, { args, requestId })

      // Create timeout promise
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Request timeout')), timeout)
      })

      // Create main promise
      const commandPromise = this.executeWithRetry(command, args, retries)

      // Race between timeout and command
      const result = await Promise.race([commandPromise, timeoutPromise])

      const processingTime = Date.now() - startTime
      console.log(`✅ [API] Command ${command} completed in ${processingTime}ms`)

      const response = createApiResponse(true, result, undefined, {
        request_id: requestId,
        processing_time_ms: processingTime,
        cache_hit: false
      })

      // Cache result if enabled
      if (cache && cacheKey) {
        this.setCache(cacheKey, response)
      }

      return response

    } catch (error) {
      const processingTime = Date.now() - startTime
      console.error(`❌ [API] Command ${command} failed after ${processingTime}ms:`, error)

      let errorInfo: ErrorInfo
      if (error instanceof Error) {
        errorInfo = handleError(error, { command, args }, requestId)
      } else {
        errorInfo = handleError(
          ErrorCode.API_SERVER_ERROR,
          { command, args, originalError: error },
          requestId
        )
      }

      return createApiResponse(false, undefined, errorInfo, {
        request_id: requestId,
        processing_time_ms: processingTime,
        cache_hit: false
      })
    }
  }

  private async executeWithRetry<T>(
    command: string,
    args?: Record<string, unknown>,
    retries: number = 0
  ): Promise<T> {
    let lastError: Error

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        return await invoke<T>(command, args)
      } catch (error) {
        lastError = error as Error

        if (attempt === retries) {
          break
        }

        // Exponential backoff
        const delay = Math.min(1000 * Math.pow(2, attempt), 5000)
        console.log(`🔄 [API] Retrying command ${command} in ${delay}ms (attempt ${attempt + 1}/${retries})`)
        await new Promise(resolve => setTimeout(resolve, delay))
      }
    }

    throw lastError!
  }

  // =============================================================================
  // Cache Management
  // =============================================================================

  private getFromCache<T>(key: string): ApiResponse<T> | null {
    const entry = this.cache.get(key) as CacheEntry<T>;
    if (!entry) return null;

    const age = Date.now() - entry.timestamp
    if (age > this.config.cacheTimeout) {
      this.cache.delete(key);
      return null;
    }

    return {
      ...entry.data,
      metadata: {
        ...entry.data.metadata,
        cache_hit: true
      }
    }
  }

  private setCache<T>(key: string, data: ApiResponse<T>): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now()
    })
  }

  public clearCache(pattern?: string): void {
    if (pattern) {
      const regex = new RegExp(pattern)
      for (const key of this.cache.keys()) {
        if (regex.test(key)) {
          this.cache.delete(key)
        }
      }
    } else {
      this.cache.clear()
    }
  }

  // =============================================================================
  // Super Lotto Draw Operations
  // =============================================================================

  async getDraws(params: PaginationParams & {
    start_date?: string
    end_date?: string
    draw_number?: string
  } = {}): Promise<ApiResponse<{ draws: SuperLottoDraw[]; total: number }>> {
    const cacheKey = `draws:${JSON.stringify(params)}`

    return this.invokeCommand('get_super_lotto_draws', params, {
      cacheKey,
      cache: true
    })
  }

  async getDrawById(id: number): Promise<ApiResponse<SuperLottoDraw>> {
    return this.invokeCommand('get_super_lotto_draw_by_id', { id }, {
      cacheKey: `draw:${id}`,
      cache: true
    })
  }

  async importDraws(draws: Partial<SuperLottoDraw>[]): Promise<ApiResponse<{
    imported: number
    failed: number
    errors: string[]
  }>> {
    // Clear cache when importing new data
    this.clearCache('draws:')

    return this.invokeCommand('import_super_lotto_draws', { draws }, {
      cache: false // Don't cache import operations
    })
  }

  // =============================================================================
  // Analysis Operations
  // =============================================================================

  async analyzeHotNumbers(params: {
    days: number
    zone?: 'FRONT' | 'BACK' | 'BOTH'
    limit?: number
  }): Promise<ApiResponse<HotNumberAnalysis[]>> {
    const cacheKey = `hot_numbers:${JSON.stringify(params)}`

    return this.invokeCommand('analyze_hot_numbers', params, {
      cacheKey,
      cache: true
    })
  }

  async analyzeColdNumbers(params: {
    days: number
    zone?: 'FRONT' | 'BACK' | 'BOTH'
    limit?: number
  }): Promise<ApiResponse<ColdNumberAnalysis[]>> {
    const cacheKey = `cold_numbers:${JSON.stringify(params)}`

    return this.invokeCommand('analyze_cold_numbers', params, {
      cacheKey,
      cache: true
    })
  }

  async analyzePatterns(params: {
    pattern_types?: string[]
    days: number
  }): Promise<ApiResponse<PatternAnalysis[]>> {
    const cacheKey = `patterns:${JSON.stringify(params)}`

    return this.invokeCommand('get_pattern_analysis', params, {
      cacheKey,
      cache: true
    })
  }

  // =============================================================================
  // Prediction Operations
  // =============================================================================

  async generatePrediction(params: {
    algorithm: AlgorithmId
    analysis_period_days?: number
    custom_parameters?: Record<string, unknown>
    include_reasoning?: boolean
  }): Promise<ApiResponse<PredictionResult>> {
    return this.invokeCommand('generate_prediction', params, {
      cache: false // Don't cache predictions
    })
  }

  async getPredictions(params: SearchParams & {
    algorithm_id?: AlgorithmId
    validated_only?: boolean
  } = {}): Promise<ApiResponse<{
    predictions: PredictionResult[]
    total: number
  }>> {
    const cacheKey = `predictions:${JSON.stringify(params)}`

    return this.invokeCommand('get_predictions', params, {
      cacheKey,
      cache: true
    })
  }

  async validatePrediction(predictionId: string): Promise<ApiResponse<PredictionResult>> {
    return this.invokeCommand('validate_prediction', { prediction_id: predictionId }, {
      cache: false
    })
  }

  async getPredictionById(id: string): Promise<ApiResponse<PredictionResult>> {
    return this.invokeCommand('get_prediction_by_id', { id }, {
      cacheKey: `prediction:${id}`,
      cache: true
    })
  }

  // =============================================================================
  // Batch Prediction Operations
  // =============================================================================

  async generateBatchPredictions(request: BatchPredictionRequest): Promise<ApiResponse<BatchPredictionResult>> {
    return this.invokeCommand('generate_all_predictions', { request }, {
      timeout: 60000, // Longer timeout for batch operations
      cache: false
    })
  }

  async getPredictionComparison(drawNumber: number, days?: number): Promise<ApiResponse<PredictionComparison>> {
    const cacheKey = `comparison:${drawNumber}:${days || 90}`

    return this.invokeCommand('get_prediction_comparison', { drawNumber, days }, {
      cacheKey,
      cache: true
    })
  }

  // =============================================================================
  // Unified Table Operations
  // =============================================================================

  async getUnifiedTableData(params: SearchParams & {
    include_predictions?: boolean
    algorithm_ids?: AlgorithmId[]
  } = {}): Promise<ApiResponse<UnifiedTableData>> {
    const cacheKey = `unified_table:${JSON.stringify(params)}`

    return this.invokeCommand('get_unified_table_data', params, {
      cacheKey,
      cache: true
    })
  }

  // =============================================================================
  // Export Operations
  // =============================================================================

  async exportData(request: ExportRequest): Promise<ApiResponse<ExportResult>> {
    return this.invokeCommand('export_table_data', { exportRequest: request }, {
      timeout: 60000, // Longer timeout for export operations
      cache: false
    })
  }

  async downloadExport(filename: string): Promise<void> {
    try {
      const result = await this.invokeCommand<string>('download_export_file', { filename })

      if (result.success && result.data) {
        // Create download link
        const blob = new Blob([result.data])
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = filename
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
      } else {
        throw new Error(result.error?.message || 'Download failed')
      }
    } catch (error) {
      handleError(error as Error, { filename })
      throw error
    }
  }

  // =============================================================================
  // Algorithm Operations
  // =============================================================================

  async getAvailableAlgorithms(): Promise<ApiResponse<AlgorithmId[]>> {
    return this.invokeCommand('get_available_algorithms', {}, {
      cacheKey: 'algorithms',
      cache: true
    })
  }

  async getAlgorithmConfig(algorithmId: AlgorithmId): Promise<ApiResponse<any>> {
    const cacheKey = `algorithm_config:${algorithmId}`

    return this.invokeCommand('get_algorithm_config', { algorithm_id: algorithmId }, {
      cacheKey,
      cache: true
    })
  }

  // =============================================================================
  // Utility Methods
  // =============================================================================

  async validateSuperLottoNumbers(redNumbers: number[], blueNumber: number): Promise<ApiResponse<{
    valid: boolean
    errors: string[]
  }>> {
    const errors: string[] = []

    // Validate red numbers
    if (redNumbers.length !== VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.count) {
      errors.push(`前区号码数量必须为${VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.count}个`)
    }

    if (new Set(redNumbers).size !== redNumbers.length) {
      errors.push('前区号码不能重复')
    }

    for (const num of redNumbers) {
      if (num < VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.range[0] ||
          num > VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.range[1]) {
        errors.push(`前区号码必须在${VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.range[0]}-${VALIDATION_RULES.SUPER_LOTTO.RED_NUMBERS.range[1]}范围内`)
      }
    }

    // Validate blue number
    if (blueNumber < VALIDATION_RULES.SUPER_LOTTO.BLUE_NUMBER.range[0] ||
        blueNumber > VALIDATION_RULES.SUPER_LOTTO.BLUE_NUMBER.range[1]) {
      errors.push(`后区号码必须在${VALIDATION_RULES.SUPER_LOTTO.BLUE_NUMBER.range[0]}-${VALIDATION_RULES.SUPER_LOTTO.BLUE_NUMBER.range[1]}范围内`)
    }

    return createApiResponse(true, {
      valid: errors.length === 0,
      errors
    })
  }

  // Health check
  async healthCheck(): Promise<ApiResponse<{ status: string; timestamp: string }>> {
    return this.invokeCommand('health_check', {}, {
      timeout: 5000,
      cache: false
    })
  }
}

// =============================================================================
// API Instance
// =============================================================================

export const superLottoApi = new SuperLottoApi()

// =============================================================================
// Vue Composable for API Usage
// =============================================================================

import { ref, reactive, computed } from 'vue'

export function useSuperLottoApi() {
  const isLoading = ref(false)
  const error = ref<ErrorInfo | null>(null)
  const lastRequestTime = ref<number>(0)

  const requestState = reactive({
    pendingRequests: 0,
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0
  })

  const isOnline = computed(() => navigator.onLine)
  const hasPendingRequests = computed(() => requestState.pendingRequests > 0)
  const successRate = computed(() => {
    if (requestState.totalRequests === 0) return 0
    return (requestState.successfulRequests / requestState.totalRequests) * 100
  })

  const executeRequest = async <T>(
    requestFn: () => Promise<ApiResponse<T>>,
    options?: {
      showLoading?: boolean
      showError?: boolean
      retryOnError?: boolean
    }
  ): Promise<T | null> => {
    const {
      showLoading = true,
      showError = true,
      retryOnError = true
    } = options || {}

    if (showLoading) {
      isLoading.value = true
    }

    requestState.pendingRequests++
    requestState.totalRequests++

    try {
      const response = await requestFn()
      lastRequestTime.value = Date.now()

      if (response.success && response.data) {
        requestState.successfulRequests++
        return response.data
      } else {
        requestState.failedRequests++
        if (response.error && showError) {
          error.value = response.error
        }
        return null
      }
    } catch (err) {
      requestState.failedRequests++
      const errorInfo = handleError(err as Error)
      error.value = errorInfo

      if (retryOnError && errorInfo.recoverable) {
        console.log('🔄 Retrying request due to recoverable error...')
        return executeRequest(requestFn, { ...options, retryOnError: false })
      }

      return null
    } finally {
      requestState.pendingRequests--
      if (showLoading && requestState.pendingRequests === 0) {
        isLoading.value = false
      }
    }
  }

  const clearError = () => {
    error.value = null
  }

  const resetStats = () => {
    requestState.pendingRequests = 0
    requestState.totalRequests = 0
    requestState.successfulRequests = 0
    requestState.failedRequests = 0
  }

  return {
    // State
    isLoading,
    error,
    lastRequestTime,
    requestState,
    isOnline,
    hasPendingRequests,
    successRate,

    // Methods
    executeRequest,
    clearError,
    resetStats,

    // API methods (wrapped with error handling)
    getDraws: (params?: any) => executeRequest(() => superLottoApi.getDraws(params)),
    getDrawById: (id: number) => executeRequest(() => superLottoApi.getDrawById(id)),
    importDraws: (draws: any[]) => executeRequest(() => superLottoApi.importDraws(draws)),
    analyzeHotNumbers: (params: any) => executeRequest(() => superLottoApi.analyzeHotNumbers(params)),
    analyzeColdNumbers: (params: any) => executeRequest(() => superLottoApi.analyzeColdNumbers(params)),
    analyzePatterns: (params: any) => executeRequest(() => superLottoApi.analyzePatterns(params)),
    generatePrediction: (params: any) => executeRequest(() => superLottoApi.generatePrediction(params)),
    getPredictions: (params?: any) => executeRequest(() => superLottoApi.getPredictions(params)),
    validatePrediction: (id: string) => executeRequest(() => superLottoApi.validatePrediction(id)),
    generateBatchPredictions: (request: BatchPredictionRequest) =>
      executeRequest(() => superLottoApi.generateBatchPredictions(request)),
    getUnifiedTableData: (params?: any) => executeRequest(() => superLottoApi.getUnifiedTableData(params)),
    exportData: (request: ExportRequest) => executeRequest(() => superLottoApi.exportData(request)),
    validateSuperLottoNumbers: (redNumbers: number[], blueNumber: number) =>
      executeRequest(() => superLottoApi.validateSuperLottoNumbers(redNumbers, blueNumber)),
    healthCheck: () => executeRequest(() => superLottoApi.healthCheck())
  }
}

// =============================================================================
// Legacy Compatibility
// =============================================================================

// Export legacy functions for backward compatibility
export const generateAllPredictions = createSafeAsyncFunction(
  (request: BatchPredictionRequest) => superLottoApi.generateBatchPredictions(request)
)

export const getPredictionComparison = createSafeAsyncFunction(
  (drawNumber: number, days?: number) => superLottoApi.getPredictionComparison(drawNumber, days)
)

export const getUnifiedTableData = createSafeAsyncFunction(
  (limit: number = 100, offset: number = 0, includePredictions: boolean = true, algorithmIds?: AlgorithmId[]) =>
    superLottoApi.getUnifiedTableData({ limit, offset, include_predictions: includePredictions, algorithm_ids: algorithmIds })
)

export const exportTableData = createSafeAsyncFunction(
  (exportRequest: ExportRequest) => superLottoApi.exportData(exportRequest)
)

console.log('🔗 [Super Lotto API] Enhanced API client initialized with error handling')