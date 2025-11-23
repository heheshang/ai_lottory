//! Database Health Monitoring Module
//!
//! Provides comprehensive database health monitoring, performance metrics,
//! and automated alerting for database issues.

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use serde_json::Value;

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub connection_count: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub response_time_ms: u64,
    pub last_check: DateTime<Utc>,
    pub uptime_percentage: f64,
    pub error_rate: f64,
    pub slow_queries_count: u64,
    pub issues: Vec<HealthIssue>,
}

impl Default for DatabaseHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            connection_count: 0,
            idle_connections: 0,
            active_connections: 0,
            response_time_ms: 0,
            last_check: Utc::now(),
            uptime_percentage: 100.0,
            error_rate: 0.0,
            slow_queries_count: 0,
            issues: Vec::new(),
        }
    }
}

/// Health issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub details: Option<String>,
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Connection,
    Performance,
    Integrity,
    Capacity,
    Configuration,
}

/// Database health thresholds
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    pub max_response_time_ms: u64,
    pub max_error_rate: f64,
    pub max_connection_usage: f64,
    pub min_uptime_percentage: f64,
    pub slow_query_threshold_ms: u64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000,      // 1 second
            max_error_rate: 0.05,            // 5%
            max_connection_usage: 0.80,      // 80%
            min_uptime_percentage: 99.5,     // 99.5%
            slow_query_threshold_ms: 500,    // 500ms
        }
    }
}

/// Health checker with monitoring capabilities
pub struct HealthChecker {
    pool: SqlitePool,
    thresholds: HealthThresholds,
    health_history: Vec<DatabaseHealth>,
    max_history_size: usize,
}

