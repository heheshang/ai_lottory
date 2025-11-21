use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryDraw {
    pub id: u32,
    pub draw_date: DateTime<Utc>,
    pub winning_numbers: Vec<u32>,
    pub bonus_number: Option<u32>,
    pub jackpot_amount: Option<f64>,
    pub lottery_type: String, // e.g., "powerball", "megamillions", etc.
    pub created_at: DateTime<Utc>,
}

// Database representation for SQLx
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LotteryDrawDB {
    pub id: u32,
    pub draw_date: DateTime<Utc>,
    pub winning_numbers: String, // JSON string for SQLite
    pub bonus_number: Option<u32>,
    pub jackpot_amount: Option<f64>,
    pub lottery_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<LotteryDrawDB> for LotteryDraw {
    fn from(db: LotteryDrawDB) -> Self {
        Self {
            id: db.id,
            draw_date: db.draw_date,
            winning_numbers: serde_json::from_str(&db.winning_numbers).unwrap_or_default(),
            bonus_number: db.bonus_number,
            jackpot_amount: db.jackpot_amount,
            lottery_type: db.lottery_type,
            created_at: db.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewLotteryDraw {
    pub draw_date: DateTime<Utc>,
    pub winning_numbers: Vec<u32>,
    pub bonus_number: Option<u32>,
    pub jackpot_amount: Option<f64>,
    pub lottery_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotterySearchQuery {
    pub lottery_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub number_filter: Option<Vec<u32>>, // Find draws containing these numbers
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LotteryDrawResponse {
    pub id: u32,
    pub draw_date: DateTime<Utc>,
    pub winning_numbers: Vec<u32>,
    pub bonus_number: Option<u32>,
    pub jackpot_amount: Option<f64>,
    pub lottery_type: String,
}