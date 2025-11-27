//! Redis cache backend

use crate::{CacheBackend, CacheError, CacheStats};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Redis-based distributed cache
pub struct RedisCache {
    conn: ConnectionManager,
    prefix: String,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl RedisCache {
    /// Create a new Redis cache connection
    pub async fn new(url: &str, prefix: String) -> Result<Self, CacheError> {
        let client = redis::Client::open(url)
            .map_err(|e| CacheError::Connection(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::Connection(format!("Failed to connect to Redis: {}", e)))?;

        tracing::info!("Connected to Redis at {}", url);

        Ok(Self {
            conn,
            prefix,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.conn.clone();

        let result: Option<String> = conn.get(&prefixed).await?;

        match result {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(value))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &str,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.conn.clone();

        let seconds = ttl.as_secs() as i64;
        if seconds > 0 {
            conn.set_ex(&prefixed, value, seconds as u64).await?;
        } else {
            conn.set(&prefixed, value).await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.conn.clone();

        let deleted: i64 = conn.del(&prefixed).await?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let prefixed = self.prefixed_key(key);
        let mut conn = self.conn.clone();

        let exists: bool = conn.exists(&prefixed).await?;
        Ok(exists)
    }

    async fn clear(&self) -> Result<(), CacheError> {
        // Note: This only clears keys with our prefix for safety
        let mut conn = self.conn.clone();
        let pattern = format!("{}*", self.prefix);

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys).await?;
        }

        tracing::info!("Cleared cache keys with prefix: {}", self.prefix);
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        let mut conn = self.conn.clone();

        // Try to get the number of keys with our prefix
        let pattern = format!("{}*", self.prefix);
        let entries: u64 = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async::<Vec<String>>(&mut conn)
            .await
            .map(|keys| keys.len() as u64)
            .unwrap_or(0);

        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries,
            backend: "redis".to_string(),
        }
    }

    fn backend_name(&self) -> &'static str {
        "redis"
    }
}

/// Redis pub/sub support for cache invalidation across instances
pub struct RedisPubSub {
    conn: ConnectionManager,
    channel: String,
}

impl RedisPubSub {
    pub async fn new(url: &str, channel: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(url)
            .map_err(|e| CacheError::Connection(format!("Failed to create Redis client: {}", e)))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::Connection(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            conn,
            channel: channel.to_string(),
        })
    }

    /// Publish cache invalidation message
    pub async fn publish_invalidation(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        conn.publish::<_, _, ()>(&self.channel, key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Redis tests require a running Redis instance
    // Use integration tests with #[ignore] or test containers

    #[test]
    fn test_prefixed_key() {
        // Can't easily test without async runtime and Redis connection
        // These would be integration tests
    }
}
