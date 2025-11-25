//! Service Lifetime and Scoping for Dependency Injection

use serde::{Deserialize, Serialize};
use std::fmt;

/// Service lifetime determines how service instances are created and cached
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lifetime {
    /// Single instance for the entire application lifetime
    Singleton,
    /// One instance per scope (request, session, etc.)
    Scoped,
    /// New instance created each time it's requested
    Transient,
}

impl Lifetime {
    /// Get the default lifetime for a service type
    pub fn default_for(service_type: ServiceType) -> Self {
        match service_type {
            ServiceType::Configuration => Lifetime::Singleton,
            ServiceType::Database => Lifetime::Singleton,
            ServiceType::Cache => Lifetime::Singleton,
            ServiceType::Service => Lifetime::Scoped,
            ServiceType::Repository => Lifetime::Transient,
            ServiceType::Controller => Lifetime::Transient,
            ServiceType::Utility => Lifetime::Singleton,
            ServiceType::Infrastructure => Lifetime::Singleton,
        }
    }

    /// Check if this lifetime requires caching
    pub fn requires_caching(&self) -> bool {
        !matches!(self, Lifetime::Transient)
    }

    /// Get the creation cost for this lifetime
    pub fn creation_cost(&self) -> CreationCost {
        match self {
            Lifetime::Singleton => CreationCost::High,
            Lifetime::Scoped => CreationCost::Medium,
            Lifetime::Transient => CreationCost::Low,
        }
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lifetime::Singleton => write!(f, "Singleton"),
            Lifetime::Scoped => write!(f, "Scoped"),
            Lifetime::Transient => write!(f, "Transient"),
        }
    }
}

/// Service scope for managing service lifetimes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceScope {
    /// Root scope (application level)
    Root,
    /// Child scope with parent reference
    Child(Box<ServiceScope>),
}

impl ServiceScope {
    /// Create a new root scope
    pub fn root() -> Self {
        Self::Root
    }

    /// Create a child scope
    pub fn child(parent: ServiceScope) -> Self {
        Self::Child(Box::new(parent))
    }

    /// Get the parent scope
    pub fn parent(&self) -> Option<&ServiceScope> {
        match self {
            ServiceScope::Root => None,
            ServiceScope::Child(parent) => Some(parent.as_ref()),
        }
    }

    /// Check if this is a root scope
    pub fn is_root(&self) -> bool {
        matches!(self, ServiceScope::Root)
    }

    /// Get the depth of this scope
    pub fn depth(&self) -> usize {
        match self {
            ServiceScope::Root => 0,
            ServiceScope::Child(parent) => 1 + parent.depth(),
        }
    }

    /// Get the path from root
    pub fn path(&self) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            match scope {
                ServiceScope::Root => break,
                ServiceScope::Child(parent) => {
                    current = Some(parent.as_ref());
                }
            }
        }

        path
    }

    /// Get a string representation
    pub fn to_string(&self) -> String {
        match self {
            ServiceScope::Root => "Root".to_string(),
            ServiceScope::Child(parent) => format!("Child({})", parent.to_string()),
        }
    }
}

impl Default for ServiceScope {
    fn default() -> Self {
        Self::Root
    }
}

/// Service type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Configuration,
    Database,
    Cache,
    Service,
    Repository,
    Controller,
    Utility,
    Infrastructure,
}

impl ServiceType {
    /// Get a human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ServiceType::Configuration => "Configuration",
            ServiceType::Database => "Database",
            ServiceType::Cache => "Cache",
            ServiceType::Service => "Service",
            ServiceType::Repository => "Repository",
            ServiceType::Controller => "Controller",
            ServiceType::Utility => "Utility",
            ServiceType::Infrastructure => "Infrastructure",
        }
    }

    /// Get the recommended lifetime for this service type
    pub fn recommended_lifetime(&self) -> Lifetime {
        Lifetime::default_for(*self)
    }

    /// Check if this service type should be thread-safe
    pub fn should_be_thread_safe(&self) -> bool {
        !matches!(self, ServiceType::Controller)
    }
}

