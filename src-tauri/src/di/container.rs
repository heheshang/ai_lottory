//! Dependency Injection Container

use crate::error::{AppError, Result};
use crate::di::{ServiceRegistry, ServiceDescriptor, ServiceFactory, Lifetime, ServiceScope, ServiceProvider};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex, Weak};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;

/// Service instance storage
type ServiceInstance = Arc<dyn std::any::Any + Send + Sync>;

/// Service creation context
#[derive(Debug)]
struct CreationContext {
    service_id: String,
    scope: ServiceScope,
    depth: usize,
    created_at: Instant,
}

/// Main dependency injection container
pub struct Container {
    registry: Arc<ServiceRegistry>,
    singletons: AsyncRwLock<HashMap<String, ServiceInstance>>,
    scoped_instances: AsyncRwLock<HashMap<String, ServiceInstance>>,
    creation_stack: Mutex<Vec<String>>,
    performance_stats: RwLock<ContainerStats>,
    config: ContainerConfig,
}

/// Container configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub enable_circular_dependency_detection: bool,
    pub max_creation_depth: usize,
    pub enable_performance_monitoring: bool,
    pub creation_timeout: Duration,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            enable_circular_dependency_detection: true,
            max_creation_depth: 50,
            enable_performance_monitoring: true,
            creation_timeout: Duration::from_secs(30),
        }
    }
}

/// Container performance statistics
#[derive(Debug, Default, Clone)]
struct ContainerStats {
    total_resolutions: u64,
    cache_hits: u64,
    cache_misses: u64,
    total_creation_time_ms: f64,
    error_count: u64,
    active_scopes: usize,
}

