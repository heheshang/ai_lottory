use crate::super_lotto::{
    models::{ValidationRequest, ValidationError},
    errors::SuperLottoError,
    analysis::{ValidationCache, NumberZone},
};

use std::collections::HashMap;
use serde_json::{json, Value};

pub struct ValidationBuilder {
    // Composable validation builder with caching
    rules: Vec<Box<dyn ValidationRule>>,
    cache: HashMap<String, ValidationCache>,
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        Self {
            rules.push(rule);
        }
    }

    pub fn build_request<T>(&self, data: T) -> ValidationRequest<T> {
        ValidationRequest {
            data,
            cache: &mut self.cache,
            rules: &self.rules,
        }
    }

    pub fn validate<T>(&self, request: ValidationRequest<T>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let cache_key = format!("{:?}", request.data);

        // Check if validation result is cached
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Ok(());
        }

        // Apply validation rules
        for rule in &self.rules {
            match rule.validate(&request) {
                Ok(_) => continue,
                Err(error) => errors.push(error),
            }
        }

        // Cache the result if no errors
        if errors.is_empty() {
            let validation_result = ValidationCache {
                is_valid: true,
                errors: Vec::new(),
                data: Some(serde_json::to_value(&request.data).unwrap_or_default()),
            };

            self.cache.insert(cache_key, validation_result);
            Ok(())
        } else {
            let validation_result = ValidationCache {
                is_valid: false,
                errors,
                data: None,
            };

            self.cache.insert(cache_key, validation_result);
            Err(errors)
        }
    }

    pub fn invalidate_cache(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

// Trait for individual validation rules
pub trait ValidationRule: Send + Sync {
    fn validate(&self, request: &ValidationRequest) -> Result<(), ValidationError>;
}

// Error types
#[derive(Debug, Clone, Serialize)]
pub enum ValidationError {
    #[error("Invalid front zone count: expected 5 numbers, got {0}")]
    InvalidFrontZoneCount(usize),
    #[error("Invalid back zone count: expected 2 numbers, got {0}")]
    InvalidBackZoneCount(usize),
    #[error("Duplicate numbers in front zone: {0:?}")]
    DuplicateFrontZoneNumbers(Vec<u32>),
    #[error("Duplicate numbers in back zone: {0:?}")]
    InvalidFrontZoneRange(u32),
    #[error("Invalid back zone range: {0:?}")]
    InvalidBackZoneRange(u32),
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    #[error("Invalid JSON data: {0}")]
    InvalidJson(String),
    #[error("Field validation failed: {field}")]
    FieldValidationError {
        field: String,
        message: String,
        value: String,
        expected: Option<String>,
        actual: String,
        error_type: ValidationError,
    },
}

// Cache entry
#[derive(Debug, Clone, Serialize)]
pub struct ValidationCache {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub data: Option<Value>,
}