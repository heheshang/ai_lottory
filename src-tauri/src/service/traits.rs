//! Service Layer Traits
//!
//! Defines the core service interfaces and business operation contracts.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use chrono::{DateTime, Utc};

/// Base service trait for all business services
#[async_trait]
pub trait Service: Send + Sync {
    /// Service metadata
    fn metadata(&self) -> &ServiceMetadata;

    /// Initialize the service
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the service gracefully
    async fn shutdown(&mut self) -> Result<()>;

    /// Health check for the service
    async fn health_check(&self) -> Result<ServiceHealth>;

    /// Get service metrics
    async fn get_metrics(&self) -> Result<ServiceMetrics>;
}

/// Domain service trait for business logic
#[async_trait]
pub trait DomainService<T, ID>: Service
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Create a new entity
    async fn create(&self, entity: &T) -> Result<T>;

    /// Find an entity by ID
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;

    /// Update an entity
    async fn update(&self, id: &ID, entity: &T) -> Result<T>;

    /// Delete an entity
    async fn delete(&self, id: &ID) -> Result<bool>;

    /// List entities with pagination
    async fn list(&self, params: &ListParams) -> Result<PaginatedResult<T>>;

    /// Search entities
    async fn search(&self, query: &SearchParams) -> Result<Vec<T>>;
}

/// Application service trait for orchestrating domain services
#[async_trait]
pub trait ApplicationService: Service {
    /// Execute a use case
    async fn execute_use_case<UC>(&self, use_case: UC) -> Result<UC::Output>
    where
        UC: UseCase + Send + 'static;

    /// Get available use cases
    fn get_available_use_cases(&self) -> Vec<&'static str>;
}

/// Query service trait for read operations
#[async_trait]
pub trait QueryService<Q, R>: Service
where
    Q: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    /// Execute a query
    async fn execute(&self, query: &Q) -> Result<R>;

    /// Execute a query with caching
    async fn execute_cached(&self, query: &Q, ttl_ms: Option<u64>) -> Result<R>;

    /// Validate a query
    fn validate_query(&self, query: &Q) -> Result<()>;
}

/// Command service trait for write operations
#[async_trait]
pub trait CommandService<C, R>: Service
where
    C: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    /// Execute a command
    async fn execute(&self, command: C) -> Result<R>;

    /// Validate a command
    fn validate_command(&self, command: &C) -> Result<()>;

    /// Check if command can be executed
    async fn can_execute(&self, command: &C) -> Result<bool>;
}

/// Event handling service
#[async_trait]
pub trait EventService: Service {
    /// Publish an event
    async fn publish<E>(&self, event: E) -> Result<()>
    where
        E: DomainEvent + Send + 'static;

    /// Subscribe to events
    async fn subscribe<E, H>(&mut self, handler: H) -> Result<()>
    where
        E: DomainEvent + Send + 'static,
        H: EventHandler<E> + Send + 'static;

    /// Unsubscribe from events
    async fn unsubscribe<E>(&mut self) -> Result<()>
    where
        E: DomainEvent + Send + 'static;
}

/// Caching service interface
#[async_trait]
pub trait CachingService: Service {
    /// Get a value from cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Set a value in cache
    async fn set<T>(&self, key: &str, value: &T, ttl_ms: Option<u64>) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Clear cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<CacheStats>;
}

/// Validation service interface
#[async_trait]
pub trait ValidationService<T>: Service {
    /// Validate an entity
    fn validate(&self, entity: &T) -> Result<ValidationResult>;

    /// Validate a field
    fn validate_field(&self, entity: &T, field: &str) -> Result<ValidationResult>;

    /// Get validation rules
    fn get_validation_rules(&self) -> Vec<ValidationRule>;
}

/// Audit service interface
#[async_trait]
pub trait AuditService: Service {
    /// Log an audit event
    async fn log(&self, event: AuditEvent) -> Result<()>;

    /// Query audit logs
    async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>>;

    /// Get audit statistics
    async fn get_stats(&self) -> Result<AuditStats>;
}

/// Notification service interface
#[async_trait]
pub trait NotificationService: Service {
    /// Send a notification
    async fn send(&self, notification: Notification) -> Result<()>;

    /// Send batch notifications
    async fn send_batch(&self, notifications: Vec<Notification>) -> Result<BatchResult>;

    /// Get notification status
    async fn get_status(&self, id: &str) -> Result<NotificationStatus>;
}

/// File service interface
#[async_trait]
pub trait FileService: Service {
    /// Upload a file
    async fn upload(&self, file: &FileUpload) -> Result<FileMetadata>;

