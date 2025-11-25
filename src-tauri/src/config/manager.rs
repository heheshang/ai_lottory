//! Configuration Manager
//!
//! Central configuration management with multiple providers and hot reloading.

use crate::config::traits::*;
use crate::config::error::ConfigError;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Configuration manager implementation
pub struct ConfigManagerImpl {
    providers: HashMap<String, Arc<dyn ConfigProvider>>,
    provider_priority: Vec<String>,
    cache: Arc<RwLock<HashMap<String, ConfigValue>>>,
    metadata: Arc<RwLock<HashMap<String, ConfigMetadata>>>,
    watchers: HashMap<WatcherId, Box<dyn ConfigChangeCallback>>,
    next_watcher_id: Arc<RwLock<WatcherId>>,
    manager_metadata: ManagerMetadata,
    stats: Arc<RwLock<ConfigStats>>,
    change_history: Arc<RwLock<Vec<ConfigChange>>>,
    validator: Option<Arc<dyn ConfigValidator>>,
}

impl ConfigManagerImpl {
    pub fn new(name: impl Into<String>) -> Self {
        let metadata = ManagerMetadata {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: "Central configuration manager".to_string(),
            providers: Vec::new(),
            capabilities: vec![
                "multi_source".to_string(),
                "caching".to_string(),
                "validation".to_string(),
                "watching".to_string(),
                "hot_reload".to_string(),
                "history".to_string(),
            ],
        };

        Self {
            providers: HashMap::new(),
            provider_priority: Vec::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            watchers: HashMap::new(),
            next_watcher_id: Arc::new(RwLock::new(1)),
            manager_metadata: metadata,
            stats: Arc::new(RwLock::new(ConfigStats::new())),
            change_history: Arc::new(RwLock::new(Vec::new())),
            validator: None,
        }
    }

    /// Add a configuration provider
    pub async fn add_provider<P>(&mut self, name: String, provider: P, priority: i32) -> Result<()>
    where
        P: ConfigProvider + 'static + Send + Sync,
    {
        // Check if provider already exists
        if self.providers.contains_key(&name) {
            return Err(AppError::Config {
                message: format!("Provider '{}' already exists", name),
                field: "provider_name".to_string(),
            });
        }

        let provider_arc = Arc::new(provider);
        self.providers.insert(name.clone(), provider_arc);

        // Update priority list
        self.provider_priority.push(name);
        self.provider_priority.sort_by(|_, _| std::cmp::Ordering::Equal); // Will be properly sorted based on priority

        // Update metadata
        self.manager_metadata.providers.push(name);

        // Clear cache to force reload with new provider
        self.cache.write().await.clear();

        tracing::info!("Added configuration provider: {}", name);
        Ok(())
    }

    /// Remove a configuration provider
    pub async fn remove_provider(&mut self, name: &str) -> Result<bool> {
        let removed = self.providers.remove(name).is_some();

        if removed {
            self.provider_priority.retain(|p| p != name);
            self.manager_metadata.providers.retain(|p| p != name);

            // Clear cache to force reload
            self.cache.write().await.clear();

            tracing::info!("Removed configuration provider: {}", name);
        }

        Ok(removed)
    }

    /// Set the validator
    pub fn set_validator(&mut self, validator: Arc<dyn ConfigValidator>) {
        self.validator = Some(validator);
    }

    /// Get configuration value from providers with priority
    async fn get_from_providers(&self, key: &str) -> Result<Option<ConfigValue>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        // Try providers in priority order
        for provider_name in &self.provider_priority {
            if let Some(provider) = self.providers.get(provider_name) {
                match provider.get(key).await {
                    Ok(Some(value)) => {
                        // Cache the value
                        {
                            let mut cache = self.cache.write().await;
                            cache.insert(key.to_string(), value.clone());
                        }

                        // Update stats
                        {
                            let mut stats = self.stats.write().await;
                            stats.cache_hits += 1;
                            *stats.keys_by_source.entry(provider_name.clone()).or_insert(0) += 1;
                        }

                        return Ok(Some(value));
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        tracing::warn!("Provider '{}' failed to get key '{}': {}", provider_name, key, e);
                        continue;
                    }
                }
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
        }

        Ok(None)
    }

