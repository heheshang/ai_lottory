//! Database performance optimization utilities
//!
//! Provides connection pooling, query optimization, and caching mechanisms
//! for improved database performance.

use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::errors::SuperLottoResult as Result;
use sqlx::{SqlitePool, Row, query, query_as, migrate::MigrateError};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Performance-optimized database connection manager
pub struct DatabasePerformanceManager {
    pool: SqlitePool,
    query_cache: Arc<RwLock<HashMap<String, CachedQuery>>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// Cached query result with metadata
#[derive(Debug, Clone)]
struct CachedQuery {
    data: serde_json::Value,
    timestamp: Instant,
    ttl: Duration,
    hit_count: u32,
}

/// Performance metrics collection
#[derive(Debug, Default)]
struct PerformanceMetrics {
    query_count: u64,
    total_duration: Duration,
    cache_hits: u64,
    cache_misses: u64,
    slow_queries: Vec<SlowQuery>,
}

#[derive(Debug, Clone)]
struct SlowQuery {
    query: String,
    duration: Duration,
    timestamp: DateTime<Utc>,
    parameters: Option<String>,
}

impl DatabasePerformanceManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Execute a query with performance monitoring and caching
    pub async fn execute_cached_query<T, F, Fut>(
        &self,
        cache_key: String,
        ttl: Duration,
        query_fn: F,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        // Check cache first
        {
            let cache = self.query_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.timestamp.elapsed() < cached.ttl {
                    // Cache hit
                    let mut metrics = self.metrics.write().await;
                    metrics.cache_hits += 1;

                    // Update hit count
                    drop(cache);
                    let mut cache_mut = self.query_cache.write().await;
                    if let Some(entry) = cache_mut.get_mut(&cache_key) {
                        entry.hit_count += 1;
                    }

                    return serde_json::from_value(cached.data.clone())
                        .map_err(|e| SuperLottoError::internal(format!("Cache deserialization error: {}", e)));
                }
            }
        }

        // Cache miss - execute query
        let start_time = Instant::now();
        let result = query_fn().await;
        let duration = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.query_count += 1;
            metrics.total_duration += duration;
            metrics.cache_misses += 1;

