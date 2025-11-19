//! Caching layer for frequently accessed data
//!
//! Provides in-memory caching with TTL (Time-To-Live) support.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::models::{Prompt, QualityScore, EfficiencyMetrics};

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// Generic cache with TTL support
pub struct Cache<T: Clone> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
    max_size: usize,
}

impl<T: Clone> Cache<T> {
    /// Create a new cache with specified TTL and max size
    pub fn new(ttl_secs: u64, max_size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
            max_size,
        }
    }

    /// Get a value from cache
    pub fn get(&self, key: &str) -> Option<T> {
        let data = self.data.read().ok()?;
        if let Some(entry) = data.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Set a value in cache
    pub fn set(&self, key: String, value: T) {
        if let Ok(mut data) = self.data.write() {
            // Evict expired entries if at capacity
            if data.len() >= self.max_size {
                self.evict_expired(&mut data);
            }

            // If still at capacity, remove oldest entry
            if data.len() >= self.max_size {
                if let Some(oldest_key) = data.keys().next().cloned() {
                    data.remove(&oldest_key);
                }
            }

            data.insert(
                key,
                CacheEntry {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
        }
    }

    /// Remove a value from cache
    pub fn remove(&self, key: &str) {
        if let Ok(mut data) = self.data.write() {
            data.remove(key);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.data.read().map(|d| d.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Evict expired entries
    fn evict_expired(&self, data: &mut HashMap<String, CacheEntry<T>>) {
        let now = Instant::now();
        data.retain(|_, entry| entry.expires_at > now);
    }
}

/// Prompt tracking cache manager
pub struct CacheManager {
    /// Cache for prompts
    pub prompts: Cache<Prompt>,
    /// Cache for quality scores
    pub quality_scores: Cache<QualityScore>,
    /// Cache for efficiency metrics
    pub efficiency_metrics: Cache<EfficiencyMetrics>,
    /// Cache for prompt counts
    pub counts: Cache<usize>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new(300, 1000) // 5 minutes TTL, 1000 max entries
    }
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(ttl_secs: u64, max_size: usize) -> Self {
        Self {
            prompts: Cache::new(ttl_secs, max_size),
            quality_scores: Cache::new(ttl_secs, max_size),
            efficiency_metrics: Cache::new(ttl_secs, max_size),
            counts: Cache::new(ttl_secs, 100),
        }
    }

    /// Invalidate all caches
    pub fn invalidate_all(&self) {
        self.prompts.clear();
        self.quality_scores.clear();
        self.efficiency_metrics.clear();
        self.counts.clear();
    }

    /// Invalidate caches for a specific prompt
    pub fn invalidate_prompt(&self, prompt_id: &str) {
        self.prompts.remove(prompt_id);
        self.quality_scores.remove(prompt_id);
        self.efficiency_metrics.remove(prompt_id);
        self.counts.remove("total");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_get() {
        let cache: Cache<String> = Cache::new(60, 100);

        cache.set("key1".to_string(), "value1".to_string());

        let result = cache.get("key1");
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache: Cache<String> = Cache::new(60, 100);

        let result = cache.get("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_remove() {
        let cache: Cache<String> = Cache::new(60, 100);

        cache.set("key1".to_string(), "value1".to_string());
        cache.remove("key1");

        let result = cache.get("key1");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: Cache<String> = Cache::new(60, 100);

        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());
        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_max_size() {
        let cache: Cache<i32> = Cache::new(60, 2);

        cache.set("1".to_string(), 1);
        cache.set("2".to_string(), 2);
        cache.set("3".to_string(), 3);

        // Should have evicted one entry
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_manager() {
        let manager = CacheManager::default();

        let mut prompt = Prompt::new("test".to_string());
        prompt.content_hash = "hash".to_string();

        manager.prompts.set("id1".to_string(), prompt.clone());

        let cached = manager.prompts.get("id1");
        assert!(cached.is_some());

        manager.invalidate_prompt("id1");

        let cached = manager.prompts.get("id1");
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_len() {
        let cache: Cache<i32> = Cache::new(60, 100);

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        cache.set("1".to_string(), 1);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }
}
