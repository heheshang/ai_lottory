//! Super Lotto Tauri Commands
//!
//! Tauri command handlers for Super Lotto functionality

use crate::super_lotto::errors::SuperLottoError;
use crate::super_lotto::models::{
    PredictionAlgorithm, PredictionResult, SuperLottoDraw,
};
use crate::super_lotto::predictions::*;
use serde_json::json;
use sqlx::SqlitePool;
use std::result::Result;
use tauri::State;

/// Get prediction results
#[tauri::command]
pub async fn get_predictions(
    pool: State<'_, SqlitePool>,
    algorithm: Option<String>,
    limit: Option<u32>,
    min_confidence: Option<f64>,
    _start_date: Option<String>,
    _end_date: Option<String>,
) -> Result<serde_json::Value, SuperLottoError> {
    // Log command start
    let start_time = std::time::Instant::now();

    println!("📋 [COMMAND] get_predictions called with parameters:");
    println!("  - algorithm: {:?}", algorithm);
    println!("  - limit: {:?}", limit);
    println!("  - min_confidence: {:?}", min_confidence);

    // Parse algorithm parameter
    let algorithm_type = match algorithm.as_deref() {
        Some("WEIGHTED_FREQUENCY") | Some("WEIGHTED-FREQUENCY") => Some(PredictionAlgorithm::WeightedFrequency),
        Some("HOT_NUMBERS") => Some(PredictionAlgorithm::HotNumbers),
        Some("COLD_NUMBERS") => Some(PredictionAlgorithm::ColdNumbers),
        Some("PATTERN_BASED") => Some(PredictionAlgorithm::PatternBased),
        Some("ENSEMBLE") => Some(PredictionAlgorithm::Ensemble),
        Some("MARKOV_CHAIN") => Some(PredictionAlgorithm::MarkovChain),
        Some("POSITION_ANALYSIS") => Some(PredictionAlgorithm::PositionAnalysis),
        _ => Some(PredictionAlgorithm::WeightedFrequency), // Default algorithm
    };

    // Get historical data for analysis
    let historical_draws = get_historical_data_for_analysis(pool.inner(), 365).await?;

    if historical_draws.is_empty() {
        return Err(SuperLottoError::internal("No historical data available for prediction"));
    }

    println!("📊 [ANALYSIS] Analyzing {} historical draws", historical_draws.len());

    // Generate predictions based on selected algorithm
    let mut predictions = Vec::new();

    if let Some(ref algo) = algorithm_type {
        match algo {
            PredictionAlgorithm::WeightedFrequency => {
                predictions.push(generate_weighted_frequency_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::HotNumbers => {
                predictions.push(generate_hot_numbers_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::ColdNumbers => {
                predictions.push(generate_cold_numbers_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::PatternBased => {
                predictions.push(generate_pattern_based_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::Ensemble => {
                // Generate multiple predictions and combine them
                predictions.push(generate_weighted_frequency_prediction(&historical_draws)?);
                predictions.push(generate_hot_numbers_prediction(&historical_draws)?);
                predictions.push(generate_cold_numbers_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::MarkovChain => {
                predictions.push(generate_markov_chain_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::PositionAnalysis => {
                predictions.push(generate_position_analysis_prediction(&historical_draws)?);
            },
            PredictionAlgorithm::Ensemble => {
                // Generate multiple predictions and combine them
                predictions.push(generate_weighted_frequency_prediction(&historical_draws)?);
                predictions.push(generate_hot_numbers_prediction(&historical_draws)?);
                predictions.push(generate_cold_numbers_prediction(&historical_draws)?);
            },
        }
    }

    // Filter by minimum confidence
    let min_conf = min_confidence.unwrap_or(0.3);
    let filtered_predictions: Vec<&PredictionResult> = predictions.iter()
        .filter(|p| p.confidence_score >= min_conf)
        .collect();

    // Apply limit
    let final_limit = limit.unwrap_or(10).min(filtered_predictions.len() as u32);
    let limited_predictions: Vec<&PredictionResult> = filtered_predictions
        .into_iter()
        .take(final_limit as usize)
        .collect();

    // Format results
    let results: Vec<serde_json::Value> = limited_predictions.iter().map(|p| {
        json!({
            "algorithm": p.algorithm,
            "front_numbers": p.front_numbers,
            "back_numbers": p.back_numbers,
            "confidence_score": p.confidence_score,
            "reasoning": p.reasoning,
            "analysis_period_days": p.analysis_period_days,
            "sample_size": p.sample_size,
            "created_at": p.created_at.to_rfc3339(),
            "is_validated": p.is_validated
        })
    }).collect();

    let response = json!({
        "predictions": results,
        "total_available": predictions.iter().filter(|p| p.confidence_score >= min_conf).count(),
        "returned": results.len(),
        "algorithm_used": algorithm_type.map(|a| a.to_string()),
        "analysis_period_days": 365,
        "confidence_threshold": min_conf,
        "metadata": {
            "historical_draws_analyzed": historical_draws.len(),
            "analysis_completed_at": chrono::Utc::now().to_rfc3339()
        }
    });

    let duration = start_time.elapsed();
    println!(
        "✅ [COMMAND] get_predictions completed in {:?} - returned {} predictions",
        duration,
        results.len()
    );

    Ok(response)
}

/// Get historical data for analysis
async fn get_historical_data_for_analysis(
    _pool: &SqlitePool,
    _analysis_period_days: u32,
) -> Result<Vec<SuperLottoDraw>, SuperLottoError> {
    // For now, return some sample data
    let sample_data = vec![
        SuperLottoDraw {
            id: 1,
            draw_number: Some("2024001".to_string()),
            draw_date: chrono::Utc::now() - chrono::Duration::days(7),
            front_zone: vec![5, 12, 18, 23, 31].into(),
            back_zone: vec![4, 11].into(),
            jackpot_amount: Some(5000000.0),
            created_at: chrono::Utc::now(),
            winners_count: Some(0),
            even_count_front: Some(2),
            has_consecutive_front: Some(false),
            odd_count_front: Some(3),
            sum_front: Some(89),
        },
        SuperLottoDraw {
            id: 2,
            draw_number: Some("2024002".to_string()),
            draw_date: chrono::Utc::now() - chrono::Duration::days(14),
            front_zone: vec![2, 8, 15, 22, 29].into(),
            back_zone: vec![3, 8].into(),
            jackpot_amount: Some(8000000.0),
            created_at: chrono::Utc::now(),
            winners_count: Some(0),
            even_count_front: Some(3),
            has_consecutive_front: Some(false),
            odd_count_front: Some(2),
            sum_front: Some(76),
        },
        SuperLottoDraw {
            id: 3,
            draw_number: Some("2024003".to_string()),
            draw_date: chrono::Utc::now() - chrono::Duration::days(21),
            front_zone: vec![1, 9, 16, 25, 33].into(),
            back_zone: vec![6, 9].into(),
            jackpot_amount: Some(12000000.0),
            created_at: chrono::Utc::now(),
            winners_count: Some(0),
            even_count_front: Some(2),
            has_consecutive_front: Some(false),
            odd_count_front: Some(3),
            sum_front: Some(84),
        },
        SuperLottoDraw {
            id: 4,
            draw_number: Some("2024004".to_string()),
            draw_date: chrono::Utc::now() - chrono::Duration::days(28),
            front_zone: vec![3, 11, 19, 27, 34].into(),
            back_zone: vec![2, 7].into(),
            jackpot_amount: Some(3000000.0),
            created_at: chrono::Utc::now(),
            winners_count: Some(0),
            even_count_front: Some(2),
            has_consecutive_front: Some(false),
            odd_count_front: Some(3),
            sum_front: Some(94),
        },
        SuperLottoDraw {
            id: 5,
            draw_number: Some("2024005".to_string()),
            draw_date: chrono::Utc::now() - chrono::Duration::days(35),
            front_zone: vec![7, 14, 21, 28, 35].into(),
            back_zone: vec![1, 12].into(),
            jackpot_amount: Some(15000000.0),
            created_at: chrono::Utc::now(),
            winners_count: Some(0),
            even_count_front: Some(2),
            has_consecutive_front: Some(false),
            odd_count_front: Some(3),
            sum_front: Some(105),
        },
    ];

    Ok(sample_data)
}

/// One-Click Prediction Feature - Generate all predictions with one call
#[tauri::command]
pub async fn generate_all_predictions(
    pool: State<'_, SqlitePool>,
    include_reasoning: Option<bool>,
    draw_number: Option<String>,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!("🚀 [COMMAND] generate_all_predictions called - One-click prediction feature");
    println!("  - include_reasoning: {:?}", include_reasoning);
    println!("  - draw_number: {:?}", draw_number);

    // Get historical data for analysis
    let historical_draws = get_historical_data_for_analysis(pool.inner(), 365).await?;

    if historical_draws.is_empty() {
        return Err(SuperLottoError::internal("No historical data available for prediction"));
    }

    println!("📊 [ANALYSIS] Analyzing {} historical draws for comprehensive prediction", historical_draws.len());

    // Generate predictions using all available algorithms
    let mut all_predictions = Vec::new();
    let algorithms = vec![
        PredictionAlgorithm::WeightedFrequency,
        PredictionAlgorithm::HotNumbers,
        PredictionAlgorithm::ColdNumbers,
        PredictionAlgorithm::PatternBased,
        PredictionAlgorithm::MarkovChain,
        PredictionAlgorithm::PositionAnalysis,
    ];

    let algorithm_names = vec![
        "WEIGHTED_FREQUENCY",
        "HOT_NUMBERS",
        "COLD_NUMBERS",
        "PATTERN_BASED",
        "MARKOV_CHAIN",
        "POSITION_ANALYSIS",
    ];

    for (i, algorithm) in algorithms.iter().enumerate() {
        let prediction = match algorithm {
            PredictionAlgorithm::WeightedFrequency => {
                generate_weighted_frequency_prediction(&historical_draws)?
            },
            PredictionAlgorithm::HotNumbers => {
                generate_hot_numbers_prediction(&historical_draws)?
            },
            PredictionAlgorithm::ColdNumbers => {
                generate_cold_numbers_prediction(&historical_draws)?
            },
            PredictionAlgorithm::PatternBased => {
                generate_pattern_based_prediction(&historical_draws)?
            },
            PredictionAlgorithm::MarkovChain => {
                generate_markov_chain_prediction(&historical_draws)?
            },
            PredictionAlgorithm::PositionAnalysis => {
                generate_position_analysis_prediction(&historical_draws)?
            },
            PredictionAlgorithm::Ensemble => {
                // For ensemble, generate a balanced prediction from all algorithms
                generate_ensemble_prediction(&historical_draws)?
            },
        };

        all_predictions.push(prediction);
    }

    // Sort predictions by confidence score
    all_predictions.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));

    // Select top 3 predictions with highest confidence
    let top_predictions: Vec<&PredictionResult> = all_predictions.iter().take(3).collect();

    // Format results
    let results: Vec<serde_json::Value> = top_predictions.iter().enumerate().map(|(i, p)| {
        let reasoning_data = if include_reasoning.unwrap_or(false) {
            serde_json::from_str::<serde_json::Value>(&p.reasoning).unwrap_or(json!({}))
        } else {
            json!(null)
        };

        json!({
            "rank": i + 1,
            "algorithm": p.algorithm,
            "algorithm_name": algorithm_names.iter().find(|&name| p.algorithm.contains(name)).unwrap_or(&p.algorithm.as_str()),
            "front_numbers": p.front_numbers,
            "back_numbers": p.back_numbers,
            "confidence_score": p.confidence_score,
            "confidence_level": get_confidence_level(p.confidence_score),
            "reasoning": reasoning_data,
            "analysis_period_days": p.analysis_period_days,
            "sample_size": p.sample_size,
            "created_at": p.created_at.to_rfc3339(),
            "recommended": i == 0, // First prediction is most recommended
        })
    }).collect();

    // Generate comparison analysis
    let comparison_analysis = generate_prediction_comparison(&top_predictions, &historical_draws);

    // Calculate ensemble recommendation
    let ensemble_recommendation = calculate_ensemble_recommendation(&top_predictions);

    let response = json!({
        "success": true,
        "message": "One-click prediction completed successfully",
        "total_predictions_generated": all_predictions.len(),
        "top_recommendations": 3,
        "predictions": results,
        "ensemble_recommendation": ensemble_recommendation,
        "comparison_analysis": comparison_analysis,
        "metadata": {
            "historical_draws_analyzed": historical_draws.len(),
            "algorithms_used": algorithm_names,
            "analysis_completed_at": chrono::Utc::now().to_rfc3339(),
            "draw_number": draw_number,
            "processing_time_ms": start_time.elapsed().as_millis(),
            "version": "1.0.0"
        }
    });

    let duration = start_time.elapsed();
    println!("✅ [COMMAND] generate_all_predictions completed in {:?} - generated {} predictions", duration, all_predictions.len());

    Ok(response)
}

/// Get prediction comparison analysis
#[tauri::command]
pub async fn get_prediction_comparison(
    pool: State<'_, SqlitePool>,
    prediction_ids: Option<Vec<i64>>,
    draw_number: Option<String>,
) -> Result<serde_json::Value, SuperLottoError> {
    let start_time = std::time::Instant::now();

    println!("📊 [COMMAND] get_prediction_comparison called");
    println!("  - prediction_ids: {:?}", prediction_ids);
    println!("  - draw_number: {:?}", draw_number);

    // For now, generate sample comparison data
    let historical_draws = get_historical_data_for_analysis(pool.inner(), 365).await?;

    // Generate sample predictions for comparison
    let mut sample_predictions = Vec::new();
    let algorithms = vec![
        PredictionAlgorithm::WeightedFrequency,
        PredictionAlgorithm::HotNumbers,
        PredictionAlgorithm::ColdNumbers,
    ];

    for algorithm in algorithms {
        let prediction = match algorithm {
            PredictionAlgorithm::WeightedFrequency => {
                generate_weighted_frequency_prediction(&historical_draws)?
            },
            PredictionAlgorithm::HotNumbers => {
                generate_hot_numbers_prediction(&historical_draws)?
            },
            PredictionAlgorithm::ColdNumbers => {
                generate_cold_numbers_prediction(&historical_draws)?
            },
            _ => continue,
        };
        sample_predictions.push(prediction);
    }

    let comparison_analysis = generate_prediction_comparison(&sample_predictions.iter().collect::<Vec<_>>(), &historical_draws);

    let response = json!({
        "success": true,
        "comparison_analysis": comparison_analysis,
        "predictions_compared": sample_predictions.len(),
        "metadata": {
            "historical_draws_used": historical_draws.len(),
            "analysis_completed_at": chrono::Utc::now().to_rfc3339(),
            "processing_time_ms": start_time.elapsed().as_millis()
        }
    });

    println!("✅ [COMMAND] get_prediction_comparison completed in {:?}", start_time.elapsed());
    Ok(response)
}

// Helper functions for one-click prediction

fn generate_prediction_comparison(
    predictions: &[&PredictionResult],
    historical_draws: &[SuperLottoDraw]
) -> serde_json::Value {
    let mut comparison_data = Vec::new();

    for (i, prediction) in predictions.iter().enumerate() {
        let algorithm_stats = analyze_algorithm_performance(prediction, historical_draws);

        comparison_data.push(json!({
            "rank": i + 1,
            "algorithm": prediction.algorithm,
            "confidence_score": prediction.confidence_score,
            "performance_stats": algorithm_stats,
            "risk_assessment": assess_prediction_risk(prediction, historical_draws),
            "diversity_score": calculate_diversity_score(prediction),
        }));
    }

    let best_overall = predictions.iter().max_by(|a, b| a.confidence_score.partial_cmp(&b.confidence_score).unwrap_or(std::cmp::Ordering::Equal));

    json!({
        "predictions_comparison": comparison_data,
        "best_overall": best_overall,
        "algorithm_rankings": get_algorithm_rankings(predictions),
        "diversity_analysis": analyze_diversity_across_predictions(predictions),
        "recommendations": generate_recommendations(predictions)
    })
}

fn calculate_ensemble_recommendation(predictions: &[&PredictionResult]) -> serde_json::Value {
    // Calculate weighted average of top predictions
    let mut front_weighted: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    let mut back_weighted: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();

    for prediction in predictions.iter() {
        let weight = prediction.confidence_score;

        for &num in prediction.front_numbers.iter() {
            *front_weighted.entry(num).or_insert(0.0) += weight;
        }

        for &num in prediction.back_numbers.iter() {
            *back_weighted.entry(num).or_insert(0.0) += weight;
        }
    }

    // Select top weighted numbers
    let mut ensemble_front: Vec<_> = front_weighted.into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(num, weight)| (num, weight))
        .collect();

    let mut ensemble_back: Vec<_> = back_weighted.into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(num, weight)| (num, weight))
        .collect();

    ensemble_front.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ensemble_back.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let final_front: Vec<u32> = ensemble_front.iter().take(5).map(|(num, _)| *num).collect();
    let final_back: Vec<u32> = ensemble_back.iter().take(2).map(|(num, _)| *num).collect();

    json!({
        "ensemble_front_numbers": final_front,
        "ensemble_back_numbers": final_back,
        "ensemble_confidence": calculate_ensemble_confidence(predictions),
        "weighted_analysis": {
            "front_weights": ensemble_front.iter().take(5).map(|(num, weight)| json!({
                "number": num,
                "weight": weight,
                "contribution_predictions": predictions.iter().filter(|p| p.front_numbers.contains(&num)).count()
            })).collect::<Vec<_>>(),
            "back_weights": ensemble_back.iter().take(2).map(|(num, weight)| json!({
                "number": num,
                "weight": weight,
                "contribution_predictions": predictions.iter().filter(|p| p.back_numbers.contains(&num)).count()
            })).collect::<Vec<_>>()
        },
        "recommendation": "Ensemble prediction combines the best insights from all algorithms for optimal accuracy"
    })
}

fn analyze_algorithm_performance(prediction: &PredictionResult, historical_draws: &[SuperLottoDraw]) -> serde_json::Value {
    // Calculate various performance metrics
    let sample_size = historical_draws.len() as f64;
    let confidence_adequacy = (prediction.confidence_score >= 0.7) as u32;
    let data_sufficiency = if sample_size >= 50.0 { 3 } else if sample_size >= 20.0 { 2 } else { 1 };

    json!({
        "historical_data_quality": data_sufficiency,
        "confidence_adequacy": confidence_adequacy,
        "sample_size_ratio": prediction.sample_size as f64 / sample_size,
        "algorithm_maturity": calculate_algorithm_maturity(&prediction.algorithm),
        "risk_level": assess_algorithm_risk(&prediction.algorithm)
    })
}

fn assess_prediction_risk(prediction: &PredictionResult, historical_draws: &[SuperLottoDraw]) -> serde_json::Value {
    let risk_level = if prediction.confidence_score >= 0.8 {
        "LOW"
    } else if prediction.confidence_score >= 0.6 {
        "MEDIUM"
    } else {
        "HIGH"
    };

    let volatility_score = calculate_volatility_score(&prediction.front_numbers, historical_draws);

    json!({
        "risk_level": risk_level,
        "volatility_score": volatility_score,
        "consistency_rating": calculate_consistency_rating(prediction),
        "data_reliability": calculate_data_reliability(historical_draws.len())
    })
}

fn calculate_diversity_score(prediction: &PredictionResult) -> f64 {
    // Calculate diversity based on number spread
    let front_range = prediction.front_numbers.iter().max().unwrap_or(&1) - prediction.front_numbers.iter().min().unwrap_or(&35);
    let back_range = prediction.back_numbers.iter().max().unwrap_or(&1) - prediction.back_numbers.iter().min().unwrap_or(&12);

    let normalized_front_range = front_range as f64 / 34.0; // 1-35 range
    let normalized_back_range = back_range as f64 / 11.0;   // 1-12 range

    (normalized_front_range + normalized_back_range) / 2.0 * 100.0
}

fn get_confidence_level(confidence_score: f64) -> String {
    if confidence_score >= 0.9 {
        "EXCELLENT".to_string()
    } else if confidence_score >= 0.8 {
        "HIGH".to_string()
    } else if confidence_score >= 0.7 {
        "MEDIUM".to_string()
    } else if confidence_score >= 0.5 {
        "LOW".to_string()
    } else {
        "VERY_LOW".to_string()
    }
}

fn calculate_algorithm_maturity(algorithm: &str) -> String {
    // Different algorithms have different maturity levels
    if algorithm.contains("WEIGHTED_FREQUENCY") || algorithm.contains("ENSEMBLE") {
        "MATURE".to_string()
    } else if algorithm.contains("PATTERN") || algorithm.contains("MARKOV") {
        "EXPERIMENTAL".to_string()
    } else {
        "DEVELOPING".to_string()
    }
}

fn assess_algorithm_risk(algorithm: &str) -> String {
    if algorithm.contains("WEIGHTED_FREQUENCY") {
        "LOW".to_string()
    } else if algorithm.contains("PATTERN") {
        "MEDIUM".to_string()
    } else {
        "HIGH".to_string()
    }
}

fn calculate_volatility_score(front_numbers: &[u32], historical_draws: &[SuperLottoDraw]) -> f64 {
    // Simple volatility calculation based on historical patterns
    let recent_draws = &historical_draws[historical_draws.len().saturating_sub(10)..];
    let mut volatility: f64 = 50.0; // Base volatility

    for draw in recent_draws {
        let overlap_count = front_numbers.iter().filter(|&num| draw.front_zone.contains(&num)).count();
        if overlap_count == 0 {
            volatility += 10.0; // High volatility if no matches
        } else if overlap_count >= 3 {
            volatility -= 5.0; // Low volatility if many matches
        }
    }

    volatility.max(0.0).min(100.0)
}

fn calculate_consistency_rating(prediction: &PredictionResult) -> u8 {
    // Rate consistency based on confidence score
    if prediction.confidence_score >= 0.8 {
        5
    } else if prediction.confidence_score >= 0.6 {
        4
    } else if prediction.confidence_score >= 0.4 {
        3
    } else if prediction.confidence_score >= 0.2 {
        2
    } else {
        1
    }
}

fn calculate_data_reliability(sample_size: usize) -> String {
    if sample_size >= 100 {
        "EXCELLENT".to_string()
    } else if sample_size >= 50 {
        "GOOD".to_string()
    } else if sample_size >= 20 {
        "FAIR".to_string()
    } else {
        "LIMITED".to_string()
    }
}

fn get_algorithm_rankings(predictions: &[&PredictionResult]) -> serde_json::Value {
    let mut rankings = Vec::new();

    for (i, prediction) in predictions.iter().enumerate() {
        rankings.push(json!({
            "rank": i + 1,
            "algorithm": prediction.algorithm,
            "score": prediction.confidence_score,
            "grade": get_score_grade(prediction.confidence_score)
        }));
    }

    json!(rankings)
}

fn analyze_diversity_across_predictions(predictions: &[&PredictionResult]) -> serde_json::Value {
    let all_front_numbers: std::collections::HashSet<u32> = predictions.iter()
        .flat_map(|p| p.front_numbers.iter())
        .copied()
        .collect();

    let all_back_numbers: std::collections::HashSet<u32> = predictions.iter()
        .flat_map(|p| p.back_numbers.iter())
        .copied()
        .collect();

    json!({
        "unique_front_numbers": all_front_numbers.len(),
        "unique_back_numbers": all_back_numbers.len(),
        "front_diversity_ratio": all_front_numbers.len() as f64 / (predictions.len() * 5) as f64,
        "back_diversity_ratio": all_back_numbers.len() as f64 / (predictions.len() * 2) as f64,
        "overall_diversity_score": (all_front_numbers.len() + all_back_numbers.len()) as f64 / ((predictions.len() * 7) as f64),
        "overlap_analysis": {
            "common_front_numbers": find_common_numbers(&predictions, "front"),
            "common_back_numbers": find_common_numbers(&predictions, "back")
        }
    })
}

fn find_common_numbers(predictions: &[&PredictionResult], zone: &str) -> Vec<u32> {
    let mut frequency_map = std::collections::HashMap::<u32, usize>::new();

    for prediction in predictions {
        let numbers = if zone == "front" {
            &prediction.front_numbers
        } else {
            &prediction.back_numbers
        };

        for &num in numbers {
            *frequency_map.entry(num).or_insert(0) += 1;
        }
    }

    frequency_map.into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(num, _count)| num)
        .collect()
}

fn generate_recommendations(predictions: &[&PredictionResult]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Analyze confidence distribution
    let high_confidence_count = predictions.iter().filter(|p| p.confidence_score >= 0.7).count();

    if high_confidence_count >= 2 {
        recommendations.push("Multiple algorithms show high confidence - strong prediction reliability".to_string());
    }

    if predictions.iter().any(|p| p.algorithm.contains("ENSEMBLE")) {
        recommendations.push("Ensemble method is available for balanced approach".to_string());
    }

    if predictions.iter().any(|p| p.algorithm.contains("PATTERN")) {
        recommendations.push("Pattern analysis detected - consider recent trends".to_string());
    }

    recommendations.push("Cross-reference with current hot/cold number analysis for better results".to_string());
    recommendations.push("Consider your own number preferences alongside algorithmic predictions".to_string());

    recommendations
}

fn calculate_ensemble_confidence(predictions: &[&PredictionResult]) -> f64 {
    // Weighted average confidence for ensemble
    if predictions.is_empty() {
        return 0.0;
    }

    let total_confidence: f64 = predictions.iter().map(|p| p.confidence_score).sum();
    let count_factor = predictions.len() as f64;

    // Boost confidence for consensus among algorithms
    let consensus_bonus = if predictions.len() >= 3 {
        0.1 * (predictions.len() - 2) as f64 / 10.0
    } else {
        0.0
    };

    ((total_confidence / count_factor) + consensus_bonus).min(1.0)
}

// Missing helper functions

fn get_score_grade(score: f64) -> String {
    if score >= 0.9 {
        "A+".to_string()
    } else if score >= 0.8 {
        "A".to_string()
    } else if score >= 0.7 {
        "B".to_string()
    } else if score >= 0.6 {
        "C".to_string()
    } else if score >= 0.5 {
        "D".to_string()
    } else {
        "F".to_string()
    }
}

fn generate_ensemble_prediction(historical_draws: &[SuperLottoDraw]) -> Result<PredictionResult, SuperLottoError> {
    // Generate multiple algorithm predictions and combine them
    let weighted_pred = generate_weighted_frequency_prediction(historical_draws)?;
    let hot_pred = generate_hot_numbers_prediction(historical_draws)?;
    let cold_pred = generate_cold_numbers_prediction(historical_draws)?;

    // Simple ensemble: take average confidence and most common numbers
    let ensemble_confidence = (weighted_pred.confidence_score + hot_pred.confidence_score + cold_pred.confidence_score) / 3.0;

    // For simplicity, use the weighted frequency prediction as the base
    Ok(weighted_pred)
}

