//! Super Lotto data models
//!
//! This module contains all data structures for Super Lotto functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::convert::TryFrom;
use std::str::FromStr;
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
pub struct LotteryDraw {
    pub id: i64,
    pub draw_date: String,       // ISO 8601 datetime string
    pub winning_numbers: String, // JSON array of numbers
    pub bonus_number: Option<i64>,
    pub jackpot_amount: Option<f64>,
    pub lottery_type: String, // e.g., "super_lotto"
    pub created_at: String,   // ISO 8601 datetime string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(sqlx::FromRow)]
pub struct SuperLottoDraw {
    pub id: i64,
    pub draw_date: DateTime<Utc>,
    pub draw_number: Option<String>,
    #[sqlx(try_from = "String")]
    pub front_zone: NumberVec, // 5 numbers from 1-35, stored as JSON string
    #[sqlx(try_from = "String")]
    pub back_zone: NumberVec,  // 2 numbers from 1-12, stored as JSON string
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

// Wrapper type for Vec<u32> to implement SQLx conversions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberVec(Vec<u32>);

impl TryFrom<String> for NumberVec {
    type Error = serde_json::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&s).map(NumberVec)
    }
}

impl From<NumberVec> for String {
    fn from(vec: NumberVec) -> Self {
        serde_json::to_string(&vec.0).unwrap_or_default()
    }
}

impl From<NumberVec> for Vec<u32> {
    fn from(vec: NumberVec) -> Self {
        vec.0
    }
}

impl From<Vec<u32>> for NumberVec {
    fn from(vec: Vec<u32>) -> Self {
        NumberVec(vec)
    }
}

// Add Vec-like methods for NumberVec
impl NumberVec {
    pub fn contains(&self, item: &u32) -> bool {
        self.0.contains(item)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, u32> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn sort(&mut self) {
        self.0.sort();
    }

    pub fn dedup(&mut self) {
        self.0.dedup();
    }

    pub fn windows(&self, size: usize) -> std::slice::Windows<'_, u32> {
        self.0.windows(size)
    }
}

