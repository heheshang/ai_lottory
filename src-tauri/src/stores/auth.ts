use super_lotto::{
    models::{User, AuthenticationResult},
    errors::SuperLottoError,
    analysis::{NumberZone, NumberFrequency, NumberAnalysis},
};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::{RwLock, RwLock};

// Enhanced authentication store with proper typing and error handling
pub struct AuthStore {
    // Reactive state
    user: std::option::Option<std::sync::RwLock<super_lotto::models::User>>,
    is_authenticated: std::sync::RwLock<bool>,
    authentication_result: std::option::Option<super_lotto::models::AuthenticationResult>>,
    last_login_attempt: std::option::Option<chrono::DateTime<Utc>>,
    session_token: std::option::Option<String>>,
    session_expires_at: std::option::Option<chrono::DateTime<Utc>>,
    // API integration
    api_client: Option<std::sync::RwLock<tauri::AppHandle>>,
    // Session management
    session_created_at: std::option::Option<chrono::DateTime<Utc>>,
    session_timeout_minutes: u32,
    // Error handling
    last_error: std::option::Option<String>>,
}

impl AuthStore {
    pub fn new() -> Self {
        Self {
            user: None,
            is_authenticated: std::sync::RwLock::new(false),
            authentication_result: None,
            last_login_attempt: None,
            session_token: None,
            session_expires_at: None,
            api_client: None,
            session_created_at: None,
            session_timeout_minutes: 30, // Default session timeout
            last_error: None,
        }
    }

