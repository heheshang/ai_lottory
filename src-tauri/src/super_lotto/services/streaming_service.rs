//! Streaming service for lottery data analysis and processing

use crate::error::{AppError, Result};
use crate::super_lotto::models::{LotteryDraw, NumberFrequency, AnalysisResult};
use crate::utils::streaming::*;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::Stream;

/// Streaming analysis configuration
#[derive(Debug, Clone)]
pub struct StreamingAnalysisConfig {
    pub batch_size: usize,
    pub max_memory_mb: usize,
    pub enable_real_time: bool,
    pub buffer_size: usize,
    pub chunk_size: usize,
}

impl Default for StreamingAnalysisConfig {
    fn default() -> Self {
        Self {
            batch_size: 500,
            max_memory_mb: 50,
            enable_real_time: true,
            buffer_size: 2000,
            chunk_size: 1000,
        }
    }
}

/// Real-time analysis result with streaming metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingAnalysisResult {
    pub data: AnalysisResult,
    pub processed_count: u64,
    pub total_count: u64,
    pub processing_rate: f64,
    pub memory_usage_mb: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Streaming service for large-scale lottery analysis
pub struct StreamingAnalysisService {
    pool: SqlitePool,
    config: StreamingAnalysisConfig,
    active_streams: Arc<RwLock<HashMap<String, StreamStats>>>,
}

