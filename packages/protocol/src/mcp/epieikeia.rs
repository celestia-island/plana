use serde_json::Value;
use uuid::Uuid;

use crate::enums::ConsultationStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TriggerAddResult {
    pub trigger_id: Uuid,
    pub trigger_type: String,
    pub event: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TriggerEntry {
    pub id: String,
    pub trigger_type: String,
    pub event: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TriggerListResult {
    pub count: usize,
    pub triggers: Vec<TriggerEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TriggerRemoveResult {
    pub trigger_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TaskScheduleResult {
    pub task_id: Uuid,
    pub schedule: Value,
    pub status: ConsultationStatus,
    pub timer_info: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TaskEntry {
    pub id: String,
    pub schedule: Value,
    pub status: ConsultationStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TaskListResult {
    pub count: usize,
    pub tasks: Vec<TaskEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct TaskCancelResult {
    pub task_id: Uuid,
    pub status: ConsultationStatus,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DeliverMessageParams {
    pub target_badge: String,
    pub message_type: String,
    pub content: String,
    pub suggested_skill: Option<String>,
    pub priority: Option<String>,
    pub source_badge: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct InjectUserPromptParams {
    pub target_badge: String,
    pub message: String,
    pub source_badge: Option<String>,
    pub suggested_skill: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ConsumeInjectedPromptsParams {
    pub target_badge: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ForkContainerOnNextActionParams {
    pub container_id: String,
    pub branch_prefix: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NotifyFileOperationParams {
    pub file_path: String,
    pub agent_type: String,
    pub instance_badge: Option<String>,
    pub observation_type: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ListFileObserversParams {
    pub file_path: String,
}

// ── Tool result structs ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct DeliverMessageResult {
    pub todo_id: Uuid,
    pub title: String,
    pub target_badge: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct InjectUserPromptResult {
    pub target_badge: String,
    pub injected: bool,
    pub prompt_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct InjectedPromptView {
    pub source_badge: String,
    pub message: String,
    pub suggested_skill: String,
    pub injected_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct ConsumeInjectedPromptsResult {
    pub consumed: Vec<InjectedPromptView>,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct ForkContainerRegistrationResult {
    pub status: String,
    pub message: String,
    pub container_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct NotifyFileOperationToolResult {
    pub file_path: String,
    pub observers_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct FileObserverView {
    pub agent_type: String,
    pub instance_badge: Option<String>,
    pub observation_type: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct ListFileObserversToolResult {
    pub file_path: String,
    pub observers: Vec<FileObserverView>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/epieikeia.ts")]
pub struct UnregisterFileOperationResult {
    pub status: String,
}
