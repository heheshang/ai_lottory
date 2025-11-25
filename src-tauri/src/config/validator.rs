//! Configuration Validators
//!
//! Implements validation for configuration values.

use crate::config::traits::*;
use crate::config::error::ConfigError;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default configuration validator
pub struct DefaultConfigValidator {
    rules: HashMap<String, ValidationRules>,
    metadata: ValidatorMetadata,
}

impl DefaultConfigValidator {
    pub fn new() -> Self {
        let metadata = ValidatorMetadata {
            name: "default".to_string(),
            version: "1.0.0".to_string(),
            description: "Default configuration validator".to_string(),
            supported_types: vec![
                "string".to_string(),
                "integer".to_string(),
                "float".to_string(),
                "boolean".to_string(),
                "datetime".to_string(),
                "duration".to_string(),
                "list".to_string(),
                "object".to_string(),
            ],
        };

        let mut rules = HashMap::new();

        // Add some common validation rules
        rules.insert("server.port".to_string(), ValidationRules::new()
            .with_type("integer")
            .required()
            .with_min_value(ConfigValue::Integer(1))
            .with_max_value(ConfigValue::Integer(65535)));

        rules.insert("database.host".to_string(), ValidationRules::new()
            .with_type("string")
            .required());

        rules.insert("database.port".to_string(), ValidationRules::new()
            .with_type("integer")
            .with_min_value(ConfigValue::Integer(1))
            .with_max_value(ConfigValue::Integer(65535)));

        rules.insert("cache.ttl_ms".to_string(), ValidationRules::new()
            .with_type("integer")
            .with_min_value(ConfigValue::Integer(0)));

        Self {
            rules,
            metadata,
        }
    }

    pub fn add_rule(&mut self, key: String, rules: ValidationRules) {
        self.rules.insert(key, rules);
    }

    pub fn remove_rule(&mut self, key: &str) -> Option<ValidationRules> {
        self.rules.remove(key)
    }
}

impl ConfigValidator for DefaultConfigValidator {
    fn validate(&self, key: &str, value: &ConfigValue) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Get rules for this key or use default rules
        let rules = self.get_rules(key).cloned().unwrap_or_else(ValidationRules::new);

        // Check if required and value is null
        if rules.required && matches!(value, ConfigValue::Null) {
            result.errors.push(ValidationError {
                code: "REQUIRED_VALUE".to_string(),
                message: format!("Key '{}' is required but has null value", key),
                field: Some(key.to_string()),
                value: Some(value.clone()),
            });
            result.is_valid = false;
        }

        if matches!(value, ConfigValue::Null) {
            return Ok(result); // Don't validate null values further
        }

        // Check value type
        if let Some(expected_type) = &rules.value_type {
            let actual_type = value.type_name();
            if actual_type != *expected_type {
                result.errors.push(ValidationError {
                    code: "INVALID_TYPE".to_string(),
                    message: format!("Expected type '{}' but got '{}'", expected_type, actual_type),
                    field: Some(key.to_string()),
                    value: Some(value.clone()),
                });
                result.is_valid = false;
            }
        }

