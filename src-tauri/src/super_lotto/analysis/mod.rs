//! Super Lotto analysis modules
//!
//! Various analysis algorithms and pattern detection.

pub mod frequency_analyzer;
pub mod pattern_detector;
pub mod predictor;

// Re-export main analysis types
pub use frequency_analyzer::FrequencyAnalyzer;
pub use pattern_detector::PatternDetector;
pub use predictor::PredictionEngine;