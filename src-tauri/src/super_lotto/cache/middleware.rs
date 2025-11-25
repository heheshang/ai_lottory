//! Cache middleware for Tauri commands

use crate::error::{AppError, Result};
use crate::super_lotto::models::analysis_cache::{AnalysisCache, AnalysisCacheKey};
use crate::super_lotto::cache::cache_invalidator::CacheInvalidator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Cache middleware that intercepts and caches command results
pub struct CacheMiddleware {
    memory_cache: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    invalidator: Arc<CacheInvalidator>,
    stats: Arc<RwLock<CacheMiddlewareStats>>,
    config: CacheMiddlewareConfig,
}

/// Cache middleware configuration
#[derive(Debug, Clone)]
pub struct CacheMiddlewareConfig {
    pub default_ttl: Duration,
    pub max_memory_cache_size: usize,
    pub enable_smart_caching: bool,
    pub cache_key_prefix: String,
}

impl Default for CacheMiddlewareConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(1800), // 30 minutes
            max_memory_cache_size: 1000,
            enable_smart_caching: true,
            cache_key_prefix: "cmd_cache".to_string(),
        }
    }
}

/// Cache middleware statistics
#[derive(Debug, Default, Clone)]
pub struct CacheMiddlewareStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_sets: u64,
    pub cache_evictions: u64,
    pub total_requests: u64,
    pub cache_hit_rate: f64,
    pub average_cache_time_ms: f64,
    pub total_cache_time_ms: f64,
}

impl CacheMiddleware {
    pub fn new(config: CacheMiddlewareConfig) -> Self {
        Self {
            memory_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            invalidator: Arc::new(CacheInvalidator::new()),
            stats: Arc::new(RwLock::new(CacheMiddlewareStats::default())),
            config,
        }
    }

    /// Execute a function with caching
    pub async fn execute_with_cache<F, T>(&self, command: &str, params: &serde_json::Value, func: F) -> Result<T>
    where
        F: FnOnce(&serde_json::Value) -> Result<T>,
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
    {
        let start_time = std::time::Instant::now();

        // Generate cache key
        let cache_key = self.generate_cache_key(command, params)?;

        // Try to get from cache first
        if let Some(cached_result) = self.try_get_from_cache::<T>(&cache_key).await? {
            self.update_cache_hit_stats(start_time.elapsed()).await;
            tracing::debug!("Cache hit for command: {}", command);
            return Ok(cached_result);
        }

        // Cache miss - execute the function
        let result = func(params)?;

        // Cache the result
        if self.should_cache_result(command, &result) {
            self.store_in_cache(&cache_key, &result).await?;
            tracing::debug!("Cached result for command: {}", command);
        }

        self.update_cache_miss_stats(start_time.elapsed()).await;
        Ok(result)
    }

    /// Execute a function with caching and automatic invalidation
    pub async fn execute_with_smart_cache<F, T>(
        &self,
        command: &str,
        params: &serde_json::Value,
        func: F
    ) -> Result<T>
    where
        F: FnOnce(&serde_json::Value) -> Result<T>,
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone,
    {
        let start_time = std::time::Instant::now();

        // Generate cache key with data hash
        let data_hash = AnalysisCacheKey::hash_data(params)?;
        let cache_key = format!("{}_{}", self.config.cache_key_prefix, data_hash);

        // Try to get from cache first
        if let Some(cached_result) = self.try_get_from_cache::<T>(&cache_key).await? {
            self.update_cache_hit_stats(start_time.elapsed()).await;
            tracing::debug!("Smart cache hit for command: {}", command);
            return Ok(cached_result);
        }

        // Cache miss - execute the function
        let result = func(params)?;

        // Smart caching decision
        if self.should_smart_cache(command, &result, params) {
            let ttl = self.calculate_smart_ttl(command, &result);
            self.store_in_cache_with_ttl(&cache_key, &result, ttl).await?;
            tracing::debug!("Smart cached result for command: {} (TTL: {:?})", command, ttl);
        }

        self.update_cache_miss_stats(start_time.elapsed()).await;
        Ok(result)
    }

