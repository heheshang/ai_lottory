import { CacheManager } from '../utils/cache'

export function useCache(cachePrefix: string) {
  const cache = new CacheManager(cachePrefix, {
    defaultTTL: 5 * 60 * 1000, // 5 minutes
    maxSize: 100,
    persistToStorage: true,
    compress: true,
    priority: 'medium'
  })

  const get = <T = any>(key: string) => cache.get<T>(key)
  const set = <T = any>(key: string, value: T, options?: any) => cache.set(key, value, options)
  const has = (key: string) => cache.has(key)
  const deleteKey = (key: string) => cache.delete(key)
  const clear = () => cache.clear()
  const clearPattern = (pattern: string | RegExp) => cache.clearPattern(pattern)
  const getStats = () => cache.getStats()

  return {
    get,
    set,
    has,
    delete: deleteKey,
    clear,
    clearPattern,
    getStats
  }
}