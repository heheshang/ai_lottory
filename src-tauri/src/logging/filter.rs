//! Log Filters
//!
//! Implements various filters for log record processing and routing.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::collections::HashSet;
use regex::Regex;

/// Level filter for log level based filtering
pub struct LevelFilter {
    min_level: LogLevel,
    metadata: FilterMetadata,
}

impl LevelFilter {
    pub fn new(min_level: LogLevel) -> Self {
        let metadata = FilterMetadata {
            name: "level".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Filter logs at or above {} level", min_level.as_str()),
            filter_type: "level".to_string(),
        };

        Self { min_level, metadata }
    }
}

#[async_trait]
impl LogFilter for LevelFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        record.level() >= self.min_level
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Target filter for logger name based filtering
pub struct TargetFilter {
    allowed_targets: HashSet<String>,
    denied_targets: HashSet<String>,
    allow_by_default: bool,
    metadata: FilterMetadata,
}

impl TargetFilter {
    pub fn new() -> Self {
        let metadata = FilterMetadata {
            name: "target".to_string(),
            version: "1.0.0".to_string(),
            description: "Filter logs by target/logger name".to_string(),
            filter_type: "target".to_string(),
        };

        Self {
            allowed_targets: HashSet::new(),
            denied_targets: HashSet::new(),
            allow_by_default: true,
            metadata,
        }
    }

    pub fn allow(mut self, target: String) -> Self {
        self.allowed_targets.insert(target);
        self
    }

    pub fn deny(mut self, target: String) -> Self {
        self.denied_targets.insert(target);
        self
    }

    pub fn allow_by_default(mut self, allow: bool) -> Self {
        self.allow_by_default = allow;
        self
    }

    pub fn add_allowed(&mut self, target: String) {
        self.allowed_targets.insert(target);
    }

    pub fn add_denied(&mut self, target: String) {
        self.denied_targets.insert(target);
    }
}

impl Default for TargetFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFilter for TargetFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        let target = record.target();

        // Check deny list first (deny takes precedence)
        if self.denied_targets.contains(target) {
            return false;
        }

        // Check allow list
        if !self.allowed_targets.is_empty() {
            self.allowed_targets.contains(target)
        } else {
            self.allow_by_default
        }
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Message filter for content-based filtering
pub struct MessageFilter {
    include_patterns: Vec<Regex>,
    exclude_patterns: Vec<Regex>,
    metadata: FilterMetadata,
}

impl MessageFilter {
    pub fn new() -> Self {
        let metadata = FilterMetadata {
            name: "message".to_string(),
            version: "1.0.0".to_string(),
            description: "Filter logs by message content patterns".to_string(),
            filter_type: "message".to_string(),
        };

        Self {
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            metadata,
        }
    }

    pub fn include_pattern(mut self, pattern: Regex) -> Self {
        self.include_patterns.push(pattern);
        self
    }

    pub fn exclude_pattern(mut self, pattern: Regex) -> Self {
        self.exclude_patterns.push(pattern);
        self
    }

    pub fn add_include_pattern(&mut self, pattern: Regex) {
        self.include_patterns.push(pattern);
    }

    pub fn add_exclude_pattern(&mut self, pattern: Regex) {
        self.exclude_patterns.push(pattern);
    }
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFilter for MessageFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        let message = record.message();

        // Check exclude patterns first (exclude takes precedence)
        for pattern in &self.exclude_patterns {
            if pattern.is_match(message) {
                return false;
            }
        }

        // Check include patterns
        if !self.include_patterns.is_empty() {
            for pattern in &self.include_patterns {
                if pattern.is_match(message) {
                    return true;
                }
            }
            return false;
        }

        true
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Field filter for structured field based filtering
pub struct FieldFilter {
    field_conditions: Vec<FieldCondition>,
    require_all_conditions: bool,
    metadata: FilterMetadata,
}

#[derive(Debug, Clone)]
pub struct FieldCondition {
    pub field_name: String,
    pub operator: FieldOperator,
    pub value: LogFieldValue,
}

#[derive(Debug, Clone)]
pub enum FieldOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
    Exists,
    NotExists,
}

impl FieldFilter {
    pub fn new() -> Self {
        let metadata = FilterMetadata {
            name: "field".to_string(),
            version: "1.0.0".to_string(),
            description: "Filter logs by structured field values".to_string(),
            filter_type: "field".to_string(),
        };

        Self {
            field_conditions: Vec::new(),
            require_all_conditions: true,
            metadata,
        }
    }

    pub fn condition(mut self, condition: FieldCondition) -> Self {
        self.field_conditions.push(condition);
        self
    }

