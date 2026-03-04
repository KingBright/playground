//! API data collector
//!
//! Collects data from REST APIs with support for:
//! - Pagination
//! - Rate limiting
//! - Authentication
//! - Custom headers
//! - Retry logic

use crate::collectors::{CollectionStats, Collector};
use crate::storage::{DataSource, RawData};
use async_trait::async_trait;
use common::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// API collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCollectorConfig {
    /// API endpoint URL
    pub endpoint: String,

    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request headers
    pub headers: HashMap<String, String>,

    /// Request body (for POST requests)
    pub body: Option<String>,

    /// Pagination configuration
    pub pagination: Option<PaginationConfig>,

    /// Rate limiting (requests per second)
    pub rate_limit: Option<f64>,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Request timeout in seconds
    pub timeout_sec: u64,
}

impl Default for ApiCollectorConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            pagination: None,
            rate_limit: None,
            max_retries: 3,
            timeout_sec: 30,
        }
    }
}

/// Pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Type of pagination
    pub pagination_type: PaginationType,

    /// Page parameter name
    pub page_param: String,

    /// Page size parameter name
    pub page_size_param: String,

    /// Page size
    pub page_size: u32,

    /// Maximum pages to fetch
    pub max_pages: u32,

    /// Total items limit
    pub max_items: u32,
}

/// Pagination types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaginationType {
    /// Page-based pagination (e.g., ?page=1&page_size=100)
    PageBased,

    /// Offset-based pagination (e.g., ?offset=0&limit=100)
    OffsetBased,

    /// Cursor-based pagination
    CursorBased { cursor_param: String },
}

/// API data collector
#[derive(Debug)]
pub struct ApiCollector {
    config: ApiCollectorConfig,
    client: reqwest::Client,
    stats: Arc<RwLock<CollectionStats>>,
}

impl ApiCollector {
    /// Create a new API collector
    pub fn new(config: ApiCollectorConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout_sec);

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|e| {
                warn!("Failed to create HTTP client with custom timeout: {}", e);
                reqwest::Client::new()
            });

        Self {
            config,
            client,
            stats: Arc::new(RwLock::new(CollectionStats::default())),
        }
    }

    /// Make an API request with retry logic
    async fn make_request(&self, url: &str) -> Result<String> {
        let mut attempt = 0;
        let mut delay = Duration::from_millis(100);

        loop {
            attempt += 1;

            let mut request = match self.config.method.as_str() {
                "GET" => self.client.get(url),
                "POST" => {
                    let mut req = self.client.post(url);
                    if let Some(body) = &self.config.body {
                        req = req.body(body.clone());
                    }
                    req
                }
                _ => {
                    return Err(Error::InvalidInput(format!(
                        "Unsupported HTTP method: {}",
                        self.config.method
                    )));
                }
            };

            // Add headers
            for (key, value) in &self.config.headers {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    request = request.header(key, header_value);
                }
            }

            // Send request
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => {
                                // Update stats
                                let mut stats = self.stats.write().await;
                                stats.successful += 1;
                                stats.last_collection = Some(chrono::Utc::now());

                                // Rate limiting
                                if let Some(rps) = self.config.rate_limit {
                                    let sleep_duration = Duration::from_secs_f64(1.0 / rps);
                                    tokio::time::sleep(sleep_duration).await;
                                }

                                return Ok(text);
                            }
                            Err(e) => {
                                warn!("Failed to read response body: {}", e);
                            }
                        }
                    } else {
                        warn!("API returned non-success status: {}", response.status());
                    }
                }
                Err(e) => {
                    warn!("API request failed: {}", e);
                }
            }

            // Retry logic
            if attempt >= self.config.max_retries {
                let mut stats = self.stats.write().await;
                stats.failed += 1;
                stats.last_error = Some(format!("Failed after {} attempts", attempt));

                return Err(Error::AgentExecutionFailed(format!(
                    "API request failed after {} attempts",
                    attempt
                )));
            }

            debug!("Retrying in {:?}", delay);
            tokio::time::sleep(delay).await;
            delay = delay.saturating_mul(2); // Exponential backoff
        }
    }

    /// Extract data from API response
    fn extract_data(&self, response: &str) -> Result<Vec<serde_json::Value>> {
        let json: serde_json::Value =
            serde_json::from_str(response).map_err(|e| Error::SerializationError(e.into()))?;

        // Try to extract array from common response formats
        let data = json["data"]
            .as_array()
            .or_else(|| json["results"].as_array())
            .or_else(|| json["items"].as_array())
            .or_else(|| json.as_array())
            .ok_or_else(|| Error::InvalidInput("No data array found in response".to_string()))?;

        Ok(data.to_vec())
    }

    /// Fetch all pages (for paginated APIs)
    async fn fetch_all_pages(&self) -> Result<Vec<String>> {
        let mut all_responses = Vec::new();
        let mut page = 1u32;
        let mut has_more = true;

        while has_more {
            let url = if let Some(pagination) = &self.config.pagination {
                match pagination.pagination_type {
                    PaginationType::PageBased => {
                        format!(
                            "{}?{}={}&{}={}",
                            self.config.endpoint,
                            pagination.page_param,
                            page,
                            pagination.page_size_param,
                            pagination.page_size
                        )
                    }
                    PaginationType::OffsetBased => {
                        let offset = (page - 1) * pagination.page_size;
                        format!(
                            "{}?{}={}&{}={}",
                            self.config.endpoint, "offset", offset, "limit", pagination.page_size
                        )
                    }
                    PaginationType::CursorBased { .. } => {
                        // TODO: Implement cursor-based pagination
                        self.config.endpoint.clone()
                    }
                }
            } else {
                self.config.endpoint.clone()
            };

            debug!("Fetching page {} from {}", page, url);
            let response = self.make_request(&url).await?;
            all_responses.push(response);

            // Check if we should continue
            if let Some(pagination) = &self.config.pagination {
                has_more = page < pagination.max_pages;
                page += 1;

                // Also check if we have enough items
                // This is a simplified check - real implementation would parse response
            } else {
                has_more = false;
            }
        }

        Ok(all_responses)
    }
}

