//! Enhanced performance tracking with detailed metrics

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use metrics::{counter, histogram, gauge};

/// Detailed performance operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub operation_id: String,
    pub operation_type: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration_ms: Option<f64>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}

impl PerformanceRecord {
    pub fn new(operation_type: String) -> Self {
        Self {
            operation_id: uuid::Uuid::new_v4().to_string(),
            operation_type,
            start_time: Instant::now(),
            end_time: None,
            duration_ms: None,
            success: None,
            error_message: None,
            metadata: serde_json::Value::Object(Default::default()),
            memory_usage_mb: None,
            cpu_usage_percent: None,
        }
    }

    pub fn complete(&mut self, success: bool) {
        self.end_time = Some(Instant::now());
        self.duration_ms = Some(self.end_time.unwrap().duration_since(self.start_time).as_millis() as f64);
        self.success = Some(success);

        // Record metrics
        histogram!(
            "operation_duration_ms",
            self.duration_ms.unwrap_or(0.0),
            "operation_type" => self.operation_type.clone()
        );

        counter!(
            "operations_total",
            1,
            "operation_type" => self.operation_type.clone(),
            "success" => success.to_string()
        );
    }

    pub fn set_error(&mut self, error: &str) {
        self.error_message = Some(error.to_string());
        counter!(
            "operation_errors_total",
            1,
            "operation_type" => self.operation_type.clone(),
            "error_type" => error.to_string()
        );
    }
}

/// Enhanced performance tracker with detailed monitoring
pub struct PerformanceTracker {
    records: Arc<RwLock<HashMap<String, PerformanceRecord>>>,
    operation_history: Arc<RwLock<Vec<PerformanceRecord>>>,
    max_history_size: usize,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 10000,
        }
    }

    /// Start tracking a new operation
    pub async fn start_operation(&self, operation_type: &str) -> String {
        let mut record = PerformanceRecord::new(operation_type.to_string());
        let operation_id = record.operation_id.clone();

        // Set initial metrics
        gauge!(
            "active_operations",
            self.records.read().await.len() as f64,
            "operation_type" => operation_type
        );

        {
            let mut records = self.records.write().await;
            records.insert(operation_id.clone(), record);
        }

        operation_id
    }

    /// Complete an operation with success status
    pub async fn complete_operation(&self, operation_id: &str, success: bool) -> Result<()> {
        let mut record = {
            let mut records = self.records.write().await;
            records.remove(operation_id)
                .ok_or_else(|| AppError::NotFound {
                    resource: format!("Operation record with ID: {}", operation_id),
                })?
        };

        record.complete(success);

        // Add to history
        {
            let mut history = self.operation_history.write().await;
            history.push(record.clone());

            // Maintain history size
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }

        tracing::debug!(
            operation_id = %operation_id,
            operation_type = %record.operation_type,
            duration_ms = %record.duration_ms.unwrap_or(0.0),
            success = success,
            "Operation completed"
        );

        Ok(())
    }

    /// Complete operation with error
    pub async fn complete_operation_with_error(
        &self,
        operation_id: &str,
        error: &str,
    ) -> Result<()> {
        let mut record = {
            let mut records = self.records.write().await;
            records.remove(operation_id)
                .ok_or_else(|| AppError::NotFound {
                    resource: format!("Operation record with ID: {}", operation_id),
                })?
        };

        record.set_error(error);
        record.complete(false);

        // Add to history
        {
            let mut history = self.operation_history.write().await;
            history.push(record.clone());

            // Maintain history size
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }

        tracing::error!(
            operation_id = %operation_id,
            operation_type = %record.operation_type,
            error = %error,
            "Operation failed"
        );

        Ok(())
    }

    /// Get operation statistics
    pub async fn get_operation_stats(&self, operation_type: &str) -> OperationStats {
        let history = self.operation_history.read().await;
        let relevant_records: Vec<_> = history
            .iter()
            .filter(|r| r.operation_type == operation_type)
            .collect();

        let mut stats = OperationStats::default();

        for record in &relevant_records {
            stats.total_operations += 1;

            if record.success.unwrap_or(false) {
                stats.successful_operations += 1;
            }

            if let Some(duration) = record.duration_ms {
                stats.total_duration_ms += duration;
                if duration > stats.max_duration_ms {
                    stats.max_duration_ms = duration;
                }
                if stats.min_duration_ms == 0.0 || duration < stats.min_duration_ms {
                    stats.min_duration_ms = duration;
                }
            }
        }

        if stats.total_operations > 0 {
            stats.success_rate = (stats.successful_operations as f64 / stats.total_operations as f64) * 100.0;
            stats.average_duration_ms = stats.total_duration_ms / stats.total_operations as f64;
        }

        stats
    }

    /// Get overall performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let records = self.records.read().await;
        let history = self.operation_history.read().await;

        let active_operations = records.len();
        let total_operations = history.len();

        let mut operation_counts = HashMap::new();
        let mut recent_operations = Vec::new();
        let now = Instant::now();

        // Count operations by type and collect recent ones
        for record in history.iter() {
            *operation_counts.entry(record.operation_type.clone()).or_insert(0) += 1;

            // Get operations from last hour
            if now.duration_since(record.start_time) < Duration::from_secs(3600) {
                recent_operations.push(record.clone());
            }
        }

        // Calculate average response time for recent operations
        let recent_avg_response_time = if !recent_operations.is_empty() {
            let total: f64 = recent_operations
                .iter()
                .filter_map(|r| r.duration_ms)
                .sum();
            total / recent_operations.len() as f64
        } else {
            0.0
        };

        PerformanceSummary {
            active_operations,
            total_operations,
            operation_counts,
            recent_avg_response_time_ms: recent_avg_response_time,
            recent_operations_count: recent_operations.len(),
        }
    }

    /// Clean up old records
    pub async fn cleanup_old_records(&self, max_age: Duration) {
        let mut records = self.records.write().await;
        let now = Instant::now();

        records.retain(|_, record| {
            now.duration_since(record.start_time) < max_age
        });

        tracing::debug!("Cleaned up old performance records");
    }

    /// Get slow operations (above threshold)
    pub async fn get_slow_operations(&self, threshold_ms: f64, limit: usize) -> Vec<PerformanceRecord> {
        let history = self.operation_history.read().await;
        let mut slow_ops: Vec<_> = history
            .iter()
            .filter(|r| {
                r.duration_ms.unwrap_or(0.0) > threshold_ms
            })
            .cloned()
            .collect();

        // Sort by duration (slowest first)
        slow_ops.sort_by(|a, b| {
            b.duration_ms.unwrap_or(0.0)
                .partial_cmp(&a.duration_ms.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        slow_ops.into_iter().take(limit).collect()
    }
}

/// Operation statistics
#[derive(Debug, Default, Clone)]
pub struct OperationStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f64,
    pub total_duration_ms: f64,
    pub average_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

/// Overall performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub active_operations: usize,
    pub total_operations: usize,
    pub operation_counts: HashMap<String, usize>,
    pub recent_avg_response_time_ms: f64,
    pub recent_operations_count: usize,
}

/// Initialize Prometheus metrics exporter
pub fn initialize_metrics() -> Result<(), AppError> {
    PrometheusBuilder::new()
        .with_http_endpoint(([127, 0, 0, 1], 9898))
        .install()
        .map_err(|e| AppError::Configuration {
            message: format!("Failed to initialize metrics: {}", e),
        })?;

    tracing::info!("Metrics exporter initialized on http://127.0.0.1:9898/metrics");
    Ok(())
}