/// Service creation cost
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationCost {
    Low,    // < 1ms
    Medium, // 1-10ms
    High,   // > 10ms
}

impl CreationCost {
    /// Get the cost as milliseconds
    pub fn as_millis(&self) -> f64 {
        match self {
            CreationCost::Low => 0.5,
            CreationCost::Medium => 5.0,
            CreationCost::High => 50.0,
        }
    }

    /// Get the cost tier
    pub fn tier(&self) -> u8 {
        match self {
            CreationCost::Low => 1,
            CreationCost::Medium => 2,
            CreationCost::High => 3,
        }
    }
}

/// Lifetime manager for tracking service creation and cleanup
pub struct LifetimeManager {
    active_scopes: std::collections::HashMap<String, ServiceScope>,
    scope_stats: std::collections::HashMap<String, ScopeStats>,
}

impl LifetimeManager {
    /// Create a new lifetime manager
    pub fn new() -> Self {
        Self {
            active_scopes: std::collections::HashMap::new(),
            scope_stats: std::collections::HashMap::new(),
        }
    }

    /// Create a new scope
    pub fn create_scope(&mut self, scope_id: &str, parent_id: Option<&str>) -> Result<ServiceScope> {
        let scope = if let Some(parent_id) = parent_id {
            let parent = self.active_scopes.get(parent_id)
                .ok_or_else(|| {
                    crate::error::AppError::DependencyInjection {
                        message: format!("Parent scope '{}' not found", parent_id),
                        service_id: "lifetime_manager".to_string(),
                    }
                })?;
            ServiceScope::child(parent.clone())
        } else {
            ServiceScope::root()
        };

        self.active_scopes.insert(scope_id.to_string(), scope.clone());
        self.scope_stats.insert(scope_id.to_string(), ScopeStats::new());

        Ok(scope)
    }

    /// Get a scope by ID
    pub fn get_scope(&self, scope_id: &str) -> Option<&ServiceScope> {
        self.active_scopes.get(scope_id)
    }

    /// Destroy a scope
    pub fn destroy_scope(&mut self, scope_id: &str) -> Result<()> {
        // Check if any child scopes exist
        let child_count = self.active_scopes
            .values()
            .filter(|scope| {
                scope.parent()
                    .and_then(|p| p.parent())
                    .map_or(false, |gp| {
                        gp.to_string() == scope_id ||
                        (gp.to_string().is_empty() && matches!(gp, ServiceScope::Root))
                    }))
                    .unwrap_or(false)
            })
            .count();

        if child_count > 0 {
            return Err(crate::error::AppError::DependencyInjection {
                message: format!("Cannot destroy scope '{}': {} child scopes exist", scope_id, child_count),
                service_id: "lifetime_manager".to_string(),
            });
        }

        self.active_scopes.remove(scope_id);
        self.scope_stats.remove(scope_id);
        Ok(())
    
    }

    /// Get statistics for a scope
    pub fn get_scope_stats(&self, scope_id: &str) -> Option<&ScopeStats> {
        self.scope_stats.get(scope_id)
    }

    /// Update scope statistics
    pub fn update_scope_stats<F>(&mut self, scope_id: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut ScopeStats),
    {
        if let Some(stats) = self.scope_stats.get_mut(scope_id) {
            update_fn(stats);
            Ok(())
        } else {
            Err(crate::error::AppError::DependencyInjection {
                message: format!("Scope '{}' not found", scope_id),
                service_id: "lifetime_manager".to_string(),
            })
        }
    }

    /// Get all active scopes
    pub fn active_scopes(&self) -> impl Iterator<Item = (&str, &ServiceScope)> {
        self.active_scopes.iter().map(|(id, scope)| (id.as_str(), scope))
    }

    /// Get scope hierarchy as a string
    pub fn get_hierarchy(&self) -> String {
        let mut hierarchy = String::new();
        let mut indent = 0;

        let root_scopes: Vec<_> = self.active_scopes
            .values()
            .filter(|scope| scope.is_root())
            .collect();

        for scope in root_scopes {
            self.format_scope_hierarchy(&mut hierarchy, &mut indent, scope);
        }

        hierarchy
    }

    fn format_scope_hierarchy(
        &self,
        hierarchy: &mut String,
        indent: &mut usize,
        scope: &ServiceScope,
    ) {
        hierarchy.push_str(&"  ".repeat(*indent));
        hierarchy.push_str(&format!("Scope ({})\n", scope.to_string()));
        *indent += 1;

        let child_scopes: Vec<_> = self.active_scopes
            .values()
            .filter(|s| s.parent().map_or(false, |p| std::ptr::eq(p, scope)))
            .collect();

        for child in child_scopes {
            self.format_scope_hierarchy(hierarchy, indent, child);
        }

        *indent -= 1;
    }
}

