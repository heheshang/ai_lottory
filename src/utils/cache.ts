/**
 * Cache Manager - Smart caching utility for API responses and computed results
 *
 * Features:
 * - Multi-tier storage (memory + localStorage)
 * - TTL (Time To Live) support
 * - Cache invalidation strategies
 * - Performance monitoring
 * - Compression for large objects
 * - Dependency management
 */

export interface CacheOptions {
  ttl?: number // Time to live in milliseconds
  maxSize?: number // Maximum cache size
  persistToStorage?: boolean // Whether to persist to localStorage
  compress?: boolean // Whether to compress data
  priority?: 'low' | 'medium' | 'high' | 'critical'
  tags?: string[] // Tags for cache invalidation
  dependencies?: string[] // Cache dependencies
}

export interface CacheEntry<T = any> {
  key: string
  value: T
  timestamp: number
  expiresAt: number
  accessCount: number
  lastAccessed: number
  size: number
  compressed: boolean
  priority: CacheOptions['priority']
  tags: string[]
  dependencies: string[]
}

export interface CacheStats {
  totalEntries: number
  totalSize: number
  hitCount: number
  missCount: number
  hitRate: number
  evictionCount: number
  averageAccessTime: number
  memoryUsage: number
  storageUsage: number
}

export type InvalidationStrategy = 'ttl' | 'lru' | 'lfu' | 'size' | 'priority' | 'tag' | 'dependency'

/**
 * Cache Manager Class
 */
export class CacheManager {
  private memoryCache: Map<string, CacheEntry> = new Map()
  private storagePrefix: string
  private maxSize: number
  private defaultTTL: number
  private stats: CacheStats
  private accessTimes: Map<string, number[]> = new Map()
  private compressionThreshold: number = 1024 // 1KB

  constructor(storagePrefix: string = 'app_cache', options: Partial<CacheOptions> = {}) {
    this.storagePrefix = storagePrefix
    this.maxSize = options.maxSize || 100
    this.defaultTTL = options.ttl || 5 * 60 * 1000 // 5 minutes default

    this.stats = {
      totalEntries: 0,
      totalSize: 0,
      hitCount: 0,
      missCount: 0,
      hitRate: 0,
      evictionCount: 0,
      averageAccessTime: 0,
      memoryUsage: 0,
      storageUsage: 0
    }

    this.loadFromStorage()
    this.startMaintenanceTimer()
  }

  // =============================================================================
  // Core Cache Operations
  // =============================================================================

  /**
   * Set a value in cache
   */
  set<T = any>(key: string, value: T, options: CacheOptions = {}): boolean {
    const startTime = Date.now()

    try {
      const ttl = options.ttl || this.defaultTTL
      const now = Date.now()
      const expiresAt = now + ttl
      const serialized = JSON.stringify(value)
      const shouldCompress = options.compress !== false && serialized.length > this.compressionThreshold

      const processedValue = shouldCompress ? this.compress(serialized) : serialized
      const entry: CacheEntry<T> = {
        key,
        value: shouldCompress ? this.decompress(processedValue) : value,
        timestamp: now,
        expiresAt,
        accessCount: 0,
        lastAccessed: now,
        size: processedValue.length,
        compressed: shouldCompress,
        priority: options.priority || 'medium',
        tags: options.tags || [],
        dependencies: options.dependencies || []
      }

      // Check if we need to evict entries
      if (this.memoryCache.size >= this.maxSize && !this.memoryCache.has(key)) {
        this.evictEntries('size')
      }

      // Store in memory
      this.memoryCache.set(key, entry)

      // Persist to storage if enabled
      if (options.persistToStorage !== false) {
        this.saveToStorage(key, entry)
      }

      // Update stats
      this.updateStats(entry.size, false)
      this.recordAccessTime(key, Date.now() - startTime)

      return true
    } catch (error) {
      console.error(`Cache set failed for key ${key}:`, error)
      return false
    }
  }

  /**
   * Get a value from cache
   */
  get<T = any>(key: string): T | null {
    const startTime = Date.now()

    try {
      // Check memory cache first
      let entry = this.memoryCache.get(key)

      // Try storage if not in memory
      if (!entry) {
        entry = this.loadFromStorageEntry(key)
        if (entry) {
          this.memoryCache.set(key, entry)
        }
      }

      // Check if entry exists and is not expired
      if (!entry || Date.now() > entry.expiresAt) {
        if (entry) {
          this.delete(key) // Clean up expired entry
        }
        this.stats.missCount++
        this.recordAccessTime(key, Date.now() - startTime)
        this.updateHitRate()
        return null
      }

      // Update access information
      entry.accessCount++
      entry.lastAccessed = Date.now()

      this.stats.hitCount++
      this.recordAccessTime(key, Date.now() - startTime)
      this.updateHitRate()

      return entry.value
    } catch (error) {
      console.error(`Cache get failed for key ${key}:`, error)
      this.stats.missCount++
      this.recordAccessTime(key, Date.now() - startTime)
      this.updateHitRate()
      return null
    }
  }

