// Enhanced Type Definitions for Super Lotto Application
// Replaces scattered type definitions with centralized, strongly-typed interfaces

// =============================================================================
// Core Domain Types
// =============================================================================

export interface SuperLottoDraw {
  readonly id: number
  readonly draw_number: number
  readonly draw_date: string
  readonly red_numbers: number[]  // 5 numbers from 01-35
  readonly blue_number: number    // 1 number from 01-12
  readonly jackpot_amount?: number
  readonly prize_info?: PrizeInfo
  readonly created_at: string
  readonly updated_at?: string
}

export interface PrizeInfo {
  readonly first_prize: number
  readonly first_prize_winners: number
  readonly second_prize: number
  readonly second_prize_winners: number
  readonly total_sales: number
}

// =============================================================================
// Analysis Types
// =============================================================================

export interface HotNumberAnalysis {
  readonly number: number
  readonly frequency: number
  readonly last_drawn?: string
  readonly hot_score: number
  readonly position_frequency: number[]  // frequency in each position
  readonly trend_direction: 'rising' | 'falling' | 'stable'
}

export interface ColdNumberAnalysis {
  readonly number: number
  readonly frequency: number
  readonly last_drawn?: string
  readonly cold_score: number
  readonly current_gap: number
  readonly average_gap: number
  readonly expected_return: number
}

export interface PatternAnalysis {
  readonly id: string
  readonly pattern_type: PatternType
  readonly analysis_data: Record<string, unknown>
  readonly confidence_score: number
  readonly sample_size: number
  readonly period_days: number
  readonly created_at: string
  readonly strength: 'weak' | 'moderate' | 'strong'
}

export type PatternType =
  | 'ODD_EVEN_DISTRIBUTION'
  | 'SUM_RANGES'
  | 'CONSECUTIVE_NUMBERS'
  | 'POSITION_PATTERNS'
  | 'AC_VALUE'
  | 'SPAN_ANALYSIS'

// =============================================================================
// Prediction Types
// =============================================================================

export interface PredictionParameters {
  readonly draw_count?: number
  readonly weight_factor?: number
  readonly pattern_weight?: number
  readonly hot_threshold?: number
  readonly cold_threshold?: number
  readonly time_decay_factor?: number
  readonly ensemble_weights?: EnsembleWeights
  readonly custom_settings?: Record<string, unknown>
}

export interface EnsembleWeights {
  readonly hot_weight: number
  readonly cold_weight: number
  readonly pattern_weight: number
  readonly frequency_weight: number
  readonly position_weight: number
}

export interface PredictionResult {
  readonly id: string
  readonly algorithm_id: AlgorithmId
  readonly algorithm_name: string
  readonly draw_number: number
  readonly predicted_numbers: PredictedNumbers
  readonly confidence_score: number
  readonly created_at: string
  readonly parameters?: PredictionParameters
  readonly accuracy?: number
  readonly is_validated: boolean
  readonly validation_results?: ValidationResults
  readonly performance_metrics?: PerformanceMetrics
}

export interface PredictedNumbers {
  readonly red_numbers: readonly number[]  // 5 numbers
  readonly blue_number: number   // 1 number
  readonly confidence?: NumberConfidence
  readonly reasoning?: NumberReasoning
}

export interface NumberConfidence {
  readonly red_confidence: readonly number[]
  readonly blue_confidence: number
  readonly overall_confidence: number
}

export interface NumberReasoning {
  readonly selection_factors: readonly string[]
  readonly risk_assessment: RiskLevel
  readonly recommendation: string
  readonly alternative_choices?: readonly PredictedNumbers[]
}

export type RiskLevel = 'low' | 'medium' | 'high' | 'very_high'

export interface ValidationResults {
  readonly actual_numbers: SuperLottoDraw
  readonly red_matches: readonly number[]
  readonly blue_match: boolean
  readonly total_matches: number
  readonly accuracy_percentage: number
  readonly prize_won?: PrizeInfo
  readonly validation_date: string
}

