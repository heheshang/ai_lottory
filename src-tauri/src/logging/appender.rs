//! Log Appenders
//!
//! Implements various appenders for different log output destinations.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use chrono::{DateTime, Utc};

/// Buffered file appender for efficient file logging
pub struct BufferedFileAppender {
    file_path: String,
    buffer: Arc<Mutex<VecDeque<String>>>,
    buffer_size: usize,
    flush_interval_ms: u64,
    metadata: AppenderMetadata,
    stats: Arc<RwLock<AppenderStats>>,
    last_flush: Arc<RwLock<DateTime<Utc>>>,
}

impl BufferedFileAppender {
    pub fn new(
        file_path: String,
        buffer_size: usize,
        flush_interval_ms: u64,
    ) -> Self {
        let metadata = AppenderMetadata {
            name: "buffered_file".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Buffered file appender for {}", file_path),
            appender_type: "file".to_string(),
            is_async: false,
            supports_buffering: true,
            supports_rotation: false,
        };

        Self {
            file_path,
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size))),
            buffer_size,
            flush_interval_ms,
            metadata,
            stats: Arc::new(RwLock::new(AppenderStats::new())),
            last_flush: Arc::new(RwLock::new(Utc::now())),
        }
    }

    async fn flush_buffer(&self) -> Result<()> {
        let mut messages = {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.drain(..).collect::<Vec<_>>()
        };

        if messages.is_empty() {
            return Ok(());
        }

        // Write all messages to file
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| LogError::io_error(
                "flush_buffer",
                Some(self.file_path.clone()),
                format!("Failed to open file for flushing: {}", e),
            ))?;

        let mut total_bytes = 0;
        for message in &messages {
            let bytes = message.as_bytes();
            file.write_all(bytes)
                .map_err(|e| LogError::io_error(
                    "flush_buffer",
                    Some(self.file_path.clone()),
                    format!("Failed to write message: {}", e),
                ))?;
            total_bytes += bytes.len();
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.records_written += messages.len() as u64;
            stats.bytes_written += total_bytes as u64;
            stats.last_write = Some(Utc::now());
        }

        // Update last flush time
        *self.last_flush.write().unwrap() = Utc::now();

        Ok(())
    }

    async fn check_buffer_size(&self) -> Result<()> {
        let buffer_len = {
            let buffer = self.buffer.lock().unwrap();
            buffer.len()
        };

        if buffer_len >= self.buffer_size {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    async fn check_flush_interval(&self) -> Result<bool> {
        let last_flush = *self.last_flush.read().unwrap();
        let elapsed = (Utc::now() - last_flush).num_milliseconds() as u64;

        if elapsed >= self.flush_interval_ms {
            self.flush_buffer().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl LogAppender for BufferedFileAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        let message = format!(
            "[{}] {} - {}: {}\n",
            record.timestamp().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.target(),
            record.message()
        );

        let mut buffer = self.buffer.lock().unwrap();

        if buffer.len() >= buffer.capacity() {
            // Buffer is full, drop the oldest message
            buffer.pop_front();
            *self.stats.write().unwrap().records_dropped += 1;
        }

        buffer.push_back(message);

        Ok(())
    }

    fn is_ready(&self) -> bool {
        // Check if file directory exists and is writable
        std::path::Path::new(&self.file_path)
            .parent()
            .map_or(false, |p| p.exists())
    }

    fn flush(&self) -> Result<()> {
        // This would need to be async in practice
        // For now, we'll use a blocking implementation
        let buffer_len = {
            let buffer = self.buffer.lock().unwrap();
            buffer.len()
        };

        if buffer_len > 0 {
            let mut buffer = self.buffer.lock().unwrap();
            let messages: Vec<String> = buffer.drain(..).collect();

            use std::fs::OpenOptions;
            use std::io::Write;

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path)
            {
                let mut total_bytes = 0;
                for message in &messages {
                    let bytes = message.as_bytes();
                    if let Err(e) = file.write_all(bytes) {
                        eprintln!("Error writing to log file: {}", e);
                        break;
                    }
                    total_bytes += bytes.len();
                }

                // Update statistics
                if let Ok(mut stats) = self.stats.try_write() {
                    stats.records_written += messages.len() as u64;
                    stats.bytes_written += total_bytes as u64;
                    stats.last_write = Some(Utc::now());
                }

                *self.last_flush.write().unwrap() = Utc::now();
            } else {
                eprintln!("Error opening log file for writing");
            }
        }

        Ok(())
    }

    fn metadata(&self) -> &AppenderMetadata {
        &self.metadata
    }

    fn stats(&self) -> AppenderStats {
        // Note: This is a simplified implementation
        // In practice, you would need to handle the lock properly
        self.stats.read().unwrap().clone()
    }
}

/// Rotating file appender for log rotation
pub struct RotatingFileAppender {
    base_path: String,
    max_file_size: u64,
    max_files: usize,
    current_file_index: Arc<RwLock<usize>>,
    current_file_size: Arc<RwLock<u64>>,
    metadata: AppenderMetadata,
    stats: Arc<RwLock<AppenderStats>>,
}

impl RotatingFileAppender {
    pub fn new(base_path: String, max_file_size: u64, max_files: usize) -> Self {
        let metadata = AppenderMetadata {
            name: "rotating_file".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Rotating file appender for {}", base_path),
            appender_type: "file".to_string(),
            is_async: false,
            supports_buffering: false,
            supports_rotation: true,
        };

        Self {
            base_path,
            max_file_size,
            max_files,
            current_file_index: Arc::new(RwLock::new(0)),
            current_file_size: Arc::new(RwLock::new(0)),
            metadata,
            stats: Arc::new(RwLock::new(AppenderStats::new())),
        }
    }

    fn get_current_file_path(&self) -> String {
        let index = *self.current_file_index.read().unwrap();
        if index == 0 {
            self.base_path.clone()
        } else {
            format!("{}.{}", self.base_path, index)
        }
    }

    fn rotate_file(&self) -> Result<()> {
        let mut index = self.current_file_index.write().unwrap();
        *index = (*index + 1) % self.max_files;

        // Remove the oldest file if we've reached the max
        if *index == 0 {
            let oldest_file = format!("{}.{}", self.base_path, self.max_files);
            if let Err(e) = std::fs::remove_file(&oldest_file) {
                tracing::warn!("Failed to remove old log file '{}': {}", oldest_file, e);
            }
        }

        // Reset current file size
        *self.current_file_size.write().unwrap() = 0;

        tracing::info!("Rotated log file, new index: {}", index);
        Ok(())
    }

    async fn check_rotation(&self) -> Result<bool> {
        let current_size = *self.current_file_size.read().unwrap();
        if current_size >= self.max_file_size {
            self.rotate_file().await?;
            return Ok(true);
        }
        Ok(false)
    }
}

#[async_trait]
impl LogAppender for RotatingFileAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        let message = format!(
            "[{}] {} - {}: {}\n",
            record.timestamp().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.target(),
            record.message()
        );

        let message_bytes = message.as_bytes().len() as u64;

        // Check if rotation is needed
        let current_size = *self.current_file_size.read().unwrap();
        if current_size + message_bytes > self.max_file_size {
            // Perform rotation
            if let Err(e) = self.rotate_file() {
                return Err(LogError::appender_error(
                    "rotating_file",
                    "append",
                    format!("Failed to rotate file: {}", e),
                ));
            }
        }

        // Write to current file
        use std::fs::OpenOptions;
        use std::io::Write;

        let file_path = self.get_current_file_path();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| LogError::io_error(
                "append",
                Some(file_path),
                format!("Failed to open log file: {}", e),
            ))?;

        file.write_all(message.as_bytes())
            .map_err(|e| LogError::io_error(
                "append",
                Some(file_path),
                format!("Failed to write message: {}", e),
            ))?;

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.records_written += 1;
            stats.bytes_written += message_bytes as u64;
            stats.last_write = Some(Utc::now());
        }

        // Update current file size
        *self.current_file_size.write().unwrap() = current_size + message_bytes;

        Ok(())
    }

    fn is_ready(&self) -> bool {
        // Check if base directory exists and is writable
        std::path::Path::new(&self.base_path)
            .parent()
            .map_or(false, |p| p.exists())
    }

    fn flush(&self) -> Result<()> {
        // Rotating file appender doesn't typically need explicit flushing
        // Each write is immediately persisted
        Ok(())
    }

    fn metadata(&self) -> &AppenderMetadata {
        &self.metadata
    }

    fn stats(&self) -> AppenderStats {
        // Simplified implementation
        AppenderStats {
            records_written: 0,
            records_dropped: 0,
            bytes_written: 0,
            errors: 0,
            last_write: None,
            buffer_size: None,
            average_write_time_ms: 0.0,
        }
    }
}