  /**
   * Check if key exists and is not expired
   */
  has(key: string): boolean {
    const entry = this.memoryCache.get(key) || this.loadFromStorageEntry(key)
    return entry !== undefined && Date.now() <= entry.expiresAt
  }

  /**
   * Delete a cache entry
   */
  delete(key: string): boolean {
    try {
      const deletedFromMemory = this.memoryCache.delete(key)
      const deletedFromStorage = this.deleteFromStorage(key)

      if (deletedFromMemory || deletedFromStorage) {
        this.stats.totalEntries--
        this.updateStats()
        return true
      }
      return false
    } catch (error) {
      console.error(`Cache delete failed for key ${key}:`, error)
      return false
    }
  }

  /**
   * Clear all cache entries
   */
  clear(): void {
    try {
      // Clear memory
      this.memoryCache.clear()
      this.accessTimes.clear()

      // Clear storage
      if (typeof localStorage !== 'undefined') {
        const keys = Object.keys(localStorage).filter(key =>
          key.startsWith(this.storagePrefix + '_')
        )
        keys.forEach(key => localStorage.removeItem(key))
      }

      // Reset stats
      this.resetStats()
    } catch (error) {
      console.error('Cache clear failed:', error)
    }
  }

  /**
   * Clear entries matching pattern
   */
  clearPattern(pattern: string | RegExp): void {
    const regex = typeof pattern === 'string' ? new RegExp(pattern) : pattern

    for (const [key] of this.memoryCache) {
      if (regex.test(key)) {
        this.delete(key)
      }
    }

    // Also clear from storage
    if (typeof localStorage !== 'undefined') {
      const keys = Object.keys(localStorage).filter(key =>
        key.startsWith(this.storagePrefix + '_') && regex.test(key)
      )
      keys.forEach(key => localStorage.removeItem(key))
    }
  }

  // =============================================================================
  // Cache Invalidation Strategies
  // =============================================================================

  /**
   * Evict entries based on strategy
   */
  evictEntries(strategy: InvalidationStrategy, limit?: number): number {
    const entries = Array.from(this.memoryCache.values())
    let evicted = 0
    const targetLimit = limit || Math.max(1, Math.floor(this.maxSize * 0.1))

    let sortedEntries: CacheEntry[] = []

    switch (strategy) {
      case 'ttl':
        // Remove expired entries
        const now = Date.now()
        sortedEntries = entries.filter(entry => now > entry.expiresAt)
        break

      case 'lru':
        // Sort by last accessed (oldest first)
        sortedEntries = entries.sort((a, b) => a.lastAccessed - b.lastAccessed)
        break

      case 'lfu':
        // Sort by access count (least frequent first)
        sortedEntries = entries.sort((a, b) => a.accessCount - b.accessCount)
        break

      case 'size':
        // Sort by size (largest first)
        sortedEntries = entries.sort((a, b) => b.size - a.size)
        break

      case 'priority':
        // Sort by priority (low priority first)
        const priorityOrder = { low: 0, medium: 1, high: 2, critical: 3 }
        sortedEntries = entries.sort((a, b) =>
          (priorityOrder[a.priority || 'medium'] || 1) - (priorityOrder[b.priority || 'medium'] || 1)
        )
        break
    }

    // Evict entries
    for (const entry of sortedEntries.slice(0, targetLimit)) {
      this.delete(entry.key)
      evicted++
    }

    this.stats.evictionCount += evicted
    return evicted
  }

  /**
   * Invalidate entries by tag
   */
  invalidateByTag(tag: string): number {
    const entries = Array.from(this.memoryCache.values())
    const toInvalidate = entries.filter(entry => entry.tags.includes(tag))

    toInvalidate.forEach(entry => this.delete(entry.key))
    return toInvalidate.length
  }

  /**
   * Invalidate entries by dependency
   */
  invalidateByDependency(dependency: string): number {
    const entries = Array.from(this.memoryCache.values())
    const toInvalidate = entries.filter(entry => entry.dependencies.includes(dependency))

    toInvalidate.forEach(entry => this.delete(entry.key))
    return toInvalidate.length
  }

  // =============================================================================
  // Cache Statistics and Monitoring
  // =============================================================================

  /**
   * Get cache statistics
   */
  getStats(): CacheStats {
    return { ...this.stats }
  }