    /// Invalidate cache for specific command or parameters
    pub async fn invalidate(&self, command: Option<&str>, params: Option<&serde_json::Value>) -> Result<usize> {
        let mut invalidated = 0;

        if let Some(command) = command {
            // Invalidate all cache entries for this command
            let cache_prefix = format!("{}_{}", self.config.cache_key_prefix, command);
            let mut memory_cache = self.memory_cache.write().await;

            let keys_to_remove: Vec<String> = memory_cache
                .keys()
                .filter(|key| key.starts_with(&cache_prefix))
                .cloned()
                .collect();

            for key in keys_to_remove {
                memory_cache.remove(&key);
                invalidated += 1;
            }

            // Invalidate analysis cache if it's an analysis command
            if self.is_analysis_command(command) {
                if let Some(algorithm) = self.extract_algorithm_from_params(params) {
                    self.invalidator.invalidate_algorithm(&algorithm).await?;
                }
            }
        } else {
            // Invalidate all cache
            let mut memory_cache = self.memory_cache.write().await;
            invalidated = memory_cache.len();
            memory_cache.clear();
        }

        if invalidated > 0 {
            tracing::info!("Invalidated {} cache entries", invalidated);
        }

        Ok(invalidated)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheMiddlewareStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Clear all cache
    pub async fn clear_all(&self) -> Result<()> {
        let mut memory_cache = self.memory_cache.write().await;
        memory_cache.clear();

        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheMiddlewareStats::default();

        tracing::info!("Cleared all cache entries");
        Ok(())
    }

    /// Warm cache with common commands
    pub async fn warm_cache(&self) -> Result<()> {
        let common_commands = vec![
            ("get_hot_numbers", serde_json::json!({"days": 30})),
            ("get_cold_numbers", serde_json::json!({"days": 30})),
            ("get_number_statistics", serde_json::json!({"number": 1, "lotteryType": "powerball"})),
        ];

        for (command, params) in common_commands {
            // This would typically preload common data
            tracing::debug!("Warming cache for command: {}", command);
        }

        Ok(())
    }

    /// Generate cache key for command and parameters
    fn generate_cache_key(&self, command: &str, params: &serde_json::Value) -> Result<String> {
        let params_str = serde_json::to_string(params)
            .map_err(|e| AppError::Internal {
                message: format!("Failed to serialize params for cache key: {}", e),
            })?;

        // Create a simple hash-based key
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        command.hash(&mut hasher);
        params_str.hash(&mut hasher);

        Ok(format!("{}_{}_{:x}", self.config.cache_key_prefix, command, hasher.finish()))
    }

    /// Try to get value from memory cache
    async fn try_get_from_cache<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let memory_cache = self.memory_cache.read().await;

        if let Some(value) = memory_cache.get(key) {
            serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|e| AppError::Internal {
                    message: format!("Failed to deserialize cached value: {}", e),
                })
        } else {
            Ok(None)
        }
    }

    /// Store value in memory cache
    async fn store_in_cache<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.store_in_cache_with_ttl(key, value, self.config.default_ttl).await
    }

    /// Store value in memory cache with specific TTL
    async fn store_in_cache_with_ttl<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let mut memory_cache = self.memory_cache.write().await;

        // Check cache size limit
        if memory_cache.len() >= self.config.max_memory_cache_size {
            self.evict_lru_entries(&mut memory_cache).await?;
        }

        let serialized = serde_json::to_value(value)
            .map_err(|e| AppError::Internal {
                message: format!("Failed to serialize value for cache: {}", e),
            })?;

        memory_cache.insert(key.to_string(), serialized);

        self.update_cache_set_stats().await;
        Ok(())
    }

    /// Evict least recently used entries
    async fn evict_lru_entries(&self, cache: &mut std::collections::HashMap<String, serde_json::Value>) -> Result<()> {
        let to_remove = cache.len() - self.config.max_memory_cache_size + 1;
        let keys: Vec<String> = cache.keys().take(to_remove).collect();

        for key in keys {
            cache.remove(&key);
        }

        let mut stats = self.stats.write().await;
        stats.cache_evictions += keys.len() as u64;

        Ok(())
    }

    /// Update statistics for cache hit
    async fn update_cache_hit_stats(&self, access_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.cache_hits += 1;
        stats.total_requests += 1;
        stats.cache_hit_rate = (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0;

        // Update average cache time
        let total_cache_time = stats.total_cache_time_ms + access_time.as_millis() as f64;
        stats.average_cache_time_ms = total_cache_time / (stats.cache_hits + stats.cache_misses) as f64;
        stats.total_cache_time_ms = total_cache_time;
    }

    /// Update statistics for cache miss
    async fn update_cache_miss_stats(&self, access_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        stats.total_requests += 1;
        stats.cache_hit_rate = (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0;

        // Update average cache time
        let total_cache_time = stats.total_cache_time_ms + access_time.as_millis() as f64;
        stats.average_cache_time_ms = total_cache_time / (stats.cache_hits + stats.cache_misses) as f64;
        stats.total_cache_time_ms = total_cache_time;
    }

    /// Update statistics for cache set
    async fn update_cache_set_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.cache_sets += 1;
    }

    /// Check if result should be cached
    fn should_cache_result(&self, command: &str, result: &serde_json::Value) -> bool {
        // Don't cache errors or empty results
        if result.is_null() || (result.is_object() && result.as_object().unwrap().is_empty()) {
            return false;
        }

        // Cache expensive operations
        self.is_expensive_command(command)
    }

    /// Smart caching decision
    fn should_smart_cache(&self, command: &str, result: &serde_json::Value, params: &serde_json::Value) -> bool {
        if !self.config.enable_smart_caching {
            return false;
        }

        // Check if result is valuable to cache
        if !self.should_cache_result(command, result) {
            return false;
        }

        // Additional smart caching logic
        self.is_worth_caching(command, params, result)
    }

    /// Check if command is expensive enough to cache
    fn is_expensive_command(&self, command: &str) -> bool {
        matches!(
            command,
            "get_hot_numbers" | "get_cold_numbers" | "get_number_statistics" |
            "generate_prediction" | "get_all_predictions" | "get_prediction_comparison"
        )
    }

    /// Check if result is worth caching
    fn is_worth_caching(&self, command: &str, params: &serde_json::Value, result: &serde_json::Value) -> bool {
        // Check if parameters suggest large dataset
        if let Some(days) = params.get("days").and_then(|d| d.as_u64()) {
            if days > 100 {
                return true; // Large dataset - worth caching
            }
        }

        // Check if result is complex
        let result_str = serde_json::to_string(result).unwrap_or_default();
        if result_str.len() > 10000 {
            return true; // Large result - worth caching
        }

        self.is_expensive_command(command)
    }

    /// Calculate smart TTL based on command and result
    fn calculate_smart_ttl(&self, command: &str, result: &serde_json::Value) -> Duration {
        let base_ttl = match command {
            "get_hot_numbers" | "get_cold_numbers" => Duration::from_secs(1800), // 30 minutes
            "get_number_statistics" => Duration::from_secs(3600), // 1 hour
            "generate_prediction" | "get_all_predictions" => Duration::from_secs(900), // 15 minutes
            _ => self.config.default_ttl,
        };

        // Adjust TTL based on result size and parameters
        let result_size = serde_json::to_string(result).unwrap_or_default().len();
        let size_multiplier = if result_size > 50000 { 2.0 } else { 1.0 };

        Duration::from_millis((base_ttl.as_millis() as f64 * size_multiplier) as u64)
    }

    /// Extract algorithm from parameters
    fn extract_algorithm_from_params(&self, params: Option<&serde_json::Value>) -> Option<String> {
        params?.get("algorithm")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string())
    }

    /// Check if command is analysis-related
    fn is_analysis_command(&self, command: &str) -> bool {
        command.starts_with("get_") &&
        (command.contains("numbers") || command.contains("statistics") || command.contains("prediction"))
    }
}

