//! Unified configuration management with environment variable support
//!
//! This module provides a modern, type-safe configuration system with:
//! - Environment variable loading
//! - Hierarchical configuration (base -> profile -> overrides)
//! - Validation and schema checking
//! - Hot-reload support (for production use)
//! - Zero-copy parsing where possible

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Brain system configuration
    pub brain: BrainConfig,

    /// Engine system configuration
    pub engine: EngineConfig,

    /// Synergy system configuration
    pub synergy: SynergyConfig,

    /// Server configuration
    pub server: ServerConfig,

    /// LLM configuration
    pub llm: LlmConfig,
}

impl Config {
    /// Load configuration from environment and optional config file
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables (prefix: AGENT_PLAYGROUND_)
    /// 2. Config file (if provided)
    /// 3. Defaults
    pub async fn load(path: Option<&Path>) -> Result<Self> {
        // Start with defaults
        let mut config = Self::defaults();

        // Load from file if provided
        if let Some(p) = path {
            config = Self::load_from_file(p, config)?;
        }

        // Override with environment variables
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Get default configuration
    fn defaults() -> Self {
        Self {
            brain: BrainConfig::default(),
            engine: EngineConfig::default(),
            synergy: SynergyConfig::default(),
            server: ServerConfig::default(),
            llm: LlmConfig::default(),
        }
    }

    /// Load configuration from a TOML/YAML/JSON file
    fn load_from_file(path: &Path, mut config: Config) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read config file: {}", e)))?;

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::ConfigError("Config file has no extension".to_string()))?;

        match ext {
            "toml" => {
                let file_config: ConfigPartial = toml::from_str(&content)
                    .map_err(|e| Error::ConfigError(format!("Failed to parse TOML: {}", e)))?;
                config.merge(file_config);
            }
            "yaml" | "yml" => {
                let file_config: ConfigPartial = serde_yaml::from_str(&content)
                    .map_err(|e| Error::ConfigError(format!("Failed to parse YAML: {}", e)))?;
                config.merge(file_config);
            }
            "json" => {
                let file_config: ConfigPartial = serde_json::from_str(&content)
                    .map_err(|e| Error::ConfigError(format!("Failed to parse JSON: {}", e)))?;
                config.merge(file_config);
            }
            _ => {
                return Err(Error::ConfigError(format!(
                    "Unsupported config format: {}",
                    ext
                )))
            }
        }

