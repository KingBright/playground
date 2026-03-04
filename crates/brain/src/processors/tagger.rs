//! Tagger Agent - Classification and tagging
//!
//! Performs:
//! - Automatic text classification
//! - Tag generation
//! - Embedding generation (using LLM)

use crate::storage::{Entity, ProcessedData, RawData};
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// Tagger agent configuration
#[derive(Debug, Clone)]
pub struct TaggerConfig {
    /// Maximum number of tags to generate
    pub max_tags: usize,

    /// Minimum tag confidence
    pub min_confidence: f64,

    /// Enable embedding generation
    pub enable_embeddings: bool,
}

impl Default for TaggerConfig {
    fn default() -> Self {
        Self {
            max_tags: 10,
            min_confidence: 0.5,
            enable_embeddings: true,
        }
    }
}

/// Tagger agent for classification and tagging
#[derive(Debug, Clone)]
pub struct TaggerAgent {
    config: TaggerConfig,
    // TODO: Add LLM client when available
}

impl TaggerAgent {
    /// Create a new tagger agent
    pub fn new(config: TaggerConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(TaggerConfig::default())
    }

    /// Generate tags from text
    pub fn generate_tags(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction (production version would use LLM)
        let mut tags = Vec::new();
        let text_lower = text.to_lowercase();

        // Extract common keywords (simplified)
        let keywords = vec![
            ("ai", vec!["ai", "artificial intelligence"]),
            ("artificial", vec!["artificial"]),
            ("intelligence", vec!["intelligence"]),
            ("machine", vec!["machine"]),
            ("learning", vec!["learning"]),
            ("data", vec!["data"]),
            ("science", vec!["science"]),
            ("programming", vec!["programming"]),
            ("rust", vec!["rust"]),
            ("python", vec!["python"]),
            ("javascript", vec!["javascript"]),
            ("web", vec!["web"]),
            ("api", vec!["api"]),
            ("cloud", vec!["cloud"]),
            ("security", vec!["security"]),
            ("database", vec!["database"]),
        ];

        for (tag, patterns) in keywords {
            if patterns.iter().any(|p| text_lower.contains(p)) && !tags.contains(&tag.to_string()) {
                tags.push(tag.to_string());
                if tags.len() >= self.config.max_tags {
                    break;
                }
            }
        }

        tags
    }

    /// Classify text into categories
    pub fn classify(&self, text: &str) -> Vec<String> {
        let mut categories = Vec::new();

        // Simple rule-based classification
        if text.contains("API") || text.contains("endpoint") {
            categories.push("technical".to_string());
        }
        if text.contains("news") || text.contains("article") {
            categories.push("news".to_string());
        }
        if text.contains("tutorial") || text.contains("guide") {
            categories.push("educational".to_string());
        }
        if text.contains("research") || text.contains("study") {
            categories.push("research".to_string());
        }

        categories
    }

    /// Generate embedding (placeholder)
    pub fn generate_embedding(&self, _text: &str) -> Vec<f32> {
        // TODO: Use actual LLM embedding
        // For now, return a dummy embedding
        vec![0.0; 1536] // OpenAI default dimension
    }
}

impl Default for TaggerAgent {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[async_trait::async_trait]
impl Agent for TaggerAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        let text = input.data["text"]
            .as_str()
            .or_else(|| input.data["content"].as_str())
            .unwrap_or("");

        debug!("Tagging text of length {}", text.len());

        // Generate tags and categories
        let tags = self.generate_tags(text);
        let categories = self.classify(text);

        // Combine all tags
        let all_tags: Vec<String> = tags.into_iter().chain(categories.into_iter()).collect();

        // Generate embedding
        let embedding = if self.config.enable_embeddings {
            Some(self.generate_embedding(text))
        } else {
            None
        };

        Ok(AgentOutput {
            data: serde_json::json!({
                "tags": all_tags,
                "embedding": embedding,
            }),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("tag_count".to_string(), all_tags.len().to_string());
                meta
            },
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        "tagger"
    }

    fn capabilities(&self) -> AgentCapabilities {
        vec![
            "tagging".to_string(),
            "classification".to_string(),
            "embedding".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tags() {
        let tagger = TaggerAgent::with_default_config();
        let text = "This article discusses artificial intelligence and machine learning.";
        let tags = tagger.generate_tags(text);
        assert!(tags.contains(&"ai".to_string()));
        assert!(tags.contains(&"machine".to_string()));
        assert!(tags.contains(&"learning".to_string()));
    }

    #[test]
    fn test_classify() {
        let tagger = TaggerAgent::with_default_config();
        let text = "This is a tutorial about the new API";
        let categories = tagger.classify(text);
        assert!(categories.contains(&"technical".to_string()));
        assert!(categories.contains(&"educational".to_string()));
    }
}
