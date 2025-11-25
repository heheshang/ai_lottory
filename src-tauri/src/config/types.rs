//! Configuration Types
//!
//! Common types used throughout the configuration system.

use crate::config::traits::*;
use serde::{Deserialize, Serialize};

/// Callback metadata for configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Simple callback implementation for configuration changes
pub struct SimpleConfigCallback {
    pub name: String,
    pub callback: Box<dyn Fn(ConfigChangeEvent) -> Result<()> + Send + Sync>,
    pub metadata: CallbackMetadata,
}

impl SimpleConfigCallback {
    pub fn new<F>(name: String, callback: F) -> Self
    where
        F: Fn(ConfigChangeEvent) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            metadata: CallbackMetadata {
                name: name.clone(),
                version: "1.0.0".to_string(),
                description: format!("Simple callback for {}", name),
                created_at: chrono::Utc::now(),
            },
            name,
            callback: Box::new(callback),
        }
    }
}

impl ConfigChangeCallback for SimpleConfigCallback {
    fn on_change(&mut self, event: ConfigChangeEvent) -> Result<()> {
        (self.callback)(event)
    }

    fn callback_metadata(&self) -> &CallbackMetadata {
        &self.metadata
    }
}

/// Configuration source builder
pub struct ConfigSourceBuilder {
    source: Option<ConfigSource>,
}

impl ConfigSourceBuilder {
    pub fn new() -> Self {
        Self { source: None }
    }

    pub fn file(mut self, path: impl Into<String>, format: ConfigFormat) -> Self {
        self.source = Some(ConfigSource::File {
            path: path.into(),
            format,
            encoding: None,
        });
        self
    }

    pub fn file_with_encoding(
        mut self,
        path: impl Into<String>,
        format: ConfigFormat,
        encoding: impl Into<String>,
    ) -> Self {
        self.source = Some(ConfigSource::File {
            path: path.into(),
            format,
            encoding: Some(encoding.into()),
        });
        self
    }

    pub fn environment(mut self) -> Self {
        self.source = Some(ConfigSource::Environment {
            prefix: None,
            separator: None,
        });
        self
    }

    pub fn environment_with_prefix(
        mut self,
        prefix: impl Into<String>,
        separator: impl Into<String>,
    ) -> Self {
        self.source = Some(ConfigSource::Environment {
            prefix: Some(prefix.into()),
            separator: Some(separator.into()),
        });
        self
    }

    pub fn command_line(mut self, args: Vec<String>) -> Self {
        self.source = Some(ConfigSource::CommandLine { args });
        self
    }

    pub fn remote(
        mut self,
        url: impl Into<String>,
        auth: Option<AuthConfig>,
        headers: Option<HashMap<String, String>>,
    ) -> Self {
        self.source = Some(ConfigSource::Remote {
            url: url.into(),
            auth,
            headers,
        });
        self
    }

    pub fn database(
        mut self,
        connection_string: impl Into<String>,
        table: impl Into<String>,
        key_column: impl Into<String>,
        value_column: impl Into<String>,
    ) -> Self {
        self.source = Some(ConfigSource::Database {
            connection_string: connection_string.into(),
            table: table.into(),
            key_column: key_column.into(),
            value_column: value_column.into(),
        });
        self
    }

    pub fn memory(mut self, data: HashMap<String, ConfigValue>) -> Self {
        self.source = Some(ConfigSource::Memory { data });
        self
    }

    pub fn build(self) -> Result<ConfigSource> {
        self.source.ok_or_else(|| AppError::Config {
            message: "No configuration source specified".to_string(),
            field: "source".to_string(),
        })
    }
}

impl Default for ConfigSourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration builder for creating common configuration setups
pub struct ConfigurationBuilder {
    name: String,
    sources: Vec<ConfigSource>,
    validator: Option<String>,
}

