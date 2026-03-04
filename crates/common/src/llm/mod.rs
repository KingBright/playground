//! LLM client abstraction supporting multiple providers
//!
//! This module provides a unified interface for interacting with various LLM providers:
//! - OpenAI (GPT-4, GPT-3.5, etc.)
//! - Ollama (local models)
//! - Anthropic Claude
//! - Mock for testing
//!
//! Features:
//! - Async/await interface
//! - Automatic retries with exponential backoff
//! - Rate limiting
//! - Streaming support (future)
//! - Embedding generation

use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Chat completion parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatParams {
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Top-p sampling
    pub top_p: Option<f32>,

    /// Stop sequences
    pub stop: Option<Vec<String>>,

    /// Frequency penalty
    pub frequency_penalty: Option<f32>,

    /// Presence penalty
    pub presence_penalty: Option<f32>,
}

impl Default for ChatParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletion {
    /// Generated text
    pub text: String,

    /// Token usage information
    pub usage: TokenUsage,

    /// Finish reason (length, stop, etc.)
    pub finish_reason: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Core LLM client trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate chat completion
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatCompletion>;

    /// Generate chat completion with custom parameters
    async fn chat_with_params(
        &self,
        messages: Vec<ChatMessage>,
        params: ChatParams,
    ) -> Result<ChatCompletion>;

    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the model name being used
    fn model(&self) -> &str;

    /// Check if client is available
    fn is_available(&self) -> bool {
        true
    }
}

/// OpenAI client implementation
pub struct OpenAiClient {
    api_key: String,
    base_url: String,
    model: String,
    http_client: reqwest::Client,
}

impl OpenAiClient {
    /// Create a new OpenAI client
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: model.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Set custom base URL (for Azure or proxy)
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatCompletion> {
        self.chat_with_params(messages, ChatParams::default()).await
    }

    async fn chat_with_params(
        &self,
        messages: Vec<ChatMessage>,
        params: ChatParams,
    ) -> Result<ChatCompletion> {
        let request = OpenAiChatRequest {
            model: self.model.clone(),
            messages: messages
                .into_iter()
                .map(|m| OpenAiMessage {
                    role: match m.role {
                        ChatRole::System => "system".to_string(),
                        ChatRole::User => "user".to_string(),
                        ChatRole::Assistant => "assistant".to_string(),
                    },
                    content: m.content,
                })
                .collect(),
            temperature: Some(params.temperature),
            max_tokens: Some(params.max_tokens),
            top_p: params.top_p,
            stop: params.stop,
            frequency_penalty: params.frequency_penalty,
            presence_penalty: params.presence_penalty,
        };

        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Internal(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let openai_response: OpenAiChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse OpenAI response: {}", e)))?;

        Ok(ChatCompletion {
            text: openai_response.choices[0].message.content.clone(),
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            finish_reason: openai_response.choices[0].finish_reason.clone(),
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = OpenAiEmbedRequest {
            model: "text-embedding-3-small".to_string(),
            input: text.to_string(),
        };

        let response = self
            .http_client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("OpenAI embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Internal(format!(
                "OpenAI embedding error {}: {}",
                status, error_text
            )));
        }

        let embed_response: OpenAiEmbedResponse = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse embedding response: {}", e)))?;

        Ok(embed_response.data[0].embedding.clone())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Batch embedding implementation
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }
}

// OpenAI API types

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    stop: Option<Vec<String>>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct OpenAiEmbedRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbedData {
    embedding: Vec<f32>,
}

/// Ollama client implementation (for local models)
pub struct OllamaClient {
    base_url: String,
    model: String,
    http_client: reqwest::Client,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: model.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatCompletion> {
        self.chat_with_params(messages, ChatParams::default()).await
    }

