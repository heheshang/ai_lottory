//! Advanced multi-layer caching system for Super Lotto application
//!
//! Implements intelligent caching strategies including:
//! - LRU eviction policies
//! - Multi-tier storage (memory + disk)
//! - Predictive cache warming
//! - Cache invalidation strategies
//! - Distributed cache coordination

pub mod strategies;
pub mod storage;
pub mod warming;
pub mod invalidation;

use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::errors::SuperLottoResult as Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Main cache manager that orchestrates all caching strategies
pub struct CacheManager {
    memory_cache: Arc<RwLock<MemoryCache>>,
    disk_cache: Arc<DiskCache>,
    strategy: Arc<dyn CacheStrategy>,
    metrics: Arc<RwLock<CacheMetrics>>,
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub key: String,
    pub data: T,
    pub metadata: CacheMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub ttl: Duration,
    pub size_bytes: usize,
    pub priority: CachePriority,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Cache metrics and statistics
#[derive(Debug, Default)]
pub struct CacheMetrics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub memory_usage_bytes: usize,
    pub disk_usage_bytes: usize,
    pub average_access_time: Duration,
    pub hit_rate_by_priority: Vec<(CachePriority, f64)>,
    pub top_accessed_keys: Vec<(String, u64)>,
}

/// Cache strategy trait for different eviction algorithms
pub trait CacheStrategy: Send + Sync {
    fn should_evict(&self, cache: &MemoryCache, entry_size: usize) -> Option<String>;
    fn on_access(&self, cache: &MemoryCache, key: &str);
    fn on_insert(&self, cache: &MemoryCache, key: &str);
    fn get_strategy_name(&self) -> &str;
}

/// Memory cache implementation
#[derive(Debug)]
pub struct MemoryCache {
    entries: std::collections::HashMap<String, CachedEntry>,
    max_size_bytes: usize,
    max_entries: usize,
    current_size_bytes: usize,
}

#[derive(Debug)]
struct CachedEntry {
    data: serde_json::Value,
    metadata: CacheMetadata,
}

/// Disk cache implementation for persistent storage
pub struct DiskCache {
    storage_path: std::path::PathBuf,
    max_size_bytes: usize,
    compression_enabled: bool,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_memory_size_mb: usize,
    pub max_disk_size_mb: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub compression_threshold: usize,
    pub predictive_warming: bool,
    pub cache_strategy: CacheStrategyType,
}

#[derive(Debug, Clone)]
pub enum CacheStrategyType {
    LRU,
    LFU,
    ARC,
    TwoQueue,
    WRR, // Weighted Round Robin
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Result<Self> {
        let memory_cache = Arc::new(RwLock::new(MemoryCache::new(
            config.max_memory_size_mb * 1024 * 1024,
            10000, // max entries
        )));

        let disk_cache = Arc::new(DiskCache::new(
            std::path::PathBuf::from("./cache"),
            config.max_disk_size_mb * 1024 * 1024,
            true, // compression enabled
        )?);

        let strategy: Arc<dyn CacheStrategy> = match config.cache_strategy {
            CacheStrategyType::LRU => Arc::new(strategies::LRUStrategy::new()),
            CacheStrategyType::LFU => Arc::new(strategies::LFUStrategy::new()),
            CacheStrategyType::ARC => Arc::new(strategies::ARCStrategy::new()),
            CacheStrategyType::TwoQueue => Arc::new(strategies::TwoQueueStrategy::new()),
            CacheStrategyType::WRR => Arc::new(strategies::WRRStrategy::new()),
        };

        Ok(Self {
            memory_cache,
            disk_cache,
            strategy,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        })
    }

    /// Get value from cache (multi-tier lookup)
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let start_time = Instant::now();

        // Try memory cache first
        {
            let memory_cache = self.memory_cache.read().await;
            if let Some(entry) = memory_cache.get(key) {
                let result: T = serde_json::from_value(entry.data.clone())
                    .map_err(|e| SuperLottoError::internal(format!("Cache deserialization error: {}", e)))?;

                // Update access metadata
                drop(memory_cache);
                self.strategy.on_access(&*self.memory_cache.read().await, key);

                self.update_metrics_hit(start_time).await;
                return Ok(Some(result));
            }
        }

