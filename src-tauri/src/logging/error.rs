//! Logging Error Types
//!
//! Defines specific error types for logging operations.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Logging-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogError {
    /// Logger initialization error
    InitializationError {
        logger_name: String,
        message: String,
    },

    /// Appender error
    AppenderError {
        appender_name: String,
        operation: String,
        message: String,
    },

    /// Formatter error
    FormatterError {
        formatter_name: String,
        message: String,
    },

    /// Filter error
    FilterError {
        filter_name: String,
        message: String,
    },

    /// Context provider error
    ContextError {
        operation: String,
        message: String,
    },

    /// Metrics collection error
    MetricsError {
        operation: String,
        message: String,
    },

    /// Configuration error
    ConfigurationError {
        parameter: String,
        message: String,
    },

    /// I/O error
    IoError {
        operation: String,
        path: Option<String>,
        message: String,
    },

    /// Serialization error
    SerializationError {
        operation: String,
        message: String,
    },

    /// Buffer overflow error
    BufferOverflow {
        buffer_name: String,
        size: usize,
        capacity: usize,
    },

    /// Channel error
    ChannelError {
        operation: String,
        message: String,
    },

    /// Timeout error
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Permission denied error
    PermissionDenied {
        operation: String,
        resource: String,
    },

    /// Resource exhausted error
    ResourceExhausted {
        resource_type: String,
        current_usage: u64,
        limit: u64,
    },

    /// Invalid log level error
    InvalidLogLevel {
        level: String,
        valid_levels: Vec<String>,
    },

    /// Invalid format error
    InvalidFormat {
        format: String,
        message: String,
    },

    /// Validation error
    ValidationError {
        field: String,
        value: String,
        reason: String,
    },

    /// Log record too large error
    RecordTooLarge {
        size_bytes: usize,
        max_size_bytes: usize,
    },

    /// Async operation error
    AsyncError {
        operation: String,
        message: String,
    },

    /// Shutdown error
    ShutdownError {
        logger_name: String,
        message: String,
    },

    /// Generic logging error
    Generic {
        component: String,
        operation: String,
        message: String,
        details: Option<HashMap<String, String>>,
    },
}

impl LogError {
    /// Create an initialization error
    pub fn initialization_error(logger_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InitializationError {
            logger_name: logger_name.into(),
            message: message.into(),
        }
    }

