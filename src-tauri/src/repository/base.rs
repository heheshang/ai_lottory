//! Base Repository Implementation
//!
//! Provides the foundation implementation for repository pattern.

use crate::error::{AppError, Result};
use crate::repository::traits::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Pool, Row};
use std::marker::PhantomData;
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Base repository implementation using SQLx
pub struct BaseRepository<T, ID, DB>
where
    DB: Database,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de> + sqlx::FromRow<'static, DB::Row>,
    ID: Send + Sync + Clone + PartialEq + sqlx::Type<DB>,
{
    pool: Arc<Pool<DB>>,
    table_name: String,
    id_column: String,
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID, DB> BaseRepository<T, ID, DB>
where
    DB: Database,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de> + sqlx::FromRow<'static, DB::Row>,
    ID: Send + Sync + Clone + PartialEq + sqlx::Type<DB>,
{
    /// Create a new repository
    pub fn new(
        pool: Arc<Pool<DB>>,
        table_name: &str,
        id_column: &str,
    ) -> Result<Self> {
        Ok(Self {
            pool,
            table_name: table_name.to_string(),
            id_column: id_column.to_string(),
            _phantom: PhantomData,
        })
    }

    /// Get the database pool
    pub fn pool(&self) -> &Arc<Pool<DB>> {
        &self.pool
    }

    /// Get the table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Get the ID column name
    pub fn id_column(&self) -> &str {
        &self.id_column
    }

    /// Build a select query with optional conditions
    fn build_select_query(&self, where_clause: Option<&str>) -> String {
        let query = format!("SELECT * FROM {}", self.table_name);
        if let Some(where_clause) = where_clause {
            format!("{} WHERE {}", query, where_clause)
        } else {
            query
        }
    }

    /// Build an insert query
    fn build_insert_query(&self, columns: &[&str]) -> String {
        let placeholders: Vec<String> = (1..=columns.len())
            .map(|i| format!("${}", i))
            .collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            self.table_name,
            columns.join(", "),
            placeholders.join(", "),
            self.id_column
        )
    }

    /// Build an update query
    fn build_update_query(&self, columns: &[&str]) -> String {
        let set_clauses: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col, i + 2)) // +2 for id and return value
            .collect();
        format!(
            "UPDATE {} SET {} WHERE {} = $1 RETURNING *",
            self.table_name,
            set_clauses.join(", "),
            self.id_column
        )
    }

    /// Execute query and map to entity
    async fn query_as_entity(&self, query: &str) -> Result<Vec<T>> {
        let rows = sqlx::query_as::<_, T>(query)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AppError::Database {
                message: format!("Query failed: {}", e),
                query: query.to_string(),
            })?;
        Ok(rows)
    }

    /// Execute query and return first entity
    async fn query_first_as_entity(&self, query: &str) -> Result<Option<T>> {
        let entity = sqlx::query_as::<_, T>(query)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|e| AppError::Database {
                message: format!("Query failed: {}", e),
                query: query.to_string(),
            })?;
        Ok(entity)
    }

    /// Execute count query
    async fn query_count(&self, query: &str) -> Result<u64> {
        let row = sqlx::query(query)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| AppError::Database {
                message: format!("Count query failed: {}", e),
                query: query.to_string(),
            })?;

        let count: u64 = row.try_get(0)
            .map_err(|e| AppError::Database {
                message: format!("Failed to get count: {}", e),
                query: query.to_string(),
            })?;
        Ok(count)
    }
}

#[async_trait]
impl<T, ID, DB> Repository<T, ID> for BaseRepository<T, ID, DB>
where
    DB: Database,
    T: Send + Sync + Serialize + for<'de> Deserialize<'de> + sqlx::FromRow<'static, DB::Row>,
    ID: Send + Sync + Clone + PartialEq + sqlx::Type<DB> + 'static,
{
    async fn create(&self, entity: &T) -> Result<ID> {
        // This is a simplified implementation
        // In practice, you'd need to serialize the entity and build the query dynamically
        // For now, we'll return an error to indicate this needs specific implementation
        Err(AppError::NotImplemented {
            message: "Create operation requires specific implementation for each entity type".to_string(),
        })
    }

    async fn find_by_id(&self, id: &ID) -> Result<Option<T>> {
        let query = format!("SELECT * FROM {} WHERE {} = $1", self.table_name, self.id_column);
        self.query_first_as_entity(&query).await
    }

    async fn find_all(&self) -> Result<Vec<T>> {
        let query = self.build_select_query(None);
        self.query_as_entity(&query).await
    }

    async fn update(&self, id: &ID, entity: &T) -> Result<T> {
        // Similar to create, this needs specific implementation
        Err(AppError::NotImplemented {
            message: "Update operation requires specific implementation for each entity type".to_string(),
        })
    }

    async fn delete(&self, id: &ID) -> Result<bool> {
        let query = format!("DELETE FROM {} WHERE {} = $1", self.table_name, self.id_column);
        let result = sqlx::query(&query)
            .bind(id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AppError::Database {
                message: format!("Delete failed: {}", e),
                query,
            })?;
        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &ID) -> Result<bool> {
        let query = format!(
            "SELECT COUNT(*) FROM {} WHERE {} = $1",
            self.table_name, self.id_column
        );
        let count = self.query_count(&query).await?;
        Ok(count > 0)
    }

    async fn count(&self) -> Result<u64> {
        let query = format!("SELECT COUNT(*) FROM {}", self.table_name);
        self.query_count(&query).await
    }
}

