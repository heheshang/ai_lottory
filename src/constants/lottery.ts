/**
 * Lottery-related constants
 */

// Number ranges
export const LOTTERY_RANGES = {
  FRONT_MIN: 1,
  FRONT_MAX: 35,
  FRONT_COUNT: 5,
  BACK_MIN: 1,
  BACK_MAX: 12,
  BACK_COUNT: 2
} as const

// Analysis periods (in days)
export const ANALYSIS_PERIODS = [
  { value: 30, label: '最近30天' },
  { value: 60, label: '最近60天' },
  { value: 90, label: '最近90天' },
  { value: 180, label: '最近180天' },
  { value: 365, label: '最近一年' }
] as const

// Confidence levels
export const CONFIDENCE_LEVELS = {
  VERY_HIGH: { min: 0.8, color: '#27ae60', label: '非常高' },
  HIGH: { min: 0.7, color: '#2ecc71', label: '高' },
  MEDIUM: { min: 0.6, color: '#f39c12', label: '中等' },
  LOW: { min: 0.5, color: '#e67e22', label: '较低' },
  VERY_LOW: { min: 0, color: '#e74c3c', label: '低' }
} as const

// Chart colors
export const CHART_COLORS = {
  PRIMARY: '#3498db',
  SUCCESS: '#27ae60',
  WARNING: '#f39c12',
  DANGER: '#e74c3c',
  INFO: '#9b59b6',
  FRONT_ZONE: ['#667eea', '#764ba2'],
  BACK_ZONE: ['#f093fb', '#f5576c'],
  GRADIENT: {
    BLUE: ['#667eea', '#764ba2'],
    PINK: ['#f093fb', '#f5576c'],
    GREEN: ['#11998e', '#38ef7d'],
    ORANGE: ['#fa709a', '#fee140']
  }
} as const

// Number display colors
export const NUMBER_COLORS = {
  HOT: '#e74c3c',
  COLD: '#3498db',
  NORMAL: '#95a5a6',
  SELECTED: '#27ae60'
} as const

// Status colors
export const STATUS_COLORS = {
  SUCCESS: '#27ae60',
  ERROR: '#e74c3c',
  WARNING: '#f39c12',
  INFO: '#3498db',
  PENDING: '#95a5a6'
} as const

// Date formats
export const DATE_FORMATS = {
  SHORT: 'YYYY-MM-DD',
  LONG: 'YYYY-MM-DD HH:mm:ss',
  DISPLAY: 'YYYY年MM月DD日',
  TIME: 'HH:mm:ss'
} as const

// Pagination
export const PAGINATION = {
  DEFAULT_PAGE_SIZE: 20,
  PAGE_SIZES: [10, 20, 50, 100],
  MAX_VISIBLE_PAGES: 7
} as const

// Cache durations (in milliseconds)
export const CACHE_DURATIONS = {
  SHORT: 5 * 60 * 1000,    // 5 minutes
  MEDIUM: 30 * 60 * 1000,  // 30 minutes
  LONG: 2 * 60 * 60 * 1000 // 2 hours
} as const

// API endpoints
export const API_ENDPOINTS = {
  DRAWS: '/api/draws',
  PREDICTIONS: '/api/predictions',
  STATISTICS: '/api/statistics',
  ANALYSIS: '/api/analysis',
  BATCH_PREDICTIONS: '/api/batch-predictions'
} as const

// Error messages
export const ERROR_MESSAGES = {
  NETWORK_ERROR: '网络连接失败，请检查您的网络设置',
  SERVER_ERROR: '服务器错误，请稍后重试',
  INVALID_DATA: '数据格式错误',
  PREDICTION_FAILED: '预测生成失败',
  LOAD_FAILED: '数据加载失败',
  SAVE_FAILED: '保存失败'
} as const

// Success messages
export const SUCCESS_MESSAGES = {
  PREDICTION_GENERATED: '预测生成成功',
  DATA_IMPORTED: '数据导入成功',
  SAVED: '保存成功',
  DELETED: '删除成功',
  UPDATED: '更新成功'
} as const

// Validation rules
export const VALIDATION_RULES = {
  FRONT_NUMBER_RANGE: {
    min: LOTTERY_RANGES.FRONT_MIN,
    max: LOTTERY_RANGES.FRONT_MAX
  },
  BACK_NUMBER_RANGE: {
    min: LOTTERY_RANGES.BACK_MIN,
    max: LOTTERY_RANGES.BACK_MAX
  },
  MIN_ANALYSIS_DAYS: 7,
  MAX_ANALYSIS_DAYS: 365,
  MIN_CONFIDENCE: 0,
  MAX_CONFIDENCE: 1
} as const
