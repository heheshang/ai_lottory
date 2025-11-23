#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::auth_service::AuthService;
    use sqlx::{Pool, Sqlite, SqlitePool};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create tables for testing
        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_login DATETIME
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE login_attempts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                attempted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let password = "test_password_123";
        let hash = AuthService::hash_password(password).unwrap();

        // Verify hash is not the same as password
        assert_ne!(hash, password);
        assert!(hash.len() > 50); // Argon2 hash should be long

        // Verify password can be verified
        assert!(AuthService::verify_password(password, &hash).unwrap());

        // Verify wrong password fails
        assert!(!AuthService::verify_password("wrong_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_username_validation() {
        assert!(AuthService::validate_username("validuser").is_ok());
        assert!(AuthService::validate_username("user123").is_ok());
        assert!(AuthService::validate_username("user_name").is_ok());

        // Invalid usernames
        assert!(AuthService::validate_username("").is_err());
        assert!(AuthService::validate_username("us").is_err()); // too short
        assert!(AuthService::validate_username("this_is_a_very_long_username_that_exceeds_limit").is_err());
        assert!(AuthService::validate_username("user@name").is_err());
        assert!(AuthService::validate_username("user name").is_err());
    }

    #[tokio::test]
    async fn test_email_validation() {
        assert!(AuthService::validate_email("user@example.com").is_ok());
        assert!(AuthService::validate_email("test.email@domain.co.uk").is_ok());

        // Invalid emails
        assert!(AuthService::validate_email("").is_err());
        assert!(AuthService::validate_email("invalid-email").is_err());
        assert!(AuthService::validate_email("@domain.com").is_err());
        assert!(AuthService::validate_email("user@").is_err());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);
        let username = "testuser";
        let ip_address = "127.0.0.1";

        // First 4 attempts should succeed
        for i in 0..4 {
            let result = auth_service.check_rate_limit(username, ip_address).await;
            assert!(result.is_ok(), "Attempt {} should succeed", i);
        }

        // 5th attempt should fail
        let result = auth_service.check_rate_limit(username, ip_address).await;
        assert!(result.is_err(), "5th attempt should be rate limited");

        // Wait for rate limit to reset
        sleep(Duration::from_millis(100)).await;

        // Should still be rate limited (not enough time passed)
        let result = auth_service.check_rate_limit(username, ip_address).await;
        assert!(result.is_err(), "Should still be rate limited");
    }

    #[tokio::test]
    async fn test_successful_registration() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        let user_request = crate::models::user::UserRegistration {
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
            password: "securePassword123".to_string(),
        };

        let result = auth_service.register(user_request).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "newuser");
        assert_eq!(user.email, "newuser@example.com");
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_duplicate_username_registration() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        let user_request = crate::models::user::UserRegistration {
            username: "duplicate".to_string(),
            email: "first@example.com".to_string(),
            password: "password123".to_string(),
        };

        // First registration should succeed
        let result1 = auth_service.register(user_request.clone()).await;
        assert!(result1.is_ok());

        // Second registration with same username should fail
        let result2 = auth_service.register(user_request).await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_sql_injection_prevention() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        // Register a normal user first
        let user_request = crate::models::user::UserRegistration {
            username: "normaluser".to_string(),
            email: "normal@example.com".to_string(),
            password: "password123".to_string(),
        };
        auth_service.register(user_request).await.unwrap();

        // Try SQL injection in username
        let malicious_request = crate::models::user::UserRegistration {
            username: "'; DROP TABLE users; --".to_string(),
            email: "malicious@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.register(malicious_request).await;

        // Should fail due to invalid username format, not SQL injection
        assert!(result.is_err());

        // Verify users table still exists and normal user is still there
        let login_request = crate::models::user::UserLogin {
            username: "normaluser".to_string(),
            password: "password123".to_string(),
        };

        let login_result = auth_service.login(login_request).await;
        assert!(login_result.is_ok());
    }

    #[tokio::test]
    async fn test_session_token_generation() {
        let pool = create_test_pool().await;
        let auth_service = AuthService::new(pool);

        let user_id = 42;
        let token = auth_service.generate_session_token(user_id).unwrap();

        // Token should be a valid base64 string
        assert!(token.len() > 20);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));

        // Different calls should generate different tokens
        let token2 = auth_service.generate_session_token(user_id).unwrap();
        assert_ne!(token, token2);
    }
}