    /// Create an appender error
    pub fn appender_error(
        appender_name: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::AppenderError {
            appender_name: appender_name.into(),
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a formatter error
    pub fn formatter_error(formatter_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FormatterError {
            formatter_name: formatter_name.into(),
            message: message.into(),
        }
    }

    /// Create a filter error
    pub fn filter_error(filter_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FilterError {
            filter_name: filter_name.into(),
            message: message.into(),
        }
    }

    /// Create a context error
    pub fn context_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ContextError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a metrics error
    pub fn metrics_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::MetricsError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// Create an I/O error
    pub fn io_error(operation: impl Into<String>, path: Option<String>, message: impl Into<String>) -> Self {
        Self::IoError {
            operation: operation.into(),
            path,
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::SerializationError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a buffer overflow error
    pub fn buffer_overflow(buffer_name: impl Into<String>, size: usize, capacity: usize) -> Self {
        Self::BufferOverflow {
            buffer_name: buffer_name.into(),
            size,
            capacity,
        }
    }

    /// Create a channel error
    pub fn channel_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ChannelError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            resource: resource.into(),
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(resource_type: impl Into<String>, current_usage: u64, limit: u64) -> Self {
        Self::ResourceExhausted {
            resource_type: resource_type.into(),
            current_usage,
            limit,
        }
    }

    /// Create an invalid log level error
    pub fn invalid_log_level(level: impl Into<String>, valid_levels: Vec<String>) -> Self {
        Self::InvalidLogLevel {
            level: level.into(),
            valid_levels,
        }
    }

    /// Create an invalid format error
    pub fn invalid_format(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidFormat {
            format: format.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error(
        field: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            field: field.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a record too large error
    pub fn record_too_large(size_bytes: usize, max_size_bytes: usize) -> Self {
        Self::RecordTooLarge {
            size_bytes,
            max_size_bytes,
        }
    }

    /// Create an async operation error
    pub fn async_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AsyncError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a shutdown error
    pub fn shutdown_error(logger_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ShutdownError {
            logger_name: logger_name.into(),
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn generic(
        component: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Generic {
            component: component.into(),
            operation: operation.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a generic error with details
    pub fn generic_with_details(
        component: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
        details: HashMap<String, String>,
    ) -> Self {
        Self::Generic {
            component: component.into(),
            operation: operation.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            LogError::IoError { .. } => true,
            LogError::ChannelError { .. } => true,
            LogError::Timeout { .. } => true,
            LogError::AsyncError { .. } => true,
            LogError::ResourceExhausted { .. } => false,
            LogError::PermissionDenied { .. } => false,
            _ => false,
        }
    }

    /// Check if this is a client error
    pub fn is_client_error(&self) -> bool {
        match self {
            LogError::ValidationError { .. } => true,
            LogError::InvalidLogLevel { .. } => true,
            LogError::InvalidFormat { .. } => true,
            LogError::RecordTooLarge { .. } => true,
            LogError::ConfigurationError { .. } => true,
            _ => false,
        }
    }

    /// Check if this is a server error
    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        match self {
            LogError::InitializationError { .. } => "initialization",
            LogError::AppenderError { .. } => "appender",
            LogError::FormatterError { .. } => "formatter",
            LogError::FilterError { .. } => "filter",
            LogError::ContextError { .. } => "context",
            LogError::MetricsError { .. } => "metrics",
            LogError::ConfigurationError { .. } => "configuration",
            LogError::IoError { .. } => "io",
            LogError::SerializationError { .. } => "serialization",
            LogError::BufferOverflow { .. } => "buffer",
            LogError::ChannelError { .. } => "channel",
            LogError::Timeout { .. } => "timeout",
            LogError::PermissionDenied { .. } => "permission",
            LogError::ResourceExhausted { .. } => "resource",
            LogError::InvalidLogLevel { .. } => "level",
            LogError::InvalidFormat { .. } => "format",
            LogError::ValidationError { .. } => "validation",
            LogError::RecordTooLarge { .. } => "size",
            LogError::AsyncError { .. } => "async",
            LogError::ShutdownError { .. } => "shutdown",
            LogError::Generic { .. } => "generic",
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            LogError::InitializationError { .. } => ErrorSeverity::Critical,
            LogError::AppenderError { .. } => ErrorSeverity::Error,
            LogError::FormatterError { .. } => ErrorSeverity::Error,
            LogError::FilterError { .. } => ErrorSeverity::Error,
            LogError::ContextError { .. } => ErrorSeverity::Error,
            LogError::MetricsError { .. } => ErrorSeverity::Warning,
            LogError::ConfigurationError { .. } => ErrorSeverity::Error,
            LogError::IoError { .. } => ErrorSeverity::Error,
            LogError::SerializationError { .. } => ErrorSeverity::Error,
            LogError::BufferOverflow { .. } => ErrorSeverity::Warning,
            LogError::ChannelError { .. } => ErrorSeverity::Error,
            LogError::Timeout { .. } => ErrorSeverity::Warning,
            LogError::PermissionDenied { .. } => ErrorSeverity::Error,
            LogError::ResourceExhausted { .. } => ErrorSeverity::Critical,
            LogError::InvalidLogLevel { .. } => ErrorSeverity::Error,
            LogError::InvalidFormat { .. } => ErrorSeverity::Error,
            LogError::ValidationError { .. } => ErrorSeverity::Error,
            LogError::RecordTooLarge { .. } => ErrorSeverity::Warning,
            LogError::AsyncError { .. } => ErrorSeverity::Error,
            LogError::ShutdownError { .. } => ErrorSeverity::Error,
            LogError::Generic { .. } => ErrorSeverity::Error,
        }
    }
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::InitializationError { logger_name, message } => {
                write!(f, "Initialization error for logger '{}': {}", logger_name, message)
            }
            LogError::AppenderError { appender_name, operation, message } => {
                write!(f, "Appender '{}' error during '{}': {}", appender_name, operation, message)
            }
            LogError::FormatterError { formatter_name, message } => {
                write!(f, "Formatter '{}' error: {}", formatter_name, message)
            }
            LogError::FilterError { filter_name, message } => {
                write!(f, "Filter '{}' error: {}", filter_name, message)
            }
            LogError::ContextError { operation, message } => {
                write!(f, "Context error during '{}': {}", operation, message)
            }
            LogError::MetricsError { operation, message } => {
                write!(f, "Metrics error during '{}': {}", operation, message)
            }
            LogError::ConfigurationError { parameter, message } => {
                write!(f, "Configuration error for parameter '{}': {}", parameter, message)
            }
            LogError::IoError { operation, path, message } => {
                if let Some(path) = path {
                    write!(f, "I/O error during '{}' on '{}': {}", operation, path, message)
                } else {
                    write!(f, "I/O error during '{}': {}", operation, message)
                }
            }
            LogError::SerializationError { operation, message } => {
                write!(f, "Serialization error during '{}': {}", operation, message)
            }
            LogError::BufferOverflow { buffer_name, size, capacity } => {
                write!(f, "Buffer overflow in '{}': {} bytes exceeds capacity {}", buffer_name, size, capacity)
            }
            LogError::ChannelError { operation, message } => {
                write!(f, "Channel error during '{}': {}", operation, message)
            }
            LogError::Timeout { operation, timeout_ms } => {
                write!(f, "Timeout during '{}' after {}ms", operation, timeout_ms)
            }
            LogError::PermissionDenied { operation, resource } => {
                write!(f, "Permission denied to '{}' on '{}'", operation, resource)
            }
            LogError::ResourceExhausted { resource_type, current_usage, limit } => {
                write!(f, "Resource '{}' exhausted: {}/{}", resource_type, current_usage, limit)
            }
            LogError::InvalidLogLevel { level, valid_levels } => {
                write!(f, "Invalid log level '{}'. Valid levels: {:?}", level, valid_levels.join(", "))
            }
            LogError::InvalidFormat { format, message } => {
                write!(f, "Invalid format '{}': {}", format, message)
            }
            LogError::ValidationError { field, value, reason } => {
                write!(f, "Validation error for field '{}' with value '{}': {}", field, value, reason)
            }
            LogError::RecordTooLarge { size_bytes, max_size_bytes } => {
                write!(f, "Log record too large: {} bytes exceeds maximum {}", size_bytes, max_size_bytes)
            }
            LogError::AsyncError { operation, message } => {
                write!(f, "Async error during '{}': {}", operation, message)
            }
            LogError::ShutdownError { logger_name, message } => {
                write!(f, "Shutdown error for logger '{}': {}", logger_name, message)
            }
            LogError::Generic { component, operation, message, .. } => {
                write!(f, "Logging error in '{}' during '{}': {}", component, operation, message)
            }
        }
    }
}

impl std::error::Error for LogError {}

impl From<LogError> for AppError {
    fn from(err: LogError) -> Self {
        AppError::Logging {
            message: err.to_string(),
            field: match err {
                LogError::ConfigurationError { parameter, .. } => parameter.clone(),
                LogError::ValidationError { field, .. } => field.clone(),
                _ => "logging".to_string(),
            },
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Log error context for enhanced error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogErrorContext {
    pub component: String,
    pub operation: String,
    pub logger_name: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub thread_id: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl LogErrorContext {
    pub fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            logger_name: None,
            timestamp: chrono::Utc::now(),
            thread_id: None,
            request_id: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_logger_name(mut self, logger_name: impl Into<String>) -> Self {
        self.logger_name = Some(logger_name.into());
        self
    }

    pub fn with_thread_id(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Error metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_component: HashMap<String, u64>,
    pub errors_by_severity: HashMap<String, u64>,
    pub retryable_errors: u64,
    pub client_errors: u64,
    pub server_errors: u64,
    pub average_resolution_time_ms: f64,
}

impl LogErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            errors_by_component: HashMap::new(),
            errors_by_severity: HashMap::new(),
            retryable_errors: 0,
            client_errors: 0,
            server_errors: 0,
            average_resolution_time_ms: 0.0,
        }
    }

    pub fn record_error(&mut self, error: &LogError, component: &str, resolution_time_ms: u64) {
        self.total_errors += 1;

        // Category count
        let category = error.category();
        *self.errors_by_category.entry(category.to_string()).or_insert(0) += 1;

        // Component count
        *self.errors_by_component.entry(component.to_string()).or_insert(0) += 1;

        // Severity count
        let severity = format!("{:?}", error.severity());
        *self.errors_by_severity.entry(severity).or_insert(0) += 1;

        // Error type counts
        if error.is_retryable() {
            self.retryable_errors += 1;
        }

        if error.is_client_error() {
            self.client_errors += 1;
        } else {
            self.server_errors += 1;
        }

        // Update average resolution time
        let total_time = self.average_resolution_time_ms * (self.total_errors - 1) as f64 + resolution_time_ms as f64;
        self.average_resolution_time_ms = total_time / self.total_errors as f64;
    }

    pub fn get_error_rate(&self, total_operations: u64) -> f64 {
        if total_operations > 0 {
            self.total_errors as f64 / total_operations as f64
        } else {
            0.0
        }
    }

    pub fn get_retry_rate(&self) -> f64 {
        if self.total_errors > 0 {
            self.retryable_errors as f64 / self.total_errors as f64
        } else {
            0.0
        }
    }

    pub fn get_critical_error_rate(&self) -> f64 {
        self.errors_by_severity
            .get("Critical")
            .unwrap_or(&0)
            .to_owned() as f64 / self.total_errors as f64
    }
}

impl Default for LogErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}