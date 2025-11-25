//! Error logging infrastructure for centralized error tracking

use crate::error::{AppError, ErrorContext, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Error logger for centralized error tracking
pub struct ErrorLogger {
    log_file: Arc<RwLock<Option<File>>>,
    max_file_size: u64,
    log_dir: PathBuf,
}

impl ErrorLogger {
    /// Create a new error logger
    pub async fn new<P: Into<PathBuf>>(log_dir: P) -> Result<Self, AppError> {
        let log_dir = log_dir.into();

        // Create log directory if it doesn't exist
        tokio::fs::create_dir_all(&log_dir).await
            .map_err(|e| AppError::Io {
                message: format!("Failed to create log directory: {}", e),
            })?;

        let logger = Self {
            log_file: Arc::new(RwLock::new(None)),
            max_file_size: 10 * 1024 * 1024, // 10MB
            log_dir,
        };

        // Initialize log file
        logger.rotate_log_file().await?;

        Ok(logger)
    }

    /// Log an error with context
    pub async fn log_error(&self, error: &AppError, context: &ErrorContext) -> Result<(), AppError> {
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            error_code: error.code().to_string(),
            error_message: error.user_message(),
            severity: context.severity,
            error_id: context.error_id.clone(),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            operation: context.operation.clone(),
            request_id: context.request_id.clone(),
            metadata: context.metadata.clone(),
            stack_trace: self.get_stack_trace(),
        };

        let log_line = serde_json::to_string(&log_entry)
            .map_err(|e| AppError::Internal {
                message: format!("Failed to serialize log entry: {}", e),
            })?;

        // Write to log file
        {
            let mut log_file = self.log_file.write().await;
            if let Some(file) = log_file.as_mut() {
                file.write_all(log_line.as_bytes()).await
                    .map_err(|e| AppError::Io {
                        message: format!("Failed to write to log file: {}", e),
                    })?;
                file.write_all(b"\n").await
                    .map_err(|e| AppError::Io {
                        message: format!("Failed to write newline to log file: {}", e),
                    })?;
                file.flush().await
                    .map_err(|e| AppError::Io {
                        message: format!("Failed to flush log file: {}", e),
                    })?;
            }
        }

        // Check if we need to rotate the log file
        self.check_file_size().await?;

        // Also log to tracing
        match log_entry.severity {
            ErrorSeverity::Critical => {
                tracing::error!(
                    error_id = %log_entry.error_id,
                    error_code = %log_entry.error_code,
                    user_id = ?log_entry.user_id,
                    operation = ?log_entry.operation,
                    "{}", log_entry.error_message
                );
            }
            ErrorSeverity::High => {
                tracing::error!(
                    error_id = %log_entry.error_id,
                    error_code = %log_entry.error_code,
                    "{}", log_entry.error_message
                );
            }
            ErrorSeverity::Medium => {
                tracing::warn!(
                    error_id = %log_entry.error_id,
                    "{}", log_entry.error_message
                );
            }
            ErrorSeverity::Low => {
                tracing::info!(
                    error_id = %log_entry.error_id,
                    "{}", log_entry.error_message
                );
            }
        }

        Ok(())
    }

    /// Get recent errors for analysis
    pub async fn get_recent_errors(&self, limit: usize) -> Result<Vec<LogEntry>, AppError> {
        let log_file_path = self.get_current_log_path();

        let content = tokio::fs::read_to_string(&log_file_path).await
            .map_err(|e| AppError::Io {
                message: format!("Failed to read log file: {}", e),
            })?;

        let mut entries = Vec::new();
        let lines: Vec<&str> = content.lines().rev().take(limit).collect();

        for line in lines {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get error statistics for a time period
    pub async fn get_error_stats(&self, since: DateTime<Utc>) -> Result<ErrorStats, AppError> {
        let entries = self.get_recent_errors(10000).await?; // Get up to 10k entries

        let mut stats = ErrorStats::default();
        let mut operation_counts = std::collections::HashMap::new();

        for entry in entries {
            if entry.timestamp > since {
                stats.total_errors += 1;

                match entry.severity {
                    ErrorSeverity::Critical => stats.critical += 1,
                    ErrorSeverity::High => stats.high += 1,
                    ErrorSeverity::Medium => stats.medium += 1,
                    ErrorSeverity::Low => stats.low += 1,
                }

                // Count errors by operation
                if let Some(ref operation) = entry.operation {
                    *operation_counts.entry(operation.clone()).or_insert(0) += 1;
                }
            }
        }

        stats.operation_counts = operation_counts;
        Ok(stats)
    }

    /// Rotate log file if it's too large
    async fn rotate_log_file(&self) -> Result<(), AppError> {
        let log_file_path = self.get_current_log_path();

        // Check if file exists and its size
        if log_file_path.exists() {
            let metadata = tokio::fs::metadata(&log_file_path).await
                .map_err(|e| AppError::Io {
                    message: format!("Failed to get log file metadata: {}", e),
                })?;

            if metadata.len() > self.max_file_size {
                // Move current log to archive
                let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                let archive_path = log_file_path.with_extension(format!("log.{}", timestamp));

                tokio::fs::rename(&log_file_path, &archive_path).await
                    .map_err(|e| AppError::Io {
                        message: format!("Failed to archive log file: {}", e),
                    })?;
            }
        }

        // Create new log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .await
            .map_err(|e| AppError::Io {
                message: format!("Failed to open log file: {}", e),
            })?;

        *self.log_file.write().await = Some(file);
        Ok(())
    }

    /// Check if file needs rotation
    async fn check_file_size(&self) -> Result<(), AppError> {
        let log_file_path = self.get_current_log_path();

        if log_file_path.exists() {
            let metadata = tokio::fs::metadata(&log_file_path).await
                .map_err(|e| AppError::Io {
                    message: format!("Failed to get log file metadata: {}", e),
                })?;

            if metadata.len() > self.max_file_size {
                self.rotate_log_file().await?;
            }
        }

        Ok(())
    }

    /// Get current log file path
    fn get_current_log_path(&self) -> PathBuf {
        self.log_dir.join("errors.log")
    }

    /// Get current stack trace
    fn get_stack_trace(&self) -> Option<String> {
        std::backtrace::Backtrace::capture()
            .to_string()
            .into()
    }
}

/// Log entry structure
#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    error_code: String,
    error_message: String,
    severity: ErrorSeverity,
    error_id: String,
    user_id: Option<String>,
    session_id: Option<String>,
    operation: Option<String>,
    request_id: Option<String>,
    metadata: serde_json::Value,
    stack_trace: Option<String>,
}

/// Error statistics with operation breakdown
#[derive(Debug, Default)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub operation_counts: std::collections::HashMap<String, usize>,
}

impl ErrorStats {
    pub fn critical_rate(&self) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            (self.critical as f64 / self.total_errors as f64) * 100.0
        }
    }

    pub fn get_top_operations(&self, limit: usize) -> Vec<(String, usize)> {
        let mut ops: Vec<_> = self.operation_counts.iter().collect();
        ops.sort_by(|a, b| b.1.cmp(a.1));
        ops.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }
}