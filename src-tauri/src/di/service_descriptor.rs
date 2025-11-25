//! Service Descriptor for Dependency Injection

use crate::error::{AppError, Result};
use crate::di::{ServiceFactory, Lifetime, ServiceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Service descriptor containing metadata and factory
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    id: String,
    name: String,
    description: String,
    service_type: String,
    lifetime: Lifetime,
    factory: Arc<dyn ServiceFactory>,
    dependencies: Vec<String>,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl ServiceDescriptor {
    /// Create a new service descriptor
    pub fn new(
        id: String,
        name: String,
        description: String,
        service_type: String,
        lifetime: Lifetime,
        factory: Arc<dyn ServiceFactory>,
        dependencies: Vec<String>,
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            service_type,
            lifetime,
            factory,
            dependencies,
            tags,
            metadata,
        }
    }

    /// Get the service ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the service name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the service description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the service type
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// Get the service lifetime
    pub fn lifetime(&self) -> Lifetime {
        self.lifetime
    }

    /// Get the service factory
    pub fn factory(&self) -> &Arc<dyn ServiceFactory> {
        &self.factory
    }

    /// Get the service dependencies
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Get the service tags
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Get the service metadata
    pub fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }

    /// Validate the service descriptor
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(AppError::DependencyInjection {
                message: "Service ID cannot be empty".to_string(),
                service_id: self.id.clone(),
            });
        }

        if self.name.is_empty() {
            return Err(AppError::DependencyInjection {
                message: "Service name cannot be empty".to_string(),
                service_id: self.id.clone(),
            });
        }

        if self.service_type.is_empty() {
            return Err(AppError::DependencyInjection {
                message: "Service type cannot be empty".to_string(),
                service_id: self.id.clone(),
            });
        }

        // Validate dependencies
        for dep in &self.dependencies {
            if dep.is_empty() {
                return Err(AppError::DependencyInjection {
                    message: "Dependency ID cannot be empty".to_string(),
                    service_id: self.id.clone(),
                });
            }
        }

        // Validate that dependency list doesn't contain duplicates
        let mut seen_deps = std::collections::HashSet::new();
        for dep in &self.dependencies {
            if !seen_deps.insert(dep.clone()) {
                return Err(AppError::DependencyInjection {
                    message: format!("Duplicate dependency '{}' in service '{}'", dep, self.id),
                    service_id: self.id.clone(),
                });
            }
        }

        Ok(())
    }

    /// Check if this service depends on another service
    pub fn depends_on(&self, service_id: &str) -> bool {
        self.dependencies.contains(&service_id.to_string())
    }

    /// Get dependency depth (how many levels of dependencies)
    pub fn dependency_depth(&self, registry: &crate::di::ServiceRegistry) -> usize {
        let mut depth = 0;
        let mut visited = std::collections::HashSet::new();
        self.calculate_dependency_depth_recursive(&mut visited, registry, &mut depth);
        depth
    }

    fn calculate_dependency_depth_recursive(
        &self,
        visited: &mut std::collections::HashSet<String>,
        registry: &crate::di::ServiceRegistry,
        current_depth: &mut usize,
    ) {
        for dep_id in &self.dependencies {
            if visited.insert(dep_id.clone()) {
                continue; // Already visited
            }

            if let Some(dep_descriptor) = registry.get(dep_id) {
                *current_depth += 1;
                dep_descriptor.calculate_dependency_depth_recursive(visited, registry, current_depth);
            }
        }
    }

    /// Check if this service creates a circular dependency with another
    pub fn creates_circular_dependency_with(
        &self,
        target_service_id: &str,
        registry: &crate::di::ServiceRegistry,
    ) -> bool {
        let mut visited = std::collections::HashSet::new();
        visited.insert(self.id.clone());
        self.check_circular_dependency_recursive(&visited, registry, target_service_id)
    }

    fn check_circular_dependency_recursive(
        &self,
        visited: &mut std::collections::HashSet<String>,
        registry: &crate::di::ServiceRegistry,
        target_service_id: &str,
    ) -> bool {
        if self.id == target_service_id {
            return true;
        }

        for dep_id in &self.dependencies {
            if visited.contains(dep_id) {
                continue;
            }

            if let Some(dep_descriptor) = registry.get(dep_id) {
                visited.insert(dep_id.clone());
                if dep_descriptor.check_circular_dependency_recursive(visited, registry, target_service_id) {
                    return true;
                }
            }
        }

        false
    }

    /// Clone the descriptor with a new factory
    pub fn with_factory(&self, factory: Arc<dyn ServiceFactory>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            service_type: self.service_type.clone(),
            lifetime: self.lifetime,
            factory,
            dependencies: self.dependencies.clone(),
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Clone the descriptor with new dependencies
    pub fn with_dependencies(&self, dependencies: Vec<String>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            service_type: self.service_type.clone(),
            lifetime: self.lifetime,
            factory: self.factory.clone(),
            dependencies,
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Clone the descriptor with new tags
    pub fn with_tags(&self, tags: Vec<String>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            service_type: self.service_type.clone(),
            lifetime: self.lifetime,
            factory: self.factory.clone(),
            dependencies: self.dependencies.clone(),
            tags,
            metadata: self.metadata.clone(),
        }
    }

    /// Clone the descriptor with new metadata
    pub fn with_metadata(&self, metadata: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            service_type: self.service_type.clone(),
            lifetime: self.lifetime,
            factory: self.factory.clone(),
            dependencies: self.dependencies.clone(),
            tags: self.tags.clone(),
            metadata,
        }
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| AppError::Serialization {
            message: format!("Failed to serialize service descriptor: {}", e),
            context: "service_descriptor".to_string(),
        })
    }

    /// Create from JSON representation
    pub fn from_json(json: &serde_json::Value) -> Result<Self> {
        serde_json::from_value(json).map_err(|e| AppError::Deserialization {
            message: format!("Failed to deserialize service descriptor: {}", e),
            context: "service_descriptor".to_string(),
        })
    }

    /// Get a summary of the service descriptor
    pub fn summary(&self) -> ServiceDescriptorSummary {
        ServiceDescriptorSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            service_type: self.service_type.clone(),
            lifetime: self.lifetime,
            dependency_count: self.dependencies.len(),
            tag_count: self.tags.len(),
            metadata_keys: self.metadata.keys().cloned().collect(),
        }
    }
}

