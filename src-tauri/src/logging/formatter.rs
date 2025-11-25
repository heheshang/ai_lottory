//! Log Formatters
//!
//! Implements various formatters for different log output formats.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Simple text formatter for human-readable logs
pub struct SimpleFormatter {
    include_timestamp: bool,
    include_level: bool,
    include_target: bool,
    timestamp_format: String,
    metadata: FormatterMetadata,
}

impl SimpleFormatter {
    pub fn new() -> Self {
        let metadata = FormatterMetadata {
            name: "simple".to_string(),
            version: "1.0.0".to_string(),
            description: "Simple human-readable log formatter".to_string(),
            format_type: "text".to_string(),
            supports_colors: false,
            supports_structured: false,
        };

        Self {
            include_timestamp: true,
            include_level: true,
            include_target: true,
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            metadata,
        }
    }

    pub fn with_timestamp_format(mut self, format: String) -> Self {
        self.timestamp_format = format;
        self
    }

    pub fn with_timestamp(mut self, include: bool) -> Self {
        self.include_timestamp = include;
        self
    }

    pub fn with_level(mut self, include: bool) -> Self {
        self.include_level = include;
        self
    }

    pub fn with_target(mut self, include: bool) -> Self {
        self.include_target = include;
        self
    }
}

impl Default for SimpleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFormatter for SimpleFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        let mut parts = Vec::new();

        // Add timestamp
        if self.include_timestamp {
            parts.push(record.timestamp().format(&self.timestamp_format).to_string());
        }

        // Add log level
        if self.include_level {
            parts.push(record.level().as_str().to_string());
        }

        // Add target
        if self.include_target {
            parts.push(record.target().to_string());
        }

        // Add message
        parts.push(record.message().to_string());

        // Add structured fields
        if !record.fields().is_empty() {
            let field_strs: Vec<String> = record.fields()
                .iter()
                .map(|(k, v)| format!("{}={}", k, self.format_field_value(v)))
                .collect();
            parts.push(format!("[{}]", field_strs.join(", ")));
        }

        Ok(parts.join(" - "))
    }

    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String> {
        let mut base_output = self.format(record)?;

        // Add context fields
        if !context.fields.is_empty() {
            let context_strs: Vec<String> = context.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, self.format_field_value(v)))
                .collect();
            base_output.push_str(&format!(" | ctx: {}", context_strs.join(", ")));
        }

        // Add scope information
        if !context.scopes.is_empty() {
            let scope_names: Vec<String> = context.scopes
                .iter()
                .map(|s| s.name.clone())
                .collect();
            base_output.push_str(&format!(" | scopes: {}", scope_names.join(" -> ")));
        }

        Ok(base_output)
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}

