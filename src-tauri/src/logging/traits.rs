//! Logging Traits
//!
//! Defines the core logging interfaces and contracts.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use chrono::{DateTime, Utc};

/// Log record trait
pub trait LogRecord: Send + Sync {
    /// Get the log level
    fn level(&self) -> LogLevel;

    /// Get the target/logger name
    fn target(&self) -> &str;

    /// Get the log message
    fn message(&self) -> &str;

    /// Get the timestamp
    fn timestamp(&self) -> DateTime<Utc>;

    /// Get the module path
    fn module_path(&self) -> Option<&str>;

    /// Get the file path
    fn file(&self) -> Option<&str>;

    /// Get the line number
    fn line(&self) -> Option<u32>;

    /// Get the structured fields
    fn fields(&self) -> &HashMap<String, LogFieldValue>;

    /// Get the key-value pairs
    fn kv(&self) -> &[(String, LogFieldValue)];

    /// Check if the record contains a specific field
    fn has_field(&self, key: &str) -> bool {
        self.fields().contains_key(key)
    }

    /// Get a field value
    fn get_field(&self, key: &str) -> Option<&LogFieldValue> {
        self.fields().get(key)
    }

    /// Get the record ID
    fn id(&self) -> Option<&str>;

    /// Get the span/context ID
    fn span_id(&self) -> Option<&str>;

    /// Get the trace ID
    fn trace_id(&self) -> Option<&str>;

    /// Get the parent span ID
    fn parent_span_id(&self) -> Option<&str>;
}

/// Logger trait
pub trait Logger: Send + Sync {
    /// Log a record
    fn log(&self, record: &dyn LogRecord);

    /// Check if a log level is enabled
    fn enabled(&self, level: LogLevel, target: &str) -> bool;

    /// Get the logger name
    fn name(&self) -> &str;

    /// Get the logger configuration
    fn config(&self) -> &LoggerConfig;

    /// Flush any buffered records
    fn flush(&self);

    /// Get logger metadata
    fn metadata(&self) -> &LoggerMetadata;
}

/// Log formatter trait
pub trait LogFormatter: Send + Sync {
    /// Format a log record
    fn format(&self, record: &dyn LogRecord) -> Result<String>;

    /// Format a log record with context
    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String>;

    /// Get formatter metadata
    fn metadata(&self) -> &FormatterMetadata;
}

/// Log appender trait
pub trait LogAppender: Send + Sync {
    /// Append a log record
    fn append(&self, record: &dyn LogRecord) -> Result<()>;

    /// Check if the appender is ready
    fn is_ready(&self) -> bool;

    /// Flush any buffered records
    fn flush(&self) -> Result<()>;

    /// Get appender metadata
    fn metadata(&self) -> &AppenderMetadata;

    /// Get appender statistics
    fn stats(&self) -> AppenderStats;
}

/// Log filter trait
pub trait LogFilter: Send + Sync {
    /// Check if a log record should be processed
    fn should_log(&self, record: &dyn LogRecord) -> bool;

    /// Get filter metadata
    fn metadata(&self) -> &FilterMetadata;
}

/// Log context provider trait
pub trait LogContextProvider: Send + Sync {
    /// Get current log context
    fn get_context(&self) -> &LogContext;

    /// Set current log context
    fn set_context(&self, context: LogContext);

    /// Update context with key-value pairs
    fn update_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext);

    /// Push a new context scope
    fn push_scope(&self, scope: LogScope);

    /// Pop a context scope
    fn pop_scope(&self) -> Option<LogScope>;

    /// Get context metadata
    fn metadata(&self) &ContextMetadata;
}

/// Log metrics collector trait
#[async_trait]
pub trait LogMetricsCollector: Send + Sync {
    /// Record a log event
    async fn record_log(&self, record: &dyn LogRecord);

    /// Get metrics
    async fn get_metrics(&self) -> Result<LogMetrics>;

    /// Reset metrics
    async fn reset_metrics(&self) -> Result<()>;

    /// Get metrics metadata
    fn metadata(&self) &MetricsMetadata;
}

/// Async logger trait for logging operations that may block
#[async_trait]
pub trait AsyncLogger: Send + Sync {
    /// Log a record asynchronously
    async fn log_async(&self, record: Box<dyn LogRecord>) -> Result<()>;

    /// Check if a log level is enabled asynchronously
    async fn enabled_async(&self, level: LogLevel, target: &str) -> bool;

    /// Flush asynchronously
    async fn flush_async(&self) -> Result<()>;

    /// Shutdown the logger
    async fn shutdown(&self) -> Result<()>;
}

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    /// Get the string representation of the log level
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    /// Get the ANSI color code for the log level
    pub fn ansi_color(self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[37m",      // White
            LogLevel::Debug => "\x1b[36m",      // Cyan
            LogLevel::Info => "\x1b[32m",       // Green
            LogLevel::Warn => "\x1b[33m",       // Yellow
            LogLevel::Error => "\x1b[31m",      // Red
            LogLevel::Fatal => "\x1b[41;37m",   // Red background, white text
        }
    }

    /// Get the reset ANSI color code
    pub fn ansi_reset() -> &'static str {
        "\x1b[0m"
    }

    /// Check if this level should be logged for the given filter level
    pub fn should_log(self, filter_level: LogLevel) -> bool {
        self >= filter_level
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Log field value enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogFieldValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Duration(std::time::Duration),
    Array(Vec<LogFieldValue>),
    Object(HashMap<String, LogFieldValue>),
    Null,
}