impl ConfigurationBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sources: Vec::new(),
            validator: None,
        }
    }

    pub fn add_file(mut self, path: impl Into<String>, format: ConfigFormat) -> Self {
        self.sources.push(ConfigSource::File {
            path: path.into(),
            format,
            encoding: None,
        });
        self
    }

    pub fn add_environment(mut self, prefix: Option<String>) -> Self {
        self.sources.push(ConfigSource::Environment {
            prefix,
            separator: None,
        });
        self
    }

    pub fn add_command_line(mut self, args: Vec<String>) -> Self {
        self.sources.push(ConfigSource::CommandLine { args });
        self
    }

    pub fn with_validator(mut self, validator: impl Into<String>) -> Self {
        self.validator = Some(validator.into());
        self
    }

    pub fn build(self) -> Result<ConfigurationSpec> {
        Ok(ConfigurationSpec {
            name: self.name,
            sources: self.sources,
            validator: self.validator,
        })
    }
}

/// Configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSpec {
    pub name: String,
    pub sources: Vec<ConfigSource>,
    pub validator: Option<String>,
}

/// Utility functions for common configuration patterns
pub mod utils {
    use super::*;

    /// Create a standard application configuration
    pub fn create_app_config(name: String) -> ConfigurationSpec {
        ConfigurationBuilder::new(name)
            .add_file("config/app.json", ConfigFormat::Json)
            .add_file("config/app.local.json", ConfigFormat::Json)
            .add_environment(Some("APP_".to_string()))
            .with_validator("default".to_string())
            .build()
            .unwrap()
    }

    /// Create a server configuration
    pub fn create_server_config(name: String) -> ConfigurationSpec {
        ConfigurationBuilder::new(name)
            .add_file("config/server.json", ConfigFormat::Json)
            .add_file("config/server.toml", ConfigFormat::Toml)
            .add_environment(Some("SERVER_".to_string()))
            .with_validator("default".to_string())
            .build()
            .unwrap()
    }

    /// Create a database configuration
    pub fn create_database_config(name: String) -> ConfigurationSpec {
        ConfigurationBuilder::new(name)
            .add_file("config/database.json", ConfigFormat::Json)
            .add_file("config/database.yaml", ConfigFormat::Yaml)
            .add_environment(Some("DB_".to_string()))
            .with_validator("default".to_string())
            .build()
            .unwrap()
    }

    /// Parse a key from environment variable format
    pub fn parse_env_key(env_key: &str, prefix: Option<&str>, separator: &str) -> Option<String> {
        let env_key = env_key.to_lowercase();

        let start_pos = if let Some(prefix) = prefix {
            let prefix = prefix.to_lowercase();
            env_key.strip_prefix(&prefix)?.len() + separator.len()
        } else {
            0
        };

        let key = &env_key[start_pos..];
        Some(key.replace(separator, "."))
    }

    /// Format a key for environment variable
    pub fn format_env_key(key: &str, prefix: Option<&str>, separator: &str) -> String {
        let formatted = key.replace(".", separator).to_uppercase();

        if let Some(prefix) = prefix {
            format!("{}{}{}", prefix, separator, formatted)
        } else {
            formatted
        }
    }

    /// Convert a nested configuration map to a flat map
    pub fn flatten_config_map(
        map: &HashMap<String, ConfigValue>,
        separator: &str,
    ) -> HashMap<String, ConfigValue> {
        let mut result = HashMap::new();

        for (key, value) in map {
            if let ConfigValue::Object(nested) = value {
                let flattened = flatten_nested_object(nested, key, separator);
                result.extend(flattened);
            } else {
                result.insert(key.clone(), value.clone());
            }
        }

        result
    }

    /// Convert a flat configuration map to nested structure
    pub fn nest_config_map(
        map: HashMap<String, ConfigValue>,
        separator: &str,
    ) -> HashMap<String, ConfigValue> {
        let mut result = HashMap::new();

        for (key, value) in map {
            if key.contains(separator) {
                let parts: Vec<&str> = key.split(separator).collect();
                insert_nested_value(&mut result, &parts, value.clone());
            } else {
                result.insert(key, value);
            }
        }

        result
    }

    fn flatten_nested_object(
        object: &HashMap<String, ConfigValue>,
        prefix: &str,
        separator: &str,
    ) -> HashMap<String, ConfigValue> {
        let mut result = HashMap::new();

        for (key, value) in object {
            let new_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}{}{}", prefix, separator, key)
            };