/// Repository factory implementation
pub struct BaseRepositoryFactory {
    config: RepositoryConfig,
}

impl BaseRepositoryFactory {
    pub fn new(config: RepositoryConfig) -> Self {
        Self { config }
    }

    /// Create a repository with custom table and ID column
    pub fn create_repository_with_config<T, ID, DB>(
        &self,
        table_name: &str,
        id_column: &str,
    ) -> Result<BaseRepository<T, ID, DB>>
    where
        DB: Database,
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + sqlx::FromRow<'static, DB::Row> + 'static,
        ID: Send + Sync + Clone + PartialEq + sqlx::Type<DB> + 'static,
    {
        // Note: This would need actual database connection in real implementation
        // For now, return a placeholder error
        Err(AppError::NotImplemented {
            message: "Database connection setup required".to_string(),
        })
    }
}

impl RepositoryFactory for BaseRepositoryFactory {
    fn create_repository<T, ID>(&self) -> Result<Box<dyn Repository<T, ID>>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
        ID: Send + Sync + Clone + PartialEq + 'static,
    {
        // This is a generic implementation that would need to be specialized
        // for each entity type and database
        Err(AppError::NotImplemented {
            message: "Generic repository creation requires specific type configuration".to_string(),
        })
    }

    fn create_cached_repository<T, ID>(&self) -> Result<Box<dyn CachedRepository<T, ID>>>
    where
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + 'static,
        ID: Send + Sync + Clone + PartialEq + 'static,
    {
        Err(AppError::NotImplemented {
            message: "Cached repository creation requires specific type configuration".to_string(),
        })
    }
}

/// Repository builder for fluent API
pub struct RepositoryBuilder<T, ID> {
    config: RepositoryConfig,
    table_name: Option<String>,
    id_column: Option<String>,
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID> RepositoryBuilder<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    pub fn new() -> Self {
        Self {
            config: RepositoryConfig::default(),
            table_name: None,
            id_column: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_config(mut self, config: RepositoryConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_table(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn with_id_column(mut self, id_column: &str) -> Self {
        self.id_column = Some(id_column.to_string());
        self
    }

    pub fn build<DB>(self) -> Result<BaseRepository<T, ID, DB>>
    where
        DB: Database,
        T: Send + Sync + Serialize + for<'de> Deserialize<'de> + sqlx::FromRow<'static, DB::Row> + 'static,
        ID: Send + Sync + Clone + PartialEq + sqlx::Type<DB> + 'static,
    {
        let table_name = self.table_name
            .ok_or_else(|| AppError::Validation {
                message: "Table name is required".to_string(),
                field: "table_name".to_string(),
            })?;

        let id_column = self.id_column
            .ok_or_else(|| AppError::Validation {
                message: "ID column is required".to_string(),
                field: "id_column".to_string(),
            })?;

        // Note: Would need actual database pool in real implementation
        Err(AppError::NotImplemented {
            message: "Database connection setup required".to_string(),
        })
    }
}

impl<T, ID> Default for RepositoryBuilder<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Repository metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetrics {
    pub queries_executed: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
}

impl RepositoryMetrics {
    pub fn new() -> Self {
        Self {
            queries_executed: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            transactions_committed: 0,
            transactions_rolled_back: 0,
        }
    }

    pub fn record_query(&mut self, execution_time_ms: u64) {
        self.queries_executed += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.queries_executed as f64;
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn record_transaction_commit(&mut self) {
        self.transactions_committed += 1;
    }

    pub fn record_transaction_rollback(&mut self) {
        self.transactions_rolled_back += 1;
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

impl Default for RepositoryMetrics {
    fn default() -> Self {
        Self::new()
    }
}