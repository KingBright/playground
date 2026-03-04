//! Universal Agent - 可复用的通用Agent
//!
//! 可以挂载到任意Session的Agent服务

use crate::agent::{AgentConfig, AgentContext};
use brain::processors::{
    pipeline::ProcessingPipeline, CleanerAgent, ExtractorAgent, SummarizerAgent, TaggerAgent,
};
use common::{Agent, AgentCapabilities, AgentInput, AgentOutput, Error, Result};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Universal Agent配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniversalAgentConfig {
    /// Agent名称
    pub name: String,

    /// 使用的处理器
    pub processors: Vec<String>,

    /// Agent功能描述
    pub description: Option<String>,
}

/// Universal Agent - 可挂载到任意Session
#[derive(Debug)]
pub struct UniversalAgent {
    config: UniversalAgentConfig,
    pipeline: Arc<ProcessingPipeline>,
    id: uuid::Uuid,
}

impl UniversalAgent {
    /// 创建新的Universal Agent
    pub fn new(config: UniversalAgentConfig) -> Self {
        info!("Creating universal agent: {}", config.name);

        // 创建处理流水线
        let pipeline_config = brain::processors::pipeline::PipelineConfig {
            enable_cleaner: config.processors.contains(&"cleaner".to_string()),
            enable_tagger: config.processors.contains(&"tagger".to_string()),
            enable_extractor: config.processors.contains(&"extractor".to_string()),
            enable_summarizer: config.processors.contains(&"summarizer".to_string()),
            max_concurrent: 10,
        };

        let pipeline = Arc::new(ProcessingPipeline::new(pipeline_config));

        Self {
            id: uuid::Uuid::new_v4(),
            config,
            pipeline,
        }
    }

    /// 处理文本
    pub async fn process_text(&self, text: &str) -> Result<brain::storage::ProcessedData> {
        debug!("Universal agent {} processing text", self.config.name);

        self.pipeline.process(text).await
    }

    /// 获取Agent ID
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }
}

#[async_trait::async_trait]
impl Agent for UniversalAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        debug!("UniversalAgent {} invoked", self.config.name);

        // 获取输入文本
        let text = input
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if text.is_empty() {
            return Ok(AgentOutput {
                data: json!({
                    "error": "No text provided"
                }),
                metadata: std::collections::HashMap::new(),
                need_help: None,
            });
        }

        // 处理文本
        let processed = self.pipeline.process(text).await?;

        // 构建输出
        let output_data = json!({
            "processed_text": processed.content,
            "entities": processed.entities,
            "tags": processed.tags,
            "summary": processed.summary,
            "has_embedding": processed.embedding.is_some(),
        });

        Ok(AgentOutput {
            data: output_data,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("agent_type".to_string(), "universal".to_string());
                meta.insert("agent_id".to_string(), self.id.to_string());
                meta
            },
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn capabilities(&self) -> AgentCapabilities {
        let mut caps = vec!["text_processing".to_string()];

        for processor in &self.config.processors {
            caps.push(format!("processor_{}", processor));
        }

        caps
    }
}

/// 创建文本分析Universal Agent
pub fn create_text_analyzer() -> UniversalAgent {
    UniversalAgentConfig {
        name: "text_analyzer".to_string(),
        description: Some("Analyzes text for entities, tags, and summary".to_string()),
        processors: vec![
            "cleaner".to_string(),
            "tagger".to_string(),
            "extractor".to_string(),
            "summarizer".to_string(),
        ],
    }
    .into()
}

/// 创建知识提取Universal Agent
pub fn create_knowledge_extractor() -> UniversalAgent {
    UniversalAgentConfig {
        name: "knowledge_extractor".to_string(),
        description: Some("Extracts knowledge from text".to_string()),
        processors: vec![
            "cleaner".to_string(),
            "extractor".to_string(),
            "tagger".to_string(),
        ],
    }
    .into()
}

impl From<UniversalAgentConfig> for UniversalAgent {
    fn from(config: UniversalAgentConfig) -> Self {
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_analyzer_creation() {
        let agent = create_text_analyzer();
        assert_eq!(agent.name(), "text_analyzer");
    }

    #[tokio::test]
    async fn test_universal_agent_invoke() {
        let agent = create_text_analyzer();

        let input = AgentInput::new(json!({
            "text": "<p>This is a <strong>test</strong> article.</p>"
        }));

        let result = agent.invoke(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();

        // 应该有处理后的内容
        assert!(output.data["processed_text"].is_string());
    }

    #[tokio::test]
    async fn test_process_text() {
        let agent = create_text_analyzer();

        let text = "<p>AI is transforming technology.</p>";
        let result = agent.process_text(text).await;

        assert!(result.is_ok());
        let processed = result.unwrap();

        // 内容应该被清洗（无HTML标签）
        assert!(!processed.content.contains("<"));
    }
}
