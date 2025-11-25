//! Repository Caching Layer
//!
//! Provides intelligent caching for repository operations.

use crate::error::{AppError, Result};
use crate::repository::traits::{CacheStats, CachedRepository};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub size_bytes: usize,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl_ms: Option<u64>) -> Self {
        let now = Utc::now();
        let expires_at = ttl_ms.map(|ttl| now + Duration::milliseconds(ttl as i64));
        let size_bytes = std::mem::size_of::<T>();

        Self {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            size_bytes,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size_bytes: usize,
    pub default_ttl_ms: u64,
    pub cleanup_interval_ms: u64,
    pub max_entries: Option<usize>,
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl_ms: 300000, // 5 minutes
            cleanup_interval_ms: 60000, // 1 minute
            max_entries: Some(10000),
            eviction_policy: EvictionPolicy::LeastRecentlyUsed,
        }
    }
}

/// Eviction policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LeastRecentlyUsed,
    LeastFrequentlyUsed,
    FirstInFirstOut,
    Random,
}

/// In-memory cache implementation
pub struct MemoryCache<T: Send + Sync + Clone> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    current_size_bytes: Arc<RwLock<usize>>,
    access_order: Arc<RwLock<Vec<String>>>,
}

impl<T: Send + Sync + Clone> MemoryCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::new())),
            current_size_bytes: Arc::new(RwLock::new(0)),
            access_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<T>> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;
        let mut access_order = self.access_order.write().await;

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                let size = entry.size_bytes;
                entries.remove(key);
                access_order.retain(|k| k != key);

                let mut current_size = self.current_size_bytes.write().await;
                *current_size = current_size.saturating_sub(size);

                stats.misses += 1;
                stats.deletions += 1;
                stats.calculate_hit_rate();
                return Ok(None);
            }

            // Update access information
            entry.access();

            // Update access order for LRU
            access_order.retain(|k| k != key);
            access_order.push(key.to_string());

            stats.hits += 1;
            stats.calculate_hit_rate();

            Ok(Some(entry.value.clone()))
        } else {
            stats.misses += 1;
            stats.calculate_hit_rate();
            Ok(None)
        }
    }

    pub async fn put(&self, key: &str, value: T, ttl_ms: Option<u64>) -> Result<()> {
        let ttl_ms = ttl_ms.or(Some(self.config.default_ttl_ms));
        let entry = CacheEntry::new(value.clone(), ttl_ms);
        let size = entry.size_bytes;

        // Check if we need to evict entries
        self.ensure_capacity(size).await?;

        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        let mut access_order = self.access_order.write().await;
        let mut stats = self.stats.write().await;

        // Remove existing entry if present
        if let Some(old_entry) = entries.get(key) {
            *current_size = current_size.saturating_sub(old_entry.size_bytes);
            access_order.retain(|k| k != key);
        }

        // Add new entry
        entries.insert(key.to_string(), entry);
        *current_size += size;
        access_order.push(key.to_string());

        stats.sets += 1;
        Ok(())
    }

    pub async fn remove(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        let mut access_order = self.access_order.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = entries.remove(key) {
            *current_size = current_size.saturating_sub(entry.size_bytes);
            access_order.retain(|k| k != key);

            stats.deletions += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        let mut access_order = self.access_order.write().await;
        let mut stats = self.stats.write().await;

        let entries_count = entries.len();
        entries.clear();
        *current_size = 0;
        access_order.clear();

        stats.deletions += entries_count as u64;
        Ok(())
    }

    async fn ensure_capacity(&self, required_size: usize) -> Result<()> {
        let current_size = *self.current_size_bytes.read().await;
        let max_size = self.config.max_size_bytes;

        if current_size + required_size <= max_size {
            return Ok(());
        }

        // Need to evict entries
        self.evict_entries(required_size).await
    }

    async fn evict_entries(&self, required_size: usize) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        let mut access_order = self.access_order.write().await;

        let target_size = self.config.max_size_bytes - required_size;
        let mut evicted_keys = Vec::new();

        match self.config.eviction_policy {
            EvictionPolicy::LeastRecentlyUsed => {
                // Evict least recently used entries
                while *current_size > target_size && !access_order.is_empty() {
                    if let Some(key) = access_order.first().cloned() {
                        if let Some(entry) = entries.remove(&key) {
                            *current_size = current_size.saturating_sub(entry.size_bytes);
                            evicted_keys.push(key);
                        }
                        access_order.remove(0);
                    } else {
                        break;
                    }
                }
            }
            EvictionPolicy::LeastFrequentlyUsed => {
                // Sort entries by access count and evict least frequently used
                let mut entries_vec: Vec<_> = entries.iter().collect();
                entries_vec.sort_by_key(|(_, entry)| entry.access_count);

                for (key, entry) in entries_vec {
                    if *current_size <= target_size {
                        break;
                    }
                    *current_size = current_size.saturating_sub(entry.size_bytes);
                    evicted_keys.push(key.clone());
                }

                for key in &evicted_keys {
                    entries.remove(key);
                    access_order.retain(|k| k != key);
                }
            }
            EvictionPolicy::FirstInFirstOut => {
                // Evict oldest entries
                let mut entries_vec: Vec<_> = entries.iter().collect();
                entries_vec.sort_by_key(|(_, entry)| entry.created_at);

                for (key, entry) in entries_vec {
                    if *current_size <= target_size {
                        break;
                    }
                    *current_size = current_size.saturating_sub(entry.size_bytes);
                    evicted_keys.push(key.clone());
                }

                for key in &evicted_keys {
                    entries.remove(key);
                    access_order.retain(|k| k != key);
                }
            }
            EvictionPolicy::Random => {
                // Random eviction
                let mut keys: Vec<_> = entries.keys().cloned().collect();
                while *current_size > target_size && !keys.is_empty() {
                    let index = fastrand::usize(0..keys.len());
                    let key = keys.swap_remove(index);

                    if let Some(entry) = entries.remove(&key) {
                        *current_size = current_size.saturating_sub(entry.size_bytes);
                        evicted_keys.push(key.clone());
                    }
                    access_order.retain(|k| k != key);
                }
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.deletions += evicted_keys.len() as u64;

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size_bytes.write().await;
        let mut access_order = self.access_order.write().await;
        let mut stats = self.stats.write().await;

        let now = Utc::now();
        let mut expired_keys = Vec::new();

        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        for key in &expired_keys {
            if let Some(entry) = entries.remove(key) {
                *current_size = current_size.saturating_sub(entry.size_bytes);
                access_order.retain(|k| k != key);
            }
        }

        stats.deletions += expired_keys.len() as u64;
        Ok(expired_keys.len())
    }

    pub async fn size(&self) -> usize {
        *self.current_size_bytes.read().await
    }

    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

/// Cached repository wrapper
pub struct CachedRepositoryWrapper<T, ID, Repo>
where
    T: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
    Repo: Repository<T, ID>,
{
    inner: Repo,
    cache: MemoryCache<T>,
    cache_key_prefix: String,
}

impl<T, ID, Repo> CachedRepositoryWrapper<T, ID, Repo>
where
    T: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq,
    Repo: Repository<T, ID>,
{
    pub fn new(inner: Repo, cache_config: CacheConfig, cache_key_prefix: &str) -> Self {
        Self {
            inner,
            cache: MemoryCache::new(cache_config),
            cache_key_prefix: cache_key_prefix.to_string(),
        }
    }

    fn make_cache_key(&self, key: &str) -> String {
        format!("{}:{}", self.cache_key_prefix, key)
    }

    async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
        // In a real implementation, this would support pattern-based invalidation
        // For now, we'll just clear all
        self.cache.clear().await
    }
}

#[async_trait]
impl<T, ID, Repo> Repository<T, ID> for CachedRepositoryWrapper<T, ID, Repo>
where
    T: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq + ToString,
    Repo: Repository<T, ID>,
{
    async fn create(&self, entity: &T) -> Result<ID> {
        let result = self.inner.create(entity).await?;

        // Invalidate caches that might be affected by creation
        self.invalidate_pattern("all").await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: &ID) -> Result<Option<T>> {
        let cache_key = self.make_cache_key(&format!("id:{}", id));

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(Some(cached));
        }

        // Cache miss - fetch from repository
        let result = self.inner.find_by_id(id).await?;

        // Cache the result
        if let Some(ref entity) = result {
            self.cache.put(&cache_key, entity.clone(), None).await?;
        }

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<T>> {
        let cache_key = self.make_cache_key("all");

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - fetch from repository
        let result = self.inner.find_all().await?;

        // Cache the result
        self.cache.put(&cache_key, result.clone(), None).await?;

        Ok(result)
    }

    async fn update(&self, id: &ID, entity: &T) -> Result<T> {
        let result = self.inner.update(id, entity).await?;

        // Update caches
        let cache_key = self.make_cache_key(&format!("id:{}", id));
        self.cache.put(&cache_key, result.clone(), None).await?;

        // Invalidate other caches that might be affected
        self.invalidate_pattern("all").await?;

        Ok(result)
    }

    async fn delete(&self, id: &ID) -> Result<bool> {
        let result = self.inner.delete(id).await?;

        if result {
            // Remove from cache
            let cache_key = self.make_cache_key(&format!("id:{}", id));
            self.cache.remove(&cache_key).await?;

            // Invalidate other caches
            self.invalidate_pattern("all").await?;
        }

        Ok(result)
    }

    async fn exists(&self, id: &ID) -> Result<bool> {
        let cache_key = self.make_cache_key(&format!("exists:{}", id));

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - check repository
        let result = self.inner.exists(id).await?;

        // Cache the result (with shorter TTL for existence checks)
        self.cache.put(&cache_key, result, Some(60000)).await?; // 1 minute TTL

        Ok(result)
    }

    async fn count(&self) -> Result<u64> {
        let cache_key = self.make_cache_key("count");

        // Try cache first
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - fetch from repository
        let result = self.inner.count().await?;

        // Cache the result (with shorter TTL for count)
        self.cache.put(&cache_key, result, Some(120000)).await?; // 2 minutes TTL

        Ok(result)
    }
}

#[async_trait]
impl<T, ID, Repo> CachedRepository<T, ID> for CachedRepositoryWrapper<T, ID, Repo>
where
    T: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>,
    ID: Send + Sync + Clone + PartialEq + ToString,
    Repo: Repository<T, ID>,
{
    async fn invalidate_cache(&self, id: &ID) -> Result<()> {
        let cache_key = self.make_cache_key(&format!("id:{}", id));
        self.cache.remove(&cache_key).await?;
        Ok(())
    }

    async fn invalidate_all_cache(&self) -> Result<()> {
        self.cache.clear().await?;
        Ok(())
    }

    async fn preload_cache(&self, ids: &[ID]) -> Result<()> {
        // Load entities in parallel
        let futures: Vec<_> = ids.iter()
            .map(|id| {
                let repo = &self.inner;
                let cache = &self.cache;
                let cache_key_prefix = &self.cache_key_prefix;
                async move {
                    if let Ok(Some(entity)) = repo.find_by_id(id).await {
                        let cache_key = format!("{}:id:{}", cache_key_prefix, id);
                        let _ = cache.put(&cache_key, entity, None).await;
                    }
                }
            })
            .collect();

        futures::future::join_all(futures).await;
        Ok(())
    }

    async fn cache_stats(&self) -> Result<CacheStats> {
        Ok(self.cache.stats().await)
    }
}