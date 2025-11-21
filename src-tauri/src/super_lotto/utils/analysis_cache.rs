//! Analysis cache utilities
//!
//! Caching system for expensive analysis operations.

use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::super_lotto::{models::AnalysisCache, errors::{Result, SuperLottoError}};

pub struct AnalysisCacheManager {
    cache: HashMap<String, (AnalysisCache, DateTime<Utc>)>,
    default_ttl: Duration,
}

impl AnalysisCacheManager {
    pub fn new(default_ttl_hours: u32) -> Self {
        Self {
            cache: HashMap::new(),
            default_ttl: Duration::from_secs(default_ttl_hours as u64 * 3600),
        }
    }

    pub fn get_or_insert<F, Fut, T>(&mut self, key: &str, generator: F) -> Fut
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
    {
        // Check if cache entry exists and is not expired
        if let Some((cached_entry, _)) = self.cache.get(key) {
            if !cached_entry.is_expired() {
                // Return cached result
                if let Ok(data) = cached_entry.get_result_data::<T>() {
                    return async move { Ok(data) };
                }
            }
        }

        // Generate new result
        async move {
            let result = generator().await?;
            // TODO: Cache the result
            result
        }
    }

    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.cache.retain(|_, (_, timestamp)| {
            *timestamp > now - self.default_ttl
        });
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for AnalysisCacheManager {
    fn default() -> Self {
        Self::new(1) // Default 1 hour TTL
    }
}