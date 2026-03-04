//! Data collectors for the Brain system
//!
//! This module provides collectors for various data sources:
//! - API endpoints
//! - RSS feeds
//! - File uploads
//! - Web scraping
//!
//! Each collector implements the `Collector` trait and can be scheduled
//! for automatic data collection.

pub mod api_collector;
pub mod file_handler;
pub mod rss_collector;

use crate::storage::{DataSource, RawData, StorageId};
use chrono::{DateTime, Utc};
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Total items collected
    pub total_collected: u64,

    /// Successful collections
    pub successful: u64,

    /// Failed collections
    pub failed: u64,

    /// Last collection time
    pub last_collection: Option<DateTime<Utc>>,

    /// Last error
    pub last_error: Option<String>,
}

impl Default for CollectionStats {
    fn default() -> Self {
        Self {
            total_collected: 0,
            successful: 0,
            failed: 0,
            last_collection: None,
            last_error: None,
        }
    }
}

/// Cron schedule for periodic collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    /// Cron expression (e.g., "0 */5 * * * *" for every 5 minutes)
    pub expression: String,

    /// Timezone
    pub timezone: String,
}

impl CronSchedule {
    /// Create a new cron schedule
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            timezone: "UTC".to_string(),
        }
    }

    /// Create a schedule that runs every N minutes
    pub fn every_minutes(minutes: u32) -> Self {
        Self::new(format!("0 */{} * * * *", minutes))
    }

    /// Create a schedule that runs every hour
    pub fn hourly() -> Self {
        Self::new("0 * * * *".to_string())
    }

    /// Create a schedule that runs every day at midnight
    pub fn daily() -> Self {
        Self::new("0 0 * * *".to_string())
    }
}

/// Collection trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    /// Manual trigger
    Manual,

    /// Scheduled trigger with cron
    Scheduled(CronSchedule),

    /// Event-based trigger
    Event(String),
}

/// Core collector trait
#[async_trait::async_trait]
pub trait Collector: Send + Sync {
    /// Collect data from the source
    async fn collect(&self) -> Result<Vec<RawData>>;

    /// Get the collector name
    fn name(&self) -> &str;

    /// Get the collector's schedule (if any)
    fn schedule(&self) -> Option<CronSchedule> {
        None
    }

    /// Get collection statistics
    async fn stats(&self) -> CollectionStats {
        CollectionStats::default()
    }

    /// Check if collector is healthy
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    /// Get the data source identifier
    fn source_id(&self) -> String {
        self.name().to_string()
    }
}

/// Collection result with metadata
#[derive(Debug, Clone)]
pub struct CollectionResult {
    /// Source of the collection
    pub source: String,

    /// Number of items collected
    pub count: usize,

    /// IDs of stored raw data items
    pub data_ids: Vec<StorageId>,

    /// Collection timestamp
    pub timestamp: DateTime<Utc>,

    /// Any errors that occurred
    pub errors: Vec<String>,
}

/// Collector registry for managing multiple collectors
pub struct CollectorRegistry {
    collectors: Arc<RwLock<HashMap<String, Arc<dyn Collector>>>>,
}

impl std::fmt::Debug for CollectorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CollectorRegistry { ... }")
    }
}

impl CollectorRegistry {
    /// Create a new collector registry
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a collector
    pub async fn register(&self, collector: Arc<dyn Collector>) {
        let name = collector.name().to_string();
        let mut collectors = self.collectors.write().await;
        collectors.insert(name, collector);
    }

    /// Unregister a collector
    pub async fn unregister(&self, name: &str) -> bool {
        let mut collectors = self.collectors.write().await;
        collectors.remove(name).is_some()
    }

    /// Get a collector by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Collector>> {
        let collectors = self.collectors.read().await;
        collectors.get(name).cloned()
    }

    /// List all collectors
    pub async fn list(&self) -> Vec<String> {
        let collectors = self.collectors.read().await;
        collectors.keys().cloned().collect()
    }

    /// Collect from all registered collectors
    pub async fn collect_all(&self) -> Result<Vec<CollectionResult>> {
        let collectors = self.collectors.read().await;
        let mut results = Vec::new();

        for collector in collectors.values() {
            match collector.collect().await {
                Ok(data) => {
                    // TODO: Store data using RawArchive
                    let result = CollectionResult {
                        source: collector.name().to_string(),
                        count: data.len(),
                        data_ids: data.iter().map(|d| d.id).collect(),
                        timestamp: Utc::now(),
                        errors: vec![],
                    };
                    results.push(result);
                }
                Err(e) => {
                    let result = CollectionResult {
                        source: collector.name().to_string(),
                        count: 0,
                        data_ids: vec![],
                        timestamp: Utc::now(),
                        errors: vec![e.to_string()],
                    };
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Collect from a specific collector
    pub async fn collect_from(&self, name: &str) -> Result<CollectionResult> {
        let collectors = self.collectors.read().await;
        let collector = collectors
            .get(name)
            .ok_or_else(|| Error::NotFound(format!("Collector '{}' not found", name)))?;

        match collector.collect().await {
            Ok(data) => {
                // TODO: Store data using RawArchive
                Ok(CollectionResult {
                    source: name.to_string(),
                    count: data.len(),
                    data_ids: data.iter().map(|d| d.id).collect(),
                    timestamp: Utc::now(),
                    errors: vec![],
                })
            }
            Err(e) => Ok(CollectionResult {
                source: name.to_string(),
                count: 0,
                data_ids: vec![],
                timestamp: Utc::now(),
                errors: vec![e.to_string()],
            }),
        }
    }

    /// Health check all collectors
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let collectors = self.collectors.read().await;
        let mut results = HashMap::new();

        for (name, collector) in collectors.iter() {
            let healthy = collector.health_check().await.unwrap_or(false);
            results.insert(name.clone(), healthy);
        }

        results
    }
}

impl Default for CollectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCollector {
        name: String,
        data: Vec<RawData>,
    }

    #[async_trait::async_trait]
    impl Collector for MockCollector {
        async fn collect(&self) -> Result<Vec<RawData>> {
            Ok(self.data.clone())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn schedule(&self) -> Option<CronSchedule> {
            Some(CronSchedule::every_minutes(5))
        }
    }

    #[tokio::test]
    async fn test_collector_registry() {
        let registry = CollectorRegistry::new();

        let collector = Arc::new(MockCollector {
            name: "test_collector".to_string(),
            data: vec![],
        });

        registry.register(collector).await;

        let collectors = registry.list().await;
        assert_eq!(collectors.len(), 1);
        assert!(collectors.contains(&"test_collector".to_string()));
    }

    #[test]
    fn test_cron_schedule() {
        let schedule = CronSchedule::every_minutes(5);
        assert_eq!(schedule.expression, "0 */5 * * * *");

        let hourly = CronSchedule::hourly();
        assert_eq!(hourly.expression, "0 * * * *");

        let daily = CronSchedule::daily();
        assert_eq!(daily.expression, "0 0 * * *");
    }
}
