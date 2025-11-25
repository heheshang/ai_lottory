//! Plugin Traits - Standardized interfaces for prediction algorithms

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Core trait that all prediction plugins must implement
pub trait PredictionPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;

    /// Validate input parameters
    fn validate_parameters(&self, parameters: &PredictionParameters) -> Result<()>;

    /// Execute the prediction algorithm
    async fn predict(
        &self,
        historical_data: &[LotteryDraw],
        parameters: &PredictionParameters,
    ) -> Result<PredictionResult>;

    /// Get supported parameter types
    fn supported_parameters(&self) -> &[ParameterType];

    /// Get required data columns
    fn required_data_columns(&self) -> &[DataColumn];

    /// Check if the plugin can handle the given dataset size
    fn can_handle_dataset(&self, dataset_size: usize) -> bool;

    /// Get resource requirements for this plugin
    fn resource_requirements(&self) -> &ResourceRequirements;

    /// Cleanup resources
    fn cleanup(&mut self) -> Result<()>;

    /// Get plugin state
    fn state(&self) -> PluginState;

    /// Reset plugin to initial state
    fn reset(&mut self) -> Result<()>;
}

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin category (e.g., "statistical", "machine-learning", "neural-network")
    pub category: String,
    /// Plugin tags for searching
    pub tags: Vec<String>,
    /// Minimum required data size
    pub min_data_size: usize,
    /// Maximum recommended data size
    pub max_data_size: usize,
    /// Supported lottery types
    pub supported_lottery_types: Vec<String>,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Plugin complexity score (0-100)
    pub complexity_score: u32,
    /// Estimated execution time (ms)
    pub estimated_execution_time_ms: u64,
    /// Accuracy score (0-100) based on historical performance
    pub accuracy_score: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Can generate hot number predictions
    HotNumbers,
    /// Can generate cold number predictions
    ColdNumbers,
    /// Can predict complete number sets
    CompletePrediction,
    /// Can provide probability distributions
    ProbabilityDistribution,
    /// Can perform trend analysis
    TrendAnalysis,
    /// Can handle missing data
    MissingDataHandling,
    /// Can provide confidence intervals
    ConfidenceIntervals,
    /// Supports batch processing
    BatchProcessing,
}

/// Plugin dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency identifier
    pub id: String,
    /// Required version range
    pub version_range: String,
    /// Whether this is an optional dependency
    pub optional: bool,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin-specific configuration
    pub custom_config: HashMap<String, serde_json::Value>,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            custom_config: HashMap::new(),
            performance: PerformanceConfig::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Performance configuration for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Enable caching for this plugin
    pub enable_caching: bool,
    /// Cache TTL for results
    pub cache_ttl: Duration,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Number of parallel workers
    pub parallel_workers: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            enable_caching: true,
            cache_ttl: Duration::from_secs(3600),
            enable_parallel: false,
            parallel_workers: 1,
        }
    }
}

/// Resource requirements for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory requirement in MB
    pub min_memory_mb: u64,
    /// Recommended memory in MB
    pub recommended_memory_mb: u64,
    /// Minimum CPU cores
    pub min_cpu_cores: usize,
    /// Recommended CPU cores
    pub recommended_cpu_cores: usize,
    /// Disk space requirement in MB
    pub disk_space_mb: u64,
    /// Network requirements
    pub network_requirements: NetworkRequirements,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_memory_mb: 64,
            recommended_memory_mb: 256,
            min_cpu_cores: 1,
            recommended_cpu_cores: 2,
            disk_space_mb: 10,
            network_requirements: NetworkRequirements::None,
        }
    }
}

/// Network requirements for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkRequirements {
    /// No network access required
    None,
    /// Optional network access
    Optional,
    /// Required network access
    Required,
    /// Specific URLs or domains allowed
    AllowedUrls(Vec<String>),
}

/// Plugin execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    /// Plugin is uninitialized
    Uninitialized,
    /// Plugin is initializing
    Initializing,
    /// Plugin is ready for execution
    Ready,
    /// Plugin is currently executing
    Executing,
    /// Plugin execution completed successfully
    Completed,
    /// Plugin execution failed
    Failed(String),
    /// Plugin is paused
    Paused,
    /// Plugin is shutting down
    ShuttingDown,
}

