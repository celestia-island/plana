//! WebSocket protocol types — upstream-compatible definitions from entelecheia.
//!
//! # Allow(dead_code) rationale
//!
//! Many types in this module are not directly referenced by Rust consuming code
//! (core, mock_scepter, lsp-bridge) but are still required because:
//!
//! 1. **Upstream protocol fidelity** — these types are exact copies from
//!    entelecheia's `state_types` / `mcp_types` crates. They MUST be kept in
//!    sync with upstream even if not yet consumed locally.
//! 2. **TypeScript codegen** — the `#[ts(export)]` attribute generates TS type
//!    definitions consumed by the `arona` webui. A type that
//!    appears "dead" in Rust may be actively used in TypeScript.
//! 3. **Forward compatibility** — new features (knowledge base, cosmos VM
//!    snapshots, human review, etc.) will consume these types when implemented.
//!
//! Do NOT remove types from this module solely because they trigger dead_code
//! warnings. Only remove if the upstream entelecheia project has also removed them.

#![allow(dead_code)]

pub mod jsonrpc;

use serde::{Deserialize, Serialize};

use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Upstream-compatible enums (exact copies from entelecheia)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentBadge(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum AgentStatus {
    Initializing,
    Online,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum RequestState {
    #[default]
    Idle,
    Waiting,
    Streaming,
    Retrying,
    WaitingTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum CompletionOutcome {
    #[default]
    None,
    Reported,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum ModelTier {
    Deep,
    Normal,
    Basic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum RetryReason {
    EmptyOutput,
    ReportNotCaptured,
    LlmError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum StreamChunkKind {
    #[default]
    Text,
    Thinking,
    DeepThinking,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum StreamMcpEvent {
    McpCall {
        tool_name: String,
        call_id: String,
        #[serde(default)]
        #[ts(optional)]
        params_summary: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        agent_type: Option<String>,
    },
    McpResult {
        tool_name: String,
        call_id: String,
        result: String,
        success: bool,
        #[serde(default)]
        #[ts(optional)]
        duration_ms: Option<u64>,
        #[serde(default)]
        #[ts(optional)]
        agent_type: Option<String>,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum SystemNotification {
    WebUiStarted,
    WebUiStopped,
    WebUiRestarted,
    WebUiError {
        error: String,
    },
    WebUiUrl {
        url: String,
    },
    ContainerError {
        container: String,
        error: String,
    },
    CosmosError {
        agent: String,
        error: String,
    },
    ServerError {
        error: String,
    },
    AutoModeChanged {
        enabled: bool,
        timeout_secs: Option<u64>,
    },
    AutoModeUsage,
    WorkspaceOpened {
        repo_url: String,
        branch: Option<String>,
    },
    WorkspaceError {
        error: String,
    },
    Generic {
        key: String,
        params: Vec<String>,
    },
    SecurityPolicyChanged {
        action: String,
        details: String,
        changed_by: String,
    },
    SecurityToolBlocked {
        agent: String,
        tool: String,
        reason: String,
    },
    ScreenSessionStarted {
        #[ts(type = "string")]
        session_id: uuid::Uuid,
        node_id: String,
    },
    ScreenSessionEnded {
        #[ts(type = "string")]
        session_id: uuid::Uuid,
        node_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum AskAnswerSource {
    Human,
    Auto,
    Timeout,
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RouteInfo {
    pub direction: String,
    pub target: String,
    #[serde(default)]
    #[ts(optional)]
    pub target_token: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// JSON-RPC notification builder
// ═══════════════════════════════════════════════════════════════
// Protocol / Connection
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ServerVersionParams {
    pub version: String,
    pub build_info: String,
}

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
// Log entry (shared for server + container logs)
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentTransferParams {
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub from_skill: String,
    pub to_skill: String,
    #[serde(default)]
    #[ts(optional)]
    pub summary: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub model_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub stream: Option<LlmStream>,
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct StreamingTailParams {
    pub agent_id: String,
    pub tail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HumanReviewRequestParams {
    pub review_id: String,
    pub agent_type: Agent,
    pub agent_id: String,
    pub title: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HumanReviewResponseParams {
    pub review_id: String,
    pub choice: String,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AskHumanRequestParams {
    pub consultation_id: String,
    pub agent_type: Agent,
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    pub question: String,
    pub question_localized: String,
    #[serde(default)]
    #[ts(optional)]
    pub context: Option<String>,
    pub options: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub recommended: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AskHumanReplyParams {
    pub consultation_id: String,
    pub selected_options: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub custom_answer: Option<String>,
    pub answered_by: AskAnswerSource,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AutoModeUpdateParams {
    pub enabled: bool,
    #[serde(default)]
    #[ts(optional)]
    pub timeout_secs: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════
// Agent list / snapshot
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentUpdateParams {
    pub agent: TuiAgentInfo,
}

// ═══════════════════════════════════════════════════════════════
// Task management
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
pub struct AgentSnapshotParams {
    pub snapshot: AgentSnapshotData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentSnapshotData {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SnapshotModelInfo {
    pub id: String,
    pub name: String,
    pub provider_name: String,
    pub model_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub context_length: Option<u32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SnapshotProviderInfo {
    pub name: String,
    pub display_name: String,
    pub api_endpoint: String,
    pub has_api_key: bool,
    #[serde(default)]
    #[ts(optional)]
    pub default_model: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelsSnapshotParams {
    #[serde(default)]
    pub models: Vec<SnapshotModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProvidersSnapshotParams {
    #[serde(default)]
    pub providers: Vec<SnapshotProviderInfo>,
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
// Snapshot patches (upstream: state_types::gateway::tui_types::snapshot)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AgentPatch {
    pub agent_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub agent_number: Option<AgentBadge>,
    #[serde(default)]
    #[ts(optional)]
    pub agent_type: Option<Agent>,
    pub version: u64,
    #[serde(default)]
    #[ts(optional)]
    pub llm_working_changed: Option<bool>,
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
    pub token_usage_delta: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage_absolute: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub request_state: Option<RequestState>,
    #[serde(default)]
    #[ts(optional)]
    pub mcp_tool_calls_delta: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub skill_calls_delta: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub cpu_usage: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub memory_mb: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub completion_outcome: Option<CompletionOutcome>,
    #[serde(default)]
    #[ts(optional)]
    pub retry_count: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub max_retries: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub current_stage: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub next_stage: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub current_tool_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ContainerPatch {
    pub container_id: String,
    pub version: u64,
    #[serde(default)]
    #[ts(optional)]
    pub status_changed: Option<ContainerStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub cpu_usage_changed: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub memory_usage_changed: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub branch_changed: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub is_read_only_changed: Option<bool>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub badge_changed: Option<AgentBadge>,
    #[serde(default)]
    #[ts(optional)]
    pub current_skill_changed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct TaskPatch {
    #[ts(type = "string")]
    pub task_id: uuid::Uuid,
    pub version: u64,
    #[serde(default)]
    #[ts(optional)]
    pub status_changed: Option<TaskStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub progress_changed: Option<u8>,
}

// ═══════════════════════════════════════════════════════════════
// Config Filesystem
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ProviderFsInfoParams {
    pub providers: Vec<ProviderFsInfo>,
}

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
pub struct ModelFsInfoParams {
    pub models: Vec<ModelFsInfo>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum SubscriptionType {
    GitHubRepo,
    GitRepo,
    Website,
    Rss,
    LocalDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum SubscriptionStatus {
    NotSynced,
    Syncing,
    Synced,
    Error,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub enum DocumentStatus {
    Pending,
    Indexing,
    Indexed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CreateKnowledgeBaseRequest {
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub embedding_model: Option<EmbeddingModel>,
    #[serde(default)]
    #[ts(optional)]
    pub custom_embedding_endpoint: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CreateKnowledgeBaseResponse {
    #[ts(type = "string")]
    pub knowledge_base_id: uuid::Uuid,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AddDocumentRequest {
    #[ts(type = "string")]
    pub knowledge_base_id: uuid::Uuid,
    pub content: String,
    #[serde(default)]
    #[ts(optional)]
    pub title: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub source_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct AddDocumentResponse {
    #[ts(type = "string")]
    pub document_id: uuid::Uuid,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct QueryKnowledgeBaseRequest {
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub knowledge_base_id: Option<uuid::Uuid>,
    pub query: String,
    #[serde(default)]
    #[ts(optional)]
    pub top_k: Option<usize>,
    #[serde(default)]
    #[ts(optional)]
    pub score_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct QueryResultChunk {
    #[ts(type = "string")]
    pub document_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub document_title: Option<String>,
    pub content: String,
    pub score: f64,
    #[serde(default)]
    #[ts(optional)]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct QueryKnowledgeBaseResponse {
    pub results: Vec<QueryResultChunk>,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CreateSubscriptionRequest {
    #[ts(type = "string")]
    pub knowledge_base_id: uuid::Uuid,
    pub subscription_type: SubscriptionType,
    pub url: String,
    #[serde(default)]
    #[ts(optional)]
    pub sync_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub sync_interval_hours: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CreateSubscriptionResponse {
    #[ts(type = "string")]
    pub subscription_id: uuid::Uuid,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SyncSubscriptionRequest {
    #[ts(type = "string")]
    pub subscription_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub force_full_sync: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SyncSubscriptionResponse {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub added_documents: Option<usize>,
    #[serde(default)]
    #[ts(optional)]
    pub updated_documents: Option<usize>,
    #[serde(default)]
    #[ts(optional)]
    pub deleted_documents: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DeleteSubscriptionResponse {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct KnowledgeBaseFilters {
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<KnowledgeBaseStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DeleteKnowledgeBaseResponse {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
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

// ═══════════════════════════════════════════════════════════════
// WebRTC Screen Signaling
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebrtcOfferParams {
    #[ts(type = "string")]
    pub session_id: uuid::Uuid,
    pub node_id: String,
    pub sdp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebrtcAnswerParams {
    #[ts(type = "string")]
    pub session_id: uuid::Uuid,
    pub node_id: String,
    pub sdp: String,
    #[serde(default)]
    pub displays: Vec<DisplayInfoParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct DisplayInfoParams {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebrtcIceCandidateParams {
    #[ts(type = "string")]
    pub session_id: uuid::Uuid,
    pub node_id: String,
    #[ts(type = "unknown")]
    pub candidate: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WebrtcIceServersParams {
    pub ice_servers: Vec<IceServerParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct IceServerParams {
    pub urls: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub username: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub credential: Option<String>,
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
pub struct SystemMessageParams {
    pub notification: SystemNotification,
    pub timestamp: String,
}

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
    pub tasks: Vec<YoloTierTaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloFullConfig {
    pub tiers: Vec<YoloTierConfig>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum YoloTaskTier {
    Realtime,
    Periodic,
    Daily,
    Strategic,
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

// ═══════════════════════════════════════════════════════════════
// Arbiter
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ArbiterStatusData {
    pub locked_down: bool,
    pub active_policies: u32,
    #[serde(default)]
    pub violations: Vec<String>,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ArbiterStatusResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<ArbiterStatusData>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ArbiterLockdownResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ArbiterRestoreResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// VM Snapshot
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CosmosContainerInfo {
    pub container_id: String,
    pub container_name: String,
    pub agent_type: String,
    #[ts(type = "string")]
    pub instance_uuid: uuid::Uuid,
    pub socket_path: String,
    pub image: String,
    #[serde(default)]
    #[ts(optional)]
    pub branch: Option<String>,
    #[serde(default)]
    #[ts(type = "string")]
    #[ts(optional)]
    pub badge: Option<AgentBadge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CosmosOperationLogEntry {
    pub timestamp: String,
    pub tool_name: String,
    #[serde(default)]
    pub params_preview: String,
    pub success: bool,
    #[serde(default)]
    pub result_preview: String,
    #[serde(default)]
    pub error: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct VmSnapshotParams {
    pub agent_id: String,
    #[serde(default)]
    #[ts(type = "unknown")]
    pub globals: serde_json::Value,
    #[serde(default)]
    #[ts(optional)]
    pub container_info: Option<CosmosContainerInfo>,
    #[serde(default)]
    pub tool_list: Vec<String>,
    #[serde(default)]
    pub op_log: Vec<CosmosOperationLogEntry>,
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
