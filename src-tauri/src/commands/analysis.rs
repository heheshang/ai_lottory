use crate::models::analysis::{AnalysisRequest, NumberStatistics, HotNumbersResponse, ColdNumbersResponse};
use crate::services::AnalysisService;
use sqlx::Pool;
use tauri::State;

#[tauri::command]
pub async fn get_hot_numbers(
    request: AnalysisRequest,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<HotNumbersResponse, String> {
    let analysis_service = AnalysisService::new(pool.inner().clone());

    analysis_service
        .get_hot_numbers(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cold_numbers(
    request: AnalysisRequest,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<ColdNumbersResponse, String> {
    let analysis_service = AnalysisService::new(pool.inner().clone());

    analysis_service
        .get_cold_numbers(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_number_statistics(
    number: u32,
    lottery_type: String,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<NumberStatistics, String> {
    let analysis_service = AnalysisService::new(pool.inner().clone());

    analysis_service
        .get_number_statistics(number, &lottery_type)
        .await
        .map_err(|e| e.to_string())
}