    /// Download a file
    async fn download(&self, path: &str) -> Result<FileDownload>;

    /// Delete a file
    async fn delete(&self, path: &str) -> Result<bool>;

    /// List files
    async fn list(&self, path: &str) -> Result<Vec<FileMetadata>>;
}

/// Configuration service interface
#[async_trait]
pub trait ConfigurationService: Service {
    /// Get a configuration value
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Set a configuration value
    async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Delete a configuration value
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Reload configuration
    async fn reload(&mut self) -> Result<()>;
}

/// Use case trait
pub trait UseCase: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;

    /// Execute the use case
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;

    /// Validate input
    fn validate_input(&self, input: &Self::Input) -> Result<()>;

    /// Get use case metadata
    fn metadata(&self) -> &UseCaseMetadata;
}

/// Domain event trait
pub trait DomainEvent: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Get event ID
    fn event_id(&self) -> &str;

    /// Get event type
    fn event_type(&self) -> &str;

    /// Get event timestamp
    fn timestamp(&self) -> DateTime<Utc>;

    /// Get event version
    fn version(&self) -> &str;

    /// Get aggregate ID if applicable
    fn aggregate_id(&self) -> Option<&str>;
}

/// Event handler trait
pub trait EventHandler<E>: Send + Sync
where
    E: DomainEvent,
{
    /// Handle an event
    async fn handle(&mut self, event: &E) -> Result<()>;

    /// Get handler metadata
    fn metadata(&self) -> &HandlerMetadata;
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ServiceMetadata {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: String::new(),
            author: String::new(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            tags: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub message: String,
    pub last_check: DateTime<Utc>,
    pub uptime_ms: u64,
    pub details: HashMap<String, String>,
}

impl ServiceHealth {
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: "Service is healthy".to_string(),
            last_check: Utc::now(),
            uptime_ms: 0,
            details: HashMap::new(),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            last_check: Utc::now(),
            uptime_ms: 0,
            details: HashMap::new(),
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            last_check: Utc::now(),
            uptime_ms: 0,
            details: HashMap::new(),
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = details;
        self
    }

    pub fn with_uptime(mut self, uptime_ms: u64) -> Self {
        self.uptime_ms = uptime_ms;
        self
    }
}

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub average_response_time_ms: f64,
    pub last_request_time: Option<DateTime<Utc>>,
    pub error_rate: f64,
    pub uptime_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub custom_metrics: HashMap<String, f64>,
}

impl ServiceMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: 0,
            requests_success: 0,
            requests_failed: 0,
            average_response_time_ms: 0.0,
            last_request_time: None,
            error_rate: 0.0,
            uptime_ms: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            custom_metrics: HashMap::new(),
        }
    }

    pub fn record_request(&mut self, success: bool, response_time_ms: u64) {
        self.requests_total += 1;
        if success {
            self.requests_success += 1;
        } else {
            self.requests_failed += 1;
        }

        // Update average response time
        let total_time = self.average_response_time_ms * (self.requests_total - 1) as f64 + response_time_ms as f64;
        self.average_response_time_ms = total_time / self.requests_total as f64;

        // Update error rate
        self.error_rate = self.requests_failed as f64 / self.requests_total as f64;

        self.last_request_time = Some(Utc::now());
    }

    pub fn success_rate(&self) -> f64 {
        if self.requests_total > 0 {
            self.requests_success as f64 / self.requests_total as f64
        } else {
            0.0
        }
    }
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Support types for service layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParams {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub fields: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub filters: HashMap<String, String>,
}

/// Use case metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCaseMetadata {
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub output_type: String,
    pub version: String,
}

/// Handler metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerMetadata {
    pub name: String,
    pub event_types: Vec<String>,
    pub version: String,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub code: String,
    pub message: String,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: ValidationRuleType,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_validator: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    String,
    Number,
    Email,
    Phone,
    Url,
    Date,
    Boolean,
    Custom,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletions: u64,
    pub size: u64,
    pub hit_rate: f64,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub user_id: Option<String>,
    pub details: HashMap<String, String>,
}

/// Audit query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub user_id: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: u64,
    pub events_by_action: HashMap<String, u64>,
    pub events_by_resource: HashMap<String, u64>,
    pub events_by_user: HashMap<String, u64>,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    SMS,
    Push,
    InApp,
    Webhook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub errors: Vec<String>,
}

/// Notification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Cancelled,
}

/// File types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpload {
    pub name: String,
    pub content: Vec<u8>,
    pub content_type: String,
    pub size: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDownload {
    pub name: String,
    pub content: Vec<u8>,
    pub content_type: String,
    pub size: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}