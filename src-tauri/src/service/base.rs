//! Base Service Implementation
//!
//! Provides the foundation implementation for service layer.

use crate::error::{AppError, Result};
use crate::service::traits::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Base service implementation
pub struct BaseService {
    metadata: ServiceMetadata,
    metrics: Arc<RwLock<ServiceMetrics>>,
    started_at: DateTime<Utc>,
    is_initialized: Arc<RwLock<bool>>,
}

impl BaseService {
    pub fn new(metadata: ServiceMetadata) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ServiceMetrics::new())),
            started_at: Utc::now(),
            is_initialized: Arc::new(RwLock::new(false)),
            metadata,
        }
    }

    /// Get service metadata
    pub fn metadata(&self) -> &ServiceMetadata {
        &self.metadata
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> ServiceMetrics {
        self.metrics.read().await.clone()
    }

    /// Record a request
    pub async fn record_request(&self, success: bool, response_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request(success, response_time_ms);
    }

    /// Update custom metric
    pub async fn update_custom_metric(&self, key: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.custom_metrics.insert(key.to_string(), value);
    }

    /// Get service uptime
    pub async fn uptime_ms(&self) -> u64 {
        let now = Utc::now();
        (now - self.started_at).num_milliseconds() as u64
    }

    /// Check if service is initialized
    pub async fn is_initialized(&self) -> bool {
        *self.is_initialized.read().await
    }

    /// Mark service as initialized
    async fn mark_initialized(&self) {
        let mut initialized = self.is_initialized.write().await;
        *initialized = true;
    }

    /// Execute a function with metrics tracking
    async fn execute_with_metrics<F, R, Fut>(&self, operation: &str, f: F) -> Result<R>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
    {
        let start_time = std::time::Instant::now();
        let result = f().await;
        let elapsed = start_time.elapsed();

        let success = result.is_ok();
        self.record_request(success, elapsed.as_millis() as u64).await;

        if let Err(ref error) = result {
            tracing::error!(
                "Service '{}' operation '{}' failed: {}",
                self.metadata.name,
                operation,
                error
            );
        } else {
            tracing::debug!(
                "Service '{}' operation '{}' completed in {}ms",
                self.metadata.name,
                operation,
                elapsed.as_millis()
            );
        }

        result
    }
}

#[async_trait]
impl Service for BaseService {
    fn metadata(&self) -> &ServiceMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<()> {
        self.execute_with_metrics("initialize", || async {
            if self.is_initialized().await {
                return Ok(());
            }

            tracing::info!("Initializing service: {}", self.metadata.name);

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.uptime_ms = 0;

            self.mark_initialized().await;

            tracing::info!("Service '{}' initialized successfully", self.metadata.name);
            Ok(())
        }).await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.execute_with_metrics("shutdown", || async {
            tracing::info!("Shutting down service: {}", self.metadata.name);

            // Mark as not initialized
            let mut initialized = self.is_initialized.write().await;
            *initialized = false;

            tracing::info!("Service '{}' shut down successfully", self.metadata.name);
            Ok(())
        }).await
    }

    async fn health_check(&self) -> Result<ServiceHealth> {
        self.execute_with_metrics("health_check", || async {
            let uptime = self.uptime_ms().await;
            let is_initialized = self.is_initialized().await;

            let health = if is_initialized {
                ServiceHealth::healthy()
                    .with_uptime(uptime)
                    .with_details({
                        let mut details = HashMap::new();
                        details.insert("version".to_string(), self.metadata.version.clone());
                        details.insert("initialized".to_string(), is_initialized.to_string());
                        details
                    })
            } else {
                ServiceHealth::unhealthy("Service not initialized")
                    .with_uptime(uptime)
            };

            Ok(health)
        }).await
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        let mut metrics = self.get_current_metrics().await;
        metrics.uptime_ms = self.uptime_ms().await;
        Ok(metrics)
    }
}

/// Domain service base implementation
pub struct BaseDomainService<T, ID, Repo> {
    base: BaseService,
    repository: Arc<Repo>,
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID, Repo> BaseDomainService<T, ID, Repo>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
    Repo: Send + Sync,
{
    pub fn new(metadata: ServiceMetadata, repository: Arc<Repo>) -> Self {
        Self {
            base: BaseService::new(metadata),
            repository,
            _phantom: PhantomData,
        }
    }

    pub fn repository(&self) -> &Arc<Repo> {
        &self.repository
    }

    pub async fn base(&self) -> &BaseService {
        &self.base
    }
}

#[async_trait]
impl<T, ID, Repo> Service for BaseDomainService<T, ID, Repo>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
    Repo: Send + Sync,
{
    fn metadata(&self) -> &ServiceMetadata {
        self.base.metadata()
    }

    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown().await
    }

    async fn health_check(&self) -> Result<ServiceHealth> {
        self.base.health_check().await
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        self.base.get_metrics().await
    }
}

/// Query service base implementation
pub struct BaseQueryService<Q, R, Handler> {
    base: BaseService,
    handler: Arc<Handler>,
    _phantom: PhantomData<(Q, R)>,
}

impl<Q, R, Handler> BaseQueryService<Q, R, Handler>
where
    Q: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    Handler: Send + Sync,
{
    pub fn new(metadata: ServiceMetadata, handler: Arc<Handler>) -> Self {
        Self {
            base: BaseService::new(metadata),
            handler,
            _phantom: PhantomData,
        }
    }

    pub fn handler(&self) -> &Arc<Handler> {
        &self.handler
    }

    pub async fn base(&self) -> &BaseService {
        &self.base
    }
}