export interface PerformanceMetrics {
  readonly historical_accuracy: number
  readonly prediction_count: number
  readonly success_rate: number
  readonly average_confidence: number
  readonly last_updated: string
}

// =============================================================================
// Algorithm Configuration
// =============================================================================

export type AlgorithmId =
  | 'frequency_analysis'
  | 'pattern_recognition'
  | 'hot_cold_balance'
  | 'position_analysis'
  | 'trend_analysis'
  | 'ai_prediction'
  | 'markov_chain'
  | 'ensemble'
  | 'weighted_frequency'

export interface AlgorithmConfig {
  readonly id: AlgorithmId
  readonly name: string
  readonly description: string
  readonly enabled: boolean
  readonly category: AlgorithmCategory
  readonly complexity: ComplexityLevel
  readonly data_requirements: DataRequirements
  readonly parameters: AlgorithmParameter[]
  readonly default_parameters: PredictionParameters
  readonly performance_stats: AlgorithmPerformanceStats
  readonly version: string
}

export type AlgorithmCategory = 'statistical' | 'machine_learning' | 'hybrid' | 'ensemble'
export type ComplexityLevel = 'low' | 'medium' | 'high' | 'very_high'

export interface DataRequirements {
  readonly min_draws: number
  readonly recommended_draws: number
  readonly max_draws?: number
  readonly data_types: readonly ('historical' | 'pattern' | 'trend')[]
}

export interface AlgorithmParameter {
  readonly key: string
  readonly label: string
  readonly description: string
  readonly type: ParameterType
  readonly min?: number
  readonly max?: number
  readonly step?: number
  readonly default_value: unknown
  readonly required: boolean
  readonly validation?: ParameterValidation
}

export type ParameterType = 'number' | 'boolean' | 'string' | 'array' | 'object'
export type ParameterValidation = 'range' | 'pattern' | 'custom'

export interface AlgorithmPerformanceStats {
  readonly total_predictions: number
  readonly average_accuracy: number
  readonly best_accuracy: number
  readonly worst_accuracy: number
  readonly recent_performance: number
  readonly reliability_score: number
}

// =============================================================================
// Batch Prediction Types
// =============================================================================

export interface BatchPredictionRequest {
  readonly draw_number: number
  readonly algorithm_ids: readonly AlgorithmId[]
  readonly parameters?: PredictionParameters
  readonly include_consensus: boolean
  readonly consensus_threshold?: number  // Minimum agreement percentage
  readonly comparison_enabled?: boolean
  readonly validation_enabled?: boolean
}

export interface BatchPredictionResult {
  readonly request_id: string
  readonly draw_number: number
  readonly generated_at: string
  readonly total_algorithms: number
  readonly successful_predictions: number
  readonly failed_predictions: number
  readonly predictions: readonly PredictionResult[]
  readonly consensus_numbers?: ConsensusNumbers
  readonly processing_time_ms: number
  readonly errors?: readonly ErrorInfo[]
  readonly metadata: BatchMetadata
}

export interface BatchMetadata {
  readonly analysis_period_days: number
  readonly sample_size: number
  readonly data_quality: DataQuality
  readonly processing_info: ProcessingInfo
}

export interface DataQuality {
  readonly completeness_score: number
  readonly accuracy_score: number
  readonly freshness_score: number
  readonly overall_quality: 'excellent' | 'good' | 'fair' | 'poor'
}

export interface ProcessingInfo {
  readonly parallel_processing: boolean
  readonly cache_hit: boolean
  readonly optimization_applied: boolean
}

export interface ConsensusNumbers {
  readonly algorithm_count: number
  readonly agreement_percentage: number
  readonly recommended_numbers: PredictedNumbers
  readonly confidence_score: number
  readonly strength: ConsensusStrength
  readonly top_combinations: readonly PredictedNumbers[]
  readonly voting_details: readonly VotingDetail[]
  readonly confidence_intervals: ConfidenceInterval[]
}

export type ConsensusStrength = 'weak' | 'moderate' | 'strong' | 'very_strong'

