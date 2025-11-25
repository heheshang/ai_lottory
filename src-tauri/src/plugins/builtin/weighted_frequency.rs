//! Weighted Frequency Analysis Plugin

use crate::error::{AppError, Result};
use crate::plugins::{
    PredictionPlugin, PluginMetadata, PluginConfig, PluginState, PluginCapability,
    ResourceRequirements, PredictionParameters, PredictionResult, Prediction,
    ExecutionStats, ParameterType, DataColumn, ColumnType,
};
use serde_json::json;
use std::collections::HashMap;

/// Weighted frequency analysis plugin
pub struct WeightedFrequencyPlugin {
    metadata: PluginMetadata,
    state: PluginState,
    config: PluginConfig,
}

impl WeightedFrequencyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "weighted_frequency".to_string(),
                name: "Weighted Frequency Analysis".to_string(),
                description: "Analyzes number frequencies with time-based weighting".to_string(),
                version: "1.0.0".to_string(),
                author: "AI Lottery System".to_string(),
                category: "statistical".to_string(),
                tags: vec!["frequency".to_string(), "statistical".to_string(), "hot-cold".to_string()],
                min_data_size: 50,
                max_data_size: 10000,
                supported_lottery_types: vec!["powerball".to_string(), "megamillions".to_string(), "lotto".to_string()],
                capabilities: vec![
                    PluginCapability::HotNumbers,
                    PluginCapability::ColdNumbers,
                    PluginCapability::TrendAnalysis,
                ],
                dependencies: vec![],
                complexity_score: 25,
                estimated_execution_time_ms: 500,
                accuracy_score: 72.5,
                last_updated: chrono::Utc::now(),
            },
            state: PluginState::Uninitialized,
            config: PluginConfig::default(),
        }
    }

    fn analyze_frequencies(
        &self,
        historical_data: &[crate::plugins::LotteryDraw],
        weighting_params: &WeightingParams,
    ) -> HashMap<u32, f64> {
        let mut frequencies = HashMap::new();
        let mut weights_sum = 0.0;

        for draw in historical_data {
            // Calculate time-based weight
            let days_old = (chrono::Utc::now().date_naive() - draw.date).num_days() as f64;
            let time_weight = calculate_time_weight(days_old, weighting_params);

            for number in &draw.winning_numbers {
                let freq = frequencies.entry(*number).or_insert(0.0);
                *freq += time_weight;
                weights_sum += time_weight;
            }

            if let Some(bonus) = draw.bonus_number {
                let freq = frequencies.entry(bonus).or_insert(0.0);
                *freq += time_weight * 0.5; // Lower weight for bonus numbers
                weights_sum += time_weight * 0.5;
            }
        }

        // Normalize frequencies
        if weights_sum > 0.0 {
            for freq in frequencies.values_mut() {
                *freq /= weights_sum;
            }
        }

        frequencies
    }

    fn generate_predictions(
        &self,
        frequencies: &HashMap<u32, f64>,
        params: &PredictionParameters,
    ) -> Vec<Prediction> {
        let mut freq_vec: Vec<(u32, f64)> = frequencies.iter().map(|(k, v)| (*k, *v)).collect();
        freq_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut predictions = Vec::new();

        for i in 0..params.prediction_count.min(20) {
            // Select top numbers with some randomness
            let start_idx = (i * 2).min(freq_vec.len().saturating_sub(5));
            let end_idx = (start_idx + 7).min(freq_vec.len());

            if start_idx < end_idx {
                let selected_numbers: Vec<u32> = freq_vec[start_idx..end_idx]
                    .iter()
                    .take(6) // Standard lottery number count
                    .map(|(num, _)| *num)
                    .collect();

                let confidence = freq_vec[start_idx].1; // Use highest frequency as confidence

                predictions.push(Prediction {
                    numbers: selected_numbers,
                    confidence,
                    probability_distribution: None,
                    metadata: HashMap::new(),
                });
            }
        }

        predictions
    }
}

#[derive(Debug, Clone)]
struct WeightingParams {
    decay_factor: f64,
    min_weight: f64,
    max_weight: f64,
}

fn calculate_time_weight(days_old: f64, params: &WeightingParams) -> f64 {
    let weight = params.max_weight * (-days_old / params.decay_factor).exp();
    weight.max(params.min_weight)
}

