//! Lottery-specific Repository Implementations
//!
//! Provides repository implementations for lottery data models.

use crate::error::Result;
use crate::repository::traits::*;
use crate::repository::base::{BaseRepository, RepositoryBuilder};
use crate::repository::queries::{QueryParams, QueryBuilder, QueryCondition, QueryValue, SortSpec};
use crate::repository::cache::{CachedRepositoryWrapper, CacheConfig, MemoryCache};
use crate::repository::transactions::{BaseTransaction, TransactionManager};
use crate::models::{LotteryDraw, NumberFrequency, NumberStatistics};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use chrono::{DateTime, Utc, NaiveDate};

/// Lottery draw repository
pub struct LotteryDrawRepository {
    base: BaseRepository<LotteryDraw, i32, sqlx::Sqlite>,
    cached: Option<CachedRepositoryWrapper<LotteryDraw, i32, BaseRepository<LotteryDraw, i32, sqlx::Sqlite>>>,
}

impl LotteryDrawRepository {
    pub async fn new(pool: Arc<SqlitePool>, enable_cache: bool) -> Result<Self> {
        let base = BaseRepository::new(pool, "lottery_draws", "id")?;

        let cached = if enable_cache {
            let cache_config = CacheConfig {
                max_size_bytes: 50 * 1024 * 1024, // 50MB for lottery draws
                default_ttl_ms: 600000, // 10 minutes
                max_entries: Some(50000),
                ..Default::default()
            };

            Some(CachedRepositoryWrapper::new(
                base.clone(),
                cache_config,
                "lottery_draw",
            ))
        } else {
            None
        };

        Ok(Self { base, cached })
    }

    /// Find draws within a date range
    pub async fn find_by_date_range(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<LotteryDraw>> {
        let query = format!(
            "SELECT * FROM lottery_draws WHERE draw_date >= $1 AND draw_date <= $2 ORDER BY draw_date DESC"
        );

        let pool = self.base.pool();
        let rows = sqlx::query_as::<_, LotteryDraw>(&query)
            .bind(start_date)
            .bind(end_date)
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query lottery draws by date range: {}", e),
                query: query.clone(),
            })?;

