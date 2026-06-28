//! Task management — task creation and status updates.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Agent, AgentBadge, TaskStatus};

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
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
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct TaskStatusUpdateParams {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    pub status: TaskStatus,
    pub progress: u8,
}
