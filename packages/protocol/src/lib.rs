//! Shared protocol types for the entelecheia multi-agent platform.
//!
//! Every type in this crate MUST be:
//! - Defined in entelecheia (canonical source of truth)
//! - Consumed by shittim-chest (core, mock_scepter, or webui)
//!
//! If a type is not paired on both sides, it does not belong here.

pub mod http;
pub mod jsonrpc;
pub mod mcp;

#[cfg(feature = "tracing-helpers")]
pub mod tracing_helpers;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Core enums
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum Agent {
    HapLotes,
    SkoPeo,
    HubRis,
    KaLos,
    NeiKos,
    SkeMma,
    ApoRia,
    EleOs,
    EpieiKeia,
    OreXis,
    PhiLia,
    PoleMos,
    WebAutomation,
    ClassicSoftwareEngineering,
    WebUiPanel,
    IndustrialIoT,
    RemoteOperations,
}

#[derive(JsonSchema, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentBadge(pub String);

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum AgentStatus {
    Initializing,
    Online,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum WorkStatus {
    Thinking,
    StreamingResponse,
    Executing { skill_name: String },
    Retrying { retry_count: u32, max_retries: u32 },
    Nudging,
    Completed,
    RequestFailed,
    Failed,
    ToolLoopTerminated,
    CallingTool,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum RequestState {
    #[default]
    Idle,
    Waiting,
    Streaming,
    Retrying,
    WaitingTool,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum CompletionOutcome {
    #[default]
    None,
    Reported,
    Failed,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum ModelTier {
    Deep,
    Normal,
    Basic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum SkillStage {
    Started(String),
    Done(String),
    Complete(String),
    Failed(String),
    ToolCall(String),
    Retrying(String, usize, usize, Option<RetryReason>),
    TryingModel(String, String),
    ModelFailed(String, String, String),
    Nudging(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum RetryReason {
    EmptyOutput,
    ReportNotCaptured,
    LlmError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Query,
    Human,
    Reply,
    SkillTerminal,
    SkillStep,
    NextActionFallback,
    ChainMaxDepth,
    ChainCycle,
    SkillFailed,
    SkillEmptyOutput,
    SkillMissingReport,
    Error,
    System,
    /// Emitted when the server begins processing a user message. Acts as a
    /// transient placeholder — the real `Reply`/`Error` report replaces it
    /// once `task_decompose` (or a downstream skill) finishes. Tui renders
    /// this as a status indicator rather than a resident card.
    Pending,
}

/// Selection semantics for an inquiry (`report_type: "query"`) report's
/// `preset_options`. Defaults to `Single` when omitted.
#[derive(JsonSchema, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ReportSelection {
    #[default]
    Single,
    Multiple,
}

#[derive(JsonSchema, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum StreamChunkKind {
    #[default]
    Text,
    Thinking,
    DeepThinking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum StreamSegment {
    Text {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
    },
    Thinking {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
    },
    DeepThinking {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
    },
    McpCall {
        tool_name: String,
        call_id: String,
        #[ts(type = "unknown")]
        params: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        agent_type: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
    },
    McpResult {
        tool_name: String,
        call_id: String,
        success: bool,
        #[ts(type = "unknown")]
        data: serde_json::Value,
        #[serde(default)]
        #[ts(optional)]
        duration_ms: Option<u64>,
        #[serde(default)]
        #[ts(optional)]
        agent_type: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct LlmStream {
    #[serde(default)]
    pub segments: Vec<StreamSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RouteInfo {
    pub direction: String,
    pub target: String,
    #[serde(default)]
    #[ts(optional)]
    pub target_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS, thiserror::Error)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorCode {
    #[error("model has no providers")]
    ModelNoProviders,
    #[error("model has no models")]
    ModelNoModels,
    #[error("model tier mismatch")]
    ModelTierMismatch,
    #[error("all models excluded")]
    ModelAllExcluded,
    #[error("model env incomplete")]
    ModelEnvIncomplete,
    #[error("model selection retry exhausted")]
    ModelSelectionRetryExhausted,
    #[error("LLM call failed")]
    LlmCallFailed,
    #[error("LLM empty response")]
    LlmEmptyResponse,
    #[error("LLM rate limited")]
    LlmRateLimited,
    #[error("LLM auth failed")]
    LlmAuthFailed,
    #[error("LLM timeout")]
    LlmTimeout,
    #[error("cosmos no connection")]
    CosmosNoConnection,
    #[error("cosmos tool failed")]
    CosmosToolFailed,
    #[error("cosmos local unavailable")]
    CosmosLocalUnavailable,
    #[error("chain max depth")]
    ChainMaxDepth,
    #[error("chain cycle detected")]
    ChainCycle,
    #[error("chain failed")]
    ChainFailed,
    #[error("skill failed")]
    SkillFailed,
    #[error("skill empty output")]
    SkillEmptyOutput,
    #[error("skill missing report")]
    SkillMissingReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct StructuredAgentError {
    pub code: AgentErrorCode,
    #[serde(default)]
    #[ts(optional)]
    pub detail: Option<String>,
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,
}

#[derive(JsonSchema, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Failed,
    Warning,
    Waiting { deadline: String, handle: String },
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ContainerStatus {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum PeriodType {
    Hour5,
    Day7,
    Month1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum KnowledgeBaseStatus {
    #[default]
    Uninitialized,
    Indexing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum EmbeddingModel {
    OpenAiSmall,
    OpenAiLarge,
    OpenAiAda,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum YoloTaskTier {
    Realtime,
    Periodic,
    Daily,
    Strategic,
}

// ═══════════════════════════════════════════════════════════════
// Connection / Handshake
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HandshakeAckParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ScepterIdentityParams {
    #[ts(type = "string")]
    pub device_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PingParams {
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════
// Client capability + handshake payload
//
// Mirrors `entelecheia/packages/shared/state_types/src/gateway/
// tui_types/message/types/mod.rs`. The webui declares capabilities
// in its `Tui.ConnectHandshake` so scepter's `client_node_registry`
// can route capability-scoped requests back to it (e.g. NOA
// handshakes are only sent to sessions that declared
// `ClientCapability::NoaWorkspace`).
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    FileRelay,
    Terminal,
    ScreenCapture,
    NoaWorkspace,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ClientNodeInfo {
    pub hostname: String,
    pub os: String,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ConnectHandshakeParams {
    pub token: String,
    #[serde(default)]
    #[ts(optional)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<ClientCapability>,
    #[serde(default)]
    #[ts(optional)]
    pub node_info: Option<ClientNodeInfo>,
    /// Stringified UUID — kept as a string on the wire so consumers
    /// don't need a UUID parser to round-trip the JSON-RPC payload.
    #[serde(default)]
    #[ts(optional)]
    pub workspace_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// NOA Workspace — TuiMessage variant params
//
// Mirrors `entelecheia/packages/shared/state_types/src/gateway/
// tui_types/message/types/mod.rs` lines 677-720. The NOA handshake
// is a 4-message round trip:
//
//   scepter → client   RequestNoaHandshake
//   client  → scepter  NoaHandshakeResponse
//   scepter → client   NoaAuthRequest   (branch picker)
//   client  → scepter  NoaAuthResponse  (user's choice)
//   scepter → client   NoaReady         (terminal event)
//
// Plus a bidirectional event-sync pair used after NoaReady.
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    #[serde(default)]
    #[ts(optional)]
    pub file_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub content_hash: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "unknown")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestNoaHandshakeParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub remote_name: String,
    pub remote_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaHandshakeResponseParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub repo_id: String,
    pub current_branch: String,
    #[serde(default)]
    pub noa_initialized: bool,
    #[serde(default)]
    pub gitignore_updated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaAuthRequestParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub branches: Vec<String>,
    pub suggested_branch: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaAuthResponseParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub selected_branch: String,
    #[serde(default)]
    #[ts(optional)]
    pub branch_base: Option<String>,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaReadyParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub branch: String,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaEventSyncParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub events: Vec<NoaEvent>,
    #[serde(default)]
    #[ts(optional)]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct NoaEventSyncAckParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub last_event_id: String,
}

// ═══════════════════════════════════════════════════════════════
// Log entry
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct LogEntryData {
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub instance_uuid: Option<String>,
    pub level: String,
    #[serde(default)]
    #[ts(optional)]
    pub target: Option<String>,
    pub message: String,
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub fields: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ServerLogEntryParams {
    pub entry: LogEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ContainerLogEntryParams {
    pub instance_uuid: String,
    pub entry: LogEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SubscribeLogsResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    pub entries: Vec<LogEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UnsubscribeLogsResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Agent lifecycle
// ═══════════════════════════════════════════════════════════════

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentListResponseParams {
    pub agents: Vec<TuiAgentInfo>,
}

// ═══════════════════════════════════════════════════════════════
// Task management
// ═══════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════
// LLM Provider configuration
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct LlmProviderConfiguredParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderRenamedParams {
    pub provider_name: String,
    pub new_display_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderEditedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderDeletedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ConfiguredProviderInfo {
    pub provider_name: String,
    pub display_name: String,
    #[serde(default)]
    #[ts(optional)]
    pub api_endpoint: Option<String>,
    pub default_model: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ConfiguredProvidersListParams {
    pub providers: Vec<ConfiguredProviderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelProviderConfigUpdatedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct EndpointValidatedParams {
    pub provider_name: String,
    pub is_reachable: bool,
    #[serde(default)]
    #[ts(optional)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UsagePeriodData {
    pub user_id: String,
    pub period_type: PeriodType,
    pub used_tokens: u64,
    pub cost: f64,
    pub start_time: String,
    #[serde(default)]
    #[ts(optional)]
    pub remaining_tokens: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub remaining_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UsagePeriodResponseParams {
    pub data: Vec<UsagePeriodData>,
}

// ═══════════════════════════════════════════════════════════════
// State sync / Snapshots
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
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
#[ts(export, export_to = "WsTypes.ts")]
pub struct GlobalSnapshotParams {
    pub snapshot: GlobalSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct GlobalSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub containers: Vec<ContainerInfo>,
    pub active_tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ContainerSnapshotParams {
    pub snapshot: ContainerSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ContainerSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct TasksSnapshotParams {
    pub snapshot: TasksSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct TasksSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}

// ═══════════════════════════════════════════════════════════════
// Config Filesystem
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct EntrypointApiConfigInfo {
    pub protocol: String,
    pub base_url: String,
    pub chat_endpoint: String,
    #[serde(default)]
    #[ts(optional)]
    pub models_endpoint: Option<String>,
    pub auth_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub auth_header: Option<String>,
    pub env_var: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct MaxConcurrentInfo {
    #[serde(default)]
    pub deep: usize,
    #[serde(default)]
    pub normal: usize,
    #[serde(default)]
    pub basic: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct EntrypointDefaultsInfo {
    #[serde(default)]
    pub deep: Vec<String>,
    #[serde(default)]
    pub normal: Vec<String>,
    #[serde(default)]
    pub basic: Vec<String>,
    #[serde(default)]
    pub max_concurrent: MaxConcurrentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct QuotaInfo {
    pub data_limit: u64,
    #[serde(default)]
    #[ts(optional)]
    pub period_hours: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub period_days: Option<u32>,
    #[serde(default)]
    pub billing_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct EntrypointConfigInfo {
    pub id: String,
    #[serde(default)]
    pub name: std::collections::HashMap<String, String>,
    #[serde(default, rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub billing_type: String,
    #[serde(default)]
    pub plan_tier: String,
    pub api: EntrypointApiConfigInfo,
    #[serde(default)]
    pub defaults: EntrypointDefaultsInfo,
    #[serde(default)]
    pub quotas: Vec<QuotaInfo>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderCapabilitiesInfo {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub function_calling: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub reasoning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderLimitsInfo {
    #[serde(default)]
    pub max_concurrent: u32,
    #[serde(default)]
    #[ts(optional)]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default)]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderFsInfoParams {
    pub providers: Vec<ProviderFsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelFsPricing {
    #[serde(default)]
    #[ts(optional)]
    pub input_per_million: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub output_per_million: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub cached_per_million: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RateRuleInfo {
    pub timezone_offset: i32,
    pub peak_start: u32,
    pub peak_end: u32,
    pub peak_multiplier: f64,
    pub off_peak_multiplier: f64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelFsInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u64,
    pub max_output_tokens: u64,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default = "default_true")]
    pub supports_function_calling: bool,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub pricing: Option<ModelFsPricing>,
    #[serde(default)]
    #[ts(optional)]
    pub rate_multiplier: Option<f64>,
    #[serde(default)]
    pub rate_rules: Vec<RateRuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelFsInfoParams {
    pub models: Vec<ModelFsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderFsInfo {
    pub id: String,
    #[serde(default)]
    pub name: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    #[ts(optional)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub entry_points: Vec<EntrypointConfigInfo>,
    #[serde(default)]
    pub models: Vec<ModelFsInfo>,
    #[serde(default)]
    pub capabilities: ProviderCapabilitiesInfo,
    #[serde(default)]
    pub limits: ProviderLimitsInfo,
    #[serde(default)]
    pub pricing_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UserConfigResponseParams {
    pub config: UserInfoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UserInfoConfig {
    pub preferred_language: String,
    #[serde(default)]
    pub default_models: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub enabled_providers: Vec<String>,
    #[serde(default)]
    pub enabled_models: Vec<String>,
    #[serde(default)]
    pub model_priorities: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub auto_import_from_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct KeysListResponseParams {
    pub keys: Vec<KeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct KeyInfo {
    pub provider: String,
    pub display_name: String,
    pub has_key: bool,
    pub created_at: String,
    pub updated_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ApiKeyInfoResponseParams {
    pub info: KeyInfo,
}

// ═══════════════════════════════════════════════════════════════
// Knowledge Base
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct KbGenericResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct KnowledgeBaseInfo {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: KnowledgeBaseStatus,
    #[serde(default)]
    #[ts(optional)]
    pub embedding_model: Option<EmbeddingModel>,
    #[serde(default)]
    #[ts(optional)]
    pub custom_embedding_endpoint: Option<String>,
    pub document_count: usize,
    #[serde(default)]
    pub subscription_count: usize,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ListKnowledgeBasesResponseParams {
    pub knowledge_bases: Vec<KnowledgeBaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct GetKnowledgeBaseResponseParams {
    pub knowledge_base: Option<KnowledgeBaseInfo>,
}

// ═══════════════════════════════════════════════════════════════
// Layer-2 / Custom Agents
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentInfo {
    pub name: String,
    pub description: String,
    pub mcp_count: usize,
    pub skills_count: usize,
    pub languages: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentListResponseParams {
    pub agents: Vec<Layer2AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2McpToolInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub references_layer1: Vec<String>,
    pub references_layer2: Vec<String>,
    pub related_items: Vec<String>,
    pub referenced_by_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentMcpResponseParams {
    pub agent_name: String,
    pub tools: Vec<Layer2McpToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2SkillInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub references_layer1: Vec<String>,
    pub references_layer2: Vec<String>,
    pub related_items: Vec<String>,
    pub referenced_by_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentSkillsResponseParams {
    pub agent_name: String,
    pub skills: Vec<Layer2SkillInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2McpPromptResponseParams {
    pub agent_name: String,
    pub tool: String,
    pub lang: String,
    pub content: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2SkillPromptResponseParams {
    pub agent_name: String,
    pub skill: String,
    pub lang: String,
    pub content: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CustomAgentInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub skills_count: usize,
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub version: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CustomAgentListResponseParams {
    pub agents: Vec<CustomAgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SubscribeCustomAgentResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub agent: Option<CustomAgentInfo>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UnsubscribeCustomAgentResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Workspace
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceStatusParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub connection_kind: String,
    #[serde(default)]
    #[ts(optional)]
    pub resolved_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub remote_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub branch: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub host_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PolemosDeviceInfo {
    #[ts(type = "string")]
    pub node_id: uuid::Uuid,
    pub name: String,
    pub address: String,
    pub status: String,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ide_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PolemosDeviceListParams {
    pub devices: Vec<PolemosDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RegisterPolemosDeviceResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub device: Option<PolemosDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SwitchWorkspaceResponseParams {
    pub success: bool,
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// System / UI
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebUiControlResponseParams {
    pub command: String,
    pub success: bool,
    pub message: String,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebUiStatusParams {
    pub running: bool,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub container_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Authentication
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthLoginResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub token: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub session_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub display_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub role: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthRegisterResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UserProfileSummary {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthListUsersResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub users: Option<Vec<UserProfileSummary>>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthGetUserResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub user: Option<UserProfileSummary>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthDeleteUserResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AuthChangePasswordResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// YOLO Cruise Control
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStartResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStopResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTerminateResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTaskResult {
    pub success: bool,
    pub duration_ms: u64,
    pub completed_at: String,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTaskStatus {
    pub agent: String,
    pub skill: String,
    pub enabled: bool,
    #[serde(default)]
    #[ts(optional)]
    pub last_result: Option<YoloTaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierStatus {
    pub tier: YoloTaskTier,
    pub enabled: bool,
    pub interval_secs: u64,
    #[serde(default)]
    #[ts(optional)]
    pub last_run_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub next_run_at: Option<String>,
    #[serde(default)]
    pub tasks: Vec<YoloTaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStatusResponseParams {
    pub active: bool,
    pub loop_count: u64,
    #[serde(default)]
    #[ts(optional)]
    pub started_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub current_cycle: Option<String>,
    #[serde(default)]
    pub tiers: Vec<YoloTierStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloCycleStepParams {
    pub skill: String,
    pub loop_count: u64,
    pub status: String,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloCycleCompleteParams {
    pub loop_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierTaskConfig {
    pub agent: String,
    pub skill: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierConfig {
    pub tier: YoloTaskTier,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub interval_secs: u64,
    #[serde(default)]
    pub tasks: Vec<YoloTierTaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloConfigResponseParams {
    pub tiers: Vec<YoloTierConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloUpdateTaskResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloSetTierIntervalResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloRunTierNowResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Base messages
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct BaseHeartbeatParams {
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct BaseErrorParams {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct BaseAckParams {
    pub message_id: String,
}

// ═══════════════════════════════════════════════════════════════
// Industrial Control — Telemetry, Alarms, Discovery, Write Approval
// ═══════════════════════════════════════════════════════════════

// ── Telemetry ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialSensorReading {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub name: String,
    pub raw_value: f64,
    pub scaled_value: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub quality: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialTelemetryBatch {
    pub readings: Vec<IndustrialSensorReading>,
    pub station_id: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialTelemetryPushParams {
    pub batch: IndustrialTelemetryBatch,
}

// ── Alarms ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum IndustrialAlarmLevel {
    Log,
    LowLow,
    Low,
    High,
    HighHigh,
    RateOfChange,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialAlarmEvent {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    #[serde(default)]
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialAlarmPushParams {
    pub alarm: IndustrialAlarmEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialAlarmAckParams {
    pub station_id: String,
    pub address: String,
    pub acknowledged_by: String,
}

// ── Discovery ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum DiscoveryPhase {
    TransportScan,
    ProtocolIdentify,
    DataModelScan,
    SemanticInference,
    ManifestGeneration,
    ManifestValidation,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DiscoveryProgressEvent {
    pub session_id: String,
    pub phase: DiscoveryPhase,
    pub message: String,
    #[serde(default)]
    pub found_devices: u32,
    #[serde(default)]
    pub progress_percent: u8,
    #[serde(default)]
    #[ts(optional)]
    pub raw_findings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DiscoveryProgressPushParams {
    pub event: DiscoveryProgressEvent,
}

// ── Write Approval (human-in-the-loop) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WriteApprovalRequest {
    /// Mirrors `_shared_state_sync::WriteApprovalRequest::request_id`. The
    /// operator UI echoes this back in `industrial.approveWrite` so the
    /// resolver can match the response to the pending producer oneshot.
    #[serde(default)]
    pub request_id: String,
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub current_value: f64,
    pub proposed_value: f64,
    #[serde(default)]
    pub unit: String,
    pub reason: String,
    pub agent: String,
    #[serde(default)]
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WriteApprovalRequestParams {
    pub request: WriteApprovalRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WriteApprovalResponseParams {
    pub request_id: String,
    pub approved: bool,
    pub approved_by: String,
    #[serde(default)]
    #[ts(optional)]
    pub modified_value: Option<f64>,
}

// ── Topology (station metadata for UI) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AlarmThresholdInfo {
    #[serde(default)]
    #[ts(optional)]
    pub ll: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub l: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub h: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub hh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct StationFieldInfo {
    pub address: String,
    pub name: String,
    #[serde(default)]
    pub data_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub alarm: Option<AlarmThresholdInfo>,
    #[serde(default)]
    #[ts(optional)]
    pub current_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialStationInfo {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub connection: String,
    #[serde(default)]
    pub device_class: String,
    #[serde(default)]
    #[ts(optional)]
    pub vendor: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub model: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub firmware: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub fields: Vec<StationFieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IndustrialTopologyParams {
    pub stations: Vec<IndustrialStationInfo>,
}

// ═══════════════════════════════════════════════════════════════
// Abstract View Interface — pluggable dashboard views
// ═══════════════════════════════════════════════════════════════

/// View type identifier — determines which frontend renderer handles the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum ViewKind {
    /// Industrial SCADA / HMI panel (P&ID, gauges, alarm panel, trend charts)
    IndustrialScada,
    /// Chat / conversation interface (default demiurge view)
    Chat,
    /// Kanban board (task cards, columns, drag-drop)
    Kanban,
    /// Gantt chart (timeline, milestones, dependencies)
    Gantt,
    /// Data table / spreadsheet (like Feishu multi-dimensional table)
    DataTable,
    /// Audio/video generation flow (node graph like ComfyUI)
    MediaFlow,
    /// File explorer / code browser
    FileExplorer,
    /// Custom (plugin-rendered)
    Custom,
}

/// A view instance — one panel in the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ViewInstance {
    /// Unique view ID within the workspace.
    pub view_id: String,
    /// What kind of renderer to use.
    pub kind: ViewKind,
    /// Display title.
    pub title: String,
    /// Data source identifier — what the view is bound to.
    /// Examples: "industrial:station:19", "chat:conversation:abc",
    /// "kanban:project:xyz", "media:flow:comfyui"
    pub data_source: String,
    /// View-specific configuration (JSON, interpreted by the renderer).
    #[serde(default)]
    pub config: serde_json::Value,
    /// Layout position (grid area, tab order, etc.).
    #[serde(default)]
    #[ts(optional)]
    pub layout: Option<ViewLayout>,
}

/// Layout descriptor for a view within the dashboard grid.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ViewLayout {
    /// Grid column start (1-based).
    #[serde(default)]
    pub col: u32,
    /// Grid row start (1-based).
    #[serde(default)]
    pub row: u32,
    /// Column span.
    #[serde(default)]
    pub col_span: u32,
    /// Row span.
    #[serde(default)]
    pub row_span: u32,
    /// Minimum width in pixels.
    #[serde(default)]
    #[ts(optional)]
    pub min_width: Option<u32>,
    /// Minimum height in pixels.
    #[serde(default)]
    #[ts(optional)]
    pub min_height: Option<u32>,
}

/// Dashboard layout — a collection of views arranged in a grid.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DashboardLayout {
    /// Workspace ID this dashboard belongs to.
    pub workspace_id: String,
    /// Dashboard name.
    pub name: String,
    /// All view instances in this dashboard.
    pub views: Vec<ViewInstance>,
    /// Grid columns count (0 = auto).
    #[serde(default)]
    pub grid_columns: u32,
}

/// Push a dashboard layout update to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DashboardLayoutPushParams {
    pub layout: DashboardLayout,
}

/// View data update — incremental data push for a specific view.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ViewDataPushParams {
    /// Target view ID.
    pub view_id: String,
    /// Data payload (format depends on ViewKind).
    pub data: serde_json::Value,
    /// Whether this is a full replacement or incremental update.
    #[serde(default)]
    pub full_replace: bool,
}

// ═══════════════════════════════════════════════════════════════
// File Browsing — TuiMessage variant params
//
// Browse/read files inside a container (#demiurge / #NNN), on a host
// machine, or in a workspace checkout. Targets are distinguished by
// `FileTargetKind`. The node-list container cards, the Bridge Network
// host cards and the workspace cards all open this same file browser.
//
// Mirrors `entelecheia/.../tui_types/message/types/mod.rs`.
// ═══════════════════════════════════════════════════════════════

/// Which filesystem a file operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum FileTargetKind {
    /// A container slot — `#demiurge` or `#NNN` (id is the slot badge).
    Container,
    /// A host machine (id is the host_id / device id; "localhost" for self).
    Host,
    /// A workspace checkout (id is the workspace_id).
    Workspace,
}

/// A file-operation target — a (kind, id) pair plus optional workspace
/// context (container slots are per-workspace).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct FileTarget {
    pub kind: FileTargetKind,
    /// Container badge (`#demiurge` / `#001`), host id, or workspace id.
    pub id: String,
    /// Owning workspace id (container slots are workspace-scoped).
    #[serde(default)]
    #[ts(optional, type = "string")]
    pub workspace_id: Option<uuid::Uuid>,
}

/// One entry in a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct FileTreeEntry {
    pub name: String,
    /// `"file"` | `"dir"` | `"symlink"`.
    pub kind: String,
    pub size: u64,
}

/// `Tui.RequestFileTree` — list one level of a directory.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestFileTreeParams {
    pub target: FileTarget,
    /// Sub-path under the target root (empty/`""` = root).
    #[serde(default)]
    pub path: String,
}

/// `Tui.FileTree` — directory listing response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct FileTreeParams {
    pub target: FileTarget,
    pub path: String,
    pub entries: Vec<FileTreeEntry>,
}

/// `Tui.RequestFileRead` — read a single (text) file, capped server-side.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestFileReadParams {
    pub target: FileTarget,
    pub path: String,
}

/// `Tui.FileRead` — file-content response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct FileReadParams {
    pub target: FileTarget,
    pub path: String,
    pub content: String,
    pub size: u64,
    /// `true` when content was truncated to the server read cap.
    #[serde(default)]
    pub truncated: bool,
}

// ═══════════════════════════════════════════════════════════════
// Bridge Network — host machines + their workspaces
//
// The 3rd chat sub-page ("桥接网络"): the left column lists host machines
// (localhost + remote polemos devices) with live performance; the right
// column lists the workspaces attached to the selected host with their
// noa-git status + token usage. Clicking a host opens its file browser
// (default /home); clicking a workspace opens its on-disk directory.
//
// Mirrors `entelecheia/.../tui_types/message/types/mod.rs`.
// ═══════════════════════════════════════════════════════════════

/// Live performance snapshot for one host machine.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HostMetrics {
    /// Stable host id ("localhost" for self, or a polemos device id).
    pub host_id: String,
    pub hostname: String,
    pub os: String,
    /// CPU utilisation, 0..100.
    pub cpu_usage_percent: f64,
    /// Logical CPU core count (shown with an i18n "cores" unit).
    pub cpu_cores: u32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    /// Outbound network rate (bytes/sec). Omitted when unknown.
    #[serde(default)]
    #[ts(optional)]
    pub net_up_bps: Option<u64>,
    /// Inbound network rate (bytes/sec). Omitted when unknown.
    #[serde(default)]
    #[ts(optional)]
    pub net_down_bps: Option<u64>,
}

/// noa-git status for a workspace checkout (branch / dirty / ahead / behind).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceGitStatus {
    pub branch: String,
    /// Modified/untracked file count.
    #[serde(default)]
    pub modified: u32,
    /// Commits ahead of upstream.
    #[serde(default)]
    pub ahead: u32,
    /// Commits behind upstream.
    #[serde(default)]
    pub behind: u32,
    /// `true` when there are uncommitted changes.
    #[serde(default)]
    pub dirty: bool,
}

/// One agent's token usage within a workspace (top-N entries).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceTokenUsage {
    pub agent: String,
    pub input: u64,
    pub output: u64,
}

/// A workspace attached to a host, with its git + token-usage summary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceNode {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub host_id: String,
    pub path: String,
    #[serde(default)]
    #[ts(optional)]
    pub alias: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub git: Option<WorkspaceGitStatus>,
    /// Top token consumers in this workspace (max 3).
    #[serde(default)]
    pub token_usage: Vec<WorkspaceTokenUsage>,
}

/// `Tui.RequestBridgeNetwork` — request the host/workspace roster.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestBridgeNetworkParams {}

/// `Tui.BridgeNetwork` — the host/workspace roster response/push.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct BridgeNetworkParams {
    pub hosts: Vec<HostMetrics>,
    pub workspaces: Vec<WorkspaceNode>,
}
