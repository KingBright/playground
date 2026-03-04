use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatCardData {
    pub label: String,
    pub value: String,
    pub change: Option<i32>,
    pub change_label: Option<String>,
    pub icon: String,
    pub icon_color: String,
    pub trend: Option<String>, // "up" | "down" | "stable"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    pub id: String,
    pub name: String,
    pub status: String, // "running" | "paused" | "completed" | "error"
    pub environment: String,
    pub agents: Vec<String>,
    pub start_time: String,
    pub progress: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String, // "universal" | "local"
    pub description: String,
    pub capabilities: Vec<String>,
    pub status: String, // "active" | "inactive" | "error"
    pub version: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSlice {
    pub id: String,
    pub name: String,
    pub node_count: u32,
    pub status: String, // "active" | "inactive"
    pub last_updated: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String, // "manual" | "scheduled" | "event"
    pub status: String, // "active" | "paused" | "completed"
    pub schedule: Option<String>,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub dashboard_stats: Vec<StatCardData>,
    pub active_simulations: Vec<Simulation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskListResponse {
    pub tasks: Vec<ScheduledTask>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeListResponse {
    pub slices: Vec<KnowledgeSlice>,
}