/// JSON structured appender for machine-readable logs
pub struct JsonAppender {
    inner: Box<dyn LogAppender>,
    metadata: AppenderMetadata,
}

impl JsonAppender {
    pub fn new(inner: Box<dyn LogAppender>) -> Self {
        let metadata = AppenderMetadata {
            name: "json".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON structured logging appender".to_string(),
            appender_type: "json".to_string(),
            is_async: false,
            supports_buffering: false,
            supports_rotation: false,
        };

        Self { inner, metadata }
    }
}

#[async_trait]
impl LogAppender for JsonAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        // Convert the record to JSON
        let json_record = JsonLogRecord::from_record(record);
        let json_string = serde_json::to_string(&json_record)
            .map_err(|e| LogError::serialization_error(
                "json_append",
                format!("Failed to serialize log record to JSON: {}", e),
            ))?;

        // Create a temporary record with JSON message
        let json_record = SimpleLogRecord::new(
            record.level(),
            record.target().to_string(),
            json_string,
        );

        self.inner.append(&json_record)
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }

    fn metadata(&self) -> &AppenderMetadata {
        &self.metadata
    }

    fn stats(&self) -> AppenderStats {
        self.inner.stats()
    }
}

/// Async log appender wrapper for non-blocking operations
pub struct AsyncAppender {
    inner: Arc<dyn LogAppender>,
    sender: tokio::sync::mpsc::UnboundedSender<Box<dyn LogRecord>>,
    handle: tokio::task::JoinHandle<()>,
    stats: Arc<RwLock<AppenderStats>>,
}

