//! Shared protocol types for the entelecheia multi-agent platform.
//!
//! Every type in this crate MUST be:
//! - Defined in entelecheia (canonical source of truth)
//! - Consumed by shittim-chest (core, mock_scepter, or webui)
//!
//! If a type is not paired on both sides, it does not belong here.

// ── Module tree ─────────────────────────────────────────────
// Foundational shared enums are defined directly in this file (below). Every
// other type group lives in its own domain module; the glob re-exports at the
// bottom keep all of them reachable at the crate root (`arona::TypeName`).
pub mod agent_lifecycle;
pub mod auth;
pub mod base_messages;
pub mod bridge_network;
pub mod enums;
pub mod external_mcp;
pub mod file_browsing;
pub mod handshake;
pub mod http;
pub mod industrial;
pub mod jsonrpc;
pub mod knowledge_base;
pub mod layer2;
pub mod llm_provider;
pub mod logs;
pub mod mcp;
pub mod model;
pub mod noa;
pub mod state_sync;
pub mod system_ui;
pub mod tasks;
pub mod views;
pub mod workspace;
pub mod yolo;

#[cfg(feature = "tracing-helpers")]
pub mod tracing_helpers;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Protocol version advertised by the platform.
pub const PROTOCOL_VERSION: &str = "1.0.0";
/// Default report type when none is specified.
pub const DEFAULT_REPORT_TYPE: &str = "general";

/// Serde default helper for `bool` fields that default to `true`.
pub(crate) fn default_true() -> bool {
    true
}

// ═══════════════════════════════════════════════════════════════
// Core enums
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub struct AgentBadge(pub String);

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum AgentStatus {
    Initializing,
    Online,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub enum RequestState {
    #[default]
    Idle,
    Waiting,
    Streaming,
    Retrying,
    WaitingTool,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum CompletionOutcome {
    #[default]
    None,
    Reported,
    Failed,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum ModelTier {
    Deep,
    Normal,
    Basic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub enum RetryReason {
    EmptyOutput,
    ReportNotCaptured,
    LlmError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
#[serde(rename_all = "snake_case")]
pub enum ReportSelection {
    #[default]
    Single,
    Multiple,
}

#[derive(JsonSchema, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum StreamChunkKind {
    #[default]
    Text,
    Thinking,
    DeepThinking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub struct LlmStream {
    #[serde(default)]
    pub segments: Vec<StreamSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub struct RouteInfo {
    pub direction: String,
    pub target: String,
    #[serde(default)]
    #[ts(optional)]
    pub target_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS, thiserror::Error)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub struct StructuredAgentError {
    pub code: AgentErrorCode,
    #[serde(default)]
    #[ts(optional)]
    pub detail: Option<String>,
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,
}

#[derive(JsonSchema, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
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
#[ts(export, export_to = "ws/core.ts")]
pub enum PeriodType {
    Hour5,
    Day7,
    Month1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum KnowledgeBaseStatus {
    #[default]
    Uninitialized,
    Indexing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum EmbeddingModel {
    OpenAiSmall,
    OpenAiLarge,
    OpenAiAda,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
#[serde(rename_all = "snake_case")]
pub enum YoloTaskTier {
    Realtime,
    Periodic,
    Daily,
    Strategic,
}

// ═══════════════════════════════════════════════════════════════
// Root re-exports
//
// The foundational enums above stay defined here. The WS-protocol domain
// structs live in dedicated modules (`handshake`, `agent_lifecycle`, …) but
// are re-exported at the crate root so the public surface is unchanged —
// `arona::TuiAgentInfo`, `arona::HandshakeAckParams`, etc. all still resolve.
// ═══════════════════════════════════════════════════════════════

pub use agent_lifecycle::*;
pub use auth::*;
pub use base_messages::*;
pub use bridge_network::*;
pub use file_browsing::*;
pub use handshake::*;
pub use industrial::*;
pub use knowledge_base::*;
pub use layer2::*;
pub use llm_provider::*;
pub use logs::*;
pub use noa::*;
pub use state_sync::*;
pub use system_ui::*;
pub use tasks::*;
pub use views::*;
pub use workspace::*;
pub use yolo::*;