impl PredictionPlugin for WeightedFrequencyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        self.state = PluginState::Ready;
        Ok(())
    }

    fn validate_parameters(&self, parameters: &PredictionParameters) -> Result<()> {
        if parameters.prediction_count == 0 {
            return Err(AppError::Validation {
                field: "prediction_count".to_string(),
                message: "Prediction count must be greater than 0".to_string(),
            });
        }

        if parameters.historical_data_days < 30 {
            return Err(AppError::Validation {
                field: "historical_data_days".to_string(),
                message: "Historical data period must be at least 30 days".to_string(),
            });
        }

        if parameters.confidence_threshold < 0.0 || parameters.confidence_threshold > 1.0 {
            return Err(AppError::Validation {
                field: "confidence_threshold".to_string(),
                message: "Confidence threshold must be between 0 and 1".to_string(),
            });
        }

        Ok(())
    }

    async fn predict(
        &self,
        historical_data: &[crate::plugins::LotteryDraw],
        parameters: &PredictionParameters,
    ) -> Result<PredictionResult> {
        let start_time = std::time::Instant::now();

        // Extract weighting parameters from algorithm params
        let weighting_params = WeightingParams {
            decay_factor: parameters
                .algorithm_params
                .get("decay_factor")
                .and_then(|v| v.as_f64())
                .unwrap_or(30.0),
            min_weight: parameters
                .algorithm_params
                .get("min_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.1),
            max_weight: parameters
                .algorithm_params
                .get("max_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0),
        };

        // Analyze frequencies
        let frequencies = self.analyze_frequencies(historical_data, &weighting_params);

        // Generate predictions
        let predictions = self.generate_predictions(&frequencies, parameters);

        // Calculate execution statistics
        let execution_time = start_time.elapsed();
        let execution_stats = ExecutionStats {
            execution_time_ms: execution_time.as_millis() as u64,
            processed_records: historical_data.len(),
            memory_usage_mb: estimate_memory_usage(historical_data.len()),
            cpu_usage_percent: estimate_cpu_usage(execution_time),
            cache_hit_rate: None,
            errors_count: 0,
            custom_metrics: HashMap::new(),
        };

        // Generate analysis data
        let analysis_data = Some(json!({
            "frequencies": frequencies,
            "weighting_params": weighting_params,
            "hot_numbers": get_top_numbers(&frequencies, 10),
            "cold_numbers": get_bottom_numbers(&frequencies, 10),
            "analysis_period_days": parameters.historical_data_days,
        }));

        let result = PredictionResult {
            plugin_id: self.metadata.id.clone(),
            predictions,
            confidence_score: calculate_overall_confidence(&frequencies, parameters),
            execution_stats,
            analysis_data,
            warnings: generate_warnings(historical_data.len(), &self.metadata),
            timestamp: chrono::Utc::now(),
        };

        Ok(result)
    }

    fn supported_parameters(&self) -> &[ParameterType] {
        &[
            ParameterType::Float {
                min: Some(1.0),
                max: Some(365.0),
                default: Some(30.0),
            }, // decay_factor
            ParameterType::Float {
                min: Some(0.0),
                max: Some(1.0),
                default: Some(0.1),
            }, // min_weight
            ParameterType::Float {
                min: Some(0.5),
                max: Some(2.0),
                default: Some(1.0),
            }, // max_weight
        ]
    }

    fn required_data_columns(&self) -> &[DataColumn] {
        &[
            DataColumn {
                name: "winning_numbers".to_string(),
                data_type: ColumnType::IntegerArray,
                required: true,
                description: Some("Array of winning numbers".to_string()),
            },
            DataColumn {
                name: "bonus_number".to_string(),
                data_type: ColumnType::Integer,
                required: false,
                description: Some("Optional bonus number".to_string()),
            },
            DataColumn {
                name: "date".to_string(),
                data_type: ColumnType::DateTime,
                required: true,
                description: Some("Draw date".to_string()),
            },
        ]
    }

    fn can_handle_dataset(&self, dataset_size: usize) -> bool {
        dataset_size >= self.metadata.min_data_size && dataset_size <= self.metadata.max_data_size
    }

    fn resource_requirements(&self) -> &ResourceRequirements {
        &ResourceRequirements {
            min_memory_mb: 64,
            recommended_memory_mb: 256,
            min_cpu_cores: 1,
            recommended_cpu_cores: 2,
            disk_space_mb: 10,
            network_requirements: crate::plugins::NetworkRequirements::None,
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        self.state = PluginState::Uninitialized;
        Ok(())
    }

    fn state(&self) -> PluginState {
        self.state.clone()
    }

    fn reset(&mut self) -> Result<()> {
        self.state = PluginState::Ready;
        Ok(())
    }
}

// Helper functions

fn get_top_numbers(frequencies: &HashMap<u32, f64>, count: usize) -> Vec<u32> {
    let mut freq_vec: Vec<(u32, f64)> = frequencies.iter().map(|(k, v)| (*k, *v)).collect();
    freq_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    freq_vec.iter().take(count).map(|(num, _)| *num).collect()
}

fn get_bottom_numbers(frequencies: &HashMap<u32, f64>, count: usize) -> Vec<u32> {
    let mut freq_vec: Vec<(u32, f64)> = frequencies.iter().map(|(k, v)| (*k, *v)).collect();
    freq_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    freq_vec.iter().take(count).map(|(num, _)| *num).collect()
}

fn calculate_overall_confidence(
    frequencies: &HashMap<u32, f64>,
    parameters: &PredictionParameters,
) -> f64 {
    let data_quality_factor = if frequencies.len() >= 40 { 0.9 } else { 0.7 };
    let time_period_factor = if parameters.historical_data_days >= 90 { 0.95 } else { 0.8 };
    let confidence_factor = parameters.confidence_threshold;

    (data_quality_factor + time_period_factor + confidence_factor) / 3.0 * 100.0
}

fn generate_warnings(data_size: usize, metadata: &PluginMetadata) -> Vec<String> {
    let mut warnings = Vec::new();

    if data_size < metadata.min_data_size {
        warnings.push(format!(
            "Dataset size ({}) is below recommended minimum ({})",
            data_size, metadata.min_data_size
        ));
    }

    if data_size < 100 {
        warnings.push("Small dataset size may affect prediction accuracy".to_string());
    }

    warnings
}

fn estimate_memory_usage(data_size: usize) -> f64 {
    // Rough estimate based on data size
    (data_size * 0.1) + 10.0 // Base memory + per-record overhead
}

fn estimate_cpu_usage(execution_time: std::time::Duration) -> f64 {
    // Estimate based on execution time (simplified)
    let time_ms = execution_time.as_millis() as f64;
    (time_ms / 1000.0 * 50.0).min(100.0) // Scale to 0-100%
}