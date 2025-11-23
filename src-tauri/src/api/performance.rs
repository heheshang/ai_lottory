//! Performance-optimized API layer for Tauri commands
//!
//! Implements request batching, compression, and intelligent caching
//! for improved API performance and reduced data transfer.

use crate::database::performance::DatabasePerformanceManager;
use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::errors::SuperLottoResult as Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Performance-optimized API manager
pub struct ApiPerformanceManager {
    db_performance: Arc<DatabasePerformanceManager>,
    request_cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    batch_queue: Arc<RwLock<Vec<BatchRequest>>>,
    rate_limiter: Arc<Semaphore>,
    metrics: Arc<RwLock<ApiMetrics>>,
}

/// Cached API response
#[derive(Debug, Clone)]
struct CachedResponse {
    data: serde_json::Value,
    timestamp: Instant,
    ttl: Duration,
    etag: Option<String>,
    compressed: bool,
}

/// Batch request for bulk operations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchRequest {
    id: String,
    command: String,
    parameters: serde_json::Value,
    timestamp: DateTime<Utc>,
}

/// Batch response containing multiple results
#[derive(Debug, Serialize)]
struct BatchResponse {
    request_id: String,
    results: Vec<BatchResult>,
    total_time_ms: u64,
    success_count: usize,
    error_count: usize,
}

#[derive(Debug, Serialize)]
struct BatchResult {
    request_id: String,
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
    execution_time_ms: u64,
}

/// API performance metrics
#[derive(Debug, Default)]
struct ApiMetrics {
    total_requests: u64,
    cache_hits: u64,
    cache_misses: u64,
    batch_requests: u64,
    average_response_time: Duration,
    compression_ratio: f64,
    slow_requests: Vec<SlowRequest>,
}

#[derive(Debug, Clone)]
struct SlowRequest {
    endpoint: String,
    duration: Duration,
    timestamp: DateTime<Utc>,
    parameters: Option<String>,
}

impl ApiPerformanceManager {
    pub fn new(db_performance: Arc<DatabasePerformanceManager>) -> Self {
        Self {
            db_performance,
            request_cache: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(Semaphore::new(100)), // Limit to 100 concurrent requests
            metrics: Arc::new(RwLock::new(ApiMetrics::default())),
        }
    }

