//! Simple in-memory caching for Super Lotto application

use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::errors::SuperLottoResult as Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Simple cache manager for basic in-memory caching
pub struct CacheManager {
    cache: Arc<RwLock<HashMap<String, CachedEntry>>>,
    max_size: usize,
    metrics: Arc<RwLock<CacheMetrics>>,
}

#[derive(Debug, Clone)]
struct CachedEntry {
    data: serde_json::Value,
    timestamp: Instant,
    ttl: Duration,
}

/// Basic cache metrics
#[derive(Debug, Default, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub total_requests: u64,
}

impl CacheManager {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size: max_size_mb * 1024 * 1024, // Convert to bytes
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get value from cache
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get(key) {
            if entry.timestamp.elapsed() < entry.ttl {
                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.hits += 1;
                metrics.total_requests += 1;

                return serde_json::from_value(entry.data.clone())
                    .map_err(|e| SuperLottoError::internal(format!("Cache deserialization error: {e}")))
                    .map(Some);
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }

        // Update metrics for miss
        let mut metrics = self.metrics.write().await;
        metrics.misses += 1;
        metrics.total_requests += 1;

        Ok(None)
    }

    /// Put value into cache
    pub async fn put<T>(&self, key: String, value: &T, ttl: Duration) -> Result<()>
    where
        T: serde::Serialize + Send + Sync + 'static,
    {
        let serialized = serde_json::to_value(value)
            .map_err(|e| SuperLottoError::internal(format!("Cache serialization error: {e}")))?;

        let mut cache = self.cache.write().await;

        // Simple size management - remove oldest entries if cache is full
        if cache.len() >= 1000 { // Limit number of entries
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(key, CachedEntry {
            data: serialized,
            timestamp: Instant::now(),
            ttl,
        });

        Ok(())
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    /// Clear cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache hit rate
    pub async fn hit_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.total_requests == 0 {
            0.0
        } else {
            metrics.hits as f64 / metrics.total_requests as f64 * 100.0
        }
    }
}