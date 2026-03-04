//! Synergy System - 神经中枢
//!
//! Agent调度、注册、协同机制

pub mod api;
pub mod events;
pub mod registry;
pub mod scheduler;

pub use api::SynergyApi;
pub use events::{Event, EventBus, EventType};
pub use registry::{AgentDefinition, AgentRegistry, AgentType};
pub use scheduler::{Mission, MissionControl, MissionExecution, MissionStatus, SchedulerConfig, TriggerType};

use common::Result;

/// Synergy版本
pub const SYNERGY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 初始化Synergy系统
pub fn init() -> Result<()> {
    tracing::info!("Synergy v{} initializing", SYNERGY_VERSION);
    Ok(())
}
