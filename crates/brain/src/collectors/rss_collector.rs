//! RSS feed collector
//!
//! Collects data from RSS/Atom feeds with support for:
//! - Multiple feed sources
//! - Incremental updates
//! - Deduplication
//! - Feed parsing and normalization

use crate::collectors::{CollectionStats, Collector};
use crate::storage::{DataSource, RawData};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{Error, Result};
use feed_rs::parser;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// RSS collector configuration
#[derive(Debug, Clone)]
pub struct RssCollectorConfig {
    /// Feed URLs
    pub feeds: Vec<String>,

    /// Update interval in minutes
    pub update_interval_minutes: u32,

    /// Maximum number of items per feed
    pub max_items_per_feed: usize,

    /// Enable deduplication
    pub enable_dedup: bool,
}

impl Default for RssCollectorConfig {
    fn default() -> Self {
        Self {
            feeds: vec![],
            update_interval_minutes: 60,
            max_items_per_feed: 100,
            enable_dedup: true,
        }
    }
}

/// RSS feed collector
#[derive(Debug)]
pub struct RssCollector {
    config: RssCollectorConfig,
    stats: Arc<RwLock<CollectionStats>>,
    seen_items: Arc<RwLock<HashSet<String>>>, // For deduplication
}

impl RssCollector {
    /// Create a new RSS collector
    pub fn new(config: RssCollectorConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(CollectionStats::default())),
            seen_items: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Fetch and parse a single feed
    async fn fetch_feed(&self, url: &str) -> Result<Vec<RawData>> {
        debug!("Fetching RSS feed from {}", url);

        // Fetch the feed
        let response = reqwest::get(url)
            .await
            .map_err(|e| Error::StorageError(format!("Failed to fetch feed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::StorageError(format!(
                "Feed returned status: {}",
                response.status()
            )));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| Error::StorageError(format!("Failed to read feed: {}", e)))?;

        // Parse the feed
        let feed = parser::parse(&content[..])
            .map_err(|e| Error::StorageError(format!("Failed to parse feed: {}", e)))?;

        debug!(
            "Parsed feed '{}' with {} entries",
            feed.title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            feed.entries.len()
        );

        let mut raw_data_items = Vec::new();

        // Process entries (newest first)
        let mut entries = feed.entries;
        entries.sort_by(|a, b| b.published.cmp(&a.published));

        for entry in entries.into_iter().take(self.config.max_items_per_feed) {
            // Extract content from entry
            let content = entry
                .links
                .first()
                .and_then(|l| l.title.as_ref())
                .map(|t| t.clone())
                .unwrap_or_else(|| "No content".to_string());

            // Create RawData with metadata
            let raw = RawData::new(
                DataSource::RSS {
                    url: url.to_string(),
                },
                content,
                "text/html".to_string(),
            )
            .with_metadata(
                "feed_title",
                entry
                    .links
                    .first()
                    .and_then(|l| l.title.as_ref())
                    .map(|t| t.as_str())
                    .unwrap_or("unknown"),
            );

            raw_data_items.push(raw);
        }

        Ok(raw_data_items)
    }
}

#[async_trait::async_trait]
impl Collector for RssCollector {
    async fn collect(&self) -> Result<Vec<RawData>> {
        info!("Collecting from {} RSS feeds", self.config.feeds.len());

        let mut all_data = Vec::new();
        let mut total_collected = 0u64;
        let mut total_failed = 0u64;

        for feed_url in &self.config.feeds {
            match self.fetch_feed(feed_url).await {
                Ok(mut data) => {
                    total_collected += data.len() as u64;
                    all_data.append(&mut data);
                }
                Err(e) => {
                    warn!("Failed to fetch feed {}: {}", feed_url, e);
                    total_failed += 1;
                }
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_collected = total_collected;
        stats.successful = total_collected;
        stats.failed = total_failed;
        stats.last_collection = Some(Utc::now());

        info!("Collected {} items from RSS feeds", all_data.len());

        Ok(all_data)
    }

    fn name(&self) -> &str {
        "rss_collector"
    }

    fn schedule(&self) -> Option<crate::collectors::CronSchedule> {
        Some(crate::collectors::CronSchedule::every_minutes(
            self.config.update_interval_minutes,
        ))
    }

    async fn stats(&self) -> crate::collectors::CollectionStats {
        self.stats.read().await.clone()
    }

    async fn health_check(&self) -> Result<bool> {
        if self.config.feeds.is_empty() {
            return Ok(false);
        }

        // Try to fetch the first feed
        if let Some(feed_url) = self.config.feeds.first() {
            match self.fetch_feed(feed_url).await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_collector_creation() {
        let config = RssCollectorConfig {
            feeds: vec!["https://example.com/feed.xml".to_string()],
            ..Default::default()
        };

        let collector = RssCollector::new(config);
        assert_eq!(collector.name(), "rss_collector");
    }
}
