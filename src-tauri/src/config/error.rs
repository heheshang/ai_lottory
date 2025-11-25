//! Configuration Error Types
//!
//! Defines specific error types for configuration management.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    /// Configuration value not found
    NotFound {
        key: String,
        provider: Option<String>,
    },

    /// Configuration value is invalid
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },

    /// Configuration source not available
    SourceUnavailable {
        source: String,
        reason: String,
    },

    /// Parse error for configuration file
    ParseError {
        source: String,
        line: Option<u32>,
        column: Option<u32>,
        message: String,
    },

    /// Validation error
    ValidationError {
        key: String,
        value: String,
        rule: String,
        message: String,
    },

    /// Provider error
    ProviderError {
        provider: String,
        operation: String,
        message: String,
    },

    /// Loader error
    LoaderError {
        loader: String,
        source: String,
        message: String,
    },

    /// Watcher error
    WatcherError {
        watcher: String,
        source: String,
        message: String,
    },

    /// Permission denied
    PermissionDenied {
        operation: String,
        resource: String,
    },

    /// Configuration conflict
    Conflict {
        key: String,
        sources: Vec<String>,
        values: Vec<String>,
    },

    /// Encryption/decryption error
    CryptoError {
        operation: String,
        message: String,
    },

    /// Network error for remote configuration
    NetworkError {
        url: String,
        status_code: Option<u16>,
        message: String,
    },

    /// Timeout error
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Quota exceeded
    QuotaExceeded {
        resource: String,
        current_usage: u64,
        limit: u64,
    },

    /// Circular dependency detected
    CircularDependency {
        keys: Vec<String>,
    },

    /// Unsupported operation
    UnsupportedOperation {
        operation: String,
        provider: String,
        reason: String,
    },

    /// Invalid configuration structure
    InvalidStructure {
        message: String,
        path: Option<String>,
    },

    /// Schema validation error
    SchemaError {
        schema: String,
        message: String,
        path: String,
    },

    /// Version mismatch
    VersionMismatch {
        expected: String,
        actual: String,
        component: String,
    },

    /// Generic configuration error
    Generic {
        message: String,
        component: Option<String>,
        details: Option<HashMap<String, String>>,
    },
}

impl ConfigError {
    /// Create a not found error
    pub fn not_found(key: impl Into<String>) -> Self {
        Self::NotFound {
            key: key.into(),
            provider: None,
        }
    }