  /**
   * Get detailed cache information
   */
  getDetailedInfo(): {
    entries: CacheEntry[]
    memoryUsage: number
    storageUsage: number
    hitRateByPriority: Record<string, number>
    averageAccessTime: number
  } {
    const entries = Array.from(this.memoryCache.values())
    const memoryUsage = this.calculateMemoryUsage()
    const storageUsage = this.calculateStorageUsage()

    // Calculate hit rate by priority
    const hitRateByPriority: Record<string, number> = {}
    const priorityGroups: Record<string, { hits: number; total: number }> = {}

    entries.forEach(entry => {
      const priority = entry.priority || 'medium'
      if (!priorityGroups[priority]) {
        priorityGroups[priority] = { hits: 0, total: 0 }
      }
      priorityGroups[priority].total += entry.accessCount
      if (entry.accessCount > 0) {
        priorityGroups[priority].hits += 1
      }
    })

    Object.entries(priorityGroups).forEach(([priority, data]) => {
      hitRateByPriority[priority] = data.total > 0 ? data.hits / data.total : 0
    })

    return {
      entries,
      memoryUsage,
      storageUsage,
      hitRateByPriority,
      averageAccessTime: this.stats.averageAccessTime
    }
  }

  /**
   * Reset cache statistics
   */
  resetStats(): void {
    this.stats = {
      totalEntries: this.memoryCache.size,
      totalSize: this.calculateTotalSize(),
      hitCount: 0,
      missCount: 0,
      hitRate: 0,
      evictionCount: 0,
      averageAccessTime: 0,
      memoryUsage: this.calculateMemoryUsage(),
      storageUsage: this.calculateStorageUsage()
    }
    this.accessTimes.clear()
  }

  // =============================================================================
  // Cache Optimization
  // =============================================================================

  /**
   * Optimize cache performance
   */
  optimize(): void {
    // Remove expired entries
    this.evictEntries('ttl')

    // If still over size limit, remove LRU entries
    if (this.memoryCache.size > this.maxSize) {
      this.evictEntries('lru')
    }

    // Compact access times history
    this.compactAccessTimes()

    // Update statistics
    this.updateStats()
  }

  /**
   * Warm cache with common entries
   */
  async warmCache(entries: Array<{ key: string; value: any; options?: CacheOptions }>): Promise<void> {
    const promises = entries.map(({ key, value, options }) =>
      new Promise<void>((resolve) => {
        setTimeout(() => {
          this.set(key, value, options)
          resolve()
        }, Math.random() * 100) // Random delay to prevent blocking
      })
    )

    await Promise.all(promises)
  }

  // =============================================================================
  // Private Methods
  // =============================================================================

  private updateStats(size = 0, isDelete = false): void {
    this.stats.totalEntries = this.memoryCache.size
    this.stats.totalSize = this.calculateTotalSize()
    this.stats.memoryUsage = this.calculateMemoryUsage()
    this.stats.storageUsage = this.calculateStorageUsage()

    if (!isDelete && size > 0) {
      this.stats.totalSize += size
    }
  }

  private updateHitRate(): void {
    const total = this.stats.hitCount + this.stats.missCount
    this.stats.hitRate = total > 0 ? this.stats.hitCount / total : 0
  }

  private recordAccessTime(key: string, time: number): void {
    if (!this.accessTimes.has(key)) {
      this.accessTimes.set(key, [])
    }

    const times = this.accessTimes.get(key)!
    times.push(time)

    // Keep only last 100 access times
    if (times.length > 100) {
      times.splice(0, times.length - 100)
    }

    // Update average access time
    const allTimes = Array.from(this.accessTimes.values()).flat()
    this.stats.averageAccessTime = allTimes.reduce((sum, time) => sum + time, 0) / allTimes.length
  }

  private compactAccessTimes(): void {
    for (const [key, times] of this.accessTimes) {
      if (times.length > 50) {
        // Keep only every other time for older entries
        this.accessTimes.set(key, times.filter((_, index) => index % 2 === 0))
      }
    }
  }

  private calculateTotalSize(): number {
    return Array.from(this.memoryCache.values()).reduce((sum, entry) => sum + entry.size, 0)
  }

  private calculateMemoryUsage(): number {
    // Rough estimation of memory usage
    return this.calculateTotalSize() + (this.memoryCache.size * 200) // Overhead per entry
  }

