//! Task management — task creation and status updates.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Agent, AgentBadge, TaskStatus};

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/tasks.ts")]
pub struct TaskCreatedParams {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    #[ts(type = "string")]
    pub issue_id: uuid::Uuid,
    pub title: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub assigned_agent: Option<Agent>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub parent_task_id: Option<uuid::Uuid>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub badge: Option<AgentBadge>,
    #[serde(default)]
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
    /// Topic correlation: the conversation this task was spawned for,
    /// when the spawning chain knows one.
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub conversation_id: Option<uuid::Uuid>,
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/tasks.ts")]
pub struct TaskStatusUpdateParams {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    pub status: TaskStatus,
    pub progress: u8,
}