impl AsyncAppender {
    pub fn new(inner: Arc<dyn LogAppender>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let stats = Arc::new(RwLock::new(AppenderStats::new()));

        let handle = {
            let inner_clone = inner.clone();
            let stats_clone = stats.clone();
            let receiver = receiver;

            tokio::spawn(async move {
                while let Some(record) = receiver.recv().await {
                    match inner_clone.append(&*record) {
                        Ok(()) => {
                            if let Ok(mut stats) = stats_clone.try_write() {
                                stats.records_written += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error in async appender: {}", e);
                            if let Ok(mut stats) = stats_clone.try_write() {
                                stats.errors += 1;
                            }
                        }
                    }
                }
            })
        };

        Self {
            inner,
            sender,
            handle,
            stats,
        }
    }
}

#[async_trait]
impl LogAppender for AsyncAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        // Clone the record and send it to the async task
        // In practice, you'd need to implement proper cloning
        let record_box = Box::new(SimpleLogRecord::new(
            record.level(),
            record.target().to_string(),
            record.message().to_string(),
        ));

        self.sender
            .send(record_box)
            .map_err(|e| LogError::channel_error(
                "async_append",
                format!("Failed to send record to async task: {}", e),
            ))
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    fn flush(&self) -> Result<()> {
        // Send a flush signal to the async task
        // This is a simplified implementation
        self.inner.flush()
    }

    fn metadata(&self) -> &AppenderMetadata {
        self.inner.metadata()
    }

    fn stats(&self) -> AppenderStats {
        // Return stats from the in-memory tracking
        self.stats.read().unwrap().clone()
    }
}

/// Metrics collecting appender wrapper
pub struct MetricsCollectingAppender {
    inner: Arc<dyn LogAppender>,
    metrics_collector: Option<Arc<dyn LogMetricsCollector>>,
}

impl MetricsCollectingAppender {
    pub fn new(inner: Arc<dyn LogAppender>) -> Self {
        Self {
            inner,
            metrics_collector: None,
        }
    }

    pub fn with_metrics_collector(mut self, metrics_collector: Arc<dyn LogMetricsCollector>) -> Self {
        self.metrics_collector = Some(metrics_collector);
        self
    }
}

#[async_trait]
impl LogAppender for MetricsCollectingAppender {
    fn append(&self, record: &dyn LogRecord) -> Result<()> {
        // Log to the inner appender
        let result = self.inner.append(record);

        // Collect metrics if collector is available
        if let (Ok(()), Some(metrics)) = (&result, &self.metrics_collector) {
            let record_clone = SimpleLogRecord::new(
                record.level(),
                record.target().to_string(),
                record.message().to_string(),
            );
            if let Err(e) = metrics.record_log(&*record_clone).await {
                eprintln!("Metrics collection error: {}", e);
            }
        }

        result
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    fn flush(&self) -> Result<()> {
        let result = self.inner.flush();

        // Flush metrics if available
        if let (Ok(()), Some(metrics)) = (&result, &self.metrics_collector) {
            if let Err(e) = metrics.flush().await {
                eprintln!("Metrics flush error: {}", e);
            }
        }

        result
    }

    fn metadata(&self) -> &AppMetadata {
        self.inner.metadata()
    }

    fn stats(&self) -> AppenderStats {
        self.inner.stats()
    }
}

/// JSON log record for structured logging
#[derive(Debug, Serialize, Deserialize)]
struct JsonLogRecord {
    timestamp: DateTime<Utc>,
    level: String,
    target: String,
    message: String,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    fields: std::collections::HashMap<String, crate::logging::traits::LogFieldValue>,
    id: Option<String>,
    span_id: Option<String>,
    trace_id: Option<String>,
    parent_span_id: Option<String>,
}

impl JsonLogRecord {
    fn from_record(record: &dyn LogRecord) -> Self {
        let mut fields = std::collections::HashMap::new();

        // Convert record fields
        for (key, value) in record.fields() {
            fields.insert(key.clone(), value.clone());
        }

        Self {
            timestamp: record.timestamp(),
            level: record.level().as_str().to_string(),
            target: record.target().to_string(),
            message: record.message().to_string(),
            module_path: record.module_path().map(|s| s.to_string()),
            file: record.file().map(|s| s.to_string()),
            line: record.line(),
            fields,
            id: record.id().map(|s| s.to_string()),
            span_id: record.span_id().map(|s| s.to_string()),
            trace_id: record.trace_id().map(|s| s.to_string()),
            parent_span_id: record.parent_span_id().map(|s| s.to_string()),
        }
    }
}