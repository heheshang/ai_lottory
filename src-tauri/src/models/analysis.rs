use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberFrequency {
    pub number: u32,
    pub frequency: u32,
    pub last_drawn: Option<DateTime<Utc>>,
    pub hot_score: f64,
    pub cold_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberStatistics {
    pub number: u32,
    pub total_draws: u32,
    pub frequency: f64,
    pub average_gap: f64,
    pub current_gap: u32,
    pub longest_gap: u32,
    pub shortest_gap: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub lottery_type: String,
    pub days: Option<u32>,       // Analysis period in days
    pub draw_count: Option<u32>, // Number of recent draws to analyze
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotNumbersResponse {
    pub numbers: Vec<NumberFrequency>,
    pub analysis_period: String,
    pub total_draws_analyzed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColdNumbersResponse {
    pub numbers: Vec<NumberFrequency>,
    pub analysis_period: String,
    pub total_draws_analyzed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberPattern {
    pub pattern: Vec<u32>,
    pub frequency: u32,
    pub last_occurrence: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub number: u32,
    pub trend_direction: String, // "hot", "cold", "stable"
    pub recent_frequency: f64,
    pub historical_frequency: f64,
    pub trend_strength: f64,
}