/// Service descriptor summary
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct ServiceDescriptorSummary {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub lifetime: Lifetime,
    pub dependency_count: usize,
    pub tag_count: usize,
    pub metadata_keys: Vec<String>,
}

/// Service descriptor builder
pub struct ServiceDescriptorBuilder {
    id: String,
    name: String,
    description: String,
    service_type: String,
    lifetime: Lifetime,
    factory: Option<Arc<dyn ServiceFactory>>,
    dependencies: Vec<String>,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl ServiceDescriptorBuilder {
    /// Create a new service descriptor builder
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            service_type: "service".to_string(),
            lifetime: Lifetime::Transient,
            factory: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the service ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the service name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the service description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the service type
    pub fn service_type(mut self, service_type: impl Into<String>) -> Self {
        self.service_type = service_type.into();
        self
    }

    /// Set the service lifetime
    pub fn lifetime(mut self, lifetime: Lifetime) -> Self {
        self.lifetime = lifetime;
        self
    }

    /// Set the service factory
    pub fn factory(mut self, factory: Arc<dyn ServiceFactory>) -> Self {
        self.factory = Some(factory);
        self
    }

    /// Add a dependency
    pub fn dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies
    pub fn dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add metadata key-value pair
    pub fn metadata_entry(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the service descriptor
    pub fn build(self) -> Result<ServiceDescriptor> {
        if self.id.is_empty() {
            return Err(AppError::DependencyInjection {
                message: "Service ID is required".to_string(),
                service_id: "builder".to_string(),
            });
        }

        if self.factory.is_none() {
            return Err(AppError::DependencyInjection {
                message: "Service factory is required".to_string(),
                service_id: self.id,
            });
        }

        Ok(ServiceDescriptor::new(
            self.id,
            self.name,
            self.description,
            self.service_type,
            self.lifetime,
            self.factory.unwrap(),
            self.dependencies,
            self.consolidate_tags(),
            self.metadata,
        ))
    }

    /// Consolidate tags (remove duplicates and sort)
    fn consolidate_tags(self) -> Vec<String> {
        let mut tags: Vec<String> = self.tags;
        tags.sort();
        tags.dedup();
        tags
    }
}

impl Default for ServiceDescriptorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for creating service descriptors
pub mod builders {
    use super::*;

    /// Create a singleton service descriptor
    pub fn singleton<T>(
        service_id: &str,
        name: &str,
        factory: impl Fn() -> Result<Arc<T>> + Send + Sync + 'static,
    ) -> Result<ServiceDescriptor>
    where
        T: Send + Sync + 'static,
    {
        let descriptor = ServiceDescriptorBuilder::new()
            .id(service_id)
            .name(name)
            .service_type("singleton")
            .lifetime(Lifetime::Singleton)
            .factory(crate::di::service_factory::factory_utils::constant(factory()?))
            .build()?;

        Ok(descriptor)
    }

    /// Create a scoped service descriptor
    pub fn scoped<T>(
        service_id: &str,
        name: &str,
        factory: impl Fn() -> Result<Arc<T>> + Send + Sync + 'static,
    ) -> Result<ServiceDescriptor>
    where
        T: Send + Sync + 'static,
    {
        let descriptor = ServiceDescriptorBuilder::new()
            .id(service_id)
            .name(name)
            .service_type("scoped")
            .lifetime(Lifetime::Scoped)
            .factory(crate::di::service_factory::factory_utils::factory_fn(factory))
            .build()?;

        Ok(descriptor)
    }

    /// Create a transient service descriptor
    pub fn transient<T>(
        service_id: &str,
        name: &str,
        factory: impl Fn(&dyn crate::di::ServiceProvider) -> Result<Arc<T>> + Send + Sync + 'static,
    ) -> Result<ServiceDescriptor>
    where
        T: Send + Sync + 'static,
    {
        let descriptor = ServiceDescriptorBuilder::new()
            .id(service_id)
            .name(name)
            .service_type("transient")
            .lifetime(Lifetime::Transient)
            .factory(crate::di::service_factory::factory_utils::from_fn(factory))
            .build()?;

        Ok(descriptor)
    }

    /// Create a service descriptor from an existing instance
    pub fn from_instance<T>(
        service_id: &str,
        name: &str,
        instance: T,
    ) -> Result<ServiceDescriptor>
    where
        T: Send + Sync + 'static,
    {
        let descriptor = ServiceDescriptorBuilder::new()
            .id(service_id)
            .name(name)
            .service_type("singleton")
            .lifetime(Lifetime::Singleton)
            .factory(crate::di::service_factory::factory_utils::constant(Arc::new(instance)))
            .build()?;

        Ok(descriptor)
    }
}

/// Service descriptor utilities
pub mod utils {
    use super::*;

    /// Validate a service descriptor
    pub fn validate(descriptor: &ServiceDescriptor) -> Result<()> {
        descriptor.validate()
    }

    /// Check if two descriptors are compatible (same type and compatible lifetime)
    pub fn are_compatible(
        desc1: &ServiceDescriptor,
        desc2: &ServiceDescriptor,
    ) -> bool {
        // Services are compatible if they have the same type
        desc1.service_type() == desc2.service_type()
    }

    /// Get the creation priority for a service (lower = higher priority)
    pub fn get_creation_priority(descriptor: &ServiceDescriptor) -> u8 {
        match descriptor.lifetime() {
            Lifetime::Singleton => 1, // Highest priority
            Lifetime::Scoped => 2,
            Lifetime::Transient => 3, // Lowest priority
        }
    }

    /// Get the resource usage estimate for a service
    pub fn estimate_resource_usage(descriptor: &ServiceDescriptor) -> ResourceUsage {
        ResourceUsage {
            memory_mb: estimate_memory_usage(descriptor),
            cpu_cost: estimate_cpu_cost(descriptor),
            creation_time_ms: estimate_creation_time(descriptor),
        }
    }

    fn estimate_memory_usage(descriptor: &ServiceDescriptor) -> f64 {
        // Base memory plus per-dependency overhead
        let base = 50.0; // Base memory in MB
        let per_dependency = 10.0;
        base + (descriptor.dependencies().len() as f64 * per_dependency)
    }

    fn estimate_cpu_cost(descriptor: &ServiceDescriptor) -> CPUCost {
        match descriptor.lifetime() {
            Lifetime::Singleton => CPUCost::High, // One-time expensive creation
            Lifetime::Scoped => CPUCost::Medium,
            Lifetime::Transient => CPUCost::Low, // Quick creation
        }
    }

    fn estimate_creation_time(descriptor: &ServiceDescriptor) -> f64 {
        match descriptor.lifetime() {
            Lifetime::Singleton => 100.0, // 100ms average
            Lifetime::Scoped => 50.0,   // 50ms average
            Lifetime::Transient => 10.0,  // 10ms average
        }
    }
}

/// Resource usage estimates
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_cost: CPUCost,
    pub creation_time_ms: f64,
}

/// CPU creation cost levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CPUCost {
    Low,
    Medium,
    High,
}

