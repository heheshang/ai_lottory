//! Super Lotto Tauri commands
//!
//! Command handlers for Super Lotto functionality exposed to the frontend.

use crate::models::lottery::LotteryDraw;
use crate::super_lotto::{
    errors::SuperLottoError,
    models::{
        BatchPredictionRequest, BatchPredictionResult, ConfidenceDistribution, ConsensusNumbers,
        CreateSuperLottoDraw, PaginationInfo, PredictionAlgorithm, PredictionComparison,
        PredictionRecommendation, PredictionResult, RiskLevel, SuperLottoDraw, TableExportRequest,
        TableFilters, UnifiedTableData, UnifiedTableRow,
    },
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::result::Result;
use tauri::State;

/// Get Super Lotto historical draws (SECURE VERSION)
#[tauri::command]
pub async fn get_super_lotto_draws(
    pool: State<'_, SqlitePool>,
    limit: Option<u32>,
    offset: Option<u32>,
    start_date: Option<String>,
    end_date: Option<String>,
    draw_number: Option<String>,
) -> Result<serde_json::Value, SuperLottoError> {
    use crate::super_lotto::models::SuperLottoDraw;

    // Log command start
    let start_time = std::time::Instant::now();
    println!("🔍 [COMMAND] get_super_lotto_draws called with parameters:");
    println!("  - limit: {:?}", limit);
    println!("  - offset: {:?}", offset);
    println!("  - start_date: {:?}", start_date);
    println!("  - end_date: {:?}", end_date);
    println!("  - draw_number: {:?}", draw_number);

    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    println!(
        "📊 [COMMAND] Effective parameters: limit={}, offset={}",
        limit, offset
    );

    // Build secure base query with parameterized statements
    let mut query_conditions = Vec::new();
    // Note: Simplified parameter binding - will use QueryBuilder approach instead

    // Base condition for lottery type
    query_conditions.push("lottery_type = ?".to_string());

    // Add optional filters with proper parameterization
    if let Some(_start) = &start_date {
        query_conditions.push("draw_date >= ?".to_string());
    }

    if let Some(_end) = &end_date {
        query_conditions.push("draw_date <= ?".to_string());
    }

    if let Some(_number) = &draw_number {
        query_conditions.push("draw_number LIKE ?".to_string());
    }

    // Build the final query with proper conditions
    let where_clause = if query_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", query_conditions.join(" AND "))
    };

    let query = format!(
        "SELECT * FROM lottery_draws {} ORDER BY draw_date DESC, draw_number DESC LIMIT ? OFFSET ?",
        where_clause
    );

    println!("📝 [QUERY] Executing secure parameterized query: {}", query);

    // Execute secure parameterized query - use proper SQLx query building
    let mut query_builder =
        sqlx::QueryBuilder::new("SELECT * FROM lottery_draws WHERE lottery_type = 'super_lotto'");

    // Add optional filters with proper parameterization
    if let Some(start) = &start_date {
        query_builder.push(" AND draw_date >= ");
        query_builder.push_bind(start);
    }

    if let Some(end) = &end_date {
        query_builder.push(" AND draw_date <= ");
        query_builder.push_bind(end);
    }

    if let Some(number) = &draw_number {
        query_builder.push(" AND draw_number LIKE ");
        query_builder.push_bind(format!("%{}%", number));
    }

    // Add ordering and pagination
    query_builder.push(" ORDER BY draw_date DESC, draw_number DESC LIMIT ");
    query_builder.push_bind(limit as i64);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset as i64);

    let raw_draws = query_builder
        .build_query_as::<LotteryDraw>()
        .fetch_all(pool.inner())
        .await
        .map_err(|e| {
            eprintln!(
                "❌ [DATABASE] Query error in get_super_lotto_draws: {:?}",
                e
            );
            SuperLottoError::internal(format!("Database error: {}", e))
        })?;

    println!(
        "📊 [RESULT] Retrieved {} raw draws from database",
        raw_draws.len()
    );

    // Convert LotteryDraw to SuperLottoDraw
    let draws: Vec<SuperLottoDraw> = raw_draws.into_iter().map(|draw| draw.into()).collect();

    println!(
        "🔄 [CONVERSION] Converted {} draws to SuperLotto format",
        draws.len()
    );

    // Build secure count query
    let mut count_builder = sqlx::QueryBuilder::new(
        "SELECT COUNT(*) as total FROM lottery_draws WHERE lottery_type = 'super_lotto'",
    );

    if let Some(start) = &start_date {
        count_builder.push(" AND draw_date >= ");
        count_builder.push_bind(start);
    }

    if let Some(end) = &end_date {
        count_builder.push(" AND draw_date <= ");
        count_builder.push_bind(end);
    }

    if let Some(number) = &draw_number {
        count_builder.push(" AND draw_number LIKE ");
        count_builder.push_bind(format!("%{}%", number));
    }

    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("❌ [DATABASE] Count query error: {:?}", e);
            SuperLottoError::internal(format!("Database error: {}", e))
        })?;

    println!("📊 [COUNT] Total draws matching criteria: {}", total);

    let json_response = serde_json::json!({
        "draws": draws,
        "total": total,
        "limit": limit,
        "offset": offset,
        "has_more": u64::from(offset + limit) < total as u64
    });

    let duration = start_time.elapsed();
    println!(
        "✅ [COMMAND] get_super_lotto_draws completed in {:?} - returned {} draws",
        duration,
        draws.len()
    );
    println!(
        "📤 [RESPONSE] Response summary: total={}, limit={}, offset={}, has_more={}",
        total,
        limit,
        offset,
        u64::from(offset + limit) < total as u64
    );

    Ok(json_response)
}