impl HealthChecker {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            thresholds: HealthThresholds::default(),
            health_history: Vec::new(),
            max_history_size: 1000, // Keep last 1000 health checks
        }
    }

    pub fn with_thresholds(mut self, thresholds: HealthThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Perform comprehensive health check
    pub async fn check_health(&mut self) -> Result<DatabaseHealth> {
        println!("🏥 [HEALTH] Starting comprehensive database health check...");
        let start_time = std::time::Instant::now();

        let mut health = DatabaseHealth::default();
        let mut issues = Vec::new();

        // 1. Basic connectivity test
        match self.check_connectivity().await {
            Ok(response_time) => {
                health.response_time_ms = response_time;
                if response_time > self.thresholds.max_response_time_ms {
                    issues.push(HealthIssue {
                        severity: IssueSeverity::Warning,
                        category: IssueCategory::Performance,
                        message: format!("Database response time is {}ms (threshold: {}ms)",
                            response_time, self.thresholds.max_response_time_ms),
                        details: None,
                        first_detected: Utc::now(),
                        last_detected: Utc::now(),
                        count: 1,
                    });
                }
            }
            Err(e) => {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Connection,
                    message: format!("Database connectivity test failed: {}", e),
                    details: Some(e.to_string()),
                    first_detected: Utc::now(),
                    last_detected: Utc::now(),
                    count: 1,
                });
                health.is_healthy = false;
            }
        }

        // 2. Connection pool health
        if let Ok(pool_stats) = self.check_connection_pool().await {
            health.connection_count = pool_stats.total_connections;
            health.idle_connections = pool_stats.idle_connections;
            health.active_connections = pool_stats.active_connections;

            let usage_percentage = pool_stats.total_connections as f64 / 10.0; // Assuming max 10 connections
            if usage_percentage > self.thresholds.max_connection_usage {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Capacity,
                    message: format!("Connection pool usage is {:.1}% (threshold: {:.1}%)",
                        usage_percentage * 100.0, self.thresholds.max_connection_usage * 100.0),
                    details: Some(format!("Total: {}, Active: {}, Idle: {}",
                        pool_stats.total_connections, pool_stats.active_connections, pool_stats.idle_connections)),
                    first_detected: Utc::now(),
                    last_detected: Utc::now(),
                    count: 1,
                });
            }
        }

        // 3. Database integrity check
        match self.check_integrity().await {
            Ok(is_valid) => {
                if !is_valid {
                    issues.push(HealthIssue {
                        severity: IssueSeverity::Critical,
                        category: IssueCategory::Integrity,
                        message: "Database integrity check failed".to_string(),
                        details: None,
                        first_detected: Utc::now(),
                        last_detected: Utc::now(),
                        count: 1,
                    });
                    health.is_healthy = false;
                }
            }
            Err(e) => {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Integrity,
                    message: format!("Could not perform integrity check: {}", e),
                    details: Some(e.to_string()),
                    first_detected: Utc::now(),
                    last_detected: Utc::now(),
                    count: 1,
                });
            }
        }

        // 4. Performance metrics
        if let Ok(performance_stats) = self.check_performance().await {
            health.slow_queries_count = performance_stats.slow_queries;

            if performance_stats.slow_queries > 0 {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Performance,
                    message: format!("{} slow queries detected", performance_stats.slow_queries),
                    details: Some(format!("Slow query threshold: {}ms", self.thresholds.slow_query_threshold_ms)),
                    first_detected: Utc::now(),
                    last_detected: Utc::now(),
                    count: performance_stats.slow_queries as u32,
                });
            }
        }

        // 5. Database size and capacity
        if let Ok(size_info) = self.check_database_size().await {
            let size_mb = size_info.size_mb;
            if size_mb > 1000.0 { // 1GB
                issues.push(HealthIssue {
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::Capacity,
                    message: format!("Database size is {:.1}MB", size_mb),
                    details: Some("Consider database cleanup or archiving".to_string()),
                    first_detected: Utc::now(),
                    last_detected: Utc::now(),
                    count: 1,
                });
            }
        }

        health.issues = issues;
        health.last_check = Utc::now();

        // Calculate overall health
        if health.is_healthy {
            health.is_healthy = !health.issues.iter().any(|issue| matches!(issue.severity, IssueSeverity::Critical));
        }

        // Add to history
        self.health_history.push(health.clone());
        if self.health_history.len() > self.max_history_size {
            self.health_history.remove(0);
        }

        // Update calculated metrics
        health.uptime_percentage = self.calculate_uptime_percentage();
        health.error_rate = self.calculate_error_rate();

        let total_time = start_time.elapsed().as_millis() as u64;
        println!("✅ [HEALTH] Health check completed in {}ms - Status: {}",
            total_time, if health.is_healthy { "HEALTHY" } else { "UNHEALTHY" });

        if !health.issues.is_empty() {
            println!("⚠️ [HEALTH] Issues detected: {}", health.issues.len());
        }

        Ok(health)
    }

    /// Quick health check for basic connectivity
    pub async fn quick_health_check(&self) -> Result<bool> {
        let start_time = std::time::Instant::now();

        match sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(response_time <= self.thresholds.max_response_time_ms)
            }
            Err(_) => Ok(false)
        }
    }

    /// Get health history
    pub fn get_health_history(&self, limit: Option<usize>) -> &[DatabaseHealth] {
        let end_limit = limit.unwrap_or(self.health_history.len()).min(self.health_history.len());
        &self.health_history[self.health_history.len() - end_limit..]
    }

    /// Get health summary
    pub fn get_health_summary(&self) -> Value {
        let current_health = self.health_history.last().cloned().unwrap_or_default();

        serde_json::json!({
            "current_health": current_health,
            "history_size": self.health_history.len(),
            "thresholds": {
                "max_response_time_ms": self.thresholds.max_response_time_ms,
                "max_error_rate": self.thresholds.max_error_rate,
                "max_connection_usage": self.thresholds.max_connection_usage,
                "min_uptime_percentage": self.thresholds.min_uptime_percentage,
                "slow_query_threshold_ms": self.thresholds.slow_query_threshold_ms
            },
            "checked_at": Utc::now().to_rfc3339()
        })
    }

    // Private helper methods

    async fn check_connectivity(&self) -> Result<u64> {
        let start_time = std::time::Instant::now();

        sqlx::query("SELECT 1 as test")
            .fetch_one(&self.pool)
            .await?;

        Ok(start_time.elapsed().as_millis() as u64)
    }

    async fn check_connection_pool(&self) -> Result<ConnectionPoolStats> {
        // SQLx doesn't expose detailed pool stats directly, so we'll estimate
        // In a real implementation, you might track this manually
        Ok(ConnectionPoolStats {
            total_connections: 5, // Placeholder
            active_connections: 2, // Placeholder
            idle_connections: 3,   // Placeholder
        })
    }

    async fn check_integrity(&self) -> Result<bool> {
        let result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;
        Ok(result == "ok")
    }

    async fn check_performance(&self) -> Result<PerformanceStats> {
        // This would typically track slow queries from application logs
        // For now, we'll return placeholder data
        Ok(PerformanceStats {
            slow_queries: 0, // Would be calculated from actual query logs
        })
    }

    async fn check_database_size(&self) -> Result<DatabaseSizeInfo> {
        let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;

        let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;

        let size_bytes = page_size * page_count;
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

        Ok(DatabaseSizeInfo { size_mb })
    }

    fn calculate_uptime_percentage(&self) -> f64 {
        if self.health_history.is_empty() {
            return 100.0;
        }

        let healthy_count = self.health_history.iter()
            .filter(|h| h.is_healthy)
            .count();

        (healthy_count as f64 / self.health_history.len() as f64) * 100.0
    }

    fn calculate_error_rate(&self) -> f64 {
        if self.health_history.is_empty() {
            return 0.0;
        }

        let total_issues: u32 = self.health_history.iter()
            .map(|h| h.issues.len() as u32)
            .sum();

        let total_checks = self.health_history.len() as u32;
        (total_issues as f64 / total_checks as f64) * 100.0
    }
}

