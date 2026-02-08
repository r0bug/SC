//! Caching layer supporting both in-memory (moka) and Redis backends
//!
//! # Features
//! - `memory` (default): In-memory cache using moka
//! - `redis`: Redis-based distributed cache
//!
//! # Usage
//! ```rust,ignore
//! use cache_layer::{Cache, CacheConfig, create_cache};
//!
//! // Create with default memory backend
//! let cache = create_cache(CacheConfig::default()).await?;
//! cache.set("key", "value", Duration::from_secs(3600)).await?;
//! let value: Option<String> = cache.get("key").await?;
//! ```

mod error;
mod memory;
#[cfg(feature = "redis")]
mod redis_backend;

pub use error::CacheError;
pub use memory::MemoryCache;
#[cfg(feature = "redis")]
pub use redis_backend::RedisCache;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Cache backend trait (dyn-compatible version using String values)
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a raw string value from cache
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError>;

    /// Set a raw string value in cache with TTL
    async fn set_raw(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;

    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear all cache entries (use with caution)
    async fn clear(&self) -> Result<(), CacheError>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Get the backend name
    fn backend_name(&self) -> &'static str;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub backend: String,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Redis URL (only used with redis feature)
    pub redis_url: Option<String>,

    /// Maximum cache size (entries) for memory backend
    pub max_capacity: u64,

    /// Default TTL for cache entries
    pub default_ttl: Duration,

    /// Time to idle (evict entries not accessed within this time)
    pub time_to_idle: Option<Duration>,

    /// Prefix for all cache keys
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: None,
            max_capacity: 10_000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            time_to_idle: Some(Duration::from_secs(1800)), // 30 minutes
            key_prefix: "sc:".to_string(),
        }
    }
}

impl CacheConfig {
    /// Create config for Redis backend
    pub fn redis(url: &str) -> Self {
        Self {
            redis_url: Some(url.to_string()),
            ..Default::default()
        }
    }

    /// Create config for memory backend
    pub fn memory() -> Self {
        Self::default()
    }

    /// Set custom key prefix
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.key_prefix = prefix.to_string();
        self
    }

    /// Set max capacity
    pub fn with_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Set default TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }
}

/// Type-safe cache wrapper around CacheBackend
#[derive(Clone)]
pub struct Cache {
    backend: Arc<dyn CacheBackend>,
}

impl Cache {
    pub fn new(backend: Arc<dyn CacheBackend>) -> Self {
        Self { backend }
    }

    /// Get a typed value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        match self.backend.get_raw(key).await? {
            Some(raw) => {
                let value: T = serde_json::from_str(&raw)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a typed value in cache
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let raw = serde_json::to_string(value)?;
        self.backend.set_raw(key, &raw, ttl).await
    }

    /// Delete a value
    pub async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        self.backend.delete(key).await
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        self.backend.exists(key).await
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<(), CacheError> {
        self.backend.clear().await
    }

    /// Get statistics
    pub async fn stats(&self) -> CacheStats {
        self.backend.stats().await
    }

    /// Get backend name
    pub fn backend_name(&self) -> &'static str {
        self.backend.backend_name()
    }
}

/// Create a cache instance based on configuration
pub async fn create_cache(config: CacheConfig) -> Result<Cache, CacheError> {
    #[cfg(feature = "redis")]
    if let Some(redis_url) = &config.redis_url {
        tracing::info!("Creating Redis cache backend");
        let backend = RedisCache::new(redis_url, config.key_prefix.clone()).await?;
        return Ok(Cache::new(Arc::new(backend)));
    }

    #[cfg(feature = "memory")]
    {
        tracing::info!(
            "Creating in-memory cache backend (max {} entries)",
            config.max_capacity
        );
        let backend = MemoryCache::new(config);
        Ok(Cache::new(Arc::new(backend)))
    }

    #[cfg(not(any(feature = "memory", feature = "redis")))]
    {
        compile_error!("At least one cache backend feature must be enabled");
    }
}

/// Create cache from environment variables
/// - REDIS_URL: Redis connection URL (enables Redis backend)
/// - CACHE_PREFIX: Key prefix (default: "sc:")
/// - CACHE_MAX_CAPACITY: Max entries for memory cache (default: 10000)
pub async fn create_cache_from_env() -> Result<Cache, CacheError> {
    let redis_url = std::env::var("REDIS_URL").ok();
    let prefix = std::env::var("CACHE_PREFIX").unwrap_or_else(|_| "sc:".to_string());
    let max_capacity = std::env::var("CACHE_MAX_CAPACITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let config = CacheConfig {
        redis_url,
        key_prefix: prefix,
        max_capacity,
        ..Default::default()
    };

    create_cache(config).await
}

/// Specialized cache for AI responses
#[derive(Clone)]
pub struct AiResponseCache {
    cache: Cache,
    ttl: Duration,
}

impl AiResponseCache {
    pub fn new(cache: Cache, ttl: Duration) -> Self {
        Self { cache, ttl }
    }

