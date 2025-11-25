//! Configuration Providers
//!
//! Implements various configuration providers for different sources.

use crate::config::traits::*;
use crate::config::error::ConfigError;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// In-memory configuration provider
pub struct MemoryConfigProvider {
    data: Arc<RwLock<HashMap<String, ConfigValue>>>,
    metadata: Arc<RwLock<HashMap<String, ConfigMetadata>>>,
    provider_metadata: ProviderMetadata,
}

impl MemoryConfigProvider {
    pub fn new(name: impl Into<String>) -> Self {
        let metadata = ProviderMetadata {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: "In-memory configuration provider".to_string(),
            capabilities: vec!["read".to_string(), "write".to_string(), "fast".to_string()],
            supported_formats: vec![],
            read_only: false,
            supports_watching: false,
            supports_encryption: false,
        };

        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            provider_metadata: metadata,
        }
    }

    pub async fn set_initial_values(&mut self, values: HashMap<String, ConfigValue>) {
        let mut data = self.data.write().await;
        let mut metadata = self.metadata.write().await;

        for (key, value) in values {
            let config_metadata = ConfigMetadata::new(key.clone(), "memory".to_string())
                .with_value_type(value.type_name());

            data.insert(key.clone(), value);
            metadata.insert(key, config_metadata);
        }
    }
}

#[async_trait]
impl ConfigProvider for MemoryConfigProvider {
    async fn get(&self, key: &str) -> Result<Option<ConfigValue>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: ConfigValue) -> Result<()> {
        let mut data = self.data.write().await;
        let mut metadata = self.metadata.write().await;

        let mut config_metadata = metadata
            .get(key)
            .cloned()
            .unwrap_or_else(|| ConfigMetadata::new(key.to_string(), "memory".to_string()));

        config_metadata.touch();
        config_metadata.value_type = value.type_name();

        data.insert(key.to_string(), value.clone());
        metadata.insert(key.to_string(), config_metadata);

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write().await;
        let mut metadata = self.metadata.write().await;

        let data_removed = data.remove(key).is_some();
        let metadata_removed = metadata.remove(key).is_some();

        Ok(data_removed || metadata_removed)
    }

    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        let data = self.data.read().await;

        let keys: Vec<String> = if let Some(pattern) = pattern {
            // Simple glob-like pattern matching
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                data.keys()
                    .filter(|key| regex.is_match(key))
                    .cloned()
                    .collect()
            } else {
                // Fallback to string contains
                data.keys()
                    .filter(|key| key.contains(pattern))
                    .cloned()
                    .collect()
            }
        } else {
            data.keys().cloned().collect()
        };

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<ConfigMetadata>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(key).cloned())
    }

    fn provider_metadata(&self) -> &ProviderMetadata {
        &self.provider_metadata
    }

    async fn reload(&mut self) -> Result<()> {
        // Memory provider doesn't need reloading
        Ok(())
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let data = self.data.read().await;
        let details = {
            let mut map = HashMap::new();
            map.insert("key_count".to_string(), data.len().to_string());
            map
        };

        Ok(ProviderHealth {
            status: HealthStatus::Healthy,
            message: "Memory provider is healthy".to_string(),
            last_check: Utc::now(),
            response_time_ms: Some(1), // Very fast
            details,
        })
    }
}

/// Environment variable configuration provider
pub struct EnvironmentConfigProvider {
    prefix: Option<String>,
    separator: Option<String>,
    provider_metadata: ProviderMetadata,
}

impl EnvironmentConfigProvider {
    pub fn new(prefix: Option<String>, separator: Option<String>) -> Self {
        let metadata = ProviderMetadata {
            name: "environment".to_string(),
            version: "1.0.0".to_string(),
            description: "Environment variable configuration provider".to_string(),
            capabilities: vec!["read".to_string(), "system".to_string()],
            supported_formats: vec![],
            read_only: true,
            supports_watching: false,
            supports_encryption: false,
        };

        Self {
            prefix,
            separator: separator.or_else(|| ".".to_string().into()),
            provider_metadata: metadata,
        }
    }

