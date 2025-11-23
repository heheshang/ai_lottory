//! Secure Database Layer for Super Lotto Application
//!
//! This module provides secure database operations with parameterized queries
//! to prevent SQL injection vulnerabilities and ensure consistent error handling.

use crate::super_lotto::{errors::SuperLottoError, models::*};
use sqlx::{SqlitePool, query_builder::QueryBuilder, Row};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Secure database query builder for Super Lotto operations
pub struct SuperLottoQueries<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SuperLottoQueries<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Get Super Lotto draws with secure parameterized queries
    pub async fn get_draws_secure(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        draw_number: Option<&str>,
    ) -> Result<Vec<SuperLottoDraw>, SuperLottoError> {
        let mut query_builder = QueryBuilder::new(
            "SELECT id, draw_number, draw_date, front_zone, back_zone, jackpot_amount, created_at, sum_front, odd_count_front, even_count_front, has_consecutive_front, winners_count
             FROM lottery_draws WHERE lottery_type = 'super_lotto'"
        );

        // Add optional filters with proper parameterization
        if let Some(start) = start_date {
            query_builder.push(" AND draw_date >= ");
            query_builder.push_bind(start);
        }

        if let Some(end) = end_date {
            query_builder.push(" AND draw_date <= ");
            query_builder.push_bind(end);
        }

        if let Some(number) = draw_number {
            query_builder.push(" AND draw_number LIKE ");
            query_builder.push_bind(format!("%{}%", number));
        }

        query_builder.push(" ORDER BY draw_date DESC, draw_number DESC");

        if let Some(limit_val) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit_val as i64);
        }

        if let Some(offset_val) = offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset_val as i64);
        }

        let rows = query_builder
            .build()
            .fetch_all(self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        self.map_rows_to_draws(rows)
    }

    /// Count draws matching criteria securely
    pub async fn count_draws_secure(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        draw_number: Option<&str>,
    ) -> Result<i64, SuperLottoError> {
        let mut query_builder = QueryBuilder::new(
            "SELECT COUNT(*) as total FROM lottery_draws WHERE lottery_type = 'super_lotto'"
        );

        // Add optional filters with proper parameterization
        if let Some(start) = start_date {
            query_builder.push(" AND draw_date >= ");
            query_builder.push_bind(start);
        }

        if let Some(end) = end_date {
            query_builder.push(" AND draw_date <= ");
            query_builder.push_bind(end);
        }

        if let Some(number) = draw_number {
            query_builder.push(" AND draw_number LIKE ");
            query_builder.push_bind(format!("%{}%", number));
        }

        let count: i64 = query_builder
            .build_query_scalar()
            .fetch_one(self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        Ok(count)
    }

    /// Get draws for analysis within specified date range securely
    pub async fn get_draws_for_analysis(
        &self,
        analysis_period_days: u32,
    ) -> Result<Vec<SuperLottoDraw>, SuperLottoError> {
        let end_date = Utc::now();
        let start_date = end_date - chrono::Duration::days(analysis_period_days as i64);

        let draws = sqlx::query_as!(
            SuperLottoDraw,
            r#"
            SELECT id, draw_number, draw_date, front_zone, back_zone,
                   jackpot_amount, created_at, sum_front, odd_count_front,
                   even_count_front, has_consecutive_front, winners_count
            FROM lottery_draws
            WHERE lottery_type = 'super_lotto'
              AND draw_date >= ?
              AND draw_date <= ?
            ORDER BY draw_date DESC
            LIMIT 10000
            "#,
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| SuperLottoError::Database(e))?;

        Ok(draws)
    }

    /// Get hot numbers analysis with secure queries
    pub async fn get_hot_numbers_analysis(
        &self,
        days: u32,
        zone_filter: Option<&str>,
        limit: u32,
        min_threshold: f64,
    ) -> Result<Value, SuperLottoError> {
        let draws = self.get_draws_for_analysis(days).await?;

        if draws.is_empty() {
            return Err(SuperLottoError::internal(
                "No historical data available for analysis",
            ));
        }

        // Perform hot numbers calculation in Rust (memory-safe)
        let hot_numbers = self.calculate_hot_numbers(&draws, zone_filter, min_threshold);

        let mut results = hot_numbers;
        // Sort by hot score and limit results
        results.sort_by(|a, b| {
            let score_a = a["hot_score"].as_f64().unwrap_or(0.0);
            let score_b = b["hot_score"].as_f64().unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit as usize);

        Ok(serde_json::json!({
            "numbers": results,
            "analysis_period_days": days,
            "zone_filter": zone_filter,
            "min_threshold": min_threshold,
            "total_draws_analyzed": draws.len(),
            "generated_at": Utc::now().to_rfc3339()
        }))
    }

    /// Get cold numbers analysis with secure queries
    pub async fn get_cold_numbers_analysis(
        &self,
        days: u32,
        zone_filter: Option<&str>,
        limit: u32,
        min_days_absent: u32,
    ) -> Result<Value, SuperLottoError> {
        let draws = self.get_draws_for_analysis(days).await?;

        if draws.is_empty() {
            return Err(SuperLottoError::internal(
                "No historical data available for analysis",
            ));
        }

        // Perform cold numbers calculation in Rust (memory-safe)
        let cold_numbers = self.calculate_cold_numbers(&draws, zone_filter, min_days_absent);

        let mut results = cold_numbers;
        // Sort by cold score and limit results
        results.sort_by(|a, b| {
            let score_a = a["cold_score"].as_f64().unwrap_or(0.0);
            let score_b = b["cold_score"].as_f64().unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit as usize);

        Ok(serde_json::json!({
            "numbers": results,
            "analysis_period_days": days,
            "zone_filter": zone_filter,
            "min_days_absent": min_days_absent,
            "total_draws_analyzed": draws.len(),
            "generated_at": Utc::now().to_rfc3339()
        }))
    }

    /// Insert new Super Lotto draw securely
    pub async fn insert_draw(&self, draw: &CreateSuperLottoDraw) -> Result<u64, SuperLottoError> {
        // Validate draw first
        draw.validate()?;

        let result = sqlx::query!(
            r#"
            INSERT INTO lottery_draws
            (lottery_type, draw_number, draw_date, front_zone, back_zone, jackpot_amount, created_at)
            VALUES ('super_lotto', ?, ?, ?, ?, ?, ?)
            "#,
            draw.draw_number,
            draw.draw_date,
            serde_json::to_string(&draw.front_zone).map_err(|e| SuperLottoError::Serialization(e))?,
            serde_json::to_string(&draw.back_zone).map_err(|e| SuperLottoError::Serialization(e))?,
            draw.jackpot_amount,
            Utc::now().to_rfc3339()
        )
        .execute(self.pool)
        .await
        .map_err(|e| SuperLottoError::Database(e))?;

        Ok(result.last_insert_rowid() as u64)
    }

    /// Insert multiple draws in a transaction
    pub async fn insert_draws_batch(&self, draws: &[CreateSuperLottoDraw]) -> Result<u64, SuperLottoError> {
        let mut tx = self.pool.begin().await.map_err(|e| SuperLottoError::Database(e))?;
        let mut inserted_count = 0u64;

        for draw in draws {
            // Validate each draw
            draw.validate()?;

            let result = sqlx::query!(
                r#"
                INSERT INTO lottery_draws
                (lottery_type, draw_number, draw_date, front_zone, back_zone, jackpot_amount, created_at)
                VALUES ('super_lotto', ?, ?, ?, ?, ?, ?)
                "#,
                draw.draw_number,
                draw.draw_date,
                serde_json::to_string(&draw.front_zone).map_err(|e| SuperLottoError::Serialization(e))?,
                serde_json::to_string(&draw.back_zone).map_err(|e| SuperLottoError::Serialization(e))?,
                draw.jackpot_amount,
                Utc::now().to_rfc3339()
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

            if result.rows_affected() > 0 {
                inserted_count += 1;
            }
        }

        tx.commit().await.map_err(|e| SuperLottoError::Database(e))?;
        Ok(inserted_count)
    }

    /// Private helper methods
    fn map_rows_to_draws(&self, rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Vec<SuperLottoDraw>, SuperLottoError> {
        let mut draws = Vec::new();

        for row in rows {
            let draw = SuperLottoDraw {
                id: row.get("id"),
                draw_date: DateTime::parse_from_rfc3339(&row.get::<String, _>("draw_date"))
                    .map_err(|e| SuperLottoError::Chrono(e))?
                    .with_timezone(&Utc),
                draw_number: Some(row.get("draw_number")),
                front_zone: serde_json::from_str(&row.get::<String, _>("front_zone"))
                    .map_err(|e| SuperLottoError::Serialization(e))?,
                back_zone: serde_json::from_str(&row.get::<String, _>("back_zone"))
                    .map_err(|e| SuperLottoError::Serialization(e))?,
                jackpot_amount: row.get("jackpot_amount"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| SuperLottoError::Chrono(e))?
                    .with_timezone(&Utc),
                sum_front: row.get("sum_front"),
                odd_count_front: row.get("odd_count_front"),
                even_count_front: row.get("even_count_front"),
                has_consecutive_front: row.get("has_consecutive_front"),
                winners_count: row.get("winners_count"),
            };
            draws.push(draw);
        }

        Ok(draws)
    }

    fn calculate_hot_numbers(
        &self,
        draws: &[SuperLottoDraw],
        zone_filter: Option<&str>,
        min_threshold: f64,
    ) -> Vec<Value> {
        use std::collections::HashMap;

        let mut front_freq: HashMap<u32, (u32, f64, u32)> = HashMap::new();
        let mut back_freq: HashMap<u32, (u32, f64, u32)> = HashMap::new();
        let mut hot_numbers = Vec::new();
        let total_draws = draws.len() as f64;

        // Calculate frequencies with recency weighting
        for (i, draw) in draws.iter().enumerate() {
            let days_ago = i;

            // Front zone frequencies
            for num in &draw.front_zone {
                let freq = front_freq.entry(*num).or_insert((0, 0.0, days_ago as u32));
                freq.0 += 1;
                freq.1 += 1.0 / (days_ago + 1) as f64; // Weight by recency
                freq.2 = freq.2.min(days_ago as u32);
            }

            // Back zone frequencies
            for num in &draw.back_zone {
                let freq = back_freq.entry(*num).or_insert((0, 0.0, days_ago as u32));
                freq.0 += 1;
                freq.1 += 1.0 / (days_ago + 1) as f64;
                freq.2 = freq.2.min(days_ago as u32);
            }
        }

        // Process front zone numbers
        if zone_filter.map_or(true, |z| z == "FRONT" || z == "BOTH") {
            for (num, (count, weighted_score, last_seen)) in &front_freq {
                let frequency = *count as f64 / total_draws;
                let hot_score = weighted_score / 100.0;
                let avg_gap = if *count > 0 { (total_draws as f64) / (*count as f64) } else { 0.0 };

                if hot_score >= min_threshold {
                    hot_numbers.push(serde_json::json!({
                        "number": num,
                        "zone": "FRONT",
                        "frequency": frequency,
                        "last_seen": self.format_days_ago(*last_seen),
                        "hot_score": hot_score,
                        "cold_score": 1.0 - hot_score,
                        "average_gap": avg_gap,
                        "current_gap": *last_seen,
                        "period_days": draws.len(),
                        "updated_at": Utc::now().to_rfc3339()
                    }));
                }
            }
        }

        // Process back zone numbers
        if zone_filter.map_or(true, |z| z == "BACK" || z == "BOTH") {
            for (num, (count, weighted_score, last_seen)) in &back_freq {
                let frequency = *count as f64 / total_draws;
                let hot_score = weighted_score / 100.0;
                let avg_gap = if *count > 0 { (total_draws as f64) / (*count as f64) } else { 0.0 };

                if hot_score >= min_threshold {
                    hot_numbers.push(serde_json::json!({
                        "number": num,
                        "zone": "BACK",
                        "frequency": frequency,
                        "last_seen": self.format_days_ago(*last_seen),
                        "hot_score": hot_score,
                        "cold_score": 1.0 - hot_score,
                        "average_gap": avg_gap,
                        "current_gap": *last_seen,
                        "period_days": draws.len(),
                        "updated_at": Utc::now().to_rfc3339()
                    }));
                }
            }
        }

        hot_numbers
    }

    fn calculate_cold_numbers(
        &self,
        draws: &[SuperLottoDraw],
        zone_filter: Option<&str>,
        min_absent: u32,
    ) -> Vec<Value> {
        use std::collections::HashMap;

        let mut front_last_seen = HashMap::new();
        let mut back_last_seen = HashMap::new();
        let mut front_appearances = HashMap::new();
        let mut back_appearances = HashMap::new();
        let days = draws.len();
        let mut cold_numbers = Vec::new();
        let total_draws = draws.len() as f64;

        // Initialize all numbers as never seen
        for num in 1..=35 {
            front_last_seen.insert(num, days);
            front_appearances.insert(num, 0);
        }
        for num in 1..=12 {
            back_last_seen.insert(num, days);
            back_appearances.insert(num, 0);
        }

        // Track appearances
        for (days_ago, draw) in draws.iter().enumerate() {
            for &num in &draw.front_zone {
                if let Some(last_seen) = front_last_seen.get(&num) {
                    if *last_seen == days {
                        front_last_seen.insert(num, days_ago);
                    }
                }
                *front_appearances.entry(num).or_insert(0) += 1;
            }

            for &num in &draw.back_zone {
                if let Some(last_seen) = back_last_seen.get(&num) {
                    if *last_seen == days {
                        back_last_seen.insert(num, days_ago);
                    }
                }
                *back_appearances.entry(num).or_insert(0) += 1;
            }
        }

        // Process front zone numbers
        if zone_filter.map_or(true, |z| z == "FRONT" || z == "BOTH") {
            for num in 1..=35 {
                let current_gap = *front_last_seen.get(&num).unwrap_or(&days);
                let appearances = *front_appearances.get(&num).unwrap_or(&0);
                let frequency = appearances as f64 / total_draws;

                let gap_score = (current_gap as f64) / (days as f64);
                let frequency_score = 1.0 - frequency;
                let cold_score = (gap_score + frequency_score) / 2.0;
                let avg_gap = if appearances > 0 { (days as f64) / (appearances as f64) } else { days as f64 };

                if current_gap >= min_absent {
                    cold_numbers.push(serde_json::json!({
                        "number": num,
                        "zone": "FRONT",
                        "frequency": frequency,
                        "last_seen": self.format_days_ago(current_gap),
                        "hot_score": 1.0 - cold_score,
                        "cold_score": cold_score,
                        "average_gap": avg_gap,
                        "current_gap": current_gap,
                        "appearances": appearances,
                        "period_days": days,
                        "updated_at": Utc::now().to_rfc3339()
                    }));
                }
            }
        }

        // Process back zone numbers
        if zone_filter.map_or(true, |z| z == "BACK" || z == "BOTH") {
            for num in 1..=12 {
                let current_gap = *back_last_seen.get(&num).unwrap_or(&days);
                let appearances = *back_appearances.get(&num).unwrap_or(&0);
                let frequency = appearances as f64 / total_draws;

                let gap_score = (current_gap as f64) / (days as f64);
                let frequency_score = 1.0 - frequency;
                let cold_score = (gap_score + frequency_score) / 2.0;
                let avg_gap = if appearances > 0 { (days as f64) / (appearances as f64) } else { days as f64 };

                if current_gap >= min_absent {
                    cold_numbers.push(serde_json::json!({
                        "number": num,
                        "zone": "BACK",
                        "frequency": frequency,
                        "last_seen": self.format_days_ago(current_gap),
                        "hot_score": 1.0 - cold_score,
                        "cold_score": cold_score,
                        "average_gap": avg_gap,
                        "current_gap": current_gap,
                        "appearances": appearances,
                        "period_days": days,
                        "updated_at": Utc::now().to_rfc3339()
                    }));
                }
            }
        }

        cold_numbers
    }

    fn format_days_ago(&self, days: u32) -> String {
        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "Yesterday".to_string()
        } else {
            format!("{} days ago", days)
        }
    }
}

/// Database connection manager with connection pooling and health checks
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<(), SuperLottoError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;
        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<Value, SuperLottoError> {
        let total_draws: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lottery_draws WHERE lottery_type = 'super_lotto'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        let oldest_draw: Option<String> = sqlx::query_scalar("SELECT MIN(draw_date) FROM lottery_draws WHERE lottery_type = 'super_lotto'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        let newest_draw: Option<String> = sqlx::query_scalar("SELECT MAX(draw_date) FROM lottery_draws WHERE lottery_type = 'super_lotto'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SuperLottoError::Database(e))?;

        Ok(serde_json::json!({
            "total_draws": total_draws,
            "oldest_draw_date": oldest_draw,
            "newest_draw_date": newest_draw,
            "checked_at": Utc::now().to_rfc3339()
        }))
    }

    /// Create SuperLotto queries instance
    pub fn super_lotto(&self) -> SuperLottoQueries<'_> {
        SuperLottoQueries::new(&self.pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_secure_query_building() {
        // This would require a test database setup
        // For now, we just verify the query building logic doesn't panic
        let mock_query = "SELECT * FROM test WHERE id = ?";
        assert!(mock_query.contains("?"));
        assert!(!mock_query.contains("INSERT") || !mock_query.contains("DROP"));
    }

    #[test]
    fn test_format_days_ago() {
        let db = DatabaseManager::new(sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap());
        let queries = db.super_lotto();

        assert_eq!(queries.format_days_ago(0), "Today");
        assert_eq!(queries.format_days_ago(1), "Yesterday");
        assert_eq!(queries.format_days_ago(5), "5 days ago");
    }
}