#[async_trait]
impl<Q, R, Handler> Service for BaseQueryService<Q, R, Handler>
where
    Q: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    Handler: Send + Sync,
{
    fn metadata(&self) -> &ServiceMetadata {
        self.base.metadata()
    }

    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown().await
    }

    async fn health_check(&self) -> Result<ServiceHealth> {
        self.base.health_check().await
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        self.base.get_metrics().await
    }
}

/// Command service base implementation
pub struct BaseCommandService<C, R, Handler> {
    base: BaseService,
    handler: Arc<Handler>,
    _phantom: PhantomData<(C, R)>,
}

impl<C, R, Handler> BaseCommandService<C, R, Handler>
where
    C: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    Handler: Send + Sync,
{
    pub fn new(metadata: ServiceMetadata, handler: Arc<Handler>) -> Self {
        Self {
            base: BaseService::new(metadata),
            handler,
            _phantom: PhantomData,
        }
    }

    pub fn handler(&self) -> &Arc<Handler> {
        &self.handler
    }

    pub async fn base(&self) -> &BaseService {
        &self.base
    }
}

#[async_trait]
impl<C, R, Handler> Service for BaseCommandService<C, R, Handler>
where
    C: Send + Sync,
    R: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    Handler: Send + Sync,
{
    fn metadata(&self) -> &ServiceMetadata {
        self.base.metadata()
    }

    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown().await
    }

    async fn health_check(&self) -> Result<ServiceHealth> {
        self.base.health_check().await
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        self.base.get_metrics().await
    }
}

/// Application service base implementation
pub struct BaseApplicationService {
    base: BaseService,
    use_cases: HashMap<String, Box<dyn UseCaseHandle + Send + Sync>>,
}

impl BaseApplicationService {
    pub fn new(metadata: ServiceMetadata) -> Self {
        Self {
            base: BaseService::new(metadata),
            use_cases: HashMap::new(),
        }
    }

    pub fn register_use_case<UC>(&mut self, name: &str, use_case: UC)
    where
        UC: UseCase + 'static + Send + Sync,
    {
        self.use_cases.insert(name.to_string(), Box::new(use_case));
    }

    pub fn use_cases(&self) -> &HashMap<String, Box<dyn UseCaseHandle + Send + Sync>> {
        &self.use_cases
    }

    pub async fn base(&self) -> &BaseService {
        &self.base
    }
}

#[async_trait]
impl Service for BaseApplicationService {
    fn metadata(&self) -> &ServiceMetadata {
        self.base.metadata()
    }

    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.base.shutdown().await
    }

    async fn health_check(&self) -> Result<ServiceHealth> {
        self.base.health_check().await
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        let mut metrics = self.base.get_metrics().await?;
        metrics.custom_metrics.insert(
            "registered_use_cases".to_string(),
            self.use_cases.len() as f64,
        );
        Ok(metrics)
    }
}

#[async_trait]
impl ApplicationService for BaseApplicationService {
    async fn execute_use_case<UC>(&self, use_case: UC) -> Result<UC::Output>
    where
        UC: UseCase + Send + 'static,
    {
        self.base.execute_with_metrics("execute_use_case", || async {
            let input = use_case.metadata().name.clone();
            tracing::debug!("Executing use case: {}", input);

            let result = use_case.execute(use_case.metadata()).await;

            match &result {
                Ok(_) => tracing::debug!("Use case '{}' completed successfully", input),
                Err(e) => tracing::error!("Use case '{}' failed: {}", input, e),
            }

            result
        }).await
    }

    fn get_available_use_cases(&self) -> Vec<&'static str> {
        self.use_cases.keys().map(|k| k.as_str()).collect()
    }
}

/// Trait for handling use cases in the application service
#[async_trait]
pub trait UseCaseHandle: Send + Sync {
    async fn execute(&self, input: Box<dyn std::any::Any + Send>) -> Result<Box<dyn std::any::Any + Send>>;
    fn metadata(&self) -> &UseCaseMetadata;
}

/// Service factory for creating service instances
pub trait ServiceFactory: Send + Sync {
    /// Create a service by name
    fn create_service(&self, name: &str) -> Result<Box<dyn Service>>;

    /// Get available service types
    fn get_available_services(&self) -> Vec<&'static str>;

    /// Register a service creator
    fn register_service_creator<F>(&mut self, name: &str, creator: F) -> Result<()>
    where
        F: Fn() -> Result<Box<dyn Service>> + Send + Sync + 'static;
}

/// Base service factory implementation
pub struct BaseServiceFactory {
    creators: HashMap<String, Box<dyn Fn() -> Result<Box<dyn Service>> + Send + Sync>>,
}

impl BaseServiceFactory {
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
        }
    }

    pub fn register_service<F>(&mut self, name: &str, creator: F) -> Result<()>
    where
        F: Fn() -> Result<Box<dyn Service>> + Send + Sync + 'static,
    {
        self.creators.insert(name.to_string(), Box::new(creator));
        Ok(())
    }
}

impl ServiceFactory for BaseServiceFactory {
    fn create_service(&self, name: &str) -> Result<Box<dyn Service>> {
        self.creators
            .get(name)
            .ok_or_else(|| AppError::Service {
                message: format!("Service '{}' not found", name),
                service_name: name.to_string(),
            })?
            .call(())
    }

    fn get_available_services(&self) -> Vec<&'static str> {
        self.creators.keys().map(|k| k.as_str()).collect()
    }

    fn register_service_creator<F>(&mut self, name: &str, creator: F) -> Result<()>
    where
        F: Fn() -> Result<Box<dyn Service>> + Send + Sync + 'static,
    {
        self.register_service(name, creator)
    }
}

impl Default for BaseServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}