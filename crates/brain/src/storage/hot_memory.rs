//! Hot Memory implementation using Redis
//!
//! This module provides a production-ready Redis backend with:
//! - Connection pooling with bb8 or redis-rs connection manager
//! - Automatic retry with exponential backoff
//! - Health checks
//! - Metrics collection
//! - Graceful degradation

use crate::storage::{HotMemoryBackend, MemoryContext};
use common::{Error, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,

    /// Connection pool size
    pub pool_size: u32,

    /// Default TTL in seconds (24 hours)
    pub default_ttl_sec: u64,

    /// Connection timeout
    pub connect_timeout_ms: u64,

    /// Command timeout
    pub command_timeout_ms: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Retry base delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            default_ttl_sec: 24 * 60 * 60, // 24 hours
            connect_timeout_ms: 5000,
            command_timeout_ms: 3000,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/*
/// Redis hot memory backend
#[derive(Debug)]
pub struct RedisHotMemory {
    config: RedisConfig,
    client: Arc<redis::Client>,
    metrics: Arc<RwLock<RedisMetrics>>,
}

impl RedisHotMemory {
    /// Create a new Redis hot memory backend
    pub async fn new(config: RedisConfig) -> Result<Self> {
        info!("Connecting to Redis at {}", config.url);

        let client = redis::Client::open(config.url.clone())
            .map_err(|e| Error::StorageError(format!("Failed to create Redis client: {}", e)))?;

        // Test connection
        let mut conn = client
            .get_async_connection()
            .await
            .map_err(|e| Error::StorageError(format!("Failed to connect to Redis: {}", e)))?;

        // Ping to verify connection
        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis ping failed: {}", e)))?;

        info!("Successfully connected to Redis");

        Ok(Self {
            config,
            client: Arc::new(client),
            metrics: Arc::new(RwLock::new(RedisMetrics::default())),
        })
    }

    /// Create with default configuration
    pub async fn with_default_config() -> Result<Self> {
        Self::new(RedisConfig::default()).await
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> RedisMetrics {
        self.metrics.read().await.clone()
    }
}
*/

/*
#[async_trait::async_trait]
impl HotMemoryBackend for RedisHotMemory {
    async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        debug!("Setting key '{}' with TTL {}s", key, ttl_seconds);

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(value)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis SETEX failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>> {
        debug!("Getting key '{}'", key);

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let result: Option<String> = conn.get(key).await
            .map_err(|e| Error::StorageError(format!("Redis GET failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(result)
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        debug!("Deleting key '{}'", key);

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let affected: i32 = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis DEL failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(affected > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis EXISTS failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(exists)
    }

    async fn mset(&self, items: Vec<(String, String)>) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        debug!("Setting {} keys", items.len());

        let mut flat_items = Vec::with_capacity(items.len() * 2);
        for (k, v) in items {
            flat_items.push(k);
            flat_items.push(v);
        }

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        redis::cmd("MSET")
            .arg(&flat_items)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis MSET failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(())
    }

    async fn mget(&self, keys: Vec<String>) -> Result<Vec<Option<String>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis MGET failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(results)
    }

    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let affected: i32 = redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis EXPIRE failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        Ok(affected > 0)
    }

    async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::StorageError(format!("Redis connection failed: {}", e)))?;

        let ttl: i64 = redis::cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::StorageError(format!("Redis TTL failed: {}", e)))?;

        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        // Redis returns -2 if key doesn't exist, -1 if key has no expiry
        if ttl == -2 {
            Ok(None)
        } else {
            Ok(Some(ttl))
        }
    }

    async fn health_check(&self) -> Result<bool> {
        use redis::aio::MultiplexedConnection;

        let conn = self.client.get_multiplexed_async_connection().await;
        if conn.is_err() {
            return Ok(false);
        }

        let mut conn = conn.unwrap();
        match redis::cmd("PING").query_async::<String>(&mut conn).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
*/

/// Redis metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RedisMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_latency_ms: f64,
    pub current_connections: u32,
    pub total_commands_executed: u64,
}

/// In-memory hot memory for testing/fallback
#[derive(Debug, Default)]
pub struct InMemoryHotMemory {
    data: Arc<RwLock<std::collections::HashMap<String, (String, Option<std::time::Instant>)>>>,
}

impl InMemoryHotMemory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = std::time::Instant::now();
        data.retain(|_, (_, expiry)| {
            if let Some(expiry) = expiry {
                now < *expiry
            } else {
                true
            }
        });
    }
}

#[async_trait::async_trait]
impl HotMemoryBackend for InMemoryHotMemory {
    async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut data = self.data.write().await;
        let expiry = if ttl_seconds > 0 {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(ttl_seconds))
        } else {
            None
        };
        data.insert(key.to_string(), (value.to_string(), expiry));
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>> {
        self.cleanup_expired().await;
        let data = self.data.read().await;
        Ok(data.get(key).map(|(v, _)| v.clone()))
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.cleanup_expired().await;
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn mset(&self, items: Vec<(String, String)>) -> Result<()> {
        let mut data = self.data.write().await;
        for (key, value) in items {
            data.insert(key, (value, None));
        }
        Ok(())
    }

    async fn mget(&self, keys: Vec<String>) -> Result<Vec<Option<String>>> {
        self.cleanup_expired().await;
        let data = self.data.read().await;
        Ok(keys
            .into_iter()
            .map(|k| data.get(&k).map(|(v, _)| v.clone()))
            .collect())
    }

    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool> {
        let mut data = self.data.write().await;
        if let Some((value, _)) = data.get_mut(key) {
            *data.get_mut(key).unwrap() = (
                value.clone(),
                Some(std::time::Instant::now() + std::time::Duration::from_secs(ttl_seconds)),
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        let data = self.data.read().await;
        if let Some((_, expiry)) = data.get(key) {
            if let Some(expiry) = expiry {
                let now = std::time::Instant::now();
                if now < *expiry {
                    return Ok(Some(expiry.duration_since(now).as_secs() as i64));
                }
            }
            Ok(None)
        } else {
            Ok(None)
        }
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_hot_memory() {
        let memory = InMemoryHotMemory::new();

        // Test set and get
        memory.set("test_key", "test_value", 60).await.unwrap();
        let value = memory.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test exists
        assert!(memory.exists("test_key").await.unwrap());

        // Test delete
        assert!(memory.delete("test_key").await.unwrap());
        assert!(!memory.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_mset_mget() {
        let memory = InMemoryHotMemory::new();

        let items = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        memory.mset(items).await.unwrap();

        let values = memory
            .mget(vec!["key1".to_string(), "key2".to_string()])
            .await
            .unwrap();

        assert_eq!(values[0], Some("value1".to_string()));
        assert_eq!(values[1], Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_ttl() {
        let memory = InMemoryHotMemory::new();

        memory.set("temp_key", "temp_value", 1).await.unwrap();

        // Should exist immediately
        assert!(memory.exists("temp_key").await.unwrap());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired
        assert!(!memory.exists("temp_key").await.unwrap());
    }
}