    fn normalize_key(&self, env_key: &str) -> Option<String> {
        let env_key = env_key.to_lowercase();

        let start_pos = if let Some(prefix) = &self.prefix {
            let prefix = prefix.to_lowercase();
            env_key.strip_prefix(&prefix)? + 1 // Remove the separator
        } else {
            0
        };

        let key = &env_key[start_pos..];
        let separator = self.separator.as_deref().unwrap_or("_");

        Some(key.replace(separator, "."))
    }

    fn format_key(&self, key: &str) -> String {
        let separator = self.separator.as_deref().unwrap_or("_");
        let formatted = key.replace(".", separator).to_uppercase();

        if let Some(prefix) = &self.prefix {
            format!("{}{}{}", prefix, separator, formatted)
        } else {
            formatted
        }
    }
}

#[async_trait]
impl ConfigProvider for EnvironmentConfigProvider {
    async fn get(&self, key: &str) -> Result<Option<ConfigValue>> {
        let env_key = self.format_key(key);

        match std::env::var(&env_key) {
            Ok(value) => {
                let config_value = self.parse_env_value(&value)?;
                Ok(Some(config_value))
            }
            Err(_) => Ok(None),
        }
    }

    async fn set(&self, _key: &str, _value: ConfigValue) -> Result<()> {
        Err(AppError::Config {
            message: "Environment provider is read-only".to_string(),
            field: "provider".to_string(),
        })
    }

    async fn delete(&self, _key: &str) -> Result<bool> {
        Err(AppError::Config {
            message: "Environment provider is read-only".to_string(),
            field: "provider".to_string(),
        })
    }

    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        let mut keys = Vec::new();

        for (env_key, value) in std::env::vars() {
            if let Some(normalized_key) = self.normalize_key(&env_key) {
                if let Some(pattern) = pattern {
                    if !normalized_key.contains(pattern) {
                        continue;
                    }
                }
                keys.push(normalized_key);
            }
        }

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let env_key = self.format_key(key);
        Ok(std::env::var(&env_key).is_ok())
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<ConfigMetadata>> {
        let exists = self.exists(key).await?;
        if !exists {
            return Ok(None);
        }

        let env_key = self.format_key(key);
        let value = std::env::var(&env_key)?;
        let config_value = self.parse_env_value(&value)?;

        Ok(Some(ConfigMetadata::new(key.to_string(), "environment".to_string())
            .with_value_type(config_value.type_name())))
    }

    fn provider_metadata(&self) -> &ProviderMetadata {
        &self.provider_metadata
    }

    async fn reload(&mut self) -> Result<()> {
        // Environment variables are automatically reloaded
        Ok(())
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let details = {
            let mut map = HashMap::new();
            map.insert("provider_type".to_string(), "environment".to_string());
            if let Some(prefix) = &self.prefix {
                map.insert("prefix".to_string(), prefix.clone());
            }
            map
        };

        Ok(ProviderHealth {
            status: HealthStatus::Healthy,
            message: "Environment provider is healthy".to_string(),
            last_check: Utc::now(),
            response_time_ms: Some(5), // Very fast
            details,
        })
    }
}

impl EnvironmentConfigProvider {
    fn parse_env_value(&self, value: &str) -> Result<ConfigValue> {
        // Try to parse as different types in order of preference
        if value.to_lowercase() == "true" {
            return Ok(ConfigValue::Boolean(true));
        }
        if value.to_lowercase() == "false" {
            return Ok(ConfigValue::Boolean(false));
        }

        // Try integer
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(ConfigValue::Integer(int_val));
        }

        // Try float
        if let Ok(float_val) = value.parse::<f64>() {
            return Ok(ConfigValue::Float(float_val));
        }

        // Try JSON
        if value.starts_with('{') || value.starts_with('[') {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(value) {
                return Ok(self.json_value_to_config_value(json_value));
            }
        }

