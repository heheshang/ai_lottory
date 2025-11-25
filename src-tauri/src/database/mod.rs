//! Database Module - Comprehensive Database Management
//!
//! This module provides secure database connection management, migrations,
//! connection pooling, and database health monitoring.

pub mod connection;
pub mod migrations;
pub mod health;
pub mod query_optimizer;

pub use health::HealthChecker;
pub use query_optimizer::QueryOptimizer;

use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite, Row};
use anyhow::Result;
use serde_json::Value;

/// Database configuration and initialization
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:./lottery.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        }
    }
}

impl DatabaseConfig {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }

    pub fn with_connection_pool(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Initialize database with all necessary components
    pub async fn initialize(&self) -> Result<SqlitePool> {
        println!("🔧 [DATABASE] Initializing database with URL: {}", self.database_url);

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&self.database_url).await? {
            println!("📝 [DATABASE] Creating database: {}", self.database_url);
            Sqlite::create_database(&self.database_url).await?;
        }

        // Create connection pool
        let pool = self.create_connection_pool().await?;

        // Run migrations
        migrations::run_migrations(&pool).await?;

        // Create health checker
        let _health_checker = HealthChecker::new(pool.clone());

        println!("✅ [DATABASE] Database initialization completed successfully");

        Ok(pool)
    }

    async fn create_connection_pool(&self) -> Result<SqlitePool> {
        println!("🔗 [DATABASE] Creating connection pool...");

        let mut pool_options = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime);

        // Enable WAL mode for better performance and concurrency
        pool_options = pool_options.after_connect(|conn, _meta| {
            Box::pin(async move {
                // Enable WAL mode
                sqlx::query("PRAGMA journal_mode=WAL")
                    .execute(&mut *conn)
                    .await?;

                // Enable foreign key constraints
                sqlx::query("PRAGMA foreign_keys=ON")
                    .execute(&mut *conn)
                    .await?;

                // Optimize performance
                sqlx::query("PRAGMA synchronous=NORMAL")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA cache_size=10000")
                    .execute(&mut *conn)
                    .await?;

                sqlx::query("PRAGMA temp_store=MEMORY")
                    .execute(&mut *conn)
                    .await?;

                Ok(())
            })
        });

        let pool = pool_options.connect(&self.database_url).await?;

        println!("✅ [DATABASE] Connection pool created successfully");
        println!("📊 [DATABASE] Pool config - Max connections: {}, Min connections: {}",
            self.max_connections, self.min_connections);

        Ok(pool)
    }
}

/// Database utilities and helper functions
pub struct DatabaseUtils;

impl DatabaseUtils {
    /// Get database size information
    pub async fn get_database_info(pool: &SqlitePool) -> Result<Value> {
        let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(pool)
            .await?;

        let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
            .fetch_one(pool)
            .await?;

        let size_bytes = page_size * page_count;
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_one(pool)
        .await?;

        let index_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_one(pool)
            .await?;

        Ok(serde_json::json!({
            "size_bytes": size_bytes,
            "size_mb": size_mb,
            "page_size": page_size,
            "page_count": page_count,
            "table_count": table_count,
            "index_count": index_count,
            "checked_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Validate database integrity
    pub async fn validate_integrity(pool: &SqlitePool) -> Result<bool> {
        println!("🔍 [DATABASE] Starting database integrity check...");

        let result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(pool)
            .await?;

        let is_valid = result == "ok";

        if is_valid {
            println!("✅ [DATABASE] Integrity check passed");
        } else {
            println!("❌ [DATABASE] Integrity check failed: {}", result);
        }

        Ok(is_valid)
    }

    /// Get table row counts
    pub async fn get_table_counts(pool: &SqlitePool) -> Result<Value> {
        let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
            .fetch_all(pool)
            .await?;

        let mut counts = serde_json::Map::new();

        for table in tables {
            let table_name: String = table.get("name");
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table_name))
                .fetch_one(pool)
                .await
                .unwrap_or(0);

            counts.insert(table_name, serde_json::Value::Number(count.into()));
        }

        Ok(serde_json::json!({
            "table_counts": counts,
            "checked_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}