    pub fn require_all(mut self, require_all: bool) -> Self {
        self.require_all_conditions = require_all;
        self
    }

    pub fn add_condition(&mut self, condition: FieldCondition) {
        self.field_conditions.push(condition);
    }
}

impl Default for FieldFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFilter for FieldFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        if self.field_conditions.is_empty() {
            return true;
        }

        let mut results = Vec::new();

        for condition in &self.field_conditions {
            let result = match record.get_field(&condition.field_name) {
                Some(field_value) => self.evaluate_condition(field_value, condition),
                None => matches!(condition.operator, FieldOperator::NotExists),
            };
            results.push(result);
        }

        if self.require_all_conditions {
            results.iter().all(|&result| result)
        } else {
            results.iter().any(|&result| result)
        }
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

impl FieldFilter {
    fn evaluate_condition(&self, field_value: &LogFieldValue, condition: &FieldCondition) -> bool {
        match &condition.operator {
            FieldOperator::Equals => field_value == &condition.value,
            FieldOperator::NotEquals => field_value != &condition.value,
            FieldOperator::GreaterThan => self.compare_values(field_value, &condition.value) > 0,
            FieldOperator::LessThan => self.compare_values(field_value, &condition.value) < 0,
            FieldOperator::Contains => self.string_contains(field_value, &condition.value),
            FieldOperator::StartsWith => self.string_starts_with(field_value, &condition.value),
            FieldOperator::EndsWith => self.string_ends_with(field_value, &condition.value),
            FieldOperator::Exists => true,
            FieldOperator::NotExists => false,
        }
    }

    fn compare_values(&self, a: &LogFieldValue, b: &LogFieldValue) -> i32 {
        use std::cmp::Ordering;

        match (a, b) {
            (LogFieldValue::Integer(a_i), LogFieldValue::Integer(b_i)) => {
                match a_i.cmp(b_i) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                }
            }
            (LogFieldValue::Float(a_f), LogFieldValue::Float(b_f)) => {
                match a_f.partial_cmp(b_f) {
                    Some(Ordering::Less) => -1,
                    Some(Ordering::Equal) => 0,
                    Some(Ordering::Greater) => 1,
                    None => 0,
                }
            }
            (LogFieldValue::String(a_s), LogFieldValue::String(b_s)) => {
                match a_s.cmp(b_s) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                }
            }
            _ => 0, // Cannot compare different types
        }
    }

    fn string_contains(&self, a: &LogFieldValue, b: &LogFieldValue) -> bool {
        match (a, b) {
            (LogFieldValue::String(a_s), LogFieldValue::String(b_s)) => a_s.contains(b_s),
            _ => false,
        }
    }

    fn string_starts_with(&self, a: &LogFieldValue, b: &LogFieldValue) -> bool {
        match (a, b) {
            (LogFieldValue::String(a_s), LogFieldValue::String(b_s)) => a_s.starts_with(b_s),
            _ => false,
        }
    }

    fn string_ends_with(&self, a: &LogFieldValue, b: &LogFieldValue) -> bool {
        match (a, b) {
            (LogFieldValue::String(a_s), LogFieldValue::String(b_s)) => a_s.ends_with(b_s),
            _ => false,
        }
    }
}

/// Time-based filter for temporal log filtering
pub struct TimeFilter {
    start_time: Option<chrono::DateTime<Utc>>,
    end_time: Option<chrono::DateTime<Utc>>,
    metadata: FilterMetadata,
}

impl TimeFilter {
    pub fn new() -> Self {
        let metadata = FilterMetadata {
            name: "time".to_string(),
            version: "1.0.0".to_string(),
            description: "Filter logs by timestamp range".to_string(),
            filter_type: "time".to_string(),
        };

        Self {
            start_time: None,
            end_time: None,
            metadata,
        }
    }

    pub fn start_time(mut self, time: chrono::DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    pub fn end_time(mut self, time: chrono::DateTime<Utc>) -> Self {
        self.end_time = Some(time);
        self
    }

    pub fn time_range(mut self, start: chrono::DateTime<Utc>, end: chrono::DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }
}

impl Default for TimeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LogFilter for TimeFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        let record_time = record.timestamp();

        // Check start time
        if let Some(start) = self.start_time {
            if record_time < start {
                return false;
            }
        }

        // Check end time
        if let Some(end) = self.end_time {
            if record_time > end {
                return false;
            }
        }

