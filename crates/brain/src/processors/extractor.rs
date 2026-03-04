//! Extractor Agent - Entity and relationship extraction
//!
//! Performs:
//! - Named Entity Recognition (NER)
//! - Relationship extraction
//! - Key phrase extraction

use crate::storage::{Entity, ProcessedData, RawData};
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use std::collections::HashMap;
use tracing::debug;

/// Extractor agent configuration
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// Enable entity extraction
    pub enable_entities: bool,

    /// Enable relationship extraction
    pub enable_relationships: bool,

    /// Minimum entity confidence
    pub min_confidence: f64,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            enable_entities: true,
            enable_relationships: true,
            min_confidence: 0.6,
        }
    }
}

/// Extractor agent for NER and relationship extraction
#[derive(Debug, Clone)]
pub struct ExtractorAgent {
    config: ExtractorConfig,
}

impl ExtractorAgent {
    /// Create a new extractor agent
    pub fn new(config: ExtractorConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(ExtractorConfig::default())
    }

    /// Extract entities from text (simplified rule-based)
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // Simple pattern matching for common entity types
        // In production, this would use an NLP library or LLM

        // Extract URLs
        let url_re = regex::Regex::new(r"https?://[^\s]+").unwrap();
        for mat in url_re.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: "URL".to_string(),
                confidence: 0.95,
                start: mat.start(),
                end: mat.end(),
            });
        }

        // Extract email addresses
        let email_re =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        for mat in email_re.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: "EMAIL".to_string(),
                confidence: 0.95,
                start: mat.start(),
                end: mat.end(),
            });
        }

        // Extract dates (simple format)
        let date_re = regex::Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap();
        for mat in date_re.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: "DATE".to_string(),
                confidence: 0.9,
                start: mat.start(),
                end: mat.end(),
            });
        }

        entities
    }

    /// Extract relationships between entities
    pub fn extract_relationships(
        &self,
        _text: &str,
        entities: &[Entity],
    ) -> Vec<(String, String, String)> {
        // Simplified relationship extraction
        // In production, this would use dependency parsing or LLM
        let mut relationships = Vec::new();

        // Example: Find entities that appear close to each other
        for (i, entity1) in entities.iter().enumerate() {
            for entity2 in entities.iter().skip(i + 1) {
                let distance = (entity1.end as i32 - entity2.start as i32).abs();
                if distance < 50 {
                    // Assume "mentions" relationship for nearby entities
                    relationships.push((
                        entity1.text.clone(),
                        "mentions".to_string(),
                        entity2.text.clone(),
                    ));
                }
            }
        }

        relationships
    }
}

impl Default for ExtractorAgent {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[async_trait::async_trait]
impl Agent for ExtractorAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        let text = input.data["text"]
            .as_str()
            .or_else(|| input.data["content"].as_str())
            .unwrap_or("");

        debug!("Extracting entities from text of length {}", text.len());

        let entities = if self.config.enable_entities {
            self.extract_entities(text)
        } else {
            vec![]
        };

        let relationships = if self.config.enable_relationships {
            self.extract_relationships(text, &entities)
        } else {
            vec![]
        };

        Ok(AgentOutput {
            data: serde_json::json!({
                "entities": entities,
                "relationships": relationships,
            }),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("entity_count".to_string(), entities.len().to_string());
                meta.insert(
                    "relationship_count".to_string(),
                    relationships.len().to_string(),
                );
                meta
            },
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        "extractor"
    }

    fn capabilities(&self) -> AgentCapabilities {
        vec![
            "entity_extraction".to_string(),
            "relationship_extraction".to_string(),
            "ner".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_entities() {
        let extractor = ExtractorAgent::with_default_config();
        let text = "Visit https://example.com or email test@example.com on 2024-01-15";
        let entities = extractor.extract_entities(text);
        assert_eq!(entities.len(), 3);
    }

    #[tokio::test]
    async fn test_agent_invoke() {
        let extractor = ExtractorAgent::with_default_config();
        let input = AgentInput::new(serde_json::json!({
            "text": "Email user@example.com for more info"
        }));

        let output = extractor.invoke(input).await.unwrap();
        assert!(output.data["entities"].is_array());
        assert!(output.data["entities"].as_array().unwrap().len() > 0);
    }
}