impl StreamingAnalysisService {
    pub fn new(pool: SqlitePool, config: StreamingAnalysisConfig) -> Self {
        Self {
            pool,
            config,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stream lottery draws for analysis with memory management
    pub async fn stream_lottery_draws(
        &self,
        lottery_type: Option<String>,
        date_from: Option<String>,
        date_to: Option<String>,
    ) -> impl Stream<Item = Result<LotteryDraw>> {
        let mut query = "
            SELECT
                id, draw_date, lottery_type, winning_numbers,
                bonus_number, jackpot_amount, created_at
            FROM lottery_draws
            WHERE 1=1
        ".to_string();

        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(lt) = &lottery_type {
            query.push_str(&format!(" AND lottery_type = ${}", param_index));
            params.push(lt.clone());
            param_index += 1;
        }

        if let Some(df) = &date_from {
            query.push_str(&format!(" AND draw_date >= ${}", param_index));
            params.push(df.clone());
            param_index += 1;
        }

        if let Some(dt) = &date_to {
            query.push_str(&format!(" AND draw_date <= ${}", param_index));
            params.push(dt.clone());
            param_index += 1;
        }

        query.push_str(" ORDER BY draw_date DESC");

        let config = StreamConfig {
            buffer_size: self.config.buffer_size,
            batch_size: self.config.chunk_size,
            timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes
            backpressure_delay: std::time::Duration::from_millis(5),
            max_memory_usage_mb: self.config.max_memory_mb,
            enable_compression: false,
        };

        DatabaseStream::new(self.pool.clone(), query, config)
            .stream()
            .await
    }

    /// Perform hot number analysis with streaming
    pub async fn analyze_hot_numbers_streaming(
        &self,
        lottery_type: Option<String>,
        days: u32,
    ) -> impl Stream<Item = Result<StreamingAnalysisResult>> {
        let pool = self.pool.clone();
        let config = self.config.clone();

        async_stream::stream! {
            let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);
            let start_time = std::time::Instant::now();
            let mut processed_count = 0u64;
            let mut total_count = 0u64;

            // Get total count first
            if let Ok(count) = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM lottery_draws WHERE draw_date >= ?"
            )
            .bind(cutoff_date.format("%Y-%m-%d").to_string())
            .fetch_one(&pool)
            .await
            {
                total_count = count as u64;
            }

            // Stream the data in chunks
            let offset_stream = utils::iterator_stream(
                0i64..,
                StreamConfig {
                    buffer_size: 100,
                    batch_size: 1,
                    ..Default::default()
                }
            );

            let mut stream = Box::pin(offset_stream);

            while let Some(offset_result) = stream.next().await {
                match offset_result {
                    Ok(offset) => {
                        let chunk_size = config.chunk_size as i64;
                        let query = "
                            SELECT winning_numbers, lottery_type
                            FROM lottery_draws
                            WHERE draw_date >= ?
                            ORDER BY draw_date
                            LIMIT ? OFFSET ?
                        ";

                        match sqlx::query(query)
                            .bind(cutoff_date.format("%Y-%m-%d").to_string())
                            .bind(chunk_size)
                            .bind(offset)
                            .fetch_all(&pool)
                            .await
                        {
                            Ok(rows) => {
                                if rows.is_empty() {
                                    break;
                                }

                                // Process chunk
                                let mut number_freq: HashMap<u32, u32> = HashMap::new();
                                let mut recent_freq: HashMap<u32, u32> = HashMap::new();

                                for row in rows {
                                    let numbers_str: String = row.get("winning_numbers");
                                    let numbers: Vec<u32> = numbers_str
                                        .split(',')
                                        .filter_map(|s| s.trim().parse().ok())
                                        .collect();

                                    let lotto_type: String = row.get("lottery_type");

                                    // Skip if lottery type filter is applied
                                    if let Some(ref filter_type) = lottery_type {
                                        if lotto_type != *filter_type {
                                            continue;
                                        }
                                    }

                                    for number in numbers {
                                        *number_freq.entry(number).or_insert(0) += 1;
                                        *recent_freq.entry(number).or_insert(0) += 1;
                                    }

                                    processed_count += 1;
                                }

                                // Calculate hot numbers
                                let hot_numbers: Vec<NumberFrequency> = number_freq
                                    .into_iter()
                                    .map(|(number, frequency)| {
                                        let hot_score = calculate_streaming_hot_score(
                                            frequency,
                                            processed_count as u32,
                                            days
                                        );

                                        NumberFrequency {
                                            number,
                                            frequency,
                                            last_drawn_at: None, // Would need additional query
                                            draw_count: processed_count as u32,
                                            hot_score,
                                        }
                                    })
                                    .filter(|nf| nf.hot_score > 1.0)
                                    .take(20) // Top 20 hot numbers
                                    .collect();

                                // Create analysis result
                                let analysis_result = AnalysisResult {
                                    algorithm: "STREAMING_HOT_NUMBERS".to_string(),
                                    parameters: serde_json::json!({
                                        "lottery_type": lottery_type,
                                        "days": days,
                                        "method": "streaming"
                                    }),
                                    result: serde_json::json!({
                                        "hot_numbers": hot_numbers,
                                        "analysis_period_days": days,
                                        "total_draws_analyzed": processed_count
                                    }),
                                    confidence_score: calculate_confidence_score(processed_count, days),
                                    timestamp: chrono::Utc::now(),
                                };

                                let elapsed = start_time.elapsed().as_secs_f64();
                                let processing_rate = processed_count as f64 / elapsed.max(0.1);

                                let streaming_result = StreamingAnalysisResult {
                                    data: analysis_result,
                                    processed_count,
                                    total_count,
                                    processing_rate,
                                    memory_usage_mb: estimate_memory_usage(processed_count),
                                    timestamp: chrono::Utc::now(),
                                };

                                yield Ok(streaming_result);

                                // Apply backpressure
                                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            }
                            Err(e) => {
                                yield Err(AppError::Database {
                                    message: format!("Failed to fetch lottery data chunk: {}", e),
                                    query: "SELECT winning_numbers...".to_string(),
                                });
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(AppError::Internal {
                            message: format!("Stream error: {}", e),
                        });
                        break;
                    }
                }

                // Check if we've processed all data
                if total_count > 0 && processed_count >= total_count {
                    break;
                }
            }
        }
    }

    /// Perform frequency analysis with streaming and memory management
    pub async fn analyze_frequency_streaming(
        &self,
        lottery_type: Option<String>,
        numbers: Vec<u32>,
    ) -> impl Stream<Item = Result<StreamingAnalysisResult>> {
        let pool = self.pool.clone();
        let config = self.config.clone();

        async_stream::stream! {
            let start_time = std::time::Instant::now();
            let mut processed_count = 0u64;

            // Stream analysis for each number
            let number_stream = utils::iterator_stream(numbers.into_iter(), StreamConfig::default());
            let mut stream = Box::pin(number_stream);

            while let Some(number_result) = stream.next().await {
                match number_result {
                    Ok(number) => {
                        // Analyze frequency for this specific number
                        let query = "
                            SELECT
                                COUNT(*) as frequency,
                                MAX(draw_date) as last_drawn,
                                MIN(draw_date) as first_drawn
                            FROM lottery_draws
                            WHERE ',' || winning_numbers || ',' LIKE ?
                        ";

                        let like_pattern = format!("%,%,%", number, number);

                        match sqlx::query(query)
                            .bind(like_pattern)
                            .fetch_one(&pool)
                            .await
                        {
                            Ok(row) => {
                                let frequency: i64 = row.get("frequency");
                                let last_drawn: Option<String> = row.get("last_drawn");
                                let first_drawn: Option<String> = row.get("first_drawn");

                                processed_count += 1;

                                let analysis_result = AnalysisResult {
                                    algorithm: "STREAMING_FREQUENCY_ANALYSIS".to_string(),
                                    parameters: serde_json::json!({
                                        "number": number,
                                        "lottery_type": lottery_type
                                    }),
                                    result: serde_json::json!({
                                        "number": number,
                                        "frequency": frequency,
                                        "last_drawn": last_drawn,
                                        "first_drawn": first_drawn,
                                        "analysis_type": "individual"
                                    }),
                                    confidence_score: calculate_number_confidence_score(frequency as u32),
                                    timestamp: chrono::Utc::now(),
                                };

                                let elapsed = start_time.elapsed().as_secs_f64();
                                let processing_rate = processed_count as f64 / elapsed.max(0.1);

                                let streaming_result = StreamingAnalysisResult {
                                    data: analysis_result,
                                    processed_count,
                                    total_count: numbers.len() as u64,
                                    processing_rate,
                                    memory_usage_mb: estimate_memory_usage(processed_count),
                                    timestamp: chrono::Utc::now(),
                                };

                                yield Ok(streaming_result);
                            }
                            Err(e) => {
                                yield Err(AppError::Database {
                                    message: format!("Failed to analyze frequency for number {}: {}", number, e),
                                    query: "SELECT COUNT(*)...".to_string(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(AppError::Internal {
                            message: format!("Number stream error: {}", e),
                        });
                    }
                }
            }
        }
    }

    /// Batch analysis with streaming for large datasets
    pub async fn batch_analysis_streaming(
        &self,
        algorithms: Vec<String>,
        parameters: serde_json::Value,
    ) -> impl Stream<Item = Result<StreamingAnalysisResult>> {
        let pool = self.pool.clone();
        let config = self.config.clone();

        async_stream::stream! {
            let start_time = std::time::Instant::now();
            let mut processed_count = 0u64;
            let total_count = algorithms.len() as u64;

            for algorithm in algorithms {
                let algorithm_start = std::time::Instant::now();

                // Create stream for this algorithm's data processing
                let draw_stream = self.stream_lottery_draws(
                    parameters.get("lottery_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    parameters.get("date_from").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    parameters.get("date_to").and_then(|v| v.as_str()).map(|s| s.to_string()),
                );

                // Process draws in batches
                let processor = StreamProcessor::new(
                    StreamConfig {
                        batch_size: config.batch_size,
                        max_memory_usage_mb: config.max_memory_mb / algorithms.len(),
                        ..Default::default()
                    },
                    |batch: Vec<LotteryDraw>| {
                        // Simulate algorithm processing
                        Ok(batch.iter().map(|draw| {
                            serde_json::json!({
                                "draw_id": draw.id,
                                "algorithm": algorithm,
                                "processed_at": chrono::Utc::now()
                            })
                        }).collect())
                    }
                );

                let processed_stream = processor.process_stream(draw_stream);
                let mut stream = Box::pin(processed_stream);

                let mut batch_results = Vec::new();
                let mut algorithm_processed = 0u64;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(batch_item) => {
                            batch_results.push(batch_item);
                            algorithm_processed += 1;
                            processed_count += 1;
                        }
                        Err(e) => {
                            yield Err(e);
                            continue;
                        }
                    }
                }

                let algorithm_elapsed = algorithm_start.elapsed().as_secs_f64();

                // Create analysis result for this algorithm
                let analysis_result = AnalysisResult {
                    algorithm,
                    parameters: parameters.clone(),
                    result: serde_json::json!({
                        "processed_items": algorithm_processed,
                        "batch_results": batch_results,
                        "processing_time_seconds": algorithm_elapsed
                    }),
                    confidence_score: calculate_batch_confidence_score(algorithm_processed as u32),
                    timestamp: chrono::Utc::now(),
                };

                let elapsed = start_time.elapsed().as_secs_f64();
                let processing_rate = processed_count as f64 / elapsed.max(0.1);

                let streaming_result = StreamingAnalysisResult {
                    data: analysis_result,
                    processed_count,
                    total_count,
                    processing_rate,
                    memory_usage_mb: estimate_memory_usage(processed_count),
                    timestamp: chrono::Utc::now(),
                };

                yield Ok(streaming_result);
            }
        }
    }

    /// Get statistics for all active streams
    pub async fn get_stream_statistics(&self) -> HashMap<String, StreamStats> {
        let stats = self.active_streams.read().await;
        stats.clone()
    }

    /// Cancel a specific stream by ID
    pub async fn cancel_stream(&self, stream_id: &str) -> Result<bool> {
        let mut stats = self.active_streams.write().await;
        Ok(stats.remove(stream_id).is_some())
    }
}

// Helper functions for streaming analysis

fn calculate_streaming_hot_score(frequency: u32, total_draws: u32, days: u32) -> f64 {
    if total_draws == 0 {
        return 0.0;
    }

    let frequency_ratio = frequency as f64 / total_draws as f64;
    let expected_frequency = 1.0 / 45.0; // Assuming 45 numbers (typical lottery)
    let hotness_factor = (frequency_ratio / expected_frequency).max(0.0);

    // Apply recency weighting (newer draws get higher weight)
    let recency_weight = if days < 30 { 1.5 } else if days < 90 { 1.2 } else { 1.0 };

    hotness_factor * recency_weight * 100.0
}

fn calculate_confidence_score(processed_count: u64, days: u32) -> f64 {
    // Higher confidence with more data and longer time periods
    let data_factor = (processed_count as f64 / 1000.0).min(1.0);
    let time_factor = (days as f64 / 365.0).min(1.0);

    (data_factor * 0.6 + time_factor * 0.4) * 100.0
}

fn calculate_number_confidence_score(frequency: u32) -> f64 {
    // Confidence increases with frequency
    ((frequency as f64).ln() / 10.0).min(1.0) * 100.0
}

fn calculate_batch_confidence_score(processed_items: u32) -> f64 {
    // Confidence based on batch size
    ((processed_items as f64 / 100.0).ln() / 5.0).min(1.0) * 100.0
}

fn estimate_memory_usage(processed_count: u64) -> f64 {
    // Rough estimate: 1KB per processed item
    (processed_count as f64 * 1024.0) / (1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_hot_score_calculation() {
        let score1 = calculate_streaming_hot_score(50, 1000, 30);
        let score2 = calculate_streaming_hot_score(100, 1000, 30);

        assert!(score2 > score1);
    }

    #[test]
    fn test_confidence_score_calculation() {
        let confidence1 = calculate_confidence_score(100, 30);
        let confidence2 = calculate_confidence_score(1000, 90);

        assert!(confidence2 > confidence1);
    }
}