    /// Execute a command with performance optimizations
    pub async fn execute_command<T>(
        &self,
        command: &str,
        parameters: serde_json::Value,
        options: ExecutionOptions,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
    {
        // Rate limiting
        let _permit = self.rate_limiter.acquire().await.map_err(|_| {
            SuperLottoError::internal("Rate limit exceeded")
        })?;

        let start_time = Instant::now();

        // Check cache first
        if options.cache_ttl > Duration::ZERO {
            let cache_key = self.generate_cache_key(command, &parameters, &options);

            if let Some(cached) = self.get_from_cache(&cache_key).await {
                self.update_metrics_cache_hit().await;
                return serde_json::from_value(cached.data)
                    .map_err(|e| SuperLottoError::internal(format!("Cache deserialization error: {}", e)));
            }
        }

        self.update_metrics_cache_miss().await;

        // Execute the command
        let result = self.execute_command_internal::<T>(command, parameters, &options).await?;

        // Cache successful result
        if options.cache_ttl > Duration::ZERO {
            let cache_key = self.generate_cache_key(command, &parameters, &options);
            self.cache_response(&cache_key, &result, options.cache_ttl).await?;
        }

        // Update metrics
        let duration = start_time.elapsed();
        self.update_request_metrics(command, duration).await;

        Ok(result)
    }

    /// Execute batch commands for improved performance
    pub async fn execute_batch(
        &self,
        requests: Vec<BatchRequest>,
        options: BatchExecutionOptions,
    ) -> Result<BatchResponse> {
        let start_time = Instant::now();
        let batch_id = uuid::Uuid::new_v4().to_string();

        tracing::info!("🚀 Starting batch execution of {} requests", requests.len());

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        // Process requests in parallel batches
        let batch_size = options.max_concurrent_requests.min(requests.len());
        let chunks: Vec<_> = requests.chunks(batch_size).collect();

        for chunk in chunks {
            let chunk_results = futures::future::join_all(
                chunk.iter().map(|request| {
                    self.execute_single_batch_request(request, &options)
                })
            ).await;

            for result in chunk_results {
                match result {
                    Ok(batch_result) => {
                        if batch_result.success {
                            success_count += 1;
                        } else {
                            error_count += 1;
                        }
                        results.push(batch_result);
                    }
                    Err(e) => {
                        error_count += 1;
                        results.push(BatchResult {
                            request_id: uuid::Uuid::new_v4().to_string(),
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                            execution_time_ms: 0,
                        });
                    }
                }
            }
        }

        let total_time = start_time.elapsed();

        let response = BatchResponse {
            request_id: batch_id,
            results,
            total_time_ms: total_time.as_millis() as u64,
            success_count,
            error_count,
        };

        tracing::info!(
            "✅ Batch execution completed: {} success, {} errors in {}ms",
            success_count,
            error_count,
            total_time.as_millis()
        );

        Ok(response)
    }

    /// Compress response data to reduce transfer size
    pub fn compress_response(&self, data: &serde_json::Value) -> Result<Vec<u8>> {
        // Serialize to JSON
        let json_str = serde_json::to_string(data)
            .map_err(|e| SuperLottoError::internal(format!("JSON serialization error: {}", e)))?;

        // Use simple compression (you can implement proper compression here)
        // For now, just return the JSON bytes
        // In production, you'd use libraries like flate2 or lz4
        Ok(json_str.into_bytes())
    }

    /// Decompress response data
    pub fn decompress_response(&self, compressed: &[u8]) -> Result<serde_json::Value> {
        // Simple decompression (you'd implement proper decompression here)
        let json_str = String::from_utf8(compressed.to_vec())
            .map_err(|e| SuperLottoError::internal(format!("Decompression error: {}", e)))?;

        serde_json::from_str(&json_str)
            .map_err(|e| SuperLottoError::internal(format!("JSON deserialization error: {}", e)))
    }

    /// Get API performance metrics
    pub async fn get_api_metrics(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let cache = self.request_cache.read().await;

        serde_json::json!({
            "performance": {
                "total_requests": metrics.total_requests,
                "cache_hit_rate": if metrics.cache_hits + metrics.cache_misses > 0 {
                    (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
                } else { 0.0 },
                "average_response_time_ms": metrics.average_response_time.as_millis(),
                "batch_requests_processed": metrics.batch_requests,
                "compression_ratio": metrics.compression_ratio,
                "cached_responses": cache.len()
            },
            "slow_requests": metrics.slow_requests.iter().rev().take(10).collect::<Vec<_>>(),
            "collected_at": Utc::now().to_rfc3339()
        })
    }

    /// Clear API cache
    pub async fn clear_cache(&self, pattern: Option<&str>) -> Result<()> {
        let mut cache = self.request_cache.write().await;

        if let Some(pattern) = pattern {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| SuperLottoError::internal(format!("Invalid regex pattern: {}", e)))?;

            cache.retain(|key, _| !regex.is_match(key));
        } else {
            cache.clear();
        }

        tracing::info!("✅ API cache cleared");
        Ok(())
    }

    /// Optimize API performance based on metrics
    pub async fn optimize_performance(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let cache = self.request_cache.read().await;

        let mut recommendations = Vec::new();

        // Analyze cache hit rate
        let cache_hit_rate = if metrics.cache_hits + metrics.cache_misses > 0 {
            (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
        } else { 0.0 };

        if cache_hit_rate < 30.0 {
            recommendations.push("Increase cache TTL for frequently accessed endpoints".to_string());
        }

        // Analyze response time
        if metrics.average_response_time > Duration::from_millis(1000) {
            recommendations.push("Optimize slow database queries or increase request timeout".to_string());
        }

        // Analyze cache size
        if cache.len() > 1000 {
            recommendations.push("Implement cache size limits and LRU eviction policy".to_string());
        }

        // Analyze slow requests
        if metrics.slow_requests.len() > 5 {
            recommendations.push("Review slow requests and implement specific optimizations".to_string());
        }

        serde_json::json!({
            "optimization_analysis": {
                "cache_hit_rate": cache_hit_rate,
                "average_response_time_ms": metrics.average_response_time.as_millis(),
                "cache_size": cache.len(),
                "slow_request_count": metrics.slow_requests.len(),
                "performance_score": self.calculate_performance_score(&metrics)
            },
            "recommendations": recommendations,
            "optimization_applied": false,
            "analyzed_at": Utc::now().to_rfc3339()
        })
    }

    // Private helper methods

    async fn execute_command_internal<T>(
        &self,
        command: &str,
        parameters: serde_json::Value,
        options: &ExecutionOptions,
    ) -> Result<T> {
        match command {
            "get_super_lotto_draws" => {
                let result = self.db_performance
                    .get_super_lotto_draws_optimized(
                        parameters["limit"].as_i64().unwrap_or(100),
                        parameters["offset"].as_i64().unwrap_or(0),
                        parameters["start_date"].as_str().map(|s| s.to_string()),
                        parameters["end_date"].as_str().map(|s| s.to_string()),
                        parameters["draw_number"].as_str().map(|s| s.to_string()),
                    )
                    .await?;

                serde_json::from_value(result)
                    .map_err(|e| SuperLottoError::internal(format!("Response deserialization error: {}", e)))
            },
            "analyze_hot_numbers" => {
                let days = parameters["days"].as_u64().unwrap_or(90) as u32;
                let result = self.db_performance.compute_number_frequencies(days).await?;

                serde_json::from_value(result)
                    .map_err(|e| SuperLottoError::internal(format!("Response deserialization error: {}", e)))
            },
            // Add other command handlers here...
            _ => Err(SuperLottoError::internal(format!("Unknown command: {}", command)))
        }
    }

    async fn execute_single_batch_request(
        &self,
        request: &BatchRequest,
        options: &BatchExecutionOptions,
    ) -> Result<BatchResult> {
        let start_time = Instant::now();

        let result = self.execute_command_internal::<serde_json::Value>(
            &request.command,
            request.parameters.clone(),
            &options.default_options,
        ).await;

        let execution_time = start_time.elapsed();

        match result {
            Ok(data) => Ok(BatchResult {
                request_id: request.id.clone(),
                success: true,
                data: Some(data),
                error: None,
                execution_time_ms: execution_time.as_millis() as u64,
            }),
            Err(e) => Ok(BatchResult {
                request_id: request.id.clone(),
                success: false,
                data: None,
                error: Some(e.to_string()),
                execution_time_ms: execution_time.as_millis() as u64,
            }),
        }
    }

    fn generate_cache_key(
        &self,
        command: &str,
        parameters: &serde_json::Value,
        options: &ExecutionOptions,
    ) -> String {
        let mut key = format!("{}:", command);

        // Add relevant parameters to key
        if let Some(fields) = &options.cache_key_fields {
            for field in fields {
                if let Some(value) = parameters.get(field) {
                    key.push_str(&format!("{}:{},", field, value));
                }
            }
        } else {
            key.push_str(&parameters.to_string());
        }

        // Add user context if applicable
        if let Some(user_id) = &options.user_context {
            key.push_str(&format!("user:{}", user_id));
        }

        // Create hash for consistent key
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    async fn get_from_cache(&self, key: &str) -> Option<CachedResponse> {
        let cache = self.request_cache.read().await;
        cache.get(key).and_then(|cached| {
            if cached.timestamp.elapsed() < cached.ttl {
                Some(cached.clone())
            } else {
                None
            }
        })
    }

    async fn cache_response<T>(
        &self,
        key: String,
        data: &T,
        ttl: Duration,
    ) -> Result<()>
    where
        T: serde::Serialize,
    {
        let serialized = serde_json::to_value(data)
            .map_err(|e| SuperLottoError::internal(format!("Cache serialization error: {}", e)))?;

        let cached = CachedResponse {
            data: serialized,
            timestamp: Instant::now(),
            ttl,
            etag: None,
            compressed: false,
        };

        let mut cache = self.request_cache.write().await;
        cache.insert(key, cached);

        // Implement cache size limits
        if cache.len() > 1000 {
            // Remove oldest entries
            let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
            entries.sort_by_key(|&(_, timestamp)| timestamp);

            for (key, _) in entries.iter().take(100) {
                cache.remove(key);
            }
        }

        Ok(())
    }

    async fn update_metrics_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    async fn update_metrics_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    async fn update_request_metrics(&self, endpoint: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        // Update average response time
        let total_time = metrics.average_response_time * (metrics.total_requests - 1) as u32 + duration;
        metrics.average_response_time = total_time / metrics.total_requests as u32;

        // Track slow requests
        if duration > Duration::from_millis(2000) {
            metrics.slow_requests.push(SlowRequest {
                endpoint: endpoint.to_string(),
                duration,
                timestamp: Utc::now(),
                parameters: None,
            });

            // Keep only last 50 slow requests
            if metrics.slow_requests.len() > 50 {
                metrics.slow_requests.remove(0);
            }
        }
    }

    fn calculate_performance_score(&self, metrics: &ApiMetrics) -> f64 {
        let cache_score = if metrics.cache_hits + metrics.cache_misses > 0 {
            (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
        } else { 0.0 };

        let speed_score = (1.0 / (1.0 + metrics.average_response_time.as_millis() as f64 / 1000.0)) * 100.0;

        (cache_score + speed_score) / 2.0
    }
}

/// Configuration for command execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub cache_ttl: Duration,
    pub cache_key_fields: Option<Vec<String>>,
    pub user_context: Option<String>,
    pub compress_response: bool,
    pub timeout: Option<Duration>,
}

/// Configuration for batch execution
#[derive(Debug, Clone)]
pub struct BatchExecutionOptions {
    pub max_concurrent_requests: usize,
    pub default_options: ExecutionOptions,
}

impl Default for BatchExecutionOptions {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            default_options: ExecutionOptions::default(),
        }
    }
}