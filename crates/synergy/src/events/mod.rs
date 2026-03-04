//! Event System - 事件订阅与发布
//!
//! 提供轻量级的事件驱动机制

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};

/// 事件类型
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EventType {
    /// 数据更新事件
    DataUpdated(String),
    /// Agent状态变更
    AgentStatusChanged(String),
    /// 会话状态变更
    SessionStatusChanged(String),
    /// 定时触发
    Scheduled,
    /// 自定义事件
    Custom(String),
}

impl EventType {
    pub fn name(&self) -> String {
        match self {
            EventType::DataUpdated(source) => format!("data_updated:{}", source),
            EventType::AgentStatusChanged(agent) => format!("agent_status:{}", agent),
            EventType::SessionStatusChanged(session) => format!("session_status:{}", session),
            EventType::Scheduled => "scheduled".to_string(),
            EventType::Custom(name) => name.clone(),
        }
    }
}

/// 事件数据
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Event {
    pub fn new(event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            event_type,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// 事件处理器类型
type EventHandler = Arc<dyn Fn(Event) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

/// 事件总线
pub struct EventBus {
    /// 订阅者映射: 事件名称 -> 处理器列表
    subscribers: Arc<RwLock<HashMap<String, Vec<EventHandler>>>>,
    /// 事件发送通道
    sender: mpsc::UnboundedSender<Event>,
    /// 事件接收通道
    receiver: Arc<RwLock<mpsc::UnboundedReceiver<Event>>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscribers_count", &0) // 无法直接打印 subscribers
            .field("sender", &self.sender)
            .field("receiver", &"<receiver>")
            .finish()
    }
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            sender,
            receiver: Arc::new(RwLock::new(receiver)),
        }
    }

    /// 订阅事件
    pub async fn subscribe<F, Fut>(&self, event_name: &str, handler: F)
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let wrapped_handler: EventHandler = Arc::new(move |event| {
            Box::pin(handler(event))
        });

        let mut subs = self.subscribers.write().await;
        subs.entry(event_name.to_string())
            .or_insert_with(Vec::new)
            .push(wrapped_handler);

        debug!("Subscribed to event: {}", event_name);
    }

    /// 取消订阅（简化实现：清空该事件的所有处理器）
    pub async fn unsubscribe(&self, event_name: &str) {
        let mut subs = self.subscribers.write().await;
        subs.remove(event_name);
        debug!("Unsubscribed from event: {}", event_name);
    }

    /// 发布事件
    pub fn publish(&self, event: Event) -> Result<(), String> {
        self.sender.send(event)
            .map_err(|e| format!("Failed to publish event: {}", e))
    }

    /// 启动事件分发循环
    pub async fn start_dispatch(&self) {
        info!("Starting event dispatch loop");

        loop {
            let event = {
                let mut receiver = self.receiver.write().await;
                receiver.recv().await
            };

            match event {
                Some(event) => {
                    let event_name = event.event_type.name();
                    debug!("Dispatching event: {}", event_name);

                    let handlers = {
                        let subs = self.subscribers.read().await;
                        subs.get(&event_name).cloned()
                    };

                    if let Some(handlers) = handlers {
                        for handler in handlers {
                            let event_clone = event.clone();
                            tokio::spawn(async move {
                                handler(event_clone).await;
                            });
                        }
                    }
                }
                None => {
                    info!("Event channel closed, stopping dispatch");
                    break;
                }
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let received = Arc::new(RwLock::new(false));
        let received_clone = received.clone();

        // 订阅事件
        bus.subscribe("test_event", move |_event| {
            let received = received_clone.clone();
            async move {
                let mut r = received.write().await;
                *r = true;
            }
        }).await;

        // 启动分发循环（在后台运行）
        let bus_clone = EventBus {
            subscribers: bus.subscribers.clone(),
            sender: bus.sender.clone(),
            receiver: bus.receiver.clone(),
        };
        tokio::spawn(async move {
            bus_clone.start_dispatch().await;
        });

        // 发布事件
        let event = Event::new(
            EventType::Custom("test_event".to_string()),
            serde_json::json!({"data": "test"}),
        );
        bus.publish(event).unwrap();

        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let r = received.read().await;
        assert!(*r);
    }

    #[test]
    fn test_event_type_name() {
        let event_type = EventType::DataUpdated("brain".to_string());
        assert_eq!(event_type.name(), "data_updated:brain");

        let event_type = EventType::Custom("my_event".to_string());
        assert_eq!(event_type.name(), "my_event");
    }
}