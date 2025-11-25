//! Base cache manager trait and implementations

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache key generator trait
pub trait CacheKey {
    fn generate_key(&self) -> String;
    fn hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.generate_key().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Base cache manager trait
#[async_trait]
pub trait CacheManager: Send + Sync {
    /// Get value from cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Put value into cache with TTL
    async fn put<T>(&self, key: String, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Remove value from cache
    async fn remove(&self, key: &str) -> Result<bool>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Check if cache contains key
    async fn contains(&self, key: &str) -> Result<bool>;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub evictions: u64,
    pub total_size_bytes: u64,
    pub entry_count: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            (self.hits as f64 / (self.hits + self.misses) as f64) * 100.0
        }
    }

    pub fn eviction_rate(&self) -> f64 {
        if self.sets == 0 {
            0.0
        } else {
            (self.evictions as f64 / self.sets as f64) * 100.0
        }
    }
}

/// Cache invalidation strategies
#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    /// TTL-based expiration
    TimeToLive(Duration),
    /// LRU (Least Recently Used)
    LeastRecentlyUsed,
    /// LFU (Least Frequently Used)
    LeastFrequentlyUsed,
    /// Manual invalidation
    Manual,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size_bytes: u64,
    pub max_entries: u64,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub invalidation_strategy: InvalidationStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            max_entries: 10000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            invalidation_strategy: InvalidationStrategy::TimeToLive(Duration::from_secs(3600)),
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: std::time::Instant,
    pub last_accessed: std::time::Instant,
    pub ttl: Option<Duration>,
    pub access_count: u64,
    pub size_bytes: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>, size_bytes: u64) -> Self {
        let now = std::time::Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            ttl,
            access_count: 0,
            size_bytes,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = std::time::Instant::now();
        self.access_count += 1;
    }
}

/// Multi-level cache manager that combines L1, L2, and L3 caching
pub struct MultiLevelCacheManager {
    l1_cache: crate::cache::memory_cache::MemoryCache,
    l2_cache: crate::cache::disk_cache::DiskCache,
    config: CacheConfig,
    stats: std::sync::Arc<std::sync::Mutex<CacheStats>>,
}

impl MultiLevelCacheManager {
    pub fn new(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            l1_cache: crate::cache::memory_cache::MemoryCache::new(config.max_entries)?,
            l2_cache: crate::cache::disk_cache::DiskCache::new(&config)?,
            config,
            stats: std::sync::Arc::new(std::sync::Mutex::new(CacheStats::default())),
        })
    }

    /// Get value from cache, checking L1 first, then L2
    pub async fn get_multi_level<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        // Try L1 cache first
        match self.l1_cache.get(key).await? {
            Some(value) => {
                self.update_stats(true, false);
                return Ok(Some(value));
            }
            None => {
                // Try L2 cache
                match self.l2_cache.get(key).await? {
                    Some(value) => {
                        // Promote to L1 cache
                        let ttl = Duration::from_secs(300); // 5 minutes for L1
                        self.l1_cache.put(key.to_string(), &value, ttl).await?;
                        self.update_stats(false, true);
                        return Ok(Some(value));
                    }
                    None => {
                        self.update_stats(false, false);
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// Put value into cache (both L1 and L2)
    pub async fn put_multi_level<T>(&self, key: String, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        // Store in L1 with shorter TTL
        self.l1_cache.put(key.clone(), value, Duration::from_secs(300)).await?;

        // Store in L2 with longer TTL
        self.l2_cache.put(key, value, self.config.default_ttl).await?;

        self.update_stats_set();
        Ok(())
    }

    fn update_stats(&self, hit: bool, miss: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            if hit {
                stats.hits += 1;
            }
            if miss {
                stats.misses += 1;
            }
        }
    }

    fn update_stats_set(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.sets += 1;
        }
    }
}

#[async_trait]
impl CacheManager for MultiLevelCacheManager {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        self.get_multi_level(key).await
    }

    async fn put<T>(&self, key: String, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        // Store in both levels
        self.l1_cache.put(key.clone(), value, Duration::min(ttl, Duration::from_secs(300))).await?;
        self.l2_cache.put(key, value, ttl).await?;
        self.update_stats_set();
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<bool> {
        let l1_removed = self.l1_cache.remove(key).await?;
        let l2_removed = self.l2_cache.remove(key).await?;
        Ok(l1_removed || l2_removed)
    }

    async fn clear(&self) -> Result<()> {
        self.l1_cache.clear().await?;
        self.l2_cache.clear().await?;
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            CacheStats::default()
        }
    }

    async fn contains(&self, key: &str) -> Result<bool> {
        // Check L1 first, then L2
        if self.l1_cache.contains(key).await? {
            Ok(true)
        } else {
            self.l2_cache.contains(key).await
        }
    }
}

/// Cache warming utilities
pub struct CacheWarmer;

impl CacheWarmer {
    /// Warm cache with commonly accessed data
    pub async fn warm_common_data<T, F>(
        cache: &dyn CacheManager,
        keys: Vec<String>,
        loader: F,
        ttl: Duration,
    ) -> Result<()>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
        F: Fn(&str) -> Result<T>,
    {
        for key in keys {
            // Check if already cached
            if !cache.contains(&key).await? {
                // Load and cache the data
                match loader(&key) {
                    Ok(value) => {
                        cache.put(key, &value, ttl).await?;
                        tracing::debug!("Warmed cache entry: {}", key);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to warm cache entry {}: {}", key, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Preload cache with bulk data
    pub async fn bulk_preload<T, F>(
        cache: &dyn CacheManager,
        loader: F,
        ttl: Duration,
    ) -> Result<()>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
        F: FnOnce() -> Result<Vec<(String, T)>>,
    {
        let items = loader()?;

        for (key, value) in items {
            cache.put(key, &value, ttl).await?;
        }

        tracing::info!("Bulk preloaded {} cache entries", items.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_stats() {
        let mut stats = CacheStats::default();

        stats.hits = 80;
        stats.misses = 20;
        stats.sets = 100;
        stats.evictions = 10;

        assert_eq!(stats.hit_rate(), 80.0);
        assert_eq!(stats.eviction_rate(), 10.0);
    }

    #[tokio::test]
    async fn test_cache_entry() {
        let entry = CacheEntry::new(
            "test_value",
            Some(Duration::from_secs(60)),
            100
        );

        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);

        let mut entry = entry;
        entry.touch();
        assert_eq!(entry.access_count, 1);
    }
}