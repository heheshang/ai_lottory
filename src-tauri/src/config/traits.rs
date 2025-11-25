//! Configuration Traits
//!
//! Defines the core configuration interfaces and contracts.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use chrono::{DateTime, Utc};

/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Get a configuration value
    async fn get(&self, key: &str) -> Result<Option<ConfigValue>>;

    /// Set a configuration value
    async fn set(&self, key: &str, value: ConfigValue) -> Result<()>;

    /// Delete a configuration key
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Get all configuration keys
    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Get configuration metadata
    async fn get_metadata(&self, key: &str) -> Result<Option<ConfigMetadata>>;

    /// Get provider metadata
    fn provider_metadata(&self) -> &ProviderMetadata;

    /// Reload configuration
    async fn reload(&mut self) -> Result<()>;

    /// Health check for the provider
    async fn health_check(&self) -> Result<ProviderHealth>;
}

/// Configuration loader trait
#[async_trait]
pub trait ConfigLoader: Send + Sync {
    /// Load configuration from a source
    async fn load(&self, source: &ConfigSource) -> Result<ConfigData>;

    /// Save configuration to a source
    async fn save(&self, source: &ConfigSource, data: &ConfigData) -> Result<()>;

    /// Watch for configuration changes
    async fn watch(&self, source: &ConfigSource) -> Result<Box<dyn ConfigWatcher>>;

    /// Get loader metadata
    fn loader_metadata(&self) -> &LoaderMetadata;

    /// Validate configuration source
    fn validate_source(&self, source: &ConfigSource) -> Result<()>;
}

/// Configuration validator trait
pub trait ConfigValidator: Send + Sync {
    /// Validate a configuration value
    fn validate(&self, key: &str, value: &ConfigValue) -> Result<ValidationResult>;

    /// Get validation rules for a key
    fn get_rules(&self, key: &str) -> Option<&ValidationRules>;

    /// Validate the entire configuration
    fn validate_all(&self, config: &ConfigData) -> Result<ValidationSummary>;

    /// Get validator metadata
    fn validator_metadata(&self) -> &ValidatorMetadata;
}

/// Configuration watcher trait
#[async_trait]
pub trait ConfigWatcher: Send + Sync {
    /// Start watching for changes
    async fn start(&mut self) -> Result<()>;

    /// Stop watching
    async fn stop(&mut self) -> Result<()>;

    /// Subscribe to change events
    fn subscribe(&mut self, callback: Box<dyn ConfigChangeCallback>) -> Result<WatcherId>;

    /// Unsubscribe from change events
    fn unsubscribe(&mut self, watcher_id: WatcherId) -> Result<bool>;

    /// Get watcher status
    fn status(&self) -> WatcherStatus;

    /// Get watcher metadata
    fn watcher_metadata(&self) -> &WatcherMetadata;
}

/// Configuration manager trait
#[async_trait]
pub trait ConfigManager: Send + Sync {
    /// Get a configuration value
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Get a configuration value with default
    async fn get_with_default<T>(&self, key: &str, default: T) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Send + Sync + Clone;

    /// Set a configuration value
    async fn set<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Delete a configuration key
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Get all configuration as a map
    async fn get_all(&self) -> Result<HashMap<String, ConfigValue>>;

    /// Get configuration by pattern
    async fn get_by_pattern(&self, pattern: &str) -> Result<HashMap<String, ConfigValue>>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Reload all configuration
    async fn reload(&mut self) -> Result<()>;

    /// Save current configuration
    async fn save(&self) -> Result<()>;

    /// Get manager metadata
    fn manager_metadata(&self) -> &ManagerMetadata;

    /// Get configuration statistics
    async fn get_stats(&self) -> Result<ConfigStats>;

    /// Subscribe to configuration changes
    fn subscribe(&mut self, callback: Box<dyn ConfigChangeCallback>) -> Result<WatcherId>;

    /// Unsubscribe from configuration changes
    fn unsubscribe(&mut self, watcher_id: WatcherId) -> Result<bool>;

