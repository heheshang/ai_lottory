/**
 * Core types for Super Lotto system
 */

// Algorithm Types
export type AlgorithmId = 
  | 'WEIGHTED_FREQUENCY'
  | 'PATTERN_BASED'
  | 'MARKOV_CHAIN'
  | 'ENSEMBLE'
  | 'HOT_NUMBERS'
  | 'COLD_NUMBERS'
  | 'POSITION_ANALYSIS'

export interface AlgorithmConfig {
  id: AlgorithmId
  name: string
  description: string
  enabled: boolean
  parameters?: Record<string, any>
}

// Draw Types
export interface SuperLottoDraw {
  id: number
  draw_number: string
  draw_date: string
  front_numbers: number[]
  back_numbers: number[]
  prize_pool?: number
  sales_amount?: number
  jackpot_winners?: number
  created_at: string
  updated_at: string
}

// Prediction Types
export interface PredictionResult {
  id: number
  algorithm: string
  algorithm_id: AlgorithmId
  front_numbers: number[]
  back_numbers: number[]
  confidence_score: number
  reasoning: PredictionReasoning
  analysis_period_days: number
  sample_size: number
  created_at: string
  updated_at: string
  is_validated: boolean
  accuracy?: number
  validation_result?: ValidationResult
}

export interface PredictionReasoning {
  method: string
  hot_numbers?: number[]
  cold_numbers?: number[]
  frequency_analysis?: FrequencyAnalysis
  pattern_analysis?: PatternAnalysis
  confidence_factors?: ConfidenceFactor[]
  recommendation?: string
}

export interface FrequencyAnalysis {
  front_zone: FrequencyData[]
  back_zone: FrequencyData[]
}

export interface FrequencyData {
  number: number
  count: number
  frequency: number
  last_seen: number
}

export interface PatternAnalysis {
  odd_even_ratio: number
  sum_range: [number, number]
  consecutive_pairs: number[]
  gap_patterns: number[]
}

export interface ConfidenceFactor {
  factor: string
  weight: number
  impact: 'positive' | 'negative' | 'neutral'
  description: string
}

export interface ValidationResult {
  hit_count_front: number
  hit_count_back: number
  total_hits: number
  accuracy: number
  validated_at: string
  actual_draw: SuperLottoDraw
}

// Prediction Request Types
export interface PredictionParams {
  algorithm: AlgorithmId
  analysis_period_days?: number
  custom_parameters?: Record<string, any>
  include_reasoning?: boolean
}

export interface BatchPredictionRequest {
  algorithms: AlgorithmId[]
  analysis_period_days: number
  include_reasoning: boolean
}

export interface BatchPredictionResult {
  id: number
  request_id: string
  predictions: PredictionResult[]
  generated_at: string
  total_predictions: number
  successful_predictions: number
  failed_predictions: number
  processing_time_ms: number
  analysis_period_days: number
  sample_size: number
}

// Analysis Types
export interface StatisticsData {
  hot_numbers: HotColdNumber[]
  cold_numbers: HotColdNumber[]
  frequency_distribution: FrequencyDistribution
  position_patterns: PositionPattern[]
  odd_even_distribution: OddEvenDistribution
  sum_range_analysis: SumRangeAnalysis
  consecutive_patterns: ConsecutivePattern[]
  gap_patterns: GapPattern[]
}

export interface HotColdNumber {
  number: number
  count: number
  percentage: number
  last_appearance: number
  trend: 'rising' | 'falling' | 'stable'
}

export interface FrequencyDistribution {
  labels: number[]
  front_zone_counts: number[]
  back_zone_counts: number[]
}

export interface PositionPattern {
  position: number
  most_frequent: number[]
  average: number
  distribution: Record<number, number>
}

export interface OddEvenDistribution {
  front_zone: {
    odd: number
    even: number
    ratio: number
  }
  back_zone: {
    odd: number
    even: number
    ratio: number
  }
}

export interface SumRangeAnalysis {
  ranges: SumRange[]
  most_common_range: string
  average_sum: number
}

export interface SumRange {
  range: string
  count: number
  percentage: number
}

export interface ConsecutivePattern {
  pattern: number[]
  count: number
  frequency: number
}

export interface GapPattern {
  number: number
  average_gap: number
  max_gap: number
  current_gap: number
  prediction: string
}

// Query Types
export interface DrawQueryParams {
  limit?: number
  offset?: number
  start_date?: string
  end_date?: string
  order_by?: 'draw_date' | 'draw_number'
  order_direction?: 'asc' | 'desc'
}

export interface PredictionQueryParams {
  algorithm?: AlgorithmId
  limit?: number
  offset?: number
  is_validated?: boolean
  min_confidence?: number
}

// Filter Types
export interface SearchFilters {
  drawNumber?: string
  startDate?: string
  endDate?: string
  minPrizePool?: number
  maxPrizePool?: number
  hasWinners?: boolean
}

export interface AnalysisFilters {
  periodDays: number
  includeWeekends: boolean
  minDraws: number
  algorithms: AlgorithmId[]
}

// =============================================================================
// Shared Utility Types
// =============================================================================

export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical'

export interface ErrorDetails {
  technicalMessage?: string
  originalError?: string
  stack?: string
  context?: Record<string, unknown>
  userAgent?: string
  timestamp?: string
  url?: string
  [key: string]: unknown
}

export interface ErrorInfo {
  code: string
  message: string
  details?: ErrorDetails
  timestamp: string
  request_id?: string
  severity: ErrorSeverity
  recoverable: boolean
  suggestions?: string[]
}

export interface ApiMetadata {
  request_id?: string
  processing_time_ms?: number
  cache_hit?: boolean
  [key: string]: unknown
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: ErrorInfo
  metadata?: ApiMetadata
}

export const createApiResponse = <T>(
  success: boolean,
  data?: T,
  error?: ErrorInfo,
  metadata: ApiMetadata = {}
): ApiResponse<T> => ({
  success,
  data,
  error,
  metadata
})

export const createErrorResponse = (
  error: ErrorInfo,
  metadata: ApiMetadata = {}
): ApiResponse<null> => ({
  success: false,
  error,
  metadata
})

export interface LoadingState {
  loading: boolean
  loading_text?: string
  progress?: number
  cancellable: boolean
}

export interface ErrorState {
  has_error: boolean
  error_message?: string
  error_code?: string
  retry_count: number
  can_retry: boolean
}

export interface SelectionState<T> {
  selected_items: T[]
  selected_ids: string[]
  last_selected?: T
  selection_mode: 'single' | 'multiple'
}

export interface FilterDefinition {
  id?: string
  name?: string
  date_range?: {
    start_date: string
    end_date: string
  }
  algorithm_ids?: AlgorithmId[]
  [key: string]: unknown
}

export interface FilterState {
  active_filters: FilterDefinition[]
  saved_filters: FilterDefinition[]
  current_preset?: string
}

export interface SearchParams {
  query?: string
  start_date?: string
  end_date?: string
  filters?: Record<string, unknown>
}

export interface PaginationParams {
  page?: number
  limit?: number
  sort_by?: string
  sort_direction?: 'asc' | 'desc'
}