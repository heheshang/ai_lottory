use crate::models::user::{User, UserRegistration, UserLogin, UserResponse};
use sqlx::Pool;
use sqlx::Sqlite;
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use anyhow::Result;
use chrono::Utc;

pub struct AuthService {
    pool: Pool<Sqlite>,
}

impl AuthService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn register(&self, registration: UserRegistration) -> Result<UserResponse> {
        // Check if username already exists
        let existing_user = sqlx::query("SELECT id FROM users WHERE username = ?")
            .bind(&registration.username)
            .fetch_optional(&self.pool)
            .await?;

        if existing_user.is_some() {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(registration.password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        // Insert user
        let result = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash, created_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&registration.username)
        .bind(&registration.email)
        .bind(&password_hash)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        let user_id = result.last_insert_rowid() as u32;

        // Retrieve created user
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UserResponse::from(user))
    }

    pub async fn login(&self, login: UserLogin) -> Result<UserResponse> {
        println!("🔵 [Auth Service] Starting login process for: {}", login.username);

        // Find user by username
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE username = ?"
        )
        .bind(&login.username)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))?;

        println!("🔵 [Auth Service] User found: {}", user.username);

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        let argon2 = Argon2::default();
        argon2.verify_password(login.password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("Invalid username or password"))?;

        println!("🔵 [Auth Service] Password verified successfully");

        // Update last login
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(&now)
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        println!("🔵 [Auth Service] Last login updated: {}", now);

        // Return updated user
        let updated_user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE id = ?"
        )
        .bind(user.id)
        .fetch_one(&self.pool)
        .await?;

        println!("🔵 [Auth Service] Login process completed successfully");
        Ok(UserResponse::from(updated_user))
    }

    pub async fn get_user_by_id(&self, user_id: u32) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, last_login FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(UserResponse::from))
    }
}