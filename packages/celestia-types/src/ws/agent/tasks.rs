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
    /// Task Decompose estimate in degrees; absent = no estimate produced.
    /// Consumers MUST use `!= null` — 0 is a legitimate zero-cost estimate.
    ///
    /// Round-21's finding: this field (and TaskEstimateUpdatedParams
    /// below) existed ONLY as a hand-edit in the generated binding — any
    /// regeneration silently wiped them from the published npm surface.
    /// They now live in the SOURCE, so regeneration is faithful.
    #[serde(default)]
    #[ts(optional)]
    pub estimated_degrees: Option<f64>,
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/tasks.ts")]
pub struct TaskEstimateUpdatedParams {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub estimated_degrees: Option<f64>,
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/tasks.ts")]
pub struct TaskStatusUpdateParams {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    pub status: TaskStatus,
    pub progress: u8,
}