/// Helper structs for health monitoring
#[derive(Debug)]
struct ConnectionPoolStats {
    total_connections: u32,
    active_connections: u32,
    idle_connections: u32,
}

#[derive(Debug)]
struct PerformanceStats {
    slow_queries: u64,
}

#[derive(Debug)]
struct DatabaseSizeInfo {
    size_mb: f64,
}

/// Automated health monitoring service
pub struct HealthMonitor {
    checker: HealthChecker,
    check_interval: std::time::Duration,
    is_monitoring: bool,
}

impl HealthMonitor {
    pub fn new(checker: HealthChecker) -> Self {
        Self {
            checker,
            check_interval: std::time::Duration::from_secs(60), // Check every minute
            is_monitoring: false,
        }
    }

    pub fn with_interval(mut self, interval: std::time::Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(&mut self) -> Result<()> {
        if self.is_monitoring {
            println!("⚠️ [MONITOR] Health monitoring is already running");
            return Ok(());
        }

        println!("🚀 [MONITOR] Starting continuous health monitoring (interval: {:?})", self.check_interval);
        self.is_monitoring = true;

        while self.is_monitoring {
            match self.checker.check_health().await {
                Ok(health) => {
                    if !health.is_healthy {
                        println!("🚨 [MONITOR] CRITICAL: Database health issues detected!");
                        // In a real application, you'd send alerts here
                        for issue in &health.issues {
                            if matches!(issue.severity, IssueSeverity::Critical) {
                                println!("  🚨 {}: {}",
                                    match issue.severity {
                                        IssueSeverity::Critical => "CRITICAL",
                                        IssueSeverity::Warning => "WARNING",
                                        IssueSeverity::Info => "INFO",
                                    },
                                    issue.message
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ [MONITOR] Health check failed: {}", e);
                }
            }

            tokio::time::sleep(self.check_interval).await;
        }

        Ok(())
    }

    /// Stop health monitoring
    pub fn stop_monitoring(&mut self) {
        if self.is_monitoring {
            println!("🛑 [MONITOR] Stopping health monitoring");
            self.is_monitoring = false;
        }
    }

    /// Get current monitoring status
    pub fn is_monitoring(&self) -> bool {
        self.is_monitoring
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_health_checker() -> Result<()> {
        // Note: This test would require a test database setup
        // For now, we'll just test the basic structure

        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let mut checker = HealthChecker::new(pool);

        let health = checker.check_health().await?;
        assert!(health.is_healthy);

        Ok(())
    }
}