//! Repository Error Types
//!
//! Defines specific error types for repository operations.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repository-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryError {
    /// Database connection or query error
    Database {
        message: String,
        query: Option<String>,
        code: Option<String>,
    },

    /// Entity not found
    NotFound {
        entity_type: String,
        id: String,
    },

    /// Duplicate entity violation
    Duplicate {
        entity_type: String,
        field: String,
        value: String,
    },

    /// Validation error
    Validation {
        message: String,
        field: String,
        value: Option<String>,
    },

    /// Transaction error
    Transaction {
        message: String,
        transaction_id: Option<String>,
    },

    /// Constraint violation
    ConstraintViolation {
        constraint: String,
        message: String,
    },

    /// Permission denied
    PermissionDenied {
        operation: String,
        resource: String,
    },

    /// Timeout error
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// Concurrency error
    Concurrency {
        message: String,
        entity_id: Option<String>,
    },

    /// Cache error
    Cache {
        message: String,
        operation: String,
    },

    /// Serialization/deserialization error
    Serialization {
        message: String,
        entity_type: String,
    },

    /// Configuration error
    Configuration {
        message: String,
        field: Option<String>,
    },

    /// Migration error
    Migration {
        version: String,
        message: String,
    },

    /// Connection pool error
    ConnectionPool {
        message: String,
        pool_size: Option<u32>,
    },

    /// Query builder error
    QueryBuilder {
        message: String,
        clause: Option<String>,
    },

    /// Lock error
    Lock {
        message: String,
        resource: String,
        lock_type: String,
    },

    /// Audit error
    Audit {
        message: String,
        operation: String,
    },

    /// Generic repository error
    Generic {
        message: String,
        operation: String,
        details: Option<HashMap<String, String>>,
    },
}