  private calculateStorageUsage(): number {
    if (typeof localStorage === 'undefined') return 0

    let size = 0
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key && key.startsWith(this.storagePrefix + '_')) {
        const value = localStorage.getItem(key)
        if (value) {
          size += key.length + value.length
        }
      }
    }
    return size
  }

  private compress(data: string): string {
    // Simple compression using LZ-string or similar
    // For now, just return the data (in real implementation, use compression library)
    return data
  }

  private decompress(compressedData: string): string {
    // Decompression logic
    // For now, just return the data (in real implementation, use compression library)
    return compressedData
  }

  private saveToStorage(key: string, entry: CacheEntry): void {
    if (typeof localStorage === 'undefined') return

    try {
      const storageKey = `${this.storagePrefix}_${key}`
      const dataToStore = {
        ...entry,
        // Don't store the actual value in localStorage for large objects
        value: entry.size > this.compressionThreshold ? undefined : entry.value
      }
      localStorage.setItem(storageKey, JSON.stringify(dataToStore))
    } catch (error) {
      console.error(`Failed to save cache entry to storage:`, error)
    }
  }

  private loadFromStorage(): void {
    if (typeof localStorage === 'undefined') return

    try {
      const keys = Object.keys(localStorage).filter(key =>
        key.startsWith(this.storagePrefix + '_')
      )

      for (const storageKey of keys) {
        const data = localStorage.getItem(storageKey)
        if (data) {
          const entry = JSON.parse(data) as CacheEntry
          // Only load metadata, not the actual value (lazy loading)
          this.memoryCache.set(entry.key, {
            ...entry,
            value: undefined // Will be loaded on demand
          })
        }
      }
    } catch (error) {
      console.error('Failed to load cache from storage:', error)
    }
  }

  private loadFromStorageEntry<T = any>(key: string): CacheEntry<T> | undefined {
    if (typeof localStorage === 'undefined') return undefined

    try {
      const storageKey = `${this.storagePrefix}_${key}`
      const data = localStorage.getItem(storageKey)
      if (!data) return undefined

      return JSON.parse(data) as CacheEntry<T>
    } catch (error) {
      console.error(`Failed to load cache entry from storage:`, error)
      return undefined
    }
  }

  private deleteFromStorage(key: string): boolean {
    if (typeof localStorage === 'undefined') return false

    try {
      const storageKey = `${this.storagePrefix}_${key}`
      localStorage.removeItem(storageKey)
      return true
    } catch (error) {
      console.error(`Failed to delete cache entry from storage:`, error)
      return false
    }
  }

  private startMaintenanceTimer(): void {
    // Run maintenance every 5 minutes
    setInterval(() => {
      this.optimize()
    }, 5 * 60 * 1000)
  }
}

// =============================================================================
// Factory Functions and Utilities
// =============================================================================

/**
 * Create a cache manager with default options
 */
export function createCacheManager(
  storagePrefix: string,
  options?: Partial<CacheOptions>
): CacheManager {
  return new CacheManager(storagePrefix, options)
}

/**
 * Create a cache manager for API responses
 */
export function createAPICacheManager(apiName: string): CacheManager {
  return new CacheManager(`api_${apiName}`, {
    defaultTTL: 10 * 60 * 1000, // 10 minutes
    maxSize: 50,
    persistToStorage: true,
    compress: true,
    priority: 'high'
  })
}

/**
 * Create a cache manager for computed results
 */
export function createComputedCacheManager(componentName: string): CacheManager {
  return new CacheManager(`computed_${componentName}`, {
    defaultTTL: 2 * 60 * 1000, // 2 minutes
    maxSize: 20,
    persistToStorage: false,
    compress: false,
    priority: 'medium'
  })
}

/**
 * Create a cache manager for user preferences
 */
export function createPreferencesCacheManager(): CacheManager {
  return new CacheManager('preferences', {
    defaultTTL: 24 * 60 * 60 * 1000, // 24 hours
    maxSize: 10,
    persistToStorage: true,
    compress: false,
    priority: 'critical'
  })
}

// =============================================================================
// Vue Composable Integration
// =============================================================================

/**
 * Vue composable for cache management
 */
export function useCache(storagePrefix: string, options?: Partial<CacheOptions>) {
  const cache = new CacheManager(storagePrefix, options)

  const get = <T = any>(key: string) => cache.get<T>(key)
  const set = <T = any>(key: string, value: T, cacheOptions?: CacheOptions) =>
    cache.set(key, value, cacheOptions)
  const has = (key: string) => cache.has(key)
  const del = (key: string) => cache.delete(key)
  const clear = () => cache.clear()
  const clearPattern = (pattern: string | RegExp) => cache.clearPattern(pattern)
  const getStats = () => cache.getStats()
  const optimize = () => cache.optimize()

  return {
    get,
    set,
    has,
    delete: del,
    clear,
    clearPattern,
    getStats,
    optimize
  }
}