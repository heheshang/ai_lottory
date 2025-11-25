//! Analysis cache model for caching prediction results

use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Analysis cache entry for storing computed results
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisCache {
    /// Unique identifier
    pub id: String,
    /// Algorithm used for analysis
    pub algorithm: String,
    /// Parameters used in analysis
    pub parameters: serde_json::Value,
    /// Hash of input data for cache invalidation
    pub data_hash: String,
    /// Computed analysis result
    pub result: serde_json::Value,
    /// Cache creation timestamp
    pub created_at: DateTime<Utc>,
    /// Cache expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Number of times accessed (for LRU)
    pub access_count: i64,
    /// Size of cached data in bytes
    pub size_bytes: i64,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Analysis execution time (in milliseconds)
    pub execution_time_ms: f64,
    /// Cache generation number (for cache invalidation)
    pub generation: i64,
}

impl AnalysisCache {
    /// Create a new cache entry
    pub fn new(
        algorithm: String,
        parameters: serde_json::Value,
        data_hash: String,
        result: serde_json::Value,
        ttl_seconds: i64,
        execution_time_ms: f64,
    ) -> Self {
        let now = Utc::now();
        let size_bytes = serde_json::to_string(&result)
            .map(|s| s.len() as i64)
            .unwrap_or(0);

        Self {
            id: Uuid::new_v4().to_string(),
            algorithm,
            parameters,
            data_hash,
            result,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds),
            access_count: 0,
            size_bytes,
            hit_rate: 0.0,
            execution_time_ms,
            generation: 1,
        }
    }

    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update access statistics
    pub fn update_access(&mut self) {
        self.access_count += 1;

        // Update hit rate (simple moving average)
        let new_rate = 100.0 / (self.access_count as f64 + 1.0);
        self.hit_rate = self.hit_rate * 0.9 + new_rate * 0.1;
    }

    /// Increment cache generation
    pub fn increment_generation(&mut self) {
        self.generation += 1;
    }

    /// Get cache efficiency score
    pub fn efficiency_score(&self) -> f64 {
        let age_hours = (Utc::now() - self.created_at).num_hours() as f64;
        if age_hours == 0.0 {
            100.0
        } else {
            // Score based on hit rate, recency, and reuse
            let recency_factor = (24.0 - age_hours.min(24.0)) / 24.0;
            let reuse_factor = (self.access_count as f64 / 100.0).min(1.0);
            (self.hit_rate * 0.4 + recency_factor * 30.0 + reuse_factor * 30.0)
        }
    }
}

/// Cache statistics for analysis operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisCacheStats {
    pub total_entries: i64,
    pub total_size_mb: f64,
    pub hit_rate: f64,
    pub average_execution_time_ms: f64,
    pub cache_saved_time_ms: f64,
    pub entries_by_algorithm: std::collections::HashMap<String, i64>,
}

impl AnalysisCacheStats {
    /// Calculate time saved by cache
    pub fn calculate_time_saved(&self) -> f64 {
        let average_time = self.average_execution_time_ms;
        let hits = self.total_entries as f64 * (self.hit_rate / 100.0);
        hits * average_time
    }
}

/// Cache key generator for analysis operations
pub struct AnalysisCacheKey;

impl AnalysisCacheKey {
    /// Generate cache key from algorithm and parameters
    pub fn generate(algorithm: &str, parameters: &serde_json::Value, data_hash: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Combine algorithm, parameters, and data hash
        algorithm.hash(&mut hasher);
        parameters.to_string().hash(&mut hasher);
        data_hash.hash(&mut hasher);

        format!("analysis_{}_{:x}", algorithm, hasher.finish())
    }

    /// Generate data hash from input data
    pub fn hash_data<T: Serialize>(data: &T) -> Result<String> {
        let serialized = serde_json::to_string(data)
            .map_err(|e| AppError::Internal {
                message: format!("Failed to serialize data for hashing: {}", e),
            })?;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }
}

/// Cache invalidation strategies
#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    /// Time-based expiration
    TimeBased,
    /// Data hash changes
    DataHashBased,
    /// Algorithm version changes
    AlgorithmVersionBased,
    /// Manual invalidation
    Manual,
}

/// Cache warming strategies
pub enum WarmingStrategy {
    /// Pre-compute common analyses
    Precompute,
    /// Cache on first access
    LazyLoad,
    /// Background warming
    Background,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_analysis_cache_creation() {
        let cache = AnalysisCache::new(
            "WEIGHTED_FREQUENCY".to_string(),
            json!({"days": 30}),
            "hash123".to_string(),
            json!({"result": "test"}),
            3600,
            1500.0,
        );

        assert_eq!(cache.algorithm, "WEIGHTED_FREQUENCY");
        assert_eq!(cache.data_hash, "hash123");
        assert_eq!(cache.access_count, 0);
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_cache_key_generation() {
        let key = AnalysisCacheKey::generate(
            "WEIGHTED_FREQUENCY",
            &json!({"days": 30}),
            "data123",
        );

        assert!(key.starts_with("analysis_WEIGHTED_FREQUENCY_"));
    }

    #[test]
    fn test_data_hashing() {
        let data = json!({"numbers": [1, 2, 3, 4, 5]});
        let hash1 = AnalysisCacheKey::hash_data(&data).unwrap();
        let hash2 = AnalysisCacheKey::hash_data(&data).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_cache_efficiency_score() {
        let mut cache = AnalysisCache::new(
            "ALGORITHM".to_string(),
            json!({}),
            "hash".to_string(),
            json!({}),
            3600,
            100.0,
        );

        // Initially high score due to recency
        assert!(cache.efficiency_score() > 90.0);

        // Update access to improve score
        for _ in 0..10 {
            cache.update_access();
        }

        assert_eq!(cache.access_count, 11);
        assert!(cache.efficiency_score() > 50.0);
    }
}