//! Shared protocol types for the entelecheia multi-agent platform.
//!
//! Every type in this crate MUST be:
//! - Defined in entelecheia (canonical source of truth)
//! - Consumed by shittim-chest (core, mock_scepter, or webui)
//!
//! If a type is not paired on both sides, it does not belong here.

// ── Module tree ─────────────────────────────────────────────
// Foundational shared enums are defined directly in this file (below). The
// other type groups live under a small set of domain folders:
//   protocol/ — JSON-RPC envelope, base messages, handshake (WS transport)
//   ws/       — SyncMessage variant params (agent / ui / services sub-groups)
//   mcp/      — per-agent MCP tool I/O structs
// and a few single-file modules at the root (enums, http, model,
// external_mcp). The glob re-exports at the bottom keep every type reachable
// at the crate root (`arona::TypeName`).
pub mod enums;
pub mod external_mcp;
pub mod http;
pub mod identity;
pub mod malkuth;
pub mod mcp;
pub mod model;
pub mod protocol;
pub mod rbac;
pub mod ws;

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
}

impl Agent {
    pub fn all() -> &'static [Agent] {
        &[
            Agent::HapLotes,
            Agent::SkoPeo,
            Agent::HubRis,
            Agent::KaLos,
            Agent::NeiKos,
            Agent::SkeMma,
            Agent::ApoRia,
            Agent::EleOs,
            Agent::EpieiKeia,
            Agent::OreXis,
            Agent::PhiLia,
            Agent::PoleMos,
            Agent::WebAutomation,
        ]
    }
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

impl ReportType {
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::Error
                | Self::ChainMaxDepth
                | Self::ChainCycle
                | Self::SkillFailed
                | Self::SkillEmptyOutput
                | Self::SkillMissingReport
        )
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Reply
                | Self::SkillTerminal
                | Self::Error
                | Self::System
                | Self::NextActionFallback
        )
    }
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

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
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

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub struct LlmStream {
    #[serde(default)]
    pub segments: Vec<StreamSegment>,
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub struct RouteInfo {
    pub direction: String,
    pub target: String,
    #[serde(default)]
    #[ts(optional)]
    pub target_token: Option<String>,
}

#[derive(
    JsonSchema, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS, thiserror::Error,
)]
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

impl AgentErrorCode {
    pub fn is_llm_error(&self) -> bool {
        matches!(
            self,
            Self::LlmCallFailed
                | Self::LlmEmptyResponse
                | Self::LlmRateLimited
                | Self::LlmAuthFailed
                | Self::LlmTimeout
        )
    }

    pub fn is_cosmos_error(&self) -> bool {
        matches!(
            self,
            Self::CosmosNoConnection | Self::CosmosToolFailed | Self::CosmosLocalUnavailable
        )
    }

    pub fn is_chain_error(&self) -> bool {
        matches!(
            self,
            Self::ChainMaxDepth | Self::ChainCycle | Self::ChainFailed
        )
    }

    pub fn is_skill_error(&self) -> bool {
        matches!(
            self,
            Self::SkillFailed | Self::SkillEmptyOutput | Self::SkillMissingReport
        )
    }

