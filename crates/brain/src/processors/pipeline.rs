//! Processing Pipeline - Chained data processing
//!
//! Orchestrates multiple processors in sequence:
//! - Cleaner -> Tagger -> Extractor
//! - Parallel processing support
//! - Error handling and recovery

use crate::processors::{CleanerAgent, ExtractorAgent, SummarizerAgent, TaggerAgent};
use crate::storage::ProcessedData;
use common::{Agent, AgentInput, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Processing pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable cleaner step
    pub enable_cleaner: bool,

    /// Enable tagger step
    pub enable_tagger: bool,

    /// Enable extractor step
    pub enable_extractor: bool,

    /// Enable summarizer step
    pub enable_summarizer: bool,

    /// Maximum concurrent processing
    pub max_concurrent: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_cleaner: true,
            enable_tagger: true,
            enable_extractor: true,
            enable_summarizer: false,
            max_concurrent: 10,
        }
    }
}

/// Processing pipeline for chaining multiple processors
#[derive(Debug)]
pub struct ProcessingPipeline {
    config: PipelineConfig,
    cleaner: Option<CleanerAgent>,
    tagger: Option<TaggerAgent>,
    extractor: Option<ExtractorAgent>,
    summarizer: Option<SummarizerAgent>,
}

impl ProcessingPipeline {
    /// Create a new processing pipeline
    pub fn new(config: PipelineConfig) -> Self {
        let cleaner = if config.enable_cleaner {
            Some(CleanerAgent::with_default_config())
        } else {
            None
        };

        let tagger = if config.enable_tagger {
            Some(TaggerAgent::with_default_config())
        } else {
            None
        };

        let extractor = if config.enable_extractor {
            Some(ExtractorAgent::with_default_config())
        } else {
            None
        };

        let summarizer = if config.enable_summarizer {
            Some(SummarizerAgent::with_default_config())
        } else {
            None
        };

        Self {
            config,
            cleaner,
            tagger,
            extractor,
            summarizer,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// Process a single text through the pipeline
    pub async fn process(&self, text: &str) -> Result<ProcessedData> {
        debug!("Processing text through pipeline");

        let mut current_text = text.to_string();
        let mut tags = Vec::new();
        let mut entities = Vec::new();
        let mut summary = None;
        let mut embedding = None;

        // Step 1: Cleaner
        if let Some(cleaner) = &self.cleaner {
            let input = AgentInput::new(serde_json::json!({ "text": current_text }));
            let output = cleaner.invoke(input).await?;

            if let Some(cleaned) = output.data["text"].as_str() {
                current_text = cleaned.to_string();
            }

            debug!("Text after cleaning: {} chars", current_text.len());
        }

        // Step 2: Tagger (can run in parallel with Extractor)
        if let Some(tagger) = &self.tagger {
            let input = AgentInput::new(serde_json::json!({ "text": current_text }));
            let output = tagger.invoke(input).await?;

            if let Some(tags_array) = output.data["tags"].as_array() {
                tags = tags_array
                    .iter()
                    .filter_map(|t| t.as_str())
                    .map(|s| s.to_string())
                    .collect();
            }

            if let Some(embed) = output.data["embedding"].as_array() {
                let embed_vec: Vec<f32> = embed
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .map(|v| v as f32)
                    .collect();
                if !embed_vec.is_empty() {
                    embedding = Some(embed_vec);
                }
            }

            debug!("Generated {} tags", tags.len());
        }

        // Step 3: Extractor (can run in parallel with Tagger)
        if let Some(extractor) = &self.extractor {
            let input = AgentInput::new(serde_json::json!({ "text": current_text }));
            let output = extractor.invoke(input).await?;

            if let Some(entities_array) = output.data["entities"].as_array() {
                for entity in entities_array {
                    if let (
                        Some(text),
                        Some(entity_type),
                        Some(confidence),
                        Some(start),
                        Some(end),
                    ) = (
                        entity["text"].as_str(),
                        entity["entity_type"].as_str(),
                        entity["confidence"].as_f64(),
                        entity["start"].as_u64(),
                        entity["end"].as_u64(),
                    ) {
                        entities.push(crate::storage::Entity {
                            text: text.to_string(),
                            entity_type: entity_type.to_string(),
                            confidence,
                            start: start as usize,
                            end: end as usize,
                        });
                    }
                }
            }

            debug!("Extracted {} entities", entities.len());
        }

        // Step 4: Summarizer (runs last, depends on cleaned text)
        if let Some(summarizer) = &self.summarizer {
            let input = AgentInput::new(serde_json::json!({ "text": current_text }));
            let output = summarizer.invoke(input).await?;

            if let Some(summary_text) = output.data["summary"].as_str() {
                summary = Some(summary_text.to_string());
            }

            debug!("Generated summary");
        }

        // Create ProcessedData
        Ok(ProcessedData {
            id: uuid::Uuid::new_v4(),
            raw_data_id: uuid::Uuid::new_v4(), // TODO: Link to actual raw data
            content: current_text,
            entities,
            tags,
            summary,
            embedding,
            processed_at: chrono::Utc::now(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("pipeline_version".to_string(), "1.0".to_string());
                meta
            },
        })
    }

    /// Process multiple texts in batch
    pub async fn process_batch(&self, texts: Vec<String>) -> Result<Vec<ProcessedData>> {
        info!("Processing {} texts through pipeline", texts.len());

        let mut results = Vec::new();

        // Process with controlled concurrency
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent));
        let mut tasks = Vec::new();

        for text in texts {
            let sem = semaphore.clone();
            let pipeline = self.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                pipeline.process(&text).await
            });

            tasks.push(task);
        }

        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    warn!("Pipeline processing failed: {}", e);
                    // Continue processing other items
                }
                Err(e) => {
                    warn!("Task failed: {}", e);
                }
            }
        }

        info!("Pipeline processed {} items successfully", results.len());

        Ok(results)
    }
}

impl Clone for ProcessingPipeline {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cleaner: self.cleaner.clone(),
            tagger: self.tagger.clone(),
            extractor: self.extractor.clone(),
            summarizer: self.summarizer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline() {
        let pipeline = ProcessingPipeline::with_default_config();

        let text =
            "<p>This is a test article about artificial intelligence and machine learning.</p>";

        let result = pipeline.process(text).await.unwrap();

        assert!(!result.content.is_empty());
        assert!(!result.tags.is_empty());
        // Entities may or may not be extracted depending on the text
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let pipeline = ProcessingPipeline::with_default_config();

        let texts = vec![
            "First article about AI.".to_string(),
            "Second article about ML.".to_string(),
            "Third article about Rust.".to_string(),
        ];

        let results = pipeline.process_batch(texts).await.unwrap();

        assert_eq!(results.len(), 3);
    }
}
