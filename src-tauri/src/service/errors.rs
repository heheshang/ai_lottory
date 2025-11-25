//! Service Layer Error Types
//!
//! Defines specific error types for service operations.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceError {
    /// Business logic validation error
    BusinessValidation {
        message: String,
        field: Option<String>,
        code: String,
    },

    /// Service not available
    ServiceUnavailable {
        service_name: String,
        reason: String,
    },

    /// Operation timeout
    OperationTimeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Resource not found
    ResourceNotFound {
        resource_type: String,
        resource_id: String,
    },

    /// Resource conflict
    ResourceConflict {
        resource_type: String,
        resource_id: String,
        conflict_details: HashMap<String, String>,
    },

    /// Permission denied
    PermissionDenied {
        operation: String,
        resource: String,
        user_id: Option<String>,
    },

    /// Quota exceeded
    QuotaExceeded {
        quota_type: String,
        current_usage: u64,
        limit: u64,
    },

    /// Dependency service error
    DependencyError {
        service_name: String,
        error_message: String,
    },

    /// Configuration error
    ConfigurationError {
        message: String,
        field: Option<String>,
    },

    /// Initialization error
    InitializationError {
        service_name: String,
        message: String,
    },

    /// Integration error
    IntegrationError {
        system: String,
        operation: String,
        message: String,
    },

    /// Concurrency error
    ConcurrencyError {
        message: String,
        resource_id: Option<String>,
    },

    /// State error
    StateError {
        current_state: String,
        expected_state: String,
        operation: String,
    },

    /// Rate limit exceeded
    RateLimitExceeded {
        limit: u32,
        window_ms: u64,
        retry_after_ms: u64,
    },

    /// Circuit breaker open
    CircuitBreakerOpen {
        service_name: String,
        failure_count: u32,
        timeout_ms: u64,
    },

    /// Invalid operation
    InvalidOperation {
        operation: String,
        reason: String,
    },

    /// Data consistency error
    DataConsistencyError {
        message: String,
        affected_entities: Vec<String>,
    },

    /// External service error
    ExternalServiceError {
        service_name: String,
        status_code: Option<u16>,
        message: String,
    },

    /// Cache error
    CacheError {
        operation: String,
        message: String,
    },

    /// Event handling error
    EventError {
        event_type: String,
        handler_name: String,
        message: String,
    },

    /// Use case execution error
    UseCaseError {
        use_case_name: String,
        input_data: Option<String>,
        message: String,
    },

    /// Generic service error
    Generic {
        service_name: String,
        operation: String,
        message: String,
        details: Option<HashMap<String, String>>,
    },
}

impl ServiceError {
    /// Create a business validation error
    pub fn business_validation(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::BusinessValidation {
            message: message.into(),
            field: None,
            code: code.into(),
        }
    }

    /// Create a business validation error with field
    pub fn business_validation_with_field(
        message: impl Into<String>,
        field: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self::BusinessValidation {
            message: message.into(),
            field: Some(field.into()),
            code: code.into(),
        }
    }

