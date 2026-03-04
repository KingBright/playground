//! Cleaner Agent - Text cleaning and normalization
//!
//! Performs text cleaning operations:
//! - HTML tag removal
//! - Special character removal
//! - Whitespace normalization
//! - Text deduplication

use crate::storage::RawData;
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use regex::Regex;
use std::sync::Arc;
use tracing::debug;

/// Cleaner agent configuration
#[derive(Debug, Clone)]
pub struct CleanerConfig {
    /// Remove HTML tags
    pub remove_html: bool,

    /// Remove special characters
    pub remove_special_chars: bool,

    /// Normalize whitespace
    pub normalize_whitespace: bool,

    /// Remove duplicate lines
    pub remove_duplicates: bool,
}

impl Default for CleanerConfig {
    fn default() -> Self {
        Self {
            remove_html: true,
            remove_special_chars: true,
            normalize_whitespace: true,
            remove_duplicates: false,
        }
    }
}

/// Text cleaner agent
#[derive(Debug, Clone)]
pub struct CleanerAgent {
    config: CleanerConfig,
}

impl CleanerAgent {
    /// Create a new cleaner agent
    pub fn new(config: CleanerConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(CleanerConfig::default())
    }

    /// Clean HTML content
    fn clean_html(&self, text: &str) -> String {
        // Simple HTML tag removal using regex - replace with empty string
        let re = Regex::new(r"<[^>]+>").unwrap();
        re.replace_all(text, "").to_string()
    }

    /// Remove special characters
    fn remove_special_chars(&self, text: &str) -> String {
        // Keep only alphanumeric, spaces, and basic punctuation
        let re = Regex::new(r"[^\w\s\.\,\!\?\;\:\-\(\)]+").unwrap();
        re.replace_all(text, " ").to_string()
    }

    /// Normalize whitespace
    fn normalize_whitespace(&self, text: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(text.trim(), " ").to_string()
    }

    /// Remove duplicate lines
    fn remove_duplicate_lines(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut seen = std::collections::HashSet::new();
        let mut unique_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if seen.insert(trimmed) {
                unique_lines.push(trimmed);
            }
        }

        unique_lines.join("\n")
    }

    /// Process text through all cleaning steps
    pub fn clean(&self, text: &str) -> String {
        let mut result = text.to_string();

        if self.config.remove_html {
            result = self.clean_html(&result);
        }

        if self.config.remove_special_chars {
            result = self.remove_special_chars(&result);
        }

        if self.config.normalize_whitespace {
            result = self.normalize_whitespace(&result);
        }

        if self.config.remove_duplicates {
            result = self.remove_duplicate_lines(&result);
        }

        result
    }
}

impl Default for CleanerAgent {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[async_trait::async_trait]
impl Agent for CleanerAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        // Extract text from input
        let text = input.data["text"]
            .as_str()
            .or_else(|| input.data["content"].as_str())
            .unwrap_or("");

        debug!("Cleaning text of length {}", text.len());

        // Clean the text
        let cleaned = self.clean(text);

        Ok(AgentOutput {
            data: serde_json::json!({
                "text": cleaned,
                "original_length": text.len(),
                "cleaned_length": cleaned.len(),
            }),
            metadata: std::collections::HashMap::new(),
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        "cleaner"
    }

    fn capabilities(&self) -> AgentCapabilities {
        vec![
            "text_cleaning".to_string(),
            "html_removal".to_string(),
            "text_normalization".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html() {
        let cleaner = CleanerAgent::with_default_config();
        let html = "<p>Hello <b>world</b>!</p>";
        let cleaned = cleaner.clean(html);
        assert_eq!(cleaned, "Hello world!");
    }

    #[test]
    fn test_normalize_whitespace() {
        let cleaner = CleanerAgent::with_default_config();
        let text = "Hello    world\n\n  Test";
        let cleaned = cleaner.normalize_whitespace(text);
        assert_eq!(cleaned, "Hello world Test");
    }

    #[tokio::test]
    async fn test_agent_invoke() {
        let cleaner = CleanerAgent::with_default_config();
        let input = AgentInput::new(serde_json::json!({
            "text": "<p>Hello   world!</p>"
        }));

        let output = cleaner.invoke(input).await.unwrap();
        assert_eq!(output.data["text"], "Hello world!");
    }
}
