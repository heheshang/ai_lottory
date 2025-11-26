/**
 * Unified API Layer - Single interface for all backend operations
 *
 * This file provides a consolidated, type-safe interface to all Tauri commands
 * and external API calls. It abstracts away the complexity of different endpoints
 * and provides consistent error handling, caching, and retry logic.
 *
 * Key Features:
 * - Type-safe API calls
 * - Automatic error handling and retry
 * - Response caching and deduplication
 * - Request/Response interceptors
 * - Performance monitoring
 * - Offline support capabilities
 */

import { invoke } from '@tauri-apps/api/core'
import { useCache } from '../composables/useAPIcache'
import { useErrorHandler } from '../composables/useErrorHandler'
import { useUIStore } from '../stores/ui'

// =============================================================================
// Type Definitions
// =============================================================================

export interface APIRequest<T = any> {
  endpoint: string
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  data?: T
  params?: Record<string, any>
  headers?: Record<string, string>
  cache?: {
    ttl?: number
    key?: string
    invalidate?: string[]
  }
  timeout?: number
  retries?: number
  priority?: 'low' | 'medium' | 'high' | 'critical'
  bypassCache?: boolean
}

export interface APIResponse<T = any> {
  data: T
  status: number
  statusText: string
  headers: Record<string, string>
  cached: boolean
  requestId: string
  duration: number
}

export interface APIError {
  code: string
  message: string
  details?: any
  retryable: boolean
  endpoint: string
  requestId: string
  timestamp: Date
}

// =============================================================================
// API Configuration
// =============================================================================

interface APIConfig {
  baseURL: string
  timeout: number
  retries: number
  cache: {
    enabled: boolean
    defaultTTL: number
    maxSize: number
  }
  interceptors: {
    request: Array<(request: APIRequest) => APIRequest>
    response: Array<(response: APIResponse) => APIResponse>
    error: Array<(error: APIError) => void>
  }
  performance: {
    enableMetrics: boolean
    slowRequestThreshold: number
    enableTracing: boolean
  }
}

const defaultConfig: APIConfig = {
  baseURL: '',
  timeout: 30000,
  retries: 3,
  cache: {
    enabled: true,
    defaultTTL: 5 * 60 * 1000, // 5 minutes
    maxSize: 100
  },
  interceptors: {
    request: [],
    response: [],
    error: []
  },
  performance: {
    enableMetrics: true,
    slowRequestThreshold: 2000, // 2 seconds
    enableTracing: true
  }
}

// =============================================================================
// Unified API Class
// =============================================================================