#[async_trait::async_trait]
impl Collector for ApiCollector {
    async fn collect(&self) -> Result<Vec<RawData>> {
        info!("Starting API collection from {}", self.config.endpoint);

        let responses = if self.config.pagination.is_some() {
            self.fetch_all_pages().await?
        } else {
            vec![self.make_request(&self.config.endpoint).await?]
        };

        let mut raw_data_items = Vec::new();

        for response in responses {
            let items = self.extract_data(&response)?;

            for item in items {
                let content = serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string());

                let raw = RawData::new(
                    DataSource::API {
                        endpoint: self.config.endpoint.clone(),
                    },
                    content,
                    "application/json".to_string(),
                );

                raw_data_items.push(raw);
            }

            let mut stats = self.stats.write().await;
            stats.total_collected += raw_data_items.len() as u64;
        }

        info!("Collected {} items from API", raw_data_items.len());

        Ok(raw_data_items)
    }

    fn name(&self) -> &str {
        "api_collector"
    }

    fn schedule(&self) -> Option<crate::collectors::CronSchedule> {
        Some(crate::collectors::CronSchedule::hourly())
    }

    async fn stats(&self) -> crate::collectors::CollectionStats {
        self.stats.read().await.clone()
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple health check - try to make a request
        match self.make_request(&self.config.endpoint).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_collector_creation() {
        let config = ApiCollectorConfig {
            endpoint: "https://api.example.com/data".to_string(),
            method: "GET".to_string(),
            ..Default::default()
        };

        let collector = ApiCollector::new(config);
        assert_eq!(collector.name(), "api_collector");
    }

    #[test]
    fn test_extract_data() {
        let config = ApiCollectorConfig::default();
        let collector = ApiCollector::new(config);

        let response = r#"{"data": [{"id": 1, "name": "test1"}, {"id": 2, "name": "test2"}]}"#;

        let items = collector.extract_data(response).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["id"], 1);
    }
}