    /// Validate configuration
    async fn validate(&self) -> Result<ValidationSummary>;

    /// Get configuration history
    async fn get_history(&self, key: &str, limit: Option<u32>) -> Result<Vec<ConfigChange>>;

    /// Rollback configuration to a previous state
    async fn rollback(&mut self, timestamp: DateTime<Utc>) -> Result<()>;
}

/// Configuration change callback trait
pub trait ConfigChangeCallback: Send + Sync {
    /// Handle configuration change
    fn on_change(&mut self, event: ConfigChangeEvent) -> Result<()>;

    /// Get callback metadata
    fn callback_metadata(&self) -> &CallbackMetadata;
}

/// Configuration source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// File-based configuration
    File {
        path: String,
        format: ConfigFormat,
        encoding: Option<String>,
    },
    /// Environment variables
    Environment {
        prefix: Option<String>,
        separator: Option<String>,
    },
    /// Command line arguments
    CommandLine {
        args: Vec<String>,
    },
    /// Remote configuration service
    Remote {
        url: String,
        auth: Option<AuthConfig>,
        headers: Option<HashMap<String, String>>,
    },
    /// Database configuration
    Database {
        connection_string: String,
        table: String,
        key_column: String,
        value_column: String,
    },
    /// Vault/Secret manager
    Vault {
        vault_type: VaultType,
        address: String,
        auth: VaultAuth,
        path: String,
    },
    /// In-memory configuration
    Memory {
        data: HashMap<String, ConfigValue>,
    },
}

/// Configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Ini,
    Xml,
    Properties,
}

/// Vault types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultType {
    HashiCorpVault,
    AwsSecretsManager,
    AzureKeyVault,
    GoogleSecretManager,
}

/// Vault authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultAuth {
    Token(String),
    AppRole { role_id: String, secret_id: String },
    AwsIam { role: String },
    AzureManagedIdentity,
    GcpServiceAccount { key_path: String },
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub credentials: HashMap<String, String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Authentication types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthType {
    None,
    BearerToken,
    BasicAuth,
    ApiKey,
    OAuth2,
    Custom,
}

/// Configuration value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Duration(std::time::Duration),
    List(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            ConfigValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Integer(i) => Some(*i as f64),
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Some(true),
                    "false" | "0" | "no" | "off" => Some(false),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::String(_) => "string",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::Boolean(_) => "boolean",
            ConfigValue::DateTime(_) => "datetime",
            ConfigValue::Duration(_) => "duration",
            ConfigValue::List(_) => "list",
            ConfigValue::Object(_) => "object",
            ConfigValue::Null => "null",
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            ConfigValue::String(_) | ConfigValue::Integer(_) | ConfigValue::Float(_) | ConfigValue::Boolean(_)
        )
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, ConfigValue::List(_) | ConfigValue::Object(_))
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Float(value)
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
}

impl From<DateTime<Utc>> for ConfigValue {
    fn from(value: DateTime<Utc>) -> Self {
        ConfigValue::DateTime(value)
    }
}

impl From<Vec<ConfigValue>> for ConfigValue {
    fn from(value: Vec<ConfigValue>) -> Self {
        ConfigValue::List(value)
    }
}

impl From<HashMap<String, ConfigValue>> for ConfigValue {
    fn from(value: HashMap<String, ConfigValue>) -> Self {
        ConfigValue::Object(value)
    }
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub key: String,
    pub value_type: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    pub checksum: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub sensitive: bool,
    pub encrypted: bool,
}

impl ConfigMetadata {
    pub fn new(key: String, source: String) -> Self {
        let now = Utc::now();
        Self {
            key,
            value_type: "unknown".to_string(),
            source,
            created_at: now,
            updated_at: now,
            version: 1,
            checksum: None,
            tags: Vec::new(),
            description: None,
            sensitive: false,
            encrypted: false,
        }
    }