    /// Notify watchers of configuration changes
    async fn notify_watchers(&mut self, event: ConfigChangeEvent) -> Result<()> {
        let mut failed_watchers = Vec::new();

        for (watcher_id, callback) in &mut self.watchers {
            match callback.on_change(event.clone()) {
                Ok(()) => {
                    tracing::debug!("Notified watcher {} of configuration change", watcher_id);
                }
                Err(e) => {
                    tracing::error!("Failed to notify watcher {}: {}", watcher_id, e);
                    failed_watchers.push(*watcher_id);
                }
            }
        }

        // Remove failed watchers
        for watcher_id in failed_watchers {
            self.watchers.remove(&watcher_id);
            tracing::warn!("Removed failed watcher: {}", watcher_id);
        }

        Ok(())
    }

    /// Record configuration change in history
    async fn record_change(&mut self, event: ConfigChangeEvent, metadata: ConfigMetadata) {
        let change = ConfigChange {
            event: event.clone(),
            metadata,
        };

        let mut history = self.change_history.write().await;
        history.push(change);

        // Keep only the last 1000 changes
        if history.len() > 1000 {
            history.remove(0);
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.change_count += 1;
    }

    /// Validate a configuration value
    async fn validate_value(&self, key: &str, value: &ConfigValue) -> Result<()> {
        if let Some(validator) = &self.validator {
            let result = validator.validate(key, value);
            if !result.is_valid {
                let errors = result.errors.iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(AppError::Config {
                    message: format!("Validation failed for '{}': {}", key, errors),
                    field: key.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Reload configuration from all providers
    async fn reload_from_providers(&mut self) -> Result<()> {
        let mut new_cache = HashMap::new();
        let mut new_metadata = HashMap::new();

        for provider_name in &self.provider_priority {
            if let Some(provider) = self.providers.get(provider_name) {
                // Get all keys from this provider
                match provider.list_keys(None).await {
                    Ok(keys) => {
                        for key in keys {
                            match provider.get(&key).await {
                                Ok(Some(value)) => {
                                    // Only add if not already present (respect priority)
                                    if !new_cache.contains_key(&key) {
                                        new_cache.insert(key.clone(), value.clone());

                                        // Get metadata
                                        if let Ok(Some(metadata)) = provider.get_metadata(&key).await {
                                            new_metadata.insert(key, metadata);
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    tracing::warn!("Failed to get key '{}' from provider '{}': {}", key, provider_name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to list keys from provider '{}': {}", provider_name, e);
                    }
                }
            }
        }

        // Update cache and metadata
        {
            let mut cache = self.cache.write().await;
            let mut metadata = self.metadata.write().await;

            *cache = new_cache;
            *metadata = new_metadata;
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.reload_count += 1;
            stats.last_reload = Some(Utc::now());
            stats.total_keys = self.cache.read().await.len() as u64;
        }

        tracing::info!("Configuration reloaded from {} providers", self.providers.len());
        Ok(())
    }
}

#[async_trait]
impl ConfigManager for ConfigManagerImpl {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        match self.get_from_providers(key).await? {
            Some(config_value) => {
                match serde_json::to_value(&config_value) {
                    Ok(json_value) => {
                        match serde_json::from_value::<T>(json_value) {
                            Ok(value) => Ok(Some(value)),
                            Err(e) => Err(AppError::Config {
                                message: format!("Failed to deserialize config value for '{}': {}", key, e),
                                field: key.to_string(),
                            }),
                        }
                    }
                    Err(e) => Err(AppError::Config {
                        message: format!("Failed to convert config value for '{}': {}", key, e),
                        field: key.to_string(),
                    }),
                }
            }
            None => Ok(None),
        }
    }

    async fn get_with_default<T>(&self, key: &str, default: T) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Send + Sync + Clone,
    {
        match self.get::<T>(key).await? {
            Some(value) => Ok(value),
            None => Ok(default),
        }
    }

    async fn set<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let json_value = serde_json::to_value(&value)?;
        let config_value = self.json_value_to_config_value(json_value);

        // Validate the value
        self.validate_value(key, &config_value).await?;

        // Get old value for change tracking
        let old_value = self.get_from_providers(key).await?;

        // Set in the first writable provider
        for provider_name in &self.provider_priority {
            if let Some(provider) = self.providers.get(provider_name) {
                if !provider.provider_metadata().read_only {
                    match provider.set(key, config_value.clone()).await {
                        Ok(()) => {
                            // Update cache
                            {
                                let mut cache = self.cache.write().await;
                                cache.insert(key.to_string(), config_value.clone());
                            }

                            // Create change event
                            let event = ConfigChangeEvent {
                                key: key.to_string(),
                                old_value,
                                new_value: Some(config_value),
                                change_type: if old_value.is_some() {
                                    ChangeType::Updated
                                } else {
                                    ChangeType::Created
                                },
                                source: provider_name.clone(),
                                timestamp: Utc::now(),
                                user_id: None,
                                session_id: None,
                                request_id: None,
                            };

                            // Record change and notify watchers
                            let metadata = ConfigMetadata::new(key.to_string(), provider_name.clone())
                                .with_value_type(config_value.type_name());
                            self.record_change(event.clone(), metadata).await;
                            self.notify_watchers(event).await?;

                            tracing::debug!("Set configuration key '{}' in provider '{}'", key, provider_name);
                            return Ok(());
                        }
                        Err(e) => {
                            tracing::error!("Failed to set key '{}' in provider '{}': {}", key, provider_name, e);
                            continue;
                        }
                    }
                }
            }
        }

        Err(AppError::Config {
            message: "No writable configuration provider available".to_string(),
            field: key.to_string(),
        })
    }

    async fn delete(&mut self, key: &str) -> Result<bool> {
        let old_value = self.get_from_providers(key).await?;

        if old_value.is_none() {
            return Ok(false);
        }

        // Delete from the first writable provider
        for provider_name in &self.provider_priority {
            if let Some(provider) = self.providers.get(provider_name) {
                if !provider.provider_metadata().read_only {
                    match provider.delete(key).await {
                        Ok(deleted) => {
                            if deleted {
                                // Update cache
                                {
                                    let mut cache = self.cache.write().await;
                                    cache.remove(key);
                                }

                                // Create change event
                                let event = ConfigChangeEvent {
                                    key: key.to_string(),
                                    old_value,
                                    new_value: None,
                                    change_type: ChangeType::Deleted,
                                    source: provider_name.clone(),
                                    timestamp: Utc::now(),
                                    user_id: None,
                                    session_id: None,
                                    request_id: None,
                                };

                                // Record change and notify watchers
                                let metadata = ConfigMetadata::new(key.to_string(), provider_name.clone());
                                self.record_change(event.clone(), metadata).await;
                                self.notify_watchers(event).await?;

                                tracing::debug!("Deleted configuration key '{}' from provider '{}'", key, provider_name);
                                return Ok(true);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to delete key '{}' from provider '{}': {}", key, provider_name, e);
                            continue;
                        }
                    }
                }
            }
        }

        Err(AppError::Config {
            message: "No writable configuration provider available".to_string(),
            field: key.to_string(),
        })
    }

    async fn get_all(&self) -> Result<HashMap<String, ConfigValue>> {
        let cache = self.cache.read().await;
        Ok(cache.clone())
    }

    async fn get_by_pattern(&self, pattern: &str) -> Result<HashMap<String, ConfigValue>> {
        let cache = self.cache.read().await;

        let results: HashMap<String, ConfigValue> = cache
            .iter()
            .filter(|(key, _)| key.contains(pattern))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(results)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.get_from_providers(key).await.map(|v| v.is_some())
    }

    async fn reload(&mut self) -> Result<()> {
        self.reload_from_providers().await
    }

    async fn save(&self) -> Result<()> {
        // Save configuration from writable providers
        for (provider_name, provider) in &self.providers {
            if !provider.provider_metadata().read_only {
                // For memory providers, we might want to save to file
                // This would be provider-specific implementation
                tracing::debug!("Saving configuration from provider: {}", provider_name);
            }
        }

        Ok(())
    }

    fn manager_metadata(&self) -> &ManagerMetadata {
        &self.manager_metadata
    }

    async fn get_stats(&self) -> Result<ConfigStats> {
        let cache = self.cache.read().await;
        let stats = self.stats.read().await.clone();
        let mut result = stats;

        // Update current key count
        result.total_keys = cache.len() as u64;

        // Count keys by type
        result.keys_by_type.clear();
        for value in cache.values() {
            *result.keys_by_type.entry(value.type_name().to_string()).or_insert(0) += 1;
        }

        Ok(result)
    }

    fn subscribe(&mut self, callback: Box<dyn ConfigChangeCallback>) -> Result<WatcherId> {
        let mut next_id = self.next_watcher_id.write().unwrap();
        let watcher_id = *next_id;
        *next_id += 1;

        self.watchers.insert(watcher_id, callback);
        tracing::info!("Added configuration watcher: {}", watcher_id);

        Ok(watcher_id)
    }

    fn unsubscribe(&mut self, watcher_id: WatcherId) -> Result<bool> {
        let removed = self.watchers.remove(&watcher_id).is_some();
        if removed {
            tracing::info!("Removed configuration watcher: {}", watcher_id);
        }
        Ok(removed)
    }

    async fn validate(&self) -> Result<ValidationSummary> {
        if let Some(validator) = &self.validator {
            let cache = self.cache.read().await;
            validator.validate_all(&ConfigData {
                values: cache.clone(),
                metadata: self.metadata.read().await.clone(),
                source_info: crate::config::traits::SourceInfo {
                    source_type: "manager".to_string(),
                    source_location: "memory".to_string(),
                    format: None,
                    encoding: None,
                    last_modified: None,
                    etag: None,
                    size_bytes: None,
                    checksum: None,
                },
                checksum: None,
                loaded_at: Utc::now(),
            })
        } else {
            Ok(ValidationSummary {
                total_keys: self.cache.read().await.len(),
                valid_keys: self.cache.read().await.len(),
                invalid_keys: 0,
                warnings: Vec::new(),
                errors: Vec::new(),
                is_valid: true,
            })
        }
    }

    async fn get_history(&self, key: &str, limit: Option<u32>) -> Result<Vec<ConfigChange>> {
        let history = self.change_history.read().await;
        let filtered: Vec<ConfigChange> = history
            .iter()
            .filter(|change| change.event.key == key)
            .rev()
            .take(limit.unwrap_or(100) as usize)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn rollback(&mut self, timestamp: DateTime<Utc>) -> Result<()> {
        // Find the configuration state before the specified timestamp
        let mut target_state = HashMap::new();

        let history = self.change_history.read().await;
        for change in history.iter().rev() {
            if change.event.timestamp < timestamp {
                if let Some(ref value) = change.event.new_value {
                    target_state.insert(change.event.key.clone(), value.clone());
                } else {
                    // Key was deleted, don't include in rollback state
                    target_state.remove(&change.event.key);
                }
            }
        }

        // Apply the target state
        for (key, value) in target_state {
            let json_value = serde_json::to_value(&value)?;
            let config_value = self.json_value_to_config_value(json_value);
            self.set(key, config_value).await?;
        }

        tracing::info!("Configuration rolled back to timestamp: {}", timestamp);
        Ok(())
    }

    // Helper method for JSON value conversion
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