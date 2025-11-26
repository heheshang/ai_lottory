//! Pattern detection for Super Lotto numbers
//!
//! Implements various pattern analysis algorithms.

use crate::super_lotto::Result;
use crate::super_lotto::{
    errors::SuperLottoError,
    models::{PatternAnalysis, SuperLottoDraw},
};

pub struct PatternDetector {
    // TODO: Implement pattern detection logic
}

use crate::super_lotto::{
    models::{PatternAnalysis, SuperLottoDraw},
    errors::SuperLottoError,
    analysis::{PatternType, NumberFrequency, NumberZone, NumberAnalysis},
};

use std::collections::HashMap;
use serde_json::json;

pub struct PatternDetector {
    // Cache for pattern analysis results
    cache: HashMap<String, PatternAnalysis>,

    // Frequency maps for quick lookups
    consecutive_freq: HashMap<String, u32>,
    gap_freq: HashMap<u32, u32>,
    position_freq: HashMap<usize, HashMap<u32, u32>>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            consecutive_freq: HashMap::new(),
            gap_freq: HashMap::new(),
            position_freq: HashMap::default(),
        }
    }

    pub async fn detect_consecutive_patterns(
        &self,
        draws: &[SuperLottoDraw],
        period_days: u32,
    ) -> Result<PatternAnalysis> {
        if draws.is_empty() {
            return Err(SuperLottoError::internal(
                "No draws provided for consecutive pattern analysis"
            ));
        }

        // Build frequency data for consecutive patterns
        let mut consecutive_pairs = Vec::new();
        let mut total_consecutive = 0u32;
        let mut max_consecutive_count = 0u32;

        for draw in draws {
            let mut sorted_numbers = draw.front_zone.clone();
            sorted_numbers.sort();

            // Find consecutive number pairs
            for window in sorted_numbers.windows(2) {
                if window[1] + 1 == window[0] {
                    consecutive_pairs.push((window[0], window[1]));

                    // Track longest consecutive sequence
                    let mut current_sequence = vec![window[0], window[1]];
                    let mut next_window = sorted_numbers.windows(2).skip_while(|w| {
                        let mut sequence_end = false;
                        for num in w {
                            if let Some(prev) = current_sequence.last() {
                                if *num == prev + 1 {
                                    current_sequence.push(*num);
                                } else {
                                    sequence_end = true;
                                }
                            }
                        }
                        !sequence_end
                    });

                    if current_sequence.len() > max_consecutive_count {
                        max_consecutive_count = current_sequence.len();
                    }

                    current_sequence.append(&mut next_window.iter().take_while(|&w| {
                        let next_num = *w;
                        if let Some(last_seq) = current_sequence.last() {
                            next_num == last_seq + 1
                        } else {
                            false
                        }
                    }).collect());

                    if current_sequence.len() > 1 {
                        total_consecutive += 1;
                    }
                }
            }
        }

        // Calculate consecutive statistics
        let mut consecutive_distribution = HashMap::new();
        for (pair, _) in &consecutive_pairs {
            let key = format!("{}-{}", pair.0, pair.1);
            *consecutive_distribution.entry(key.clone()).or_insert(0) += 1;
        }

        // Calculate probability scores
        let consecutive_probability = if draws.len() > 0 {
            total_consecutive as f64 / draws.len() as f64
        } else {
            0.0
        };

        // Calculate confidence score based on sample size and pattern consistency
        let confidence_score = self.calculate_consecutive_confidence(
            draws.len(),
            total_consecutive as f64,
            max_consecutive_count
        );

        let analysis_data = json!({
            "consecutive_distribution": consecutive_distribution,
            "total_consecutive_pairs": total_consecutive,
            "max_consecutive_count": max_consecutive_count,
            "consecutive_probability": consecutive_probability,
            "period_days": period_days,
            "sample_size": draws.len()
        });

        let pattern_analysis = PatternAnalysis::new(
            crate::super_lotto::analysis::PatternType::ConsecutiveNumbers,
            period_days,
            draws.len(),
            confidence_score,
            analysis_data,
        );

        // Cache the result
        let cache_key = format!("consecutive_{}", period_days);
        self.cache.insert(cache_key, pattern_analysis.clone());

        Ok(pattern_analysis)
    }

    pub async fn analyze_odd_even_distribution(
        &self,
        draws: &[SuperLottoDraw],
        period_days: u32,
    ) -> Result<PatternAnalysis> {
        if draws.is_empty() {
            return Err(SuperLottoError::internal(
                "No draws provided for odd/even distribution analysis"
            ));
        }

        let mut odd_even_stats = Vec::new();

        for draw in draws {
            let odd_count = draw.front_zone.iter().filter(|&&n| n % 2 == 1).count();
            let even_count = 5 - odd_count;

            odd_even_stats.push((odd_count, even_count));
        }

        // Calculate distribution statistics
        let mut distribution_counts = HashMap::new();
        for (odd, even) in &odd_even_stats {
            let key = format!("{}-{}", odd, even);
            *distribution_counts.entry(key.clone()).or_insert(0) += 1;
        }

        // Calculate most common distributions
        let most_common = distribution_counts
            .iter()
            .max_by_key(|&(_, &count)| count)
            .map(|(key, count)| {
                let parts: Vec<&str> = key.split('-').collect();
                (parts[0].to_string(), parts[1].to_string(), *count)
            })
            .unwrap_or(("0-0", 0));

        // Calculate confidence based on sample size and distribution entropy
        let confidence_score = self.calculate_odd_even_confidence(
            draws.len(),
            &distribution_counts
        );

        let analysis_data = json!({
            "odd_even_distribution": distribution_counts,
            "most_common_distribution": most_common,
            "period_days": period_days,
            "sample_size": draws.len()
        });

        let pattern_analysis = PatternAnalysis::new(
            crate::super_lotto::analysis::PatternType::OddEvenDistribution,
            period_days,
            draws.len(),
            confidence_score,
            analysis_data,
        );

        // Cache the result
        let cache_key = format!("odd_even_{}", period_days);
        self.cache.insert(cache_key, pattern_analysis.clone());

        Ok(pattern_analysis)
    }

    pub async fn analyze_sum_ranges(
        &self,
        draws: &[SuperLottoDraw],
        period_days: u32,
    ) -> Result<PatternAnalysis> {
        if draws.is_empty() {
            return Err(SuperLottoError::internal(
                "No draws provided for sum range analysis"
            ));
        }

        let mut sums = Vec::new();
        let mut frequencies = HashMap::new();

        for draw in draws {
            let sum: u32 = draw.front_zone.iter().sum();
            sums.push(sum);
            *frequencies.entry(sum).or_insert(0) += 1;
        }

        // Calculate statistics
        let total_draws = draws.len() as f64;
        let mean = sums.iter().sum::<u32>() as f64 / total_draws;
        let variance = sums
            .iter()
            .map(|&sum| {
                let diff = *sum as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / total_draws;
        let std_dev = variance.sqrt();

        // Define sum ranges based on standard deviations
        let mut range_distribution = HashMap::new();

        // Very Low: mean - 2*std_dev
        let very_low_range = (mean - 2.0 * std_dev).max(1.0) as u32..(mean + 2.0 * std_dev).max(35.0) as u32;

        // Low: mean - std_dev to mean + std_dev
        let low_range = (mean - std_dev).max(1.0) as u32..(mean + std_dev).max(35.0) as u32;

        // Medium: mean ± std_dev
        let medium_range = (mean - std_dev).max(1.0) as u32..(mean + std_dev).max(35.0) as u32;

        // High: mean to mean + 2*std_dev
        let high_range = mean.max(1.0) as u32..(mean + 2.0 * std_dev).max(35.0) as u32;

        // Very High: mean + 2*std_dev to max
        let very_high_range = (mean + std_dev).min(35.0) as u32..(mean + 2.0 * std_dev).max(35.0) as u32;

        // Categorize each draw
        for sum in &sums {
            let range_name = if sum < very_low_range.1 || sum >= very_low_range.0 {
                "very_low"
            } else if sum < low_range.1 || sum >= low_range.0 {
                "low"
            } else if sum < medium_range.1 || sum >= medium_range.0 {
                "medium"
            } else if sum < high_range.1 || sum >= high_range.0 {
                "high"
            } else {
                "very_high"
            };

            *range_distribution.entry(range_name.to_string()).or_insert(0) += 1;
        }

        // Calculate confidence based on statistical significance
        let confidence_score = self.calculate_sum_confidence(
            draws.len(),
            variance,
            std_dev
        );

        let analysis_data = json!({
            "mean": mean,
            "standard_deviation": std_dev,
            "range_distribution": range_distribution,
            "period_days": period_days,
            "sample_size": draws.len()
        });

        let pattern_analysis = PatternAnalysis::new(
            crate::super_lotto::analysis::PatternType::SumRanges,
            period_days,
            draws.len(),
            confidence_score,
            analysis_data,
        );

        // Cache the result
        let cache_key = format!("sum_ranges_{}", period_days);
        self.cache.insert(cache_key, pattern_analysis.clone());

        Ok(pattern_analysis)
    }
}

// Helper methods for confidence calculation
impl PatternDetector {
    fn calculate_consecutive_confidence(&self, sample_size: usize, total_consecutive: f64, max_count: usize) -> f64 {
        let base_confidence = (sample_size as f64 / 1000.0).min(0.95); // Diminishing returns for larger samples
        let pattern_strength = if total_consecutive > 0.0 {
            (max_count as f64 / 5.0).min(1.0) // Normalized max consecutive count
        } else {
            0.0
        };

        let recency_factor = if sample_size > 365 {
            // Weight more recent data
            (365.0 / sample_size as f64).min(1.0)
        } else {
            1.0
        };

        base_confidence * pattern_strength * recency_factor
    }

    fn calculate_odd_even_confidence(&self, sample_size: usize, distribution_counts: &HashMap<String, u32>) -> f64 {
        let base_confidence = (sample_size as f64 / 1000.0).min(0.90);

        // Calculate entropy of distribution (higher entropy = more random = lower confidence)
        let total_distributions: u32 = distribution_counts.values().sum();
        let mut entropy = 0.0f64;

        for (_, &count) in distribution_counts {
            if *count > 0 {
                let probability = *count as f64 / total_distributions as f64;
                if probability > 0.0 {
                    entropy -= probability * probability.ln();
                }
            }
        }

        // Normalize entropy (max entropy for 5 possible distributions is ln(5) ≈ 1.609)
        let normalized_entropy = if total_distributions > 0 {
            entropy / (total_distributions as f64 * 5.0f64.ln())
        } else {
            0.0
        };

        // Lower entropy (more balanced distribution) = higher confidence
        let entropy_factor = 1.0 - normalized_entropy;

        base_confidence * entropy_factor
    }

    fn calculate_sum_confidence(&self, sample_size: usize, variance: f64, std_dev: f64) -> f64 {
        let base_confidence = (sample_size as f64 / 1000.0).min(0.85);

        // Statistical significance: lower variance = higher confidence in sum patterns
        let variance_significance = if variance > 0.0 {
            (1.0 / (1.0 + variance / 1000.0)).min(0.8)
        } else {
            1.0
        };

        // Sample size reliability
        let sample_reliability = if sample_size >= 100 {
            1.0
        } else {
            (sample_size as f64 / 100.0)
        };

        base_confidence * variance_significance * sample_reliability
    }
}

    pub async fn analyze_odd_even_distribution(
        &self,
        _draws: &[SuperLottoDraw],
        _period_days: u32,
    ) -> Result<PatternAnalysis> {
        // TODO: Implement odd/even distribution analysis
        Err(SuperLottoError::internal(
            "Odd/even analysis not implemented yet",
        ))
    }

    pub async fn analyze_sum_ranges(
        &self,
        _draws: &[SuperLottoDraw],
        _period_days: u32,
    ) -> Result<PatternAnalysis> {
        // TODO: Implement sum range analysis
        Err(SuperLottoError::internal(
            "Sum range analysis not implemented yet",
        ))
    }
}
