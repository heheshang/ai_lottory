//! Built-in prediction plugins

pub mod weighted_frequency;
pub mod pattern_analysis;
pub mod neural_network;

pub use weighted_frequency::WeightedFrequencyPlugin;
pub use pattern_analysis::PatternAnalysisPlugin;
pub use neural_network::NeuralNetworkPlugin;