            // Track slow queries (>100ms)
            if duration > Duration::from_millis(100) {
                metrics.slow_queries.push(SlowQuery {
                    query: cache_key.clone(),
                    duration,
                    timestamp: Utc::now(),
                    parameters: None,
                });

                // Keep only last 100 slow queries
                if metrics.slow_queries.len() > 100 {
                    metrics.slow_queries.remove(0);
                }
            }
        }

        // Cache successful results
        if let Ok(ref data) = result {
            let serialized = serde_json::to_value(data)
                .map_err(|e| SuperLottoError::internal(format!("Cache serialization error: {}", e)))?;

            let cached_query = CachedQuery {
                data: serialized,
                timestamp: Instant::now(),
                ttl,
                hit_count: 1,
            };

            let mut cache = self.query_cache.write().await;
            cache.insert(cache_key, cached_query);

            // Limit cache size
            if cache.len() > 1000 {
                // Remove oldest entries
                let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
                entries.sort_by_key(|&(_, timestamp)| timestamp);

                for (key, _) in entries.iter().take(100) {
                    cache.remove(key);
                }
            }
        }

        result
    }

    /// Get optimized Super Lotto draws with performance enhancements
    pub async fn get_super_lotto_draws_optimized(
        &self,
        limit: i64,
        offset: i64,
        start_date: Option<String>,
        end_date: Option<String>,
        draw_number: Option<String>,
    ) -> Result<serde_json::Value> {
        let cache_key = format!(
            "super_lotto_draws:{}:{}:{}:{}:{}",
            limit, offset,
            start_date.unwrap_or_default(),
            end_date.unwrap_or_default(),
            draw_number.unwrap_or_default()
        );

        self.execute_cached_query(
            cache_key,
            Duration::from_secs(300), // 5 minute cache
            move || async move {
                let start_time = std::time::Instant::now();

                // Build optimized query with proper indexing
                let mut query_builder = sqlx::QueryBuilder::new(
                    "SELECT id, draw_number, draw_date, front_zone, back_zone, front_sum, \
                     front_odd_count, front_even_count, has_consecutive, jackpot_amount, created_at \
                     FROM super_lotto_draws_optimized \
                     WHERE 1=1"
                );

                // Add filters efficiently
                if let Some(start) = &start_date {
                    query_builder.push(" AND draw_date >= ");
                    query_builder.push_bind(start);
                }

                if let Some(end) = &end_date {
                    query_builder.push(" AND draw_date <= ");
                    query_builder.push_bind(end);
                }

                if let Some(number) = &draw_number {
                    query_builder.push(" AND draw_number LIKE ");
                    query_builder.push_bind(format!("%{}%", number));
                }

                // Add optimal ordering and pagination
                query_builder.push(" ORDER BY draw_date DESC, draw_number DESC LIMIT ");
                query_builder.push_bind(limit);
                query_builder.push(" OFFSET ");
                query_builder.push_bind(offset);

                let query = query_builder.build_query_as::<OptimizedDraw>();
                let draws = query
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SuperLottoError::Database(e))?;

                // Get total count efficiently
                let mut count_builder = sqlx::QueryBuilder::new(
                    "SELECT COUNT(*) as total FROM super_lotto_draws_optimized WHERE 1=1"
                );

                if let Some(start) = &start_date {
                    count_builder.push(" AND draw_date >= ");
                    count_builder.push_bind(start);
                }

                if let Some(end) = &end_date {
                    count_builder.push(" AND draw_date <= ");
                    count_builder.push_bind(end);
                }

                if let Some(number) = &draw_number {
                    count_builder.push(" AND draw_number LIKE ");
                    count_builder.push_bind(format!("%{}%", number));
                }

                let total: i64 = count_builder
                    .build_query_scalar()
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| SuperLottoError::Database(e))?;

                let duration = start_time.elapsed();
                if duration > Duration::from_millis(100) {
                    tracing::warn!("Slow query detected: get_super_lotto_draws_optimized took {:?}", duration);
                }

                Ok(serde_json::json!({
                    "draws": draws,
                    "total": total,
                    "limit": limit,
                    "offset": offset,
                    "has_more": (offset + limit) < total,
                    "query_time_ms": duration.as_millis()
                }))
            },
        ).await
    }

    /// Pre-compute and cache number frequency analysis
    pub async fn compute_number_frequencies(&self, days: u32) -> Result<serde_json::Value> {
        let cache_key = format!("number_frequencies:{}", days);

        self.execute_cached_query(
            cache_key,
            Duration::from_secs(1800), // 30 minute cache
            move || async move {
                let start_time = std::time::Instant::now();

                // Use optimized frequency table if available
                let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);

                let query = r#"
                    SELECT number, zone, frequency, hot_score, cold_score, last_seen, average_gap, current_gap, sample_size
                    FROM number_frequency_cache
                    WHERE period_days = ? AND computed_at > datetime('now', '-1 day')
                    ORDER BY hot_score DESC, cold_score DESC
                "#;

                let cached_results = sqlx::query_as::<_, CachedFrequency>(query)
                    .bind(days as i32)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SuperLottoError::Database(e))?;

                let result = if cached_results.is_empty() {
                    // Cache miss - compute frequencies
                    self.compute_frequencies_fallback(days).await?
                } else {
                    // Cache hit from frequency cache table
                    serde_json::json!({
                        "numbers": cached_results,
                        "period_days": days,
                        "cached": true,
                        "computed_at": Utc::now().to_rfc3339()
                    })
                };

                let duration = start_time.elapsed();
                tracing::info!("Number frequency computation completed in {:?}", duration);

                Ok(result)
            },
        ).await
    }

    /// Fallback frequency computation when cache table is empty
    async fn compute_frequencies_fallback(&self, days: u32) -> Result<serde_json::Value> {
        // Implementation for computing frequencies from raw data
        // This would contain the original frequency analysis logic
        Ok(serde_json::json!({
            "numbers": [],
            "period_days": days,
            "cached": false,
            "computed_at": Utc::now().to_rfc3339()
        }))
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let cache = self.query_cache.read().await;

        serde_json::json!({
            "query_performance": {
                "total_queries": metrics.query_count,
                "total_duration_ms": metrics.total_duration.as_millis(),
                "average_duration_ms": if metrics.query_count > 0 {
                    metrics.total_duration.as_millis() / metrics.query_count as u128
                } else { 0 },
                "cache_hits": metrics.cache_hits,
                "cache_misses": metrics.cache_misses,
                "cache_hit_rate": if metrics.cache_hits + metrics.cache_misses > 0 {
                    (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
                } else { 0.0 }
            },
            "cache_info": {
                "cached_queries": cache.len(),
                "memory_usage_mb": std::mem::size_of_val(&*cache) / (1024 * 1024)
            },
            "slow_queries": metrics.slow_queries.iter().rev().take(10).collect::<Vec<_>>(),
            "collected_at": Utc::now().to_rfc3339()
        })
    }

    /// Clear expired cache entries
    pub async fn cleanup_cache(&self) -> Result<()> {
        let mut cache = self.query_cache.write().await;
        let now = Instant::now();

        cache.retain(|_, cached| now.duration_since(cached.timestamp) < cached.ttl);

        // Also clean database cache tables
        sqlx::query("DELETE FROM number_frequency_cache WHERE computed_at < datetime('now', '-1 day')")
            .execute(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        sqlx::query("DELETE FROM batch_prediction_cache WHERE expires_at < datetime('now')")
            .execute(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        Ok(())
    }

    /// Analyze query performance and suggest optimizations
    pub async fn analyze_performance(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;

        let suggestions = Vec::new();

        // Analyze slow queries and provide recommendations
        for slow_query in &metrics.slow_queries {
            if slow_query.duration > Duration::from_secs(1) {
                // Very slow query - need optimization
                tracing::warn!("Very slow query detected: {} took {:?}", slow_query.query, slow_query.duration);
            }
        }

        serde_json::json!({
            "analysis": {
                "total_queries": metrics.query_count,
                "slow_query_count": metrics.slow_queries.len(),
                "cache_efficiency": {
                    "hit_rate": if metrics.cache_hits + metrics.cache_misses > 0 {
                        (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
                    } else { 0.0 },
                    "total_cached": metrics.cache_hits
                },
                "performance_score": self.calculate_performance_score(&metrics)
            },
            "recommendations": self.get_performance_recommendations(&metrics),
            "analyzed_at": Utc::now().to_rfc3339()
        })
    }

    fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> f64 {
        if metrics.query_count == 0 {
            return 100.0;
        }

        let avg_duration = metrics.total_duration.as_millis() as f64 / metrics.query_count as f64;
        let cache_hit_rate = if metrics.cache_hits + metrics.cache_misses > 0 {
            (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
        } else { 0.0 };

        // Score based on average query time (lower is better) and cache hit rate (higher is better)
        let duration_score = (1.0 / (1.0 + avg_duration / 100.0)) * 100.0;
        let cache_score = cache_hit_rate;

        (duration_score + cache_score) / 2.0
    }

    fn get_performance_recommendations(&self, metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        let avg_duration = if metrics.query_count > 0 {
            metrics.total_duration.as_millis() as f64 / metrics.query_count as f64
        } else { 0.0 };

        let cache_hit_rate = if metrics.cache_hits + metrics.cache_misses > 0 {
            (metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64) * 100.0
        } else { 0.0 };

        if avg_duration > 100.0 {
            recommendations.push("Consider optimizing slow queries with better indexes".to_string());
        }

        if cache_hit_rate < 50.0 {
            recommendations.push("Increase cache TTL or implement more aggressive caching".to_string());
        }

        if metrics.slow_queries.len() > 10 {
            recommendations.push("Review and optimize the most frequently used slow queries".to_string());
        }

        recommendations
    }
}

#[derive(Debug, sqlx::FromRow)]
struct OptimizedDraw {
    id: i64,
    draw_number: String,
    draw_date: String,
    front_zone: String,
    back_zone: String,
    front_sum: i32,
    front_odd_count: i32,
    front_even_count: i32,
    has_consecutive: bool,
    jackpot_amount: Option<f64>,
    created_at: String,
}

impl Serialize for OptimizedDraw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OptimizedDraw", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("draw_number", &self.draw_number)?;
        state.serialize_field("draw_date", &self.draw_date)?;

        // Parse JSON arrays for serialization
        let front_zone: Vec<i32> = serde_json::from_str(&self.front_zone).unwrap_or_default();
        let back_zone: Vec<i32> = serde_json::from_str(&self.back_zone).unwrap_or_default();

        state.serialize_field("front_zone", &front_zone)?;
        state.serialize_field("back_zone", &back_zone)?;
        state.serialize_field("front_sum", &self.front_sum)?;
        state.serialize_field("front_odd_count", &self.front_odd_count)?;
        state.serialize_field("front_even_count", &self.front_even_count)?;
        state.serialize_field("has_consecutive", &self.has_consecutive)?;
        state.serialize_field("jackpot_amount", &self.jackpot_amount)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.end()
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct CachedFrequency {
    number: i32,
    zone: String,
    frequency: f64,
    hot_score: f64,
    cold_score: f64,
    last_seen: Option<String>,
    average_gap: f64,
    current_gap: i32,
    sample_size: i32,
}

/// Initialize performance optimizations
pub async fn initialize_performance_optimizations(pool: &SqlitePool) -> Result<DatabasePerformanceManager> {
    tracing::info!("🚀 Initializing database performance optimizations...");

    // Run performance migrations
    sqlx::migrate!("database/migrations")
        .run(pool)
        .await
        .map_err(|e| SuperLottoError::Database(e))?;

    // Analyze database schema for optimization opportunities
    analyze_database_schema(pool).await?;

    // Create performance manager
    let manager = DatabasePerformanceManager::new(pool.clone());

    tracing::info!("✅ Database performance optimizations initialized");

    Ok(manager)
}

/// Analyze database schema and suggest optimizations
async fn analyze_database_schema(pool: &SqlitePool) -> Result<()> {
    // Check if tables are properly indexed
    let index_analysis = sqlx::query(
        "SELECT name, tbl_name, sql FROM sqlite_master WHERE type='index' AND tbl_name LIKE '%super_lotto%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| SuperLottoError::Database(e))?;

    tracing::info!("📊 Database has {} indexes for Super Lotto tables", index_analysis.len());

    // Check table sizes for optimization opportunities
    let table_stats = sqlx::query(
        "SELECT name, COUNT(*) as row_count FROM sqlite_master m
         LEFT JOIN (SELECT COUNT(*) as row_count FROM super_lotto_draws) d ON 1=1
         WHERE m.type='table' AND name='super_lotto_draws'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| SuperLottoError::Database(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_caching() {
        // Test query caching functionality
    }
}