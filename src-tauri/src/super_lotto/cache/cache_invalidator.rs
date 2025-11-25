//! Cache invalidation strategies and utilities

use crate::error::{AppError, Result};
use crate::super_lotto::models::analysis_cache::{AnalysisCache, InvalidationStrategy};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache invalidation manager
pub struct CacheInvalidator {
    invalidation_rules: Arc<RwLock<HashMap<String, InvalidationRule>>>,
    generation_counters: Arc<RwLock<HashMap<String, u64>>>,
}

/// Invalidation rule for cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationRule {
    pub algorithm: String,
    pub strategy: InvalidationStrategy,
    pub ttl_seconds: i64,
    pub max_age_hours: i64,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
}

impl CacheInvalidator {
    pub fn new() -> Self {
        let mut invalidator = Self {
            invalidation_rules: Arc::new(RwLock::new(HashMap::new())),
            generation_counters: Arc::new(RwLock::new(HashMap::new())),
        };

        // Add default rules
        tokio::spawn({
            let rules = invalidator.invalidation_rules.clone();
            async move {
                let mut rules_guard = rules.write().await;

                // Weighted frequency analysis - longer cache
                rules_guard.insert("WEIGHTED_FREQUENCY".to_string(), InvalidationRule {
                    algorithm: "WEIGHTED_FREQUENCY".to_string(),
                    strategy: InvalidationStrategy::DataHashBased,
                    ttl_seconds: 86400, // 24 hours
                    max_age_hours: 168, // 1 week
                    dependencies: vec![],
                    tags: vec!["analysis".to_string(), "frequent".to_string()],
                });

                // Pattern analysis - medium cache
                rules_guard.insert("PATTERN_BASED".to_string(), InvalidationRule {
                    algorithm: "PATTERN_BASED".to_string(),
                    strategy: InvalidationStrategy::TimeBased,
                    ttl_seconds: 3600, // 1 hour
                    max_age_hours: 24,
                    dependencies: vec![],
                    tags: vec!["analysis".to_string()],
                });

                // Hot numbers - shorter cache due to changing trends
                rules_guard.insert("HOT_NUMBERS".to_string(), InvalidationRule {
                    algorithm: "HOT_NUMBERS".to_string(),
                    strategy: InvalidationStrategy::TimeBased,
                    ttl_seconds: 1800, // 30 minutes
                    max_age_hours: 6,
                    dependencies: vec!["LOTTERY_DATA".to_string()],
                    tags: vec!["analysis".to_string(), "numbers".to_string()],
                });

                // Cold numbers - medium cache
                rules_guard.insert("COLD_NUMBERS".to_string(), InvalidationRule {
                    algorithm: "COLD_NUMBERS".to_string(),
                    strategy: InvalidationStrategy::DataHashBased,
                    ttl_seconds: 7200, // 2 hours
                    max_age_hours: 48,
                    dependencies: vec!["LOTTERY_DATA".to_string()],
                    tags: vec!["analysis".to_string(), "numbers".to_string()],
                });

                // Ensemble algorithm - shortest cache
                rules_guard.insert("ENSEMBLE".to_string(), InvalidationRule {
                    algorithm: "ENSEMBLE".to_string(),
                    strategy: InvalidationStrategy::TimeBased,
                    ttl_seconds: 900, // 15 minutes
                    max_age_hours: 2,
                    dependencies: vec!["WEIGHTED_FREQUENCY".to_string(), "PATTERN_BASED".to_string(), "HOT_NUMBERS".to_string()],
                    tags: vec!["analysis".to_string(), "ensemble".to_string()],
                });
            }
        });

        invalidator
    }