    /// Cache key for AI prompt
    fn cache_key(&self, prompt_hash: &str) -> String {
        format!("ai:response:{}", prompt_hash)
    }

    /// Get cached AI response
    pub async fn get(&self, prompt_hash: &str) -> Result<Option<String>, CacheError> {
        self.cache.get(&self.cache_key(prompt_hash)).await
    }

    /// Cache AI response
    pub async fn set(&self, prompt_hash: &str, response: &str) -> Result<(), CacheError> {
        self.cache
            .set(&self.cache_key(prompt_hash), &response, self.ttl)
            .await
    }
}

/// Session cache for authentication tokens
#[derive(Clone)]
pub struct SessionCache {
    cache: Cache,
    session_ttl: Duration,
}

impl SessionCache {
    pub fn new(cache: Cache, session_ttl: Duration) -> Self {
        Self { cache, session_ttl }
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    /// Get session data
    pub async fn get_session<T: DeserializeOwned>(
        &self,
        session_id: &str,
    ) -> Result<Option<T>, CacheError> {
        self.cache.get(&self.session_key(session_id)).await
    }

    /// Store session data
    pub async fn set_session<T: Serialize>(
        &self,
        session_id: &str,
        data: &T,
    ) -> Result<(), CacheError> {
        self.cache
            .set(&self.session_key(session_id), data, self.session_ttl)
            .await
    }

    /// Delete session
    pub async fn delete_session(&self, session_id: &str) -> Result<bool, CacheError> {
        self.cache.delete(&self.session_key(session_id)).await
    }

    /// Extend session TTL
    pub async fn refresh_session<T: Serialize + DeserializeOwned>(
        &self,
        session_id: &str,
    ) -> Result<bool, CacheError> {
        if let Some(data) = self.get_session::<T>(session_id).await? {
            self.set_session(session_id, &data).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Rate limiter using cache
#[derive(Clone)]
pub struct RateLimiter {
    cache: Cache,
    window: Duration,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(cache: Cache, window: Duration, max_requests: u32) -> Self {
        Self {
            cache,
            window,
            max_requests,
        }
    }

    fn rate_key(&self, identifier: &str) -> String {
        format!("rate:{}", identifier)
    }

    /// Check if request is allowed and increment counter
    pub async fn check(&self, identifier: &str) -> Result<RateLimitResult, CacheError> {
        let key = self.rate_key(identifier);

        let current: Option<u32> = self.cache.get(&key).await?;
        let count = current.unwrap_or(0);

        if count >= self.max_requests {
            return Ok(RateLimitResult::Limited {
                current: count,
                limit: self.max_requests,
            });
        }

        let new_count = count + 1;
        self.cache.set(&key, &new_count, self.window).await?;

        Ok(RateLimitResult::Allowed {
            remaining: self.max_requests - new_count,
            limit: self.max_requests,
        })
    }

    /// Reset rate limit for identifier
    pub async fn reset(&self, identifier: &str) -> Result<(), CacheError> {
        let key = self.rate_key(identifier);
        self.cache.delete(&key).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: u32, limit: u32 },
    Limited { current: u32, limit: u32 },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_basic() {
        let cache = create_cache(CacheConfig::default()).await.unwrap();

        // Set and get
        cache
            .set(
                "test_key",
                &"test_value".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Delete
        let deleted = cache.delete("test_key").await.unwrap();
        assert!(deleted);

        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_ai_response_cache() {
        let cache = create_cache(CacheConfig::default()).await.unwrap();
        let ai_cache = AiResponseCache::new(cache, Duration::from_secs(3600));

        ai_cache
            .set("prompt_hash_123", "AI response text")
            .await
            .unwrap();
        let response = ai_cache.get("prompt_hash_123").await.unwrap();
        assert_eq!(response, Some("AI response text".to_string()));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let cache = create_cache(CacheConfig::default()).await.unwrap();
        let limiter = RateLimiter::new(cache, Duration::from_secs(60), 3);

        // Should allow first 3 requests
        for _ in 0..3 {
            let result = limiter.check("user_123").await.unwrap();
            assert!(result.is_allowed());
        }

        // 4th request should be limited
        let result = limiter.check("user_123").await.unwrap();
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_session_cache() {
        let cache = create_cache(CacheConfig::default()).await.unwrap();
        let sessions = SessionCache::new(cache, Duration::from_secs(3600));

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
        struct SessionData {
            user_id: String,
            roles: Vec<String>,
        }

        let data = SessionData {
            user_id: "user_123".to_string(),
            roles: vec!["admin".to_string()],
        };

        sessions.set_session("session_abc", &data).await.unwrap();
        let retrieved: Option<SessionData> = sessions.get_session("session_abc").await.unwrap();
        assert_eq!(retrieved, Some(data));

        sessions.delete_session("session_abc").await.unwrap();
        let retrieved: Option<SessionData> = sessions.get_session("session_abc").await.unwrap();
        assert!(retrieved.is_none());
    }
}