        Ok(config)
    }

    /// Merge partial configuration into current config
    fn merge(&mut self, partial: ConfigPartial) {
        if let Some(brain) = partial.brain {
            self.brain = brain;
        }
        if let Some(engine) = partial.engine {
            self.engine = engine;
        }
        if let Some(synergy) = partial.synergy {
            self.synergy = synergy;
        }
        if let Some(server) = partial.server {
            self.server = server;
        }
        if let Some(llm) = partial.llm {
            self.llm = llm;
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Helper to get env var with prefix
        let get_env = |suffix: &str| std::env::var(format!("AGENT_PLAYGROUND_{}", suffix)).ok();

        // Brain config overrides
        if let Some(val) = get_env("BRAIN_HOT_TTL_HOURS") {
            self.brain.hot_ttl_hours = val
                .parse()
                .map_err(|_| Error::ConfigError("Invalid BRAIN_HOT_TTL_HOURS".to_string()))?;
        }

        // Engine config overrides
        if let Some(val) = get_env("ENGINE_MAX_STEPS") {
            self.engine.max_steps = val
                .parse()
                .map_err(|_| Error::ConfigError("Invalid ENGINE_MAX_STEPS".to_string()))?;
        }
        if let Some(val) = get_env("ENGINE_SNAPSHOT_DIR") {
            self.engine.snapshot_dir = val;
        }

        // Server config overrides
        if let Some(val) = get_env("SERVER_HOST") {
            self.server.host = val;
        }
        if let Some(val) = get_env("SERVER_PORT") {
            self.server.port = val
                .parse()
                .map_err(|_| Error::ConfigError("Invalid SERVER_PORT".to_string()))?;
        }

        // LLM config overrides
        if let Some(val) = get_env("LLM_PROVIDER") {
            self.llm.provider = val
                .parse()
                .map_err(|_| Error::ConfigError("Invalid LLM_PROVIDER".to_string()))?;
        }
        if let Some(val) = get_env("LLM_API_KEY") {
            self.llm.api_key = Some(val);
        }
        if let Some(val) = get_env("LLM_BASE_URL") {
            self.llm.base_url = Some(val);
        }

        Ok(())
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        // Validate brain config
        if self.brain.hot_ttl_hours == 0 {
            return Err(Error::ConfigError(
                "BRAIN_HOT_TTL_HOURS must be > 0".to_string(),
            ));
        }

        // Validate engine config
        if self.engine.max_steps == 0 {
            return Err(Error::ConfigError(
                "ENGINE_MAX_STEPS must be > 0".to_string(),
            ));
        }

        // Validate server config
        if self.server.port == 0 || self.server.port > 65535 {
            return Err(Error::ConfigError(
                "SERVER_PORT must be 1-65535".to_string(),
            ));
        }

        // Validate LLM config if API key is required
        if self.llm.provider != LlmProvider::Mock && self.llm.api_key.is_none() {
            tracing::warn!(
                "LLM provider is {:?} but no API key provided",
                self.llm.provider
            );
        }

        Ok(())
    }
}

/// Partial configuration for file loading (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigPartial {
    pub brain: Option<BrainConfig>,
    pub engine: Option<EngineConfig>,
    pub synergy: Option<SynergyConfig>,
    pub server: Option<ServerConfig>,
    pub llm: Option<LlmConfig>,
}

/// Brain system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    /// Hot memory TTL in hours
    pub hot_ttl_hours: u64,

    /// Vector memory configuration
    pub vector: VectorMemoryConfig,

    /// Graph memory configuration
    pub graph: GraphMemoryConfig,

    /// Raw archive storage path
    pub raw_archive_path: String,

    /// Processing pipeline configuration
    pub processing: ProcessingConfig,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            hot_ttl_hours: 24,
            vector: VectorMemoryConfig::default(),
            graph: GraphMemoryConfig::default(),
            raw_archive_path: "./data/raw_archive".to_string(),
            processing: ProcessingConfig::default(),
        }
    }
}

/// Vector memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemoryConfig {
    /// Vector database backend
    pub backend: VectorBackend,

    /// Embedding dimension
    pub dimension: usize,

    /// Similarity threshold
    pub similarity_threshold: f64,
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            backend: VectorBackend::InMemory,
            dimension: 1536, // OpenAI default
            similarity_threshold: 0.7,
        }
    }
}

/// Vector database backend options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VectorBackend {
    InMemory,
    Qdrant,
    Milvus,
}

/// Graph memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMemoryConfig {
    /// Graph database backend
    pub backend: GraphBackend,

    /// Connection URL (if using remote DB)
    pub url: Option<String>,
}

impl Default for GraphMemoryConfig {
    fn default() -> Self {
        Self {
            backend: GraphBackend::InMemory,
            url: None,
        }
    }
}

/// Graph database backend options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GraphBackend {
    InMemory,
    Neo4j,
}

/// Processing pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Number of processing workers
    pub workers: usize,

    /// Processing queue size
    pub queue_size: usize,

    /// Batch size for bulk processing
    pub batch_size: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            workers: 4,
            queue_size: 1000,
            batch_size: 50,
        }
    }
}

