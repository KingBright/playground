//! Data processors - Universal Agents for processing collected data
//!
//! This module contains processing agents:
//! - Cleaner: Text cleaning and normalization
//! - Extractor: Entity and relationship extraction
//! - Summarizer: Text summarization
//! - Tagger: Classification and tagging
//! - Pipeline: Chained processing workflow

pub mod cleaner;
pub mod extractor;
pub mod pipeline;
pub mod summarizer;
pub mod tagger;

use crate::storage::ProcessedData;
use common::Result;

/// Processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Processed data items
    pub items: Vec<ProcessedData>,

    /// Number of items processed
    pub count: usize,

    /// Processing errors
    pub errors: Vec<String>,
}

/// Processing pipeline step
#[derive(Debug, Clone)]
pub enum PipelineStep {
    Cleaner,
    Tagger,
    Extractor,
    Summarizer,
}

// Re-export processors
pub use cleaner::CleanerAgent;
pub use extractor::ExtractorAgent;
pub use pipeline::ProcessingPipeline;
pub use summarizer::SummarizerAgent;
pub use tagger::TaggerAgent;