impl LogFieldValue {
    /// Get the string representation
    pub fn as_string(&self) -> Option<&str> {
        match self {
            LogFieldValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the integer representation
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            LogFieldValue::Integer(i) => Some(*i),
            LogFieldValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Get the float representation
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            LogFieldValue::Integer(i) => Some(*i as f64),
            LogFieldValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get the boolean representation
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            LogFieldValue::Boolean(b) => Some(*b),
            LogFieldValue::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Some(true),
                    "false" | "0" | "no" | "off" => Some(false),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Get the array representation
    pub fn as_array(&self) -> Option<&Vec<LogFieldValue>> {
        match self {
            LogFieldValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the object representation
    pub fn as_object(&self) -> Option<&HashMap<String, LogFieldValue>> {
        match self {
            LogFieldValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get the type name
    pub fn type_name(&self) -> &'static str {
        match self {
            LogFieldValue::String(_) => "string",
            LogFieldValue::Integer(_) => "integer",
            LogFieldValue::Float(_) => "float",
            LogFieldValue::Boolean(_) => "boolean",
            LogFieldValue::DateTime(_) => "datetime",
            LogFieldValue::Duration(_) => "duration",
            LogFieldValue::Array(_) => "array",
            LogFieldValue::Object(_) => "object",
            LogFieldValue::Null => "null",
        }
    }

    /// Convert to JSON value
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            LogFieldValue::String(s) => serde_json::Value::String(s.clone()),
            LogFieldValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            LogFieldValue::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
            LogFieldValue::Boolean(b) => serde_json::Value::Bool(*b),
            LogFieldValue::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
            LogFieldValue::Duration(d) => serde_json::Value::String(format!("{}ms", d.as_millis())),
            LogFieldValue::Array(arr) => {
                let json_arr: Vec<serde_json::Value> = arr.iter()
                    .map(|v| v.to_json_value())
                    .collect();
                serde_json::Value::Array(json_arr)
            }
            LogFieldValue::Object(obj) => {
                let json_obj: serde_json::Map<String, serde_json::Value> = obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json_value()))
                    .collect();
                serde_json::Value::Object(json_obj)
            }
            LogFieldValue::Null => serde_json::Value::Null,
        }
    }

    /// Check if the value is empty
    pub fn is_empty(&self) -> bool {
        match self {
            LogFieldValue::String(s) => s.is_empty(),
            LogFieldValue::Array(arr) => arr.is_empty(),
            LogFieldValue::Object(obj) => obj.is_empty(),
            LogFieldValue::Null => true,
            _ => false,
        }
    }

    /// Get the size of the value
    pub fn size(&self) -> usize {
        match self {
            LogFieldValue::String(s) => s.len(),
            LogFieldValue::Array(arr) => arr.len(),
            LogFieldValue::Object(obj) => obj.len(),
            _ => 1,
        }
    }
}

impl From<String> for LogFieldValue {
    fn from(value: String) -> Self {
        LogFieldValue::String(value)
    }
}

impl From<i64> for LogFieldValue {
    fn from(value: i64) -> Self {
        LogFieldValue::Integer(value)
    }
}

impl From<f64> for LogFieldValue {
    fn from(value: f64) -> Self {
        LogFieldValue::Float(value)
    }
}

impl From<bool> for LogFieldValue {
    fn from(value: bool) -> Self {
        LogFieldValue::Boolean(value)
    }
}

impl From<DateTime<Utc>> for LogFieldValue {
    fn from(value: DateTime<Utc>) -> Self {
        LogFieldValue::DateTime(value)
    }
}

impl From<std::time::Duration> for LogFieldValue {
    fn from(value: std::time::Duration) -> Self {
        LogFieldValue::Duration(value)
    }
}

impl From<Vec<LogFieldValue>> for LogFieldValue {
    fn from(value: Vec<LogFieldValue>) -> Self {
        LogFieldValue::Array(value)
    }
}

impl From<HashMap<String, LogFieldValue>> for LogFieldValue {
    fn from(value: HashMap<String, LogFieldValue>) -> Self {
        LogFieldValue::Object(value)
    }
}

/// Log context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub fields: HashMap<String, LogFieldValue>,
    pub scopes: Vec<LogScope>,
    pub timestamp: DateTime<Utc>,
}

