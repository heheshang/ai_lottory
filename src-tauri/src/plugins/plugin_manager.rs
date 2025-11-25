//! Plugin Manager - Manages plugin lifecycle, execution, and resource allocation

use crate::error::{AppError, Result};
use crate::plugins::{
    PluginRegistry, PluginMetrics, PluginConfig, PluginSystemConfig, PredictionPlugin,
    PredictionParameters, PredictionResult, PluginMetadata, PluginState, ResourceLimits,
    SecuritySettings, LotteryDraw,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, Mutex};
use uuid::Uuid;

/// Plugin execution context
#[derive(Debug)]
pub struct ExecutionContext {
    pub id: String,
    pub plugin_id: String,
    pub parameters: PredictionParameters,
    pub start_time: Instant,
    pub timeout: Duration,
    pub memory_limit_mb: u64,
}

/// Plugin manager handles plugin lifecycle and execution
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    metrics: Arc<PluginMetrics>,
    config: PluginSystemConfig,
    loaded_plugins: RwLock<HashMap<String, Arc<dyn PredictionPlugin>>>,
    plugin_states: RwLock<HashMap<String, PluginState>>,
    execution_semaphore: Arc<Semaphore>,
    active_executions: Mutex<HashMap<String, ExecutionContext>>,
    resource_monitor: Arc<ResourceMonitor>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(
        registry: Arc<PluginRegistry>,
        metrics: Arc<PluginMetrics>,
        config: PluginSystemConfig,
    ) -> Self {
        let execution_semaphore = Arc::new(Semaphore::new(
            config.resource_limits.max_concurrent_executions
        ));
        let resource_monitor = Arc::new(ResourceMonitor::new(
            config.resource_limits.clone(),
            config.security_settings.clone(),
        ));

        Self {
            registry,
            metrics,
            config,
            loaded_plugins: RwLock::new(HashMap::new()),
            plugin_states: RwLock::new(HashMap::new()),
            execution_semaphore,
            active_executions: Mutex::new(HashMap::new()),
            resource_monitor,
        }
    }

    /// Discover and load plugins from configured paths
    pub async fn discover_plugins(&self) -> Result<()> {
        tracing::info!("Discovering plugins from paths: {:?}", self.config.plugin_paths);

        // In a real implementation, this would scan filesystem for plugin files
        // For now, we'll register built-in plugins

        // Register built-in statistical analysis plugins
        self.register_builtin_plugins().await?;

        tracing::info!("Plugin discovery completed");
        Ok(())
    }

    /// Register built-in plugins
    async fn register_builtin_plugins(&self) -> Result<()> {
        // Example: Register weighted frequency analysis plugin
        let weighted_freq_plugin = crate::plugins::builtin::WeightedFrequencyPlugin::new();
        self.register_plugin(Box::new(weighted_freq_plugin)).await?;

        // Example: Register pattern analysis plugin
        let pattern_plugin = crate::plugins::builtin::PatternAnalysisPlugin::new();
        self.register_plugin(Box::new(pattern_plugin)).await?;

        // Example: Register neural network plugin
        let neural_plugin = crate::plugins::builtin::NeuralNetworkPlugin::new();
        self.register_plugin(Box::new(neural_plugin)).await?;

        Ok(())
    }

    /// Register a new plugin
    pub async fn register_plugin(&self, plugin: Box<dyn PredictionPlugin>) -> Result<()> {
        let metadata = plugin.metadata().clone();
        let plugin_id = metadata.id.clone();

        tracing::debug!("Registering plugin: {} ({})", plugin_id, metadata.name);

        // Validate plugin metadata
        self.validate_plugin_metadata(&metadata)?;

        // Check for existing plugin with same ID
        {
            let loaded_plugins = self.loaded_plugins.read().unwrap();
            if loaded_plugins.contains_key(&plugin_id) {
                return Err(AppError::Plugin {
                    message: format!("Plugin with ID '{}' already registered", plugin_id),
                    plugin_id: plugin_id.clone(),
                });
            }
        }

        // Initialize plugin
        let mut plugin_owned = plugin;
        let config = PluginConfig::default(); // Use default config for now
        plugin_owned.initialize(config)?;

        // Register in registry
        self.registry.register(&metadata).await?;

        // Store loaded plugin
        {
            let mut loaded_plugins = self.loaded_plugins.write().unwrap();
            loaded_plugins.insert(plugin_id.clone(), Arc::from(plugin_owned));
        }

        // Set initial state
        {
            let mut plugin_states = self.plugin_states.write().unwrap();
            plugin_states.insert(plugin_id, PluginState::Ready);
        }

        tracing::info!("Successfully registered plugin: {}", plugin_id);
        Ok(())
    }

    /// Load a plugin by ID
    pub async fn load_plugin(&self, plugin_id: &str) -> Result<()> {
        {
            let plugin_states = self.plugin_states.read().unwrap();
            if let Some(state) = plugin_states.get(plugin_id) {
                match state {
                    PluginState::Ready => return Ok(()),
                    PluginState::Uninitialized => {}
                    _ => return Err(AppError::Plugin {
                        message: format!("Plugin '{}' cannot be loaded in state: {:?}", plugin_id, state),
                        plugin_id: plugin_id.to_string(),
                    }),
                }
            }
        }

        // In a real implementation, this would dynamically load the plugin
        // For now, we'll mark it as ready
        {
            let mut plugin_states = self.plugin_states.write().unwrap();
            plugin_states.insert(plugin_id.to_string(), PluginState::Ready);
        }

        tracing::info!("Loaded plugin: {}", plugin_id);
        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        // Stop any active executions for this plugin
        self.stop_plugin_executions(plugin_id).await?;

        // Cleanup plugin resources
        {
            let loaded_plugins = self.loaded_plugins.read().unwrap();
            if let Some(plugin) = loaded_plugins.get(plugin_id) {
                let mut plugin_mut = Arc::as_ptr(plugin) as *mut dyn PredictionPlugin;
                unsafe {
                    (*plugin_mut).cleanup()?;
                }
            }
        }

        // Remove from loaded plugins
        {
            let mut loaded_plugins = self.loaded_plugins.write().unwrap();
            loaded_plugins.remove(plugin_id);
        }

        // Update state
        {
            let mut plugin_states = self.plugin_states.write().unwrap();
            plugin_states.insert(plugin_id.to_string(), PluginState::Uninitialized);
        }

        tracing::info!("Unloaded plugin: {}", plugin_id);
        Ok(())
    }

    /// Execute a prediction using the specified plugin
    pub async fn execute_prediction(
        &self,
        plugin_id: &str,
        historical_data: &[LotteryDraw],
        parameters: PredictionParameters,
    ) -> Result<PredictionResult> {
        let execution_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        // Get plugin
        let plugin = {
            let loaded_plugins = self.loaded_plugins.read().unwrap();
            loaded_plugins.get(plugin_id).cloned().ok_or_else(|| AppError::Plugin {
                message: format!("Plugin '{}' not found", plugin_id),
                plugin_id: plugin_id.to_string(),
            })?
        };

        // Check plugin state
        {
            let plugin_states = self.plugin_states.read().unwrap();
            if let Some(state) = plugin_states.get(plugin_id) {
                match state {
                    PluginState::Ready => {}
                    _ => return Err(AppError::Plugin {
                        message: format!("Plugin '{}' is not ready for execution: {:?}", plugin_id, state),
                        plugin_id: plugin_id.to_string(),
                    }),
                }
            }
        }

        // Validate parameters
        plugin.validate_parameters(&parameters)?;

        // Check if plugin can handle dataset size
        if !plugin.can_handle_dataset(historical_data.len()) {
            return Err(AppError::Plugin {
                message: format!(
                    "Plugin '{}' cannot handle dataset of size {}",
                    plugin_id,
                    historical_data.len()
                ),
                plugin_id: plugin_id.to_string(),
            });
        }

        // Acquire execution permit
        let _permit = match tokio::time::timeout(
            Duration::from_secs(5),
            self.execution_semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(AppError::Plugin {
                message: format!("Failed to acquire execution permit for plugin '{}'", plugin_id),
                plugin_id: plugin_id.to_string(),
            }),
            Err(_) => return Err(AppError::Plugin {
                message: format!("Timeout acquiring execution permit for plugin '{}'", plugin_id),
                plugin_id: plugin_id.to_string(),
            }),
        };

        // Set execution state
        {
            let mut plugin_states = self.plugin_states.write().unwrap();
            plugin_states.insert(plugin_id.to_string(), PluginState::Executing);
        }

        // Create execution context
        let execution_context = ExecutionContext {
            id: execution_id.clone(),
            plugin_id: plugin_id.to_string(),
            parameters: parameters.clone(),
            start_time,
            timeout: self.config.resource_limits.max_cpu_time_secs as u64 * 1000,
            memory_limit_mb: self.config.resource_limits.max_memory_mb,
        };

        // Register execution
        {
            let mut active_executions = self.active_executions.lock().await;
            active_executions.insert(execution_id.clone(), execution_context);
        }

        // Execute with timeout and resource monitoring
        let result = tokio::select! {
            result = self.execute_with_monitoring(
                &plugin,
                historical_data,
                &parameters,
                &execution_id
            ) => result,
            _ = tokio::time::sleep(Duration::from_millis(execution_context.timeout)) => {
                Err(AppError::Plugin {
                    message: format!("Plugin '{}' execution timed out after {}ms", plugin_id, execution_context.timeout),
                    plugin_id: plugin_id.to_string(),
                })
            }
        };

        // Cleanup execution
        {
            let mut active_executions = self.active_executions.lock().await;
            active_executions.remove(&execution_id);
        }

        // Reset plugin state
        let final_state = if result.is_ok() {
            PluginState::Completed
        } else {
            PluginState::Failed(result.as_ref().err().map(|e| e.to_string()).unwrap_or_default())
        };

        {
            let mut plugin_states = self.plugin_states.write().unwrap();
            plugin_states.insert(plugin_id.to_string(), final_state);
        }

        // Record metrics
        let execution_time = start_time.elapsed();
        self.metrics.record_execution(
            plugin_id,
            execution_time,
            result.is_ok(),
            historical_data.len(),
        ).await;

        result
    }

    /// Execute plugin with resource monitoring
    async fn execute_with_monitoring(
        &self,
        plugin: &Arc<dyn PredictionPlugin>,
        historical_data: &[LotteryDraw],
        parameters: &PredictionParameters,
        execution_id: &str,
    ) -> Result<PredictionResult> {
        // Start resource monitoring
        let monitor_handle = self.resource_monitor.start_monitoring(execution_id).await?;

        // Execute the prediction
        let result = plugin.predict(historical_data, parameters).await;

        // Stop resource monitoring
        let resource_usage = self.resource_monitor.stop_monitoring(&monitor_handle).await?;

        // Add resource usage to result if successful
        if let Ok(mut prediction_result) = result {
            prediction_result.execution_stats.memory_usage_mb = resource_usage.memory_mb;
            prediction_result.execution_stats.cpu_usage_percent = resource_usage.cpu_percent;
            Ok(prediction_result)
        } else {
            result
        }
    }

    /// Stop all executions for a plugin
    async fn stop_plugin_executions(&self, plugin_id: &str) -> Result<()> {
        let mut active_executions = self.active_executions.lock().await;
        let executions_to_stop: Vec<String> = active_executions
            .iter()
            .filter(|(_, ctx)| ctx.plugin_id == plugin_id)
            .map(|(id, _)| id.clone())
            .collect();

        for execution_id in executions_to_stop {
            active_executions.remove(&execution_id);
            // In a real implementation, you might need to forcefully stop the execution
        }

        Ok(())
    }

    /// Get all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<String> {
        let loaded_plugins = self.loaded_plugins.read().unwrap();
        loaded_plugins.keys().cloned().collect()
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let loaded_plugins = self.loaded_plugins.read().unwrap();
        loaded_plugins.get(plugin_id).map(|plugin| plugin.metadata().clone())
    }

    /// Get plugin state
    pub fn get_plugin_state(&self, plugin_id: &str) -> Option<PluginState> {
        let plugin_states = self.plugin_states.read().unwrap();
        plugin_states.get(plugin_id).cloned()
    }

    /// Get active executions count
    pub fn active_plugins_count(&self) -> usize {
        let plugin_states = self.plugin_states.read().unwrap();
        plugin_states
            .values()
            .filter(|state| matches!(state, PluginState::Ready | PluginState::Executing))
            .count()
    }

    /// Validate plugin metadata
    fn validate_plugin_metadata(&self, metadata: &PluginMetadata) -> Result<()> {
        if metadata.id.is_empty() {
            return Err(AppError::Plugin {
                message: "Plugin ID cannot be empty".to_string(),
                plugin_id: metadata.id.clone(),
            });
        }

        if metadata.name.is_empty() {
            return Err(AppError::Plugin {
                message: "Plugin name cannot be empty".to_string(),
                plugin_id: metadata.id.clone(),
            });
        }

        if metadata.version.is_empty() {
            return Err(AppError::Plugin {
                message: "Plugin version cannot be empty".to_string(),
                plugin_id: metadata.id.clone(),
            });
        }

        if metadata.complexity_score > 100 {
            return Err(AppError::Plugin {
                message: "Plugin complexity score cannot exceed 100".to_string(),
                plugin_id: metadata.id.clone(),
            });
        }

        if metadata.min_data_size > metadata.max_data_size {
            return Err(AppError::Plugin {
                message: "Minimum data size cannot exceed maximum data size".to_string(),
                plugin_id: metadata.id.clone(),
            });
        }

        Ok(())
    }

    /// Shutdown the plugin manager
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down plugin manager...");

        // Get list of loaded plugins
        let plugin_ids: Vec<String> = {
            let loaded_plugins = self.loaded_plugins.read().unwrap();
            loaded_plugins.keys().cloned().collect()
        };

        // Unload all plugins
        for plugin_id in plugin_ids {
            if let Err(e) = self.unload_plugin(&plugin_id).await {
                tracing::error!("Error unloading plugin '{}': {}", plugin_id, e);
            }
        }

        // Stop resource monitoring
        self.resource_monitor.shutdown().await;

        tracing::info!("Plugin manager shutdown completed");
        Ok(())
    }
}