impl SimpleFormatter {
    fn format_field_value(&self, value: &LogFieldValue) -> String {
        match value {
            LogFieldValue::String(s) => format!("\"{}\"", s),
            LogFieldValue::Integer(i) => i.to_string(),
            LogFieldValue::Float(f) => format!("{:.2}", f),
            LogFieldValue::Boolean(b) => b.to_string(),
            LogFieldValue::DateTime(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            LogFieldValue::Duration(d) => format!("{}ms", d.as_millis()),
            LogFieldValue::Array(arr) => {
                let values: Vec<String> = arr.iter()
                    .map(|v| self.format_field_value(v))
                    .collect();
                format!("[{}]", values.join(", "))
            }
            LogFieldValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_field_value(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            LogFieldValue::Null => "null".to_string(),
        }
    }
}

/// JSON formatter for structured machine-readable logs
pub struct JsonFormatter {
    pretty_print: bool,
    include_metadata: bool,
    metadata: FormatterMetadata,
}

impl JsonFormatter {
    pub fn new() -> Self {
        let metadata = FormatterMetadata {
            name: "json".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON structured log formatter".to_string(),
            format_type: "json".to_string(),
            supports_colors: false,
            supports_structured: true,
        };

        Self {
            pretty_print: false,
            include_metadata: true,
            metadata,
        }
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFormatter for JsonFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        let mut log_entry = serde_json::Map::new();

        // Basic fields
        log_entry.insert("timestamp".to_string(),
            serde_json::Value::String(record.timestamp().to_rfc3339()));
        log_entry.insert("level".to_string(),
            serde_json::Value::String(record.level().as_str().to_lowercase()));
        log_entry.insert("target".to_string(),
            serde_json::Value::String(record.target().to_string()));
        log_entry.insert("message".to_string(),
            serde_json::Value::String(record.message().to_string()));

        // Structured fields
        if !record.fields().is_empty() {
            let mut fields_map = serde_json::Map::new();
            for (key, value) in record.fields() {
                fields_map.insert(key.clone(), value.to_json_value());
            }
            log_entry.insert("fields".to_string(), serde_json::Value::Object(fields_map));
        }

        // Optional metadata
        if self.include_metadata {
            if let Some(module_path) = record.module_path() {
                log_entry.insert("module".to_string(),
                    serde_json::Value::String(module_path.to_string()));
            }
            if let Some(file) = record.file() {
                log_entry.insert("file".to_string(),
                    serde_json::Value::String(file.to_string()));
            }
            if let Some(line) = record.line() {
                log_entry.insert("line".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(line)));
            }
        }

        let json_result = if self.pretty_print {
            serde_json::to_value(&log_entry)
                .and_then(|v| serde_json::to_string_pretty(&v))
        } else {
            serde_json::to_value(&log_entry)
                .and_then(|v| serde_json::to_string(&v))
        };

        json_result.map_err(|e| LogError::formatter_error(
            "json_format",
            format!("Failed to serialize log entry to JSON: {}", e),
        ))
    }

    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String> {
        let mut base_json = self.format(record)?;

        // Parse the base JSON to add context
        let mut log_value: serde_json::Value = serde_json::from_str(&base_json)
            .map_err(|e| LogError::formatter_error(
                "json_format_context",
                format!("Failed to parse base JSON: {}", e),
            ))?;

        if let serde_json::Value::Object(ref mut map) = log_value {
            // Add context fields
            if !context.fields.is_empty() {
                let mut context_fields = serde_json::Map::new();
                for (key, value) in &context.fields {
                    context_fields.insert(key.clone(), value.to_json_value());
                }
                map.insert("context".to_string(), serde_json::Value::Object(context_fields));
            }

            // Add scope information
            if !context.scopes.is_empty() {
                let scopes: Vec<serde_json::Value> = context.scopes
                    .iter()
                    .map(|scope| {
                        let mut scope_obj = serde_json::Map::new();
                        scope_obj.insert("name".to_string(),
                            serde_json::Value::String(scope.name.clone()));
                        scope_obj.insert("level".to_string(),
                            serde_json::Value::String(scope.level.as_str().to_string()));

                        if !scope.fields.is_empty() {
                            let mut scope_fields = serde_json::Map::new();
                            for (key, value) in &scope.fields {
                                scope_fields.insert(key.clone(), value.to_json_value());
                            }
                            scope_obj.insert("fields".to_string(),
                                serde_json::Value::Object(scope_fields));
                        }

                        serde_json::Value::Object(scope_obj)
                    })
                    .collect();
                map.insert("scopes".to_string(), serde_json::Value::Array(scopes));
            }

            // Add context timestamp
            map.insert("context_timestamp".to_string(),
                serde_json::Value::String(context.timestamp.to_rfc3339()));
        }

        let result = if self.pretty_print {
            serde_json::to_string_pretty(&log_value)
        } else {
            serde_json::to_string(&log_value)
        };

        result.map_err(|e| LogError::formatter_error(
            "json_format_context",
            format!("Failed to serialize context-enhanced JSON: {}", e),
        ))
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}

/// Colored console formatter for development environments
pub struct ColoredFormatter {
    base_formatter: SimpleFormatter,
    use_colors: bool,
    metadata: FormatterMetadata,
}

impl ColoredFormatter {
    pub fn new() -> Self {
        let metadata = FormatterMetadata {
            name: "colored".to_string(),
            version: "1.0.0".to_string(),
            description: "Colored console log formatter".to_string(),
            format_type: "colored_text".to_string(),
            supports_colors: true,
            supports_structured: false,
        };

        Self {
            base_formatter: SimpleFormatter::new(),
            use_colors: true,
            metadata,
        }
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    pub fn with_base_formatter(mut self, formatter: SimpleFormatter) -> Self {
        self.base_formatter = formatter;
        self
    }
}

impl Default for ColoredFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFormatter for ColoredFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        let mut formatted = self.base_formatter.format(record)?;

        if self.use_colors {
            // Add level-specific colors
            let level = record.level();
            let color_code = level.ansi_color();
            let reset_code = LogLevel::ansi_reset();

            // Replace level in the formatted string with colored version
            let level_str = level.as_str();
            let colored_level = format!("{}{}{}", color_code, level_str, reset_code);
            formatted = formatted.replace(level_str, &colored_level);

            // Add timestamp color
            if self.base_formatter.include_timestamp {
                if let Some(timestamp_start) = formatted.find('[') {
                    if let Some(timestamp_end) = formatted.find(']') {
                        let timestamp = &formatted[timestamp_start..=timestamp_end];
                        let colored_timestamp = format!("\x1b[90m{}\x1b[0m", timestamp); // Gray
                        formatted = formatted.replace(timestamp, &colored_timestamp);
                    }
                }
            }
        }

        Ok(formatted)
    }

    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String> {
        let mut formatted = self.base_formatter.format_with_context(record, context)?;

        if self.use_colors {
            // Colorize context information
            if formatted.contains("ctx:") {
                let context_part = formatted.split("ctx:").nth(1)
                    .unwrap_or("")
                    .split(" | scopes:").next()
                    .unwrap_or("");

                if !context_part.is_empty() {
                    let colored_context = format!("\x1b[36mctx: {}\x1b[0m", context_part.trim()); // Cyan
                    formatted = formatted.replace(&format!("ctx: {}", context_part.trim()), &colored_context);
                }
            }

            // Colorize scope information
            if formatted.contains("scopes:") {
                if let Some(scopes_start) = formatted.find("scopes:") {
                    let scopes_part = &formatted[scopes_start..];
                    let colored_scopes = format!("\x1b[35m{}\x1b[0m", scopes_part); // Magenta
                    formatted = formatted.replace(scopes_part, &colored_scopes);
                }
            }
        }

        Ok(formatted)
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}

/// Template-based formatter with customizable output patterns
pub struct TemplateFormatter {
    template: String,
    metadata: FormatterMetadata,
}

impl TemplateFormatter {
    pub fn new(template: String) -> Self {
        let metadata = FormatterMetadata {
            name: "template".to_string(),
            version: "1.0.0".to_string(),
            description: "Template-based log formatter".to_string(),
            format_type: "template".to_string(),
            supports_colors: false,
            supports_structured: true,
        };

        Self { template, metadata }
    }

    fn apply_template(&self, record: &dyn LogRecord, context: Option<&LogContext>) -> Result<String> {
        let mut result = self.template.clone();

        // Replace standard placeholders
        result = result.replace("{timestamp}",
            &record.timestamp().format("%Y-%m-%d %H:%M:%S%.3f").to_string());
        result = result.replace("{level}", record.level().as_str());
        result = result.replace("{target}", record.target());
        result = result.replace("{message}", record.message());

        // Replace field placeholders
        for (key, value) in record.fields() {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, &self.format_field_value(value));
        }

        // Replace context placeholders if provided
        if let Some(ctx) = context {
            for (key, value) in &ctx.fields {
                let placeholder = format!("{{ctx.{}}}", key);
                result = result.replace(&placeholder, &self.format_field_value(value));
            }

            // Replace scope placeholders
            if !ctx.scopes.is_empty() {
                let scope_names: Vec<String> = ctx.scopes
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();
                result = result.replace("{scopes}", &scope_names.join(" -> "));
            }
        }

        Ok(result)
    }

    fn format_field_value(&self, value: &LogFieldValue) -> String {
        match value {
            LogFieldValue::String(s) => s.clone(),
            LogFieldValue::Integer(i) => i.to_string(),
            LogFieldValue::Float(f) => format!("{:.2}", f),
            LogFieldValue::Boolean(b) => b.to_string(),
            LogFieldValue::DateTime(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            LogFieldValue::Duration(d) => format!("{}ms", d.as_millis()),
            LogFieldValue::Array(arr) => {
                let values: Vec<String> = arr.iter()
                    .map(|v| self.format_field_value(v))
                    .collect();
                format!("[{}]", values.join(", "))
            }
            LogFieldValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_field_value(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            LogFieldValue::Null => "null".to_string(),
        }
    }
}

#[async_trait]
impl LogFormatter for TemplateFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        self.apply_template(record, None)
    }

    fn format_with_context(&self, record: &dyn LogRecord, context: &LogContext) -> Result<String> {
        self.apply_template(record, Some(context))
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}

/// Compact formatter for high-throughput scenarios
pub struct CompactFormatter {
    metadata: FormatterMetadata,
}

impl CompactFormatter {
    pub fn new() -> Self {
        let metadata = FormatterMetadata {
            name: "compact".to_string(),
            version: "1.0.0".to_string(),
            description: "Compact high-performance log formatter".to_string(),
            format_type: "compact".to_string(),
            supports_colors: false,
            supports_structured: false,
        };

        Self { metadata }
    }
}

impl Default for CompactFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFormatter for CompactFormatter {
    fn format(&self, record: &dyn LogRecord) -> Result<String> {
        let level_char = match record.level() {
            LogLevel::Trace => 'T',
            LogLevel::Debug => 'D',
            LogLevel::Info => 'I',
            LogLevel::Warn => 'W',
            LogLevel::Error => 'E',
            LogLevel::Fatal => 'F',
        };

        let timestamp = record.timestamp().format("%H%M%S%.3f");
        let target_short = record.target().split("::").last().unwrap_or(record.target());

        format!("{} {} {} {}",
            timestamp,
            level_char,
            target_short,
            record.message())
    }

    fn format_with_context(&self, record: &dyn LogRecord, _context: &LogContext) -> Result<String> {
        // Compact formatter ignores context for maximum performance
        self.format(record)
    }

    fn metadata(&self) -> &FormatterMetadata {
        &self.metadata
    }
}