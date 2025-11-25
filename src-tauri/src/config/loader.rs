//! Configuration Loaders
//!
//! Implements loaders for different configuration sources.

use crate::config::traits::*;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File configuration loader
pub struct FileConfigLoader {
    supported_formats: Vec<ConfigFormat>,
}

impl FileConfigLoader {
    pub fn new() -> Self {
        Self {
            supported_formats: vec![
                ConfigFormat::Json,
                ConfigFormat::Yaml,
                ConfigFormat::Toml,
                ConfigFormat::Ini,
            ],
        }
    }
}

#[async_trait]
impl ConfigLoader for FileConfigLoader {
    async fn load(&self, source: &ConfigSource) -> Result<ConfigData> {
        match source {
            ConfigSource::File { path, format, encoding } => {
                let content = tokio::fs::read_to_string(path).await
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to read config file: {}", e),
                        field: "file_path".to_string(),
                    })?;

                let (values, metadata) = self.parse_content(&content, *format)?;

                let source_info = SourceInfo {
                    source_type: "file".to_string(),
                    source_location: path.clone(),
                    format: Some(*format),
                    encoding: encoding.clone(),
                    last_modified: None,
                    etag: None,
                    size_bytes: Some(content.len() as u64),
                    checksum: None,
                };

                Ok(ConfigData {
                    values,
                    metadata,
                    source_info,
                    checksum: None,
                    loaded_at: chrono::Utc::now(),
                })
            }
            _ => Err(AppError::Config {
                message: "File loader only supports file sources".to_string(),
                field: "source_type".to_string(),
            }),
        }
    }

    async fn save(&self, source: &ConfigSource, data: &ConfigData) -> Result<()> {
        match source {
            ConfigSource::File { path, format, .. } => {
                let content = match format {
                    ConfigFormat::Json => serde_json::to_string_pretty(&data.values)?,
                    ConfigFormat::Yaml => serde_yaml::to_string(&data.values)?,
                    _ => {
                        return Err(AppError::Config {
                            message: format!("Saving not supported for format: {:?}", format),
                            field: "format".to_string(),
                        });
                    }
                };

                tokio::fs::write(path, content).await
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to write config file: {}", e),
                        field: "file_path".to_string(),
                    })?;

                Ok(())
            }
            _ => Err(AppError::Config {
                message: "File loader only supports file sources".to_string(),
                field: "source_type".to_string(),
            }),
        }
    }

    async fn watch(&self, source: &ConfigSource) -> Result<Box<dyn ConfigWatcher>> {
        // This would implement file watching logic
        // For now, return a placeholder
        Err(AppError::Config {
            message: "File watching not implemented yet".to_string(),
            field: "watching".to_string(),
        })
    }

    fn loader_metadata(&self) -> &LoaderMetadata {
        static METADATA: LoaderMetadata = LoaderMetadata {
            name: "file".to_string(),
            version: "1.0.0".to_string(),
            description: "File configuration loader".to_string(),
            supported_sources: vec!["file".to_string()],
            supported_formats: vec![
                ConfigFormat::Json,
                ConfigFormat::Yaml,
                ConfigFormat::Toml,
                ConfigFormat::Ini,
            ],
        };
        &METADATA
    }

    fn validate_source(&self, source: &ConfigSource) -> Result<()> {
        match source {
            ConfigSource::File { path, format, .. } => {
                if self.supported_formats.contains(format) {
                    Ok(())
                } else {
                    Err(AppError::Config {
                        message: format!("Unsupported format: {:?}", format),
                        field: "format".to_string(),
                    })
                }
            }
            _ => Err(AppError::Config {
                message: "File loader only supports file sources".to_string(),
                field: "source_type".to_string(),
            }),
        }
    }
}

impl FileConfigLoader {
    fn parse_content(&self, content: &str, format: ConfigFormat) -> Result<(HashMap<String, ConfigValue>, HashMap<String, ConfigMetadata>)> {
        match format {
            ConfigFormat::Json => {
                let json_value: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse JSON: {}", e),
                        field: "content".to_string(),
                    })?;

                let values = self.json_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse YAML: {}", e),
                        field: "content".to_string(),
                    })?;

                let json_value = self.yaml_to_json(&yaml_value);
                let values = self.json_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(content)
                    .map_err(|e| AppError::Config {
                        message: format!("Failed to parse TOML: {}", e),
                        field: "content".to_string(),
                    })?;

                let json_value = self.toml_to_json(&toml_value);
                let values = self.json_to_flat_map(&json_value, "")?;
                Ok((values, HashMap::new()))
            }
            _ => Err(AppError::Config {
                message: format!("Unsupported format: {:?}", format),
                field: "format".to_string(),
            }),
        }
    }

    fn json_to_flat_map(&self, value: &serde_json::Value, prefix: &str) -> Result<HashMap<String, ConfigValue>> {
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
                        let nested_result = self.json_to_flat_map(val, &new_key)?;
                        result.extend(nested_result);
                    } else {
                        let config_value = self.json_to_config_value(val.clone());
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
                        let nested_result = self.json_to_flat_map(val, &new_key)?;
                        result.extend(nested_result);
                    } else {
                        let config_value = self.json_to_config_value(val.clone());
                        result.insert(new_key, config_value);
                    }
                }
            }
            _ => {
                if !prefix.is_empty() {
                    let config_value = self.json_to_config_value(value.clone());
                    result.insert(prefix.to_string(), config_value);
                }
            }
        }

        Ok(result)
    }

    fn json_to_config_value(&self, json_value: serde_json::Value) -> ConfigValue {
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
                    .map(|v| self.json_to_config_value(v))
                    .collect();
                ConfigValue::List(config_values)
            }
            serde_json::Value::Object(obj) => {
                let config_object: HashMap<String, ConfigValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, self.json_to_config_value(v)))
                    .collect();
                ConfigValue::Object(config_object)
            }
        }
    }

    fn yaml_to_json(&self, yaml_value: &serde_yaml::Value) -> serde_json::Value {
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
                    .map(|v| self.yaml_to_json(v))
                    .collect();
                serde_json::Value::Array(arr)
            }
            serde_yaml::Value::Mapping(mapping) => {
                let map: serde_json::Map<String, serde_json::Value> = mapping.iter()
                    .filter_map(|(k, v)| {
                        k.as_str().map(|key_str| (key_str.to_string(), self.yaml_to_json(v)))
                    })
                    .collect();
                serde_json::Value::Object(map)
            }
        }
    }

    fn toml_to_json(&self, toml_value: &toml::Value) -> serde_json::Value {
        match toml_value {
            toml::Value::String(s) => serde_json::Value::String(s.clone()),
            toml::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            toml::Value::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
            toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
            toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
            toml::Value::Array(arr) => {
                let json_arr: Vec<serde_json::Value> = arr.iter()
                    .map(|v| self.toml_to_json(v))
                    .collect();
                serde_json::Value::Array(json_arr)
            }
            toml::Value::Table(table) => {
                let json_obj: serde_json::Map<String, serde_json::Value> = table.iter()
                    .map(|(k, v)| (k.clone(), self.toml_to_json(v)))
                    .collect();
                serde_json::Value::Object(json_obj)
            }
        }
    }
}

impl Default for FileConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}