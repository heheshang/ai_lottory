//! Dependency Injection Container
//!
//! This module provides an IoC (Inversion of Control) container for managing
//! service dependencies, lifecycle, and promoting loose coupling between components.

pub mod container;
pub mod service_descriptor;
pub mod service_factory;
pub mod lifetime;
pub mod injection;
pub mod registry;

pub use container::Container;
pub use service_descriptor::{ServiceDescriptor, ServiceDescriptorBuilder};
pub use service_factory::{ServiceFactory, ServiceFactoryFn};
pub use lifetime::{Lifetime, ServiceScope};
pub use injection::{Inject, Injectable, ServiceProvider};
pub use registry::ServiceRegistry;

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Main dependency injection system
pub struct DependencyInjection {
    container: Arc<Container>,
    registry: Arc<ServiceRegistry>,
}

impl DependencyInjection {
    /// Create a new dependency injection system
    pub fn new() -> Self {
        let registry = Arc::new(ServiceRegistry::new());
        let container = Arc::new(Container::new(registry.clone()));

        Self { container, registry }
    }

    /// Get a reference to the container
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// Get a reference to the registry
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Initialize the DI system with default services
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing dependency injection system...");

        // Register core services
        self.register_core_services().await?;

        tracing::info!("Dependency injection system initialized successfully");
        Ok(())
    }

    /// Register core application services
    async fn register_core_services(&self) -> Result<()> {
        // Register database services
        self.register_database_services().await?;

        // Register plugin system
        self.register_plugin_services().await?;

        // Register performance services
        self.register_performance_services().await?;

        // Register cache services
        self.register_cache_services().await?;

        // Register analysis services
        self.register_analysis_services().await?;

        Ok(())
    }

    async fn register_database_services(&self) -> Result<()> {
        // Database connection pool
        self.registry.register(
            "database_pool",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .factory(ServiceFactoryFn::new(|_| {
                    // This would create the actual database pool
                    // For now, return a placeholder
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        // Query optimizer
        self.registry.register(
            "query_optimizer",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .dependencies(&["database_pool"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _pool = sp.get_required::<Arc<()>>("database_pool")?;
                    // Create actual query optimizer
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        Ok(())
    }

    async fn register_plugin_services(&self) -> Result<()> {
        // Plugin system
        self.registry.register(
            "plugin_system",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .factory(ServiceFactoryFn::new(|_| {
                    // Create actual plugin system
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        // Plugin manager
        self.registry.register(
            "plugin_manager",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .dependencies(&["plugin_system"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _plugin_system = sp.get_required::<Arc<()>>("plugin_system")?;
                    // Create actual plugin manager
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        Ok(())
    }

    async fn register_performance_services(&self) -> Result<()> {
        // Performance tracker
        self.registry.register(
            "performance_tracker",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .factory(ServiceFactoryFn::new(|_| {
                    // Create actual performance tracker
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        // Performance metrics
        self.registry.register(
            "performance_metrics",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .dependencies(&["performance_tracker"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _tracker = sp.get_required::<Arc<()>>("performance_tracker")?;
                    // Create actual metrics collector
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        Ok(())
    }

    async fn register_cache_services(&self) -> Result<()> {
        // Cache manager
        self.registry.register(
            "cache_manager",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .factory(ServiceFactoryFn::new(|_| {
                    // Create actual cache manager
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        // Cache invalidator
        self.registry.register(
            "cache_invalidator",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Singleton)
                .dependencies(&["cache_manager"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _cache_manager = sp.get_required::<Arc<()>>("cache_manager")?;
                    // Create actual cache invalidator
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        Ok(())
    }

    async fn register_analysis_services(&self) -> Result<()> {
        // Analysis service
        self.registry.register(
            "analysis_service",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Scoped)
                .dependencies(&["query_optimizer", "cache_manager"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _optimizer = sp.get_required::<Arc<()>>("query_optimizer")?;
                    let _cache_manager = sp.get_required::<Arc<()>>("cache_manager")?;
                    // Create actual analysis service
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        // Streaming analysis service
        self.registry.register(
            "streaming_service",
            ServiceDescriptor::builder()
                .lifetime(Lifetime::Transient)
                .dependencies(&["query_optimizer"])
                .factory(ServiceFactoryFn::new(|sp| {
                    let _optimizer = sp.get_required::<Arc<()>>("query_optimizer")?;
                    // Create actual streaming service
                    Ok(Arc::new(()))
                }))
                .build(),
        )?;

        Ok(())
    }

    /// Shutdown the DI system
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down dependency injection system...");
        self.container.shutdown().await?;
        tracing::info!("Dependency injection system shut down successfully");
        Ok(())
    }
}

/// DI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIConfig {
    /// Enable automatic service discovery
    pub enable_auto_discovery: bool,
    /// Service discovery paths
    pub discovery_paths: Vec<String>,
    /// Enable circular dependency detection
    pub enable_circular_dependency_detection: bool,
    /// Maximum service creation depth
    pub max_creation_depth: usize,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for DIConfig {
    fn default() -> Self {
        Self {
            enable_auto_discovery: false,
            discovery_paths: vec!["services".to_string()],
            enable_circular_dependency_detection: true,
            max_creation_depth: 50,
            enable_performance_monitoring: true,
        }
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_id: String,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub check_duration_ms: u64,
    pub details: HashMap<String, serde_json::Value>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// DI Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIStatistics {
    pub total_services: usize,
    pub active_services: usize,
    pub total_resolutions: u64,
    pub average_resolution_time_ms: f64,
    pub cache_hit_rate: f64,
    pub error_count: u64,
    pub memory_usage_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_di_creation() {
        let di = DependencyInjection::new();
        assert!(di.container().is_ready());
        assert!(di.registry().is_empty());
    }

    #[test]
    fn test_di_config() {
        let config = DIConfig::default();
        assert!(!config.enable_auto_discovery);
        assert_eq!(config.discovery_paths.len(), 1);
        assert!(config.enable_circular_dependency_detection);
    }
}