/// Prediction input parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionParameters {
    /// Algorithm-specific parameters
    pub algorithm_params: HashMap<String, serde_json::Value>,
    /// Historical data range in days
    pub historical_data_days: u32,
    /// Number of predictions to generate
    pub prediction_count: u32,
    /// Confidence threshold
    pub confidence_threshold: f64,
    /// Random seed for reproducible results
    pub random_seed: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Plugin identifier that generated this result
    pub plugin_id: String,
    /// Generated predictions
    pub predictions: Vec<Prediction>,
    /// Confidence score for the overall result
    pub confidence_score: f64,
    /// Execution statistics
    pub execution_stats: ExecutionStats,
    /// Additional analysis data
    pub analysis_data: Option<serde_json::Value>,
    /// Warnings or recommendations
    pub warnings: Vec<String>,
    /// Result timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Predicted numbers
    pub numbers: Vec<u32>,
    /// Confidence score for this prediction
    pub confidence: f64,
    /// Probability distribution
    pub probability_distribution: Option<Vec<f64>>,
    /// Additional prediction metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of processed records
    pub processed_records: usize,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Cache hit rate if applicable
    pub cache_hit_rate: Option<f64>,
    /// Number of errors encountered
    pub errors_count: u64,
    /// Additional performance metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Parameter type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// Boolean parameter
    Boolean { default: Option<bool> },
    /// Integer parameter
    Integer {
        min: Option<i64>,
        max: Option<i64>,
        default: Option<i64>,
    },
    /// Float parameter
    Float {
        min: Option<f64>,
        max: Option<f64>,
        default: Option<f64>,
    },
    /// String parameter
    String {
        allowed_values: Option<Vec<String>>,
        default: Option<String>,
    },
    /// Array parameter
    Array {
        element_type: Box<ParameterType>,
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    /// Object parameter
    Object {
        properties: HashMap<String, ParameterType>,
        required: Vec<String>,
    },
}

/// Data column definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataColumn {
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: ColumnType,
    /// Whether this column is required
    pub required: bool,
    /// Column description
    pub description: Option<String>,
}

/// Column data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnType {
    /// Integer values
    Integer,
    /// Floating point values
    Float,
    /// String values
    String,
    /// Date/time values
    DateTime,
    /// Boolean values
    Boolean,
    /// Array of integers
    IntegerArray,
    /// JSON object
    Json,
}

/// Lottery draw data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryDraw {
    /// Draw ID
    pub id: u64,
    /// Draw date
    pub date: chrono::NaiveDate,
    /// Winning numbers
    pub winning_numbers: Vec<u32>,
    /// Bonus number (if applicable)
    pub bonus_number: Option<u32>,
    /// Jackpot amount
    pub jackpot_amount: Option<f64>,
    /// Additional draw data
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Plugin validation trait
pub trait PluginValidator {
    /// Validate plugin metadata
    fn validate_metadata(&self, metadata: &PluginMetadata) -> Result<()>;

    /// Validate plugin configuration
    fn validate_config(&self, config: &PluginConfig, metadata: &PluginMetadata) -> Result<()>;

    /// Validate plugin dependencies
    fn validate_dependencies(&self, metadata: &PluginMetadata) -> Result<()>;

    /// Validate plugin implementation
    fn validate_implementation(&self, plugin: &dyn PredictionPlugin) -> Result<()>;
}

/// Plugin lifecycle trait for additional lifecycle hooks
pub trait PluginLifecycle {
    /// Called when plugin is loaded
    fn on_load(&mut self) -> Result<()>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<()>;

    /// Called before plugin execution
    fn before_execution(&mut self, parameters: &PredictionParameters) -> Result<()>;

    /// Called after plugin execution
    fn after_execution(&mut self, result: &PredictionResult) -> Result<()>;

    /// Called when plugin encounters an error
    fn on_error(&mut self, error: &AppError) -> Result<()>;
}

/// Plugin health check trait
pub trait PluginHealthCheck {
    /// Check plugin health
    fn check_health(&self) -> PluginHealthStatus;

    /// Get detailed health information
    fn get_health_details(&self) -> HashMap<String, serde_json::Value>;
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealthStatus {
    /// Overall health status
    pub status: HealthStatus,
    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Health score (0-100)
    pub health_score: u8,
    /// Issues found
    pub issues: Vec<String>,
    /// Resource usage
    pub resource_usage: ResourceUsage,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in MB
    pub memory_mb: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Disk usage in MB
    pub disk_mb: f64,
    /// Network usage in bytes
    pub network_bytes: u64,
}