        // Default to string
        Ok(ConfigValue::String(value.to_string()))
    }

    fn json_value_to_config_value(&self, json_value: serde_json::Value) -> ConfigValue {
        match json_value {
            serde_json::Value::Null => ConfigValue::Null,
            serde_json::Value::Bool(b) => ConfigValue::Boolean(b),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    ConfigValue::Integer(n.as_i64().unwrap())
                } else {
                    ConfigValue::Float(n.as_f64().unwrap())
                }
            }
            serde_json::Value::String(s) => ConfigValue::String(s),
            serde_json::Value::Array(arr) => {
                let config_values: Vec<ConfigValue> = arr
                    .into_iter()
                    .map(|v| self.json_value_to_config_value(v))
                    .collect();
                ConfigValue::List(config_values)
            }
            serde_json::Value::Object(obj) => {
                let config_object: HashMap<String, ConfigValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, self.json_value_to_config_value(v)))
                    .collect();
                ConfigValue::Object(config_object)
            }
        }
    }
}

/// File configuration provider
pub struct FileConfigProvider {
    file_path: String,
    format: ConfigFormat,
    data: Arc<RwLock<HashMap<String, ConfigValue>>>,
    metadata: Arc<RwLock<HashMap<String, ConfigMetadata>>>,
    provider_metadata: ProviderMetadata,
    last_modified: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl FileConfigProvider {
    pub fn new(file_path: String, format: ConfigFormat) -> Self {
        let metadata = ProviderMetadata {
            name: "file".to_string(),
            version: "1.0.0".to_string(),
            description: format!("File configuration provider for {}", file_path).to_string(),
            capabilities: vec!["read".to_string(), "write".to_string(), "watching".to_string()],
            supported_formats: vec![format],
            read_only: false,
            supports_watching: true,
            supports_encryption: false,
        };

        Self {
            file_path,
            format,
            data: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            provider_metadata: metadata,
            last_modified: Arc::new(RwLock::new(None)),
        }
    }

    async fn load_file(&self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.file_path).await
            .map_err(|e| AppError::Config {
                message: format!("Failed to read config file: {}", e),
                field: "file_path".to_string(),
            })?;

        let (values, metadata) = self.parse_content(&content)?;

        let mut data = self.data.write().await;
        let mut config_metadata = self.metadata.write().await;

        data.clear();
        config_metadata.clear();

        for (key, value) in values {
            let meta = metadata
                .get(&key)
                .cloned()
                .unwrap_or_else(|| ConfigMetadata::new(key.clone(), self.file_path.clone())
                    .with_value_type(value.type_name()));

            data.insert(key, value);
            config_metadata.insert(key, meta);
        }

        // Update last modified time
        if let Ok(metadata) = tokio::fs::metadata(&self.file_path).await {
            if let Ok(modified) = metadata.modified() {
                let datetime: DateTime<Utc> = modified.into();
                *self.last_modified.write().await = Some(datetime);
            }
        }

        Ok(())
    }

