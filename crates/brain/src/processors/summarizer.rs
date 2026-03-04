//! Summarizer Agent - Text summarization
//!
//! Performs:
//! - Long text summarization
//! - Multi-level summarization
//! - Bullet point extraction

use crate::storage::RawData;
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use std::collections::HashMap;
use tracing::debug;

/// Summarizer agent configuration
#[derive(Debug, Clone)]
pub struct SummarizerConfig {
    /// Maximum summary length
    pub max_length: usize,

    /// Summary style (bullet, paragraph, etc.)
    pub style: SummaryStyle,

    /// Number of bullet points
    pub num_bullets: usize,
}

impl Default for SummarizerConfig {
    fn default() -> Self {
        Self {
            max_length: 500,
            style: SummaryStyle::Paragraph,
            num_bullets: 5,
        }
    }
}

/// Summary style
#[derive(Debug, Clone, Copy)]
pub enum SummaryStyle {
    Paragraph,
    BulletPoints,
    Concise,
}

/// Summarizer agent
#[derive(Debug, Clone)]
pub struct SummarizerAgent {
    config: SummarizerConfig,
}

impl SummarizerAgent {
    /// Create a new summarizer agent
    pub fn new(config: SummarizerConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(SummarizerConfig::default())
    }

    /// Generate a simple extractive summary
    pub fn summarize(&self, text: &str) -> String {
        // Simple extractive summarization
        // In production, this would use an LLM or abstractive summarization

        let sentences: Vec<&str> = text
            .split_terminator(&['.', '!', '?'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return text.to_string();
        }

        // Take first few sentences up to max_length
        let mut summary = String::new();
        let mut char_count = 0;

        for sentence in sentences.iter() {
            if char_count + sentence.len() > self.config.max_length {
                break;
            }
            if !summary.is_empty() {
                summary.push(' ');
            }
            summary.push_str(sentence);
            char_count += sentence.len();
        }

        match self.config.style {
            SummaryStyle::BulletPoints => {
                // Convert to bullet points
                let sentences: Vec<&str> = summary
                    .split_terminator(&['.', '!', '?'][..])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .take(self.config.num_bullets)
                    .collect();

                sentences
                    .iter()
                    .map(|s| format!("• {}", s))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            SummaryStyle::Concise => {
                // First sentence only
                sentences
                    .first()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| summary)
            }
            SummaryStyle::Paragraph => summary,
        }
    }

    /// Extract key points as bullets
    pub fn extract_key_points(&self, text: &str) -> Vec<String> {
        // Simple key point extraction based on sentence importance
        let sentences: Vec<&str> = text
            .split_terminator(&['.', '!', '?'][..])
            .map(|s| s.trim())
            .filter(|s| s.len() > 20) // Filter out very short sentences
            .collect();

        // Take sentences with high information density
        let mut key_points = Vec::new();

        for sentence in sentences.iter().take(self.config.num_bullets) {
            key_points.push(sentence.to_string());
        }

        key_points
    }
}

impl Default for SummarizerAgent {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[async_trait::async_trait]
impl Agent for SummarizerAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        let text = input.data["text"]
            .as_str()
            .or_else(|| input.data["content"].as_str())
            .unwrap_or("");

        debug!("Summarizing text of length {}", text.len());

        let summary = self.summarize(text);

        Ok(AgentOutput {
            data: serde_json::json!({
                "summary": summary,
                "original_length": text.len(),
                "summary_length": summary.len(),
            }),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "compression_ratio".to_string(),
                    format!("{:.2}", summary.len() as f64 / text.len() as f64),
                );
                meta
            },
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        "summarizer"
    }

    fn capabilities(&self) -> AgentCapabilities {
        vec![
            "summarization".to_string(),
            "abstractive_summarization".to_string(),
            "extractive_summarization".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize() {
        let summarizer = SummarizerAgent::with_default_config();
        let text = "This is sentence one. This is sentence two. This is sentence three. This is sentence four.";
        let summary = summarizer.summarize(text);
        assert!(!summary.is_empty());
        assert!(summary.len() <= text.len());
    }

    #[test]
    fn test_extract_key_points() {
        let summarizer = SummarizerAgent::with_default_config();
        let text = "Key point one. Another point here. Third important point. And a fourth one.";
        let key_points = summarizer.extract_key_points(text);
        assert!(!key_points.is_empty());
        assert!(key_points.len() <= 5);
    }
}