export class UnifiedAPI {
  private config: APIConfig
  private cache = useCache('unified-api')
  private errorHandler = useErrorHandler()
  private uiStore = useUIStore()
  private metrics = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    cacheHits: 0,
    averageResponseTime: 0,
    slowRequests: 0
  }

  constructor(config: Partial<APIConfig> = {}) {
    this.config = { ...defaultConfig, ...config }
  }

  // =============================================================================
  // Core Request Method
  // =============================================================================

  async request<T = any>(requestConfig: APIRequest): Promise<APIResponse<T>> {
    const requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const startTime = Date.now()

    // Apply request interceptors
    let request = { ...requestConfig, requestId }
    for (const interceptor of this.config.interceptors.request) {
      request = interceptor(request)
    }

    // Check cache first
    if (!request.bypassCache && this.config.cache.enabled) {
      const cached = await this.checkCache<T>(request)
      if (cached) {
        this.metrics.cacheHits++
        this.updateMetrics(Date.now() - startTime, true)
        return cached
      }
    }

    this.metrics.totalRequests++

    try {
      // Execute request
      const response = await this.executeRequest<T>(request)

      // Apply response interceptors
      let processedResponse = response
      for (const interceptor of this.config.interceptors.response) {
        processedResponse = interceptor(processedResponse)
      }

      // Cache successful response
      if (this.config.cache.enabled && !request.bypassCache) {
        await this.cacheResponse(request, processedResponse)
      }

      // Update metrics
      this.updateMetrics(processedResponse.duration, true)
      this.metrics.successfulRequests++

      return processedResponse
    } catch (error) {
      const apiError = this.createAPIError(error, request, requestId)

      // Apply error interceptors
      for (const interceptor of this.config.interceptors.error) {
        interceptor(apiError)
      }

      // Handle error and retry if needed
      if (apiError.retryable && request.retries! > 0) {
        this.uiStore.showWarning('Request Retry', `Retrying ${request.endpoint}...`)
        return this.request({ ...request, retries: request.retries! - 1 })
      }

      this.metrics.failedRequests++
      this.updateMetrics(Date.now() - startTime, false)

      // Handle through error handler
      this.errorHandler.handleError({
        type: 'network',
        severity: 'high',
        message: apiError.message,
        details: apiError,
        context: { endpoint: request.endpoint, requestId }
      })

      throw apiError
    }
  }

  // =============================================================================
  // Tauri Command Execution
  // =============================================================================

  private async executeRequest<T = any>(request: APIRequest): Promise<APIResponse<T>> {
    const startTime = Date.now()

    try {
      // Execute Tauri command
      const data = await invoke<T>(request.endpoint, request.data || {})

      const response: APIResponse<T> = {
        data,
        status: 200,
        statusText: 'OK',
        headers: {},
        cached: false,
        requestId: request.requestId,
        duration: Date.now() - startTime
      }

      // Check for slow requests
      if (response.duration > this.config.performance.slowRequestThreshold) {
        this.metrics.slowRequests++
        console.warn(`Slow API request: ${request.endpoint} took ${response.duration}ms`)
      }

      return response
    } catch (error) {
      throw {
        message: error instanceof Error ? error.message : 'Unknown error',
        code: 'TAURI_ERROR',
        endpoint: request.endpoint,
        requestId: request.requestId
      }
    }
  }

  // =============================================================================
  // Caching
  // =============================================================================

  private async checkCache<T = any>(request: APIRequest): Promise<APIResponse<T> | null> {
    if (!request.cache) return null

    const cacheKey = request.cache.key || this.generateCacheKey(request)
    const cached = this.cache.get<APIResponse<T>>(cacheKey)

    if (cached && Date.now() - cached.timestamp < (request.cache.ttl || this.config.cache.defaultTTL)) {
      return { ...cached, cached: true }
    }

    return null
  }

  private async cacheResponse<T = any>(request: APIRequest, response: APIResponse<T>): Promise<void> {
    if (!request.cache) return

    const cacheKey = request.cache.key || this.generateCacheKey(request)
    const cacheEntry = {
      ...response,
      timestamp: Date.now()
    }

    this.cache.set(cacheKey, cacheEntry, {
      ttl: request.cache.ttl || this.config.cache.defaultTTL
    })

    // Invalidate related cache entries
    if (request.cache.invalidate) {
      for (const key of request.cache.invalidate) {
        this.cache.clearPattern(key)
      }
    }
  }

  private generateCacheKey(request: APIRequest): string {
    const { endpoint, method = 'GET', data, params } = request
    const keyData = JSON.stringify({ endpoint, method, data, params })
    return `${endpoint}_${Buffer.from(keyData).toString('base64')}`
  }

  // =============================================================================
  // Error Handling
  // =============================================================================

  private createAPIError(error: any, request: APIRequest, requestId: string): APIError {
    return {
      code: error.code || 'UNKNOWN_ERROR',
      message: error.message || 'An unknown error occurred',
      details: error.details || error,
      retryable: error.retryable ?? true,
      endpoint: request.endpoint,
      requestId,
      timestamp: new Date()
    }
  }

  // =============================================================================
  // Metrics and Monitoring
  // =============================================================================

  private updateMetrics(duration: number, success: boolean): void {
    if (!this.config.performance.enableMetrics) return

    const totalRequests = this.metrics.totalRequests
    const currentAvgTime = this.metrics.averageResponseTime

    this.metrics.averageResponseTime =
      (currentAvgTime * (totalRequests - 1) + duration) / totalRequests
  }

  getMetrics() {
    return {
      ...this.metrics,
      errorRate: this.metrics.totalRequests > 0 ? this.metrics.failedRequests / this.metrics.totalRequests : 0,
      cacheHitRate: this.metrics.totalRequests > 0 ? this.metrics.cacheHits / this.metrics.totalRequests : 0,
      successRate: this.metrics.totalRequests > 0 ? this.metrics.successfulRequests / this.metrics.totalRequests : 0
    }
  }

  // =============================================================================
  // Convenience Methods
  // =============================================================================

  // Authentication endpoints
  async login(credentials: { username: string; password: string }) {
    return this.request({
      endpoint: 'login',
      method: 'POST',
      data: credentials,
      cache: { enabled: false },
      priority: 'high'
    })
  }

  async logout() {
    return this.request({
      endpoint: 'logout',
      method: 'POST',
      cache: { enabled: false },
      priority: 'medium'
    })
  }

  async getCurrentUser() {
    return this.request({
      endpoint: 'get_current_user',
      method: 'GET',
      cache: { ttl: 5 * 60 * 1000 }, // 5 minutes
      priority: 'medium'
    })
  }

  // Lottery data endpoints
  async getLotteryHistory(options: { limit?: number; offset?: number; force?: boolean } = {}) {
    return this.request({
      endpoint: 'get_lottery_history',
      method: 'GET',
      data: options,
      cache: {
        ttl: options.force ? 0 : 10 * 60 * 1000, // 10 minutes
        key: options.force ? undefined : 'lottery_history'
      },
      priority: 'medium'
    })
  }

  async addLotteryDraw(draw: any) {
    return this.request({
      endpoint: 'add_lottery_draw',
      method: 'POST',
      data: { draw },
      cache: {
        enabled: false,
        invalidate: ['lottery_history*', 'lottery_search*']
      },
      priority: 'high'
    })
  }

  async searchLotteryDraws(query: any) {
    return this.request({
      endpoint: 'search_lottery_draws',
      method: 'POST',
      data: { query },
      cache: {
        ttl: 2 * 60 * 1000, // 2 minutes
        key: `lottery_search_${JSON.stringify(query)}`
      },
      priority: 'medium'
    })
  }

  // Analysis endpoints
  async getHotNumbers(periodDays?: number) {
    return this.request({
      endpoint: 'get_hot_numbers',
      method: 'GET',
      data: { period_days: periodDays },
      cache: {
        ttl: 15 * 60 * 1000, // 15 minutes
        key: `hot_numbers_${periodDays || 90}`
      },
      priority: 'medium'
    })
  }

  async getColdNumbers(periodDays?: number) {
    return this.request({
      endpoint: 'get_cold_numbers',
      method: 'GET',
      data: { period_days: periodDays },
      cache: {
        ttl: 15 * 60 * 1000, // 15 minutes
        key: `cold_numbers_${periodDays || 90}`
      },
      priority: 'medium'
    })
  }

  async generateBatchPrediction(request: BatchPredictionRequest) {
    return this.request({
      endpoint: 'generate_batch_prediction',
      method: 'POST',
      data: { request },
      cache: {
        ttl: 5 * 60 * 1000, // 5 minutes
        key: `batch_prediction_${JSON.stringify(request)}`
      },
      priority: 'high',
      timeout: 60000 // 1 minute
    })
  }

  async generatePatternAnalysis(patternTypes: string[], periodDays?: number) {
    return this.request({
      endpoint: 'generate_enhanced_pattern_analysis',
      method: 'POST',
      data: {
        pattern_types: patternTypes,
        period_days: periodDays
      },
      cache: {
        ttl: 20 * 60 * 1000, // 20 minutes
        key: `pattern_analysis_${patternTypes.join('_')}_${periodDays || 90}`
      },
      priority: 'medium'
    })
  }

  async generateConsecutivePatternAnalysis(periodDays?: number) {
    return this.request({
      endpoint: 'generate_consecutive_pattern_analysis',
      method: 'POST',
      data: { period_days: periodDays },
      cache: {
        ttl: 15 * 60 * 1000, // 15 minutes
        key: `consecutive_patterns_${periodDays || 90}`
      },
      priority: 'medium'
    })
  }

  async generateOddEvenAnalysis(periodDays?: number) {
    return this.request({
      endpoint: 'generate_odd_even_distribution_analysis',
      method: 'POST',
      data: { period_days: periodDays },
      cache: {
        ttl: 15 * 60 * 1000, // 15 minutes
        key: `odd_even_analysis_${periodDays || 90}`
      },
      priority: 'medium'
    })
  }

  async generateMarkovPrediction(order?: number, periodDays?: number, timeDecayFactor?: number) {
    return this.request({
      endpoint: 'generate_markov_chain_prediction',
      method: 'POST',
      data: {
        order: order || 2,
        period_days: periodDays || 365,
        time_decay_factor: timeDecayFactor || 0.95
      },
      cache: {
        ttl: 30 * 60 * 1000, // 30 minutes
        key: `markov_prediction_${order || 2}_${periodDays || 365}_${timeDecayFactor || 0.95}`
      },
      priority: 'high'
    })
  }

  async generateHotColdAnalysis(periodDays?: number, hotThreshold?: number, coldThreshold?: number) {
    return this.request({
      endpoint: 'generate_hot_cold_analysis',
      method: 'POST',
      data: {
        period_days: periodDays,
        hot_threshold: hotThreshold,
        cold_threshold: coldThreshold
      },
      cache: {
        ttl: 10 * 60 * 1000, // 10 minutes
        key: `hot_cold_analysis_${periodDays || 90}_${hotThreshold || 0.7}_${coldThreshold || 0.3}`
      },
      priority: 'medium'
    })
  }

  // Data management endpoints
  async importLotteryData(filePath: string) {
    return this.request({
      endpoint: 'import_lottery_data',
      method: 'POST',
      data: { file_path: filePath },
      cache: { enabled: false },
      priority: 'high',
      timeout: 300000 // 5 minutes for large files
    })
  }

  async exportLotteryData(format: 'json' | 'csv', filters?: any) {
    return this.request({
      endpoint: 'export_lottery_data',
      method: 'POST',
      data: { format, filters },
      cache: { enabled: false },
      priority: 'medium',
      timeout: 60000 // 1 minute
    })
  }

  // =============================================================================
  // Cache Management
  // =============================================================================

  clearCache(pattern?: string) {
    if (pattern) {
      this.cache.clearPattern(pattern)
    } else {
      this.cache.clear()
    }
  }

  getCacheInfo() {
    return this.cache.getStats()
  }

  // =============================================================================
  // Configuration
  // =============================================================================

  updateConfig(newConfig: Partial<APIConfig>) {
    this.config = { ...this.config, ...newConfig }
  }

  addRequestInterceptor(interceptor: (request: APIRequest) => APIRequest) {
    this.config.interceptors.request.push(interceptor)
  }

  addResponseInterceptor(interceptor: (response: APIResponse) => APIResponse) {
    this.config.interceptors.response.push(interceptor)
  }

  addErrorInterceptor(interceptor: (error: APIError) => void) {
    this.config.interceptors.error.push(interceptor)
  }

  removeRequestInterceptor(index: number) {
    this.config.interceptors.request.splice(index, 1)
  }

  removeResponseInterceptor(index: number) {
    this.config.interceptors.response.splice(index, 1)
  }

  removeErrorInterceptor(index: number) {
    this.config.interceptors.error.splice(index, 1)
  }
}