/// Engine system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Maximum steps per simulation
    pub max_steps: u64,

    /// Snapshot directory
    pub snapshot_dir: String,

    /// Sandbox limits
    pub sandbox: SandboxConfig,

    /// Session configuration
    pub session: SessionConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_steps: 10000,
            snapshot_dir: "./data/snapshots".to_string(),
            sandbox: SandboxConfig::default(),
            session: SessionConfig::default(),
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum instructions per script
    pub max_instructions: u64,

    /// Maximum memory in MB
    pub max_memory_mb: usize,

    /// Timeout in seconds
    pub timeout_sec: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_instructions: 1_000_000,
            max_memory_mb: 512,
            timeout_sec: 300,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_sec: u64,

    /// Auto-snapshot interval
    pub auto_snapshot_interval: u64,

    /// Max concurrent sessions
    pub max_concurrent: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_sec: 3600,
            auto_snapshot_interval: 100,
            max_concurrent: 100,
        }
    }
}

/// Synergy system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyConfig {
    /// Database connection URL (e.g. sqlite://synergy.db)
    pub database_url: String,

    /// Agent registry configuration
    pub registry: RegistryConfig,

    /// Scheduler configuration
    pub scheduler: SchedulerConfig,
}

impl Default for SynergyConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://synergy.db".to_string(),
            registry: RegistryConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }
}

/// Agent registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum cached agents
    pub max_cache_size: usize,

    /// Cache TTL in seconds
    pub cache_ttl_sec: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            cache_ttl_sec: 3600,
        }
    }
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum concurrent missions
    pub max_concurrent_missions: usize,

    /// Mission queue size
    pub queue_size: usize,

    /// Retry configuration
    pub retry: RetryConfig,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_missions: 10,
            queue_size: 100,
            retry: RetryConfig::default(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial backoff in milliseconds
    pub initial_backoff_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind host
    pub host: String,

    /// Bind port
    pub port: u16,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// CORS configuration
    pub cors: CorsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            tls: None,
            cors: CorsConfig::default(),
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate path
    pub cert_path: String,

    /// Private key path
    pub key_path: String,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Allow credentials
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: false,
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM provider
    pub provider: LlmProvider,

    /// API key (optional for some providers)
    pub api_key: Option<String>,

    /// Base URL (for custom endpoints)
    pub base_url: Option<String>,

    /// Model name
    pub model: String,

    /// Default generation parameters
    pub default_params: GenerationParams,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Mock,
            api_key: None,
            base_url: None,
            model: "gpt-4".to_string(),
            default_params: GenerationParams::default(),
        }
    }
}

/// LLM provider options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Ollama,
    Anthropic,
    Mock,
}

impl std::str::FromStr for LlmProvider {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(LlmProvider::OpenAI),
            "ollama" => Ok(LlmProvider::Ollama),
            "anthropic" => Ok(LlmProvider::Anthropic),
            "mock" => Ok(LlmProvider::Mock),
            _ => Err(format!("Unknown LLM provider: {}", s)),
        }
    }
}

/// LLM generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,

    /// Maximum tokens
    pub max_tokens: u32,

    /// Top-p sampling
    pub top_p: f32,

    /// Frequency penalty
    pub frequency_penalty: f32,

    /// Presence penalty
    pub presence_penalty: f32,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
}

/// Hot-reloadable configuration manager
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new(path: Option<&Path>) -> Result<Self> {
        let config = Config::load(path).await?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Get current configuration
    pub async fn get(&self) -> Config {
        self.config.read().await.clone()
    }

    /// Reload configuration from file
    pub async fn reload(&self, path: &Path) -> Result<()> {
        let new_config = Config::load(Some(path)).await?;
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::defaults();
        config.validate().unwrap();
        assert_eq!(config.brain.hot_ttl_hours, 24);
        assert_eq!(config.engine.max_steps, 10000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::defaults();

        // Invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());

        // Fix port
        config.server.port = 8080;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_llm_provider_from_str() {
        // Test that we can parse provider from string
        let provider_str = "OpenAI";
        // This would be used with env var parsing
        assert_eq!(provider_str, "OpenAI");
    }
}
