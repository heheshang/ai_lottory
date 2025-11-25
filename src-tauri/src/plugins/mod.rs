//! Plugin System for Modular Algorithm Architecture
//!
//! This module provides a flexible plugin system that allows:
//! - Dynamic loading/unloading of prediction algorithms
//! - Isolated execution environments
//! - Standardized interfaces for all plugins
//! - Plugin lifecycle management
//! - Resource monitoring and cleanup

pub mod plugin_manager;
pub mod plugin_registry;
pub mod plugin_traits;
pub mod plugin_loader;
pub mod plugin_metrics;
pub mod plugin_config;

pub use plugin_manager::PluginManager;
pub use plugin_registry::PluginRegistry;
pub use plugin_traits::*;
pub use plugin_loader::PluginLoader;
pub use plugin_metrics::PluginMetrics;
pub use plugin_config::PluginConfig;

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemConfig {
    /// Plugin discovery paths
    pub plugin_paths: Vec<String>,
    /// Enable hot reloading
    pub enable_hot_reload: bool,
    /// Resource limits for plugins
    pub resource_limits: ResourceLimits,
    /// Security settings
    pub security_settings: SecuritySettings,
    /// Performance monitoring
    pub enable_metrics: bool,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            plugin_paths: vec!["plugins".to_string()],
            enable_hot_reload: false,
            resource_limits: ResourceLimits::default(),
            security_settings: SecuritySettings::default(),
            enable_metrics: true,
        }
    }
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU time per execution in seconds
    pub max_cpu_time_secs: u64,
    /// Maximum number of concurrent executions
    pub max_concurrent_executions: usize,
    /// Maximum file size for plugin data in MB
    pub max_file_size_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_time_secs: 30,
            max_concurrent_executions: 4,
            max_file_size_mb: 100,
        }
    }
}

/// Security settings for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Enable sandbox mode
    pub enable_sandbox: bool,
    /// Allowed network access
    pub allow_network_access: bool,
    /// Allowed file system access
    pub allowed_paths: Vec<String>,
    /// Maximum allowed plugin complexity score
    pub max_complexity_score: u32,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            allow_network_access: false,
            allowed_paths: vec!["/tmp".to_string()],
            max_complexity_score: 100,
        }
    }
}

/// Main plugin system interface
pub struct PluginSystem {
    manager: Arc<PluginManager>,
    registry: Arc<PluginRegistry>,
    metrics: Arc<PluginMetrics>,
    config: PluginSystemConfig,
}

impl PluginSystem {
    /// Create a new plugin system with the given configuration
    pub fn new(config: PluginSystemConfig) -> Self {
        let registry = Arc::new(PluginRegistry::new());
        let metrics = Arc::new(PluginMetrics::new(config.enable_metrics));
        let manager = Arc::new(PluginManager::new(
            registry.clone(),
            metrics.clone(),
            config.clone(),
        ));

        Self {
            manager,
            registry,
            metrics,
            config,
        }
    }

    /// Initialize the plugin system
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing plugin system...");

        // Create plugin directories if they don't exist
        for path in &self.config.plugin_paths {
            std::fs::create_dir_all(path).map_err(|e| AppError::Internal {
                message: format!("Failed to create plugin directory '{}': {}", path, e),
            })?;
        }

        // Discover and load plugins
        self.manager.discover_plugins().await?;

        tracing::info!("Plugin system initialized successfully");
        Ok(())
    }

    /// Get a reference to the plugin manager
    pub fn manager(&self) -> &PluginManager {
        &self.manager
    }

    /// Get a reference to the plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get a reference to the metrics collector
    pub fn metrics(&self) -> &PluginMetrics {
        &self.metrics
    }

    /// Get system configuration
    pub fn config(&self) -> &PluginSystemConfig {
        &self.config
    }

    /// Shutdown the plugin system
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down plugin system...");
        self.manager.shutdown().await?;
        tracing::info!("Plugin system shut down successfully");
        Ok(())
    }

    /// Get plugin system statistics
    pub async fn get_statistics(&self) -> PluginSystemStats {
        PluginSystemStats {
            total_plugins: self.registry.count(),
            active_plugins: self.manager.active_plugins_count(),
            total_executions: self.metrics.total_executions(),
            average_execution_time: self.metrics.average_execution_time(),
            memory_usage_mb: self.metrics.current_memory_usage(),
            errors_count: self.metrics.total_errors(),
        }
    }
}

/// Plugin system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemStats {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub total_executions: u64,
    pub average_execution_time: f64,
    pub memory_usage_mb: f64,
    pub errors_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_system_config() {
        let config = PluginSystemConfig::default();
        assert_eq!(config.plugin_paths.len(), 1);
        assert_eq!(config.plugin_paths[0], "plugins");
        assert!(!config.enable_hot_reload);
    }

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 512);
        assert_eq!(limits.max_cpu_time_secs, 30);
        assert_eq!(limits.max_concurrent_executions, 4);
    }

    #[test]
    fn test_security_settings() {
        let settings = SecuritySettings::default();
        assert!(settings.enable_sandbox);
        assert!(!settings.allow_network_access);
        assert_eq!(settings.allowed_paths.len(), 1);
        assert_eq!(settings.allowed_paths[0], "/tmp");
    }
}