    fn parse_content(&self, content: &str) -> Result<(HashMap<String, ConfigValue>, HashMap<String, ConfigMetadata>)> {
        match self.format {
            ConfigFormat::Json => {
                let json_value: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse JSON: {}", e),
                        field: "content".to_string(),
                    })?;

                let values = self.json_value_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse YAML: {}", e),
                        field: "content".to_string(),
                    })?;

                let json_value = serde_yaml_to_json(&yaml_value);
                let values = self.json_value_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse TOML: {}", e),
                        field: "content".to_string(),
                    })?;

                let json_value = toml_to_json(&toml_value);
                let values = self.json_value_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            _ => Err(AppError::Config {
                message: format!("Unsupported format: {:?}", self.format),
                field: "format".to_string(),
            }),
        }
    }

    fn json_value_to_flat_map(&self, value: &serde_json::Value, prefix: &str) -> Result<HashMap<String, ConfigValue>> {
        let mut result = HashMap::new();

        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    if val.is_object() || val.is_array() {
                        let nested_result = self.json_value_to_flat_map(val, &new_key)?;
                        result.extend(nested_result);
                    } else {
                        let config_value = self.json_value_to_config_value(val.clone());
                        result.insert(new_key, config_value);
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_key = if prefix.is_empty() {
                        format!("{}", index)
                    } else {
                        format!("{}.{}", prefix, index)
                    };

                    if val.is_object() || val.is_array() {
                        let nested_result = self.json_value_to_flat_map(val, &new_key)?;
                        result.extend(nested_result);
                    } else {
                        let config_value = self.json_value_to_config_value(val.clone());
                        result.insert(new_key, config_value);
                    }
                }
            }
            _ => {
                if !prefix.is_empty() {
                    let config_value = self.json_value_to_config_value(value.clone());
                    result.insert(prefix.to_string(), config_value);
                }
            }
        }

        Ok(result)
    }

    fn json_value_to_config_value(&self, json_value: serde_json::Value) -> ConfigValue {
        match json_value {
            serde_json::Value::Null => ConfigValue::Null,
            serde_json::Value::Bool(b) => ConfigValue::Boolean(b),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    ConfigValue::Integer(n.as_i64().unwrap())
                } else {
                    ConfigValue::Float(n.as_f64().unwrap())
                }
            }
            serde_json::Value::String(s) => ConfigValue::String(s),
            serde_json::Value::Array(arr) => {
                let config_values: Vec<ConfigValue> = arr
                    .into_iter()
                    .map(|v| self.json_value_to_config_value(v))
                    .collect();
                ConfigValue::List(config_values)
            }
            serde_json::Value::Object(obj) => {
                let config_object: HashMap<String, ConfigValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, self.json_value_to_config_value(v)))
                    .collect();
                ConfigValue::Object(config_object)
            }
        }
    }

    async fn save_to_file(&self, values: &HashMap<String, ConfigValue>) -> Result<()> {
        // Convert flat map back to nested JSON structure
        let json_value = self.flat_map_to_json_value(values)?;
        let content = match self.format {
            ConfigFormat::Json => serde_json::to_string_pretty(&json_value)?,
            ConfigFormat::Yaml => serde_yaml::to_string(&json_value)?,
            _ => {
                return Err(AppError::Config {
                    message: format!("Saving not supported for format: {:?}", self.format),
                    field: "format".to_string(),
                });
            }
        };

        tokio::fs::write(&self.file_path, content).await
            .map_err(|e| AppError::Config {
                message: format!("Failed to write config file: {}", e),
                field: "file_path".to_string(),
            })?;

        Ok(())
    }

    fn flat_map_to_json_value(&self, values: &HashMap<String, ConfigValue>) -> Result<serde_json::Value> {
        let mut root = serde_json::Map::new();

        for (key, value) in values {
            self.insert_nested_value(&mut root, key, self.config_value_to_json_value(value));
        }

        Ok(serde_json::Value::Object(root))
    }

    fn insert_nested_value(&self, root: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: serde_json::Value) {
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() == 1 {
            root.insert(key.to_string(), value);
        } else {
            let mut current = root;
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    current.insert(part.to_string(), value);
                } else {
                    let part_str = part.to_string();
                    if !current.contains_key(&part_str) {
                        current.insert(part_str.clone(), serde_json::Value::Object(serde_json::Map::new()));
                    }
                    if let Some(ref mut val) = current.get_mut(&part_str) {
                        if let serde_json::Value::Object(ref mut map) = val {
                            current = map;
                        } else {
                            // Key exists but is not an object, replace it
                            let new_map = serde_json::Map::new();
                            *val = serde_json::Value::Object(new_map);
                            if let serde_json::Value::Object(ref mut map) = val {
                                current = map;
                            }
                        }
                    }
                }
            }
        }
    }

    fn config_value_to_json_value(&self, config_value: &ConfigValue) -> serde_json::Value {
        match config_value {
            ConfigValue::String(s) => serde_json::Value::String(s.clone()),
            ConfigValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            ConfigValue::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
            ConfigValue::Boolean(b) => serde_json::Value::Bool(*b),
            ConfigValue::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
            ConfigValue::Duration(d) => serde_json::Value::String(format!("{}ms", d.as_millis())),
            ConfigValue::List(l) => {
                let json_values: Vec<serde_json::Value> = l.iter()
                    .map(|v| self.config_value_to_json_value(v))
                    .collect();
                serde_json::Value::Array(json_values)
            }
            ConfigValue::Object(o) => {
                let json_object: serde_json::Map<String, serde_json::Value> = o.iter()
                    .map(|(k, v)| (k.clone(), self.config_value_to_json_value(v)))
                    .collect();
                serde_json::Value::Object(json_object)
            }
            ConfigValue::Null => serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl ConfigProvider for FileConfigProvider {
    async fn get(&self, key: &str) -> Result<Option<ConfigValue>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: ConfigValue) -> Result<()> {
        let mut data = self.data.write().await;
        let mut metadata = self.metadata.write().await;

        let mut config_metadata = metadata
            .get(key)
            .cloned()
            .unwrap_or_else(|| ConfigMetadata::new(key.to_string(), self.file_path.clone()));

        config_metadata.touch();
        config_metadata.value_type = value.type_name();

        data.insert(key.to_string(), value.clone());
        metadata.insert(key.to_string(), config_metadata);

        // Save to file
        drop(data); // Release the lock before saving
        let data = self.data.read().await;
        self.save_to_file(&*data).await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write().await;
        let mut metadata = self.metadata.write().await;

        let data_removed = data.remove(key).is_some();
        let metadata_removed = metadata.remove(key).is_some();

        if data_removed || metadata_removed {
            // Save to file
            drop(data); // Release the lock before saving
            let data = self.data.read().await;
            self.save_to_file(&*data).await?;
        }

        Ok(data_removed || metadata_removed)
    }

    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        let data = self.data.read().await;

        let keys: Vec<String> = if let Some(pattern) = pattern {
            data.keys()
                .filter(|key| key.contains(pattern))
                .cloned()
                .collect()
        } else {
            data.keys().cloned().collect()
        };

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<ConfigMetadata>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(key).cloned())
    }

    fn provider_metadata(&self) -> &ProviderMetadata {
        &self.provider_metadata
    }

    async fn reload(&mut self) -> Result<()> {
        self.load_file().await
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        let exists = tokio::fs::metadata(&self.file_path).await.is_ok();
        let status = if exists {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        };

        let details = {
            let mut map = HashMap::new();
            map.insert("file_path".to_string(), self.file_path.clone());
            map.insert("format".to_string(), format!("{:?}", self.format));
            map.insert("exists".to_string(), exists.to_string());
            map
        };

        Ok(ProviderHealth {
            status,
            message: if exists {
                "File provider is healthy".to_string()
            } else {
                "Config file not found".to_string()
            }.to_string(),
            last_check: Utc::now(),
            response_time_ms: Some(10),
            details,
        })
    }
}

// Helper functions for format conversions
fn serde_yaml_to_json(yaml_value: &serde_yaml::Value) -> serde_json::Value {
    match yaml_value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let arr: Vec<serde_json::Value> = seq.iter()
                .map(serde_yaml_to_json)
                .collect();
            serde_json::Value::Array(arr)
        }
        serde_yaml::Value::Mapping(mapping) => {
            let map: serde_json::Map<String, serde_json::Value> = mapping.iter()
                .filter_map(|(k, v)| {
                    k.as_str().map(|key_str| (key_str.to_string(), serde_yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

fn toml_to_json(toml_value: &toml::Value) -> serde_json::Value {
    match toml_value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        toml::Value::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            let json_arr: Vec<serde_json::Value> = arr.iter()
                .map(toml_to_json)
                .collect();
            serde_json::Value::Array(json_arr)
        }
        toml::Value::Table(table) => {
            let json_obj: serde_json::Map<String, serde_json::Value> = table.iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect();
            serde_json::Value::Object(json_obj)
        }
    }
}