    /// Create a service unavailable error
    pub fn service_unavailable(service_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            service_name: service_name.into(),
            reason: reason.into(),
        }
    }

    /// Create an operation timeout error
    pub fn operation_timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::OperationTimeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self::ResourceNotFound {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Create a resource conflict error
    pub fn resource_conflict(
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        conflict_details: HashMap<String, String>,
    ) -> Self {
        Self::ResourceConflict {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            conflict_details,
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            resource: resource.into(),
            user_id: None,
        }
    }

    /// Create a permission denied error with user ID
    pub fn permission_denied_with_user(
        operation: impl Into<String>,
        resource: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            resource: resource.into(),
            user_id: Some(user_id.into()),
        }
    }

    /// Create a quota exceeded error
    pub fn quota_exceeded(quota_type: impl Into<String>, current_usage: u64, limit: u64) -> Self {
        Self::QuotaExceeded {
            quota_type: quota_type.into(),
            current_usage,
            limit,
        }
    }

    /// Create a dependency error
    pub fn dependency_error(service_name: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self::DependencyError {
            service_name: service_name.into(),
            error_message: error_message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            field: None,
        }
    }

    /// Create a configuration error with field
    pub fn configuration_error_with_field(
        message: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create an initialization error
    pub fn initialization_error(service_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InitializationError {
            service_name: service_name.into(),
            message: message.into(),
        }
    }

    /// Create an integration error
    pub fn integration_error(system: impl Into<String>, operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::IntegrationError {
            system: system.into(),
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a concurrency error
    pub fn concurrency_error(message: impl Into<String>) -> Self {
        Self::ConcurrencyError {
            message: message.into(),
            resource_id: None,
        }
    }

    /// Create a concurrency error with resource ID
    pub fn concurrency_error_with_id(message: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self::ConcurrencyError {
            message: message.into(),
            resource_id: Some(resource_id.into()),
        }
    }

    /// Create a state error
    pub fn state_error(
        current_state: impl Into<String>,
        expected_state: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        Self::StateError {
            current_state: current_state.into(),
            expected_state: expected_state.into(),
            operation: operation.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(limit: u32, window_ms: u64, retry_after_ms: u64) -> Self {
        Self::RateLimitExceeded {
            limit,
            window_ms,
            retry_after_ms,
        }
    }

    /// Create a circuit breaker open error
    pub fn circuit_breaker_open(service_name: impl Into<String>, failure_count: u32, timeout_ms: u64) -> Self {
        Self::CircuitBreakerOpen {
            service_name: service_name.into(),
            failure_count,
            timeout_ms,
        }
    }

    /// Create an invalid operation error
    pub fn invalid_operation(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidOperation {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Create a data consistency error
    pub fn data_consistency_error(message: impl Into<String>, affected_entities: Vec<String>) -> Self {
        Self::DataConsistencyError {
            message: message.into(),
            affected_entities,
        }
    }

    /// Create an external service error
    pub fn external_service_error(
        service_name: impl Into<String>,
        status_code: Option<u16>,
        message: impl Into<String>,
    ) -> Self {
        Self::ExternalServiceError {
            service_name: service_name.into(),
            status_code,
            message: message.into(),
        }
    }

    /// Create a cache error
    pub fn cache_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CacheError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create an event error
    pub fn event_error(event_type: impl Into<String>, handler_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::EventError {
            event_type: event_type.into(),
            handler_name: handler_name.into(),
            message: message.into(),
        }
    }

    /// Create a use case error
    pub fn use_case_error(use_case_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::UseCaseError {
            use_case_name: use_case_name.into(),
            input_data: None,
            message: message.into(),
        }
    }

    /// Create a use case error with input data
    pub fn use_case_error_with_input(
        use_case_name: impl Into<String>,
        input_data: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::UseCaseError {
            use_case_name: use_case_name.into(),
            input_data: Some(input_data.into()),
            message: message.into(),
        }
    }

    /// Create a generic service error
    pub fn generic(service_name: impl Into<String>, operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Generic {
            service_name: service_name.into(),
            operation: operation.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a generic service error with details
    pub fn generic_with_details(
        service_name: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
        details: HashMap<String, String>,
    ) -> Self {
        Self::Generic {
            service_name: service_name.into(),
            operation: operation.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            ServiceError::ServiceUnavailable { .. } => true,
            ServiceError::OperationTimeout { .. } => true,
            ServiceError::DependencyError { .. } => true,
            ServiceError::IntegrationError { .. } => true,
            ServiceError::ConcurrencyError { .. } => true,
            ServiceError::RateLimitExceeded { .. } => true,
            ServiceError::CircuitBreakerOpen { .. } => true,
            ServiceError::ExternalServiceError { .. } => true,
            ServiceError::CacheError { .. } => true,
            ServiceError::EventError { .. } => false,
            ServiceError::UseCaseError { .. } => false,
            _ => false,
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        match self {
            ServiceError::BusinessValidation { .. } => true,
            ServiceError::ResourceNotFound { .. } => true,
            ServiceError::ResourceConflict { .. } => true,
            ServiceError::PermissionDenied { .. } => true,
            ServiceError::QuotaExceeded { .. } => true,
            ServiceError::InvalidOperation { .. } => true,
            ServiceError::RateLimitExceeded { .. } => true,
            ServiceError::UseCaseError { .. } => true,
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
            ServiceError::BusinessValidation { .. } => "business_validation",
            ServiceError::ServiceUnavailable { .. } => "service_unavailable",
            ServiceError::OperationTimeout { .. } => "timeout",
            ServiceError::ResourceNotFound { .. } => "not_found",
            ServiceError::ResourceConflict { .. } => "conflict",
            ServiceError::PermissionDenied { .. } => "permission",
            ServiceError::QuotaExceeded { .. } => "quota",
            ServiceError::DependencyError { .. } => "dependency",
            ServiceError::ConfigurationError { .. } => "configuration",
            ServiceError::InitializationError { .. } => "initialization",
            ServiceError::IntegrationError { .. } => "integration",
            ServiceError::ConcurrencyError { .. } => "concurrency",
            ServiceError::StateError { .. } => "state",
            ServiceError::RateLimitExceeded { .. } => "rate_limit",
            ServiceError::CircuitBreakerOpen { .. } => "circuit_breaker",
            ServiceError::InvalidOperation { .. } => "invalid_operation",
            ServiceError::DataConsistencyError { .. } => "data_consistency",
            ServiceError::ExternalServiceError { .. } => "external_service",
            ServiceError::CacheError { .. } => "cache",
            ServiceError::EventError { .. } => "event",
            ServiceError::UseCaseError { .. } => "use_case",
            ServiceError::Generic { .. } => "generic",
        }
    }

    /// Get HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            ServiceError::ResourceNotFound { .. } => 404,
            ServiceError::ResourceConflict { .. } => 409,
            ServiceError::PermissionDenied { .. } => 403,
            ServiceError::BusinessValidation { .. } => 400,
            ServiceError::QuotaExceeded { .. } => 429,
            ServiceError::InvalidOperation { .. } => 422,
            ServiceError::RateLimitExceeded { .. } => 429,
            ServiceError::OperationTimeout { .. } => 504,
            ServiceError::ServiceUnavailable { .. } => 503,
            ServiceError::CircuitBreakerOpen { .. } => 503,
            ServiceError::DependencyError { .. } => 502,
            ServiceError::ExternalServiceError { status_code, .. } => status_code.unwrap_or(502),
            _ => 500,
        }
    }

    /// Get retry after milliseconds if applicable
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            ServiceError::RateLimitExceeded { retry_after_ms, .. } => Some(*retry_after_ms),
            ServiceError::CircuitBreakerOpen { timeout_ms, .. } => Some(*timeout_ms),
            ServiceError::OperationTimeout { timeout_ms, .. } => Some(*timeout_ms),
            _ => None,
        }
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::BusinessValidation { message, field, .. } => {
                if let Some(field) = field {
                    write!(f, "Business validation error for field '{}': {}", field, message)
                } else {
                    write!(f, "Business validation error: {}", message)
                }
            }
            ServiceError::ServiceUnavailable { service_name, reason } => {
                write!(f, "Service '{}' unavailable: {}", service_name, reason)
            }
            ServiceError::OperationTimeout { operation, timeout_ms } => {
                write!(f, "Operation '{}' timed out after {}ms", operation, timeout_ms)
            }
            ServiceError::ResourceNotFound { resource_type, resource_id } => {
                write!(f, "Resource '{}' with ID '{}' not found", resource_type, resource_id)
            }
            ServiceError::ResourceConflict { resource_type, resource_id, .. } => {
                write!(f, "Conflict for resource '{}' with ID '{}'", resource_type, resource_id)
            }
            ServiceError::PermissionDenied { operation, resource, user_id } => {
                if let Some(user_id) = user_id {
                    write!(f, "Permission denied for user '{}' to '{}' resource '{}'", user_id, operation, resource)
                } else {
                    write!(f, "Permission denied to '{}' resource '{}'", operation, resource)
                }
            }
            ServiceError::QuotaExceeded { quota_type, current_usage, limit } => {
                write!(f, "Quota '{}' exceeded: {}/{}", quota_type, current_usage, limit)
            }
            ServiceError::DependencyError { service_name, error_message } => {
                write!(f, "Dependency service '{}' error: {}", service_name, error_message)
            }
            ServiceError::ConfigurationError { message, field } => {
                if let Some(field) = field {
                    write!(f, "Configuration error for field '{}': {}", field, message)
                } else {
                    write!(f, "Configuration error: {}", message)
                }
            }
            ServiceError::InitializationError { service_name, message } => {
                write!(f, "Initialization error for service '{}': {}", service_name, message)
            }
            ServiceError::IntegrationError { system, operation, message } => {
                write!(f, "Integration error with '{}' during '{}': {}", system, operation, message)
            }
            ServiceError::ConcurrencyError { message, .. } => {
                write!(f, "Concurrency error: {}", message)
            }
            ServiceError::StateError { current_state, expected_state, operation } => {
                write!(f, "State error during '{}': expected {}, got {}", operation, expected_state, current_state)
            }
            ServiceError::RateLimitExceeded { limit, window_ms, .. } => {
                write!(f, "Rate limit exceeded: {} requests per {}ms", limit, window_ms)
            }
            ServiceError::CircuitBreakerOpen { service_name, failure_count, .. } => {
                write!(f, "Circuit breaker open for service '{}' after {} failures", service_name, failure_count)
            }
            ServiceError::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
            }
            ServiceError::DataConsistencyError { message, .. } => {
                write!(f, "Data consistency error: {}", message)
            }
            ServiceError::ExternalServiceError { service_name, status_code, message } => {
                if let Some(status_code) = status_code {
                    write!(f, "External service '{}' returned {}: {}", service_name, status_code, message)
                } else {
                    write!(f, "External service '{}' error: {}", service_name, message)
                }
            }
            ServiceError::CacheError { operation, message } => {
                write!(f, "Cache error during '{}': {}", operation, message)
            }
            ServiceError::EventError { event_type, handler_name, message } => {
                write!(f, "Event error for '{}' in handler '{}': {}", event_type, handler_name, message)
            }
            ServiceError::UseCaseError { use_case_name, message, .. } => {
                write!(f, "Use case '{}' error: {}", use_case_name, message)
            }
            ServiceError::Generic { service_name, operation, message, .. } => {
                write!(f, "Service error during '{}' in '{}': {}", operation, service_name, message)
            }
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        AppError::Service {
            message: err.to_string(),
            service_name: match err {
                ServiceError::Generic { service_name, .. } => service_name,
                ServiceError::InitializationError { service_name, .. } => service_name,
                ServiceError::ServiceUnavailable { service_name, .. } => service_name,
                ServiceError::DependencyError { service_name, .. } => service_name,
                ServiceError::ExternalServiceError { service_name, .. } => service_name,
                _ => "unknown".to_string(),
            },
        }
    }
}

/// Service error context for enhanced error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceErrorContext {
    pub service_name: String,
    pub operation: String,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: DateTime<chrono::Utc>,
    pub duration_ms: Option<u64>,
    pub attempt_count: u32,
    pub metadata: HashMap<String, String>,
}

impl ServiceErrorContext {
    pub fn new(service_name: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            operation: operation.into(),
            request_id: None,
            user_id: None,
            session_id: None,
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            attempt_count: 1,
            metadata: HashMap::new(),
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_attempt_count(mut self, attempt_count: u32) -> Self {
        self.attempt_count = attempt_count;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Error metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_operation: HashMap<String, u64>,
    pub retryable_errors: u64,
    pub client_errors: u64,
    pub server_errors: u64,
    pub average_resolution_time_ms: f64,
    pub circuit_breaker_trips: u64,
    pub rate_limit_hits: u64,
}

impl ServiceErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            errors_by_operation: HashMap::new(),
            retryable_errors: 0,
            client_errors: 0,
            server_errors: 0,
            average_resolution_time_ms: 0.0,
            circuit_breaker_trips: 0,
            rate_limit_hits: 0,
        }
    }

    pub fn record_error(&mut self, error: &ServiceError, operation: &str, resolution_time_ms: u64) {
        self.total_errors += 1;

        // Category count
        let category = error.category();
        *self.errors_by_category.entry(category.to_string()).or_insert(0) += 1;

        // Operation count
        *self.errors_by_operation.entry(operation.to_string()).or_insert(0) += 1;

        // Error type counts
        if error.is_retryable() {
            self.retryable_errors += 1;
        }

        if error.is_client_error() {
            self.client_errors += 1;
        } else {
            self.server_errors += 1;
        }

        // Special error types
        if matches!(error, ServiceError::CircuitBreakerOpen { .. }) {
            self.circuit_breaker_trips += 1;
        }

        if matches!(error, ServiceError::RateLimitExceeded { .. }) {
            self.rate_limit_hits += 1;
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
}

impl Default for ServiceErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}