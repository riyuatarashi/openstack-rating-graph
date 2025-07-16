//! Cache module for OpenStack data to reduce API requests

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, debug};
use sha2::{Sha256, Digest};

/// Cache entry containing data and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: HashMap<String, f64>,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(data: HashMap<String, f64>, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }

    /// Get the remaining time until expiration
    pub fn remaining_ttl(&self) -> Duration {
        self.ttl.saturating_sub(self.created_at.elapsed())
    }
}

/// Simple in-memory cache for OpenStack data
#[derive(Debug)]
pub struct OpenStackCache {
    /// Cache storage
    cache: RwLock<HashMap<String, CacheEntry>>,
    /// Default TTL for cache entries
    default_ttl: Duration,
}

impl OpenStackCache {
    /// Create a new cache instance
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }

    /// Generate a cache key based on the command and parameters
    pub fn generate_key(&self, command: &str, params: &[String]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(command);
        for param in params {
            hasher.update(param);
        }
        format!("{:x}", hasher.finalize())
    }

    /// Get data from cache if available and valid
    pub async fn get(&self, key: &str) -> Option<HashMap<String, f64>> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.is_valid() {
                debug!("Cache hit for key: {}", key);
                info!("Using cached data (TTL remaining: {:?})", entry.remaining_ttl());
                return Some(entry.data.clone());
            } else {
                debug!("Cache entry expired for key: {}", key);
            }
        }
        debug!("Cache miss for key: {}", key);
        None
    }

    /// Store data in cache
    pub async fn set(&self, key: String, data: HashMap<String, f64>) {
        let entry = CacheEntry::new(data, self.default_ttl);
        let mut cache = self.cache.write().await;
        cache.insert(key.clone(), entry);
        info!("Cached data for key: {} (TTL: {:?})", key, self.default_ttl);
    }

    /// Store data in cache with custom TTL
    pub async fn set_with_ttl(&self, key: String, data: HashMap<String, f64>, ttl: Duration) {
        let entry = CacheEntry::new(data, ttl);
        let mut cache = self.cache.write().await;
        cache.insert(key.clone(), entry);
        info!("Cached data for key: {} (TTL: {:?})", key, ttl);
    }

    /// Clear expired entries from cache
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_size = cache.len();
        cache.retain(|_, entry| entry.is_valid());
        let final_size = cache.len();
        if initial_size != final_size {
            info!("Cleaned up {} expired cache entries", initial_size - final_size);
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();
        info!("Cleared {} cache entries", count);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let mut valid_entries = 0;
        let mut expired_entries = 0;

        for entry in cache.values() {
            if entry.is_valid() {
                valid_entries += 1;
            } else {
                expired_entries += 1;
            }
        }

        CacheStats {
            total_entries,
            valid_entries,
            expired_entries,
            default_ttl: self.default_ttl,
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub default_ttl: Duration,
}

impl Default for OpenStackCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes default TTL
    }
}