impl CPUCost {
    /// Get the cost as a numerical value
    pub fn as_value(&self) -> u8 {
        match self {
            CPUCost::Low => 1,
            CPUCost::Medium => 2,
            CPUCost::High => 3,
        }
    }

    /// Get the cost description
    pub fn description(&self) -> &'static str {
        match self {
            CPUCost::Low => "Low (quick creation)",
            CPUCost::Medium => "Medium (moderate creation)",
            CPUCost::High => "High (expensive creation)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::service_factory::factory_utils;

    #[test]
    fn test_service_descriptor_builder() {
        let factory = factory_utils::constant(Arc::new(()));
        let descriptor = ServiceDescriptorBuilder::new()
            .id("test")
            .name("Test Service")
            .description("A test service")
            .service_type("test")
            .lifetime(Lifetime::Singleton)
            .factory(factory)
            .tag("test")
            .tag("example")
            .metadata_entry("version", "1.0.0")
            .build()
            .unwrap();

        assert_eq!(descriptor.id(), "test");
        assert_eq!(descriptor.name(), "Test Service");
        assert_eq!(descriptor.dependencies().len(), 0);
        assert_eq!(descriptor.tags().len(), 2);
        assert_eq!(descriptor.lifetime(), Lifetime::Singleton);
    }

    #[test]
    fn test_service_descriptor_validation() {
        let descriptor = ServiceDescriptorBuilder::new()
            .id("test")
            .name("Test Service")
            .factory(factory_utils::constant(Arc::new(())))
            .build()
            .unwrap();

        assert!(descriptor.validate().is_ok());

        // Test invalid descriptor
        let invalid_descriptor = ServiceDescriptorBuilder::new()
            .id("") // Empty ID
            .name("Test")
            .factory(factory_utils::constant(Arc::new(())))
            .build()
            .unwrap();

        assert!(invalid_descriptor.validate().is_err());
    }

    #[test]
    fn test_service_descriptor_serialization() {
        let original = ServiceDescriptorBuilder::new()
            .id("test")
            .name("Test Service")
            .description("A test service")
            .factory(factory_utils::constant(Arc::new(())))
            .build()
            .unwrap();

        let json = original.to_json().unwrap();
        let restored = ServiceDescriptor::from_json(&json).unwrap();

        assert_eq!(original.id(), restored.id());
        assert_eq!(original.name(), restored.name());
        assert_eq!(original.lifetime(), restored.lifetime());
    }

    #[test]
    fn test_dependency_analysis() {
        let registry = ServiceRegistry::new();

        // Create services with dependencies
        let service_a = ServiceDescriptorBuilder::new()
            .id("service_a")
            .dependencies(vec!["service_b".to_string()])
            .factory(factory_utils::constant(Arc::new(())))
            .build()
            .unwrap();

        let service_b = ServiceDescriptorBuilder::new()
            .id("service_b")
            .factory(factory_utils::constant(Arc::new(())))
            .build()
            .unwrap();

        registry.register("service_a", service_a).unwrap();
        registry.register("service_b", service_b).unwrap();

        let service_a = registry.get("service_a").unwrap();
        assert_eq!(service_a.dependency_depth(&registry), 1);
    }
}