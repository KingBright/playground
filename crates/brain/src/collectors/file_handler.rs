//! File upload handler
//!
//! Handles file uploads with:
//! - File type validation
//! - Size limits
//! - Content extraction
//! - Metadata extraction

use crate::collectors::{CollectionStats, Collector};
use crate::storage::{DataSource, RawData};
use async_trait::async_trait;
use common::{Error, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// File handler configuration
#[derive(Debug, Clone)]
pub struct FileHandlerConfig {
    /// Maximum file size in bytes
    pub max_file_size: usize,

    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,

    /// Enable content extraction
    pub enable_extraction: bool,
}

impl Default for FileHandlerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![
                "txt".to_string(),
                "json".to_string(),
                "html".to_string(),
                "md".to_string(),
                "pdf".to_string(),
            ],
            enable_extraction: true,
        }
    }
}

/// File upload result
#[derive(Debug, Clone)]
pub struct FileUploadResult {
    /// File path
    pub path: String,

    /// File size
    pub size: u64,

    /// Content type
    pub content_type: String,

    /// Extracted content (if available)
    pub content: Option<String>,

    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// File handler for processing uploaded files
#[derive(Debug)]
pub struct FileHandler {
    config: FileHandlerConfig,
    stats: Arc<RwLock<CollectionStats>>,
}

impl FileHandler {
    /// Create a new file handler
    pub fn new(config: FileHandlerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(CollectionStats::default())),
        }
    }

    /// Validate file extension
    fn validate_extension(&self, path: &Path) -> Result<bool> {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            Ok(self.config.allowed_extensions.contains(&ext_str))
        } else {
            Ok(false)
        }
    }

    /// Extract content from file based on type
    async fn extract_content(&self, path: &Path) -> Result<String> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "txt" | "md" | "json" | "html" => {
                // Text files - read directly
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| Error::IoError(e))?;
                Ok(content)
            }
            "pdf" => {
                // TODO: Implement PDF text extraction
                warn!("PDF extraction not yet implemented");
                Ok(String::new())
            }
            _ => {
                warn!("Unsupported file type: {}", ext);
                Ok(String::new())
            }
        }
    }

    /// Process uploaded file
    pub async fn process_file(&self, path: &Path) -> Result<FileUploadResult> {
        debug!("Processing file: {:?}", path);

        // Check file exists
        if !path.exists() {
            return Err(Error::NotFound(format!("File not found: {:?}", path)));
        }

        // Get metadata
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| Error::IoError(e))?;

        let file_size = metadata.len() as u64;

        // Validate file size
        if file_size > self.config.max_file_size as u64 {
            return Err(Error::InvalidInput(format!(
                "File too large: {} bytes (max: {})",
                file_size, self.config.max_file_size
            )));
        }

        // Validate extension
        if !self.validate_extension(path)? {
            return Err(Error::InvalidInput("File type not allowed".to_string()));
        }

        // Extract content
        let content = if self.config.enable_extraction {
            Some(self.extract_content(path).await?)
        } else {
            None
        };

        // Determine content type
        let content_type: String = if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("txt") => "text/plain".to_string(),
                Some("json") => "application/json".to_string(),
                Some("html") => "text/html".to_string(),
                Some("md") => "text/markdown".to_string(),
                Some("pdf") => "application/pdf".to_string(),
                _ => "application/octet-stream".to_string(),
            }
        } else {
            "application/octet-stream".to_string()
        };

        // Build metadata
        let mut file_metadata = std::collections::HashMap::new();
        file_metadata.insert(
            "file_name".to_string(),
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
        );
        file_metadata.insert("file_size".to_string(), file_size.to_string());
        file_metadata.insert("content_type".to_string(), content_type.clone());

        Ok(FileUploadResult {
            path: path.display().to_string(),
            size: file_size,
            content_type,
            content,
            metadata: file_metadata,
        })
    }

    /// Process multiple files
    pub async fn process_files(&self, paths: &[&Path]) -> Result<Vec<FileUploadResult>> {
        let mut results = Vec::new();

        for path in paths {
            match self.process_file(path).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to process file {:?}: {}", path, e);
                    // Continue processing other files
                }
            }
        }

        Ok(results)
    }
}

// Note: FileHandler doesn't implement Collector trait directly,
// as files are typically uploaded manually rather than collected on a schedule.
// It can still be used by a collector that watches a directory.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_handler_creation() {
        let config = FileHandlerConfig::default();
        let handler = FileHandler::new(config);
        // Handler created successfully
    }

    #[test]
    fn test_validate_extension() {
        let config = FileHandlerConfig::default();
        let handler = FileHandler::new(config);

        // Test valid extension
        let path = Path::new("test.txt");
        assert!(handler.validate_extension(path).unwrap());

        // Test invalid extension
        let path = Path::new("test.exe");
        assert!(!handler.validate_extension(path).unwrap());
    }
}
