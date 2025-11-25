//! Logger Implementation
//!
//! Implements the core logging functionality with structured logging support.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Default logger implementation
pub struct DefaultLogger {
    name: String,
    config: LoggerConfig,
    metadata: LoggerMetadata,
    appenders: Vec<Arc<dyn LogAppender>>,
    filters: Vec<Arc<dyn LogFilter>>,
    formatter: Arc<dyn LogFormatter>,
    context_provider: Arc<dyn LogContextProvider>,
    metrics_collector: Option<Arc<dyn LogMetricsCollector>>,
}

impl DefaultLogger {
    pub fn new(name: String, config: LoggerConfig) -> Self {
        let metadata = LoggerMetadata {
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: format!("Default logger: {}", name),
            capabilities: vec![
                "structured".to_string(),
                "filtering".to_string(),
                "formatting".to_string(),
                "metrics".to_string(),
                "context".to_string(),
            ],
            max_level: config.max_level,
            filters: config.filters.clone(),
        };

        Self {
            name,
            config,
            metadata,
            appenders: Vec::new(),
            filters: Vec::new(),
            formatter: Arc::new(DefaultLogFormatter::new()),
            context_provider: Arc::new(DefaultLogContextProvider::new()),
            metrics_collector: None,
        }
    }

    pub fn with_formatter(mut self, formatter: Arc<dyn LogFormatter>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_context_provider(mut self, context_provider: Arc<dyn LogContextProvider>) -> Self {
        self.context_provider = context_provider;
        self
    }

    pub fn with_metrics_collector(mut self, metrics_collector: Arc<dyn LogMetricsCollector>) -> Self {
        self.metrics_collector = Some(metrics_collector);
        self
    }

    pub fn add_appender(mut self, appender: Arc<dyn LogAppender>) -> Self {
        self.appenders.push(appender);
        self
    }

    pub fn add_filter(mut self, filter: Arc<dyn LogFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    fn should_log(&self, record: &dyn LogRecord) -> bool {
        // Check if level is enabled
        if !self.enabled(record.level(), record.target()) {
            return false;
        }

        // Apply filters
        for filter in &self.filters {
            if !filter.should_log(record) {
                return false;
            }
        }

        true
    }

    fn get_full_record(&self, record: &dyn LogRecord) -> FullLogRecord {
        let context = self.context_provider.get_context();
        let merged_fields = context.merge_fields(record.fields());

        FullLogRecord {
            record: record.clone(),
            context: context.clone(),
            merged_fields,
        }
    }
}

impl Logger for DefaultLogger {
    fn log(&self, record: &dyn LogRecord) {
        if !self.should_log(record) {
            return;
        }

        let full_record = self.get_full_record(record);

        // Log to all appenders
        for appender in &self.appenders {
            if appender.is_ready() {
                if let Err(e) = appender.append(&full_record.record) {
                    // Log the error to stderr to avoid infinite loops
                    eprintln!("Logging appender error: {}", e);
                }
            }
        }

        // Record metrics asynchronously if available
        if let Some(metrics) = &self.metrics_collector {
            let record_clone = full_record.record.clone();
            tokio::spawn(async move {
                if let Err(e) = metrics.record_log(&*record_clone).await {
                    eprintln!("Metrics collection error: {}", e);
                }
            });
        }
    }

    fn enabled(&self, level: LogLevel, target: &str) -> bool {
        level >= self.config.max_level
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> &LoggerConfig {
        &self.config
    }

    fn flush(&self) {
        for appender in &self.appenders {
            if let Err(e) = appender.flush() {
                eprintln!("Error flushing appender: {}", e);
            }
        }
    }

    fn metadata(&self) -> &LoggerMetadata {
        &self.metadata
    }
}

/// Full log record with context
struct FullLogRecord {
    record: Box<dyn LogRecord>,
    context: LogContext,
    merged_fields: HashMap<String, LogFieldValue>,
}

/// Default log formatter
pub struct DefaultLogFormatter {
    config: FormatterConfig,
    metadata: FormatterMetadata,
}

impl DefaultLogFormatter {
    pub fn new() -> Self {
        let config = FormatterConfig::default();
        let metadata = FormatterMetadata {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            description: "Default log formatter".to_string(),
            format_type: "text".to_string(),
            supports_colors: true,
            supports_structured: true,
        };

        Self { config, metadata }
    }

    pub fn with_config(mut self, config: FormatterConfig) -> Self {
        self.config = config;
        self
    }
}

impl LogFormatter for DefaultLogFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        let mut output = String::new();

        // Add timestamp if configured
        if self.config.include_timestamp {
            output.push_str(&format!("{} ", record.timestamp().format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        // Add level with color if configured
        if self.config.include_level {
            let level_str = record.level().as_str();
            if self.config.use_colors {
                output.push_str(record.level().ansi_color());
                output.push_str(level_str);
                output.push_str(LogLevel::ansi_reset());
            } else {
                output.push_str(level_str);
            }
            output.push(' ');
        }

        // Add target if configured
        if self.config.include_target && !record.target().is_empty() {
            output.push('[');
            output.push_str(record.target());
            output.push(']');
            output.push(' ');
        }

        // Add message
        output.push_str(record.message());

        // Add fields if configured
        if self.config.include_fields && !record.fields().is_empty() {
            output.push_str(" |");
            for (key, value) in record.fields() {
                output.push(' ');
                output.push_str(&format!("{}={}", key, format_field_value(value)));
            }
        }

        // Add module and line if configured
        if self.config.include_location {
            if let Some(module) = record.module_path() {
                output.push_str(&format!(" [{}]", module));
            }
            if let Some(file) = record.file() {
                if let Some(line) = record.line() {
                    output.push_str(&format!(" ({}:{})", file, line));
                }
            }
        }

        Ok(output)
    }

    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String> {
        // Create a temporary record with merged fields
        let mut temp_fields = record.fields().clone();
        temp_fields.extend(context.fields.clone());

        // This is a simplified implementation
        // In a real implementation, we would create a proper merged record
        self.format(record)
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}

/// Default log context provider
pub struct DefaultLogContextProvider {
    context: Arc<RwLock<LogContext>>,
    metadata: ContextMetadata,
}

impl DefaultLogContextProvider {
    pub fn new() -> Self {
        let metadata = ContextMetadata {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            description: "Default log context provider".to_string(),
            supports_nesting: true,
            max_depth: 10,
        };

        Self {
            context: Arc::new(RwLock::new(LogContext::new())),
            metadata,
        }
    }
}

impl LogContextProvider for DefaultLogContextProvider {
    fn get_context(&self) -> &LogContext {
        // Note: This is a limitation of the trait design
        // In practice, you would need async support here
        unimplemented!("This method should be async in practice")
    }

    fn set_context(&self, context: LogContext) {
        let mut ctx = self.context.write().unwrap();
        *ctx = context;
    }

    fn update_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        let mut ctx = self.context.write().unwrap();
        updater(&mut ctx);
    }

    fn push_scope(&self, scope: LogScope) {
        let mut ctx = self.context.write().unwrap();
        ctx.push_scope(scope);
    }

    fn pop_scope(&self) -> Option<LogScope> {
        let mut ctx = self.context.write().unwrap();
        ctx.pop_scope()
    }

    fn metadata(&self) &ContextMetadata {
        &self.metadata
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub max_level: LogLevel,
    pub filters: Vec<String>,
    pub async_mode: bool,
    pub buffer_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            max_level: LogLevel::Info,
            filters: Vec::new(),
            async_mode: true,
            buffer_size: 1000,
            flush_interval_ms: 5000,
        }
    }
}

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub include_timestamp: bool,
    pub include_level: bool,
    pub include_target: bool,
    pub include_fields: bool,
    pub include_location: bool,
    pub use_colors: bool,
    pub structured_format: StructuredFormat,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_level: true,
            include_target: true,
            include_fields: true,
            include_location: false,
            use_colors: true,
            structured_format: StructuredFormat::Text,
        }
    }
}

