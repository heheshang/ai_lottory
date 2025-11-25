//! Log Metrics Collection
//!
//! Implements metrics collection and monitoring for logging system performance.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex as AsyncMutex;
use chrono::{DateTime, Utc};

/// Default metrics collector implementation
pub struct DefaultLogMetricsCollector {
    metrics: Arc<RwLock<LogMetrics>>,
    detailed_metrics: Arc<RwLock<HashMap<String, DetailedLogMetrics>>>,
    collection_config: MetricsCollectionConfig,
    metadata: MetricsMetadata,
}

#[derive(Debug, Clone)]
pub struct MetricsCollectionConfig {
    pub collect_per_target: bool,
    pub collect_per_level: bool,
    pub track_performance: bool,
    pub track_errors: bool,
    pub max_history_hours: u64,
}

impl Default for MetricsCollectionConfig {
    fn default() -> Self {
        Self {
            collect_per_target: true,
            collect_per_level: true,
            track_performance: true,
            track_errors: true,
            max_history_hours: 24,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetailedLogMetrics {
    pub target: String,
    pub level: LogLevel,
    pub count: u64,
    pub total_size: u64,
    pub average_size: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub error_count: u64,
    pub performance_ms: Vec<f64>,
}

impl DetailedLogMetrics {
    pub fn new(target: String, level: LogLevel) -> Self {
        let now = Utc::now();
        Self {
            target,
            level,
            count: 0,
            total_size: 0,
            average_size: 0.0,
            first_seen: now,
            last_seen: now,
            error_count: 0,
            performance_ms: Vec::new(),
        }
    }

    pub fn record_log(&mut self, size_bytes: usize, duration_ms: f64) {
        let now = Utc::now();

        self.count += 1;
        self.total_size += size_bytes as u64;
        self.last_seen = now;

        // Update average size
        self.average_size = self.total_size as f64 / self.count as f64;

        // Track performance if enabled
        if self.performance_ms.len() > 1000 {
            // Keep only recent 1000 measurements
            self.performance_ms.remove(0);
        }
        self.performance_ms.push(duration_ms);
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    pub fn error_rate(&self) -> f64 {
        if self.count > 0 {
            self.error_count as f64 / self.count as f64
        } else {
            0.0
        }
    }

    pub fn average_performance_ms(&self) -> f64 {
        if self.performance_ms.is_empty() {
            0.0
        } else {
            self.performance_ms.iter().sum::<f64>() / self.performance_ms.len() as f64
        }
    }
}

impl DefaultLogMetricsCollector {
    pub fn new() -> Self {
        Self::with_config(MetricsCollectionConfig::default())
    }

    pub fn with_config(config: MetricsCollectionConfig) -> Self {
        let metadata = MetricsMetadata {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            description: "Default log metrics collector".to_string(),
            supported_metrics: vec![
                "total_logs".to_string(),
                "logs_by_level".to_string(),
                "logs_by_target".to_string(),
                "error_rate".to_string(),
                "average_size".to_string(),
                "performance_ms".to_string(),
            ],
        };

        Self {
            metrics: Arc::new(RwLock::new(LogMetrics::new())),
            detailed_metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_config: config,
            metadata,
        }
    }

    pub fn get_target_metrics(&self, target: &str) -> Result<LogMetrics> {
        let detailed = self.detailed_metrics.read().unwrap();
        let mut target_metrics = LogMetrics::new();

        for (key, detailed_metric) in detailed.iter() {
            if detailed_metric.target == target {
                target_metrics.total_logs += detailed_metric.count;
                target_metrics.total_bytes += detailed_metric.total_size;

                // Update first and last log timestamps
                match (target_metrics.first_log, detailed_metric.first_seen) {
                    (None, _) => target_metrics.first_log = Some(detailed_metric.first_seen),
                    (Some(first), new) if new < first => target_metrics.first_log = Some(new),
                    _ => {}
                }

                match (target_metrics.last_log, detailed_metric.last_seen) {
                    (None, _) => target_metrics.last_log = Some(detailed_metric.last_seen),
                    (Some(last), new) if new > last => target_metrics.last_log = Some(new),
                    _ => {}
                }

                // Update level counts
                let level_str = detailed_metric.level.as_str().to_lowercase();
                *target_metrics.logs_by_level.entry(level_str).or_insert(0) += detailed_metric.count;

                // Update target counts
                *target_metrics.logs_by_target.entry(target.to_string()).or_insert(0) += detailed_metric.count;

                target_metrics.errors += detailed_metric.error_count;
                target_metrics.warnings += if matches!(detailed_metric.level, LogLevel::Warn) { detailed_metric.count } else { 0 };
            }
        }

        if target_metrics.total_logs > 0 {
            target_metrics.average_log_size = target_metrics.total_bytes as f64 / target_metrics.total_logs as f64;
        }

        Ok(target_metrics)
    }

    pub fn get_level_metrics(&self, level: LogLevel) -> Result<LogMetrics> {
        let detailed = self.detailed_metrics.read().unwrap();
        let mut level_metrics = LogMetrics::new();

        for detailed_metric in detailed.values() {
            if detailed_metric.level == level {
                level_metrics.total_logs += detailed_metric.count;
                level_metrics.total_bytes += detailed_metric.total_size;
                level_metrics.errors += detailed_metric.error_count;
                level_metrics.warnings += if matches!(level, LogLevel::Warn) { detailed_metric.count } else { 0 };
            }
        }

        if level_metrics.total_logs > 0 {
            level_metrics.average_log_size = level_metrics.total_bytes as f64 / level_metrics.total_logs as f64;
        }

        Ok(level_metrics)
    }

    pub fn get_top_targets(&self, limit: usize) -> Result<Vec<(String, u64)>> {
        let detailed = self.detailed_metrics.read().unwrap();
        let mut target_counts: HashMap<String, u64> = HashMap::new();

        for detailed_metric in detailed.values() {
            *target_counts.entry(detailed_metric.target.clone()).or_insert(0) += detailed_metric.count;
        }

        let mut counts: Vec<(String, u64)> = target_counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts.truncate(limit);

        Ok(counts)
    }

    pub fn cleanup_old_metrics(&self) {
        let cutoff_time = Utc::now() - chrono::Duration::hours(self.collection_config.max_history_hours as i64);

        let mut detailed = self.detailed_metrics.write().unwrap();
        detailed.retain(|_, metrics| metrics.last_seen > cutoff_time);
    }

    pub fn export_metrics(&self) -> Result<serde_json::Value> {
        let metrics = self.metrics.read().unwrap();
        let detailed = self.detailed_metrics.read().unwrap();

        let mut export = serde_json::Map::new();

        // General metrics
        export.insert("summary".to_string(), serde_json::to_value(&*metrics)?);

        // Detailed metrics
        let detailed_json: Vec<serde_json::Value> = detailed.values()
            .map(|m| serde_json::to_value(m))
            .collect::<Result<Vec<_>, _>>()?;
        export.insert("detailed".to_string(), serde_json::Value::Array(detailed_json));

        // Export timestamp
        export.insert("exported_at".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()));

        Ok(serde_json::Value::Object(export))
    }
}

impl Default for DefaultLogMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogMetricsCollector for DefaultLogMetricsCollector {
    async fn record_log(&self, record: &dyn LogRecord) {
        let message_size = record.message().len();
        let start_time = std::time::Instant::now();

        // Update general metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.record_log(record.level(), record.target(), message_size);
        }

        // Update detailed metrics if configured
        if self.collection_config.collect_per_target || self.collection_config.collect_per_level {
            let key = format!("{}:{}", record.target(), record.level().as_str());
            let mut detailed = self.detailed_metrics.write().unwrap();

            let detailed_metric = detailed.entry(key.clone())
                .or_insert_with(|| DetailedLogMetrics::new(record.target().to_string(), record.level()));

            let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
            detailed_metric.record_log(message_size, duration_ms);
        }
    }

    async fn get_metrics(&self) -> Result<LogMetrics> {
        let metrics = self.metrics.read().unwrap();
        Ok(metrics.clone())
    }

    async fn reset_metrics(&self) -> Result<()> {
        {
            let mut metrics = self.metrics.write().unwrap();
            *metrics = LogMetrics::new();
        }

        {
            let mut detailed = self.detailed_metrics.write().unwrap();
            detailed.clear();
        }

        Ok(())
    }

    fn metadata(&self) -> &MetricsMetadata {
        &self.metadata
    }
}

/// Performance-aware metrics collector with timing measurements
pub struct PerformanceMetricsCollector {
    inner: Arc<dyn LogMetricsCollector>,
    performance_threshold_ms: f64,
    slow_log_threshold_ms: f64,
}

impl PerformanceMetricsCollector {
    pub fn new(inner: Arc<dyn LogMetricsCollector>) -> Self {
        Self {
            inner,
            performance_threshold_ms: 100.0, // Alert if logging takes >100ms
            slow_log_threshold_ms: 10.0,    // Track logs that take >10ms
        }
    }

    pub fn with_thresholds(mut self, performance_ms: f64, slow_log_ms: f64) -> Self {
        self.performance_threshold_ms = performance_ms;
        self.slow_log_threshold_ms = slow_log_ms;
        self
    }

    pub async fn record_log_with_timing(&self, record: &dyn LogRecord) -> Result<f64> {
        let start_time = std::time::Instant::now();
        self.inner.record_log(record).await;
        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        // Alert on slow logging
        if duration_ms > self.performance_threshold_ms {
            eprintln!("⚠️  Slow logging operation: {:.2}ms for {} message",
                duration_ms, record.target());
        }

        Ok(duration_ms)
    }

    pub async fn get_performance_stats(&self) -> Result<PerformanceStats> {
        // This would need to be implemented with additional performance tracking
        Ok(PerformanceStats {
            average_log_time_ms: 0.0,
            slow_log_count: 0,
            slow_log_percentage: 0.0,
            max_log_time_ms: 0.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub average_log_time_ms: f64,
    pub slow_log_count: u64,
    pub slow_log_percentage: f64,
    pub max_log_time_ms: f64,
}

#[async_trait]
impl LogMetricsCollector for PerformanceMetricsCollector {
    async fn record_log(&self, record: &dyn LogRecord) {
        self.record_log_with_timing(record).await.ok();
    }

    async fn get_metrics(&self) -> Result<LogMetrics> {
        self.inner.get_metrics().await
    }

    async fn reset_metrics(&self) -> Result<()> {
        self.inner.reset_metrics().await
    }

    fn metadata(&self) -> &MetricsMetadata {
        self.inner.metadata()
    }
}

/// Metrics aggregator for combining multiple collectors
pub struct MetricsAggregator {
    collectors: Vec<Arc<dyn LogMetricsCollector>>,
    aggregation_mode: AggregationMode,
    metadata: MetricsMetadata,
}

#[derive(Debug, Clone)]
pub enum AggregationMode {
    Sum,    // Add all metrics together
    Average, // Average metrics across collectors
    Max,    // Take maximum values
}

impl MetricsAggregator {
    pub fn new(mode: AggregationMode) -> Self {
        let metadata = MetricsMetadata {
            name: "aggregator".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Metrics aggregator with {:?} mode", mode),
            supported_metrics: vec!["aggregated".to_string()],
        };

        Self {
            collectors: Vec::new(),
            aggregation_mode: mode,
            metadata,
        }
    }

    pub fn add_collector(mut self, collector: Arc<dyn LogMetricsCollector>) -> Self {
        self.collectors.push(collector);
        self
    }

    async fn aggregate_metrics(&self, metrics_list: Vec<LogMetrics>) -> LogMetrics {
        if metrics_list.is_empty() {
            return LogMetrics::new();
        }

        match self.aggregation_mode {
            AggregationMode::Sum => {
                let mut aggregated = LogMetrics::new();
                for metrics in metrics_list {
                    aggregated.total_logs += metrics.total_logs;
                    aggregated.total_bytes += metrics.total_bytes;
                    aggregated.errors += metrics.errors;
                    aggregated.warnings += metrics.warnings;

                    // Merge level and target counts
                    for (level, count) in metrics.logs_by_level {
                        *aggregated.logs_by_level.entry(level).or_insert(0) += count;
                    }
                    for (target, count) in metrics.logs_by_target {
                        *aggregated.logs_by_target.entry(target).or_insert(0) += count;
                    }

                    // Take earliest first_log and latest last_log
                    aggregated.first_log = match (aggregated.first_log, metrics.first_log) {
                        (None, None) => None,
                        (None, Some(time)) => Some(time),
                        (Some(time), None) => Some(time),
                        (Some(earliest), Some(latest)) if latest < earliest => Some(latest),
                        _ => aggregated.first_log,
                    };

                    aggregated.last_log = match (aggregated.last_log, metrics.last_log) {
                        (None, None) => None,
                        (None, Some(time)) => Some(time),
                        (Some(time), None) => Some(time),
                        (Some(earliest), Some(latest)) if latest > earliest => Some(latest),
                        _ => aggregated.last_log,
                    };
                }

                if aggregated.total_logs > 0 {
                    aggregated.average_log_size = aggregated.total_bytes as f64 / aggregated.total_logs as f64;
                }

                aggregated
            }
            AggregationMode::Average => {
                let mut aggregated = LogMetrics::new();
                let count = metrics_list.len() as f64;

                for metrics in &metrics_list {
                    aggregated.total_logs += metrics.total_logs;
                    aggregated.total_bytes += metrics.total_bytes;
                    aggregated.errors += metrics.errors;
                    aggregated.warnings += metrics.warnings;
                }

                aggregated.total_logs = (aggregated.total_logs as f64 / count) as u64;
                aggregated.total_bytes = (aggregated.total_bytes as f64 / count) as u64;
                aggregated.errors = (aggregated.errors as f64 / count) as u64;
                aggregated.warnings = (aggregated.warnings as f64 / count) as u64;

                if aggregated.total_logs > 0 {
                    aggregated.average_log_size = aggregated.total_bytes as f64 / aggregated.total_logs as f64;
                }

                // For timestamps, we'll take the most recent last_log
                if let Some(latest_time) = metrics_list.iter()
                    .filter_map(|m| m.last_log)
                    .max() {
                    aggregated.last_log = Some(latest_time);
                }

                aggregated
            }
            AggregationMode::Max => {
                metrics_list.into_iter()
                    .max_by_key(|m| m.total_logs)
                    .unwrap_or_default()
            }
        }
    }
}

#[async_trait]
impl LogMetricsCollector for MetricsAggregator {
    async fn record_log(&self, record: &dyn LogRecord) {
        // Record to all collectors
        for collector in &self.collectors {
            collector.record_log(record).await;
        }
    }

    async fn get_metrics(&self) -> Result<LogMetrics> {
        let mut metrics_list = Vec::new();
        for collector in &self.collectors {
            metrics_list.push(collector.get_metrics().await?);
        }

        Ok(self.aggregate_metrics(metrics_list).await)
    }

    async fn reset_metrics(&self) -> Result<()> {
        for collector in &self.collectors {
            collector.reset_metrics().await?;
        }
        Ok(())
    }

    fn metadata(&self) -> &MetricsMetadata {
        &self.metadata
    }
}