    pub fn is_model_selection_error(&self) -> bool {
        matches!(
            self,
            Self::ModelNoProviders
                | Self::ModelNoModels
                | Self::ModelTierMismatch
                | Self::ModelAllExcluded
                | Self::ModelEnvIncomplete
                | Self::ModelSelectionRetryExhausted
        )
    }
}

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
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

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum PeriodType {
    Hour5,
    Day7,
    Month1,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum KnowledgeBaseStatus {
    #[default]
    Uninitialized,
    Indexing,
    Ready,
    Error,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum EmbeddingModel {
    OpenAiSmall,
    OpenAiLarge,
    OpenAiAda,
    Custom,
}

#[derive(JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
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
// The foundational enums above stay defined here. The domain structs live in
// the folder modules (`protocol/`, `ws/{agent,ui,services}/`) but are
// re-exported at the crate root so the public surface is unchanged —
// `arona::TuiAgentInfo`, `arona::HandshakeAckParams`, etc. all still resolve.
//
// `jsonrpc` is re-exported *as a module* (not globbed) so its deep path
// `arona::jsonrpc::*` keeps working for the many consumers that use it.
// ═══════════════════════════════════════════════════════════════

// protocol/ — transport core
pub use protocol::base_messages::*;
pub use protocol::handshake::*;
pub use protocol::jsonrpc;

// enums/ — foundational shared enums (ConnectionType, Agent, WorkStatus, etc.)
pub use enums::*;

// malkuth/ — supervision protocol types (restart authorization gate)
pub use malkuth::*;

// model/ — unified model management (re-export key types to crate root
// for ergonomic access: `arona::ModelCapability` not `arona::model::…`)
pub use model::{GenerationTier, HardwareRequirements, ModelCapability};

// ws/ — SyncMessage variant params (types at crate root)
pub use ws::agent::{agent_lifecycle::*, layer2::*, state_sync::*, tasks::*, yolo::*};
pub use ws::services::{auth::*, industrial::*, knowledge_base::*, llm_provider::*};
pub use ws::ui::{
    bridge_network::*, file_browsing::*, logs::*, noa::*, system_ui::*, views::*, workspace::*,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Agent enum ─────────────────────────────────────────────────

    #[test]
    fn agent_all_returns_thirteen_unique_variants() {
        let all = Agent::all();
        assert_eq!(
            all.len(),
            13,
            "Agent::all() must return exactly 13 variants"
        );
        // Verify uniqueness.
        let mut seen = std::collections::HashSet::new();
        for a in all {
            let s = format!("{a:?}");
            assert!(seen.insert(s.clone()), "duplicate agent variant: {s}");
        }
    }

    #[test]
    fn agent_serde_round_trip_each_variant() {
        for agent in Agent::all() {
            let s = serde_json::to_string(agent).unwrap();
            let back: Agent = serde_json::from_str(&s).unwrap();
            assert_eq!(back, *agent);
        }
    }

    #[test]
    fn agent_serializes_as_pascal_case() {
        // No #[serde(rename_all)] on Agent → PascalCase variant names.
        assert_eq!(
            serde_json::to_string(&Agent::HapLotes).unwrap(),
            r#""HapLotes""#
        );
        assert_eq!(
            serde_json::to_string(&Agent::WebAutomation).unwrap(),
            r#""WebAutomation""#
        );
    }

    // ── AgentBadge newtype ─────────────────────────────────────────

    #[test]
    fn agent_badge_round_trip() {
        let badge = AgentBadge("haplotes-01".into());
        let v = serde_json::to_value(&badge).unwrap();
        assert_eq!(v, "haplotes-01");
        let back: AgentBadge = serde_json::from_value(v).unwrap();
        assert_eq!(back.0, "haplotes-01");
    }

    // ── AgentStatus ────────────────────────────────────────────────

    #[test]
    fn agent_status_round_trip_all_variants() {
        for s in [
            AgentStatus::Initializing,
            AgentStatus::Online,
            AgentStatus::Busy,
            AgentStatus::Offline,
            AgentStatus::Error,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    // ── RequestState default ───────────────────────────────────────

    #[test]
    fn request_state_default_is_idle() {
        assert_eq!(RequestState::default(), RequestState::Idle);
    }

    #[test]
    fn completion_outcome_default_is_none() {
        assert_eq!(CompletionOutcome::default(), CompletionOutcome::None);
    }

    // ── ReportType classification ──────────────────────────────────

    #[test]
    fn report_type_is_query() {
        assert!(ReportType::Query.is_query());
        assert!(!ReportType::Reply.is_query());
        assert!(!ReportType::Error.is_query());
    }

    #[test]
    fn report_type_is_error_covers_all_error_variants() {
        let error_variants = [
            ReportType::Error,
            ReportType::ChainMaxDepth,
            ReportType::ChainCycle,
            ReportType::SkillFailed,
            ReportType::SkillEmptyOutput,
            ReportType::SkillMissingReport,
        ];
        for v in &error_variants {
            assert!(v.is_error(), "{v:?} should be classified as error");
        }
        // Non-error variants.
        assert!(!ReportType::Query.is_error());
        assert!(!ReportType::Reply.is_error());
        assert!(!ReportType::Human.is_error());
        assert!(!ReportType::System.is_error());
        assert!(!ReportType::Pending.is_error());
    }

    #[test]
    fn report_type_is_pending() {
        assert!(ReportType::Pending.is_pending());
        assert!(!ReportType::Reply.is_pending());
    }

    #[test]
    fn report_type_is_terminal() {
        let terminal = [
            ReportType::Reply,
            ReportType::SkillTerminal,
            ReportType::Error,
            ReportType::System,
            ReportType::NextActionFallback,
        ];
        for v in &terminal {
            assert!(v.is_terminal(), "{v:?} should be terminal");
        }
        // Non-terminal.
        assert!(!ReportType::Query.is_terminal());
        assert!(!ReportType::Pending.is_terminal());
        assert!(!ReportType::SkillStep.is_terminal());
    }

    #[test]
    fn report_type_serde_uses_snake_case() {
        // ReportType has #[serde(rename_all = "snake_case")].
        assert_eq!(
            serde_json::to_string(&ReportType::SkillTerminal).unwrap(),
            r#""skill_terminal""#
        );
        assert_eq!(
            serde_json::to_string(&ReportType::NextActionFallback).unwrap(),
            r#""next_action_fallback""#
        );
    }

    // ── AgentErrorCode classification ──────────────────────────────

    #[test]
    fn agent_error_code_is_llm_error() {
        let llm_errors = [
            AgentErrorCode::LlmCallFailed,
            AgentErrorCode::LlmEmptyResponse,
            AgentErrorCode::LlmRateLimited,
            AgentErrorCode::LlmAuthFailed,
            AgentErrorCode::LlmTimeout,
        ];
        for e in &llm_errors {
            assert!(e.is_llm_error(), "{e:?} should be LLM error");
        }
        assert!(!AgentErrorCode::CosmosNoConnection.is_llm_error());
    }

    #[test]
    fn agent_error_code_is_cosmos_error() {
        let cosmos = [
            AgentErrorCode::CosmosNoConnection,
            AgentErrorCode::CosmosToolFailed,
            AgentErrorCode::CosmosLocalUnavailable,
        ];
        for e in &cosmos {
            assert!(e.is_cosmos_error(), "{e:?} should be cosmos error");
        }
        assert!(!AgentErrorCode::LlmTimeout.is_cosmos_error());
    }

    #[test]
    fn agent_error_code_is_chain_error() {
        assert!(AgentErrorCode::ChainMaxDepth.is_chain_error());
        assert!(AgentErrorCode::ChainCycle.is_chain_error());
        assert!(AgentErrorCode::ChainFailed.is_chain_error());
        assert!(!AgentErrorCode::SkillFailed.is_chain_error());
    }

    #[test]
    fn agent_error_code_is_skill_error() {
        assert!(AgentErrorCode::SkillFailed.is_skill_error());
        assert!(AgentErrorCode::SkillEmptyOutput.is_skill_error());
        assert!(AgentErrorCode::SkillMissingReport.is_skill_error());
        assert!(!AgentErrorCode::ChainFailed.is_skill_error());
    }

    #[test]
    fn agent_error_code_is_model_selection_error() {
        let model_errors = [
            AgentErrorCode::ModelNoProviders,
            AgentErrorCode::ModelNoModels,
            AgentErrorCode::ModelTierMismatch,
            AgentErrorCode::ModelAllExcluded,
            AgentErrorCode::ModelEnvIncomplete,
            AgentErrorCode::ModelSelectionRetryExhausted,
        ];
        for e in &model_errors {
            assert!(
                e.is_model_selection_error(),
                "{e:?} should be model selection error"
            );
        }
        assert!(!AgentErrorCode::LlmTimeout.is_model_selection_error());
    }

    #[test]
    fn agent_error_code_categories_are_mutually_exclusive() {
        // Every variant belongs to at most one category.
        for code in [
            AgentErrorCode::ModelNoProviders,
            AgentErrorCode::ModelNoModels,
            AgentErrorCode::ModelTierMismatch,
            AgentErrorCode::ModelAllExcluded,
            AgentErrorCode::ModelEnvIncomplete,
            AgentErrorCode::ModelSelectionRetryExhausted,
            AgentErrorCode::LlmCallFailed,
            AgentErrorCode::LlmEmptyResponse,
            AgentErrorCode::LlmRateLimited,
            AgentErrorCode::LlmAuthFailed,
            AgentErrorCode::LlmTimeout,
            AgentErrorCode::CosmosNoConnection,
            AgentErrorCode::CosmosToolFailed,
            AgentErrorCode::CosmosLocalUnavailable,
            AgentErrorCode::ChainMaxDepth,
            AgentErrorCode::ChainCycle,
            AgentErrorCode::ChainFailed,
            AgentErrorCode::SkillFailed,
            AgentErrorCode::SkillEmptyOutput,
            AgentErrorCode::SkillMissingReport,
        ] {
            let count = [
                code.is_llm_error(),
                code.is_cosmos_error(),
                code.is_chain_error(),
                code.is_skill_error(),
                code.is_model_selection_error(),
            ]
            .iter()
            .filter(|&&b| b)
            .count();
            assert_eq!(
                count, 1,
                "{code:?} belongs to {count} categories, expected 1"
            );
        }
    }

    #[test]
    fn agent_error_code_thiserror_display() {
        // Each variant has a non-empty error message via thiserror.
        assert_eq!(AgentErrorCode::LlmCallFailed.to_string(), "LLM call failed");
        assert_eq!(
            AgentErrorCode::ModelNoProviders.to_string(),
            "model has no providers"
        );
    }

    // ── StreamSegment variants ─────────────────────────────────────

    #[test]
    fn stream_segment_text_round_trip() {
        let seg = StreamSegment::Text {
            text: "hello".into(),
            message_id: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        // Externally tagged enum: {"Text": {"text": "hello", "message_id": null}}
        assert_eq!(v["Text"]["text"], "hello");
        let back: StreamSegment = serde_json::from_value(v).unwrap();
        match back {
            StreamSegment::Text { text, .. } => assert_eq!(text, "hello"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn stream_segment_mcp_call_round_trip() {
        let seg = StreamSegment::McpCall {
            tool_name: "kalos.file_read".into(),
            call_id: "call-1".into(),
            params: json!({"path": "/etc/hosts"}),
            agent_type: None,
            message_id: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        let back: StreamSegment = serde_json::from_value(v).unwrap();
        match back {
            StreamSegment::McpCall {
                tool_name, call_id, ..
            } => {
                assert_eq!(tool_name, "kalos.file_read");
                assert_eq!(call_id, "call-1");
            }
            other => panic!("expected McpCall, got {other:?}"),
        }
    }

    #[test]
    fn stream_segment_mcp_result_round_trip() {
        let seg = StreamSegment::McpResult {
            tool_name: "kalos.file_read".into(),
            call_id: "call-1".into(),
            success: true,
            data: json!({"content": "file data"}),
            duration_ms: Some(42),
            agent_type: Some("KaLos".into()),
            message_id: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        let back: StreamSegment = serde_json::from_value(v).unwrap();
        match back {
            StreamSegment::McpResult {
                success,
                duration_ms,
                ..
            } => {
                assert!(success);
                assert_eq!(duration_ms, Some(42));
            }
            other => panic!("expected McpResult, got {other:?}"),
        }
    }

    // ── StructuredAgentError ───────────────────────────────────────

    #[test]
    fn structured_agent_error_round_trip() {
        let mut ctx = std::collections::HashMap::new();
        ctx.insert("agent".into(), "haplotes".into());
        let e = StructuredAgentError {
            code: AgentErrorCode::LlmTimeout,
            detail: Some("provider timed out after 30s".into()),
            context: ctx,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "llm_timeout");
        assert_eq!(v["detail"], "provider timed out after 30s");
        assert_eq!(v["context"]["agent"], "haplotes");
        let back: StructuredAgentError = serde_json::from_value(v).unwrap();
        assert_eq!(back.code, AgentErrorCode::LlmTimeout);
    }

    #[test]
    fn structured_agent_error_minimal() {
        let e = StructuredAgentError {
            code: AgentErrorCode::ChainCycle,
            detail: None,
            context: std::collections::HashMap::new(),
        };
        let v = serde_json::to_value(&e).unwrap();
        // detail is #[ts(optional)] without skip → null.
        assert_eq!(v["detail"], serde_json::Value::Null);
    }

    // ── ContainerStatus ────────────────────────────────────────────

    #[test]
    fn container_status_round_trip_all_variants() {
        for s in [
            ContainerStatus::Created,
            ContainerStatus::Running,
            ContainerStatus::Paused,
            ContainerStatus::Restarting,
            ContainerStatus::Removing,
            ContainerStatus::Exited,
            ContainerStatus::Dead,
            ContainerStatus::Unknown,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: ContainerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    // ── KnowledgeBaseStatus default ────────────────────────────────

    #[test]
    fn knowledge_base_status_default_is_uninitialized() {
        assert_eq!(
            KnowledgeBaseStatus::default(),
            KnowledgeBaseStatus::Uninitialized
        );
    }

    // ── YoloTaskTier serde ─────────────────────────────────────────

    #[test]
    fn yolo_task_tier_serde_snake_case() {
        assert_eq!(
            serde_json::to_string(&YoloTaskTier::Realtime).unwrap(),
            r#""realtime""#
        );
        assert_eq!(
            serde_json::to_string(&YoloTaskTier::Strategic).unwrap(),
            r#""strategic""#
        );
    }

    // ── ReportSelection / StreamChunkKind defaults ────────────────

    #[test]
    fn report_selection_default_is_single() {
        assert_eq!(ReportSelection::default(), ReportSelection::Single);
    }

    #[test]
    fn stream_chunk_kind_default_is_text() {
        assert_eq!(StreamChunkKind::default(), StreamChunkKind::Text);
    }

    // ── Constants ──────────────────────────────────────────────────

    #[test]
    fn protocol_version_is_one_point_zero() {
        assert_eq!(PROTOCOL_VERSION, "1.0.0");
    }

    #[test]
    fn default_report_type_is_general() {
        assert_eq!(DEFAULT_REPORT_TYPE, "general");
    }
}