    pub fn with_value_type(mut self, value_type: impl Into<String>) -> Self {
        self.value_type = value_type.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    pub fn with_encrypted(mut self, encrypted: bool) -> Self {
        self.encrypted = encrypted;
        self
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
        self.version += 1;
    }
}

/// Configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub values: HashMap<String, ConfigValue>,
    pub metadata: HashMap<String, ConfigMetadata>,
    pub source_info: SourceInfo,
    pub checksum: Option<String>,
    pub loaded_at: DateTime<Utc>,
}

impl ConfigData {
    pub fn new(source_info: SourceInfo) -> Self {
        Self {
            values: HashMap::new(),
            metadata: HashMap::new(),
            source_info,
            checksum: None,
            loaded_at: Utc::now(),
        }
    }

    pub fn with_values(mut self, values: HashMap<String, ConfigValue>) -> Self {
        self.values = values;
        self
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&ConfigMetadata> {
        self.metadata.get(key)
    }

    pub fn insert(&mut self, key: String, value: ConfigValue, metadata: ConfigMetadata) {
        self.values.insert(key.clone(), value);
        self.metadata.insert(key, metadata);
    }

    pub fn remove(&mut self, key: &str) -> Option<(ConfigValue, ConfigMetadata)> {
        let value = self.values.remove(key)?;
        let metadata = self.metadata.remove(key)?;
        Some((value, metadata))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

/// Source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub source_type: String,
    pub source_location: String,
    pub format: Option<ConfigFormat>,
    pub encoding: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub etag: Option<String>,
    pub size_bytes: Option<u64>,
    pub checksum: Option<String>,
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub key: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: Option<ConfigValue>,
    pub change_type: ChangeType,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

/// Change type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
    Moved,
}

/// Configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub event: ConfigChangeEvent,
    pub metadata: ConfigMetadata,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub value: Option<ConfigValue>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub value: Option<ConfigValue>,
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub required: bool,
    pub value_type: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<ConfigValue>,
    pub max_value: Option<ConfigValue>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<ConfigValue>>,
    pub custom_validators: Vec<String>,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            required: false,
            value_type: None,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            pattern: None,
            allowed_values: None,
            custom_validators: Vec::new(),
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_type(mut self, value_type: impl Into<String>) -> Self {
        self.value_type = Some(value_type.into());
        self
    }

    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_min_value(mut self, min_value: ConfigValue) -> Self {
        self.min_value = Some(min_value);
        self
    }

    pub fn with_max_value(mut self, max_value: ConfigValue) -> Self {
        self.max_value = Some(max_value);
        self
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    pub fn with_allowed_values(mut self, allowed_values: Vec<ConfigValue>) -> Self {
        self.allowed_values = Some(allowed_values);
        self
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_keys: usize,
    pub valid_keys: usize,
    pub invalid_keys: usize,
    pub warnings: Vec<ValidationWarning>,
    pub errors: Vec<ValidationError>,
    pub is_valid: bool,
}

/// Watcher identifier
pub type WatcherId = u64;

/// Watcher status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatcherStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

/// Support metadata types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub supported_formats: Vec<ConfigFormat>,
    pub read_only: bool,
    pub supports_watching: bool,
    pub supports_encryption: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_sources: Vec<String>,
    pub supported_formats: Vec<ConfigFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub providers: Vec<String>,
    pub capabilities: Vec<String>,
}

/// Provider health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub message: String,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub details: HashMap<String, String>,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Configuration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStats {
    pub total_keys: u64,
    pub keys_by_source: HashMap<String, u64>,
    pub keys_by_type: HashMap<String, u64>,
    pub sensitive_keys: u64,
    pub encrypted_keys: u64,
    pub last_reload: Option<DateTime<Utc>>,
    pub reload_count: u64,
    pub change_count: u64,
    pub watcher_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ConfigStats {
    pub fn new() -> Self {
        Self {
            total_keys: 0,
            keys_by_source: HashMap::new(),
            keys_by_type: HashMap::new(),
            sensitive_keys: 0,
            encrypted_keys: 0,
            last_reload: None,
            reload_count: 0,
            change_count: 0,
            watcher_count: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl Default for ConfigStats {
    fn default() -> Self {
        Self::new()
    }
}