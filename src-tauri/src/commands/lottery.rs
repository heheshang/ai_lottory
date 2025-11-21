use crate::models::lottery::{NewLotteryDraw, LotterySearchQuery, LotteryDrawResponse};
use crate::services::LotteryService;
use sqlx::Pool;
use tauri::State;

#[tauri::command]
pub async fn get_lottery_history(
    lottery_type: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Vec<LotteryDrawResponse>, String> {
    let lottery_service = LotteryService::new(pool.inner().clone());

    lottery_service
        .get_lottery_history(lottery_type, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_lottery_draw(
    draw: NewLotteryDraw,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<LotteryDrawResponse, String> {
    let lottery_service = LotteryService::new(pool.inner().clone());

    lottery_service
        .add_lottery_draw(draw)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_lottery_draws(
    query: LotterySearchQuery,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Vec<LotteryDrawResponse>, String> {
    let lottery_service = LotteryService::new(pool.inner().clone());

    lottery_service
        .search_lottery_draws(query)
        .await
        .map_err(|e| e.to_string())
}