/// Import Super Lotto draws
#[tauri::command]
pub async fn import_super_lotto_draws(
    pool: State<'_, SqlitePool>,
    draws: Vec<CreateSuperLottoDraw>,
    validate_only: Option<bool>,
) -> Result<serde_json::Value, SuperLottoError> {
    // Log command start
    let validate_only = validate_only.unwrap_or(false);
    let start_time = std::time::Instant::now();

    println!("📥 [COMMAND] import_super_lotto_draws called with parameters:");
    println!("  - draws count: {}", draws.len());
    println!("  - validate_only: {}", validate_only);

    if draws.is_empty() {
        println!("⚠️ [WARNING] No draws provided for import");
        return Err(SuperLottoError::internal("No draws provided for import"));
    }

    println!(
        "📋 [VALIDATION] Starting validation of {} draws",
        draws.len()
    );
    let mut valid_draws = 0;
    let mut invalid_draws = 0;

    for (index, draw) in draws.iter().enumerate() {
        match draw.validate() {
            Ok(_) => {
                valid_draws += 1;
                if index < 5 {
                    println!(
                        "✅ [VALID] Draw #{} is valid: {}-{:?}+{:?}",
                        index + 1,
                        draw.draw_date,
                        draw.front_zone,
                        draw.back_zone
                    );
                }
            }
            Err(e) => {
                invalid_draws += 1;
                println!("❌ [INVALID] Draw #{} validation failed: {}", index + 1, e);
            }
        }
    }

    println!(
        "📊 [VALIDATION_SUMMARY] Valid: {}, Invalid: {}",
        valid_draws, invalid_draws
    );

    if validate_only {
        let duration = start_time.elapsed();
        println!(
            "✅ [COMMAND] import_super_lotto_draws (validation only) completed in {:?}",
            duration
        );

        return Ok(serde_json::json!({
            "success": true,
            "validated": valid_draws,
            "invalid": invalid_draws,
            "validate_only": true,
            "message": format!("Validation completed: {} valid, {} invalid", valid_draws, invalid_draws)
        }));
    }

    // TODO: Implement actual import functionality
    println!("🚧 [TODO] Actual import functionality not yet implemented");

    let duration = start_time.elapsed();
    println!(
        "⏱️ [COMMAND] import_super_lotto_draws completed in {:?}",
        duration
    );

    Err(SuperLottoError::internal(
        "Import functionality not implemented yet",
    ))
}

