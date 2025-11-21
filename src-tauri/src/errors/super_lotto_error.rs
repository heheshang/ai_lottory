//! Super Lotto error types
//!
//! Comprehensive error handling for Super Lotto functionality.

use thiserror::Error;
use crate::super_lotto::models::ValidationError;

#[derive(Debug, Error)]
pub enum SuperLottoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Prediction error: {0}")]
    Prediction(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Chrono parsing error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Computation error: {0}")]
    Computation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

// Convert SuperLottoError to a string that can be sent to the frontend
impl SuperLottoError {
    pub fn to_error_message(&self) -> String {
        match self {
            SuperLottoError::Database(err) => format!("Database operation failed: {}", err),
            SuperLottoError::Validation(err) => format!("Data validation failed: {}", err),
            SuperLottoError::Analysis(msg) => format!("Analysis failed: {}", msg),
            SuperLottoError::Prediction(msg) => format!("Prediction failed: {}", msg),
            SuperLottoError::Cache(msg) => format!("Cache operation failed: {}", msg),
            SuperLottoError::Io(err) => format!("File operation failed: {}", err),
            SuperLottoError::JsonSerialization(err) => format!("Data serialization failed: {}", err),
            SuperLottoError::ChronoParse(err) => format!("Date parsing failed: {}", err),
            SuperLottoError::NotFound(msg) => format!("Resource not found: {}", msg),
            SuperLottoError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            SuperLottoError::InsufficientData(msg) => format!("Insufficient data: {}", msg),
            SuperLottoError::Computation(msg) => format!("Computation failed: {}", msg),
            SuperLottoError::Configuration(msg) => format!("Configuration error: {}", msg),
            SuperLottoError::Internal(msg) => format!("Internal error: {}", msg),
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            SuperLottoError::Database(_) => "DATABASE_ERROR",
            SuperLottoError::Validation(_) => "VALIDATION_ERROR",
            SuperLottoError::Analysis(_) => "ANALYSIS_ERROR",
            SuperLottoError::Prediction(_) => "PREDICTION_ERROR",
            SuperLottoError::Cache(_) => "CACHE_ERROR",
            SuperLottoError::Io(_) => "IO_ERROR",
            SuperLottoError::JsonSerialization(_) => "SERIALIZATION_ERROR",
            SuperLottoError::ChronoParse(_) => "DATE_PARSE_ERROR",
            SuperLottoError::NotFound(_) => "NOT_FOUND",
            SuperLottoError::InvalidInput(_) => "INVALID_INPUT",
            SuperLottoError::InsufficientData(_) => "INSUFFICIENT_DATA",
            SuperLottoError::Computation(_) => "COMPUTATION_ERROR",
            SuperLottoError::Configuration(_) => "CONFIGURATION_ERROR",
            SuperLottoError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

// Type alias for convenience
pub type Result<T> = std::result::Result<T, SuperLottoError>;