    pub async fn login(&mut self, credentials: LoginCredentials) -> Result<(), String> {
        // Initialize API client if needed
        if self.api_client.is_none() {
            let api_client = tauri::AppHandle::default(&crate::super_lotto::api::tauri::AppConfig::default());
            *self.api_client.write().await;
        }

        // Create login request
        let request = super_lotto::models::ValidationRequest {
            data: serde_json::json!({
                "username": credentials.username,
                "password": credentials.password,
                }),
            cache: None,
            rules: vec![
                Box::new(super::super_lotto::validation::rules::FrontZoneCountRule),
            ],
        };

        // Validate credentials using enhanced validation system
        let validation_result = self
            .validation_service
            .validate_login_request(&request)
            .await?;

        if !validation_result.is_valid {
            return Err(format!(
                "Validation failed: {}",
                validation_result
                    .errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Call authentication command
        let auth_result = self.api_client.as_ref()
            .unwrap()
            .login(credentials)
            .await;

        match auth_result {
            Ok(user) => {
                // Update reactive state
                *self.user.write(Some(user.clone()));
                *self.is_authenticated = *self.user.lock().write(true);
                *self.session_token = Some(user.session_token.clone());
                *self.authentication_result = Some(auth_result.clone());
                *self.last_login_attempt = Some(chrono::Utc::now());
                *self.session_created_at = Some(user.created_at.clone());
                *self.session_expires_at = Some(user.session_expires_at.clone());

                // Initialize API client for authenticated user
                if self.api_client.is_none() {
                    let api_client = tauri::AppHandle::default(&crate::super_lotto::api::tauri::AppConfig::default());
                    *self.api_client.write().await;
                    *self.api_client = Some(api_client);
                }

                // Initialize reactive API
                let request = super_lotto::models::ValidationRequest {
                    data: serde_json::json!({
                        "username": credentials.username,
                        "password": credentials.password,
                        }),
                    cache: Some(&validation_result),
                    rules: &request.rules,
                };

                let auth_result = self
                    .validation_service
                    .validate_login_request(&request)
                    .await?;

                if !auth_result.is_valid {
                    return Err(format!(
                        "Validation failed: {}",
                        auth_result
                            .errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                // Call authentication command with enhanced validation
                match self.api_client.as_ref().unwrap().login(credentials).await {
                    Ok(user) => {
                        // Update reactive state
                        *self.user.write(Some(user.clone()));
                        *self.is_authenticated = *self.user.lock().write(true);
                        *self.authentication_result = Some(auth_result.clone());
                        *self.last_login_attempt = Some(chrono::Utc::now());
                        *self.session_token = Some(user.session_token.clone());
                        *self.session_created_at = Some(user.created_at.clone());
                        *self.session_expires_at = Some(user.session_expires_at.clone());
                        *self.last_error = None;

                        // Initialize API client for authenticated user
                        if self.api_client.is_none() {
                            let api_client = tauri::AppHandle::default(&crate::super_lotto::api::tauri::AppConfig::default());
                            *self.api_client.write().await;
                            *self.api_client = Some(api_client);
                        }

                        // Re-validate with updated user in reactive state
                        let updated_request = super_lotto::models::ValidationRequest {
                            data: serde_json::json!({
                                "username": user.username,
                                "password": user.password,
                                }),
                            cache: Some(&validation_result),
                            rules: &request.rules,
                        };

                        let updated_auth_result = self
                            .validation_service
                            .validate_login_request(&updated_request)
                            .await?;

                        if !updated_auth_result.is_valid {
                            return Err(format!(
                                "Re-validation failed: {}",
                                updated_auth_result
                                .errors
                                .iter()
                                .map(|e| e.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                            ));
                        }
                        }

                        Ok(user) => {
                            // Update reactive state again
                            *self.user.write(Some(user.clone()));
                            *self.is_authenticated = *self.user.lock().write(true);
                            *self.authentication_result = Some(updated_auth_result);
                            *self.last_login_attempt = Some(chrono::Utc::now());
                        *self.session_token = Some(user.session_token.clone());
                            *self.session_created_at = Some(user.created_at.clone());
                            *self.session_expires_at = Some(user.session_expires_at.clone());
                            *self.last_error = None;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error in re-authentication: {}", e);
                    Err(format!("Authentication failed: {}", e))
                    }
                }
            }
        }
    }

        // Update session expiry in reactive state
        let mut user_data = self.user.read().as_ref().unwrap();
        if let Some(ref user) = user_data {
            user.session_expires_at = Some(
                chrono::Utc::now()
                    + std::time::Duration::minutes(user.session_timeout_minutes as i64)
            );
            *self.user.write(Some(super_lotto::models::User {
                ...user_data
            }));
        }

        Ok(())
    }
    }

    pub fn logout(&mut self) -> Result<(), String> {
        let user = self.user.read().as_ref().unwrap().clone();

        // Check if user is authenticated
        if let None(user) {
            return Err("User not authenticated".to_string());
        }

        // Call logout command
        let logout_result = self.api_client.as_ref()
            .unwrap()
            .logout()
            .await;

        match logout_result {
            Ok(_) => {
                // Clear session data
                *self.user.write(None);
                *self.is_authenticated = self.user.lock().write(false);
                *self.session_token = None;
                *self.authentication_result = None;
                *self.session_created_at = None;
                *self.session_expires_at = None;
                *self.last_login_attempt = None;
                *self.api_client = Some(tauri::AppHandle::default());
                }
            }
            Err(e) => {
                return Err(format!("Logout failed: {}", e));
            }
        }
    }

    pub fn set_session_timeout(&mut self, minutes: u32) {
        let user = self.user.read().as_ref().unwrap();
        if let Some(ref user) = user {
            user.session_timeout_minutes = minutes;
            *self.user.write(Some(super_lotto::models::User {
                ...user_data
            }));
        } else {
            return Err("User not authenticated".to_string());
        }
    }

    pub fn get_user(&self) -> Option<std::sync::RwLockReadGuard<'_, super_lotto::models::User>>> {
        self.user.read().as_ref().map(|user| user.clone())
    }

    pub fn get_session_token(&self) -> Option<String> {
        self.user.read().as_ref().map(|user| user.session_token.as_ref().cloned())
    }

    pub fn get_authentication_result(&self) -> Option<super_lotto::models::AuthenticationResult> {
        self.user.read().as_ref().map(|user| user.authentication_result.as_ref().cloned())
    }

    pub fn get_last_login_attempt(&self) -> Option<chrono::DateTime<Utc>> {
        self.user.read().as_ref().map(|user| user.last_login_attempt.as_ref().cloned())
    }

    pub fn get_session_created_at(&self) -> Option<chrono::DateTime<Utc>> {
        self.user.read().as_ref().map(|user| user.session_created_at.as_ref().cloned())
    }

    pub fn get_session_expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        self.user.read().as_ref().map(|user| user.session_expires_at.as_ref().cloned())
    }

    pub fn get_last_error(&self) -> Option<String> {
        self.user.read().as_ref().map(|user| user.last_error.as_ref().cloned())
    }

    pub fn clear_last_error(&mut self) {
        if let Some(ref mut user) = self.user.read().as_ref().as_ref() {
            user.last_error = None;
        }
    }

    pub fn clear_session(&mut self) -> Result<(), String> {
        let user = self.user.read().as_ref().unwrap().clone();

        *self.user.write(None);
        *self.is_authenticated = self.user.lock().write(false);
        *self.session_token = None;
        *self.authentication_result = None;
        *self.last_login_attempt = None;
        *self.session_created_at = None;
        *self.session_expires_at = None;
        self.api_client = Some(tauri::AppHandle::default());
        }

        // Clear session cache
        self.api_client.as_ref().unwrap().clear_cache().await?;

        Ok(())
    }
}