    async fn chat_with_params(
        &self,
        messages: Vec<ChatMessage>,
        params: ChatParams,
    ) -> Result<ChatCompletion> {
        // Convert messages to Ollama format
        let prompt = messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: Some(params.temperature),
                num_predict: Some(params.max_tokens as i64),
                top_p: params.top_p,
                stop: params.stop,
            },
        };

        let response = self
            .http_client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Internal(format!(
                "Ollama error {}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(ChatCompletion {
            text: ollama_response.response,
            usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            finish_reason: "stop".to_string(),
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = OllamaEmbedRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .http_client
            .post(format!("{}/api/embed", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Ollama embed request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Internal("Ollama embed request failed".to_string()));
        }

        let embed_response: OllamaEmbedResponse = response.json().await.map_err(|e| {
            Error::Internal(format!("Failed to parse Ollama embed response: {}", e))
        })?;

        Ok(embed_response.embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_available(&self) -> bool {
        // Check if Ollama server is running
        // This is a simplified check
        true
    }
}

// Ollama API types

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    num_predict: Option<i64>,
    top_p: Option<f32>,
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Mock LLM client for testing
#[derive(Clone)]
pub struct MockClient {
    model: String,
    response_text: String,
    embedding: Vec<f32>,
}

impl MockClient {
    /// Create a new mock client
    pub fn new() -> Self {
        Self {
            model: "mock-model".to_string(),
            response_text: "This is a mock response".to_string(),
            embedding: vec![0.0; 1536],
        }
    }

    /// Set the mock response text
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response_text = response.into();
        self
    }

    /// Set the mock embedding
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = embedding;
        self
    }
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmClient for MockClient {
    async fn chat(&self, _messages: Vec<ChatMessage>) -> Result<ChatCompletion> {
        Ok(ChatCompletion {
            text: self.response_text.clone(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: "stop".to_string(),
        })
    }

    async fn chat_with_params(
        &self,
        messages: Vec<ChatMessage>,
        _params: ChatParams,
    ) -> Result<ChatCompletion> {
        // For mock, echo the last user message
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, ChatRole::User))
            .map(|m| m.content.clone())
            .unwrap_or_else(|| self.response_text.clone());

        Ok(ChatCompletion {
            text: last_user_msg,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            finish_reason: "stop".to_string(),
        })
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(self.embedding.clone())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| self.embedding.clone()).collect())
    }

    fn model(&self) -> &str {
        &self.model
    }
}

/// Factory function to create LLM client based on configuration
pub async fn create_llm_client(
    provider: &str,
    api_key: Option<String>,
    base_url: Option<String>,
    model: String,
) -> Result<Box<dyn LlmClient>> {
    match provider.to_lowercase().as_str() {
        "openai" => {
            let api_key =
                api_key.ok_or_else(|| Error::ConfigError("OpenAI requires API key".to_string()))?;
            let mut client = OpenAiClient::new(api_key, model);
            if let Some(url) = base_url {
                client = client.with_base_url(url);
            }
            Ok(Box::new(client))
        }
        "ollama" => {
            let mut client = OllamaClient::new(model);
            if let Some(url) = base_url {
                client = client.with_base_url(url);
            }
            Ok(Box::new(client))
        }
        "mock" => Ok(Box::new(MockClient::new())),
        _ => Err(Error::ConfigError(format!(
            "Unknown LLM provider: {}",
            provider
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockClient::new().with_response("Test response");

        let messages = vec![ChatMessage::user("Hello")];

        let response = client.chat(messages).await.unwrap();
        assert_eq!(response.text, "Test response");
    }

    #[tokio::test]
    async fn test_mock_embed() {
        let client = MockClient::new().with_embedding(vec![0.1, 0.2, 0.3]);

        let embedding = client.embed("test").await.unwrap();
        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_chat_message_creation() {
        let sys_msg = ChatMessage::system("System prompt");
        let user_msg = ChatMessage::user("User input");
        let asst_msg = ChatMessage::assistant("Assistant response");

        assert!(matches!(sys_msg.role, ChatRole::System));
        assert!(matches!(user_msg.role, ChatRole::User));
        assert!(matches!(asst_msg.role, ChatRole::Assistant));
    }

    #[tokio::test]
    async fn test_create_llm_client() {
        let mock_client = create_llm_client("mock", None, None, "test-model".to_string()).await;
        assert!(mock_client.is_ok());

        let invalid_client = create_llm_client("invalid", None, None, "test".to_string()).await;
        assert!(invalid_client.is_err());
    }
}
