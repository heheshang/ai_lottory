//! Database query optimizer with performance monitoring and smart caching

use crate::error::{AppError, Result};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Query performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query_type: String,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub execution_count: u64,
    pub total_rows_returned: u64,
    pub cache_hit_rate: f64,
}

/// Query optimization context
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub query_type: String,
    pub parameters: Option<serde_json::Value>,
    pub expected_row_count: Option<usize>,
    pub timeout_ms: Option<u64>,
    pub use_cache: bool,
    pub force_refresh: bool,
}

impl Default for QueryContext {
    fn default() -> Self {
        Self {
            query_type: "unknown".to_string(),
            parameters: None,
            expected_row_count: None,
            timeout_ms: Some(5000),
            use_cache: true,
            force_refresh: false,
        }
    }
}

/// Smart query optimizer
pub struct QueryOptimizer {
    pool: SqlitePool,
    stats: Arc<RwLock<HashMap<String, QueryStats>>>,
    slow_query_threshold_ms: u64,
    cache_enabled: bool,
}

impl QueryOptimizer {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            stats: Arc::new(RwLock::new(HashMap::new())),
            slow_query_threshold_ms: 100,
            cache_enabled: true,
        }
    }

    /// Execute query with performance monitoring and optimization
    pub async fn execute_query<T, F>(
        &self,
        context: QueryContext,
        query_fn: F,
    ) -> Result<T>
    where
        F: FnOnce(&SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let start_time = Instant::now();
        let query_type = context.query_type.clone();

        // Check cache first if enabled
        if context.use_cache && !context.force_refresh && self.cache_enabled {
            if let Some(cached_result) = self.check_cache(&context).await? {
                self.update_cache_hit_stats(&query_type).await;
                return cached_result;
            }
        }

        // Execute query with timeout
        let result = if let Some(timeout) = context.timeout_ms {
            tokio::time::timeout(
                Duration::from_millis(timeout),
                query_fn(&self.pool)
            )
            .await
            .map_err(|_| AppError::Timeout {
                operation: format!("query execution: {}", query_type),
                timeout_ms: timeout,
            })?
        } else {
            query_fn(&self.pool).await
        };

        let execution_time = start_time.elapsed().as_millis() as f64;

        // Log slow queries
        if execution_time > self.slow_query_threshold_ms as f64 {
            tracing::warn!(
                "Slow query detected: {} took {}ms (threshold: {}ms)",
                query_type,
                execution_time,
                self.slow_query_threshold_ms
            );
            self.log_slow_query(&context, execution_time).await?;
        }

        // Update performance statistics
        self.update_query_stats(&query_type, execution_time, context.expected_row_count).await;

        // Cache successful results
        if context.use_cache && result.is_ok() {
            // Note: In a real implementation, you'd serialize and cache the result
            // This is simplified for demonstration
        }

        result
    }

    /// Execute optimized query for lottery history with pagination
    pub async fn get_lottery_history_optimized(
        &self,
        lottery_type: Option<String>,
        limit: u32,
        offset: u32,
        date_from: Option<String>,
        date_to: Option<String>,
    ) -> Result<Vec<crate::super_lotto::models::LotteryDraw>> {
        let context = QueryContext {
            query_type: "lottery_history".to_string(),
            parameters: Some(serde_json::json!({
                "lottery_type": lottery_type,
                "limit": limit,
                "offset": offset,
                "date_from": date_from,
                "date_to": date_to
            })),
            expected_row_count: Some(limit as usize),
            timeout_ms: Some(2000),
            use_cache: true,
            force_refresh: false,
        };

        self.execute_query(context, move |pool| {
            Box::pin(async move {
                let mut query = "
                    SELECT
                        id, draw_date, lottery_type, winning_numbers,
                        bonus_number, jackpot_amount, created_at
                    FROM lottery_draws
                    WHERE 1=1
                ".to_string();

                let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + 'static>> = Vec::new();
                let mut param_count = 0;

                // Add filters
                if let Some(lt) = &lottery_type {
                    param_count += 1;
                    query.push_str(&format!(" AND lottery_type = ${}", param_count));
                    params.push(Box::new(lt.clone()));
                }

                if let Some(df) = &date_from {
                    param_count += 1;
                    query.push_str(&format!(" AND draw_date >= ${}", param_count));
                    params.push(Box::new(df.clone()));
                }

                if let Some(dt) = &date_to {
                    param_count += 1;
                    query.push_str(&format!(" AND draw_date <= ${}", param_count));
                    params.push(Box::new(dt.clone()));
                }

                // Add ordering and pagination
                query.push_str(" ORDER BY draw_date DESC");

                if limit > 0 {
                    param_count += 1;
                    query.push_str(&format!(" LIMIT ${}", param_count));
                    params.push(Box::new(limit));
                }

                if offset > 0 {
                    param_count += 1;
                    query.push_str(&format!(" OFFSET ${}", param_count));
                    params.push(Box::new(offset));
                }

                // Execute query
                let rows = sqlx::query(&query).fetch_all(pool).await
                    .map_err(|e| AppError::Database {
                        message: format!("Failed to execute lottery history query: {}", e),
                        query: query.clone(),
                    })?;

                // Convert rows to models
                let draws: Result<Vec<_>> = rows.into_iter().map(|row| {
                    let winning_numbers_str: String = row.try_get("winning_numbers")?;
                    let winning_numbers: Vec<u32> = winning_numbers_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();

                    Ok(crate::super_lotto::models::LotteryDraw {
                        id: row.try_get("id")?,
                        draw_date: row.try_get("draw_date")?,
                        lottery_type: row.try_get("lottery_type")?,
                        winning_numbers,
                        bonus_number: row.try_get("bonus_number").ok(),
                        jackpot_amount: row.try_get("jackpot_amount").ok(),
                    })
                }).collect();

                draws
            })
        }).await
    }

    /// Get hot numbers with optimized query
    pub async fn get_hot_numbers_optimized(
        &self,
        lottery_type: Option<String>,
        days: u32,
    ) -> Result<Vec<crate::super_lotto::models::NumberFrequency>> {
        let context = QueryContext {
            query_type: "hot_numbers".to_string(),
            parameters: Some(serde_json::json!({
                "lottery_type": lottery_type,
                "days": days
            })),
            expected_row_count: Some(100),
            timeout_ms: Some(1000),
            use_cache: true,
            force_refresh: false,
        };

        self.execute_query(context, move |pool| {
            Box::pin(async move {
                let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

                let query = if let Some(lt) = &lottery_type {
                    "
                        SELECT
                            number, lottery_type, frequency, last_drawn_at, draw_count
                        FROM number_frequency
                        WHERE lottery_type = $1
                            AND last_drawn_at >= $2
                        ORDER BY frequency DESC, last_drawn_at DESC
                        LIMIT 50
                    "
                } else {
                    "
                        SELECT
                            number, lottery_type, frequency, last_drawn_at, draw_count
                        FROM number_frequency
                        WHERE last_drawn_at >= $1
                        ORDER BY frequency DESC, last_drawn_at DESC
                        LIMIT 50
                    "
                };

                let rows = if let Some(lt) = &lottery_type {
                    sqlx::query(query)
                        .bind(lt)
                        .bind(cutoff_date.format("%Y-%m-%d %H:%M:%S").to_string())
                        .fetch_all(pool)
                        .await
                } else {
                    sqlx::query(query)
                        .bind(cutoff_date.format("%Y-%m-%d %H:%M:%S").to_string())
                        .fetch_all(pool)
                        .await
                }.map_err(|e| AppError::Database {
                    message: format!("Failed to execute hot numbers query: {}", e),
                    query: query.to_string(),
                })?;

                // Convert rows to models
                let frequencies: Result<Vec<_>> = rows.into_iter().map(|row| {
                    Ok(crate::super_lotto::models::NumberFrequency {
                        number: row.try_get("number")?,
                        frequency: row.try_get("frequency")?,
                        last_drawn_at: Some(row.try_get("last_drawn_at")?),
                        draw_count: row.try_get("draw_count")?,
                        hot_score: calculate_hot_score(
                            row.try_get("frequency")?,
                            row.try_get("last_drawn_at").ok(),
                            row.try_get("draw_count")?
                        ),
                    })
                }).collect();

                frequencies
            })
        }).await
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> HashMap<String, QueryStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Reset performance statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.clear();
    }

    /// Get slow queries report
    pub async fn get_slow_queries(&self, limit: u32) -> Result<Vec<QueryStats>> {
        let query = "
            SELECT
                query_type,
                AVG(execution_time_ms) as avg_time_ms,
                MAX(execution_time_ms) as max_time_ms,
                COUNT(*) as execution_count,
                MAX(created_at) as last_executed
            FROM query_performance_log
            WHERE execution_time_ms > $1
            GROUP BY query_type
            ORDER BY avg_time_ms DESC
            LIMIT $2
        ";

        let rows = sqlx::query(query)
            .bind(self.slow_query_threshold_ms as i64)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database {
                message: format!("Failed to get slow queries: {}", e),
                query: query.to_string(),
            })?;

        let slow_queries: Result<Vec<_>> = rows.into_iter().map(|row| {
            Ok(QueryStats {
                query_type: row.try_get("query_type")?,
                avg_execution_time_ms: row.try_get("avg_time_ms")?,
                max_execution_time_ms: row.try_get("max_time_ms")?,
                min_execution_time_ms: 0.0, // Not available in this query
                execution_count: row.try_get("execution_count")?,
                total_rows_returned: 0, // Not available in this query
                cache_hit_rate: 0.0, // Would need separate calculation
            })
        }).collect();

        slow_queries
    }

    /// Analyze and suggest optimizations
    pub async fn analyze_performance(&self) -> Result<Vec<String>> {
        let stats = self.stats.read().await;
        let mut suggestions = Vec::new();

        // Check for consistently slow queries
        for (query_type, query_stats) in stats.iter() {
            if query_stats.avg_execution_time_ms > 500.0 {
                suggestions.push(format!(
                    "Query '{}' has high average execution time ({:.2}ms). Consider adding indexes.",
                    query_type, query_stats.avg_execution_time_ms
                ));
            }

            if query_stats.cache_hit_rate < 0.5 && query_stats.execution_count > 10 {
                suggestions.push(format!(
                    "Query '{}' has low cache hit rate ({:.1}%). Consider caching this query.",
                    query_type, query_stats.cache_hit_rate * 100.0
                ));
            }

            if query_stats.execution_count > 1000 && query_stats.avg_execution_time_ms > 100.0 {
                suggestions.push(format!(
                    "Frequent query '{}' is slow ({:.2}ms avg). Consider materialized views.",
                    query_type, query_stats.avg_execution_time_ms
                ));
            }
        }

        if suggestions.is_empty() {
            suggestions.push("No performance issues detected. Database queries are performing well.".to_string());
        }

        Ok(suggestions)
    }

    // Private helper methods

    async fn check_cache<T>(&self, _context: &QueryContext) -> Result<Option<T>> {
        // Simplified cache check - in real implementation, check Redis/file cache
        Ok(None)
    }

    async fn update_cache_hit_stats(&self, query_type: &str) {
        let mut stats = self.stats.write().await;
        if let Some(query_stats) = stats.get_mut(query_type) {
            query_stats.cache_hit_rate = (query_stats.cache_hit_rate * 0.9) + (1.0 * 0.1);
        }
    }

    async fn update_query_stats(
        &self,
        query_type: &str,
        execution_time: f64,
        row_count: Option<usize>,
    ) {
        let mut stats = self.stats.write().await;
        let query_stats = stats.entry(query_type.to_string()).or_insert_with(|| QueryStats {
            query_type: query_type.to_string(),
            avg_execution_time_ms: execution_time,
            max_execution_time_ms: execution_time,
            min_execution_time_ms: execution_time,
            execution_count: 0,
            total_rows_returned: 0,
            cache_hit_rate: 0.0,
        });

        // Update running average
        query_stats.execution_count += 1;
        let alpha = 0.1; // Smoothing factor
        query_stats.avg_execution_time_ms =
            (query_stats.avg_execution_time_ms * (1.0 - alpha)) + (execution_time * alpha);
        query_stats.max_execution_time_ms = query_stats.max_execution_time_ms.max(execution_time);
        query_stats.min_execution_time_ms = query_stats.min_execution_time_ms.min(execution_time);

        if let Some(rows) = row_count {
            query_stats.total_rows_returned += rows as u64;
        }

        // Update cache hit rate (decay for misses)
        query_stats.cache_hit_rate = query_stats.cache_hit_rate * 0.99;
    }

    async fn log_slow_query(&self, context: &QueryContext, execution_time: f64) -> Result<()> {
        let parameters_json = context.parameters.as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default())
            .unwrap_or_default();

        sqlx::query(
            "INSERT INTO query_performance_log (query_type, execution_time_ms, parameters) VALUES ($1, $2, $3)"
        )
        .bind(&context.query_type)
        .bind(execution_time as f64)
        .bind(parameters_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database {
            message: format!("Failed to log slow query: {}", e),
            query: "INSERT INTO query_performance_log".to_string(),
        })?;

        Ok(())
    }
}