        Ok(rows)
    }

    /// Find draws by specific numbers
    pub async fn find_by_numbers(&self, numbers: &[u32]) -> Result<Vec<LotteryDraw>> {
        if numbers.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = (1..=numbers.len())
            .map(|i| format!("${}", i + 1)) // +1 for the first parameter
            .collect();

        let query = format!(
            "SELECT * FROM lottery_draws WHERE winning_numbers && ARRAY[{}] ORDER BY draw_date DESC",
            placeholders.join(", ")
        );

        let pool = self.base.pool();
        let mut query_builder = sqlx::query_as::<_, LotteryDraw>(&query);

        for &number in numbers {
            query_builder = query_builder.bind(number as i32);
        }

        let rows = query_builder
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query lottery draws by numbers: {}", e),
                query: query.clone(),
            })?;

        Ok(rows)
    }

    /// Find draws by jackpot amount range
    pub async fn find_by_jackpot_range(&self, min_amount: f64, max_amount: Option<f64>) -> Result<Vec<LotteryDraw>> {
        let query = if let Some(max_amount) = max_amount {
            format!(
                "SELECT * FROM lottery_draws WHERE jackpot_amount >= $1 AND jackpot_amount <= $2 ORDER BY jackpot_amount DESC"
            )
        } else {
            format!(
                "SELECT * FROM lottery_draws WHERE jackpot_amount >= $1 ORDER BY jackpot_amount DESC"
            )
        };

        let pool = self.base.pool();
        let mut query_builder = sqlx::query_as::<_, LotteryDraw>(&query)
            .bind(min_amount);

        if let Some(max_amount) = max_amount {
            query_builder = query_builder.bind(max_amount);
        }

        let rows = query_builder
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query lottery draws by jackpot range: {}", e),
                query: query.clone(),
            })?;

        Ok(rows)
    }

    /// Get the latest draw
    pub async fn find_latest(&self) -> Result<Option<LotteryDraw>> {
        let query = "SELECT * FROM lottery_draws ORDER BY draw_date DESC LIMIT 1";

        let pool = self.base.pool();
        let result = sqlx::query_as::<_, LotteryDraw>(query)
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query latest lottery draw: {}", e),
                query: query.to_string(),
            })?;

        Ok(result)
    }

    /// Get the first draw
    pub async fn find_first(&self) -> Result<Option<LotteryDraw>> {
        let query = "SELECT * FROM lottery_draws ORDER BY draw_date ASC LIMIT 1";

        let pool = self.base.pool();
        let result = sqlx::query_as::<_, LotteryDraw>(query)
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query first lottery draw: {}", e),
                query: query.to_string(),
            })?;

        Ok(result)
    }

    /// Get draws count by year
    pub async fn count_by_year(&self, year: i32) -> Result<u64> {
        let query = "SELECT COUNT(*) FROM lottery_draws WHERE EXTRACT(YEAR FROM draw_date) = $1";

        let pool = self.base.pool();
        let row = sqlx::query(query)
            .bind(year)
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to count lottery draws by year: {}", e),
                query: query.to_string(),
            })?;

        let count: u64 = row.try_get(0)
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to get count from query result: {}", e),
                query: query.to_string(),
            })?;

        Ok(count)
    }

    /// Get statistics for analysis
    pub async fn get_statistics(&self) -> Result<LotteryStatistics> {
        let pool = self.base.pool();

        // Total draws
        let total_query = "SELECT COUNT(*) FROM lottery_draws";
        let total_row = sqlx::query(total_query)
            .fetch_one(pool.as_ref())
            .await?;
        let total_draws: i64 = total_row.try_get(0)?;

        // Date range
        let date_query = "SELECT MIN(draw_date), MAX(draw_date) FROM lottery_draws";
        let date_row = sqlx::query(date_query)
            .fetch_one(pool.as_ref())
            .await?;
        let min_date: Option<NaiveDate> = date_row.try_get(0)?;
        let max_date: Option<NaiveDate> = date_row.try_get(1)?;

        // Average jackpot
        let jackpot_query = "SELECT AVG(jackpot_amount) FROM lottery_draws WHERE jackpot_amount IS NOT NULL";
        let jackpot_row = sqlx::query(jackpot_query)
            .fetch_optional(pool.as_ref())
            .await?;
        let avg_jackpot: Option<f64> = jackpot_row.and_then(|row| row.try_get(0).ok());

        Ok(LotteryStatistics {
            total_draws: total_draws as u64,
            date_range: DateRange { start: min_date, end: max_date },
            average_jackpot: avg_jackpot,
        })
    }

    /// Import multiple draws
    pub async fn import_draws(&self, draws: &[LotteryDraw]) -> Result<u64> {
        if draws.is_empty() {
            return Ok(0);
        }

        let pool = self.base.pool();
        let mut imported_count = 0;

        // Begin transaction for batch import
        let mut tx = pool.begin().await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to begin transaction for import: {}", e),
                query: "BEGIN".to_string(),
            })?;

        for draw in draws {
            let query = r#"
                INSERT INTO lottery_draws (
                    draw_date, winning_numbers, bonus_number, jackpot_amount, created_at
                ) VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (draw_date) DO NOTHING
            "#;

            let result = sqlx::query(query)
                .bind(draw.draw_date)
                .bind(&draw.winning_numbers)
                .bind(draw.bonus_number)
                .bind(draw.jackpot_amount)
                .bind(draw.created_at)
                .execute(&mut *tx)
                .await
                .map_err(|e| crate::error::AppError::Database {
                    message: format!("Failed to insert lottery draw: {}", e),
                    query: query.to_string(),
                })?;

            imported_count += result.rows_affected();
        }

        tx.commit().await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to commit import transaction: {}", e),
                query: "COMMIT".to_string(),
            })?;

        Ok(imported_count)
    }
}

#[async_trait]
impl Repository<LotteryDraw, i32> for LotteryDrawRepository {
    async fn create(&self, entity: &LotteryDraw) -> Result<i32> {
        if let Some(ref cached) = self.cached {
            cached.create(entity).await
        } else {
            self.base.create(entity).await
        }
    }

    async fn find_by_id(&self, id: &i32) -> Result<Option<LotteryDraw>> {
        if let Some(ref cached) = self.cached {
            cached.find_by_id(id).await
        } else {
            self.base.find_by_id(id).await
        }
    }

    async fn find_all(&self) -> Result<Vec<LotteryDraw>> {
        if let Some(ref cached) = self.cached {
            cached.find_all().await
        } else {
            self.base.find_all().await
        }
    }

    async fn update(&self, id: &i32, entity: &LotteryDraw) -> Result<LotteryDraw> {
        if let Some(ref cached) = self.cached {
            cached.update(id, entity).await
        } else {
            self.base.update(id, entity).await
        }
    }

    async fn delete(&self, id: &i32) -> Result<bool> {
        if let Some(ref cached) = self.cached {
            cached.delete(id).await
        } else {
            self.base.delete(id).await
        }
    }

    async fn exists(&self, id: &i32) -> Result<bool> {
        if let Some(ref cached) = self.cached {
            cached.exists(id).await
        } else {
            self.base.exists(id).await
        }
    }

