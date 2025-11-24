//! Simple performance monitoring for Super Lotto application

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Simple performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub query_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub slow_queries: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&mut self, duration: Duration) {
        self.query_count += 1;
        self.total_duration += duration;
        self.average_duration = self.total_duration / self.query_count.max(1) as u32;

        if duration > Duration::from_millis(100) {
            self.slow_queries += 1;
        }
    }

    pub fn get_average_ms(&self) -> f64 {
        self.average_duration.as_millis() as f64
    }

    pub fn get_slow_query_rate(&self) -> f64 {
        if self.query_count == 0 {
            0.0
        } else {
            (self.slow_queries as f64 / self.query_count as f64) * 100.0
        }
    }
}

/// Simple performance tracker
pub struct PerformanceTracker {
    start_time: Instant,
    metrics: PerformanceMetrics,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: PerformanceMetrics::new(),
        }
    }

    pub fn record_operation(&mut self, duration: Duration) {
        self.metrics.record_query(duration);
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}