export interface VotingDetail {
  readonly number: number
  readonly votes: number
  readonly vote_percentage: number
  readonly supporting_algorithms: readonly AlgorithmId[]
  readonly zone: 'FRONT' | 'BACK'
  readonly confidence_weight: number
}

export interface ConfidenceInterval {
  readonly number: number
  readonly lower_bound: number
  readonly upper_bound: number
  readonly confidence_level: number
}

// =============================================================================
// Comparison and Validation Types
// =============================================================================

export interface PredictionComparison {
  readonly draw_number: string
  readonly actual_numbers: SuperLottoDraw
  readonly predictions: readonly PredictionResult[]
  readonly consensus_numbers?: ConsensusNumbers
  readonly accuracy_scores: readonly AccuracyScore[]
  readonly best_algorithm: AlgorithmId
  readonly consensus_accuracy: number
  readonly comparison_date: string
  readonly statistical_significance: number
}

export interface AccuracyScore {
  readonly algorithm_id: AlgorithmId
  readonly algorithm_name: string
  readonly red_matches: readonly number[]
  readonly blue_match: boolean
  readonly total_matches: number
  readonly accuracy_percentage: number
  readonly confidence_vs_accuracy: number
  readonly performance_rating: PerformanceRating
  readonly statistical_significance: number
}

export type PerformanceRating = 'excellent' | 'good' | 'average' | 'poor' | 'very_poor'

// =============================================================================
// Unified Table Display Types
// =============================================================================

export interface UnifiedTableData {
  readonly draws: readonly UnifiedDrawEntry[]
  readonly total_count: number
  readonly limit: number
  readonly offset: number
  readonly has_predictions: boolean
  readonly algorithms: readonly AlgorithmInfo[]
  readonly filters_applied: FilterInfo
  readonly sort_info: SortInfo
}

export interface UnifiedDrawEntry {
  readonly draw_info: SuperLottoDraw
  readonly predictions?: readonly AlgorithmPrediction[]
  readonly consensus?: ConsensusNumbers
  readonly best_prediction?: PredictionResult
  readonly actual_accuracy?: number
  readonly validation_status: ValidationStatus
  readonly performance_rating?: PerformanceRating
}

export type ValidationStatus = 'pending' | 'validated' | 'failed' | 'not_applicable'

export interface AlgorithmPrediction {
  readonly algorithm_id: AlgorithmId
  readonly algorithm_name: string
  readonly predicted: PredictedNumbers
  readonly confidence: number
  readonly accuracy?: number
  readonly matches?: MatchResult
  readonly performance: PredictionPerformance
}

export interface PredictionPerformance {
  readonly relative_ranking: number
  readonly confidence_accuracy_ratio: number
  readonly consistency_score: number
  readonly risk_adjusted_score: number
}

export interface MatchResult {
  readonly red_matches: readonly number[]
  readonly red_match_count: number
  readonly blue_match: boolean
  readonly total_matches: number
  readonly prize_won?: PrizeTier
}

export type PrizeTier = 'first' | 'second' | 'third' | 'fourth' | 'fifth' | 'none'

export interface AlgorithmInfo {
  readonly id: AlgorithmId
  readonly name: string
  readonly description: string
  readonly average_accuracy: number
  readonly total_predictions: number
  readonly success_rate: number
  readonly reliability_score: number
  readonly last_updated: string
  readonly trend: 'improving' | 'stable' | 'declining'
}

export interface FilterInfo {
  readonly date_range?: DateRange
  readonly algorithm_ids?: readonly AlgorithmId[]
  readonly confidence_range?: [number, number]
  readonly accuracy_range?: [number, number]
  readonly validation_status?: readonly ValidationStatus[]
}

export interface DateRange {
  readonly start_date: string
  readonly end_date: string
}

export interface SortInfo {
  readonly field: SortField
  readonly direction: 'asc' | 'desc'
}

export type SortField =
  | 'draw_date'
  | 'accuracy'
  | 'confidence'
  | 'algorithm'
  | 'performance'

