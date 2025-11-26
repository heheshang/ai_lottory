// User types
export interface User {
  id: number
  username: string
  email?: string
  created_at: string
  last_login?: string
}

export interface UserLogin {
  username: string
  password: string
}

export interface UserRegistration {
  username: string
  email?: string
  password: string
}

// Lottery types
export interface LotteryDraw {
  id: number
  draw_date: string
  winning_numbers: number[]
  bonus_number?: number
  jackpot_amount?: number
  lottery_type: string
}

export interface NewLotteryDraw {
  draw_date: string
  winning_numbers: number[]
  bonus_number?: number
  jackpot_amount?: number
  lottery_type: string
}

export interface LotterySearchQuery {
  lottery_type?: string
  start_date?: string
  end_date?: string
  number_filter?: number[]
  limit?: number
  offset?: number
}

// Analysis types
export interface NumberFrequency {
  number: number
  frequency: number
  last_seen?: string
  hot_score: number
  cold_score: number
  zone?: string
}

export interface NumberStatistics {
  number: number
  total_draws: number
  frequency: number
  average_gap: number
  current_gap: number
  longest_gap: number
  shortest_gap: number
}

export interface AnalysisRequest {
  lottery_type: string
  days?: number
  draw_count?: number
}

export interface HotNumbersResponse {
  numbers: NumberFrequency[]
  analysis_period: string
  total_draws_analyzed: number
}

export interface ColdNumbersResponse {
  numbers: NumberFrequency[]
  analysis_period: string
  total_draws_analyzed: number
}

// API Response types
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// Common types
export type LotteryType = 'powerball' | 'megamillions' | 'lotto' | 'custom'

// Export ErrorCode from error handler for use across the application
export { ErrorCode, ErrorCategory } from '@/utils/errorHandler'

// Re-export from specific modules

// Reference Tauri types
/// <reference path="./tauri.d.ts" />
