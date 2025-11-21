use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs;
use std::path::PathBuf;
use super::migrations::run_migrations;

pub fn get_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use local directory for now to avoid Tauri 2.0 API issues
    let db_dir = PathBuf::from("./database");

    if !db_dir.exists() {
        fs::create_dir_all(&db_dir)?;
    }

    Ok(db_dir.join("lottery.db"))
}

pub async fn establish_connection() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    println!("🔵 [Database] Establishing database connection...");
    let db_path = get_database_path()?;
    println!("🔵 [Database] Database path: {:?}", db_path);

    // Create database file if it doesn't exist
    if !db_path.exists() {
        println!("🔵 [Database] Creating database file...");
        fs::File::create(&db_path)?;
        println!("🔵 [Database] Database file created");
    }

    let database_url = format!("sqlite:{}", db_path.to_string_lossy());
    println!("🔵 [Database] Connecting to database: {}", database_url);

    let pool = SqlitePool::connect(&database_url).await?;
    println!("🔵 [Database] Database connection established");

    // Run migrations
    println!("🔵 [Database] Running migrations...");
    run_migrations(&pool).await?;
    println!("🔵 [Database] Migrations completed successfully");

    Ok(pool)
}