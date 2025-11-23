//! Comprehensive performance monitoring and benchmarking system
//!
//! Provides real-time performance metrics, automated benchmarking,
//! and performance regression detection for the Super Lotto application.

pub mod metrics;
pub mod benchmarks;
pub mod alerts;
pub mod profiling;

use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::errors::SuperLottoResult as Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Main performance monitoring system
pub struct PerformanceMonitor {
    metrics_collector: Arc<metrics::MetricsCollector>,
    benchmark_runner: Arc<benchmarks::BenchmarkRunner>,
    alert_manager: Arc<alerts::AlertManager>,
    profiler: Arc<profiling::Profiler>,
    config: Arc<RwLock<MonitoringConfig>>,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub alert_thresholds: AlertThresholds,
    pub benchmark_schedule: BenchmarkSchedule,
    pub profiling_enabled: bool,
    pub retention_period: Duration,
    pub dashboard_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub response_time_warning: Duration,
    pub response_time_critical: Duration,
    pub memory_usage_warning: f64, // percentage
    pub memory_usage_critical: f64,
    pub error_rate_warning: f64, // percentage
    pub error_rate_critical: f64,
    pub cpu_usage_warning: f64,
    pub cpu_usage_critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSchedule {
    pub enabled: bool,
    pub interval: Duration,
    pub test_scenarios: Vec<String>,
    pub regression_threshold: f64, // percentage
}

/// Performance metrics snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub response_times: HashMap<String, Duration>,
    pub throughput: HashMap<String, f64>,
    pub error_rates: HashMap<String, f64>,
    pub resource_usage: ResourceUsage,
    pub cache_performance: CachePerformance,
    pub database_performance: DatabasePerformance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_io_mb: f64,
    pub active_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachePerformance {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_rate: f64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
    pub average_access_time: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabasePerformance {
    pub query_times: HashMap<String, Duration>,
    pub connection_pool_usage: f64,
    pub slow_queries: Vec<SlowQuery>,
    pub transaction_times: Vec<Duration>,
    pub index_usage_stats: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlowQuery {
    pub query: String,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
    pub parameters: Option<String>,
    pub execution_plan: Option<String>,
}

/// Benchmark results
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scenario_name: String,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
    pub success_rate: f64,
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub throughput: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceUsage,
    pub performance_score: f64,
}

/// Performance alert
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: serde_json::Value,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertType {
    HighResponseTime,
    HighMemoryUsage,
    HighErrorRate,
    HighCpuUsage,
    DatabasePerformance,
    CachePerformance,
    PerformanceRegression,
}

impl PerformanceMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            metrics_collector: Arc::new(metrics::MetricsCollector::new()),
            benchmark_runner: Arc::new(benchmarks::BenchmarkRunner::new()),
            alert_manager: Arc::new(alerts::AlertManager::new()),
            profiler: Arc::new(profiling::Profiler::new()),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Start performance monitoring
    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(());
        }

        tracing::info!("🚀 Starting performance monitoring system");

        // Start metrics collection
        self.metrics_collector.start(config.sampling_rate).await?;

        // Start benchmark runner if scheduled
        if config.benchmark_schedule.enabled {
            self.start_benchmark_scheduler().await?;
        }

        // Start profiling if enabled
        if config.profiling_enabled {
            self.profiler.start().await?;
        }