    /// Create a not found error with provider
    pub fn not_found_with_provider(key: impl Into<String>, provider: impl Into<String>) -> Self {
        Self::NotFound {
            key: key.into(),
            provider: Some(provider.into()),
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(key: impl Into<String>, value: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            key: key.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a source unavailable error
    pub fn source_unavailable(source: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::SourceUnavailable {
            source: source.into(),
            reason: reason.into(),
        }
    }

    /// Create a parse error
    pub fn parse_error(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError {
            source: source.into(),
            line: None,
            column: None,
            message: message.into(),
        }
    }

    /// Create a parse error with location
    pub fn parse_error_with_location(
        source: impl Into<String>,
        line: u32,
        column: u32,
        message: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            source: source.into(),
            line: Some(line),
            column: Some(column),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error(
        key: impl Into<String>,
        value: impl Into<String>,
        rule: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ValidationError {
            key: key.into(),
            value: value.into(),
            rule: rule.into(),
            message: message.into(),
        }
    }

    /// Create a provider error
    pub fn provider_error(provider: impl Into<String>, operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProviderError {
            provider: provider.into(),
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a loader error
    pub fn loader_error(loader: impl Into<String>, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::LoaderError {
            loader: loader.into(),
            source: source.into(),
            message: message.into(),
        }
    }

    /// Create a watcher error
    pub fn watcher_error(watcher: impl Into<String>, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WatcherError {
            watcher: watcher.into(),
            source: source.into(),
            message: message.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            resource: resource.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(key: impl Into<String>, sources: Vec<String>, values: Vec<String>) -> Self {
        Self::Conflict {
            key: key.into(),
            sources,
            values,
        }
    }

    /// Create a crypto error
    pub fn crypto_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CryptoError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error(url: impl Into<String>, message: impl Into<String>) -> Self {
        Self::NetworkError {
            url: url.into(),
            status_code: None,
            message: message.into(),
        }
    }

    /// Create a network error with status code
    pub fn network_error_with_status(url: impl Into<String>, status_code: u16, message: impl Into<String>) -> Self {
        Self::NetworkError {
            url: url.into(),
            status_code: Some(status_code),
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

    /// Create a quota exceeded error
    pub fn quota_exceeded(resource: impl Into<String>, current_usage: u64, limit: u64) -> Self {
        Self::QuotaExceeded {
            resource: resource.into(),
            current_usage,
            limit,
        }
    }

    /// Create a circular dependency error
    pub fn circular_dependency(keys: Vec<String>) -> Self {
        Self::CircularDependency { keys }
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation(operation: impl Into<String>, provider: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
            provider: provider.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure {
            message: message.into(),
            path: None,
        }
    }

    /// Create an invalid structure error with path
    pub fn invalid_structure_with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::InvalidStructure {
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Create a schema error
    pub fn schema_error(schema: impl Into<String>, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::SchemaError {
            schema: schema.into(),
            message: message.into(),
            path: path.into(),
        }
    }

    /// Create a version mismatch error
    pub fn versionMismatch(expected: impl Into<String>, actual: impl Into<String>, component: impl Into<String>) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            actual: actual.into(),
            component: component.into(),
        }
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            component: None,
            details: None,
        }
    }

    /// Create a generic error with component
    pub fn generic_with_component(message: impl Into<String>, component: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            component: Some(component.into()),
            details: None,
        }
    }

    /// Create a generic error with details
    pub fn generic_with_details(
        message: impl Into<String>,
        details: HashMap<String, String>,
    ) -> Self {
        Self::Generic {
            message: message.into(),
            component: None,
            details: Some(details),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            ConfigError::SourceUnavailable { .. } => true,
            ConfigError::NetworkError { .. } => true,
            ConfigError::Timeout { .. } => true,
            ConfigError::ProviderError { .. } => true,
            ConfigError::LoaderError { .. } => true,
            ConfigError::WatcherError { .. } => true,
            _ => false,
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        match self {
            ConfigError::NotFound { .. } => true,
            ConfigError::InvalidValue { .. } => true,
            ConfigError::ValidationError { .. } => true,
            ConfigError::PermissionDenied { .. } => true,
            ConfigError::Conflict { .. } => true,
            ConfigError::UnsupportedOperation { .. } => true,
            ConfigError::InvalidStructure { .. } => true,
            ConfigError::SchemaError { .. } => true,
            ConfigError::CircularDependency { .. } => true,
            _ => false,
        }
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        match self {
            ConfigError::NotFound { .. } => "not_found",
            ConfigError::InvalidValue { .. } => "invalid_value",
            ConfigError::SourceUnavailable { .. } => "source_unavailable",
            ConfigError::ParseError { .. } => "parse_error",
            ConfigError::ValidationError { .. } => "validation_error",
            ConfigError::ProviderError { .. } => "provider_error",
            ConfigError::LoaderError { .. } => "loader_error",
            ConfigError::WatcherError { .. } => "watcher_error",
            ConfigError::PermissionDenied { .. } => "permission_denied",
            ConfigError::Conflict { .. } => "conflict",
            ConfigError::CryptoError { .. } => "crypto_error",
            ConfigError::NetworkError { .. } => "network_error",
            ConfigError::Timeout { .. } => "timeout",
            ConfigError::QuotaExceeded { .. } => "quota_exceeded",
            ConfigError::CircularDependency { .. } => "circular_dependency",
            ConfigError::UnsupportedOperation { .. } => "unsupported_operation",
            ConfigError::InvalidStructure { .. } => "invalid_structure",
            ConfigError::SchemaError { .. } => "schema_error",
            ConfigError::VersionMismatch { .. } => "version_mismatch",
            ConfigError::Generic { .. } => "generic",
        }
    }

    /// Get HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            ConfigError::NotFound { .. } => 404,
            ConfigError::InvalidValue { .. } => 400,
            ConfigError::ValidationError { .. } => 422,
            ConfigError::PermissionDenied { .. } => 403,
            ConfigError::Conflict { .. } => 409,
            ConfigError::UnsupportedOperation { .. } => 501,
            ConfigError::QuotaExceeded { .. } => 429,
            ConfigError::NetworkError { status_code, .. } => status_code.unwrap_or(502),
            ConfigError::Timeout { .. } => 504,
            ConfigError::SourceUnavailable { .. } => 503,
            _ => 500,
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ConfigError::NotFound { .. } => ErrorSeverity::Warning,
            ConfigError::ValidationError { .. } => ErrorSeverity::Error,
            ConfigError::InvalidValue { .. } => ErrorSeverity::Error,
            ConfigError::PermissionDenied { .. } => ErrorSeverity::Error,
            ConfigError::SourceUnavailable { .. } => ErrorSeverity::Critical,
            ConfigError::NetworkError { .. } => ErrorSeverity::Error,
            ConfigError::Timeout { .. } => ErrorSeverity::Warning,
            ConfigError::CryptoError { .. } => ErrorSeverity::Critical,
            ConfigError::CircularDependency { .. } => ErrorSeverity::Critical,
            ConfigError::SchemaError { .. } => ErrorSeverity::Error,
            ConfigError::VersionMismatch { .. } => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound { key, provider } => {
                if let Some(provider) = provider {
                    write!(f, "Configuration key '{}' not found in provider '{}'", key, provider)
                } else {
                    write!(f, "Configuration key '{}' not found", key)
                }
            }
            ConfigError::InvalidValue { key, value, reason } => {
                write!(f, "Invalid value '{}' for key '{}': {}", value, key, reason)
            }
            ConfigError::SourceUnavailable { source, reason } => {
                write!(f, "Configuration source '{}' unavailable: {}", source, reason)
            }
            ConfigError::ParseError { source, line, column, message } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "Parse error in '{}' at line {}, column {}: {}", source, line, column, message)
                } else {
                    write!(f, "Parse error in '{}': {}", source, message)
                }
            }
            ConfigError::ValidationError { key, value, rule, message } => {
                write!(f, "Validation failed for key '{}' with value '{}': {} ({})", key, value, message, rule)
            }
            ConfigError::ProviderError { provider, operation, message } => {
                write!(f, "Provider '{}' error during '{}': {}", provider, operation, message)
            }
            ConfigError::LoaderError { loader, source, message } => {
                write!(f, "Loader '{}' error for source '{}': {}", loader, source, message)
            }
            ConfigError::WatcherError { watcher, source, message } => {
                write!(f, "Watcher '{}' error for source '{}': {}", watcher, source, message)
            }
            ConfigError::PermissionDenied { operation, resource } => {
                write!(f, "Permission denied to '{}' resource '{}'", operation, resource)
            }
            ConfigError::Conflict { key, sources, values } => {
                write!(f, "Configuration conflict for key '{}' between sources: {:?}", key, sources)
            }
            ConfigError::CryptoError { operation, message } => {
                write!(f, "Crypto error during '{}': {}", operation, message)
            }
            ConfigError::NetworkError { url, status_code, message } => {
                if let Some(status_code) = status_code {
                    write!(f, "Network error for '{}' ({}): {}", url, status_code, message)
                } else {
                    write!(f, "Network error for '{}': {}", url, message)
                }
            }
            ConfigError::Timeout { operation, timeout_ms } => {
                write!(f, "Operation '{}' timed out after {}ms", operation, timeout_ms)
            }
            ConfigError::QuotaExceeded { resource, current_usage, limit } => {
                write!(f, "Quota exceeded for '{}': {}/{}", resource, current_usage, limit)
            }
            ConfigError::CircularDependency { keys } => {
                write!(f, "Circular dependency detected: {:?}", keys)
            }
            ConfigError::UnsupportedOperation { operation, provider, reason } => {
                write!(f, "Unsupported operation '{}' for provider '{}': {}", operation, provider, reason)
            }
            ConfigError::InvalidStructure { message, path } => {
                if let Some(path) = path {
                    write!(f, "Invalid configuration structure at '{}': {}", path, message)
                } else {
                    write!(f, "Invalid configuration structure: {}", message)
                }
            }
            ConfigError::SchemaError { schema, message, path } => {
                write!(f, "Schema error for '{}' at '{}': {}", schema, path, message)
            }
            ConfigError::VersionMismatch { expected, actual, component } => {
                write!(f, "Version mismatch for '{}': expected {}, got {}", component, expected, actual)
            }
            ConfigError::Generic { message, component, .. } => {
                if let Some(component) = component {
                    write!(f, "Configuration error in '{}': {}", component, message)
                } else {
                    write!(f, "Configuration error: {}", message)
                }
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config {
            message: err.to_string(),
            field: match err {
                ConfigError::InvalidValue { key, .. } => key,
                ConfigError::ValidationError { key, .. } => key,
                ConfigError::NotFound { key, .. } => key,
                _ => "config".to_string(),
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

/// Configuration error context for enhanced error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigErrorContext {
    pub component: String,
    pub operation: String,
    pub key: Option<String>,
    pub source: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ConfigErrorContext {
    pub fn new(component: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            key: None,
            source: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Error metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_component: HashMap<String, u64>,
    pub errors_by_source: HashMap<String, u64>,
    pub retryable_errors: u64,
    pub client_errors: u64,
    pub server_errors: u64,
    pub errors_by_severity: HashMap<String, u64>,
    pub average_resolution_time_ms: f64,
}

impl ConfigErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            errors_by_component: HashMap::new(),
            errors_by_source: HashMap::new(),
            retryable_errors: 0,
            client_errors: 0,
            server_errors: 0,
            errors_by_severity: HashMap::new(),
            average_resolution_time_ms: 0.0,
        }
    }

    pub fn record_error(&mut self, error: &ConfigError, component: &str, source: Option<&str>, resolution_time_ms: u64) {
        self.total_errors += 1;

        // Category count
        let category = error.category();
        *self.errors_by_category.entry(category.to_string()).or_insert(0) += 1;

        // Component count
        *self.errors_by_component.entry(component.to_string()).or_insert(0) += 1;

        // Source count
        if let Some(source) = source {
            *self.errors_by_source.entry(source.to_string()).or_insert(0) += 1;
        }

        // Error type counts
        if error.is_retryable() {
            self.retryable_errors += 1;
        }

        if error.is_client_error() {
            self.client_errors += 1;
        } else {
            self.server_errors += 1;
        }

        // Severity count
        let severity = format!("{:?}", error.severity());
        *self.errors_by_severity.entry(severity).or_insert(0) += 1;

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
        if self.total_errors > 0 {
            self.errors_by_severity
                .get("Critical")
                .unwrap_or(&0)
                .to_owned() as f64 / self.total_errors as f64
        } else {
            0.0
        }
    }
}

impl Default for ConfigErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}