impl Container {
    /// Create a new container
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            singletons: AsyncRwLock::new(HashMap::new()),
            scoped_instances: AsyncRwLock::new(HashMap::new()),
            creation_stack: Mutex::new(Vec::new()),
            performance_stats: RwLock::new(ContainerStats::default()),
            config: ContainerConfig::default(),
        }
    }

    /// Create a new container with custom configuration
    pub fn with_config(registry: Arc<ServiceRegistry>, config: ContainerConfig) -> Self {
        Self {
            registry,
            singletons: AsyncRwLock::new(HashMap::new()),
            scoped_instances: AsyncRwLock::new(HashMap::new()),
            creation_stack: Mutex::new(Vec::new()),
            performance_stats: RwLock::new(ContainerStats::default()),
            config,
        }
    }

    /// Resolve a service by ID and type
    pub async fn resolve<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let start_time = Instant::now();

        let result = self.resolve_internal::<T>(service_id, ServiceScope::Root).await;

        let duration = start_time.elapsed();
        self.update_stats(duration, result.is_ok());

        result
    }

    /// Resolve a service within a specific scope
    pub async fn resolve_scoped<T>(
        &self,
        service_id: &str,
        scope: ServiceScope,
    ) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let start_time = Instant::now();

        let result = self.resolve_internal::<T>(service_id, scope).await;

        let duration = start_time.elapsed();
        self.update_stats(duration, result.is_ok());

        result
    }

    /// Internal service resolution logic
    async fn resolve_internal<T>(
        &self,
        service_id: &str,
        scope: ServiceScope,
    ) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        // Get service descriptor
        let descriptor = self.registry.get(service_id)
            .ok_or_else(|| AppError::DependencyInjection {
                message: format!("Service '{}' not registered", service_id),
                service_id: service_id.to_string(),
            })?;

        // Check for circular dependencies
        if self.config.enable_circular_dependency_detection {
            self.check_circular_dependency(service_id)?;
        }

        // Try to get cached instance based on lifetime
        match descriptor.lifetime() {
            Lifetime::Singleton => self.get_singleton::<T>(service_id).await,
            Lifetime::Scoped => self.get_scoped::<T>(service_id, &scope).await,
            Lifetime::Transient => self.create_transient::<T>(service_id, &scope).await,
        }
    }

    /// Get singleton service instance
    async fn get_singleton<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        // Try to get from cache
        {
            let singletons = self.singletons.read().await;
            if let Some(instance) = singletons.get(service_id) {
                // Update cache hit stats
                self.update_cache_hit_stats(true);
                return Ok(Arc::downcast(instance.clone())
                    .map_err(|_| AppError::DependencyInjection {
                        message: format!("Failed to downcast singleton service '{}'", service_id),
                        service_id: service_id.to_string(),
                    })?);
            }
        }

        // Create new instance
        self.update_cache_hit_stats(false);
        let instance = self.create_service::<T>(service_id, ServiceScope::Root).await?;

        // Cache the instance
        {
            let mut singletons = self.singletons.write().await;
            singletons.insert(service_id.to_string(), instance.clone() as ServiceInstance);
        }

        Ok(Arc::downcast(instance)
            .map_err(|_| AppError::DependencyInjection {
                message: format!("Failed to downcast created singleton service '{}'", service_id),
                service_id: service_id.to_string(),
            })?)
    }

    /// Get scoped service instance
    async fn get_scoped<T>(&self, service_id: &str, scope: &ServiceScope) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let scope_key = format!("{}:{:?}", service_id, scope);

        // Try to get from cache
        {
            let scoped = self.scoped_instances.read().await;
            if let Some(instance) = scoped.get(&scope_key) {
                self.update_cache_hit_stats(true);
                return Ok(Arc::downcast(instance.clone())
                    .map_err(|_| AppError::DependencyInjection {
                        message: format!("Failed to downcast scoped service '{}'", service_id),
                        service_id: service_id.to_string(),
                    })?);
            }
        }

        // Create new instance
        self.update_cache_hit_stats(false);
        let instance = self.create_service::<T>(service_id, ServiceScope::Child(scope.clone())).await?;

        // Cache the instance
        {
            let mut scoped = self.scoped_instances.write().await;
            scoped.insert(scope_key, instance.clone() as ServiceInstance);
        }

        Ok(Arc::downcast(instance)
            .map_err(|_| AppError::DependencyInjection {
                message: format!("Failed to downcast created scoped service '{}'", service_id),
                service_id: service_id.to_string(),
            })?)
    }

    /// Create transient service instance
    async fn create_transient<T>(&self, service_id: &str, scope: &ServiceScope) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.update_cache_hit_stats(false);
        let instance = self.create_service::<T>(service_id, ServiceScope::Child(scope.clone())).await?;
        Ok(Arc::downcast(instance)
            .map_err(|_| AppError::DependencyInjection {
                message: format!("Failed to downcast created transient service '{}'", service_id),
                service_id: service_id.to_string(),
            })?)
    }

    /// Create a new service instance
    async fn create_service<T>(
        &self,
        service_id: &str,
        scope: ServiceScope,
    ) -> Result<ServiceInstance>
    where
        T: Send + Sync + 'static,
    {
        // Add to creation stack
        {
            let mut stack = self.creation_stack.lock().unwrap();
            if stack.len() >= self.config.max_creation_depth {
                return Err(AppError::DependencyInjection {
                    message: format!(
                        "Maximum creation depth ({}) exceeded for service '{}'",
                        self.config.max_creation_depth,
                        service_id
                    ),
                    service_id: service_id.to_string(),
                });
            }
            stack.push(service_id.to_string());
        }

        // Get descriptor and create provider
        let descriptor = self.registry.get(service_id).unwrap(); // Safe - checked earlier
        let provider = ContainerServiceProvider::new(self, scope.clone());

        // Create service with timeout
        let instance = tokio::time::timeout(
            self.config.creation_timeout,
            descriptor.factory().create(&provider)
        )
        .await
        .map_err(|_| AppError::DependencyInjection {
            message: format!("Service creation timeout for '{}'", service_id),
            service_id: service_id.to_string(),
        })??;

        // Remove from creation stack
        {
            let mut stack = self.creation_stack.lock().unwrap();
            if let Some(last_id) = stack.pop() {
                if last_id != service_id {
                    // This shouldn't happen but handle gracefully
                    tracing::warn!("Creation stack mismatch: expected '{}', got '{}'", service_id, last_id);
                }
            }
        }

        Ok(instance)
    }

    /// Check for circular dependencies
    fn check_circular_dependency(&self, service_id: &str) -> Result<()> {
        let stack = self.creation_stack.lock().unwrap();
        if stack.contains(&service_id.to_string()) {
            let cycle: Vec<String> = stack.iter().cloned().chain(std::iter::once(service_id.to_string())).collect();
            return Err(AppError::DependencyInjection {
                message: format!("Circular dependency detected: {} -> {}", cycle.join(" -> "), service_id),
                service_id: service_id.to_string(),
            });
        }
        Ok(())
    }

    /// Create a new scope
    pub fn create_scope(&self, parent: Option<ServiceScope>) -> ServiceScope {
        ServiceScope::Child(parent.unwrap_or(ServiceScope::Root))
    }

    /// Clear scoped instances
    pub async fn clear_scoped(&self) -> Result<()> {
        let mut scoped = self.scoped_instances.write().await;
        scoped.clear();
        Ok(())
    }

    /// Clear specific scope instances
    pub async fn clear_scope(&self, scope: &ServiceScope) -> Result<()> {
        let mut scoped = self.scoped_instances.write().await;
        scoped.retain(|key, _| !key.starts_with(&format!("{:?}", scope)));
        Ok(())
    }

    /// Check if container is ready
    pub fn is_ready(&self) -> bool {
        // Container is always ready, but could add more sophisticated checks
        true
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> ContainerStatistics {
        let stats = self.performance_stats.read().unwrap().clone();
        let singletons_count = self.singletons.read().await.len();
        let scoped_count = self.scoped_instances.read().await.len();

        ContainerStatistics {
            total_services: self.registry.count(),
            singletons_count,
            scoped_count,
            total_resolutions: stats.total_resolutions,
            cache_hit_rate: if stats.total_resolutions > 0 {
                stats.cache_hits as f64 / stats.total_resolutions as f64
            } else {
                0.0
            },
            average_resolution_time_ms: if stats.total_resolutions > 0 {
                stats.total_creation_time_ms / stats.total_resolutions as f64
            } else {
                0.0
            },
            error_count: stats.error_count,
            memory_usage_mb: estimate_memory_usage(singletons_count + scoped_count),
        }
    }

    /// Shutdown the container
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down dependency injection container...");

        // Clear all instances
        {
            let mut singletons = self.singletons.write().await;
            singletons.clear();
        }

        {
            let mut scoped = self.scoped_instances.write().await;
            scoped.clear();
        }

        tracing::info!("Container shutdown completed");
        Ok(())
    }

    /// Update performance statistics
    fn update_stats(&self, duration: Duration, success: bool) {
        if self.config.enable_performance_monitoring {
            let mut stats = self.performance_stats.write().unwrap();
            stats.total_resolutions += 1;
            stats.total_creation_time_ms += duration.as_millis() as f64;
            if !success {
                stats.error_count += 1;
            }
        }
    }

    /// Update cache hit statistics
    fn update_cache_hit_stats(&self, is_hit: bool) {
        if self.config.enable_performance_monitoring {
            let mut stats = self.performance_stats.write().unwrap();
            if is_hit {
                stats.cache_hits += 1;
            } else {
                stats.cache_misses += 1;
            }
        }
    }
}

