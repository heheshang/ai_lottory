//! Dependency Injection Traits and Utilities

use crate::error::{AppError, Result};
use std::sync::Arc;

/// Service provider interface for dependency resolution
pub trait ServiceProvider: Send + Sync {
    /// Resolve a service by type and identifier
    fn get<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static;

    /// Optionally resolve a service (returns None if not found)
    fn get_optional<T>(&self, service_id: &str) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static;

    /// Get all service IDs available
    fn get_service_ids(&self) -> Vec<String>;
}

/// Injectable trait for services that can be injected
pub trait Injectable: Send + Sync {
    /// Get the dependencies this service requires
    fn dependencies() -> Vec<String>;

    /// Initialize the service with dependencies
    fn initialize(&mut self, provider: &dyn ServiceProvider) -> Result<()>;

    /// Get the service ID
    fn service_id(&self) -> &'static str;

    /// Get the service metadata
    fn metadata() -> ServiceMetadata;
}

/// Service metadata
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub provides: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ServiceMetadata {
    /// Create new service metadata
    pub fn new(
        id: String,
        name: String,
        description: String,
        version: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            version,
            dependencies: Vec::new(),
            provides: Vec::new(),
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Add provided services
    pub fn with_provides(mut self, provides: Vec<String>) -> Self {
        self.provides = provides;
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Injection decorator trait for automatic dependency injection
pub trait Inject {
    /// Inject dependencies into the target
    fn inject(&mut self, provider: &dyn ServiceProvider) -> Result<()>;
}

/// Macro to implement Inject for a struct
#[macro_export]
macro_rules! impl_inject {
    ($struct_name:ident, $($field:ident: $field_type:ty),* $(,)?) => {
        impl $crate::di::injection::Inject for $struct_name {
            fn inject(&mut self, provider: &$crate::di::injection::ServiceProvider) -> $crate::error::Result<()> {
                $(
                    self.$field = provider.get(&stringify!($field))?;
                )*
                Ok(())
            }
        }

        impl $struct_name {
            pub fn new_with_provider(provider: &$crate::di::injection::ServiceProvider) -> $crate::error::Result<Self> {
                let mut instance = Self::default();
                instance.inject(provider)?;
                Ok(instance)
            }
        }
    };
}

/// Macro to create an injectable service
#[macro_export]
macro_rules! injectable_service {
    (
        $vis:vis struct $struct_name:ident {
            $($field_vis:vis $field_name:ident: $field_type:ty),* $(,)?
        }
        $($extra:tt)*
    ) => {
        #[derive(Debug, Default)]
        $vis struct $struct_name {
            $($field_vis $field_name: $field_type,)*
        }

        impl $crate::di::injection::Injectable for $struct_name {
            fn dependencies() -> Vec<String> {
                vec![
                    $(
                        stringify!($field_name).to_string()
                    ),*
                ]
            }

            fn initialize(&mut self, provider: &dyn $crate::di::injection::ServiceProvider) -> $crate::error::Result<()> {
                $(
                    self.$field_name = provider.get(&stringify!($field_name))?;
                )*
                Ok(())
            }

            fn service_id() -> &'static str {
                stringify!($struct_name)
            }

            fn metadata() -> $crate::di::injection::ServiceMetadata {
                $crate::di::injection::ServiceMetadata::new(
                    stringify!($struct_name).to_string(),
                    stringify!($struct_name).to_string(),
                    concat!("Injectable service: ", stringify!($struct_name)),
                    "1.0.0".to_string()
                )
                .with_dependencies(Self::dependencies())
            }
        }

        impl $crate::di::injection::Inject for $struct_name {
            fn inject(&mut self, provider: &dyn $crate::di::injection::ServiceProvider) -> $crate::error::Result<()> {
                self.initialize(provider)
            }
        }

        $($extra)*
    };
}

/// Service locator for global service access
pub struct ServiceLocator {
    provider: Option<Arc<dyn ServiceProvider>>,
}

impl ServiceLocator {
    /// Create a new service locator
    pub fn new() -> Self {
        Self { provider: None }
    }

    /// Set the service provider
    pub fn set_provider(&mut self, provider: Arc<dyn ServiceProvider>) {
        self.provider = Some(provider);
    }

    /// Get the current service provider
    pub fn provider(&self) -> Result<&Arc<dyn ServiceProvider>> {
        self.provider.as_ref().ok_or_else(|| {
            AppError::DependencyInjection {
                message: "Service provider not set".to_string(),
                service_id: "service_locator".to_string(),
            }
        })
    }

    /// Resolve a service
    pub fn resolve<T>(&self) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let provider = self.provider()?;
        // Use the service name as the service ID
        let service_id = std::any::type_name::<T>();
        provider.get(service_id)
    }

    /// Resolve a service by ID
    pub fn resolve_by_id<T>(&self, service_id: &str) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let provider = self.provider()?;
        provider.get(service_id)
    }

    /// Optionally resolve a service
    pub fn resolve_optional<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let provider = self.provider.ok()?;
        let service_id = std::any::type_name::<T>();
        provider.get_optional(service_id)
    }

    /// Check if a service is registered
    pub fn is_registered<T>(&self) -> bool {
        let provider = match &self.provider {
            Some(p) => p,
            None => return false,
        };
        let service_id = std::any::type_name::<T>();
        provider.get_service_ids().contains(&service_id.to_string())
    }
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global service locator instance
static mut GLOBAL_LOCATOR: Option<ServiceLocator> = None;
static LOCATOR_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global service locator
pub fn global_locator() -> &'static ServiceLocator {
    unsafe {
        LOCATOR_INIT.call_once(|| {
            GLOBAL_LOCATOR = Some(ServiceLocator::new());
        });
        GLOBAL_LOCATOR.as_ref().unwrap()
    }
}

