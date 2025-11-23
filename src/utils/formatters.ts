/**
 * Shared formatting utilities for the application
 * Provides consistent date, currency, and number formatting across components
 */

/**
 * Format a date string to localized format
 * @param date - Date string or Date object
 * @param format - Format type: 'short' | 'long' | 'time'
 * @returns Formatted date string
 */
export function formatDate(
  date: string | Date,
  format: 'short' | 'long' | 'time' = 'short'
): string {
  try {
    const d = typeof date === 'string' ? new Date(date) : date
    
    switch (format) {
      case 'short':
        return d.toLocaleDateString('zh-CN')
      case 'long':
        return d.toLocaleString('zh-CN')
      case 'time':
        return d.toLocaleTimeString('zh-CN')
      default:
        return d.toLocaleDateString('zh-CN')
    }
  } catch {
    return typeof date === 'string' ? date : ''
  }
}

/**
 * Format currency amount
 * @param amount - Amount to format
 * @param currency - Currency symbol (default: '¥')
 * @returns Formatted currency string
 */
export function formatCurrency(
  amount: number | null | undefined,
  currency: string = '¥'
): string {
  if (amount === null || amount === undefined) return '-'
  return `${currency}${amount.toLocaleString('zh-CN')}`
}

/**
 * Format large numbers with unit suffixes (K, M, B)
 * @param num - Number to format
 * @param decimals - Number of decimal places
 * @returns Formatted number string
 */
export function formatNumber(
  num: number | null | undefined,
  decimals: number = 1
): string {
  if (num === null || num === undefined) return '-'
  
  const units = ['', 'K', 'M', 'B', 'T']
  const sign = num < 0 ? '-' : ''
  const absNum = Math.abs(num)
  
  if (absNum < 1000) {
    return `${sign}${absNum.toFixed(decimals)}`
  }
  
  const unitIndex = Math.floor(Math.log10(absNum) / 3)
  const scaledNum = absNum / Math.pow(1000, unitIndex)
  
  return `${sign}${scaledNum.toFixed(decimals)}${units[unitIndex]}`
}

/**
 * Format percentage
 * @param value - Value to format as percentage (0-1 or 0-100)
 * @param asDecimal - Whether input is decimal (0-1) or percentage (0-100)
 * @param decimals - Number of decimal places
 * @returns Formatted percentage string
 */
export function formatPercentage(
  value: number | null | undefined,
  asDecimal: boolean = true,
  decimals: number = 1
): string {
  if (value === null || value === undefined) return '-'
  
  const percentage = asDecimal ? value * 100 : value
  return `${percentage.toFixed(decimals)}%`
}

/**
 * Format draw number with leading zeros
 * @param drawNumber - Draw number
 * @param length - Desired length
 * @returns Padded draw number
 */
export function formatDrawNumber(
  drawNumber: string | number,
  length: number = 8
): string {
  const str = String(drawNumber)
  return str.padStart(length, '0')
}

/**
 * Format time ago from a date
 * @param date - Date to compare
 * @returns Human-readable time ago string
 */
export function formatTimeAgo(date: string | Date): string {
  const d = typeof date === 'string' ? new Date(date) : date
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffSec = Math.floor(diffMs / 1000)
  const diffMin = Math.floor(diffSec / 60)
  const diffHour = Math.floor(diffMin / 60)
  const diffDay = Math.floor(diffHour / 24)
  const diffMonth = Math.floor(diffDay / 30)
  const diffYear = Math.floor(diffDay / 365)

  if (diffYear > 0) return `${diffYear}年前`
  if (diffMonth > 0) return `${diffMonth}个月前`
  if (diffDay > 0) return `${diffDay}天前`
  if (diffHour > 0) return `${diffHour}小时前`
  if (diffMin > 0) return `${diffMin}分钟前`
  return '刚刚'
}

/**
 * Format duration in milliseconds to readable string
 * @param ms - Milliseconds
 * @returns Formatted duration string
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`
  if (ms < 3600000) return `${(ms / 60000).toFixed(2)}min`
  return `${(ms / 3600000).toFixed(2)}h`
}

/**
 * Truncate text with ellipsis
 * @param text - Text to truncate
 * @param maxLength - Maximum length
 * @param ellipsis - Ellipsis string
 * @returns Truncated text
 */
export function truncate(
  text: string,
  maxLength: number,
  ellipsis: string = '...'
): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength - ellipsis.length) + ellipsis
}

/**
 * Format file size in bytes to human-readable format
 * @param bytes - Size in bytes
 * @param decimals - Number of decimal places
 * @returns Formatted file size
 */
export function formatFileSize(
  bytes: number | null | undefined,
  decimals: number = 2
): string {
  if (bytes === null || bytes === undefined) return '-'
  if (bytes === 0) return '0 Bytes'

  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']

  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
}

/**
 * Format phone number (Chinese format)
 * @param phone - Phone number
 * @returns Formatted phone number
 */
export function formatPhone(phone: string): string {
  const cleaned = phone.replace(/\D/g, '')
  
  if (cleaned.length === 11) {
    // Mobile: 138 1234 5678
    return cleaned.replace(/(\d{3})(\d{4})(\d{4})/, '$1 $2 $3')
  } else if (cleaned.length === 10) {
    // Landline with area code: 010 1234 5678
    return cleaned.replace(/(\d{3})(\d{4})(\d{3})/, '$1 $2 $3')
  }
  
  return phone
}

/**
 * Format odds/even ratio
 * @param oddCount - Number of odd numbers
 * @param totalCount - Total count
 * @returns Formatted ratio string
 */
export function formatOddEvenRatio(oddCount: number, totalCount: number): string {
  const evenCount = totalCount - oddCount
  return `${oddCount}:${evenCount}`
}

/**
 * Capitalize first letter
 * @param text - Text to capitalize
 * @returns Capitalized text
 */
export function capitalize(text: string): string {
  if (!text) return ''
  return text.charAt(0).toUpperCase() + text.slice(1)
}

/**
 * Format confidence score with visual indicator
 * @param score - Confidence score (0-1)
 * @returns Object with formatted value and level
 */
export function formatConfidence(score: number): {
  formatted: string
  level: 'low' | 'medium' | 'high' | 'very-high'
  color: string
} {
  const percentage = (score * 100).toFixed(1)
  let level: 'low' | 'medium' | 'high' | 'very-high'
  let color: string

  if (score < 0.4) {
    level = 'low'
    color = '#e74c3c'
  } else if (score < 0.6) {
    level = 'medium'
    color = '#f39c12'
  } else if (score < 0.8) {
    level = 'high'
    color = '#3498db'
  } else {
    level = 'very-high'
    color = '#27ae60'
  }

  return {
    formatted: `${percentage}%`,
    level,
    color
  }
}