impl RepositoryError {
    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
            query: None,
            code: None,
        }
    }

    /// Create a database error with query
    pub fn database_with_query(message: impl Into<String>, query: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
            query: Some(query.into()),
            code: None,
        }
    }

    /// Create a database error with query and code
    pub fn database_with_details(
        message: impl Into<String>,
        query: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self::Database {
            message: message.into(),
            query: Some(query.into()),
            code: Some(code.into()),
        }
    }

    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a duplicate error
    pub fn duplicate(entity_type: impl Into<String>, field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Duplicate {
            entity_type: entity_type.into(),
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.into(),
            value: None,
        }
    }

    /// Create a validation error with value
    pub fn validation_with_value(
        message: impl Into<String>,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.into(),
            value: Some(value.into()),
        }
    }

    /// Create a transaction error
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
            transaction_id: None,
        }
    }

    /// Create a transaction error with ID
    pub fn transaction_with_id(message: impl Into<String>, transaction_id: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
            transaction_id: Some(transaction_id.into()),
        }
    }

    /// Create a constraint violation error
    pub fn constraint_violation(constraint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.into(),
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

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a concurrency error
    pub fn concurrency(message: impl Into<String>) -> Self {
        Self::Concurrency {
            message: message.into(),
            entity_id: None,
        }
    }

    /// Create a concurrency error with entity ID
    pub fn concurrency_with_id(message: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self::Concurrency {
            message: message.into(),
            entity_id: Some(entity_id.into()),
        }
    }

    /// Create a cache error
    pub fn cache(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            operation: operation.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            entity_type: entity_type.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field: None,
        }
    }

    /// Create a configuration error with field
    pub fn configuration_with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a migration error
    pub fn migration(version: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Migration {
            version: version.into(),
            message: message.into(),
        }
    }

    /// Create a connection pool error
    pub fn connection_pool(message: impl Into<String>) -> Self {
        Self::ConnectionPool {
            message: message.into(),
            pool_size: None,
        }
    }

    /// Create a connection pool error with size
    pub fn connection_pool_with_size(message: impl Into<String>, pool_size: u32) -> Self {
        Self::ConnectionPool {
            message: message.into(),
            pool_size: Some(pool_size),
        }
    }

    /// Create a query builder error
    pub fn query_builder(message: impl Into<String>) -> Self {
        Self::QueryBuilder {
            message: message.into(),
            clause: None,
        }
    }

    /// Create a query builder error with clause
    pub fn query_builder_with_clause(message: impl Into<String>, clause: impl Into<String>) -> Self {
        Self::QueryBuilder {
            message: message.into(),
            clause: Some(clause.into()),
        }
    }

    /// Create a lock error
    pub fn lock(message: impl Into<String>, resource: impl Into<String>, lock_type: impl Into<String>) -> Self {
        Self::Lock {
            message: message.into(),
            resource: resource.into(),
            lock_type: lock_type.into(),
        }
    }

    /// Create an audit error
    pub fn audit(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Audit {
            message: message.into(),
            operation: operation.into(),
        }
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            operation: operation.into(),
            details: None,
        }
    }

    /// Create a generic error with details
    pub fn generic_with_details(
        message: impl Into<String>,
        operation: impl Into<String>,
        details: HashMap<String, String>,
    ) -> Self {
        Self::Generic {
            message: message.into(),
            operation: operation.into(),
            details: Some(details),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            RepositoryError::Database { .. } => true,
            RepositoryError::Timeout { .. } => true,
            RepositoryError::ConnectionPool { .. } => true,
            RepositoryError::Concurrency { .. } => true,
            RepositoryError::Cache { .. } => true,
            RepositoryError::Lock { .. } => true,
            _ => false,
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        match self {
            RepositoryError::NotFound { .. } => true,
            RepositoryError::Duplicate { .. } => true,
            RepositoryError::Validation { .. } => true,
            RepositoryError::PermissionDenied { .. } => true,
            RepositoryError::ConstraintViolation { .. } => true,
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
            RepositoryError::Database { .. } => "database",
            RepositoryError::NotFound { .. } => "not_found",
            RepositoryError::Duplicate { .. } => "duplicate",
            RepositoryError::Validation { .. } => "validation",
            RepositoryError::Transaction { .. } => "transaction",
            RepositoryError::ConstraintViolation { .. } => "constraint",
            RepositoryError::PermissionDenied { .. } => "permission",
            RepositoryError::Timeout { .. } => "timeout",
            RepositoryError::Concurrency { .. } => "concurrency",
            RepositoryError::Cache { .. } => "cache",
            RepositoryError::Serialization { .. } => "serialization",
            RepositoryError::Configuration { .. } => "configuration",
            RepositoryError::Migration { .. } => "migration",
            RepositoryError::ConnectionPool { .. } => "connection_pool",
            RepositoryError::QueryBuilder { .. } => "query_builder",
            RepositoryError::Lock { .. } => "lock",
            RepositoryError::Audit { .. } => "audit",
            RepositoryError::Generic { .. } => "generic",
        }
    }

    /// Get HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            RepositoryError::NotFound { .. } => 404,
            RepositoryError::Duplicate { .. } => 409,
            RepositoryError::Validation { .. } => 400,
            RepositoryError::PermissionDenied { .. } => 403,
            RepositoryError::Timeout { .. } => 504,
            RepositoryError::ConstraintViolation { .. } => 422,
            RepositoryError::ConnectionPool { .. } => 503,
            RepositoryError::Concurrency { .. } => 409,
            _ => 500,
        }
    }
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::Database { message, .. } => write!(f, "Database error: {}", message),
            RepositoryError::NotFound { entity_type, id } => {
                write!(f, "Entity '{}' with ID '{}' not found", entity_type, id)
            }
            RepositoryError::Duplicate { entity_type, field, value } => {
                write!(f, "Duplicate '{}' found for field '{}' with value '{}'", entity_type, field, value)
            }
            RepositoryError::Validation { message, field, .. } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            RepositoryError::Transaction { message, .. } => {
                write!(f, "Transaction error: {}", message)
            }
            RepositoryError::ConstraintViolation { constraint, message } => {
                write!(f, "Constraint violation '{}': {}", constraint, message)
            }
            RepositoryError::PermissionDenied { operation, resource } => {
                write!(f, "Permission denied for operation '{}' on resource '{}'", operation, resource)
            }
            RepositoryError::Timeout { operation, timeout_ms } => {
                write!(f, "Timeout after {}ms for operation '{}'", timeout_ms, operation)
            }
            RepositoryError::Concurrency { message, .. } => {
                write!(f, "Concurrency error: {}", message)
            }
            RepositoryError::Cache { message, operation } => {
                write!(f, "Cache error during '{}': {}", operation, message)
            }
            RepositoryError::Serialization { message, entity_type } => {
                write!(f, "Serialization error for '{}': {}", entity_type, message)
            }
            RepositoryError::Configuration { message, .. } => {
                write!(f, "Configuration error: {}", message)
            }
            RepositoryError::Migration { version, message } => {
                write!(f, "Migration error for version '{}': {}", version, message)
            }
            RepositoryError::ConnectionPool { message, .. } => {
                write!(f, "Connection pool error: {}", message)
            }
            RepositoryError::QueryBuilder { message, .. } => {
                write!(f, "Query builder error: {}", message)
            }
            RepositoryError::Lock { message, resource, lock_type } => {
                write!(f, "Lock error for '{}' with lock type '{}': {}", resource, lock_type, message)
            }
            RepositoryError::Audit { message, operation } => {
                write!(f, "Audit error during '{}': {}", operation, message)
            }
            RepositoryError::Generic { message, operation, .. } => {
                write!(f, "Repository error during '{}': {}", operation, message)
            }
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        AppError::Repository {
            message: err.to_string(),
            operation: err.category().to_string(),
        }
    }
}