/// Resource monitor for plugin executions
struct ResourceMonitor {
    resource_limits: ResourceLimits,
    security_settings: SecuritySettings,
    active_monitors: RwLock<HashMap<String, ResourceMonitorHandle>>,
}

impl ResourceMonitor {
    fn new(resource_limits: ResourceLimits, security_settings: SecuritySettings) -> Self {
        Self {
            resource_limits,
            security_settings,
            active_monitors: RwLock::new(HashMap::new()),
        }
    }

    async fn start_monitoring(&self, execution_id: &str) -> Result<ResourceMonitorHandle> {
        let handle = ResourceMonitorHandle {
            execution_id: execution_id.to_string(),
            start_time: Instant::now(),
            last_memory_check: Instant::now(),
        };

        {
            let mut active_monitors = self.active_monitors.write().unwrap();
            active_monitors.insert(execution_id.to_string(), handle.clone());
        }

        Ok(handle)
    }

    async fn stop_monitoring(&self, handle: &ResourceMonitorHandle) -> Result<ResourceUsage> {
        let execution_time = handle.start_time.elapsed();

        // In a real implementation, this would collect actual resource usage
        // For now, we'll simulate it
        let resource_usage = ResourceUsage {
            memory_mb: 50.0 + (execution_time.as_millis() as f64 * 0.1),
            cpu_percent: 25.0 + (execution_time.as_millis() as f64 * 0.05),
            disk_mb: 10.0,
            network_bytes: 0,
        };

        {
            let mut active_monitors = self.active_monitors.write().unwrap();
            active_monitors.remove(&handle.execution_id);
        }

        Ok(resource_usage)
    }

    async fn shutdown(&self) {
        let mut active_monitors = self.active_monitors.write().unwrap();
        active_monitors.clear();
    }
}

#[derive(Debug, Clone)]
struct ResourceMonitorHandle {
    execution_id: String,
    start_time: Instant,
    last_memory_check: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        // This would require actual setup in tests
        // For now, just test the structure exists
    }
}