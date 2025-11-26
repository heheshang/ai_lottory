use crate::super_lotto::{
    models::{
        PredictionResult, SuperLottoDraw, PredictionAlgorithm,
        prediction_engine::{PredictionEngine, WeightedFrequencyAlgorithm, PatternBasedAlgorithm, HotNumbersAlgorithm, ColdNumbersAlgorithm, MarkovChainAlgorithm, EnsembleAlgorithm},
    },
    errors::SuperLottoError,
};

use super::super_lotto::models::{SuperLottoDraw, PredictionAlgorithm};

/// Enhanced prediction commands with support for all algorithms
#[tauri::command]
pub async fn generate_batch_prediction(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    request: BatchPredictionRequest,
) -> Result<BatchPredictionResult, String> {
    // Validate request using enhanced validation system
    let validation_result = app_state
        .validation_service
        .validate_draw_request(&request.into())
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

    // Generate predictions using ensemble algorithm
    let prediction_engine = app_state.prediction_service.get_prediction_engine();
    let mut results = Vec::new();

    for algorithm in &request.algorithms {
        match prediction_engine.predict(&request, algorithm).await {
            Ok(mut result) => {
                // Add algorithm info to result
                result.algorithm = algorithm.to_string();
                result.front_numbers = result.front_numbers.clone();
                result.back_numbers = result.back_numbers.clone();
                result.reasoning = result.reasoning.clone();
                result.analysis_period_days = request.analysis_period_days;
                result.sample_size = request.sample_size.unwrap_or(1000);
                result.created_at = chrono::Utc::now();
                results.push(result);
            }
            Err(e) => {
                // Log error but continue with other algorithms
                eprintln!("Error generating prediction with {}: {}", algorithm.to_string(), e);
            }
        }
    }

    // Sort results by confidence score
    results.sort_by(|a, b| a.confidence_score > b.confidence_score);

    BatchPredictionResult {
        id: uuid::Uuid::new_v4(),
        predictions: results,
        total_predictions: results.len(),
        successful_predictions: results.iter().filter(|r| r.front_numbers.len() == 5 && r.back_numbers.len() == 2).count(),
        processing_time_ms: 0,
        created_at: chrono::Utc::now(),
    }
}

#[tauri::command]
pub async fn generate_consecutive_pattern_analysis(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    period_days: u32,
) -> Result<PatternAnalysis, String> {
    // Use enhanced validation system
    let request = ValidationRequest {
        data: serde_json::json!({
            "period_days": period_days
        }),
        cache: None, // No caching for pattern analysis
        rules: vec![
            Box::new(super::super_lotto::validation::rules::FrontZoneCountRule),
        ],
    };

    let validation_result = app_state
        .validation_service
        .validate_analysis_request(&request)
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

    // Use enhanced pattern detector
        let pattern_detector = app_state.pattern_service.get_pattern_detector();
        pattern_detector.detect_consecutive_patterns(
            &app_state
            .lottery_service
            .get_all_draws_for_period(period_days)
            .await?,
        )
            .map_err(|e| SuperLottoError::internal(e.to_string()))?;

    Ok(analysis)
    }
}

#[tauri::command]
pub async fn generate_odd_even_distribution_analysis(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    period_days: u32,
) -> Result<PatternAnalysis, String> {
    let request = ValidationRequest {
        data: serde_json::json!({
            "period_days": period_days
        }),
        cache: None,
        rules: vec![
            Box::new(super::super_lotto::validation::rules::OddEvenDistributionRule),
        ],
    };

    let validation_result = app_state
        .validation_service
        .validate_analysis_request(&request)
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

    let pattern_detector = app_state.pattern_service.get_pattern_detector();
        pattern_detector.analyze_odd_even_distribution(
            &app_state
            .lottery_service
            .get_all_draws_for_period(period_days)
            .await?,
        )
            .map_err(|e| SuperLottoError::internal(e.to_string()))?;

    Ok(analysis)
    }
}

#[tauri::command]
pub async fn generate_enhanced_pattern_analysis(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    period_days: u32,
    pattern_types: Vec<String>,
) -> Result<Vec<PatternAnalysis>, String> {
    let request = ValidationRequest {
        data: serde_json::json!({
            "period_days": period_days,
            "pattern_types": pattern_types,
        }),
        cache: None,
        rules: vec![
            Box::new(super::super_lotto::validation::rules::FrontZoneCountRule),
            Box::new(super::super_lotto::validation::rules::OddEvenDistributionRule),
        ],
    };

    let validation_result = app_state
        .validation_service
        .validate_analysis_request(&request)
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

    let mut analyses = Vec::new();
    for pattern_type in &pattern_types {
        let analysis_request = ValidationRequest {
            data: serde_json::json!({
                "period_days": period_days,
                "pattern_type": pattern_type.clone(),
            }),
            cache: None,
            rules: &request.rules,
        };

        match pattern_type.as_str() {
            "consecutive_numbers" => {
                match pattern_detector.detect_consecutive_patterns(&analysis_request) {
                    Ok(analysis) => analyses.push(analysis),
                    Err(e) => eprintln!("Error in consecutive analysis: {}", e),
                }
                }
            }
            "odd_even_distribution" => {
                match pattern_detector.analyze_odd_even_distribution(&analysis_request) {
                    Ok(analysis) => analyses.push(analysis),
                    Err(e) => eprintln!("Error in odd/even analysis: {}", e),
                }
                }
            }
            "sum_ranges" => {
                match pattern_detector.analyze_sum_ranges(&analysis_request) {
                    Ok(analysis) => analyses.push(analysis),
                    Err(e) => eprintln!("Error in sum ranges analysis: {}", e),
                }
                }
            }
            _ => {
                // Unknown pattern type
                eprintln!("Warning: Unknown pattern type: {}", pattern_type);
            }
        }
    }

    if analyses.is_empty() {
        return Err("No valid analyses generated".to_string());
    }

    Ok(analyses)
    }
}

