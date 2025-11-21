use thiserror::Error;
use serde::Serialize;

/// Super Lotto application error types
#[derive(Debug, Error, Serialize)]
pub enum SuperLottoError {
    #[error("Database error: {0}")]
    #[serde(skip_serializing)]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    #[serde(skip_serializing)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Data error: {message}")]
    Data { message: String },

    #[error("Calculation error: {message}")]
    Calculation { message: String },

    #[error("IO error: {0}")]
    #[serde(skip_serializing)]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    #[serde(skip_serializing)]
    Serialization(#[from] serde_json::Error),

    #[error("Chrono error: {0}")]
    #[serde(skip_serializing)]
    Chrono(#[from] chrono::ParseError),

    #[error("Not found: {resource} with identifier {identifier}")]
    NotFound { resource: String, identifier: String },

    #[error("Already exists: {resource} with identifier {identifier}")]
    AlreadyExists { resource: String, identifier: String },

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Invalid input: {input} - {reason}")]
    InvalidInput { input: String, reason: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    #[error("Timeout error: {message}")]
    Timeout { message: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Business logic error: {message}")]
    BusinessLogic { message: String },

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl SuperLottoError {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a data error
    pub fn data(message: impl Into<String>) -> Self {
        Self::Data {
            message: message.into(),
        }
    }

    /// Create a calculation error
    pub fn calculation(message: impl Into<String>) -> Self {
        Self::Calculation {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            identifier: identifier.into(),
        }
    }

    /// Create an already exists error
    pub fn already_exists(resource: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
            identifier: identifier.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::PermissionDenied {
            action: action.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(input: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            input: input.into(),
            reason: reason.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create an internal server error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a business logic error
    pub fn business_logic(message: impl Into<String>) -> Self {
        Self::BusinessLogic {
            message: message.into(),
        }
    }

    /// Create an unknown error
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }

    /// Get error code for categorization
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Migration(_) => "MIGRATION_ERROR",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Data { .. } => "DATA_ERROR",
            Self::Calculation { .. } => "CALCULATION_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Chrono(_) => "CHRONO_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::AlreadyExists { .. } => "ALREADY_EXISTS",
            Self::PermissionDenied { .. } => "PERMISSION_DENIED",
            Self::InvalidInput { .. } => "INVALID_INPUT",
            Self::Configuration { .. } => "CONFIGURATION_ERROR",
            Self::Network { .. } => "NETWORK_ERROR",
            Self::Parse { .. } => "PARSE_ERROR",
            Self::Authentication { .. } => "AUTHENTICATION_ERROR",
            Self::RateLimit { .. } => "RATE_LIMIT_ERROR",
            Self::Timeout { .. } => "TIMEOUT_ERROR",
            Self::Internal { .. } => "INTERNAL_ERROR",
            Self::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::BusinessLogic { .. } => "BUSINESS_LOGIC_ERROR",
            Self::Unknown { .. } => "UNKNOWN_ERROR",
        }
    }

    /// Check if error is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Validation { .. }
                | Self::InvalidInput { .. }
                | Self::NotFound { .. }
                | Self::AlreadyExists { .. }
                | Self::PermissionDenied { .. }
                | Self::Authentication { .. }
                | Self::Parse { .. }
                | Self::RateLimit { .. }
        )
    }

    /// Check if error is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::Migration(_)
                | Self::Data { .. }
                | Self::Calculation { .. }
                | Self::Io(_)
                | Self::Serialization(_)
                | Self::Chrono(_)
                | Self::Configuration { .. }
                | Self::Network { .. }
                | Self::Timeout { .. }
                | Self::Internal { .. }
                | Self::ExternalService { .. }
                | Self::BusinessLogic { .. }
                | Self::Unknown { .. }
        )
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::Network { .. }
                | Self::Timeout { .. }
                | Self::ExternalService { .. }
                | Self::Io(_)
        )
    }

    /// Get HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::Validation { .. } | Self::InvalidInput { .. } | Self::Parse { .. } => 400,
            Self::Authentication { .. } => 401,
            Self::PermissionDenied { .. } => 403,
            Self::NotFound { .. } => 404,
            Self::AlreadyExists { .. } => 409,
            Self::RateLimit { .. } => 429,
            Self::Timeout { .. } => 408,
            Self::Internal { .. }
            | Self::Database(_)
            | Self::Migration(_)
            | Self::Data { .. }
            | Self::Calculation { .. }
            | Self::Io(_)
            | Self::Serialization(_)
            | Self::Chrono(_)
            | Self::Configuration { .. }
            | Self::Network { .. }
            | Self::ExternalService { .. }
            | Self::BusinessLogic { .. }
            | Self::Unknown { .. } => 500,
        }
    }
}

