//! Service Factory for Dependency Injection

use crate::error::{AppError, Result};
use crate::di::ServiceProvider;
use serde_json::Value;
use std::sync::Arc;

/// Service factory trait for creating service instances
pub trait ServiceFactory: Send + Sync {
    /// Create a service instance using the provided service provider
    fn create(&self, provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>>;

    /// Get the service type name for debugging
    fn service_type(&self) -> &'static str;
}

/// Function-based service factory
pub struct ServiceFactoryFn<F> {
    factory: F,
    service_type: &'static str,
}

impl<F> ServiceFactoryFn<F>
where
    F: Fn(&dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> + Send + Sync,
{
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            service_type: "unknown",
        }
    }

    pub fn with_service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }
}

impl<F> ServiceFactory for ServiceFactoryFn<F>
where
    F: Fn(&dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> + Send + Sync,
{
    fn create(&self, provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        (self.factory)(provider)
    }

    fn service_type(&self) -> &'static str {
        self.service_type
    }
}

/// Builder-based service factory
pub struct BuilderFactory<T> {
    builder: Box<dyn Fn(&dyn ServiceProvider) -> Result<T> + Send + Sync>,
    service_type: &'static str,
}

impl<T> BuilderFactory<T>
where
    T: Send + Sync + 'static,
{
    pub fn new<F>(builder: F) -> Self
    where
        F: Fn(&dyn ServiceProvider) -> Result<T> + Send + Sync + 'static,
    {
        Self {
            builder: Box::new(builder),
            service_type: std::any::type_name::<T>(),
        }
    }

    pub fn with_service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }
}

impl<T> ServiceFactory for BuilderFactory<T>
where
    T: Send + Sync + 'static,
{
    fn create(&self, provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        let instance = (self.builder)(provider)?;
        Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn service_type(&self) -> &'static str {
        self.service_type
    }
}

/// Singleton service factory
pub struct SingletonFactory<T> {
    instance: Option<Arc<T>>,
    factory: Option<Box<dyn Fn(&dyn ServiceProvider) -> Result<T> + Send + Sync>>,
    service_type: &'static str,
}

impl<T> SingletonFactory<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(instance: T) -> Self {
        Self {
            instance: Some(Arc::new(instance)),
            factory: None,
            service_type: std::any::type_name::<T>(),
        }
    }

    pub fn with_factory<F>(factory: F) -> Self
    where
        F: Fn(&dyn ServiceProvider) -> Result<T> + Send + Sync + 'static,
    {
        Self {
            instance: None,
            factory: Some(Box::new(factory)),
            service_type: std::any::type_name::<T>(),
        }
    }

    pub fn with_service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }
}

impl<T> ServiceFactory for SingletonFactory<T>
where
    T: Send + Sync + 'static,
{
    fn create(&self, provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        if let Some(ref instance) = self.instance {
            return Ok(Arc::clone(instance) as Arc<dyn std::any::Any + Send + Sync>);
        }

        if let Some(ref factory) = self.factory {
            let instance = factory(provider)?;
            Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>)
        } else {
            Err(AppError::DependencyInjection {
                message: "Singleton factory has no instance or factory function".to_string(),
                service_id: self.service_type.to_string(),
            })
        }
    }

    fn service_type(&self) -> &'static str {
        self.service_type
    }
}

/// Lazy factory that creates instances on first use
pub struct LazyFactory<T> {
    creator: Box<dyn Fn() -> Result<T> + Send + Sync>,
    service_type: &'static str,
}

impl<T> LazyFactory<T>
where
    T: Send + Sync + 'static,
{
    pub fn new<F>(creator: F) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            creator: Box::new(creator),
            service_type: std::any::type_name::<T>(),
        }
    }

    pub fn with_service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }
}

impl<T> ServiceFactory for LazyFactory<T>
where
    T: Send + Sync + 'static,
{
    fn create(&self, _provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        let instance = (self.creator)();
        Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn service_type(&self) -> &'static str {
        self.service_type
    }
}

/// Configuration-based factory
pub struct ConfigFactory<T> {
    config_key: String,
    default_creator: Box<dyn Fn() -> Result<T> + Send + Sync>,
    service_type: &'static str,
}

impl<T> ConfigFactory<T>
where
    T: Send + Sync + 'static,
{
    pub fn new<F>(config_key: String, default_creator: F) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            config_key,
            default_creator: Box::new(default_creator),
            service_type: std::any::type_name::<T>(),
        }
    }

    pub fn with_service_type(mut self, service_type: &'static str) -> Self {
        self.service_type = service_type;
        self
    }
}

