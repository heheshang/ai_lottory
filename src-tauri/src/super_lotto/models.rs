//! Super Lotto data models
//!
//! This module contains all data structures for Super Lotto functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid front zone count: expected 5 numbers, got {0}")]
    InvalidFrontZoneCount(usize),
    #[error("Invalid back zone count: expected 2 numbers, got {0}")]
    InvalidBackZoneCount(usize),
    #[error("Duplicate numbers in front zone: {0:?}")]
    DuplicateFrontZoneNumbers(Vec<u32>),
    #[error("Duplicate numbers in back zone: {0:?}")]
    DuplicateBackZoneNumbers(Vec<u32>),
    #[error("Front zone number out of range (1-35): {0}")]
    InvalidFrontZoneRange(u32),
    #[error("Back zone number out of range (1-12): {0}")]
    InvalidBackZoneRange(u32),
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),
    #[error("Invalid JSON data: {0}")]
    InvalidJson(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SuperLottoDraw {
    pub id: i64,
    pub draw_date: DateTime<Utc>,
    pub draw_number: Option<String>,
    pub front_zone: Vec<u32>,         // 5 numbers from 1-35
    pub back_zone: Vec<u32>,          // 2 numbers from 1-12
    pub jackpot_amount: Option<f64>,
    pub winners_count: Option<u32>,
    pub created_at: DateTime<Utc>,

    // Computed fields (not in database, calculated on the fly)
    #[sqlx(skip)]
    pub sum_front: Option<u32>,
    #[sqlx(skip)]
    pub odd_count_front: Option<usize>,
    #[sqlx(skip)]
    pub even_count_front: Option<usize>,
    #[sqlx(skip)]
    pub has_consecutive_front: Option<bool>,
}

impl SuperLottoDraw {
    pub fn new(
        draw_date: DateTime<Utc>,
        front_zone: Vec<u32>,
        back_zone: Vec<u32>,
        draw_number: Option<String>,
        jackpot_amount: Option<f64>,
        winners_count: Option<u32>,
    ) -> Result<Self, ValidationError> {
        let mut draw = SuperLottoDraw {
            id: 0, // Will be set by database
            draw_date,
            draw_number,
            front_zone,
            back_zone,
            jackpot_amount,
            winners_count,
            created_at: Utc::now(),
            sum_front: None,
            odd_count_front: None,
            even_count_front: None,
            has_consecutive_front: None,
        };

        draw.validate()?;
        Ok(draw)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate front zone: exactly 5 unique numbers from 1-35
        if self.front_zone.len() != 5 {
            return Err(ValidationError::InvalidFrontZoneCount(self.front_zone.len()));
        }

        let mut sorted_front = self.front_zone.clone();
        sorted_front.sort();
        sorted_front.dedup();
        if sorted_front.len() != 5 {
            return Err(ValidationError::DuplicateFrontZoneNumbers(sorted_front));
        }

        if !sorted_front.iter().all(|&n| n >= 1 && n <= 35) {
            return Err(ValidationError::InvalidFrontZoneRange(
                sorted_front.iter().find(|&&n| n < 1 || n > 35).copied().unwrap_or(0)
            ));
        }

        // Validate back zone: exactly 2 unique numbers from 1-12
        if self.back_zone.len() != 2 {
            return Err(ValidationError::InvalidBackZoneCount(self.back_zone.len()));
        }

        let mut sorted_back = self.back_zone.clone();
        sorted_back.sort();
        sorted_back.dedup();
        if sorted_back.len() != 2 {
            return Err(ValidationError::DuplicateBackZoneNumbers(sorted_back));
        }

        if !sorted_back.iter().all(|&n| n >= 1 && n <= 12) {
            return Err(ValidationError::InvalidBackZoneRange(
                sorted_back.iter().find(|&&n| n < 1 || n > 12).copied().unwrap_or(0)
            ));
        }

        Ok(())
    }

    pub fn contains_number(&self, number: u32) -> bool {
        self.front_zone.contains(&number) || self.back_zone.contains(&number)
    }

    pub fn sum_front(&self) -> u32 {
        self.front_zone.iter().sum()
    }

    pub fn odd_count_front(&self) -> usize {
        self.front_zone.iter().filter(|&&n| n % 2 == 1).count()
    }

    pub fn even_count_front(&self) -> usize {
        self.front_zone.iter().filter(|&&n| n % 2 == 0).count()
    }

    pub fn has_consecutive_front(&self) -> bool {
        let mut sorted = self.front_zone.clone();
        sorted.sort();
        sorted.windows(2).any(|w| w[1] == w[0] + 1)
    }

    // Get computed fields or calculate them if not set
    pub fn get_sum_front(&mut self) -> u32 {
        match self.sum_front {
            Some(sum) => sum,
            None => {
                let sum = self.sum_front();
                self.sum_front = Some(sum);
                sum
            }
        }
    }

    pub fn get_odd_count_front(&mut self) -> usize {
        match self.odd_count_front {
            Some(count) => count,
            None => {
                let count = self.odd_count_front();
                self.odd_count_front = Some(count);
                count
            }
        }
    }

    pub fn get_even_count_front(&mut self) -> usize {
        match self.even_count_front {
            Some(count) => count,
            None => {
                let count = self.even_count_front();
                self.even_count_front = Some(count);
                count
            }
        }
    }

    pub fn get_has_consecutive_front(&mut self) -> bool {
        match self.has_consecutive_front {
            Some(has_consecutive) => has_consecutive,
            None => {
                let has_consecutive = self.has_consecutive_front();
                self.has_consecutive_front = Some(has_consecutive);
                has_consecutive
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NumberZone {
    Front,  // 1-35
    Back,   // 1-12
}

impl std::fmt::Display for NumberZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberZone::Front => write!(f, "FRONT"),
            NumberZone::Back => write!(f, "BACK"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NumberFrequency {
    pub id: i64,
    pub number: u32,
    pub zone: String, // "FRONT" or "BACK"
    pub frequency: u32,
    pub last_seen: Option<DateTime<Utc>>,
    pub hot_score: f64,
    pub cold_score: f64,
    pub average_gap: f64,
    pub current_gap: u32,
    pub period_days: u32,
    pub updated_at: DateTime<Utc>,
}

impl NumberFrequency {
    pub fn new(number: u32, zone: NumberZone, period_days: u32) -> Self {
        NumberFrequency {
            id: 0,
            number,
            zone: zone.to_string(),
            frequency: 0,
            last_seen: None,
            hot_score: 0.0,
            cold_score: 1.0, // Start cold
            average_gap: 0.0,
            current_gap: 0,
            period_days,
            updated_at: Utc::now(),
        }
    }

    pub fn calculate_hot_score(&mut self, total_draws: u32) {
        // Hot score combines frequency with recency
        let recency_factor = match self.last_seen {
            Some(last_seen) => {
                let days_ago = (Utc::now() - last_seen).num_days() as f64;
                1.0 / (1.0 + days_ago / 30.0) // Decay over 30 days
            },
            None => 0.0,
        };

        let frequency_score = if total_draws > 0 {
            self.frequency as f64 / total_draws as f64
        } else {
            0.0
        };

        self.hot_score = frequency_score * recency_factor * 100.0;
        self.cold_score = 1.0 / (1.0 + self.hot_score);
    }

    pub fn is_hot(&self, threshold: f64) -> bool {
        self.hot_score > threshold
    }

    pub fn is_cold(&self, threshold: f64) -> bool {
        self.cold_score > threshold
    }

    pub fn get_zone(&self) -> Result<NumberZone, ValidationError> {
        match self.zone.as_str() {
            "FRONT" => Ok(NumberZone::Front),
            "BACK" => Ok(NumberZone::Back),
            _ => Err(ValidationError::InvalidJson(format!("Invalid zone: {}", self.zone))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ConsecutiveNumbers,
    GapPatterns,
    OddEvenDistribution,
    SumRanges,
    PositionPatterns,
    ZonePatterns,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::ConsecutiveNumbers => write!(f, "CONSECUTIVE_NUMBERS"),
            PatternType::GapPatterns => write!(f, "GAP_PATTERNS"),
            PatternType::OddEvenDistribution => write!(f, "ODD_EVEN_DISTRIBUTION"),
            PatternType::SumRanges => write!(f, "SUM_RANGES"),
            PatternType::PositionPatterns => write!(f, "POSITION_PATTERNS"),
            PatternType::ZonePatterns => write!(f, "ZONE_PATTERNS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatternAnalysis {
    pub id: i64,
    pub pattern_type: String,
    pub analysis_data: String, // JSON string
    pub confidence_score: f64,
    pub sample_size: u32,
    pub period_days: u32,
    pub created_at: DateTime<Utc>,
}

impl PatternAnalysis {
    pub fn new(
        pattern_type: PatternType,
        analysis_data: serde_json::Value,
        confidence_score: f64,
        sample_size: u32,
        period_days: u32,
    ) -> Result<Self, ValidationError> {
        let analysis_data_str = serde_json::to_string(&analysis_data)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

        Ok(PatternAnalysis {
            id: 0,
            pattern_type: pattern_type.to_string(),
            analysis_data: analysis_data_str,
            confidence_score,
            sample_size,
            period_days,
            created_at: Utc::now(),
        })
    }

    pub fn get_analysis_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, ValidationError> {
        serde_json::from_str(&self.analysis_data)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionAlgorithm {
    WeightedFrequency,
    PatternBased,
    MarkovChain,
    Ensemble,
    HotNumbers,
    ColdNumbers,
    PositionAnalysis,
}

impl std::fmt::Display for PredictionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PredictionAlgorithm::WeightedFrequency => write!(f, "WEIGHTED_FREQUENCY"),
            PredictionAlgorithm::PatternBased => write!(f, "PATTERN_BASED"),
            PredictionAlgorithm::MarkovChain => write!(f, "MARKOV_CHAIN"),
            PredictionAlgorithm::Ensemble => write!(f, "ENSEMBLE"),
            PredictionAlgorithm::HotNumbers => write!(f, "HOT_NUMBERS"),
            PredictionAlgorithm::ColdNumbers => write!(f, "COLD_NUMBERS"),
            PredictionAlgorithm::PositionAnalysis => write!(f, "POSITION_ANALYSIS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PredictionResult {
    pub id: i64,
    pub algorithm: String,
    pub front_numbers: Vec<u32>,
    pub back_numbers: Vec<u32>,
    pub confidence_score: f64,
    pub reasoning: String, // JSON string
    pub analysis_period_days: u32,
    pub sample_size: u32,
    pub created_at: DateTime<Utc>,
    pub is_validated: bool,
}

impl PredictionResult {
    pub fn new(
        algorithm: PredictionAlgorithm,
        front_numbers: Vec<u32>,
        back_numbers: Vec<u32>,
        confidence_score: f64,
        reasoning: serde_json::Value,
        analysis_period_days: u32,
        sample_size: u32,
    ) -> Result<Self, ValidationError> {
        // Validate prediction
        if front_numbers.len() != 5 {
            return Err(ValidationError::InvalidFrontZoneCount(front_numbers.len()));
        }

        if back_numbers.len() != 2 {
            return Err(ValidationError::InvalidBackZoneCount(back_numbers.len()));
        }

        // Check ranges
        if !front_numbers.iter().all(|&n| n >= 1 && n <= 35) {
            return Err(ValidationError::InvalidFrontZoneRange(
                front_numbers.iter().find(|&&n| n < 1 || n > 35).copied().unwrap_or(0)
            ));
        }

        if !back_numbers.iter().all(|&n| n >= 1 && n <= 12) {
            return Err(ValidationError::InvalidBackZoneRange(
                back_numbers.iter().find(|&&n| n < 1 || n > 12).copied().unwrap_or(0)
            ));
        }

        let reasoning_str = serde_json::to_string(&reasoning)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

        Ok(PredictionResult {
            id: 0,
            algorithm: algorithm.to_string(),
            front_numbers,
            back_numbers,
            confidence_score,
            reasoning: reasoning_str,
            analysis_period_days,
            sample_size,
            created_at: Utc::now(),
            is_validated: false,
        })
    }

    pub fn get_reasoning<T: for<'de> Deserialize<'de>>(&self) -> Result<T, ValidationError> {
        serde_json::from_str(&self.reasoning)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))
    }

    pub fn calculate_hit_rate(&self, actual_draw: &SuperLottoDraw) -> f64 {
        let front_hits = self.front_numbers.iter()
            .filter(|&n| actual_draw.front_zone.contains(n))
            .count() as f64;

        let back_hits = self.back_numbers.iter()
            .filter(|&n| actual_draw.back_zone.contains(n))
            .count() as f64;

        (front_hits + back_hits) / 7.0 // Total 7 numbers (5 front + 2 back)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    HotNumbers,
    ColdNumbers,
    PatternAnalysis,
    FrequencyAnalysis,
    PredictionGeneration,
}

impl std::fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisType::HotNumbers => write!(f, "HOT_NUMBERS"),
            AnalysisType::ColdNumbers => write!(f, "COLD_NUMBERS"),
            AnalysisType::PatternAnalysis => write!(f, "PATTERN_ANALYSIS"),
            AnalysisType::FrequencyAnalysis => write!(f, "FREQUENCY_ANALYSIS"),
            AnalysisType::PredictionGeneration => write!(f, "PREDICTION_GENERATION"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisCache {
    pub id: i64,
    pub cache_key: String,
    pub analysis_type: String,
    pub result_data: String, // JSON string
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub hit_count: u32,
}

impl AnalysisCache {
    pub fn new(
        cache_key: String,
        analysis_type: AnalysisType,
        result_data: serde_json::Value,
        ttl_hours: u32,
    ) -> Result<Self, ValidationError> {
        let result_data_str = serde_json::to_string(&result_data)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

        Ok(AnalysisCache {
            id: 0,
            cache_key,
            analysis_type: analysis_type.to_string(),
            result_data: result_data_str,
            expires_at: Utc::now() + chrono::Duration::hours(ttl_hours as i64),
            created_at: Utc::now(),
            hit_count: 0,
        })
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn get_result_data<T: for<'de> Deserialize<'de>>(&self) -> Result<T, ValidationError> {
        serde_json::from_str(&self.result_data)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))
    }

    pub fn generate_cache_key(
        analysis_type: &AnalysisType,
        params: &serde_json::Value,
    ) -> Result<String, ValidationError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", analysis_type).hash(&mut hasher);
        params.to_string().hash(&mut hasher);

        Ok(format!("{}_{}", analysis_type, hasher.finish()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSuperLottoDraw {
    pub draw_date: String, // ISO 8601 string
    pub draw_number: Option<String>,
    pub front_zone: Vec<u32>,
    pub back_zone: Vec<u32>,
    pub jackpot_amount: Option<f64>,
    pub winners_count: Option<u32>,
}

impl CreateSuperLottoDraw {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate date format
        let draw_date = self.draw_date.parse::<DateTime<Utc>>()
            .map_err(|_| ValidationError::InvalidDateFormat(self.draw_date.clone()))?;

        // Create temporary draw for validation
        let temp_draw = SuperLottoDraw {
            id: 0,
            draw_date,
            draw_number: self.draw_number.clone(),
            front_zone: self.front_zone.clone(),
            back_zone: self.back_zone.clone(),
            jackpot_amount: self.jackpot_amount,
            winners_count: self.winners_count,
            created_at: Utc::now(),
            sum_front: None,
            odd_count_front: None,
            even_count_front: None,
            has_consecutive_front: None,
        };

        temp_draw.validate()
    }

    pub fn to_super_lotto_draw(&self) -> Result<SuperLottoDraw, ValidationError> {
        self.validate()?;

        let draw_date = self.draw_date.parse::<DateTime<Utc>>()
            .map_err(|_| ValidationError::InvalidDateFormat(self.draw_date.clone()))?;

        SuperLottoDraw::new(
            draw_date,
            self.front_zone.clone(),
            self.back_zone.clone(),
            self.draw_number.clone(),
            self.jackpot_amount,
            self.winners_count,
        )
    }
}