/// Result type alias for convenience
pub type SuperLottoResult<T> = std::result::Result<T, SuperLottoError>;

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub resource_id: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            resource_id: None,
            user_id: None,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Enhanced error with context
#[derive(Debug)]
pub struct ContextualError {
    pub error: SuperLottoError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: SuperLottoError, context: ErrorContext) -> Self {
        Self { error, context }
    }

    pub fn with_context(error: SuperLottoError, operation: impl Into<String>) -> Self {
        Self {
            error,
            context: ErrorContext::new(operation),
        }
    }
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error in operation '{}': {}",
            self.context.operation, self.error
        )
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error handling utilities
pub struct ErrorUtils;

impl ErrorUtils {
    /// Convert any error to SuperLottoError
    pub fn convert_error<E: std::error::Error + Send + Sync + 'static>(
        err: E,
        context: &str,
    ) -> SuperLottoError {
        SuperLottoError::unknown(format!("{}: {}", context, err))
    }

    /// Handle result with context
    pub fn with_context<T, E: std::error::Error + Send + Sync + 'static>(
        result: std::result::Result<T, E>,
        operation: impl Into<String>,
    ) -> SuperLottoResult<T> {
        result.map_err(|e| Self::convert_error(e, &operation.into()))
    }

    /// Create validation error from multiple messages
    pub fn validation_errors(messages: Vec<impl Into<String>>) -> SuperLottoError {
        let message = messages.into_iter().map(|m| m.into()).collect::<Vec<_>>().join("; ");
        SuperLottoError::validation(message)
    }

    /// Handle database operation errors with user-friendly messages
    pub fn handle_database_error(err: sqlx::Error, operation: &str) -> SuperLottoError {
        match err {
            sqlx::Error::RowNotFound => {
                SuperLottoError::not_found("record", operation)
            }
            sqlx::Error::Database(ref db_err) => {
                // Just use a simple check for unique constraint - avoid the is_unique_violation method
                if db_err.message().contains("UNIQUE constraint") {
                    SuperLottoError::already_exists("record", operation)
                } else if db_err.message().contains("FOREIGN KEY constraint") {
                    SuperLottoError::validation("Referenced record does not exist")
                } else if db_err.message().contains("CHECK constraint") {
                    SuperLottoError::validation("Data constraint violation")
                } else {
                    SuperLottoError::Database(err)
                }
            }
            _ => SuperLottoError::Database(err),
        }
    }

    /// Retry operation with exponential backoff
    pub async fn retry_with_backoff<F, T>(
        mut operation: F,
        max_retries: u32,
        initial_delay: std::time::Duration,
    ) -> SuperLottoResult<T>
    where
        F: FnMut() -> SuperLottoResult<T>,
    {
        let mut delay = initial_delay;

        for attempt in 1..=max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) if attempt == max_retries => return Err(err),
                Err(err) if !err.is_retryable() => return Err(err),
                Err(_) => {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, std::time::Duration::from_secs(60));
                }
            }
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SuperLottoError::validation("Invalid number");
        assert_eq!(err.error_code(), "VALIDATION_ERROR");
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
        assert_eq!(err.http_status_code(), 400);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_resource_id("123")
            .with_user_id("user456")
            .with_metadata("key", "value");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.resource_id, Some("123".to_string()));
        assert_eq!(context.user_id, Some("user456".to_string()));
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_contextual_error() {
        let base_err = SuperLottoError::validation("Test error");
        let context = ErrorContext::new("test_operation");
        let contextual_err = ContextualError::new(base_err, context);

        assert!(contextual_err.to_string().contains("test_operation"));
    }

    #[test]
    fn test_error_utils() {
        let err = SuperLottoError::validation_errors(vec!["Error 1", "Error 2"]);
        assert!(matches!(err, SuperLottoError::Validation { .. }));
        assert!(err.to_string().contains("Error 1"));
        assert!(err.to_string().contains("Error 2"));
    }
}