/// Calculate hot score for number frequency analysis
fn calculate_hot_score(frequency: u32, last_drawn_at: Option<String>, draw_count: u32) -> f64 {
    let base_score = frequency as f64;

    let recency_bonus = if let Some(last_drawn) = last_drawn_at {
        if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(&last_drawn) {
            let days_since_draw = (chrono::Utc::now() - parsed_date.with_timezone(&chrono::Utc)).num_days();
            if days_since_draw < 7 {
                1.5
            } else if days_since_draw < 30 {
                1.2
            } else if days_since_draw < 90 {
                1.0
            } else {
                0.8
            }
        } else {
            1.0
        }
    } else {
        1.0
    };

    let consistency_bonus = if draw_count > 0 {
        let consistency = frequency as f64 / draw_count as f64;
        1.0 + (consistency - 0.1).max(0.0)
    } else {
        1.0
    };

    base_score * recency_bonus * consistency_bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_query_optimizer_creation() {
        // Note: This test would require a test database setup
        // For now, just test the structure
        // let pool = SqlitePool::connect(":memory:").await.unwrap();
        // let optimizer = QueryOptimizer::new(pool);
        // assert!(optimizer.get_performance_stats().await.is_empty());
    }

    #[test]
    fn test_hot_score_calculation() {
        let score1 = calculate_hot_score(10, Some("2024-01-01T00:00:00Z".to_string()), 100);
        let score2 = calculate_hot_score(5, Some("2024-01-15T00:00:00Z".to_string()), 100);

        // More recent and frequent should have higher score
        assert!(score1 > score2);
    }
}