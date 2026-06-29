//! State sync / snapshots — global, container, and task snapshots.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AgentBadge, ContainerStatus, TaskStatus, agent_lifecycle::TuiAgentInfo};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: ContainerStatus,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    #[ts(optional)]
    pub agent_type: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub branch_level: u32,
    #[serde(default)]
    pub is_cosmos: bool,
    #[serde(default)]
    #[ts(optional)]
    pub branch: Option<String>,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub badge: Option<AgentBadge>,
    #[serde(default)]
    #[ts(optional)]
    pub current_skill: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub git_remote_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub git_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct TaskInfo {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    #[ts(type = "string")]
    pub issue_id: uuid::Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub progress: u8,
    #[serde(default)]
    #[ts(optional)]
    pub assigned_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct GlobalSnapshotParams {
    pub snapshot: GlobalSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct GlobalSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub containers: Vec<ContainerInfo>,
    pub active_tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct ContainerSnapshotParams {
    pub snapshot: ContainerSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct ContainerSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct TasksSnapshotParams {
    pub snapshot: TasksSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/stateSync.ts")]
pub struct TasksSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}
