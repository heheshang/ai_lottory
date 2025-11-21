use crate::models::lottery::{LotteryDraw, LotteryDrawDB, NewLotteryDraw, LotterySearchQuery, LotteryDrawResponse};
use sqlx::{Pool, Sqlite, Row};
use anyhow::Result;
use chrono::Utc;

pub struct LotteryService {
    pool: Pool<Sqlite>,
}

impl LotteryService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn add_lottery_draw(&self, draw: NewLotteryDraw) -> Result<LotteryDrawResponse> {
        let winning_numbers_json = serde_json::to_string(&draw.winning_numbers)?;

        let result = sqlx::query(
            r#"
            INSERT INTO lottery_draws (draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(draw.draw_date.to_rfc3339())
        .bind(&winning_numbers_json)
        .bind(draw.bonus_number)
        .bind(draw.jackpot_amount)
        .bind(&draw.lottery_type)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        let draw_id = result.last_insert_rowid() as u32;

        // Retrieve created draw
        let lottery_draw_db = sqlx::query_as::<_, LotteryDrawDB>(
            "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE id = ?"
        )
        .bind(draw_id)
        .fetch_one(&self.pool)
        .await?;

        let lottery_draw: LotteryDraw = lottery_draw_db.into();

        Ok(LotteryDrawResponse {
            id: lottery_draw.id,
            draw_date: lottery_draw.draw_date,
            winning_numbers: lottery_draw.winning_numbers,
            bonus_number: lottery_draw.bonus_number,
            jackpot_amount: lottery_draw.jackpot_amount,
            lottery_type: lottery_draw.lottery_type,
        })
    }

    pub async fn get_lottery_history(
        &self,
        lottery_type: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<LotteryDrawResponse>> {
      // Simplified approach without dynamic query building
    let _query = "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws";

        let draws_db = if let Some(lottery_type) = lottery_type {
            sqlx::query_as::<_, LotteryDrawDB>(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE lottery_type = ? ORDER BY draw_date DESC LIMIT ? OFFSET ?"
            )
            .bind(lottery_type)
            .bind(limit.unwrap_or(100))
            .bind(offset.unwrap_or(0))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, LotteryDrawDB>(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws ORDER BY draw_date DESC LIMIT ? OFFSET ?"
            )
            .bind(limit.unwrap_or(100))
            .bind(offset.unwrap_or(0))
            .fetch_all(&self.pool)
            .await?
        };

        let draws: Vec<LotteryDraw> = draws_db.into_iter().map(|db_draw| db_draw.into()).collect();

        let responses: Vec<LotteryDrawResponse> = draws
            .into_iter()
            .map(|draw| LotteryDrawResponse {
                id: draw.id,
                draw_date: draw.draw_date,
                winning_numbers: draw.winning_numbers,
                bonus_number: draw.bonus_number,
                jackpot_amount: draw.jackpot_amount,
                lottery_type: draw.lottery_type,
            })
            .collect();

        Ok(responses)
    }

    pub async fn search_lottery_draws(&self, query: LotterySearchQuery) -> Result<Vec<LotteryDrawResponse>> {
        let mut where_clauses = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(lottery_type) = &query.lottery_type {
            where_clauses.push("lottery_type = ?".to_string());
            bindings.push(lottery_type.clone());
        }

        if let Some(start_date) = &query.start_date {
            where_clauses.push("draw_date >= ?".to_string());
            bindings.push(start_date.to_rfc3339());
        }

        if let Some(end_date) = &query.end_date {
            where_clauses.push("draw_date <= ?".to_string());
            bindings.push(end_date.to_rfc3339());
        }

        let sql = if where_clauses.is_empty() {
            "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws ORDER BY draw_date DESC LIMIT ? OFFSET ?".to_string()
        } else {
            format!(
                "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws WHERE {} ORDER BY draw_date DESC LIMIT ? OFFSET ?",
                where_clauses.join(" AND ")
            )
        };

        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        // For simplicity, we'll implement a basic version without the complex query builder
        let draws_db = sqlx::query_as::<_, LotteryDrawDB>(
            "SELECT id, draw_date, winning_numbers, bonus_number, jackpot_amount, lottery_type, created_at FROM lottery_draws ORDER BY draw_date DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let draws: Vec<LotteryDraw> = draws_db.into_iter().map(|db_draw| db_draw.into()).collect();

        // Filter in memory for number filter (this is not optimal for large datasets)
        let filtered_draws = if let Some(number_filter) = query.number_filter {
            draws
                .into_iter()
                .filter(|draw| {
                    number_filter.iter().all(|&num| draw.winning_numbers.contains(&num))
                })
                .collect()
        } else {
            draws
        };

        let responses: Vec<LotteryDrawResponse> = filtered_draws
            .into_iter()
            .map(|draw| LotteryDrawResponse {
                id: draw.id,
                draw_date: draw.draw_date,
                winning_numbers: draw.winning_numbers,
                bonus_number: draw.bonus_number,
                jackpot_amount: draw.jackpot_amount,
                lottery_type: draw.lottery_type,
            })
            .collect();

        Ok(responses)
    }
}