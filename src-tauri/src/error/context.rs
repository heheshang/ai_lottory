//! Error context tracking for better debugging and error handling

use crate::error::{AppError, ErrorContext as ErrorContextStruct, ErrorSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to a Result
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContextStruct;

    /// Add operation context
    fn with_operation(self, operation: &str) -> Result<T>;

    /// Add user context
    fn with_user(self, user_id: &str) -> Result<T>;

    /// Add session context
    fn with_session(self, session_id: &str) -> Result<T>;

    /// Add metadata context
    fn with_metadata(self, metadata: serde_json::Value) -> Result<T>;
}

impl<T> ErrorContext<T> for std::result::Result<T, AppError> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContextStruct,
    {
        self.map_err(|e| {
            let context = f();
            // TODO: Store context for error reporting
            tracing::error!(
                error_id = %context.error_id,
                operation = ?context.operation,
                user_id = ?context.user_id,
                error = %e,
                "Error occurred with context"
            );
            e
        })
    }

    fn with_operation(self, operation: &str) -> Result<T> {
        self.with_context(|| ErrorContextStruct::new(ErrorSeverity::Medium).with_operation(operation.to_string()))
    }

    fn with_user(self, user_id: &str) -> Result<T> {
        self.with_context(|| ErrorContextStruct::new(ErrorSeverity::Medium).with_user(user_id.to_string()))
    }

    fn with_session(self, session_id: &str) -> Result<T> {
        self.with_context(|| ErrorContextStruct::new(ErrorSeverity::Medium).with_session(session_id.to_string()))
    }

    fn with_metadata(self, metadata: serde_json::Value) -> Result<T> {
        self.with_context(|| ErrorContextStruct::new(ErrorSeverity::Medium).with_metadata(metadata))
    }
}

/// Global error context manager
pub struct ErrorContextManager {
    contexts: Arc<RwLock<HashMap<String, ErrorContextStruct>>>,
}

impl ErrorContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store an error context
    pub async fn store_context(&self, context: ErrorContextStruct) {
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.error_id.clone(), context);
    }

    /// Retrieve an error context
    pub async fn get_context(&self, error_id: &str) -> Option<ErrorContextStruct> {
        let contexts = self.contexts.read().await;
        contexts.get(error_id).cloned()
    }

    /// Get all contexts for a user
    pub async fn get_user_contexts(&self, user_id: &str) -> Vec<ErrorContextStruct> {
        let contexts = self.contexts.read().await;
        contexts
            .values()
            .filter(|c| c.user_id.as_ref().map_or(false, |uid| uid == user_id))
            .cloned()
            .collect()
    }

    /// Get all contexts for a session
    pub async fn get_session_contexts(&self, session_id: &str) -> Vec<ErrorContextStruct> {
        let contexts = self.contexts.read().await;
        contexts
            .values()
            .filter(|c| c.session_id.as_ref().map_or(false, |sid| sid == session_id))
            .cloned()
            .collect()
    }

    /// Clean old contexts (older than specified duration)
    pub async fn cleanup_old_contexts(&self, max_age: chrono::Duration) {
        let mut contexts = self.contexts.write().await;
        let now = chrono::Utc::now();

        contexts.retain(|_, context| {
            now.signed_duration_since(context.timestamp) < max_age
        });
    }

    /// Get error statistics
    pub async fn get_error_stats(&self) -> ErrorStats {
        let contexts = self.contexts.read().await;
        let mut stats = ErrorStats::default();

        for context in contexts.values() {
            stats.total_errors += 1;

            match context.severity {
                ErrorSeverity::Critical => stats.critical += 1,
                ErrorSeverity::High => stats.high += 1,
                ErrorSeverity::Medium => stats.medium += 1,
                ErrorSeverity::Low => stats.low += 1,
            }
        }

        stats
    }
}

/// Error statistics
#[derive(Debug, Default, Clone)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

impl ErrorStats {
    pub fn critical_rate(&self) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            (self.critical as f64 / self.total_errors as f64) * 100.0
        }
    }

    pub fn high_plus_critical_rate(&self) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            ((self.critical + self.high) as f64 / self.total_errors as f64) * 100.0
        }
    }
}

/// Macro for easy error context addition
#[macro_export]
macro_rules! error_context {
    ($result:expr, $operation:expr) => {
        $result.with_operation($operation)
    };
    ($result:expr, $operation:expr, user = $user_id:expr) => {
        $result.with_operation($operation).with_user($user_id)
    };
    ($result:expr, $operation:expr, session = $session_id:expr) => {
        $result.with_operation($operation).with_session($session_id)
    };
    ($result:expr, $operation:expr, metadata = $metadata:expr) => {
        $result.with_operation($operation).with_metadata($metadata)
    };
}