//! Oracle Protocol - Agent间协作协议
//!
//! 允许Local Agent在遇到困难时请求Universal Agent的帮助

use crate::agent::AgentContext;
use common::{Agent, AgentInput, AgentOutput, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Oracle请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRequest {
    /// 请求ID
    pub id: uuid::Uuid,

    /// 请求者Agent ID
    pub requester_id: uuid::Uuid,

    /// 请求者名称
    pub requester_name: String,

    /// 查询/问题
    pub query: String,

    /// 上下文信息
    pub context: serde_json::Value,

    /// 优先级
    pub priority: RequestPriority,

    /// 请求时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 请求优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Urgent = 3,
}

/// Oracle响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// 对应的请求ID
    pub request_id: uuid::Uuid,

    /// 响应内容
    pub answer: serde_json::Value,

    /// 置信度
    pub confidence: f64,

    /// 响应时间
    pub response_time_ms: u64,

    /// 是否成功
    pub success: bool,

    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Oracle Protocol - 管理Agent间的请求/响应
#[derive(Debug)]
pub struct OracleProtocol {
    /// 待处理的请求
    pending_requests: Arc<RwLock<Vec<OracleRequest>>>,

    /// 请求历史
    request_history: Arc<RwLock<Vec<OracleRequest>>>,
}

impl OracleProtocol {
    /// 创建新的Oracle Protocol
    pub fn new() -> Self {
        info!("Initializing Oracle Protocol");

        Self {
            pending_requests: Arc::new(RwLock::new(Vec::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 创建Oracle请求
    pub async fn create_request(
        &self,
        requester_id: uuid::Uuid,
        requester_name: String,
        query: String,
        context: serde_json::Value,
        priority: RequestPriority,
    ) -> OracleRequest {
        let request = OracleRequest {
            id: uuid::Uuid::new_v4(),
            requester_id,
            requester_name,
            query,
            context,
            priority,
            created_at: chrono::Utc::now(),
        };

        debug!("Created oracle request: {}", request.id);

        // 保存到历史
        let mut history = self.request_history.write().await;
        history.push(request.clone());

        request
    }

    /// 处理Oracle请求（简化版，直接返回模拟响应）
    pub async fn process_request(&self, request: &OracleRequest) -> Result<OracleResponse> {
        info!("Processing oracle request: {}", request.id);

        let start = std::time::Instant::now();

        // 简化实现：返回模拟响应
        // 在实际使用中，这应该查询注册的Universal Agents
        let answer = serde_json::json!({
            "response": format!("Answer to: {}", request.query),
            "confidence": 0.8
        });

        Ok(OracleResponse {
            request_id: request.id,
            answer,
            confidence: 0.8,
            response_time_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    /// 批量处理请求
    pub async fn process_requests_batch(&self, requests: &[OracleRequest]) -> Vec<OracleResponse> {
        let mut responses = Vec::new();

        for request in requests {
            match self.process_request(request).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    warn!("Failed to process request {}: {}", request.id, e);

                    responses.push(OracleResponse {
                        request_id: request.id,
                        answer: serde_json::json!({}),
                        confidence: 0.0,
                        response_time_ms: 0,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        responses
    }

    /// 获取请求历史
    pub async fn get_history(&self) -> Vec<OracleRequest> {
        self.request_history.read().await.clone()
    }

    /// 清理旧请求
    pub async fn cleanup_old_requests(&self, older_than: chrono::Duration) {
        let now = chrono::Utc::now();
        let mut history = self.request_history.write().await;

        history.retain(|req| now.signed_duration_since(req.created_at) < older_than);

        info!("Cleaned up old oracle requests");
    }
}

impl Default for OracleProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oracle_protocol_creation() {
        let protocol = OracleProtocol::new();

        // 创建请求
        let request = protocol
            .create_request(
                uuid::Uuid::new_v4(),
                "test_agent".to_string(),
                "What is this text about?".to_string(),
                serde_json::json!({"text": "Test text"}),
                RequestPriority::Medium,
            )
            .await;

        assert_eq!(request.requester_name, "test_agent");
        assert_eq!(request.priority, RequestPriority::Medium);
    }

    #[tokio::test]
    async fn test_process_request() {
        let protocol = OracleProtocol::new();

        // 创建并处理请求
        let request = protocol
            .create_request(
                uuid::Uuid::new_v4(),
                "test_agent".to_string(),
                "Process this text".to_string(),
                serde_json::json!({"text": "<p>Test</p>"}),
                RequestPriority::Medium,
            )
            .await;

        let response = protocol.process_request(&request).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.success);
        assert_eq!(response.request_id, request.id);
    }

    #[tokio::test]
    async fn test_request_history() {
        let protocol = OracleProtocol::new();

        // 创建多个请求
        for i in 0..5 {
            protocol
                .create_request(
                    uuid::Uuid::new_v4(),
                    format!("agent_{}", i),
                    format!("Query {}", i),
                    serde_json::json!({}),
                    RequestPriority::Medium,
                )
                .await;
        }

        let history = protocol.get_history().await;
        assert_eq!(history.len(), 5);
    }
}