impl LogContext {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            scopes: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_fields(fields: HashMap<String, LogFieldValue>) -> Self {
        Self {
            fields,
            scopes: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_scopes(scopes: Vec<LogScope>) -> Self {
        Self {
            fields: HashMap::new(),
            scopes,
            timestamp: Utc::now(),
        }
    }

    pub fn add_field(&mut self, key: String, value: LogFieldValue) {
        self.fields.insert(key, value);
    }

    pub fn add_fields(&mut self, fields: HashMap<String, LogFieldValue>) {
        self.fields.extend(fields);
    }

    pub fn get_field(&self, key: &str) -> Option<&LogFieldValue> {
        self.fields.get(key)
    }

    pub fn has_field(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    pub fn push_scope(&mut self, scope: LogScope) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) -> Option<LogScope> {
        self.scopes.pop()
    }

    pub fn merge_fields(&self, record_fields: &HashMap<String, LogFieldValue>) -> HashMap<String, LogFieldValue> {
        let mut merged = self.fields.clone();
        merged.extend(record_fields.clone());
        merged
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.scopes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len() + self.scopes.len()
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Log scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogScope {
    pub name: String,
    pub level: LogLevel,
    pub fields: HashMap<String, LogFieldValue>,
    pub timestamp: DateTime<Utc>,
}

impl LogScope {
    pub fn new(name: String, level: LogLevel) -> Self {
        Self {
            name,
            level,
            fields: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_fields(mut self, fields: HashMap<String, LogFieldValue>) -> Self {
        self.fields = fields;
        self
    }

    pub fn add_field(&mut self, key: String, value: LogFieldValue) {
        self.fields.insert(key, value);
    }

    pub fn get_field(&self, key: &str) -> Option<&LogFieldValue> {
        self.fields.get(key)
    }
}

/// Support metadata types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub max_level: LogLevel,
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub format_type: String,
    pub supports_colors: bool,
    pub supports_structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppenderMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub appender_type: String,
    pub is_async: bool,
    pub supports_buffering: bool,
    pub supports_rotation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub filter_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supports_nesting: bool,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_metrics: Vec<String>,
}

/// Statistics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppenderStats {
    pub records_written: u64,
    pub records_dropped: u64,
    pub bytes_written: u64,
    pub errors: u64,
    pub last_write: Option<DateTime<Utc>>,
    pub buffer_size: Option<usize>,
    pub average_write_time_ms: f64,
}

impl AppenderStats {
    pub fn new() -> Self {
        Self {
            records_written: 0,
            records_dropped: 0,
            bytes_written: 0,
            errors: 0,
            last_write: None,
            buffer_size: None,
            average_write_time_ms: 0.0,
        }
    }

    pub fn record_write(&mut self, bytes: u64, write_time_ms: f64) {
        self.records_written += 1;
        self.bytes_written += bytes;
        self.last_write = Some(Utc::now());

        // Update average write time
        if self.records_written == 1 {
            self.average_write_time_ms = write_time_ms;
        } else {
            let total_time = self.average_write_time_ms * (self.records_written - 1) as f64 + write_time_ms;
            self.average_write_time_ms = total_time / self.records_written as f64;
        }
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    pub fn record_drop(&mut self) {
        self.records_dropped += 1;
    }

    pub fn error_rate(&self) -> f64 {
        let total = self.records_written + self.records_dropped;
        if total > 0 {
            self.errors as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn drop_rate(&self) -> f64 {
        let total = self.records_written + self.records_dropped;
        if total > 0 {
            self.records_dropped as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl Default for AppenderStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    pub total_logs: u64,
    pub logs_by_level: HashMap<String, u64>,
    pub logs_by_target: HashMap<String, u64>,
    pub total_bytes: u64,
    pub average_log_size: f64,
    pub errors: u64,
    pub warnings: u64,
    pub first_log: Option<DateTime<Utc>>,
    pub last_log: Option<DateTime<Utc>>,
}

impl LogMetrics {
    pub fn new() -> Self {
        Self {
            total_logs: 0,
            logs_by_level: HashMap::new(),
            logs_by_target: HashMap::new(),
            total_bytes: 0,
            average_log_size: 0.0,
            errors: 0,
            warnings: 0,
            first_log: None,
            last_log: None,
        }
    }

    pub fn record_log(&mut self, level: LogLevel, target: &str, size_bytes: usize) {
        self.total_logs += 1;
        self.total_bytes += size_bytes as u64;
        self.last_log = Some(Utc::now());

        if self.first_log.is_none() {
            self.first_log = Some(Utc::now());
        }

        // Update average log size
        self.average_log_size = self.total_bytes as f64 / self.total_logs as f64;

        // Update level counts
        let level_str = level.as_str().to_lowercase();
        *self.logs_by_level.entry(level_str).or_insert(0) += 1;

        // Update target counts
        *self.logs_by_target.entry(target.to_string()).or_insert(0) += 1;

        // Update error and warning counts
        match level {
            LogLevel::Error => self.errors += 1,
            LogLevel::Fatal => self.errors += 1,
            LogLevel::Warn => self.warnings += 1,
            _ => {}
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_logs > 0 {
            self.errors as f64 / self.total_logs as f64
        } else {
            0.0
        }
    }

    pub fn warning_rate(&self) -> f64 {
        if self.total_logs > 0 {
            self.warnings as f64 / self.total_logs as f64
        } else {
            0.0
        }
    }
}

impl Default for LogMetrics {
    fn default() -> Self {
        Self::new()
    }
}