// =============================================================================
// Singleton Instance
// =============================================================================

export const api = new UnifiedAPI()

// =============================================================================
// Default Interceptors
// =============================================================================

// Request interceptor for adding common headers
api.addRequestInterceptor((request) => {
  return {
    ...request,
    headers: {
      'Content-Type': 'application/json',
      ...request.headers
    }
  }
})

// Response interceptor for logging successful requests
api.addResponseInterceptor((response) => {
  if (api.getMetrics().successRate > 0.95) {
    console.debug(`API Success: ${response.requestId} - ${response.duration}ms`)
  }
  return response
})

// Error interceptor for user notifications
api.addErrorInterceptor((error) => {
  const uiStore = useUIStore()

  switch (error.code) {
    case 'NETWORK_ERROR':
      uiStore.showError('Network Error', 'Please check your internet connection')
      break
    case 'AUTHENTICATION_ERROR':
      uiStore.showError('Authentication Failed', 'Please log in again')
      break
    case 'VALIDATION_ERROR':
      uiStore.showWarning('Validation Error', error.message)
      break
    default:
      uiStore.showError('Request Failed', error.message)
  }
})

// =============================================================================
// Composable for Vue Components
// =============================================================================

export function useUnifiedAPI() {
  return {
    api,
    getMetrics: () => api.getMetrics(),
    clearCache: (pattern?: string) => api.clearCache(pattern),
    getCacheInfo: () => api.getCacheInfo()
  }
}