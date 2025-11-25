// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod models;
mod services;
mod super_lotto;

// Optional modules for performance and caching
mod cache;
mod performance;

use database::connection::establish_connection;

#[tokio::main]
async fn main() {
    println!("🟢 [Tauri] Starting Tauri application...");

    // Initialize database connection
    println!("🔵 [Tauri] Establishing database connection...");
    let pool = establish_connection()
        .await
        .expect("Failed to establish database connection");
    println!("🔵 [Tauri] Database connection established successfully");

    println!("🔵 [Tauri] Building Tauri application...");

    tauri::Builder::default()
        .manage(pool)
        .invoke_handler(tauri::generate_handler![
            commands::auth::login,
            commands::auth::register,
            commands::auth::logout,
            commands::auth::get_current_user,
            commands::lottery::get_lottery_history,
            commands::lottery::add_lottery_draw,
            commands::lottery::search_lottery_draws,
            commands::analysis::get_hot_numbers,
            commands::analysis::get_cold_numbers,
            commands::analysis::get_number_statistics,
            // Super Lotto commands
            super_lotto::commands::get_predictions,
            super_lotto::commands::generate_all_predictions,
            super_lotto::commands::get_prediction_comparison,
            super_lotto::commands::save_prediction_result,
            super_lotto::commands::get_saved_predictions,
            super_lotto::commands::delete_prediction,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
