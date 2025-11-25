//! Repository Traits
//!
//! Defines the core repository interfaces and base operations.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Base repository trait providing common CRUD operations
#[async_trait]
pub trait Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Create a new entity
    async fn create(&self, entity: &T) -> Result<ID>;

    /// Find an entity by its ID
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;

    /// Find all entities
    async fn find_all(&self) -> Result<Vec<T>>;

    /// Update an existing entity
    async fn update(&self, id: &ID, entity: &T) -> Result<T>;

    /// Delete an entity by its ID
    async fn delete(&self, id: &ID) -> Result<bool>;

    /// Check if an entity exists
    async fn exists(&self, id: &ID) -> Result<bool>;

    /// Count all entities
    async fn count(&self) -> Result<u64>;
}

/// Queryable repository for advanced filtering
#[async_trait]
pub trait QueryableRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Query parameters type
    type QueryParams: Clone + Send + Sync;

    /// Find entities matching query parameters
    async fn find_where(&self, params: &Self::QueryParams) -> Result<Vec<T>>;

    /// Find first entity matching query parameters
    async fn find_one_where(&self, params: &Self::QueryParams) -> Result<Option<T>>;

    /// Count entities matching query parameters
    async fn count_where(&self, params: &Self::QueryParams) -> Result<u64>;

    /// Check if any entity matches query parameters
    async fn exists_where(&self, params: &Self::QueryParams) -> Result<bool>;
}

/// Paginated repository for large datasets
#[async_trait]
pub trait PaginatedRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Pagination parameters
    type PageParams: Clone + Send + Sync;

    /// Paginated result
    type PageResult: Clone + Send + Sync;

    /// Find entities with pagination
    async fn find_page(&self, params: &Self::PageParams) -> Result<Self::PageResult>;

    /// Find entities with cursor-based pagination
    async fn find_cursor(&self, cursor: Option<String>, limit: u32) -> Result<CursorPage<T>>;
}

/// Batch operations repository
#[async_trait]
pub trait BatchRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Create multiple entities
    async fn create_batch(&self, entities: &[T]) -> Result<Vec<ID>>;

    /// Update multiple entities
    async fn update_batch(&self, updates: &[(ID, T)]) -> Result<Vec<T>>;

    /// Delete multiple entities
    async fn delete_batch(&self, ids: &[ID]) -> Result<u64>;

    /// Find entities by IDs
    async fn find_by_ids(&self, ids: &[ID]) -> Result<Vec<T>>;
}

/// Cached repository with built-in caching support
#[async_trait]
pub trait CachedRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Invalidate cache for specific entity
    async fn invalidate_cache(&self, id: &ID) -> Result<()>;

    /// Invalidate all cache
    async fn invalidate_all_cache(&self) -> Result<()>;

    /// Preload cache with entities
    async fn preload_cache(&self, ids: &[ID]) -> Result<()>;

    /// Get cache statistics
    async fn cache_stats(&self) -> Result<CacheStats>;
}

/// Transaction-aware repository
#[async_trait]
pub trait TransactionalRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Transaction type
    type Transaction;

    /// Begin a new transaction
    async fn begin_transaction(&self) -> Result<Self::Transaction>;

    /// Execute multiple operations in a transaction
    async fn execute_in_transaction<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&Self::Transaction) -> Box<dyn std::future::Future<Output = Result<R>> + Send + '_>,
        R: Send + 'static;
}

/// Auditable repository with change tracking
#[async_trait]
pub trait AuditableRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
{
    /// Audit log entry
    type AuditEntry: Clone + Send + Sync;

    /// Enable audit logging
    async fn enable_audit(&self) -> Result<()>;

    /// Disable audit logging
    async fn disable_audit(&self) -> Result<()>;

    /// Get audit history for an entity
    async fn get_audit_history(&self, id: &ID) -> Result<Vec<Self::AuditEntry>>;

    /// Get audit history for all entities
    async fn get_all_audit_history(&self) -> Result<Vec<Self::AuditEntry>>;
}

/// Repository factory for creating repository instances
pub trait RepositoryFactory: Send + Sync {
    /// Create a repository of the specified type
    fn create_repository<T, ID>(&self) -> Result<Box<dyn Repository<T, ID>>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
        ID: Send + Sync + Clone + PartialEq + 'static;

    /// Create a cached repository
    fn create_cached_repository<T, ID>(&self) -> Result<Box<dyn CachedRepository<T, ID>>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + 'static,
        ID: Send + Sync + Clone + PartialEq + 'static;
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub connection_string: String,
    pub pool_size: u32,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub cache_enabled: bool,
    pub audit_enabled: bool,
    pub transaction_isolation: TransactionIsolation,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            connection_string: "sqlite:./data/app.db".to_string(),
            pool_size: 10,
            timeout_ms: 30000,
            retry_attempts: 3,
            cache_enabled: true,
            audit_enabled: false,
            transaction_isolation: TransactionIsolation::ReadCommitted,
        }
    }
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionIsolation {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// Cursor-based pagination result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub total_count: Option<u64>,
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

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            sets: 0,
            deletions: 0,
            size: 0,
            hit_rate: 0.0,
        }
    }

    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Generic query builder
pub struct QueryBuilder<T, ID> {
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID> QueryBuilder<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T, ID> Default for QueryBuilder<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}