        true
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Composite filter that combines multiple filters
pub struct CompositeFilter {
    filters: Vec<Box<dyn LogFilter>>,
    operation: FilterOperation,
    metadata: FilterMetadata,
}

#[derive(Debug, Clone)]
pub enum FilterOperation {
    And,  // All filters must pass
    Or,   // At least one filter must pass
    Not,  // Negates the result of the first filter
}

impl CompositeFilter {
    pub fn new(operation: FilterOperation) -> Self {
        let metadata = FilterMetadata {
            name: "composite".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Composite filter with {:?} operation", operation),
            filter_type: "composite".to_string(),
        };

        Self {
            filters: Vec::new(),
            operation,
            metadata,
        }
    }

    pub fn add_filter(mut self, filter: Box<dyn LogFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn with_filters(mut self, filters: Vec<Box<dyn LogFilter>>) -> Self {
        self.filters = filters;
        self
    }
}

#[async_trait]
impl LogFilter for CompositeFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        match self.operation {
            FilterOperation::And => {
                self.filters.iter().all(|filter| filter.should_log(record))
            }
            FilterOperation::Or => {
                self.filters.iter().any(|filter| filter.should_log(record))
            }
            FilterOperation::Not => {
                if let Some(first_filter) = self.filters.first() {
                    !first_filter.should_log(record)
                } else {
                    true // No filter means no restriction
                }
            }
        }
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Rate-limiting filter to prevent log spam
pub struct RateLimitFilter {
    max_logs_per_window: u32,
    window_duration_ms: u64,
    current_window: std::sync::Arc<std::sync::Mutex<RateLimitWindow>>,
    metadata: FilterMetadata,
}

#[derive(Debug)]
struct RateLimitWindow {
    start_time: chrono::DateTime<Utc>,
    log_count: u32,
    dropped_count: u64,
}

impl RateLimitFilter {
    pub fn new(max_logs_per_window: u32, window_duration_ms: u64) -> Self {
        let metadata = FilterMetadata {
            name: "rate_limit".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Rate limit filter: {} logs per {}ms", max_logs_per_window, window_duration_ms),
            filter_type: "rate_limit".to_string(),
        };

        Self {
            max_logs_per_window,
            window_duration_ms,
            current_window: std::sync::Arc::new(std::sync::Mutex::new(RateLimitWindow {
                start_time: chrono::Utc::now(),
                log_count: 0,
                dropped_count: 0,
            })),
            metadata,
        }
    }

    pub fn get_dropped_count(&self) -> u64 {
        if let Ok(window) = self.current_window.lock() {
            window.dropped_count
        } else {
            0
        }
    }

    pub fn reset_counters(&self) {
        if let Ok(mut window) = self.current_window.lock() {
            window.log_count = 0;
            window.dropped_count = 0;
            window.start_time = chrono::Utc::now();
        }
    }
}

#[async_trait]
impl LogFilter for RateLimitFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        let record_time = record.timestamp();

        if let Ok(mut window) = self.current_window.lock() {
            let window_elapsed = (record_time - window.start_time).num_milliseconds() as u64;

            // Reset window if it has expired
            if window_elapsed >= self.window_duration_ms {
                window.start_time = record_time;
                window.log_count = 1;
                true
            } else if window.log_count < self.max_logs_per_window {
                window.log_count += 1;
                true
            } else {
                window.dropped_count += 1;
                false
            }
        } else {
            true // If we can't get the lock, allow the log through
        }
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

/// Sampling filter for probabilistic log sampling
pub struct SamplingFilter {
    sample_rate: f64,
    random_seed: u64,
    metadata: FilterMetadata,
}

impl SamplingFilter {
    pub fn new(sample_rate: f64) -> Self {
        let metadata = FilterMetadata {
            name: "sampling".to_string(),
            version: "1.0.0".to_string(),
            description: format!("Sampling filter: {:.2}% of logs", sample_rate * 100.0),
            filter_type: "sampling".to_string(),
        };

        Self {
            sample_rate: sample_rate.clamp(0.0, 1.0),
            random_seed: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            metadata,
        }
    }
}

#[async_trait]
impl LogFilter for SamplingFilter {
    fn should_log(&self, record: &dyn LogRecord) -> bool {
        // Simple hash-based sampling for deterministic behavior
        let hash = self.simple_hash(
            record.target(),
            record.message(),
            record.timestamp().timestamp_nanos_opt().unwrap_or(0),
        );

        (hash as f64 / u64::MAX as f64) < self.sample_rate
    }

    fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }
}

impl SamplingFilter {
    fn simple_hash(&self, target: &str, message: &str, timestamp: i64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.random_seed.hash(&mut hasher);
        target.hash(&mut hasher);
        message.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        hasher.finish()
    }
}