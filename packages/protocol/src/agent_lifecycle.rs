//! Agent lifecycle — streaming, responses, reports, orchestration, snapshots.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Agent, AgentBadge, AgentStatus, CompletionOutcome, LlmStream, ModelTier, ReportSelection,
    ReportType, RequestState, RouteInfo, SkillStage, StreamChunkKind, StructuredAgentError,
    WorkStatus,
};

// ═══════════════════════════════════════════════════════════════
// Agent lifecycle
// ═══════════════════════════════════════════════════════════════

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct AgentStreamingChunkParams {
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub chunk: String,
    pub is_done: bool,
    pub timestamp: String,
    #[serde(default)]
    #[ts(optional)]
    pub task_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub chunk_kind: Option<StreamChunkKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct AgentResponseParams {
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    #[ts(optional)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct AgentReportParams {
    pub report_type: ReportType,
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub title: String,
    pub content: String,
    #[serde(default)]
    #[ts(optional)]
    pub summary: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub preset_options: Vec<String>,
    /// For `report_type: "query"`: whether `preset_options` are mutually
    /// exclusive (single) or pick-any (multiple). Omit ⇒ single.
    #[serde(default)]
    #[ts(optional)]
    pub selection_mode: Option<ReportSelection>,
    /// For `report_type: "query"`: whether the recipient may type a free-form
    /// answer in addition to (or instead of) picking presets. Omit ⇒ true
    /// when `report_type == Query`, false otherwise.
    #[serde(default)]
    #[ts(optional)]
    pub allow_custom_reply: Option<bool>,
    /// Subset of `preset_options` the agent suggests. Empty when no
    /// recommendation is offered.
    #[serde(default)]
    pub recommended_options: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub model_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub skill_count: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub mcp_count: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub next_route: Option<RouteInfo>,
    #[serde(default)]
    #[ts(optional)]
    pub stream: Option<LlmStream>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<StructuredAgentError>,
}

/// Reply payload for an inquiry (`report_type: "query"`) report.
///
/// Wire method: `Tui.AgentReportReply` (server-bound). `report_id` mirrors
/// the `agent_id` of the originating `AgentReportParams` so the upstream can
/// correlate without keeping a separate consultation registry.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct AgentReportReplyParams {
    pub report_id: String,
    #[serde(default)]
    pub selected_options: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub custom_answer: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct OrchestrationStatusParams {
    pub stage: SkillStage,
    pub agent: String,
    #[serde(default)]
    #[ts(optional)]
    pub agent_type: Option<Agent>,
    #[serde(default)]
    #[ts(optional)]
    pub tool_name: Option<String>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub call_id: Option<uuid::Uuid>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub parent_agent: Option<uuid::Uuid>,
    #[serde(default)]
    #[ts(optional)]
    pub parameters_summary: Option<String>,
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct McpToolResultParams {
    pub tool_name: String,
    #[ts(type = "string")]
    pub call_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub parameters_summary: Option<String>,
    pub result: String,
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub duration_ms: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════
// Agent list / snapshot
// ═══════════════════════════════════════════════════════════════

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct TuiAgentInfo {
    pub agent_type: Agent,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    #[serde(default)]
    #[ts(optional)]
    pub agent_uuid: Option<String>,
    pub agent_id: String,
    pub status: AgentStatus,
    pub llm_working: bool,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    #[serde(default)]
    #[ts(optional)]
    pub parent_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub work_status: Option<WorkStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub current_model: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub model_tier: Option<ModelTier>,
    #[serde(default)]
    #[ts(optional)]
    pub llm_handle: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage: Option<(u32, u32)>,
    pub mcp_tool_calls: u32,
    pub request_state: RequestState,
    pub completion_outcome: CompletionOutcome,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/agent_lifecycle.ts")]
pub struct AgentListResponseParams {
    pub agents: Vec<TuiAgentInfo>,
}