// =============================================================================
// Export and Import Types
// =============================================================================

export interface ExportRequest {
  readonly format: ExportFormat
  readonly data_type: ExportDataType
  readonly draw_range?: DrawRange
  readonly algorithm_ids?: readonly AlgorithmId[]
  readonly include_consensus: boolean
  readonly date_range?: DateRange
  readonly filename?: string
  readonly options: ExportOptions
}

export type ExportFormat = 'csv' | 'json' | 'excel' | 'pdf'
export type ExportDataType = 'predictions' | 'history' | 'comparison' | 'unified_table' | 'analytics'

export interface DrawRange {
  readonly start_draw: number
  readonly end_draw: number
}

export interface ExportOptions {
  readonly include_headers: boolean
  readonly include_metadata: boolean
  readonly compression: boolean
  readonly date_format: string
  readonly number_format: string
  readonly locale: string
}

export interface ExportResult {
  readonly success: boolean
  readonly filename: string
  readonly format: string
  readonly size_bytes: number
  readonly records_count: number
  readonly download_url?: string
  readonly error?: ErrorInfo
  readonly export_time_ms: number
}

export interface ImportRequest {
  readonly format: ImportFormat
  readonly data_type: ImportDataType
  readonly file_path: string
  readonly validation_options: ValidationOptions
  readonly mapping_options?: MappingOptions
}

export type ImportFormat = 'csv' | 'json' | 'excel' | 'xml'
export type ImportDataType = 'historical_draws' | 'predictions' | 'algorithm_config'

export interface ValidationOptions {
  readonly strict_validation: boolean
  readonly duplicate_check: boolean
  readonly range_validation: boolean
  readonly format_validation: boolean
  readonly business_rules_validation: boolean
}

export interface MappingOptions {
  readonly field_mapping: Record<string, string>
  readonly date_format: string
  readonly number_format: string
  readonly encoding: string
}

// =============================================================================
// API and System Types
// =============================================================================

export interface ApiResponse<T> {
  readonly success: boolean
  readonly data?: T
  readonly error?: ErrorInfo
  readonly metadata?: ResponseMetadata
}

export interface ResponseMetadata {
  readonly request_id: string
  readonly timestamp: string
  readonly processing_time_ms: number
  readonly cache_hit: boolean
  readonly rate_limit_remaining?: number
}

export interface ErrorInfo {
  readonly code: string
  readonly message: string
  readonly details?: Record<string, unknown>
  readonly timestamp: string
  readonly request_id?: string
  readonly severity: ErrorSeverity
  readonly recoverable: boolean
  readonly suggestions?: readonly string[]
}

export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical'

export interface PaginationParams {
  readonly page?: number
  readonly limit?: number
  readonly offset?: number
  readonly sort_by?: string
  readonly sort_direction?: 'asc' | 'desc'
}

export interface SearchParams extends PaginationParams {
  readonly query?: string
  readonly filters?: Record<string, unknown>
  readonly date_range?: DateRange
  readonly algorithm_ids?: readonly AlgorithmId[]
}

// =============================================================================
// UI State Types
// =============================================================================

export interface LoadingState {
  readonly loading: boolean
  readonly loading_text?: string
  readonly progress?: number
  readonly cancellable?: boolean
}

export interface ErrorState {
  readonly has_error: boolean
  readonly error_message?: string
  readonly error_code?: string
  readonly retry_count: number
  readonly can_retry: boolean
}

export interface SelectionState<T> {
  readonly selected_items: readonly T[]
  readonly selected_ids: readonly string[]
  readonly last_selected?: T
  readonly selection_mode: 'single' | 'multiple'
}

export interface FilterState {
  readonly active_filters: readonly FilterInfo[]
  readonly saved_filters: readonly SavedFilter[]
  readonly current_preset?: string
}

export interface SavedFilter {
  readonly id: string
  readonly name: string
  readonly description: string
  readonly filters: readonly FilterInfo[]
  readonly created_at: string
  readonly is_default: boolean
}