/// Repository error context for enhanced error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub query: Option<String>,
    pub parameters: Option<std::collections::HashMap<String, String>>,
    pub duration_ms: Option<u64>,
    pub attempt_count: u32,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            entity_type: None,
            entity_id: None,
            query: None,
            parameters: None,
            duration_ms: None,
            attempt_count: 1,
            user_id: None,
            session_id: None,
            request_id: None,
        }
    }

    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }

    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters = Some(parameters);
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

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// Enhanced repository error with context
#[derive(Debug, Clone)]
pub struct ContextualRepositoryError {
    pub error: RepositoryError,
    pub context: ErrorContext,
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ContextualRepositoryError {
    pub fn new(error: RepositoryError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            cause: None,
        }
    }

    pub fn with_cause(mut self, cause: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.cause = Some(cause);
        self
    }

    pub fn is_retryable(&self) -> bool {
        self.error.is_retryable()
    }

    pub fn category(&self) -> &'static str {
        self.error.category()
    }

    pub fn http_status_code(&self) -> u16 {
        self.error.http_status_code()
    }
}

impl std::fmt::Display for ContextualRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.context.operation, self.error)
    }
}

impl std::error::Error for ContextualRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref())
    }
}

impl From<ContextualRepositoryError> for AppError {
    fn from(err: ContextualRepositoryError) -> Self {
        AppError::Repository {
            message: err.to_string(),
            operation: err.category().to_string(),
        }
    }
}

/// Error metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: std::collections::HashMap<String, u64>,
    pub errors_by_operation: std::collections::HashMap<String, u64>,
    pub retryable_errors: u64,
    pub client_errors: u64,
    pub server_errors: u64,
    pub average_resolution_time_ms: f64,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: std::collections::HashMap::new(),
            errors_by_operation: std::collections::HashMap::new(),
            retryable_errors: 0,
            client_errors: 0,
            server_errors: 0,
            average_resolution_time_ms: 0.0,
        }
    }

    pub fn record_error(&mut self, error: &RepositoryError, operation: &str, resolution_time_ms: u64) {
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
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}