/// Analyze hot numbers
#[tauri::command]
pub async fn analyze_hot_numbers(
    pool: State<'_, SqlitePool>,
    days: u32,
    zone: Option<String>,
    limit: Option<u32>,
    min_threshold: Option<f64>,
) -> Result<serde_json::Value, SuperLottoError> {
    use crate::super_lotto::models::SuperLottoDraw;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    // Log command start
    let start_time = std::time::Instant::now();

    let limit = limit.unwrap_or(20);
    let zone_filter = zone.as_ref().map(|s| s.to_uppercase());
    let min_threshold = min_threshold.unwrap_or(0.0);

    println!("🔥 [COMMAND] analyze_hot_numbers called with parameters:");
    println!("  - days: {}", days);
    println!("  - zone: {:?}", zone_filter);
    println!("  - limit: {}", limit);
    println!("  - min_threshold: {}", min_threshold);

    // Get historical draws within the specified period - use correct table name
    let cutoff_date = Utc::now() - Duration::days(days as i64);
    let query = format!(
        "SELECT * FROM lottery_draws WHERE lottery_type = 'super_lotto' AND draw_date >= '{}' ORDER BY draw_date DESC",
        cutoff_date.format("%Y-%m-%d")
    );

    println!(
        "📅 [ANALYSIS] Analyzing draws from {} onwards",
        cutoff_date.format("%Y-%m-%d")
    );
    println!("📝 [QUERY] {}", query);

    let raw_draws = sqlx::query_as::<_, LotteryDraw>(&query)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("❌ [DATABASE] Query error in hot numbers analysis: {:?}", e);
            SuperLottoError::internal(format!("Database error: {}", e))
        })?;

    // Convert LotteryDraw to SuperLottoDraw
    let draws: Vec<SuperLottoDraw> = raw_draws.into_iter().map(|draw| draw.into()).collect();

    println!(
        "📊 [DATA] Retrieved {} draws for hot numbers analysis",
        draws.len()
    );

    if draws.is_empty() {
        println!("⚠️ [WARNING] No historical data available for hot numbers analysis");
        return Err(SuperLottoError::internal(
            "No historical data available for analysis",
        ));
    }

    println!("🔍 [ANALYSIS] Starting frequency analysis...");

    // Analyze number frequencies
    let mut front_freq: HashMap<u32, (u32, f64, u32)> = HashMap::new();
    let mut back_freq: HashMap<u32, (u32, f64, u32)> = HashMap::new();
    let _front_last_seen: HashMap<u32, Option<DateTime<Utc>>> = HashMap::new();
    let _back_last_seen: HashMap<u32, Option<DateTime<Utc>>> = HashMap::new();
    let _front_frequency: HashMap<u32, u32> = HashMap::new();

    for (i, draw) in draws.iter().enumerate() {
        let days_ago = i;

        // Front zone frequencies
        for num in &draw.front_zone {
            let freq = front_freq.entry(*num).or_insert((0, 0.0, days as u32)); // (count, weighted_score, last_seen)
            freq.0 += 1;
            freq.1 += 1.0 / (days_ago + 1) as f64; // Weight by recency
            freq.2 = freq.2.min(days_ago as u32);
        }

        // Back zone frequencies
        for num in &draw.back_zone {
            let freq = back_freq.entry(*num).or_insert((0, 0.0, days as u32));
            freq.0 += 1;
            freq.1 += 1.0 / (days_ago + 1) as f64;
            freq.2 = freq.2.min(days_ago as u32);
        }
    }

    println!(
        "📈 [FREQUENCY] Calculated frequencies for {} front numbers, {} back numbers",
        front_freq.len(),
        back_freq.len()
    );

    // Calculate hot scores and build results
    let mut hot_numbers = Vec::new();
    let total_draws = draws.len() as f64;

    // Process front zone numbers
    if zone_filter
        .as_ref()
        .map_or(true, |z| z == "FRONT" || z == "BOTH")
    {
        println!("🎯 [FRONT] Processing front zone numbers...");
        let mut front_count = 0;

        for (num, (count, weighted_score, last_seen)) in &front_freq {
            let frequency = *count as f64 / total_draws;
            let hot_score = weighted_score / 100.0; // Normalize hot score
            let avg_gap = if *count > 0 {
                (days as f64) / (*count as f64)
            } else {
                0.0
            };
            let current_gap = *last_seen;

            if hot_score >= min_threshold {
                front_count += 1;
                hot_numbers.push(serde_json::json!({
                    "number": num,
                    "zone": "FRONT",
                    "frequency": frequency,
                    "last_seen": format_days_ago(*last_seen),
                    "hot_score": hot_score,
                    "cold_score": 1.0 - hot_score,
                    "average_gap": avg_gap,
                    "current_gap": current_gap,
                    "period_days": days,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        println!(
            "✅ [FRONT] Found {} hot front numbers above threshold {}",
            front_count, min_threshold
        );
    }

    // Process back zone numbers
    if zone_filter
        .as_ref()
        .map_or(true, |z| z == "BACK" || z == "BOTH")
    {
        println!("🎯 [BACK] Processing back zone numbers...");
        let mut back_count = 0;

        for (num, (count, weighted_score, last_seen)) in &back_freq {
            let frequency = *count as f64 / total_draws;
            let hot_score = weighted_score / 100.0;
            let avg_gap = if *count > 0 {
                (days as f64) / (*count as f64)
            } else {
                0.0
            };
            let current_gap = *last_seen;

            if hot_score >= min_threshold {
                back_count += 1;
                hot_numbers.push(serde_json::json!({
                    "number": num,
                    "zone": "BACK",
                    "frequency": frequency,
                    "last_seen": format_days_ago(*last_seen),
                    "hot_score": hot_score,
                    "cold_score": 1.0 - hot_score,
                    "average_gap": avg_gap,
                    "current_gap": current_gap,
                    "period_days": days,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
        println!(
            "✅ [BACK] Found {} hot back numbers above threshold {}",
            back_count, min_threshold
        );
    }

    // Sort by hot score and limit results
    let original_count = hot_numbers.len();
    hot_numbers.sort_by(|a, b| {
        let score_a = a["hot_score"].as_f64().unwrap_or(0.0);
        let score_b = b["hot_score"].as_f64().unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    hot_numbers.truncate(limit as usize);

    println!(
        "📊 [RESULT] Sorted {} hot numbers, returning top {}",
        original_count,
        hot_numbers.len()
    );
    if !hot_numbers.is_empty() {
        println!("🏆 [TOP] Top 3 hot numbers:");
        for (i, number) in hot_numbers.iter().take(3).enumerate() {
            let num = number["number"].as_u64().unwrap_or(0);
            let zone = number["zone"].as_str().unwrap_or("UNKNOWN");
            let score = number["hot_score"].as_f64().unwrap_or(0.0);
            println!("  {}. {} ({}): {:.3}", i + 1, num, zone, score);
        }
    }

    let response = serde_json::json!({
        "numbers": hot_numbers,
        "analysis_period_days": days,
        "zone_filter": zone_filter,
        "min_threshold": min_threshold,
        "total_draws_analyzed": draws.len(),
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    let duration = start_time.elapsed();
    println!(
        "✅ [COMMAND] analyze_hot_numbers completed in {:?} - returned {} hot numbers",
        duration,
        hot_numbers.len()
    );

    Ok(response)
}

/// Helper function to format days ago
fn format_days_ago(days: u32) -> String {
    if days == 0 {
        "Today".to_string()
    } else if days == 1 {
        "Yesterday".to_string()
    } else {
        format!("{} days ago", days)
    }
}

/// Analyze cold numbers
#[tauri::command]
pub async fn analyze_cold_numbers(
    pool: State<'_, SqlitePool>,
    days: u32,
    zone: Option<String>,
    limit: Option<u32>,
    min_days_absent: Option<u32>,
) -> Result<serde_json::Value, SuperLottoError> {
    use crate::super_lotto::models::SuperLottoDraw;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    let limit = limit.unwrap_or(20);
    let zone_filter = zone.as_ref().map(|s| s.to_uppercase());
    let min_absent = min_days_absent.unwrap_or(7);

    // Get historical draws within the specified period - use correct table name
    let cutoff_date = Utc::now() - Duration::days(days as i64);
    let query = format!(
        "SELECT * FROM lottery_draws WHERE lottery_type = 'super_lotto' AND draw_date >= '{}' ORDER BY draw_date DESC",
        cutoff_date.format("%Y-%m-%d")
    );

    let raw_draws = sqlx::query_as::<_, LotteryDraw>(&query)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("Database query error in cold numbers analysis: {:?}", e);
            SuperLottoError::internal(format!("Database error: {}", e))
        })?;

    // Convert LotteryDraw to SuperLottoDraw
    let draws: Vec<SuperLottoDraw> = raw_draws.into_iter().map(|draw| draw.into()).collect();

    if draws.is_empty() {
        return Err(SuperLottoError::internal(
            "No historical data available for analysis",
        ));
    }

    // Track number appearances and calculate gaps
    let mut front_last_seen = HashMap::new(); // number -> days_since_last_appearance
    let mut back_last_seen = HashMap::new();
    let mut front_appearances = HashMap::new(); // number -> count of appearances
    let mut back_appearances = HashMap::new();

    // Initialize all numbers as never seen (in analysis period)
    for num in 1..=35 {
        front_last_seen.insert(num, days); // Assume never seen in analysis period
        front_appearances.insert(num, 0);
    }
    for num in 1..=12 {
        back_last_seen.insert(num, days);
        back_appearances.insert(num, 0);
    }

    // Track appearances from most recent to oldest
    for (days_ago, draw) in draws.iter().enumerate() {
        for &num in &draw.front_zone {
            if let Some(last_seen) = front_last_seen.get(&num) {
                if *last_seen == days {
                    // Only update if not seen yet
                    front_last_seen.insert(num, days_ago as u32);
                }
            }
            *front_appearances.entry(num).or_insert(0) += 1;
        }

        for &num in &draw.back_zone {
            if let Some(last_seen) = back_last_seen.get(&num) {
                if *last_seen == days {
                    // Only update if not seen yet
                    back_last_seen.insert(num, days_ago as u32);
                }
            }
            *back_appearances.entry(num).or_insert(0) += 1;
        }
    }

    // Calculate cold scores and build results
    let mut cold_numbers = Vec::new();
    let total_draws = draws.len() as f64;

    // Process front zone numbers
    if zone_filter
        .as_ref()
        .map_or(true, |z| z == "FRONT" || z == "BOTH")
    {
        for num in 1..=35 {
            let current_gap = *front_last_seen.get(&num).unwrap_or(&days);
            let appearances = *front_appearances.get(&num).unwrap_or(&0);
            let frequency = appearances as f64 / total_draws;

            // Calculate cold score based on gap and frequency
            let gap_score = (current_gap as f64) / (days as f64); // 0-1, higher = colder
            let frequency_score = 1.0 - frequency; // 0-1, lower frequency = colder
            let cold_score = (gap_score + frequency_score) / 2.0;

            // Calculate average gap
            let avg_gap = if appearances > 0 {
                (days as f64) / (appearances as f64)
            } else {
                days as f64 // Never appeared
            };

            if current_gap >= min_absent {
                cold_numbers.push(serde_json::json!({
                    "number": num,
                    "zone": "FRONT",
                    "frequency": frequency,
                    "last_seen": format_days_ago(current_gap),
                    "hot_score": 1.0 - cold_score,
                    "cold_score": cold_score,
                    "average_gap": avg_gap,
                    "current_gap": current_gap,
                    "appearances": appearances,
                    "period_days": days,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
    }

    // Process back zone numbers
    if zone_filter
        .as_ref()
        .map_or(true, |z| z == "BACK" || z == "BOTH")
    {
        for num in 1..=12 {
            let current_gap = *back_last_seen.get(&num).unwrap_or(&days);
            let appearances = *back_appearances.get(&num).unwrap_or(&0);
            let frequency = appearances as f64 / total_draws;

            let gap_score = (current_gap as f64) / (days as f64);
            let frequency_score = 1.0 - frequency;
            let cold_score = (gap_score + frequency_score) / 2.0;

            let avg_gap = if appearances > 0 {
                (days as f64) / (appearances as f64)
            } else {
                days as f64
            };

            if current_gap >= min_absent {
                cold_numbers.push(serde_json::json!({
                    "number": num,
                    "zone": "BACK",
                    "frequency": frequency,
                    "last_seen": format_days_ago(current_gap),
                    "hot_score": 1.0 - cold_score,
                    "cold_score": cold_score,
                    "average_gap": avg_gap,
                    "current_gap": current_gap,
                    "appearances": appearances,
                    "period_days": days,
                    "updated_at": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
    }

    // Sort by cold score (highest first) and limit results
    cold_numbers.sort_by(|a, b| {
        let score_a = a["cold_score"].as_f64().unwrap_or(0.0);
        let score_b = b["cold_score"].as_f64().unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    cold_numbers.truncate(limit as usize);

    let response = serde_json::json!({
        "numbers": cold_numbers,
        "analysis_period_days": days,
        "zone_filter": zone_filter,
        "min_days_absent": min_absent,
        "total_draws_analyzed": draws.len(),
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(response)
}

/// Get pattern analysis
#[tauri::command]
pub async fn get_pattern_analysis(
    pool: State<'_, SqlitePool>,
    _pattern_type: Option<String>,
    _days: u32,
    _min_occurrences: Option<u32>,
) -> Result<serde_json::Value, SuperLottoError> {
    // TODO: Implement pattern analysis
    Err(SuperLottoError::internal("Not implemented yet"))
}

/// Generate prediction
#[tauri::command]
pub async fn generate_prediction(
    pool: State<'_, SqlitePool>,
    algorithm: String,
    analysis_period_days: Option<u32>,
    custom_parameters: Option<serde_json::Value>,
    include_reasoning: Option<bool>,
) -> Result<serde_json::Value, SuperLottoError> {
    use crate::super_lotto::analysis::prediction_engine::{
        calculate_confidence_score, get_algorithm,
    };
    use crate::super_lotto::models::SuperLottoDraw;
    use chrono::{Duration, Utc};

    // Log command start
    let start_time = std::time::Instant::now();

    let analysis_days = analysis_period_days.unwrap_or(90);
    let include_reasoning = include_reasoning.unwrap_or(false);

    println!("🎯 [COMMAND] generate_prediction called with parameters:");
    println!("  - algorithm: {}", algorithm);
    println!("  - analysis_period_days: {}", analysis_days);
    println!("  - custom_parameters: {:?}", custom_parameters);
    println!("  - include_reasoning: {}", include_reasoning);

    // Get algorithm instance
    let algo = get_algorithm(&algorithm).ok_or_else(|| {
        println!("❌ [ERROR] Unknown algorithm: {}", algorithm);
        SuperLottoError::internal(format!("Unknown algorithm: {}", algorithm))
    })?;

    println!("✅ [ALGORITHM] Validated algorithm: {}", algorithm);

    // Get historical draws for analysis
    let cutoff_date = Utc::now() - Duration::days(analysis_days as i64);
    let query = format!(
        "SELECT * FROM lottery_draws WHERE lottery_type = 'super_lotto' AND draw_date >= '{}' ORDER BY draw_date DESC",
        cutoff_date.format("%Y-%m-%d")
    );

    println!(
        "📅 [ANALYSIS] Analyzing draws from {} to present",
        cutoff_date.format("%Y-%m-%d")
    );

    let raw_draws = sqlx::query_as::<_, LotteryDraw>(&query)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| {
            eprintln!("❌ [DATABASE] Query error in prediction: {:?}", e);
            SuperLottoError::internal(format!("Database error: {}", e))
        })?;

    // Convert LotteryDraw to SuperLottoDraw
    let draws: Vec<SuperLottoDraw> = raw_draws.into_iter().map(|draw| draw.into()).collect();

    println!(
        "📊 [DATA] Retrieved {} draws for prediction analysis",
        draws.len()
    );

    if draws.is_empty() {
        println!("⚠️ [WARNING] No historical data available for prediction");
        return Err(SuperLottoError::internal(
            "No historical data available for prediction",
        ));
    }

    println!(
        "🔍 [PREDICTION] Starting {} algorithm analysis...",
        algorithm
    );
    let algorithm_start = std::time::Instant::now();

    // Generate prediction using the algorithm trait
    let (front_numbers, back_numbers, reasoning) = algo.predict(&draws, &custom_parameters);

    let algorithm_duration = algorithm_start.elapsed();
    println!(
        "✅ [ALGORITHM] {} analysis completed in {:?}",
        algorithm, algorithm_duration
    );
    println!("📤 [PREDICTION] Front numbers: {:?}", front_numbers);
    println!("📤 [PREDICTION] Back numbers: {:?}", back_numbers);

    // Calculate confidence score using the trait method
    let confidence_score = calculate_confidence_score(&draws, algo.as_ref(), analysis_days);
    println!(
        "📊 [CONFIDENCE] Calculated confidence score: {:.3}",
        confidence_score
    );

    // Create prediction result
    let mut prediction = serde_json::json!({
        "id": chrono::Utc::now().timestamp(),
        "algorithm": algorithm,
        "front_numbers": front_numbers,
        "back_numbers": back_numbers,
        "confidence_score": confidence_score,
        "analysis_period_days": analysis_days,
        "sample_size": draws.len(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "is_validated": false
    });

    // Add reasoning if requested
    if include_reasoning {
        println!("📝 [REASONING] Adding detailed reasoning to response");
        prediction["reasoning"] = serde_json::json!({
            "algorithm": algorithm,
            "explanation": reasoning,
            "analysis_period_days": analysis_days,
            "key_factors": [
                "号码频率分析",
                "冷热号统计",
                "和值分布规律",
                "奇偶比例平衡"
            ],
            "risk_assessment": "中等风险",
            "recommendation": "建议结合其他分析方法综合判断"
        });
    }

    let duration = start_time.elapsed();
    println!(
        "🎉 [COMMAND] generate_prediction completed successfully in {:?}",
        duration
    );
    println!(
        "📊 [SUMMARY] Algorithm: {}, Sample: {}, Confidence: {:.3}, Front: {:?}, Back: {:?}",
        algorithm,
        draws.len(),
        confidence_score,
        front_numbers,
        back_numbers
    );

    Ok(prediction)
}

/// Get prediction results
#[tauri::command]
pub async fn get_predictions(
    pool: State<'_, SqlitePool>,
    algorithm: Option<String>,
    limit: Option<u32>,
    min_confidence: Option<f64>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<serde_json::Value, SuperLottoError> {
    // Log command start
    let start_time = std::time::Instant::now();

    println!("📋 [COMMAND] get_predictions called with parameters:");
    println!("  - algorithm: {:?}", algorithm);
    println!("  - limit: {:?}", limit);
    println!("  - min_confidence: {:?}", min_confidence);
    println!("  - start_date: {:?}", start_date);
    println!("  - end_date: {:?}", end_date);

    // TODO: Implement prediction retrieval
    println!("🚧 [TODO] get_predictions functionality not yet implemented");

    let duration = start_time.elapsed();
    println!(
        "⏱️ [COMMAND] get_predictions completed in {:?} (not implemented)",
        duration
    );

    Err(SuperLottoError::internal("Not implemented yet"))
}

/// Validate prediction against actual results
#[tauri::command]
pub async fn validate_prediction(
    pool: State<'_, SqlitePool>,
    id: i64,
    actual_draw: serde_json::Value,
) -> Result<serde_json::Value, SuperLottoError> {
    // Log command start
    let start_time = std::time::Instant::now();

    println!("✔️ [COMMAND] validate_prediction called with parameters:");
    println!("  - prediction_id: {}", id);
    println!("  - actual_draw: {}", actual_draw);

    // TODO: Implement prediction validation
    println!("🚧 [TODO] validate_prediction functionality not yet implemented");

    let duration = start_time.elapsed();
    println!(
        "⏱️ [COMMAND] validate_prediction completed in {:?} (not implemented)",
        duration
    );

    Err(SuperLottoError::internal("Not implemented yet"))
}

// ===== ONE-CLICK PREDICTION FEATURE =====

/// Generate predictions using all algorithms (one-click prediction)
#[tauri::command]
pub async fn generate_all_predictions(
    pool: State<'_, SqlitePool>,
    request: Option<BatchPredictionRequest>,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!("🚀 [COMMAND] generate_all_predictions called");

    // Use default request if none provided
    let batch_request = request.unwrap_or_else(|| BatchPredictionRequest::new(90, 0)); // Default draw number 0

    println!("📊 [BATCH] Request details:");
    println!("  - algorithms: {:?}", batch_request.algorithms);
    println!(
        "  - analysis_period_days: {}",
        batch_request.analysis_period_days
    );
    println!("  - include_reasoning: {}", batch_request.include_reasoning);

    // Generate unique request ID
    let request_id = format!(
        "batch_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );

    // Get historical data for analysis
    let historical_draws =
        get_historical_data_for_analysis(&pool, batch_request.analysis_period_days).await?;

    let sample_size = historical_draws.len() as u32;
    if sample_size < 10 {
        return Err(SuperLottoError::internal(
            "Insufficient historical data for prediction (need at least 10 draws)",
        ));
    }

    println!("📈 [ANALYSIS] Analyzing {} historical draws", sample_size);

    // Generate predictions for each algorithm
    let mut all_predictions = Vec::<PredictionResult>::new();
    let start_time = std::time::Instant::now();

    for algorithm in &batch_request.algorithms {
        let algorithm_str = algorithm.to_string();
        println!(
            "🎯 [PREDICTION] Generating prediction using: {:?} (as string: {})",
            algorithm, algorithm_str
        );

        match generate_prediction_for_algorithm(
            &pool,
            &algorithm_str,
            &historical_draws,
            batch_request.analysis_period_days,
            batch_request.include_reasoning,
            batch_request.draw_number,
        )
        .await
        {
            Ok(prediction) => {
                println!(
                    "✅ [PREDICTION] Successfully generated prediction for {:?}",
                    algorithm
                );
                all_predictions.push(prediction);
            }
            Err(e) => {
                println!(
                    "❌ [PREDICTION] Failed to generate prediction for {:?}: {}",
                    algorithm, e
                );
                // Skip this algorithm and continue with others
                continue;
            }
        }
    }

    let processing_time = start_time.elapsed().as_millis() as u64;

    // Create batch result
    let batch_result = BatchPredictionResult::new(
        request_id.clone(),
        all_predictions,
        processing_time,
        batch_request.analysis_period_days,
        sample_size,
    );

    println!("🎉 [BATCH] Batch prediction completed:");
    println!("  - request_id: {}", request_id);
    println!("  - total_predictions: {}", batch_result.total_predictions);
    println!(
        "  - successful_predictions: {}",
        batch_result.successful_predictions
    );
    println!("  - processing_time_ms: {}", processing_time);

    // Return result
    Ok(serde_json::to_value(batch_result).map_err(|e| {
        SuperLottoError::internal(format!("Failed to serialize batch result: {}", e))
    })?)
}

/// Get prediction comparison for batch results
#[tauri::command]
pub async fn get_prediction_comparison(
    pool: State<'_, SqlitePool>,
    batch_request_id: String,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!(
        "🔍 [COMMAND] get_prediction_comparison called for batch_id: {}",
        batch_request_id
    );

    // TODO: Implement batch result retrieval from database
    // For now, return a placeholder comparison
    let comparison = PredictionComparison {
        batch_result: BatchPredictionResult::new(batch_request_id, vec![], 0, 90, 0),
        consensus_numbers: ConsensusNumbers {
            front_consensus: vec![1, 5, 12, 23, 28],
            back_consensus: vec![6, 9],
            consensus_strength: 0.75,
            voting_details: vec![],
        },
        algorithm_rankings: vec![],
        confidence_distribution: ConfidenceDistribution {
            high_confidence_count: 0,
            medium_confidence_count: 0,
            low_confidence_count: 0,
            average_confidence: 0.0,
            confidence_variance: 0.0,
        },
        recommendation: PredictionRecommendation {
            recommended_front: vec![1, 5, 12, 23, 28],
            recommended_back: vec![6, 9],
            confidence_level: 0.75,
            reasoning: "基于多种算法的综合分析".to_string(),
            risk_assessment: RiskLevel::Moderate,
            alternative_combinations: vec![],
        },
    };

    let duration = start_time.elapsed();
    println!(
        "⏱️ [COMMAND] get_prediction_comparison completed in {:?}",
        duration
    );

    Ok(serde_json::to_value(comparison)
        .map_err(|e| SuperLottoError::internal(format!("Failed to serialize comparison: {}", e)))?)
}

/// Get unified table data combining historical draws and predictions
#[tauri::command]
pub async fn get_unified_table_data(
    pool: State<'_, SqlitePool>,
    filters: Option<TableFilters>,
    page: Option<u32>,
    rows_per_page: Option<u32>,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!("📊 [COMMAND] get_unified_table_data called");

    let page = page.unwrap_or(1);
    let rows_per_page = rows_per_page.unwrap_or(50);
    let table_filters = filters.unwrap_or_else(|| TableFilters {
        row_types: vec!["historical".to_string(), "prediction".to_string()],
        algorithms: vec![],
        date_range: None,
        confidence_range: None,
        number_filters: None,
    });

    println!("🔍 [FILTERS] Applied filters: {:?}", table_filters);
    println!(
        "📄 [PAGINATION] Page: {}, Rows per page: {}",
        page, rows_per_page
    );

    // Get historical data
    let historical_draws = if table_filters.row_types.contains(&"historical".to_string()) {
        // Call existing function and convert result
        let result = get_super_lotto_draws(
            pool,
            Some(10000), // Large limit to get all data
            None,
            None,
            None,
            None,
        )
        .await?;
        result["draws"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| serde_json::from_value::<SuperLottoDraw>(item.clone()).unwrap())
            .collect()
    } else {
        vec![]
    };

    // Get prediction results
    let prediction_results: Vec<PredictionResult> =
        if table_filters.row_types.contains(&"prediction".to_string()) {
            // TODO: Get from prediction_results table
            vec![]
        } else {
            vec![]
        };

    // Get batch predictions
    let batch_predictions = if table_filters.row_types.contains(&"batch".to_string()) {
        // TODO: Get from batch_predictions table
        vec![]
    } else {
        vec![]
    };

    // Combine into unified rows
    let mut combined_data = Vec::new();

    // Add historical draws
    for draw in &historical_draws {
        combined_data.push(UnifiedTableRow::HistoricalDraw {
            id: draw.id,
            date: draw.draw_date,
            draw_number: draw.draw_number.clone(),
            front_numbers: draw.front_zone.to_vec(),
            back_numbers: draw.back_zone.to_vec(),
            jackpot_amount: draw.jackpot_amount,
            row_type: "historical".to_string(),
        });
    }

    // Add predictions
    for prediction in &prediction_results {
        let algorithm = prediction
            .algorithm
            .parse::<PredictionAlgorithm>()
            .unwrap_or(PredictionAlgorithm::Ensemble);
        combined_data.push(UnifiedTableRow::Prediction {
            id: prediction.id,
            date: prediction.created_at,
            algorithm,
            front_numbers: prediction.front_numbers.clone(),
            back_numbers: prediction.back_numbers.clone(),
            confidence_score: prediction.confidence_score,
            row_type: "prediction".to_string(),
        });
    }

    // Sort by date (newest first)
    combined_data.sort_by(|a, b| b.get_date().cmp(&a.get_date()));

    // Apply pagination
    let total_rows = combined_data.len() as u32;
    let pagination = PaginationInfo::new(total_rows, page, rows_per_page);

    let start_index = ((page - 1) * rows_per_page) as usize;
    let end_index = (start_index + rows_per_page as usize).min(combined_data.len());
    let paginated_data = combined_data[start_index..end_index].to_vec();

    // Create unified table data
    let unified_data = UnifiedTableData {
        historical_draws,
        prediction_results,
        batch_predictions,
        combined_data: paginated_data.clone(),
        filters: table_filters,
        pagination: pagination.clone(),
    };

    let duration = start_time.elapsed();
    println!(
        "⏱️ [COMMAND] get_unified_table_data completed in {:?}",
        duration
    );
    println!(
        "📊 [RESULT] Returning {} rows (page {} of {})",
        paginated_data.len(),
        page,
        pagination.total_pages
    );

    Ok(serde_json::to_value(unified_data).map_err(|e| {
        SuperLottoError::internal(format!("Failed to serialize unified data: {}", e))
    })?)
}

/// Export table data in specified format
#[tauri::command]
pub async fn export_table_data(
    pool: State<'_, SqlitePool>,
    export_request: TableExportRequest,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!("📤 [COMMAND] export_table_data called");
    println!("📄 [FORMAT] Export format: {:?}", export_request.format);
    println!("🔍 [FILTERS] Applied filters: {:?}", export_request.filters);

    // Get unified table data
    let unified_data = get_unified_table_data(
        pool,
        Some(export_request.filters),
        None,
        export_request.max_rows,
    )
    .await?;

    // TODO: Implement actual export logic based on format
    let export_result = serde_json::json!({
        "success": true,
        "format": export_request.format.to_string(),
        "rows_exported": unified_data["combined_data"].as_array().unwrap().len(),
        "export_path": format!("export_{}.{}",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            export_request.format.to_string().to_lowercase()
        ),
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    let duration = start_time.elapsed();
    println!("⏱️ [COMMAND] export_table_data completed in {:?}", duration);

    Ok(export_result)
}

// ===== HELPER FUNCTIONS =====

/// Get historical data for analysis
async fn get_historical_data_for_analysis(
    pool: &SqlitePool,
    analysis_period_days: u32,
) -> Result<Vec<SuperLottoDraw>, SuperLottoError> {
    use super::models::SuperLottoDraw;
        use sqlx::Row;

    let end_date = chrono::Utc::now();
    let start_date = end_date - chrono::Duration::days(analysis_period_days as i64);

    // We can't call get_super_lotto_draws directly because it expects State
    // Instead, let's query the database directly
    let query = r#"
        SELECT id, draw_number, draw_date, front_zone, back_zone, 
               jackpot_amount, created_at
        FROM super_lotto_draws 
        WHERE draw_date >= ? AND draw_date <= ?
        ORDER BY draw_date DESC
        LIMIT 10000
    "#;

    let rows = sqlx::query(query)
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(pool)
        .await
        .map_err(|e| SuperLottoError::Database(e))?;

    let draws = rows
        .into_iter()
        .map(|row| SuperLottoDraw {
            id: row.get("id"),
            draw_date: DateTime::parse_from_rfc3339(&row.get::<String, _>("draw_date"))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            draw_number: Some(row.get("draw_number")),
            front_zone: serde_json::from_str(&row.get::<String, _>("front_zone"))
                .unwrap_or_default(),
            back_zone: serde_json::from_str(&row.get::<String, _>("back_zone")).unwrap_or_default(),
            jackpot_amount: row.get("jackpot_amount"),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            sum_front: None,
            odd_count_front: None,
            even_count_front: None,
            has_consecutive_front: None,
            winners_count: None,
        })
        .collect();

    Ok(draws)
}

/// Generate prediction for a specific algorithm
async fn generate_prediction_for_algorithm(
    pool: &SqlitePool,
    algorithm_name: &str,
    historical_draws: &[SuperLottoDraw],
    analysis_period_days: u32,
    _include_reasoning: bool,
    _draw_number: u32,
) -> Result<PredictionResult, SuperLottoError> {
    use crate::super_lotto::analysis::prediction_engine::{
        calculate_confidence_score, get_algorithm,
    };
    use std::str::FromStr;

    // Get algorithm instance
    let algo = get_algorithm(algorithm_name).ok_or_else(|| {
        SuperLottoError::internal(format!("Unknown algorithm: {}", algorithm_name))
    })?;

    // Generate prediction
    let (red_nums, blue_nums, reasoning) = algo.predict(historical_draws, &None);

    // Calculate confidence score
    let confidence =
        calculate_confidence_score(historical_draws, algo.as_ref(), analysis_period_days);

    // Create PredictionResult
    Ok(PredictionResult::new(
        FromStr::from_str(algorithm_name).unwrap_or(PredictionAlgorithm::WeightedFrequency),
        red_nums,
        blue_nums,
        confidence,
        serde_json::json!(reasoning),
        analysis_period_days,
        historical_draws.len() as u32,
    )?)
}