/// Statistics for a scope
#[derive(Debug, Clone, Default)]
pub struct ScopeStats {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub services_created: u64,
    pub services_destroyed: u64,
    pub peak_memory_mb: f64,
    pub total_creation_time_ms: f64,
    pub active_services: u64,
}

impl ScopeStats {
    /// Create new scope statistics
    pub fn new() -> Self {
        Self {
            created_at: chrono::Utc::now(),
            services_created: 0,
            services_destroyed: 0,
            peak_memory_mb: 0.0,
            total_creation_time_ms: 0.0,
            active_services: 0,
        }
    }

    /// Record service creation
    pub fn record_service_creation(&mut self, creation_time_ms: f64) {
        self.services_created += 1;
        self.active_services += 1;
        self.total_creation_time_ms += creation_time_ms;
    }

    /// Record service destruction
    pub fn record_service_destruction(&mut self) {
        self.services_destroyed += 1;
        self.active_services = self.active_services.saturating_sub(1);
    }

    /// Update peak memory usage
    pub fn update_peak_memory(&mut self, memory_mb: f64) {
        self.peak_memory_mb = self.peak_memory_mb.max(memory_mb);
    }

    /// Get service creation rate per second
    pub fn creation_rate_per_sec(&self) -> f64 {
        let duration = (chrono::Utc::now() - self.created_at).num_milliseconds() as f64;
        if duration > 0.0 {
            (self.services_created as f64) / (duration / 1000.0)
        } else {
            0.0
        }
    }

    /// Get average creation time in milliseconds
    pub fn avg_creation_time_ms(&self) -> f64 {
        if self.services_created > 0 {
            self.total_creation_time_ms / self.services_created as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_display() {
        assert_eq!(format!("{}", Lifetime::Singleton), "Singleton");
        assert_eq!(format!("{}", Lifetime::Scoped), "Scoped");
        assert_eq!(format!("{}", Lifetime::Transient), "Transient");
    }

    #[test]
    fn test_service_scope() {
        let root = ServiceScope::root();
        assert!(root.is_root());
        assert_eq!(root.depth(), 0);
        assert_eq!(root.parent(), None);

        let child = ServiceScope::child(root);
        assert!(!child.is_root());
        assert_eq!(child.depth(), 1);
        assert!(child.parent().is_some());
    }

    #[test]
    fn test_lifetime_manager() {
        let mut manager = LifetimeManager::new();

        let root_scope = manager.create_scope("root", None).unwrap();
        assert!(root_scope.is_root());

        let child_scope = manager.create_scope("child", Some("root")).unwrap();
        assert_eq!(child_scope.depth(), 1);

        // Test destroying scope with children should fail
        assert!(manager.destroy_scope("root").is_err());

        // Destroy child scope should succeed
        assert!(manager.destroy_scope("child").is_ok());
        assert!(manager.get_scope("child").is_none());
    }

    #[test]
    fn test_scope_stats() {
        let mut stats = ScopeStats::new();
        assert_eq!(stats.services_created, 0);
        assert_eq!(stats.active_services, 0);

        stats.record_service_creation(5.0);
        assert_eq!(stats.services_created, 1);
        assert_eq!(stats.active_services, 1);
        assert_eq!(stats.avg_creation_time_ms(), 5.0);

        stats.record_service_destruction();
        assert_eq!(stats.services_destroyed, 1);
        assert_eq!(stats.active_services, 0);
    }
}