// =============================================================================
// Configuration and Constants
// =============================================================================

export const ALGORITHM_CONFIGS: readonly AlgorithmConfig[] = [
  {
    id: 'frequency_analysis',
    name: 'Frequency Analysis',
    description: 'Analyzes historical frequency of numbers',
    enabled: true,
    category: 'statistical',
    complexity: 'low',
    data_requirements: {
      min_draws: 50,
      recommended_draws: 100,
      max_draws: 1000,
      data_types: ['historical']
    },
    parameters: [
      {
        key: 'draw_count',
        label: 'Draw Count',
        description: 'Number of historical draws to analyze',
        type: 'number',
        min: 10,
        max: 1000,
        default_value: 100,
        required: true
      },
      {
        key: 'weight_factor',
        label: 'Weight Factor',
        description: 'Weight factor for frequency calculations',
        type: 'number',
        min: 0.1,
        max: 2.0,
        step: 0.1,
        default_value: 1.0,
        required: true
      }
    ],
    default_parameters: {
      draw_count: 100,
      weight_factor: 1.0
    },
    performance_stats: {
      total_predictions: 0,
      average_accuracy: 0,
      best_accuracy: 0,
      worst_accuracy: 0,
      recent_performance: 0,
      reliability_score: 0
    },
    version: '1.0.0'
  },
  // ... other algorithm configs would be defined here
] as const

export const VALIDATION_RULES = {
  SUPER_LOTTO: {
    RED_NUMBERS: {
      count: 5,
      range: [1, 35] as const,
      unique: true
    },
    BLUE_NUMBER: {
      count: 1,
      range: [1, 12] as const,
      unique: true
    }
  },
  CONFIDENCE: {
    range: [0, 1] as const,
    precision: 3
  },
  ACCURACY: {
    range: [0, 100] as const,
    precision: 1
  }
} as const

// =============================================================================
// Utility Types and Helper Functions
// =============================================================================

export type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P]
}

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>

export type Callback<T = void> = (data: T) => void
export type AsyncCallback<T = void> = (data: T) => Promise<void>

// Type guards
export const isValidSuperLottoDraw = (draw: unknown): draw is SuperLottoDraw => {
  if (!draw || typeof draw !== 'object') return false
  const d = draw as SuperLottoDraw
  return (
    typeof d.id === 'number' &&
    typeof d.draw_number === 'number' &&
    typeof d.draw_date === 'string' &&
    Array.isArray(d.red_numbers) &&
    d.red_numbers.length === 5 &&
    d.red_numbers.every(n => typeof n === 'number' && n >= 1 && n <= 35) &&
    typeof d.blue_number === 'number' &&
    d.blue_number >= 1 &&
    d.blue_number <= 12
  )
}

export const isValidPredictionResult = (prediction: unknown): prediction is PredictionResult => {
  if (!prediction || typeof prediction !== 'object') return false
  const p = prediction as PredictionResult
  return (
    typeof p.id === 'string' &&
    typeof p.algorithm_id === 'string' &&
    typeof p.confidence_score === 'number' &&
    p.confidence_score >= 0 &&
    p.confidence_score <= 1 &&
    typeof p.created_at === 'string'
  )
}

// Utility functions
export const createErrorResponse = (
  code: string,
  message: string,
  severity: ErrorSeverity = 'medium',
  recoverable: boolean = true,
  suggestions?: readonly string[]
): ErrorInfo => ({
  code,
  message,
  severity,
  recoverable,
  suggestions,
  timestamp: new Date().toISOString()
})

export const createApiResponse = <T>(
  success: boolean,
  data?: T,
  error?: ErrorInfo,
  metadata?: Partial<ResponseMetadata>
): ApiResponse<T> => ({
  success,
  data,
  error,
  metadata: {
    request_id: crypto.randomUUID(),
    timestamp: new Date().toISOString(),
    processing_time_ms: 0,
    cache_hit: false,
    ...metadata
  }
})

console.log('🎯 [Enhanced Types] Comprehensive type definitions loaded successfully')