/// Structured log format
#[derive(Debug, Clone, Copy)]
pub enum StructuredFormat {
    Text,
    Json,
    CompactJson,
    KeyValue,
}

/// Utility function to format field values
fn format_field_value(value: &LogFieldValue) -> String {
    match value {
        LogFieldValue::String(s) => s.clone(),
        LogFieldValue::Integer(i) => i.to_string(),
        LogFieldValue::Float(f) => f.to_string(),
        LogFieldValue::Boolean(b) => b.to_string(),
        LogFieldValue::DateTime(dt) => dt.to_rfc3339(),
        LogFieldValue::Duration(d) => format!("{}ms", d.as_millis()),
        LogFieldValue::Array(arr) => {
            let values: Vec<String> = arr.iter().map(format_field_value).collect();
            format!("[{}]", values.join(", "))
        }
        LogFieldValue::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}={}", k, format_field_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        LogFieldValue::Null => "null".to_string(),
    }
}

/// Simple log record implementation
pub struct SimpleLogRecord {
    level: LogLevel,
    target: String,
    message: String,
    timestamp: DateTime<Utc>,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    fields: HashMap<String, LogFieldValue>,
    id: Option<String>,
    span_id: Option<String>,
    trace_id: Option<String>,
    parent_span_id: Option<String>,
}