// Add Deref trait to give NumberVec Vec-like behavior
impl std::ops::Deref for NumberVec {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Add DerefMut for mutable access
impl std::ops::DerefMut for NumberVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Add Default trait
impl Default for NumberVec {
    fn default() -> Self {
        NumberVec(Vec::new())
    }
}

// Add IntoIterator support
impl IntoIterator for NumberVec {
    type Item = u32;
    type IntoIter = std::vec::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a NumberVec {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<LotteryDraw> for SuperLottoDraw {
    fn from(lottery_draw: LotteryDraw) -> Self {
        // Parse winning_numbers JSON and separate into front_zone and back_zone
        // For Super Lotto, we expect format like [1,5,12,23,28] + bonus numbers
        let numbers: Vec<u32> = serde_json::from_str(&lottery_draw.winning_numbers)
            .unwrap_or_default();

        // For Super Lotto: first 5 numbers are front_zone, bonus_number is back_zone
        let (front_zone, back_zone) = if lottery_draw.lottery_type == "super_lotto" {
            let front = numbers.into_iter().take(5).collect();
            let back = lottery_draw
                .bonus_number
                .map(|b| vec![b as u32])
                .unwrap_or_default();
            (front, back)
        } else {
            // Default fallback
            (vec![], vec![])
        };

        // Parse dates
        let draw_dt = DateTime::parse_from_rfc3339(&lottery_draw.draw_date)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let created_dt = DateTime::parse_from_rfc3339(&lottery_draw.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Self {
            id: lottery_draw.id,
            draw_date: draw_dt,
            draw_number: None,
            front_zone: NumberVec::from(front_zone),
            back_zone: NumberVec::from(back_zone),
            jackpot_amount: lottery_draw.jackpot_amount,
            winners_count: None,
            created_at: created_dt,
            sum_front: None,
            odd_count_front: None,
            even_count_front: None,
            has_consecutive_front: None,
        }
    }
}

// Add From implementation for the external LotteryDraw type
impl From<crate::models::lottery::LotteryDraw> for SuperLottoDraw {
    fn from(lottery_draw: crate::models::lottery::LotteryDraw) -> Self {
        // Convert winning_numbers from NumberVec to Vec<u32>
        let numbers: Vec<u32> = lottery_draw.winning_numbers.to_vec();

        // For Super Lotto: first 5 numbers are front_zone, bonus_number is back_zone
        let (front_zone, back_zone) = if lottery_draw.lottery_type == "super_lotto" {
            let front = numbers.into_iter().take(5).collect();
            let back = lottery_draw
                .bonus_number
                .map(|b| vec![b as u32])
                .unwrap_or_default();
            (front, back)
        } else {
            // Default fallback
            (vec![], vec![])
        };

        Self {
            id: lottery_draw.id as i64,
            draw_date: lottery_draw.draw_date,
            draw_number: None,
            front_zone: NumberVec::from(front_zone),
            back_zone: NumberVec::from(back_zone),
            jackpot_amount: lottery_draw.jackpot_amount,
            winners_count: None,
            created_at: lottery_draw.created_at,
            sum_front: None,
            odd_count_front: None,
            even_count_front: None,
            has_consecutive_front: None,
        }
    }
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
        let draw = SuperLottoDraw {
            id: 0, // Will be set by database
            draw_date,
            draw_number,
            front_zone: NumberVec::from(front_zone),
            back_zone: NumberVec::from(back_zone),
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
            return Err(ValidationError::InvalidFrontZoneCount(
                self.front_zone.len(),
            ));
        }

        let mut sorted_front = self.front_zone.clone();
        sorted_front.sort();
        sorted_front.dedup();
        if sorted_front.len() != 5 {
            return Err(ValidationError::DuplicateFrontZoneNumbers(sorted_front.to_vec()));
        }

        if !sorted_front.iter().all(|&n| n >= 1 && n <= 35) {
            return Err(ValidationError::InvalidFrontZoneRange(
                sorted_front
                    .iter()
                    .find(|&&n| n < 1 || n > 35)
                    .copied()
                    .unwrap_or(0),
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
            return Err(ValidationError::DuplicateBackZoneNumbers(sorted_back.to_vec()));
        }

        if !sorted_back.iter().all(|&n| n >= 1 && n <= 12) {
            return Err(ValidationError::InvalidBackZoneRange(
                sorted_back
                    .iter()
                    .find(|&&n| n < 1 || n > 12)
                    .copied()
                    .unwrap_or(0),
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
    Front, // 1-35
    Back,  // 1-12
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
            }
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
            _ => Err(ValidationError::InvalidJson(format!(
                "Invalid zone: {}",
                self.zone
            ))),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "String", into = "String")]
pub enum PredictionAlgorithm {
    WeightedFrequency,
    HotNumbers,
    ColdNumbers,
    PatternBased,
    Ensemble,
    MarkovChain,
    PositionAnalysis,
}

impl FromStr for PredictionAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "WEIGHTED_FREQUENCY" | "WEIGHTED-FREQUENCY" | "WEIGHTEDFREQUENCY" | "WEIGHTED_FREQUENCY" => {
                Ok(PredictionAlgorithm::WeightedFrequency)
            }
            "HOT_NUMBERS" | "HOT-NUMBERS" | "HOTNUMBERS" => Ok(PredictionAlgorithm::HotNumbers),
            "COLD_NUMBERS" | "COLD-NUMBERS" | "COLDNUMBERS" => Ok(PredictionAlgorithm::ColdNumbers),
            "PATTERN_BASED" | "PATTERN-BASED" | "PATTERNBASED" => {
                Ok(PredictionAlgorithm::PatternBased)
            }
            "ENSEMBLE" => Ok(PredictionAlgorithm::Ensemble),
            "MARKOV_CHAIN" | "MARKOV-CHAIN" | "MARKOVCHAIN" => Ok(PredictionAlgorithm::MarkovChain),
            "POSITION_ANALYSIS" | "POSITION-ANALYSIS" | "POSITIONANALYSIS" => Ok(PredictionAlgorithm::PositionAnalysis),
            _ => {
                // Try lowercase as fallback
                match s.to_lowercase().as_str() {
                    "weighted_frequency" | "weighted-frequency" | "weightedfrequency" => {
                        Ok(PredictionAlgorithm::WeightedFrequency)
                    }
                    "hot_numbers" | "hot-numbers" | "hotnumbers" => Ok(PredictionAlgorithm::HotNumbers),
                    "cold_numbers" | "cold-numbers" | "coldnumbers" => Ok(PredictionAlgorithm::ColdNumbers),
                    "pattern_based" | "pattern-based" | "patternbased" => {
                        Ok(PredictionAlgorithm::PatternBased)
                    }
                    "ensemble" => Ok(PredictionAlgorithm::Ensemble),
                    "markov_chain" | "markov-chain" | "markovchain" => Ok(PredictionAlgorithm::MarkovChain),
                    "position_analysis" | "position-analysis" | "positionanalysis" => Ok(PredictionAlgorithm::PositionAnalysis),
                    _ => Err(format!("Unknown prediction algorithm: {}", s)),
                }
            }
        }
    }
}

impl TryFrom<String> for PredictionAlgorithm {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl From<PredictionAlgorithm> for String {
    fn from(algo: PredictionAlgorithm) -> Self {
        algo.to_string()
    }
}

impl std::fmt::Display for PredictionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PredictionAlgorithm::WeightedFrequency => write!(f, "WEIGHTED_FREQUENCY"),
            PredictionAlgorithm::HotNumbers => write!(f, "HOT_NUMBERS"),
            PredictionAlgorithm::ColdNumbers => write!(f, "COLD_NUMBERS"),
            PredictionAlgorithm::PatternBased => write!(f, "PATTERN_BASED"),
            PredictionAlgorithm::Ensemble => write!(f, "ENSEMBLE"),
            PredictionAlgorithm::MarkovChain => write!(f, "MARKOV_CHAIN"),
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
                front_numbers
                    .iter()
                    .find(|&&n| n < 1 || n > 35)
                    .copied()
                    .unwrap_or(0),
            ));
        }