    /// Check if cache entry should be invalidated
    pub async fn should_invalidate(&self, cache: &AnalysisCache) -> bool {
        let rules = self.invalidation_rules.read().await;

        if let Some(rule) = rules.get(&cache.algorithm) {
            // Check expiration
            if cache.is_expired() {
                return true;
            }

            // Check max age
            let age_hours = (Utc::now() - cache.created_at).num_hours();
            if age_hours > rule.max_age_hours {
                return true;
            }

            // Check generation counter
            let counters = self.generation_counters.read().await;
            if let Some(&generation) = counters.get(&cache.algorithm) {
                if generation > cache.generation as u64 {
                    return true;
                }
            }

            // Check strategy-specific conditions
            match rule.strategy {
                InvalidationStrategy::TimeBased => {
                    // Already handled by expiration check
                    false
                }
                InvalidationStrategy::DataHashBased => {
                    // Check if underlying data has changed
                    self.has_data_changed(&cache.algorithm, &cache.data_hash).await
                }
                InvalidationStrategy::AlgorithmVersionBased => {
                    // Check algorithm version
                    self.has_algorithm_version_changed(&cache.algorithm).await
                }
                InvalidationStrategy::Manual => {
                    // Only invalidated manually
                    false
                }
            }
        } else {
            // No rule found, use default behavior
            cache.is_expired()
        }
    }

    /// Invalidate cache for specific algorithm
    pub async fn invalidate_algorithm(&self, algorithm: &str) -> Result<()> {
        let mut counters = self.generation_counters.write().await;
        let generation = counters.entry(algorithm.to_string()).or_insert(0);
        *generation += 1;

        tracing::info!("Invalidated cache for algorithm: {} (generation: {})", algorithm, generation);
        Ok(())
    }

    /// Invalidate cache by tag
    pub async fn invalidate_by_tag(&self, tag: &str) -> Result<Vec<String>> {
        let rules = self.invalidation_rules.read().await;
        let mut algorithms = Vec::new();

        for (algorithm, rule) in rules.iter() {
            if rule.tags.contains(&tag.to_string()) {
                algorithms.push(algorithm.clone());
            }
        }

        for algorithm in &algorithms {
            self.invalidate_algorithm(algorithm).await?;
        }

        tracing::info!("Invalidated cache by tag '{}': {} algorithms", tag, algorithms.len());
        Ok(algorithms)
    }

    /// Add or update invalidation rule
    pub async fn add_rule(&self, rule: InvalidationRule) -> Result<()> {
        let mut rules = self.invalidation_rules.write().await;
        rules.insert(rule.algorithm.clone(), rule);
        Ok(())
    }

    /// Get invalidation statistics
    pub async fn get_invalidation_stats(&self) -> InvalidationStats {
        let rules = self.invalidation_rules.read().await;
        let counters = self.generation_counters.read().await;

        let mut stats = InvalidationStats::default();
        stats.total_rules = rules.len();
        stats.total_invalidations = counters.values().sum();

        for rule in rules.values() {
            match rule.strategy {
                InvalidationStrategy::TimeBased => stats.time_based += 1,
                InvalidationStrategy::DataHashBased => stats.data_hash_based += 1,
                InvalidationStrategy::AlgorithmVersionBased => stats.algorithm_version_based += 1,
                InvalidationStrategy::Manual => stats.manual += 1,
            }
        }

        stats
    }

    /// Check if underlying data has changed
    async fn has_data_changed(&self, algorithm: &str, cached_hash: &str) -> bool {
        // This would typically check:
        // 1. Database version/timestamp
        // 2. File modification time
        // 3. External data source version

        // For now, return false (no change detection)
        false
    }

    /// Check if algorithm version has changed
    async fn has_algorithm_version_changed(&self, algorithm: &str) -> bool {
        // This would check algorithm version against expected version
        // For now, return false (no version change detected)
        false
    }

    /// Clean up expired rules
    pub async fn cleanup_expired_rules(&self) -> Result<usize> {
        let mut rules = self.invalidation_rules.write().await;
        let initial_count = rules.len();

        // Remove rules that are too old (older than 1 year)
        let cutoff = Utc::now() - Duration::days(365);

        // This would require storing rule creation timestamps
        // For now, just return 0
        Ok(0)
    }
}

/// Invalidation statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InvalidationStats {
    pub total_rules: usize,
    pub total_invalidations: u64,
    pub time_based: usize,
    pub data_hash_based: usize,
    pub algorithm_version_based: usize,
    pub manual: usize,
}

/// Cache warming strategies
pub struct CacheWarmer {
    invalidator: Arc<CacheInvalidator>,
}

impl CacheWarmer {
    pub fn new(invalidator: Arc<CacheInvalidator>) -> Self {
        Self { invalidator }
    }