        // Validate string properties
        if let ConfigValue::String(s) = value {
            if let Some(min_length) = rules.min_length {
                if s.len() < min_length {
                    result.errors.push(ValidationError {
                        code: "MIN_LENGTH".to_string(),
                        message: format!("String length {} is less than minimum {}", s.len(), min_length),
                        field: Some(key.to_string()),
                        value: Some(value.clone()),
                    });
                    result.is_valid = false;
                }
            }

            if let Some(max_length) = rules.max_length {
                if s.len() > max_length {
                    result.errors.push(ValidationError {
                        code: "MAX_LENGTH".to_string(),
                        message: format!("String length {} exceeds maximum {}", s.len(), max_length),
                        field: Some(key.to_string()),
                        value: Some(value.clone()),
                    });
                    result.is_valid = false;
                }
            }

            if let Some(pattern) = &rules.pattern {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    if !regex.is_match(s) {
                        result.errors.push(ValidationError {
                            code: "PATTERN_MISMATCH".to_string(),
                            message: format!("String '{}' does not match pattern '{}'", s, pattern),
                            field: Some(key.to_string()),
                            value: Some(value.clone()),
                        });
                        result.is_valid = false;
                    }
                } else {
                    result.warnings.push(ValidationWarning {
                        code: "INVALID_PATTERN".to_string(),
                        message: format!("Invalid regex pattern: {}", pattern),
                        field: Some(key.to_string()),
                        value: Some(value.clone()),
                    });
                }
            }
        }

        // Validate numeric properties
        if let Some(min_value) = &rules.min_value {
            if !self.compare_values(value, min_value, true)? {
                result.errors.push(ValidationError {
                    code: "MIN_VALUE".to_string(),
                    message: format!("Value {:?} is less than minimum {:?}", value, min_value),
                    field: Some(key.to_string()),
                    value: Some(value.clone()),
                });
                result.is_valid = false;
            }
        }

        if let Some(max_value) = &rules.max_value {
            if !self.compare_values(value, max_value, false)? {
                result.errors.push(ValidationError {
                    code: "MAX_VALUE".to_string(),
                    message: format!("Value {:?} exceeds maximum {:?}", value, max_value),
                    field: Some(key.to_string()),
                    value: Some(value.clone()),
                });
                result.is_valid = false;
            }
        }

        // Validate allowed values
        if let Some(allowed_values) = &rules.allowed_values {
            if !allowed_values.contains(value) {
                result.errors.push(ValidationError {
                    code: "INVALID_VALUE".to_string(),
                    message: format!("Value {:?} is not in allowed values {:?}", value, allowed_values),
                    field: Some(key.to_string()),
                    value: Some(value.clone()),
                });
                result.is_valid = false;
            }
        }

        // Check custom validators (placeholder for future implementation)
        for validator_name in &rules.custom_validators {
            result.warnings.push(ValidationWarning {
                code: "CUSTOM_VALIDATOR".to_string(),
                message: format!("Custom validator '{}' not implemented", validator_name),
                field: Some(key.to_string()),
                value: Some(value.clone()),
            });
        }

        Ok(result)
    }

    fn get_rules(&self, key: &str) -> Option<&ValidationRules> {
        self.rules.get(key)
    }

    fn validate_all(&self, config: &ConfigData) -> Result<ValidationSummary> {
        let mut summary = ValidationSummary {
            total_keys: config.values.len(),
            valid_keys: 0,
            invalid_keys: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            is_valid: true,
        };

        for (key, value) in &config.values {
            let result = self.validate(key, value)?;

            if result.is_valid {
                summary.valid_keys += 1;
            } else {
                summary.invalid_keys += 1;
                summary.is_valid = false;
            }

            summary.warnings.extend(result.warnings);
            summary.errors.extend(result.errors);
        }

        Ok(summary)
    }

    fn validator_metadata(&self) -> &ValidatorMetadata {
        &self.metadata
    }
}

impl DefaultConfigValidator {
    fn compare_values(&self, value: &ConfigValue, bound: &ConfigValue, is_min: bool) -> Result<bool> {
        match (value, bound) {
            (ConfigValue::Integer(a), ConfigValue::Integer(b)) => {
                Ok(if is_min { a >= b } else { a <= b })
            }
            (ConfigValue::Float(a), ConfigValue::Float(b)) => {
                Ok(if is_min { a >= b } else { a <= b })
            }
            (ConfigValue::Float(a), ConfigValue::Integer(b)) => {
                let b_float = *b as f64;
                Ok(if is_min { a >= &b_float } else { a <= &b_float })
            }
            (ConfigValue::Integer(a), ConfigValue::Float(b)) => {
                let a_float = *a as f64;
                Ok(if is_min { &a_float >= b } else { &a_float <= b })
            }
            (ConfigValue::String(a), ConfigValue::String(b)) => {
                Ok(if is_min { a.len() >= b.len() } else { a.len() <= b.len() })
            }
            (ConfigValue::List(a), ConfigValue::List(b)) => {
                Ok(if is_min { a.len() >= b.len() } else { a.len() <= b.len() })
            }
            _ => Ok(true), // Unsupported comparison, assume valid
        }
    }
}

impl Default for DefaultConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}