            match value {
                ConfigValue::Object(nested) => {
                    let flattened = flatten_nested_object(nested, &new_key, separator);
                    result.extend(flattened);
                }
                _ => {
                    result.insert(new_key, value.clone());
                }
            }
        }

        result
    }

    fn insert_nested_value(
        result: &mut HashMap<String, ConfigValue>,
        parts: &[&str],
        value: ConfigValue,
    ) {
        if parts.len() == 1 {
            result.insert(parts[0].to_string(), value);
            return;
        }

        let key = parts[0];
        let remaining_parts = &parts[1..];

        let nested = result.entry(key.to_string()).or_insert_with(|| {
            ConfigValue::Object(HashMap::new())
        });

        if let ConfigValue::Object(ref mut nested_map) = nested {
            insert_nested_value(nested_map, remaining_parts, value);
        }
    }

    /// Merge two configuration maps with priority to the second map
    pub fn merge_config_maps(
        base: HashMap<String, ConfigValue>,
        overlay: HashMap<String, ConfigValue>,
    ) -> HashMap<String, ConfigValue> {
        let mut result = base;
        for (key, value) in overlay {
            result.insert(key, value);
        }
        result
    }

    /// Validate a configuration value type
    pub fn validate_config_type(value: &ConfigValue, expected_type: &str) -> bool {
        match (value, expected_type) {
            (_, "any") => true,
            (ConfigValue::String(_), "string") => true,
            (ConfigValue::Integer(_), "integer") => true,
            (ConfigValue::Float(_), "float") => true,
            (ConfigValue::Boolean(_), "boolean") => true,
            (ConfigValue::DateTime(_), "datetime") => true,
            (ConfigValue::Duration(_), "duration") => true,
            (ConfigValue::List(_), "list") => true,
            (ConfigValue::Object(_), "object") => true,
            (ConfigValue::Null, "null") => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;

    #[test]
    fn test_config_source_builder() {
        let source = ConfigSourceBuilder::new()
            .file("config.json", ConfigFormat::Json)
            .build()
            .unwrap();

        match source {
            ConfigSource::File { path, format, .. } => {
                assert_eq!(path, "config.json");
                assert_eq!(format, ConfigFormat::Json);
            }
            _ => panic!("Expected File source"),
        }
    }

    #[test]
    fn test_configuration_builder() {
        let spec = ConfigurationBuilder::new("test")
            .add_file("config.json", ConfigFormat::Json)
            .add_environment(Some("TEST_".to_string()))
            .build()
            .unwrap();

        assert_eq!(spec.name, "test");
        assert_eq!(spec.sources.len(), 2);
    }

    #[test]
    fn test_env_key_parsing() {
        assert_eq!(
            parse_env_key("APP_SERVER_PORT", Some("APP_"), "_"),
            Some("server.port".to_string())
        );
        assert_eq!(
            parse_env_key("SERVER_PORT", None, "_"),
            Some("server.port".to_string())
        );
    }

    #[test]
    fn test_env_key_formatting() {
        assert_eq!(
            format_env_key("server.port", Some("APP_"), "_"),
            "APP_SERVER_PORT".to_string()
        );
        assert_eq!(
            format_env_key("server.port", None, "_"),
            "SERVER_PORT".to_string()
        );
    }

    #[test]
    fn test_config_flattening() {
        let mut nested = HashMap::new();
        nested.insert("port".to_string(), ConfigValue::Integer(8080));
        nested.insert("host".to_string(), ConfigValue::String("localhost".to_string()));

        let mut config = HashMap::new();
        config.insert("server".to_string(), ConfigValue::Object(nested));

        let flattened = flatten_config_map(&config, ".");
        assert_eq!(flattened.get("server.port"), Some(&ConfigValue::Integer(8080)));
        assert_eq!(flattened.get("server.host"), Some(&ConfigValue::String("localhost".to_string())));
    }

    #[test]
    fn test_config_nesting() {
        let mut flat = HashMap::new();
        flat.insert("server.port".to_string(), ConfigValue::Integer(8080));
        flat.insert("server.host".to_string(), ConfigValue::String("localhost".to_string()));

        let nested = nest_config_map(flat, ".");

        if let Some(ConfigValue::Object(server_config)) = nested.get("server") {
            assert_eq!(server_config.get("port"), Some(&ConfigValue::Integer(8080)));
            assert_eq!(server_config.get("host"), Some(&ConfigValue::String("localhost".to_string())));
        } else {
            panic!("Expected nested object");
        }
    }
}