    async fn count(&self) -> Result<u64> {
        if let Some(ref cached) = self.cached {
            cached.count().await
        } else {
            self.base.count().await
        }
    }
}

/// Number frequency repository for analysis results
pub struct NumberFrequencyRepository {
    base: BaseRepository<NumberFrequency, i32, sqlx::Sqlite>,
}

impl NumberFrequencyRepository {
    pub async fn new(pool: Arc<SqlitePool>) -> Result<Self> {
        let base = BaseRepository::new(pool, "number_frequencies", "id")?;
        Ok(Self { base })
    }

    /// Find frequencies by number
    pub async fn find_by_number(&self, number: u32) -> Result<Option<NumberFrequency>> {
        let query = "SELECT * FROM number_frequencies WHERE number = $1";

        let pool = self.base.pool();
        let result = sqlx::query_as::<_, NumberFrequency>(query)
            .bind(number as i32)
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query number frequency: {}", e),
                query: query.to_string(),
            })?;

        Ok(result)
    }

    /// Get top N hot numbers
    pub async fn get_hot_numbers(&self, limit: u32) -> Result<Vec<NumberFrequency>> {
        let query = "SELECT * FROM number_frequencies ORDER BY frequency DESC, hot_score DESC LIMIT $1";

        let pool = self.base.pool();
        let rows = sqlx::query_as::<_, NumberFrequency>(query)
            .bind(limit as i64)
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query hot numbers: {}", e),
                query: query.to_string(),
            })?;

        Ok(rows)
    }

    /// Get top N cold numbers
    pub async fn get_cold_numbers(&self, limit: u32) -> Result<Vec<NumberFrequency>> {
        let query = "SELECT * FROM number_frequencies ORDER BY frequency ASC, hot_score ASC LIMIT $1";

        let pool = self.base.pool();
        let rows = sqlx::query_as::<_, NumberFrequency>(query)
            .bind(limit as i64)
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to query cold numbers: {}", e),
                query: query.to_string(),
            })?;

        Ok(rows)
    }

    /// Update frequency calculations
    pub async fn update_frequencies(&self, days: Option<u32>) -> Result<u64> {
        let pool = self.base.pool();

        // This would typically involve a complex SQL query to recalculate frequencies
        // For now, we'll return a placeholder result
        let query = if let Some(days) = days {
            format!("UPDATE number_frequencies SET frequency = frequency + 1 WHERE last_drawn >= DATE('now', '-{} days')", days)
        } else {
            "UPDATE number_frequencies SET frequency = frequency + 1".to_string()
        };

        let result = sqlx::query(&query)
            .execute(pool.as_ref())
            .await
            .map_err(|e| crate::error::AppError::Database {
                message: format!("Failed to update number frequencies: {}", e),
                query: query.clone(),
            })?;

        Ok(result.rows_affected())
    }
}

#[async_trait]
impl Repository<NumberFrequency, i32> for NumberFrequencyRepository {
    async fn create(&self, entity: &NumberFrequency) -> Result<i32> {
        self.base.create(entity).await
    }

    async fn find_by_id(&self, id: &i32) -> Result<Option<NumberFrequency>> {
        self.base.find_by_id(id).await
    }

    async fn find_all(&self) -> Result<Vec<NumberFrequency>> {
        self.base.find_all().await
    }

    async fn update(&self, id: &i32, entity: &NumberFrequency) -> Result<NumberFrequency> {
        self.base.update(id, entity).await
    }

    async fn delete(&self, id: &i32) -> Result<bool> {
        self.base.delete(id).await
    }

    async fn exists(&self, id: &i32) -> Result<bool> {
        self.base.exists(id).await
    }

    async fn count(&self) -> Result<u64> {
        self.base.count().await
    }
}

/// Support types for lottery repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryStatistics {
    pub total_draws: u64,
    pub date_range: DateRange,
    pub average_jackpot: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
}

/// Repository factory for lottery domain
pub struct LotteryRepositoryFactory {
    pool: Arc<SqlitePool>,
    enable_cache: bool,
}

impl LotteryRepositoryFactory {
    pub fn new(pool: Arc<SqlitePool>, enable_cache: bool) -> Self {
        Self { pool, enable_cache }
    }

    pub async fn create_lottery_draw_repository(&self) -> Result<LotteryDrawRepository> {
        LotteryDrawRepository::new(self.pool.clone(), self.enable_cache).await
    }

    pub async fn create_number_frequency_repository(&self) -> Result<NumberFrequencyRepository> {
        NumberFrequencyRepository::new(self.pool.clone()).await
    }
}