impl SimpleLogRecord {
    pub fn new(
        level: LogLevel,
        target: String,
        message: String,
    ) -> Self {
        Self {
            level,
            target,
            message,
            timestamp: Utc::now(),
            module_path: None,
            file: None,
            line: None,
            fields: HashMap::new(),
            id: None,
            span_id: None,
            trace_id: None,
            parent_span_id: None,
        }
    }

    pub fn with_field(mut self, key: String, value: LogFieldValue) -> Self {
        self.fields.insert(key, value);
        self
    }

    pub fn with_module_path(mut self, module_path: String) -> Self {
        self.module_path = Some(module_path);
        self
    }

    pub fn with_location(mut self, file: String, line: u32) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self
    }

    pub fn with_span_id(mut self, span_id: String) -> Self {
        self.span_id = Some(span_id);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
}

impl LogRecord for SimpleLogRecord {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn target(&self) -> &str {
        &self.target
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn module_path(&self) -> Option<&str> {
        self.module_path.as_deref()
    }

    fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    fn line(&self) -> Option<u32> {
        self.line
    }

    fn fields(&self) -> &HashMap<String, LogFieldValue> {
        &self.fields
    }

    fn kv(&self) -> &[(String, LogFieldValue)] {
        // This is a limitation - in practice, you would store key-value pairs more efficiently
        unimplemented!("kv() method not implemented for SimpleLogRecord")
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }

    fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }
}

/// Logger factory for creating configured loggers
pub struct LoggerFactory {
    default_config: LoggerConfig,
    default_formatter: Arc<dyn LogFormatter>,
}

impl LoggerFactory {
    pub fn new() -> Self {
        Self {
            default_config: LoggerConfig::default(),
            default_formatter: Arc::new(DefaultLogFormatter::new()),
        }
    }

    pub fn with_config(mut self, config: LoggerConfig) -> Self {
        self.default_config = config;
        self
    }

    pub fn with_formatter(mut self, formatter: Arc<dyn LogFormatter>) -> Self {
        self.default_formatter = formatter;
        self
    }

    pub fn create_logger(&self, name: String) -> DefaultLogger {
        let logger = DefaultLogger::new(name, self.default_config.clone())
            .with_formatter(self.default_formatter.clone());
        logger
    }

    pub fn create_console_logger(&self, name: String) -> DefaultLogger {
        let console_appender = Arc::new(ConsoleAppender::new());
        self.create_logger(name)
            .with_appender(console_appender)
    }

    pub fn create_file_logger(&self, name: String, file_path: String) -> DefaultLogger {
        let file_appender = Arc::new(FileAppender::new(file_path));
        self.create_logger(name)
            .with_appender(file_appender)
    }

    pub fn create_multi_logger(&self, name: String, appenders: Vec<Arc<dyn LogAppender>>) -> DefaultLogger {
        let mut logger = self.create_logger(name);
        for appender in appenders {
            logger = logger.with_appender(appender);
        }
        logger
    }
}

impl Default for LoggerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Console appender for logging to stdout/stderr
pub struct ConsoleAppender {
    use_stderr: bool,
    metadata: AppenderMetadata,
}