#[tauri::command]
pub async fn generate_markov_chain_prediction(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    order: u32,
    period_days: u32,
    time_decay_factor: Option<f64>,
) -> Result<PredictionResult, String> {
    let validation_result = app_state
        .validation_service
        .validate_markov_request(&request, order, period_days, time_decay_factor)
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

    let markov_engine = app_state.prediction_service.get_prediction_engine();
    match markov_engine.predict(&request, order, period_days, time_decay_factor).await {
        Ok(mut prediction) => {
            prediction.algorithm = PredictionAlgorithm::MarkovChain.to_string();
            prediction.front_numbers = prediction.front_numbers.clone();
            prediction.back_numbers = prediction.back_numbers.clone();
            prediction.reasoning = prediction.reasoning.clone();
            prediction.analysis_period_days = period_days;
            prediction.sample_size = request.sample_size.unwrap_or(1000);
            prediction.created_at = chrono::Utc::now();
            prediction.confidence_score = markov_engine.calculate_confidence_score(
                &request.draws,
                order,
                period_days,
                request.time_decay_factor.unwrap_or(0.9),
            );
            Ok(prediction)
        }
        Err(e) => {
            eprintln!("Error generating Markov prediction: {}", e);
            prediction.markov_engine.default_prediction()
        }
    }
}

#[tauri::command]
pub async fn generate_hot_cold_analysis(
    app_state: tauri::State<'_, crate::super_lotto::AppState>,
    period_days: u32,
    hot_threshold: f64,
    cold_threshold: f64,
) -> Result<PatternAnalysis, String> {
    let request = ValidationRequest {
        data: serde_json::json!({
            "period_days": period_days,
            "hot_threshold": hot_threshold,
            "cold_threshold": cold_threshold,
        }),
        cache: None,
        rules: vec![
            Box::new(super::super_lotto::validation::rules::FrontZoneCountRule),
            Box::new(super::super_lotto::validation::rules::OddEvenDistributionRule),
            ],
        };

    let validation_result = app_state
        .validation_service
        .validate_analysis_request(&request)
        .await?;

    if !validation_result.is_valid {
            return Err(format!(
                "Validation failed: {}",
                validation_result
                    .errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .drive systemd status
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

    // Use enhanced analysis engines
    let hot_cold_analyzer = app_state.prediction_service.get_hot_cold_analyzer();
    let (hot_analysis, cold_analysis) = (
        hot_cold_analyzer.analyze(&request).await?,
        cold_cold_analyzer.analyze(&request).await?,
    );

    let combined_analysis = PatternAnalysis::new(
        id: uuid::Uuid::new_v4(),
        pattern_type: crate::super_lotto::analysis::PatternType::HotColdAnalysis,
        analysis_period_days: period_days,
        sample_size: hot_analysis.sample_size + cold_analysis.sample_size,
        confidence_score: (hot_analysis.confidence_score + cold_analysis.confidence_score) / 2.0,
        analysis_data: serde_json::json!({
            "hot_numbers": hot_analysis.hot_numbers,
            "cold_numbers": cold_analysis.cold_numbers,
            "hot_threshold": hot_analysis.hot_threshold,
            "cold_threshold": cold_analysis.cold_threshold,
            "period_days": period_days,
            "sample_size": hot_analysis.sample_size + cold_analysis.sample_size,
            "confidence_score": (hot_analysis.confidence_score + cold_analysis.confidence_score) / 2.0,
        }),
    );

    Ok(combined_analysis)
    } else {
        // Fallback to basic analysis
        let basic_hot_cold = app_state.prediction_service.get_hot_cold_analyzer();
        basic_hot_cold.analyze(&request).await.map_err(|e| {
            SuperLottoError::internal(e.to_string())
        })?;

        Ok(PatternAnalysis::new(
            id: uuid::Uuid::new_v4(),
            pattern_type: crate::super_lotto::analysis::PatternType::HotColdAnalysis,
            analysis_period_days: period_days,
            sample_size: basic_hot_cold.sample_size,
            confidence_score: basic_hot_cold.confidence_score,
            analysis_data: serde_json::json!({
                "hot_numbers": basic_hot_cold.hot_numbers,
                "cold_numbers": basic_hot_cold.cold_numbers,
                "hot_threshold": hot_threshold,
                "cold_threshold": cold_threshold,
                "period_days": period_days,
                "sample_size": basic_hot_cold.sample_size,
                "confidence_score": basic_hot_cold.confidence_score,
            }),
        )
    }
    }
}