/// Set the global service provider
pub fn set_global_provider(provider: Arc<dyn ServiceProvider>) -> Result<()> {
    unsafe {
        if let Some(locator) = &mut GLOBAL_LOCATOR {
            locator.set_provider(provider);
            Ok(())
        } else {
            Err(AppError::DependencyInjection {
                message: "Global locator not initialized".to_string(),
                service_id: "global_locator".to_string(),
            })
        }
    }
}

/// Convenient macro to resolve a service from the global locator
#[macro_export]
macro_rules! resolve_service {
    ($service_type:ty) => {
        $crate::di::injection::global_locator().resolve::<$service_type>()
    };
    ($service_type:ty, $service_id:expr) => {
        $crate::di::injection::global_locator().resolve_by_id::<$service_type>($service_id)
    };
}

/// Mock service provider for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    pub struct MockServiceProvider {
        services: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
    }

    impl MockServiceProvider {
        pub fn new() -> Self {
            Self {
                services: HashMap::new(),
            }
        }

        pub fn register<T>(&mut self, service_id: &str, service: Arc<T>)
        where
            T: Send + Sync + 'static,
        {
            self.services.insert(service_id.to_string(), service);
        }
    }

        pub fn with_service<T>(mut self, service_id: &str, service: Arc<T>) -> Self
        where
            T: Send + Sync + 'static,
        {
            self.register(service_id, service);
            self
        }
    }

    impl ServiceProvider for MockServiceProvider {
        fn get<T>(&self, service_id: &str) -> Result<Arc<T>>
        where
            T: Send + Sync + 'static,
        {
            self.services
                .get(service_id)
                .and_then(|s| s.clone().downcast().ok())
                .ok_or_else(|| AppError::DependencyInjection {
                    message: format!("Service '{}' not found", service_id),
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

        fn get_service_ids(&self) -> Vec<String> {
            self.services.keys().cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct TestService {
        value: String,
    }

    #[test]
    fn test_service_locator() {
        let mut locator = ServiceLocator::new();

        // Should fail when no provider is set
        assert!(locator.resolve::<TestService>().is_err());

        // Set up mock provider
        let mut mock_provider = mock::MockServiceProvider::new();
        let test_service = Arc::new(TestService {
            value: "test".to_string(),
        });
        mock_provider.register("test_service", test_service);

        let provider = Arc::new(mock_provider);
        locator.set_provider(provider);

        // Should succeed now
        let resolved = locator.resolve::<TestService>();
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap().value, "test");
    }

    #[test]
    fn test_global_locator() {
        let resolved = global_locator().resolve_optional::<TestService>();
        assert!(resolved.is_none()); // No provider set yet
    }
}