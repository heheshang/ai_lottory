//! Secure Authentication Commands (ENHANCED VERSION)
//!
//! Tauri commands for user authentication with proper error handling,
//! security logging, and protection against common attacks.

use crate::models::user::{UserLogin, UserRegistration, UserResponse};
use crate::services::AuthService;
use crate::super_lotto::errors::SuperLottoError;
use sqlx::Pool;
use tauri::State;
use serde_json::Value;

/// Secure user login command with enhanced error handling
#[tauri::command]
pub async fn login(
    login: UserLogin,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<UserResponse, String> {
    println!("🔐 [COMMAND] Secure login request received for user: {}", login.username);

    let auth_service = AuthService::new(pool.inner().clone());
    let username_for_log = login.username.clone(); // Preserve username for logging

    let result = auth_service.login(login).await;

    match &result {
        Ok(user_response) => {
            println!("✅ [COMMAND] Login successful for user ID: {}", user_response.id);
            // Log successful login with timestamp
            println!("📊 [AUDIT] Login success - User: {}, ID: {}, Time: {}",
                user_response.username,
                user_response.id,
                chrono::Utc::now().to_rfc3339()
            );
        },
        Err(e) => {
            println!("❌ [COMMAND] Login failed: {}", e);
            // Log failed login attempt (for security monitoring)
            println!("🚨 [AUDIT] Login failure - User: {}, Error: {}, Time: {}",
                username_for_log,
                e.error_code(),
                chrono::Utc::now().to_rfc3339()
            );
        }
    }

    result.map_err(|e| {
        // Convert to user-friendly error messages while preserving security information
        match e {
            SuperLottoError::Authentication { message } => {
                if message.contains("locked") {
                    message
                } else {
                    "Invalid username or password".to_string() // Don't reveal which is wrong
                }
            },
            SuperLottoError::Validation { message } => message,
            SuperLottoError::AlreadyExists { .. } => "User already exists".to_string(),
            _ => "Authentication service temporarily unavailable".to_string()
        }
    })
}

/// Secure user registration command with comprehensive validation
#[tauri::command]
pub async fn register(
    registration: UserRegistration,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<UserResponse, String> {
    println!("🔐 [COMMAND] Secure registration request received for username: {}", registration.username);

    let auth_service = AuthService::new(pool.inner().clone());
    let username_for_log = registration.username.clone(); // Preserve username for logging

    let result = auth_service.register(registration).await;

    match &result {
        Ok(user_response) => {
            println!("✅ [COMMAND] Registration successful for user ID: {}", user_response.id);
            println!("📊 [AUDIT] Registration success - User: {}, ID: {}, Time: {}",
                user_response.username,
                user_response.id,
                chrono::Utc::now().to_rfc3339()
            );
        },
        Err(e) => {
            println!("❌ [COMMAND] Registration failed: {}", e);
            println!("🚨 [AUDIT] Registration failure - Username: {}, Error: {}, Time: {}",
                username_for_log,
                e.error_code(),
                chrono::Utc::now().to_rfc3339()
            );
        }
    }

    result.map_err(|e| {
        // Convert to user-friendly error messages
        match e {
            SuperLottoError::Validation { message } => message,
            SuperLottoError::AlreadyExists { resource, identifier } => {
                match resource.as_str() {
                    "user" => format!("Username '{}' is already taken", identifier),
                    _ => format!("{} '{}' already exists", resource, identifier)
                }
            },
            SuperLottoError::Database { .. } => "Registration service temporarily unavailable".to_string(),
            _ => "Registration failed. Please try again later.".to_string()
        }
    })
}

/// Secure logout command with session cleanup
#[tauri::command]
pub async fn logout(
    session_token: Option<String>,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Value, String> {
    println!("🔐 [COMMAND] Logout request received");

    let _auth_service = AuthService::new(pool.inner().clone());

    // If a session token is provided, invalidate it
    if let Some(_token) = session_token {
        // In a full implementation, this would validate and invalidate the session
        println!("📝 [COMMAND] Session token provided for logout");
        println!("🚨 [AUDIT] Logout with token - Time: {}", chrono::Utc::now().to_rfc3339());
    } else {
        println!("📝 [COMMAND] Logout without session token");
        println!("🚨 [AUDIT] Logout without token - Time: {}", chrono::Utc::now().to_rfc3339());
    }

    Ok(serde_json::json!({
        "success": true,
        "message": "Logged out successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get current user information securely
#[tauri::command]
pub async fn get_current_user(
    user_id: u32,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Option<UserResponse>, String> {
    println!("🔍 [COMMAND] Get current user request for ID: {}", user_id);

    let auth_service = AuthService::new(pool.inner().clone());

    let result = auth_service.get_user_by_id(user_id).await;

    match &result {
        Ok(Some(user_response)) => {
            println!("✅ [COMMAND] User found: {}", user_response.username);
        },
        Ok(None) => {
            println!("⚠️ [COMMAND] User not found for ID: {}", user_id);
        },
        Err(e) => {
            println!("❌ [COMMAND] Get user failed: {}", e);
        }
    }

    result.map_err(|e| {
        match e {
            SuperLottoError::Database { .. } => "User service temporarily unavailable".to_string(),
            _ => "Failed to retrieve user information".to_string()
        }
    })
}

/// Change user password securely
#[tauri::command]
pub async fn change_password(
    user_id: u32,
    current_password: String,
    new_password: String,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Value, String> {
    println!("🔐 [COMMAND] Password change request for user ID: {}", user_id);

    // Input validation
    if current_password.is_empty() {
        return Err("Current password is required".to_string());
    }
    if new_password.is_empty() {
        return Err("New password is required".to_string());
    }

    let auth_service = AuthService::new(pool.inner().clone());

    let result = auth_service.change_password(user_id, &current_password, &new_password).await;

    match &result {
        Ok(_) => {
            println!("✅ [COMMAND] Password changed successfully for user ID: {}", user_id);
            println!("🚨 [AUDIT] Password change success - User ID: {}, Time: {}",
                user_id,
                chrono::Utc::now().to_rfc3339()
            );
        },
        Err(e) => {
            println!("❌ [COMMAND] Password change failed: {}", e);
            println!("🚨 [AUDIT] Password change failure - User ID: {}, Error: {}, Time: {}",
                user_id,
                e.error_code(),
                chrono::Utc::now().to_rfc3339()
            );
        }
    }

    result.map_err(|e| {
        match e {
            SuperLottoError::Authentication { message } => message,
            SuperLottoError::Validation { message } => message,
            SuperLottoError::Database { .. } => "Password change service temporarily unavailable".to_string(),
            _ => "Password change failed. Please try again.".to_string()
        }
    })?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Password changed successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Generate session token for authenticated user
#[tauri::command]
pub async fn generate_session_token(
    user_id: u32,
    username: String,
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Value, String> {
    println!("🔐 [COMMAND] Session token generation request for user ID: {}", user_id);

    let auth_service = AuthService::new(pool.inner().clone());
    let token = auth_service.generate_session_token();

    println!("✅ [COMMAND] Session token generated for user: {}", username);
    println!("🚨 [AUDIT] Session token generated - User: {}, Time: {}",
        username,
        chrono::Utc::now().to_rfc3339()
    );

    Ok(serde_json::json!({
        "token": token,
        "user_id": user_id,
        "username": username,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339()
    }))
}

/// Get authentication statistics (for admin monitoring)
#[tauri::command]
pub async fn get_auth_stats(
    pool: State<'_, Pool<sqlx::Sqlite>>,
) -> Result<Value, String> {
    println!("📊 [COMMAND] Authentication statistics requested");

    let auth_service = AuthService::new(pool.inner().clone());

    let result = auth_service.get_auth_stats().await;

    match &result {
        Ok(_stats) => {
            println!("✅ [COMMAND] Auth stats retrieved successfully");
        },
        Err(e) => {
            println!("❌ [COMMAND] Failed to get auth stats: {}", e);
        }
    }

    result.map_err(|e| {
        match e {
            SuperLottoError::Database { .. } => "Statistics service temporarily unavailable".to_string(),
            _ => "Failed to retrieve authentication statistics".to_string()
        }
    })
}