    /// Warm cache with pre-computed analyses
    pub async fn warm_common_analyses(
        &self,
        algorithm: &str,
        parameters: serde_json::Value,
    ) -> Result<()> {
        tracing::info!("Warming cache for algorithm: {}", algorithm);

        // This would:
        // 1. Check if cache already has recent entry
        // 2. Run the analysis if needed
        // 3. Store result in cache

        // For now, just log the intent
        Ok(())
    }

    /// Schedule background warming
    pub async fn schedule_warming(&self) -> Result<()> {
        let invalidator = self.invalidator.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                // Warm frequently used algorithms
                let algorithms = vec!["WEIGHTED_FREQUENCY", "HOT_NUMBERS", "COLD_NUMBERS"];
                for algorithm in algorithms {
                    tracing::debug!("Background warming for algorithm: {}", algorithm);
                    // warming logic here
                }
            }
        });

        Ok(())
    }
}

/// Smart cache invalidation utilities
pub struct SmartInvalidator {
    invalidator: Arc<CacheInvalidator>,
}

impl SmartInvalidator {
    pub fn new(invalidator: Arc<CacheInvalidator>) -> Self {
        Self { invalidator }
    }

    /// Invalidate related caches when data changes
    pub async fn invalidate_related(&self, primary_algorithm: &str) -> Result<Vec<String>> {
        let rules = self.invalidator.invalidation_rules.read().await;
        let mut related = Vec::new();

        // Find algorithms that depend on the primary
        for (algorithm, rule) in rules.iter() {
            if rule.dependencies.contains(&primary_algorithm.to_string()) {
                related.push(algorithm.clone());
            }
        }

        // Invalidate each related algorithm
        for algorithm in &related {
            self.invalidator.invalidate_algorithm(algorithm).await?;
        }

        if !related.is_empty() {
            tracing::info!("Invalidated {} related caches for: {}", related.len(), primary_algorithm);
        }

        Ok(related)
    }

    /// Invalidate all analysis caches
    pub async fn invalidate_all_analyses(&self) -> Result<()> {
        let algorithms = vec!["WEIGHTED_FREQUENCY", "PATTERN_BASED", "HOT_NUMBERS", "COLD_NUMBERS", "MARKOV_CHAIN", "POSITION_ANALYSIS", "ENSEMBLE"];

        let mut invalidated = Vec::new();
        for algorithm in algorithms {
            if self.invalidator.invalidate_algorithm(algorithm).await.is_ok() {
                invalidated.push(algorithm.to_string());
            }
        }

        tracing::info!("Invalidated all analysis caches: {} algorithms", invalidated.len());
        Ok(())
    }

    /// Predictive invalidation based on usage patterns
    pub async fn predictive_invalidation(&self) -> Result<Vec<String>> {
        // Analyze cache hit rates and access patterns
        // Predict which caches will likely be stale and pre-emptively invalidate

        // For now, return empty
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_invalidator_creation() {
        let invalidator = CacheInvalidator::new();
        let stats = invalidator.get_invalidation_stats().await;

        assert!(stats.total_rules > 0);
        assert!(stats.time_based > 0);
    }

    #[tokio::test]
    async fn test_algorithm_invalidation() {
        let invalidator = CacheInvalidator::new();

        // Initial invalidation
        invalidator.invalidate_algorithm("WEIGHTED_FREQUENCY").await.unwrap();

        // Second invalidation should increment generation
        invalidator.invalidate_algorithm("WEIGHTED_FREQUENCY").await.unwrap();
    }

    #[tokio::test]
    async fn test_rule_creation() {
        let invalidator = CacheInvalidator::new();

        let rule = InvalidationRule {
            algorithm: "TEST_ALGORITHM".to_string(),
            strategy: InvalidationStrategy::TimeBased,
            ttl_seconds: 1800,
            max_age_hours: 12,
            dependencies: vec!["DEPENDENCY".to_string()],
            tags: vec!["test".to_string()],
        };

        invalidator.add_rule(rule).await.unwrap();

        let stats = invalidator.get_invalidation_stats().await;
        assert!(stats.total_rules > 7); // Default rules + new rule
    }
}