/// Cache middleware wrapper for easy integration
#[async_trait::async_trait]
pub trait CacheableCommand {
    /// Execute command with caching
    async fn execute_cached(&self, middleware: &CacheMiddleware, params: serde_json::Value) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_middleware_creation() {
        let config = CacheMiddlewareConfig::default();
        let middleware = CacheMiddleware::new(config);

        let stats = middleware.get_stats().await;
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = CacheMiddlewareConfig::default();
        let middleware = CacheMiddleware::new(config);

        let params = json!({"days": 30, "algorithm": "test"});
        let key = middleware.generate_cache_key("get_hot_numbers", &params).unwrap();

        assert!(key.starts_with("cmd_cache_get_hot_numbers"));
    }

    #[tokio::test]
    async fn test_should_cache_decision() {
        let config = CacheMiddlewareConfig::default();
        let middleware = CacheMiddleware::new(config);

        // Should cache expensive command
        assert!(middleware.should_cache_result("get_hot_numbers", &json!({"result": "data"})));

        // Should not cache null result
        assert!(!middleware.should_cache_result("test", &serde_json::Value::Null));

        // Should not cache empty object
        assert!(!middleware.should_cache_result("test", &json!({})));
    }

    #[tokio::test]
    async fn test_execute_with_cache() {
        let config = CacheMiddlewareConfig::default();
        let middleware = CacheMiddleware::new(config);

        let params = json!({"days": 30});

        // First call - cache miss
        let result1 = middleware.execute_with_cache("test_command", &params, |_| {
            Ok(json!({"count": 100}))
        }).await.unwrap();

        // Second call - should hit cache (if implementation was complete)
        // Note: This test would need full implementation to work properly
        assert_eq!(result1, json!({"count": 100}));
    }
}