use crate::models::user::{UserLogin, UserRegistration, UserResponse};
use crate::services::AuthService;
use sqlx::Pool;
use tauri::State;

#[tauri::command]
pub async fn login(
    login: UserLogin,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<UserResponse, String> {
    println!("🔵 [Rust Command] Login request received for user: {}", login.username);

    let auth_service = AuthService::new(pool.inner().clone());

    let result = auth_service
        .login(login)
        .await;

    match &result {
        Ok(user_response) => println!("🔵 [Rust Command] Login successful for user ID: {}", user_response.id),
        Err(e) => println!("🔴 [Rust Command] Login failed: {}", e),
    }

    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register(
    registration: UserRegistration,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<UserResponse, String> {
    let auth_service = AuthService::new(pool.inner().clone());

    auth_service
        .register(registration)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<(), String> {
    // In a real application, you'd want to invalidate the session/token
    // For now, we'll just return success
    Ok(())
}

#[tauri::command]
pub async fn get_current_user(
    user_id: u32,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Option<UserResponse>, String> {
    let auth_service = AuthService::new(pool.inner().clone());

    auth_service
        .get_user_by_id(user_id)
        .await
        .map_err(|e| e.to_string())
}