impl ConsoleAppender {
    pub fn new() -> Self {
        let metadata = AppenderMetadata {
            name: "console".to_string(),
            version: "1.0.0".to_string(),
            description: "Console appender for logging to stdout/stderr".to_string(),
            appender_type: "console".to_string(),
            is_async: false,
            supports_buffering: false,
            supports_rotation: false,
        };

        Self {
            use_stderr: false,
            metadata,
        }
    }

    pub fn with_stderr(mut self, use_stderr: bool) -> Self {
        self.use_stderr = use_stderr;
        self
    }
}

impl LogAppender for ConsoleAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        let message = format!(
            "[{}] {} - {}: {}",
            record.timestamp().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.message()
        );

        if self.use_stderr || record.level() >= LogLevel::Error {
            eprintln!("{}", message);
        } else {
            println!("{}", message);
        }

        Ok(())
    }

    fn is_ready(&self) -> bool {
        true // Console is always ready
    }

    fn flush(&self) -> Result<()> {
        // Console doesn't need flushing
        Ok(())
    }

    fn metadata(&self) -> &AppenderMetadata {
        &self.metadata
    }

    fn stats(&self) -> AppenderStats {
        AppenderStats {
            records_written: 0, // Would need to track this properly
            records_dropped: 0,
            bytes_written: 0,
            errors: 0,
            last_write: None,
            buffer_size: None,
            average_write_time_ms: 0.0,
        }
    }
}

/// File appender for logging to files
pub struct FileAppender {
    file_path: String,
    metadata: AppenderMetadata,
}

impl FileAppender {
    pub fn new(file_path: String) -> Self {
        let metadata = AppenderMetadata {
            name: "file".to_string(),
            version: "1.0.0".to_string(),
            description: format!("File appender for logging to {}", file_path),
            appender_type: "file".to_string(),
            is_async: false,
            supports_buffering: true,
            supports_rotation: false,
        };

        Self { file_path, metadata }
    }
}

impl LogAppender for FileAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        let message = format!(
            "[{}] {} - {}: {}\n",
            record.timestamp().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.message()
        );

        // Use std::fs::write with append mode
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| AppError::Logging {
                message: format!("Failed to open log file: {}", e),
                field: "file_path".to_string(),
            })?;

        file.write_all(message.as_bytes())
            .map_err(|e| AppError::Logging {
                message: format!("Failed to write to log file: {}", e),
                field: "write".to_string(),
            })?;

        Ok(())
    }

    fn is_ready(&self) -> bool {
        // Check if file is writable
        std::path::Path::new(&self.file_path)
            .parent()
            .map_or(false, |p| p.exists())
    }

    fn flush(&self) -> Result<()> {
        // File appender doesn't need explicit flushing in most cases
        Ok(())
    }

    fn metadata(&self) -> &AppenderMetadata {
        &self.metadata
    }

    fn stats(&self) -> AppenderStats {
        AppenderStats::default() // Would need to implement proper tracking
    }
}

/// Level filter for filtering logs by level
pub struct LevelFilter {
    min_level: LogLevel,
    metadata: FilterMetadata,
}

impl LevelFilter {
    pub fn new(min_level: LogLevel) -> Self {
        let metadata = FilterMetadata {
            name: "level".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Level filter for logs >= {}", min_level),
            filter_type: "level".to_string(),
        };

        Self { min_level, metadata }
    }
}

impl LogFilter for LevelFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        record.level() >= self.min_level
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Target filter for filtering logs by target/logger name
pub struct TargetFilter {
    allowed_targets: std::collections::HashSet<String>,
    metadata: FilterMetadata,
}

impl TargetFilter {
    pub fn new(allowed_targets: Vec<String>) -> Self {
        let metadata = FilterMetadata {
            name: "target".to_string(),
            version: "1.0.0".to_string(),
            description: "Target filter for specific loggers".to_string(),
            filter_type: "target".to_string(),
        };

        Self {
            allowed_targets: allowed_targets.into_iter().collect(),
            metadata,
        }
    }

    pub fn allow_target(mut self, target: String) -> Self {
        self.allowed_targets.insert(target);
        self
    }
}

impl LogFilter for TargetFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        self.allowed_targets.contains(record.target())
            || self.allowed_targets.is_empty()
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}