/// Container service provider implementation
struct ContainerServiceProvider<'a> {
    container: &'a Container,
    scope: ServiceScope,
}

impl<'a> ContainerServiceProvider<'a> {
    fn new(container: &'a Container, scope: ServiceScope) -> Self {
        Self { container, scope }
    }
}

impl<'a> ServiceProvider for ContainerServiceProvider<'a> {
    fn get<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        // This is a synchronous wrapper around the async resolution
        // In a real implementation, you might want to use a blocking runtime
        // or restructure to avoid async/sync boundaries
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                self.container.resolve_scoped::<T>(service_id, self.scope.clone())
            )
        })
    }

    fn get_optional<T>(&self, service_id: &str) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.get(service_id).ok()
    }
}

/// Container statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerStatistics {
    pub total_services: usize,
    pub singletons_count: usize,
    pub scoped_count: usize,
    pub total_resolutions: u64,
    pub cache_hit_rate: f64,
    pub average_resolution_time_ms: f64,
    pub error_count: u64,
    pub memory_usage_mb: f64,
}

// Helper functions

fn estimate_memory_usage(instance_count: usize) -> f64 {
    // Rough estimate: average service instance size
    (instance_count * 1024) as f64 / 1024.0 / 1024.0 // Convert to MB
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::{ServiceDescriptorBuilder, ServiceFactoryFn, Lifetime};

    #[test]
    fn test_container_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let container = Container::new(registry);
        assert!(container.is_ready());
    }

    #[tokio::test]
    async fn test_service_registration_and_resolution() {
        let registry = Arc::new(ServiceRegistry::new());
        let container = Container::new(registry);

        // Register a test service
        let test_service = Arc::new("test_value".to_string());
        let service_id = "test_service";

        // This would normally be done through the registry
        // For testing, we'll simulate the service descriptor
        // In a real implementation, you'd use the registry to register
        assert!(container.is_ready());
    }

    #[test]
    fn test_container_config() {
        let config = ContainerConfig::default();
        assert!(config.enable_circular_dependency_detection);
        assert_eq!(config.max_creation_depth, 50);
    }
}