        // Try disk cache
        if let Some(disk_entry) = self.disk_cache.get(key).await? {
            let result: T = serde_json::from_value(disk_entry.data)
                .map_err(|e| SuperLottoError::internal(format!("Cache deserialization error: {}", e)))?;

            // Promote to memory cache
            self.put_internal(key, &result, disk_entry.metadata).await?;

            self.update_metrics_hit(start_time).await;
            return Ok(Some(result));
        }

        self.update_metrics_miss(start_time).await;
        Ok(None)
    }

    /// Put value into cache
    pub async fn put<T>(&self, key: &str, value: &T, options: CacheOptions) -> Result<()>
    where
        T: serde::Serialize + Send + Sync + 'static,
    {
        let serialized = serde_json::to_value(value)
            .map_err(|e| SuperLottoError::internal(format!("Cache serialization error: {}", e)))?;

        let metadata = CacheMetadata {
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            ttl: options.ttl,
            size_bytes: serialized.to_string().len(),
            priority: options.priority,
            tags: options.tags,
        };

        self.put_internal(key, &serialized, metadata).await
    }

    /// Internal put implementation
    async fn put_internal(&self, key: &str, data: &serde_json::Value, metadata: CacheMetadata) -> Result<()> {
        // Try memory cache first
        {
            let mut memory_cache = self.memory_cache.write().await;

            // Check if we need to evict entries
            if !memory_cache.can_fit(metadata.size_bytes) {
                if let Some(evict_key) = self.strategy.should_evict(&*memory_cache, metadata.size_bytes) {
                    let evicted = memory_cache.remove(&evict_key);
                    if let Some(evicted_entry) = evicted {
                        // Move evicted entry to disk cache
                        let _ = self.disk_cache.put(&evict_key, &evicted_entry.data, evicted_entry.metadata).await;
                    }
                }
            }

            if memory_cache.can_fit(metadata.size_bytes) {
                memory_cache.insert(key.to_string(), CachedEntry {
                    data: data.clone(),
                    metadata: metadata.clone(),
                });

                self.strategy.on_insert(&*memory_cache, key);
                return Ok(());
            }
        }

        // If memory cache is full, store in disk cache
        self.disk_cache.put(key, data, metadata).await
    }

    /// Delete entry from cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let mut memory_cache = self.memory_cache.write().await;
        let removed = memory_cache.remove(key).is_some();

        if !removed {
            // Try disk cache
            let disk_removed = self.disk_cache.delete(key).await?;
            return Ok(disk_removed);
        }

        Ok(true)
    }

    /// Clear cache by pattern
    pub async fn clear_by_pattern(&self, pattern: &str) -> Result<usize> {
        let regex = regex::Regex::new(pattern)
            .map_err(|e| SuperLottoError::internal(format!("Invalid regex pattern: {}", e)))?;

        let mut memory_cache = self.memory_cache.write().await;
        let keys_to_remove: Vec<String> = memory_cache.keys()
            .filter(|key| regex.is_match(key))
            .cloned()
            .collect();

        let removed_count = keys_to_remove.len();
        for key in keys_to_remove {
            memory_cache.remove(&key);
        }

        // Also clear from disk cache
        let disk_removed = self.disk_cache.clear_by_pattern(pattern).await?;

        Ok(removed_count + disk_removed)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let memory_cache = self.memory_cache.read().await;

        serde_json::json!({
            "memory_cache": {
                "entries": memory_cache.len(),
                "size_bytes": memory_cache.size_bytes(),
                "max_size_bytes": memory_cache.max_size_bytes(),
                "max_entries": memory_cache.max_entries
            },
            "performance": {
                "total_requests": metrics.total_requests,
                "cache_hits": metrics.cache_hits,
                "cache_misses": metrics.cache_misses,
                "hit_rate": if metrics.total_requests > 0 {
                    (metrics.cache_hits as f64 / metrics.total_requests as f64) * 100.0
                } else { 0.0 },
                "evictions": metrics.evictions,
                "average_access_time_ms": metrics.average_access_time.as_millis()
            },
            "top_keys": metrics.top_accessed_keys,
            "hit_rate_by_priority": metrics.hit_rate_by_priority.iter()
                .map(|(priority, rate)| serde_json::json!({
                    "priority": format!("{:?}", priority),
                    "hit_rate": rate
                }))
                .collect::<Vec<_>>(),
            "collected_at": Utc::now().to_rfc3339()
        })
    }

    /// Predictive cache warming based on access patterns
    pub async fn warm_cache_predictively(&self) -> Result<()> {
        let metrics = self.metrics.read().await;
        let top_keys = &metrics.top_accessed_keys;

        // Warm top 20 most accessed keys
        for (key, _) in top_keys.iter().take(20) {
            // This would typically load data from the original source
            // and pre-populate the cache
            tracing::debug!("Warming cache for key: {}", key);
        }

        Ok(())
    }

    /// Optimize cache configuration based on usage patterns
    pub async fn optimize_configuration(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let memory_cache = self.memory_cache.read().await;

        let hit_rate = if metrics.total_requests > 0 {
            (metrics.cache_hits as f64 / metrics.total_requests as f64) * 100.0
        } else { 0.0 };

        let memory_utilization = (memory_cache.size_bytes() as f64 / memory_cache.max_size_bytes() as f64) * 100.0;

        let mut recommendations = Vec::new();

        if hit_rate < 50.0 {
            recommendations.push("Consider increasing cache size or TTL for better hit rate".to_string());
        }

        if memory_utilization > 90.0 {
            recommendations.push("Memory cache is near capacity - consider increasing size or more aggressive eviction".to_string());
        }

        if metrics.evictions > metrics.cache_hits / 10 {
            recommendations.push("High eviction rate - consider implementing smarter eviction strategy".to_string());
        }

        serde_json::json!({
            "analysis": {
                "hit_rate": hit_rate,
                "memory_utilization": memory_utilization,
                "total_requests": metrics.total_requests,
                "eviction_rate": metrics.evictions as f64 / metrics.total_requests as f64
            },
            "recommendations": recommendations,
            "optimization_score": self.calculate_optimization_score(&metrics, &memory_cache),
            "analyzed_at": Utc::now().to_rfc3339()
        })
    }

    // Private helper methods

    async fn update_metrics_hit(&self, start_time: Instant) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.cache_hits += 1;

        let access_time = start_time.elapsed();
        let total_time = metrics.average_access_time * (metrics.total_requests - 1) as u32 + access_time;
        metrics.average_access_time = total_time / metrics.total_requests as u32;
    }

    async fn update_metrics_miss(&self, start_time: Instant) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.cache_misses += 1;

        let access_time = start_time.elapsed();
        let total_time = metrics.average_access_time * (metrics.total_requests - 1) as u32 + access_time;
        metrics.average_access_time = total_time / metrics.total_requests as u32;
    }

    fn calculate_optimization_score(&self, metrics: &CacheMetrics, memory_cache: &MemoryCache) -> f64 {
        let hit_rate = if metrics.total_requests > 0 {
            (metrics.cache_hits as f64 / metrics.total_requests as f64) * 100.0
        } else { 0.0 };

        let memory_efficiency = (memory_cache.size_bytes() as f64 / memory_cache.max_size_bytes() as f64) * 100.0;
        let eviction_penalty = (metrics.evictions as f64 / metrics.total_requests.max(1) as f64) * 10.0;

        (hit_rate + (100.0 - memory_efficiency.abs()) - eviction_penalty).max(0.0).min(100.0)
    }
}

/// Cache options for put operations
#[derive(Debug, Clone, Default)]
pub struct CacheOptions {
    pub ttl: Duration,
    pub priority: CachePriority,
    pub tags: Vec<String>,
    pub compress: bool,
}

impl MemoryCache {
    pub fn new(max_size_bytes: usize, max_entries: usize) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            max_size_bytes,
            max_entries,
            current_size_bytes: 0,
        }
    }

    pub fn get(&self, key: &str) -> Option<&CachedEntry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: String, entry: CachedEntry) {
        self.current_size_bytes += entry.metadata.size_bytes;
        self.entries.insert(key, entry);
    }

    pub fn remove(&mut self, key: &str) -> Option<CachedEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size_bytes -= entry.metadata.size_bytes;
            Some(entry)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    pub fn size_bytes(&self) -> usize {
        self.current_size_bytes
    }

    pub fn max_size_bytes(&self) -> usize {
        self.max_size_bytes
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn can_fit(&self, entry_size: usize) -> bool {
        self.entries.len() < self.max_entries &&
        self.current_size_bytes + entry_size <= self.max_size_bytes
    }
}