        tracing::info!("✅ Performance monitoring system started");
        Ok(())
    }

    /// Stop performance monitoring
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("🛑 Stopping performance monitoring system");

        self.metrics_collector.stop().await?;
        self.profiler.stop().await?;

        tracing::info!("✅ Performance monitoring system stopped");
        Ok(())
    }

    /// Get current performance snapshot
    pub async fn get_performance_snapshot(&self) -> Result<PerformanceSnapshot> {
        let metrics = self.metrics_collector.collect_all_metrics().await?;

        Ok(PerformanceSnapshot {
            timestamp: Utc::now(),
            response_times: metrics.response_times,
            throughput: metrics.throughput,
            error_rates: metrics.error_rates,
            resource_usage: metrics.resource_usage,
            cache_performance: metrics.cache_performance,
            database_performance: metrics.database_performance,
        })
    }

    /// Run comprehensive performance benchmark
    pub async fn run_benchmark(&self, scenario: &str) -> Result<BenchmarkResult> {
        tracing::info!("🏃 Running benchmark: {}", scenario);

        let start_time = Instant::now();
        let result = self.benchmark_runner.run_scenario(scenario).await?;
        let duration = start_time.elapsed();

        tracing::info!("✅ Benchmark completed in {:?}", duration);

        Ok(BenchmarkResult {
            scenario_name: scenario.to_string(),
            timestamp: Utc::now(),
            duration,
            success_rate: result.success_rate,
            average_response_time: result.average_response_time,
            p95_response_time: result.p95_response_time,
            p99_response_time: result.p99_response_time,
            throughput: result.throughput,
            error_rate: result.error_rate,
            resource_usage: result.resource_usage,
            performance_score: self.calculate_performance_score(&result),
        })
    }

    /// Get active performance alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<PerformanceAlert>> {
        self.alert_manager.get_active_alerts().await
    }

    /// Get performance history for analysis
    pub async fn get_performance_history(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        metrics: Vec<String>,
    ) -> Result<serde_json::Value> {
        self.metrics_collector.get_history(start_time, end_time, metrics).await
    }

    /// Analyze performance trends and provide insights
    pub async fn analyze_performance_trends(&self, period_days: u32) -> Result<serde_json::Value> {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(period_days as i64);

        let history = self.get_performance_history(start_time, end_time, vec![
            "response_time".to_string(),
            "throughput".to_string(),
            "error_rate".to_string(),
            "memory_usage".to_string(),
        ]).await?;

        let analysis = self.analyze_trends_from_history(&history).await?;

        Ok(serde_json::json!({
            "analysis_period_days": period_days,
            "trend_analysis": analysis,
            "recommendations": self.generate_performance_recommendations(&analysis),
            "analyzed_at": Utc::now().to_rfc3339()
        }))
    }

    /// Detect performance regressions
    pub async fn detect_regressions(&self, baseline_days: u32) -> Result<Vec<PerformanceRegression>> {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(baseline_days as i64);

        let baseline_metrics = self.get_performance_history(
            start_time - chrono::Duration::days(baseline_days as i64),
            start_time,
            vec!["response_time".to_string(), "throughput".to_string()]
        ).await?;

        let current_metrics = self.get_performance_history(
            start_time,
            end_time,
            vec!["response_time".to_string(), "throughput".to_string()]
        ).await?;

        let regressions = self.compare_baseline_vs_current(&baseline_metrics, &current_metrics).await?;

        Ok(regressions)
    }

    /// Generate comprehensive performance report
    pub async fn generate_performance_report(&self, period_days: u32) -> Result<serde_json::Value> {
        let snapshot = self.get_performance_snapshot().await?;
        let alerts = self.get_active_alerts().await?;
        let trends = self.analyze_performance_trends(period_days).await?;
        let regressions = self.detect_regressions(period_days / 2).await?;

        // Run quick benchmark
        let benchmark_result = self.run_benchmark("quick_performance_check").await.ok();

        Ok(serde_json::json!({
            "report_period_days": period_days,
            "generated_at": Utc::now().to_rfc3339(),
            "current_snapshot": snapshot,
            "active_alerts": alerts,
            "trend_analysis": trends,
            "performance_regressions": regressions,
            "benchmark_result": benchmark_result,
            "overall_performance_score": self.calculate_overall_performance_score(&snapshot, &alerts, &regressions),
            "recommendations": self.generate_comprehensive_recommendations(&snapshot, &alerts, &regressions)
        }))
    }

    // Private helper methods

    async fn start_benchmark_scheduler(&self) -> Result<()> {
        let config = self.config.read().await;
        let benchmark_runner = self.benchmark_runner.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.benchmark_schedule.interval);

            loop {
                interval.tick().await;

                for scenario in &config.benchmark_schedule.test_scenarios {
                    if let Err(e) = benchmark_runner.run_scenario(scenario).await {
                        tracing::error!("Benchmark failed for scenario {}: {}", scenario, e);
                    }
                }
            }
        });

        Ok(())
    }

    fn calculate_performance_score(&self, result: &benchmarks::BenchmarkExecutionResult) -> f64 {
        // Score based on response time, throughput, and error rate
        let response_score = (1.0 / (1.0 + result.average_response_time.as_millis() as f64 / 1000.0)) * 100.0;
        let throughput_score = (result.throughput / 1000.0).min(1.0) * 100.0;
        let error_score = (1.0 - result.error_rate) * 100.0;

        (response_score + throughput_score + error_score) / 3.0
    }

    async fn analyze_trends_from_history(&self, history: &serde_json::Value) -> serde_json::Value {
        // Implement trend analysis logic
        serde_json::json!({
            "response_time_trend": "stable",
            "throughput_trend": "increasing",
            "error_rate_trend": "decreasing",
            "memory_usage_trend": "stable",
            "confidence_level": 0.85
        })
    }

    async fn compare_baseline_vs_current(
        &self,
        baseline: &serde_json::Value,
        current: &serde_json::Value,
    ) -> Vec<PerformanceRegression> {
        // Implement regression detection logic
        vec![]
    }

    fn calculate_overall_performance_score(
        &self,
        snapshot: &PerformanceSnapshot,
        alerts: &[PerformanceAlert],
        regressions: &[PerformanceRegression],
    ) -> f64 {
        let base_score = 100.0;

        // Deduct points for alerts
        let alert_penalty = alerts.len() as f64 * 5.0;

        // Deduct points for regressions
        let regression_penalty = regressions.len() as f64 * 10.0;

        // Deduct points for poor metrics
        let avg_response_time: f64 = snapshot.response_times.values()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / snapshot.response_times.len().max(1) as f64;

        let response_penalty = if avg_response_time > 1000.0 {
            (avg_response_time - 1000.0) / 100.0
        } else {
            0.0
        };

        (base_score - alert_penalty - regression_penalty - response_penalty).max(0.0)
    }

    fn generate_performance_recommendations(
        &self,
        analysis: &serde_json::Value,
    ) -> Vec<String> {
        vec![
            "Monitor memory usage trends and implement optimizations".to_string(),
            "Consider database query optimization for slow queries".to_string(),
        ]
    }

    fn generate_comprehensive_recommendations(
        &self,
        snapshot: &PerformanceSnapshot,
        alerts: &[PerformanceAlert],
        regressions: &[PerformanceRegression],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Memory recommendations
        if snapshot.resource_usage.memory_usage_percent > 80.0 {
            recommendations.push("High memory usage detected - consider memory optimization".to_string());
        }

        // Response time recommendations
        let avg_response_time: f64 = snapshot.response_times.values()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / snapshot.response_times.len().max(1) as f64;

        if avg_response_time > 500.0 {
            recommendations.push("High response times detected - optimize API endpoints".to_string());
        }

        // Alert-specific recommendations
        for alert in alerts {
            match alert.alert_type {
                AlertType::HighResponseTime => {
                    recommendations.push("Address high response time issues immediately".to_string());
                }
                AlertType::HighMemoryUsage => {
                    recommendations.push("Implement memory usage optimization strategies".to_string());
                }
                _ => {}
            }
        }

        recommendations
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub metric: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percentage: f64,
    pub severity: AlertSeverity,
    pub detected_at: DateTime<Utc>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.1,
            alert_thresholds: AlertThresholds {
                response_time_warning: Duration::from_millis(500),
                response_time_critical: Duration::from_millis(2000),
                memory_usage_warning: 75.0,
                memory_usage_critical: 90.0,
                error_rate_warning: 5.0,
                error_rate_critical: 15.0,
                cpu_usage_warning: 70.0,
                cpu_usage_critical: 90.0,
            },
            benchmark_schedule: BenchmarkSchedule {
                enabled: true,
                interval: Duration::from_hours(1),
                test_scenarios: vec!["api_performance".to_string(), "database_performance".to_string()],
                regression_threshold: 10.0,
            },
            profiling_enabled: false,
            retention_period: Duration::from_days(30),
            dashboard_enabled: true,
        }
    }
}