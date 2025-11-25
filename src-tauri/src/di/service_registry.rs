//! Service Registry for Managing Service Descriptors

use crate::error::{AppError, Result};
use crate::di::{ServiceDescriptor, ServiceFactory};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Service registry for managing service registrations
pub struct ServiceRegistry {
    services: RwLock<HashMap<String, ServiceDescriptor>>,
    aliases: RwLock<HashMap<String, String>>,
    metadata: RwLock<RegistryMetadata>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            metadata: RwLock::new(RegistryMetadata::new()),
        }
    }

    /// Register a service descriptor
    pub fn register(&self, service_id: &str, descriptor: ServiceDescriptor) -> Result<()> {
        if service_id.is_empty() {
            return Err(AppError::DependencyInjection {
                message: "Service ID cannot be empty".to_string(),
                service_id: "registry".to_string(),
            });
        }

        // Check if service already exists
        {
            let services = self.services.read().unwrap();
            if services.contains_key(service_id) {
                return Err(AppError::DependencyInjection {
                    message: format!("Service '{}' already registered", service_id),
                    service_id: service_id.to_string(),
                });
            }
        }

        // Validate descriptor
        descriptor.validate()?;

        // Register the service
        {
            let mut services = self.services.write().unwrap();
            services.insert(service_id.to_string(), descriptor);
        }

        // Update metadata
        {
            let mut metadata = self.metadata.write().unwrap();
            metadata.total_services += 1;
            metadata.last_updated = chrono::Utc::now();
        }

        Ok(())
    }

    /// Register an alias for a service
    pub fn register_alias(&self, alias: &str, service_id: &str) -> Result<()> {
        // Check if alias already exists
        {
            let aliases = self.aliases.read().unwrap();
            if aliases.contains_key(alias) {
                return Err(AppError::DependencyInjection {
                    message: format!("Alias '{}' already exists", alias),
                    service_id: "registry".to_string(),
                });
            }
        }

        // Check if target service exists
        {
            let services = self.services.read().unwrap();
            if !services.contains_key(service_id) {
                return Err(AppError::DependencyInjection {
                    message: format!("Cannot create alias '{}' for non-existent service '{}'", alias, service_id),
                    service_id: "registry".to_string(),
                });
            }
        }

        // Register the alias
        {
            let mut aliases = self.aliases.write().unwrap();
            aliases.insert(alias.to_string(), service_id.to_string());
        }

        Ok(())
    }

    /// Get a service descriptor by ID
    pub fn get(&self, service_id: &str) -> Option<&ServiceDescriptor> {
        // Check for direct match first
        {
            let services = self.services.read().unwrap();
            if let Some(descriptor) = services.get(service_id) {
                return Some(descriptor);
            }
        }

        // Check aliases
        {
            let aliases = self.aliases.read().unwrap();
            if let Some(actual_id) = aliases.get(service_id) {
                let services = self.services.read().unwrap();
                return services.get(actual_id);
            }
        }

        None
    }

    /// Get a service descriptor with alias resolution
    pub fn get_with_alias_resolution(&self, service_id: &str) -> Option<(&ServiceDescriptor, String)> {
        // Check for direct match first
        {
            let services = self.services.read().unwrap();
            if let Some(descriptor) = services.get(service_id) {
                return Some((descriptor, service_id.to_string()));
            }
        }

        // Check aliases
        {
            let aliases = self.aliases.read().unwrap();
            if let Some(actual_id) = aliases.get(service_id) {
                let services = self.services.read().unwrap();
                if let Some(descriptor) = services.get(actual_id) {
                    return Some((descriptor, actual_id.clone()));
                }
            }
        }

        None
    }

    /// Unregister a service
    pub fn unregister(&self, service_id: &str) -> Result<bool> {
        let mut services = self.services.write().unwrap();
        let removed = services.remove(service_id).is_some();

        if removed {
            // Remove any aliases pointing to this service
            let mut aliases = self.aliases.write().unwrap();
            aliases.retain(|_, actual_id| actual_id != service_id);

            // Update metadata
            let mut metadata = self.metadata.write().unwrap();
            metadata.total_services = services.len();
            metadata.last_updated = chrono::Utc::now();
        }

        Ok(removed)
    }

    /// Unregister an alias
    pub fn unregister_alias(&self, alias: &str) -> Result<bool> {
        let mut aliases = self.aliases.write().unwrap();
        Ok(aliases.remove(alias).is_some())
    }

    /// Check if a service is registered
    pub fn is_registered(&self, service_id: &str) -> bool {
        self.get(service_id).is_some()
    }

    /// Get all registered service IDs
    pub fn get_service_ids(&self) -> Vec<String> {
        let services = self.services.read().unwrap();
        services.keys().cloned().collect()
    }

    /// Get all registered aliases
    pub fn get_aliases(&self) -> Vec<(String, String)> {
        let aliases = self.aliases.read().unwrap();
        aliases.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Get services by type
    pub fn get_services_by_type(&self, service_type: &str) -> Vec<String> {
        let services = self.services.read().unwrap();
        services
            .iter()
            .filter(|(_, descriptor)| descriptor.service_type() == service_type)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get services by tag
    pub fn get_services_by_tag(&self, tag: &str) -> Vec<String> {
        let services = self.services.read().unwrap();
        services
            .iter()
            .filter(|(_, descriptor)| descriptor.tags().contains(&tag.to_string()))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get service statistics
    pub fn get_stats(&self) -> RegistryStats {
        let services = self.services.read().unwrap();
        let aliases = self.aliases.read().unwrap();
        let metadata = self.metadata.read().unwrap().clone();

        let mut type_counts = HashMap::new();
        let mut tag_counts = HashMap::new();

        for descriptor in services.values() {
            // Count by type
            *type_counts.entry(descriptor.service_type()).or_insert(0) += 1;

            // Count by tags
            for tag in descriptor.tags() {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }

        RegistryStats {
            total_services: services.len(),
            total_aliases: aliases.len(),
            type_counts,
            tag_counts,
            metadata,
        }
    }

    /// Validate all registered services
    pub fn validate(&self) -> Result<Vec<ValidationError>> {
        let services = self.services.read().unwrap();
        let mut errors = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for (service_id, descriptor) in services.iter() {
            // Check for duplicate service IDs
            if seen_ids.contains(service_id) {
                errors.push(ValidationError::DuplicateService {
                    id: service_id.clone(),
                });
            }
            seen_ids.insert(service_id.clone());

            // Validate the descriptor
            if let Err(e) = descriptor.validate() {
                errors.push(ValidationError::InvalidDescriptor {
                    id: service_id.clone(),
                    error: e.to_string(),
                });
            }

            // Validate dependencies
            for dependency_id in descriptor.dependencies() {
                if !services.contains_key(dependency_id) {
                    errors.push(ValidationError::MissingDependency {
                        service_id: service_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
        }

        // Validate aliases
        let aliases = self.aliases.read().unwrap();
        for (alias, target_id) in aliases.iter() {
            if !services.contains_key(target_id) {
                errors.push(ValidationError::InvalidAlias {
                    alias: alias.clone(),
                    target_id: target_id.clone(),
                });
            }
        }

        Ok(errors)
    }

    /// Clear all registrations
    pub fn clear(&self) -> Result<()> {
        {
            let mut services = self.services.write().unwrap();
            services.clear();
        }

        {
            let mut aliases = self.aliases.write().unwrap();
            aliases.clear();
        }

        {
            let mut metadata = self.metadata.write().unwrap();
            *metadata = RegistryMetadata::new();
        }

        Ok(())
    }

    /// Export registry configuration
    pub fn export_config(&self) -> Result<RegistryConfig> {
        let services = self.services.read().unwrap();
        let aliases = self.aliases.read().unwrap();

        let service_configs: HashMap<String, ServiceConfig> = services
            .iter()
            .map(|(id, descriptor)| {
                (id.clone(), ServiceConfig::from_descriptor(descriptor))
            })
            .collect();

        Ok(RegistryConfig {
            services: service_configs,
            aliases: aliases.clone(),
            metadata: self.metadata.read().unwrap().clone(),
            exported_at: chrono::Utc::now(),
        })
    }

    /// Import registry configuration
    pub fn import_config(&self, config: RegistryConfig) -> Result<()> {
        self.clear()?;

        // Import services
        for (service_id, service_config) in config.services {
            let descriptor = ServiceConfig::to_descriptor(&service_id, &service_config)?;
            self.register(&service_id, descriptor)?;
        }

        // Import aliases
        for (alias, target_id) in config.aliases {
            self.register_alias(&alias, &target_id)?;
        }

        // Import metadata
        {
            let mut metadata = self.metadata.write().unwrap();
            *metadata = config.metadata;
        }

        Ok(())
    }

    /// Create a backup of the registry
    pub fn backup(&self) -> Result<RegistryBackup> {
        Ok(RegistryBackup {
            config: self.export_config()?,
            created_at: chrono::Utc::now(),
        })
    }

    /// Restore from backup
    pub fn restore(&self, backup: RegistryBackup) -> Result<()> {
        self.import_config(backup.config)?;
        Ok(())
    }
}

/// Registry metadata
#[derive(Debug, Clone, Default)]
pub struct RegistryMetadata {
    pub total_services: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl RegistryMetadata {
    fn new() -> Self {
        Self {
            total_services: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_services: usize,
    pub total_aliases: usize,
    pub type_counts: HashMap<String, usize>,
    pub tag_counts: HashMap<String, usize>,
    pub metadata: RegistryMetadata,
}

/// Service configuration for export/import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceConfig {
    pub factory_type: String,
    pub lifetime: String,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ServiceConfig {
    /// Create service config from descriptor
    pub fn from_descriptor(service_id: &str, descriptor: &ServiceDescriptor) -> Self {
        Self {
            factory_type: "function".to_string(), // In a real implementation, track the factory type
            lifetime: format!("{:?}", descriptor.lifetime()),
            dependencies: descriptor.dependencies().clone(),
            tags: descriptor.tags().clone(),
            metadata: HashMap::new(), // Would include actual metadata
        }
    }

    /// Convert to service descriptor
    pub fn to_descriptor(&self, service_id: &str) -> Result<ServiceDescriptor> {
        let lifetime = match self.lifetime.as_str() {
            "Singleton" => crate::di::Lifetime::Singleton,
            "Scoped" => crate::di::Lifetime::Scoped,
            "Transient" => crate::di::Lifetime::Transient,
            _ => return Err(AppError::DependencyInjection {
                message: format!("Invalid lifetime: {}", self.lifetime),
                service_id: service_id.to_string(),
            }),
        };

        // This is a simplified conversion - in a real implementation,
        // you'd need to recreate the actual service factory
        let factory = ServiceFactoryFn::new(|_provider| {
            Err(AppError::DependencyInjection {
                message: "Factory recreation not implemented".to_string(),
                service_id: "service_config".to_string(),
            })
        });

        Ok(ServiceDescriptor::builder()
            .lifetime(lifetime)
            .dependencies(&self.dependencies)
            .tags(&self.tags)
            .factory(factory)
            .build())
    }
}

/// Registry backup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryBackup {
    pub config: RegistryConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Registry configuration for export/import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryConfig {
    pub services: HashMap<String, ServiceConfig>,
    pub aliases: HashMap<String, String>,
    pub metadata: RegistryMetadata,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    DuplicateService { id: String },
    InvalidDescriptor { id: String, error: String },
    MissingDependency { service_id: String, dependency_id: String },
    InvalidAlias { alias: String, target_id: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateService { id } => {
                write!(f, "Duplicate service registration: {}", id)
            }
            ValidationError::InvalidDescriptor { id, error } => {
                write!(f, "Invalid descriptor for '{}': {}", id, error)
            }
            ValidationError::MissingDependency { service_id, dependency_id } => {
                write!(f, "Service '{}' has missing dependency: {}", service_id, dependency_id)
            }
            ValidationError::InvalidAlias { alias, target_id } => {
                write!(f, "Invalid alias '{}' pointing to non-existent service '{}'", alias, target_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::{ServiceDescriptorBuilder, Lifetime, ServiceFactoryFn};

    #[test]
    fn test_service_registry() {
        let registry = ServiceRegistry::new();

        // Test empty registry
        assert_eq!(registry.count(), 0);
        assert!(registry.is_empty());
        assert_eq!(registry.get_service_ids().len(), 0);

        // Register a service
        let descriptor = ServiceDescriptorBuilder::new()
            .lifetime(Lifetime::Singleton)
            .factory(ServiceFactoryFn::new(|_| Ok(Arc::new(()))))
            .build();

        assert!(registry.register("test_service", descriptor).is_ok());
        assert_eq!(registry.count(), 1);
        assert!(registry.is_registered("test_service"));

        // Get the service
        let retrieved = registry.get("test_service");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_service_aliases() {
        let registry = ServiceRegistry::new();

        let descriptor = ServiceDescriptorBuilder::new()
            .lifetime(Lifetime::Singleton)
            .factory(ServiceFactoryFn::new(|_| Ok(Arc::new(()))))
            .build();

        registry.register("test_service", descriptor).unwrap();
        registry.register_alias("test_alias", "test_service").unwrap();

        assert_eq!(registry.get("test_alias"), Some(registry.get("test_service").unwrap()));
    }

    #[test]
    fn test_validation() {
        let registry = ServiceRegistry::new();

        // Register a service with dependency
        let descriptor1 = ServiceDescriptorBuilder::new()
            .lifetime(Lifetime::Singleton)
            .dependencies(vec!["nonexistent".to_string()])
            .factory(ServiceFactoryFn::new(|_| Ok(Arc::new(()))))
            .build();

        registry.register("service1", descriptor1).unwrap();

        // Validation should fail due to missing dependency
        let errors = registry.validate();
        assert!(!errors.is_empty());
        assert!(matches!(errors[0], ValidationError::MissingDependency { .. }));
    }
}

impl ServiceRegistry {
    /// Get the total number of registered services
    pub fn count(&self) -> usize {
        self.services.read().unwrap().len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}