impl<T> ServiceFactory for ConfigFactory<T>
where
    T: Send + Sync + 'static,
{
    fn create(&self, provider: &dyn ServiceProvider) -> Result<Arc<dyn std::any::Any + Send + Sync>> {
        // Try to get configuration from service provider
        if let Some(config_service) = provider.get_optional::<Arc<dyn ConfigService>>("config_service") {
            if let Ok(config_value) = config_service.get(&self.config_key) {
                // Create instance from configuration
                if let Ok(instance) = self.create_from_config(&config_value) {
                    return Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>);
                }
            }
        }

        // Fall back to default creator
        let instance = (self.default_creator)();
        Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>)
    }

    fn service_type(&self) -> &'static str {
        self.service_type
    }
}

impl<T> ConfigFactory<T>
where
    T: Send + Sync + 'static,
{
    fn create_from_config(&self, config: &Value) -> Result<T> {
        // In a real implementation, this would deserialize the config
        // For now, return a placeholder error
        Err(AppError::DependencyInjection {
            message: format!("Config-based creation not implemented for service type: {}", self.service_type),
            service_id: self.service_type.to_string(),
        })
    }
}

/// Trait for configuration services
pub trait ConfigService: Send + Sync {
    fn get(&self, key: &str) -> Result<Value>;
    fn set(&self, key: &str, value: Value) -> Result<()>;
    fn get_optional(&self, key: &str) -> Option<Value>;
}

/// Factory utilities
pub mod factory_utils {
    use super::*;

    /// Create a factory from a closure
    pub fn from_fn<T, F>(factory: F) -> ServiceFactoryFn<F>
    where
        F: Fn(&dyn ServiceProvider) -> Result<Arc<T>> + Send + Sync,
        T: Send + Sync + 'static,
    {
        ServiceFactoryFn::new(move |provider| {
            let result = (factory)(provider)?;
            Ok(result as Arc<dyn std::any::Any + Send + Sync>)
        }).with_service_type(std::any::type_name::<T>())
    }

    /// Create a factory that returns a constant value
    pub fn constant<T>(value: T) -> SingletonFactory<T>
    where
        T: Send + Sync + 'static,
    {
        SingletonFactory::new(value)
    }

    /// Create a factory from a builder function
    pub fn builder<T, F>(builder: F) -> BuilderFactory<T>
    where
        F: Fn(&dyn ServiceProvider) -> Result<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        BuilderFactory::new(builder)
    }

    /// Create a lazy factory
    pub fn lazy<T, F>(creator: F) -> LazyFactory<T>
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        LazyFactory::new(creator)
    }

    /// Create a configuration-based factory
    pub fn from_config<T, F>(
        config_key: String,
        default_creator: F,
    ) -> ConfigFactory<T>
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        ConfigFactory::new(config_key, default_creator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::MockServiceProvider;

    struct MockService {
        value: String,
    }

    impl MockService {
        fn new(value: String) -> Self {
            Self { value }
        }
    }

    #[test]
    fn test_service_factory_fn() {
        let factory = factory_utils::from_fn(|_provider: &dyn ServiceProvider| {
            Ok(Arc::new(MockService::new("test".to_string())))
        });

        let mock_provider = MockServiceProvider::new();
        let result = factory.create(&mock_provider);
        assert!(result.is_ok());
    }

    #[test]
    fn test_singleton_factory() {
        let factory = factory_utils::constant(MockService::new("singleton".to_string()));

        let mock_provider = MockServiceProvider::new();
        let result1 = factory.create(&mock_provider);
        let result2 = factory.create(&mock_provider);
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        // Both should point to the same instance
    }

    #[test]
    fn test_lazy_factory() {
        let factory = factory_utils::lazy(|| Ok(MockService::new("lazy".to_string())));

        let mock_provider = MockServiceProvider::new();
        let result = factory.create(&mock_provider);
        assert!(result.is_ok());
    }
}

// Mock service provider for testing
#[cfg(test)]
pub struct MockServiceProvider {
    services: std::collections::HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

#[cfg(test)]
impl MockServiceProvider {
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, service_id: &str, service: Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        self.services.insert(service_id.to_string(), service);
    }
}

#[cfg(test)]
impl ServiceProvider for MockServiceProvider {
    fn get<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .get(service_id)
            .and_then(|s| s.clone().downcast().ok())
            .ok_or_else(|| AppError::DependencyInjection {
                message: format!("Service '{}' not found in mock provider", service_id),
                service_id: service_id.to_string(),
            })
    }

    fn get_optional<T>(&self, service_id: &str) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .get(service_id)
            .and_then(|s| s.clone().downcast().ok())
    }
}