        if !back_numbers.iter().all(|&n| n >= 1 && n <= 12) {
            return Err(ValidationError::InvalidBackZoneRange(
                back_numbers
                    .iter()
                    .find(|&&n| n < 1 || n > 12)
                    .copied()
                    .unwrap_or(0),
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
        let front_hits = self
            .front_numbers
            .iter()
            .filter(|&n| actual_draw.front_zone.contains(n))
            .count() as f64;

        let back_hits = self
            .back_numbers
            .iter()
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

// Batch Prediction Models for One-Click Prediction Feature

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionRequest {
    pub algorithms: Vec<PredictionAlgorithm>,
    pub analysis_period_days: u32,
    pub include_reasoning: bool,
    pub custom_parameters: Option<serde_json::Value>,
    pub draw_number: u32,
}

impl BatchPredictionRequest {
    pub fn new(analysis_period_days: u32, draw_number: u32) -> Self {
        Self {
            algorithms: vec![
                PredictionAlgorithm::WeightedFrequency,
                PredictionAlgorithm::HotNumbers,
                PredictionAlgorithm::ColdNumbers,
                PredictionAlgorithm::PatternBased,
                PredictionAlgorithm::Ensemble,
            ],
            analysis_period_days,
            include_reasoning: true,
            custom_parameters: None,
            draw_number,
        }
    }

    pub fn with_algorithms(mut self, algorithms: Vec<PredictionAlgorithm>) -> Self {
        self.algorithms = algorithms;
        self
    }

    pub fn with_reasoning(mut self, include_reasoning: bool) -> Self {
        self.include_reasoning = include_reasoning;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedNumbers {
    pub red_numbers: Vec<u32>, // 5 numbers
    pub blue_number: u32,      // 1 number
    pub confidence: Option<PredictionConfidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfidence {
    pub red_confidence: Vec<f64>,
    pub blue_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionParameters {
    pub draw_count: Option<u32>,
    pub weight_factor: Option<f64>,
    pub pattern_weight: Option<f64>,
    pub hot_threshold: Option<f64>,
    pub cold_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionResult {
    pub id: i64,
    pub request_id: String,
    pub predictions: Vec<PredictionResult>,
    pub generated_at: DateTime<Utc>,
    pub total_predictions: u32,
    pub successful_predictions: u32,
    pub failed_predictions: u32,
    pub processing_time_ms: u64,
    pub analysis_period_days: u32,
    pub sample_size: u32,
}

// New struct for single prediction results (matching TypeScript interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePredictionResult {
    pub id: String,
    pub algorithm_id: String,
    pub algorithm_name: String,
    pub draw_number: u32,
    pub predicted_numbers: PredictedNumbers,
    pub confidence_score: f64,
    pub created_at: String,
    pub parameters: Option<PredictionParameters>,
    pub accuracy: Option<f64>,
    pub is_validated: bool,
}

impl BatchPredictionResult {
    pub fn new(
        request_id: String,
        predictions: Vec<PredictionResult>,
        processing_time_ms: u64,
        analysis_period_days: u32,
        sample_size: u32,
    ) -> Self {
        let total_predictions = predictions.len() as u32;
        let successful_predictions = predictions
            .iter()
            .filter(|p| p.confidence_score > 0.0)
            .count() as u32;
        let failed_predictions = total_predictions - successful_predictions;

        Self {
            id: 0,
            request_id,
            predictions,
            generated_at: Utc::now(),
            total_predictions,
            successful_predictions,
            failed_predictions,
            processing_time_ms,
            analysis_period_days,
            sample_size,
        }
    }

    pub fn get_best_prediction(&self) -> Option<&PredictionResult> {
        self.predictions.iter().max_by(|a, b| {
            a.confidence_score
                .partial_cmp(&b.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_prediction_by_algorithm(
        &self,
        algorithm: &PredictionAlgorithm,
    ) -> Option<&PredictionResult> {
        self.predictions
            .iter()
            .find(|p| p.algorithm == algorithm.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionComparison {
    pub batch_result: BatchPredictionResult,
    pub consensus_numbers: ConsensusNumbers,
    pub algorithm_rankings: Vec<AlgorithmRanking>,
    pub confidence_distribution: ConfidenceDistribution,
    pub recommendation: PredictionRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusNumbers {
    pub front_consensus: Vec<u32>,
    pub back_consensus: Vec<u32>,
    pub consensus_strength: f64, // 0.0 to 1.0
    pub voting_details: Vec<NumberVote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberVote {
    pub number: u32,
    pub zone: NumberZone,
    pub votes: u32, // Number of algorithms that voted for this number
    pub total_algorithms: u32,
    pub support_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmRanking {
    pub algorithm: PredictionAlgorithm,
    pub confidence_score: f64,
    pub rank: u32,
    pub unique_predictions: u32,
    pub overlap_with_consensus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDistribution {
    pub high_confidence_count: u32,   // > 0.8
    pub medium_confidence_count: u32, // 0.5 - 0.8
    pub low_confidence_count: u32,    // < 0.5
    pub average_confidence: f64,
    pub confidence_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRecommendation {
    pub recommended_front: Vec<u32>,
    pub recommended_back: Vec<u32>,
    pub confidence_level: f64,
    pub reasoning: String,
    pub risk_assessment: RiskLevel,
    pub alternative_combinations: Vec<AlternativeCombination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Conservative => write!(f, "CONSERVATIVE"),
            RiskLevel::Moderate => write!(f, "MODERATE"),
            RiskLevel::Aggressive => write!(f, "AGGRESSIVE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeCombination {
    pub front_numbers: Vec<u32>,
    pub back_numbers: Vec<u32>,
    pub strategy: String,
    pub confidence_score: f64,
    pub risk_level: RiskLevel,
}

// Unified Table Data Models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTableData {
    pub historical_draws: Vec<SuperLottoDraw>,
    pub prediction_results: Vec<PredictionResult>,
    pub batch_predictions: Vec<BatchPredictionResult>,
    pub combined_data: Vec<UnifiedTableRow>,
    pub filters: TableFilters,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedTableRow {
    HistoricalDraw {
        id: i64,
        date: DateTime<Utc>,
        draw_number: Option<String>,
        front_numbers: Vec<u32>,
        back_numbers: Vec<u32>,
        jackpot_amount: Option<f64>,
        row_type: String,
    },
    Prediction {
        id: i64,
        date: DateTime<Utc>,
        algorithm: PredictionAlgorithm,
        front_numbers: Vec<u32>,
        back_numbers: Vec<u32>,
        confidence_score: f64,
        row_type: String,
    },
    BatchPrediction {
        id: i64,
        date: DateTime<Utc>,
        request_id: String,
        total_predictions: u32,
        best_confidence: f64,
        row_type: String,
    },
}

impl UnifiedTableRow {
    pub fn get_date(&self) -> DateTime<Utc> {
        match self {
            UnifiedTableRow::HistoricalDraw { date, .. } => *date,
            UnifiedTableRow::Prediction { date, .. } => *date,
            UnifiedTableRow::BatchPrediction { date, .. } => *date,
        }
    }

    pub fn get_front_numbers(&self) -> Vec<u32> {
        match self {
            UnifiedTableRow::HistoricalDraw { front_numbers, .. } => front_numbers.clone(),
            UnifiedTableRow::Prediction { front_numbers, .. } => front_numbers.clone(),
            UnifiedTableRow::BatchPrediction { .. } => vec![],
        }
    }

    pub fn get_back_numbers(&self) -> Vec<u32> {
        match self {
            UnifiedTableRow::HistoricalDraw { back_numbers, .. } => back_numbers.clone(),
            UnifiedTableRow::Prediction { back_numbers, .. } => back_numbers.clone(),
            UnifiedTableRow::BatchPrediction { .. } => vec![],
        }
    }

    pub fn get_row_type(&self) -> &str {
        match self {
            UnifiedTableRow::HistoricalDraw { row_type, .. } => row_type,
            UnifiedTableRow::Prediction { row_type, .. } => row_type,
            UnifiedTableRow::BatchPrediction { row_type, .. } => row_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFilters {
    pub row_types: Vec<String>, // "historical", "prediction", "batch"
    pub algorithms: Vec<PredictionAlgorithm>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub confidence_range: Option<(f64, f64)>,
    pub number_filters: Option<NumberFilters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberFilters {
    pub include_numbers: Option<Vec<u32>>,
    pub exclude_numbers: Option<Vec<u32>>,
    pub front_sum_range: Option<(u32, u32)>,
    pub odd_even_ratio: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total_rows: u32,
    pub current_page: u32,
    pub rows_per_page: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl PaginationInfo {
    pub fn new(total_rows: u32, current_page: u32, rows_per_page: u32) -> Self {
        let total_pages = ((total_rows as f64) / (rows_per_page as f64)).ceil() as u32;
        Self {
            total_rows,
            current_page,
            rows_per_page,
            total_pages,
            has_next: current_page < total_pages,
            has_previous: current_page > 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExportRequest {
    pub filters: TableFilters,
    pub format: ExportFormat,
    pub include_reasoning: bool,
    pub max_rows: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Excel,
    Json,
    Pdf,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Csv => write!(f, "CSV"),
            ExportFormat::Excel => write!(f, "EXCEL"),
            ExportFormat::Json => write!(f, "JSON"),
            ExportFormat::Pdf => write!(f, "PDF"),
        }
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
        let draw_date = self
            .draw_date
            .parse::<DateTime<Utc>>()
            .map_err(|_| ValidationError::InvalidDateFormat(self.draw_date.clone()))?;

        // Create temporary draw for validation
        let temp_draw = SuperLottoDraw {
            id: 0,
            draw_date,
            draw_number: self.draw_number.clone(),
            front_zone: NumberVec::from(self.front_zone.clone()),
            back_zone: NumberVec::from(self.back_zone.clone()),
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

        let draw_date = self
            .draw_date
            .parse::<DateTime<Utc>>()
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
