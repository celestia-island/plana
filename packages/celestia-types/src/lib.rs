//! Celestia platform domain profile for the PLANA protocol.
//!
//! `plana-celestia-types` is the celestia-island platform's domain profile:
//! the agent, task, panel, industrial and tool domain messages, plus the
//! scepter-flavored handshake and client-capability payloads. Every type is
//! `Serialize`/`Deserialize` with JSON Schema and TypeScript bindings
//! generation support.
//!
//! The generic protocol core (base messages, handshake primitives,
//! health/network descriptors, RBAC, region policy, identity) lives in the
//! separate `plana-protocol-core` crate, and the JSON-RPC 2.0 envelope and
//! machinery live in `plana-jsonrpc`. The `plana` umbrella crate re-exports
//! all three.
//!
//! A type belongs here only when it is defined in this crate as the
//! canonical source of truth and consumed on both sides of a wire protocol.
//! Anything else stays out.
//!
//! > **Migration note:** the `tracing-helpers` feature that previously lived
//! > in this crate has moved to `plana-protocol-core` (`plana_protocol_core::tracing_helpers`,
//! > feature `tracing-helpers`), forwarded by the `plana` umbrella as
//! > `plana::tracing_helpers`. Consumers enabling `tracing-helpers` on this
//! > crate's old versions should enable it on `plana` or `plana-protocol-core`.

// ── Module tree ─────────────────────────────────────────────
// Foundational shared enums are defined directly in this file (below). The
// other type groups live under a small set of domain folders:
//   ws/       — SyncMessage variant params (agent / ui / services sub-groups)
//   tools/    — per-tool I/O structs
// and a few single-file modules at the root (enums, engine, http, model,
// external_mcp, malkuth, mdd). The glob re-exports at the bottom keep every
// type reachable at the crate root (`plana_celestia_types::TypeName`).
pub mod engine;
pub mod enums;
pub mod external_mcp;
pub mod http;
pub mod malkuth;
pub mod mdd;
pub mod model;
pub mod protocol;
pub mod tools;
pub mod ws;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Default report type when none is specified.
pub const DEFAULT_REPORT_TYPE: &str = "general";

/// Generic health/network descriptors, re-exported here so the `celestia`
/// module surface keeps matching the umbrella root (`plana::http::*`).
///
/// NOTE: this explicit re-export also pins `HealthResponse` at the crate
/// root to the generic HTTP type, shadowing the unrelated supervision
/// `malkuth::HealthResponse` (reachable at
/// `plana::celestia::malkuth::HealthResponse`).
pub use plana_protocol_core::http::{BackendKind, HealthResponse, NetworkInfo, ServiceStatus};

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
    #[serde(alias = "EpieiKeia")]
    Epieikeia,
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
            Agent::Epieikeia,
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
    ToolCall {
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
    ToolResult {
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
    #[error("no model with the required capability is available in the tier")]
    ModelNoCapableModel,
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
                | Self::ModelNoCapableModel
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
// the folder modules (`ws/{agent,ui,services}/`) but are re-exported at the
// crate root so the public surface is unchanged — `TuiAgentInfo`,
// `ConnectHandshakeParams`, etc. all still resolve. Generic protocol-core
// types are NOT re-exported here — except `base_messages`, kept for
// root-surface parity with the pre-split crate; everything else comes from
// `plana-protocol-core` and is re-exported once by the `plana` umbrella crate.
// ═══════════════════════════════════════════════════════════════

// The client-capability handshake payload types (scepter-flavored) stay at
// the crate root. The generic handshake primitives (HandshakeAckParams,
// PingParams, HANDSHAKE_VERSION) live in plana-protocol-core.
pub use protocol::base_messages::*;
pub use protocol::handshake::*;
// The platform-specific JSON-RPC error codes stay reachable at the crate
// root (`plana_celestia_types::jsonrpc::error_codes`) as a re-export of the
// single canonical definition in plana-jsonrpc (`plana_jsonrpc::types`),
// which the umbrella re-exports as `plana::jsonrpc`.
pub use protocol::jsonrpc;

// enums/ — foundational shared enums (Agent, WorkStatus, etc.)
pub use enums::*;

// malkuth/ — supervision protocol types (restart authorization gate)
pub use malkuth::*;

// model/ — unified model management (re-export key types to crate root
// for ergonomic access: `arona::ModelCapability` not `arona::model::…`)
pub use model::{GenerationTier, HardwareRequirements, ModelCapability};

// mdd/ — model deployment descriptor schema v1
pub use mdd::*;

// ws/ — SyncMessage variant params (types at crate root)
pub use ws::agent::{agent_lifecycle::*, layer2::*, state_sync::*, tasks::*, yolo::*};
pub use ws::services::{auth::*, industrial::*, knowledge_base::*, llm_provider::*};
pub use ws::ui::{
    bridge_network::*, file_browsing::*, logs::*, noa::*, realtime::*, system_ui::*, views::*,
    workspace::*,
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
    fn stream_segment_tool_call_round_trip() {
        let seg = StreamSegment::ToolCall {
            tool_name: "kalos.file_read".into(),
            call_id: "call-1".into(),
            params: json!({"path": "/etc/hosts"}),
            agent_type: None,
            message_id: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        let back: StreamSegment = serde_json::from_value(v).unwrap();
        match back {
            StreamSegment::ToolCall {
                tool_name, call_id, ..
            } => {
                assert_eq!(tool_name, "kalos.file_read");
                assert_eq!(call_id, "call-1");
            }
            other => panic!("expected ToolCall, got {other:?}"),
        }
    }

    #[test]
    fn stream_segment_tool_result_round_trip() {
        let seg = StreamSegment::ToolResult {
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
            StreamSegment::ToolResult {
                success,
                duration_ms,
                ..
            } => {
                assert!(success);
                assert_eq!(duration_ms, Some(42));
            }
            other => panic!("expected ToolResult, got {other:?}"),
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
    fn default_report_type_is_general() {
        assert_eq!(DEFAULT_REPORT_TYPE, "general");
    }

    // ── CEP engine protocol ────────────────────────────────────────

    #[test]
    fn engine_protocol_version_is_three() {
        assert_eq!(engine::ENGINE_PROTOCOL_VERSION, 3);
    }

    #[test]
    fn engine_binary_max_frame_is_256k() {
        assert_eq!(engine::ENGINE_BINARY_MAX_FRAME_BYTES, 256 * 1024);
    }

    #[test]
    fn engine_binary_receive_timeout_is_sixty_seconds() {
        assert_eq!(engine::ENGINE_BINARY_RECEIVE_TIMEOUT_SECS, 60);
    }

    #[test]
    fn engine_handshake_round_trips_with_optional_fields() {
        let params = engine::EngineHandshakeParams {
            token: None,
            engine: engine::EngineIdentity {
                name: "llamacpp".into(),
                version: "b1234".into(),
                language: Some("cpp".into()),
                vendor: Some("ggml-org".into()),
            },
            capabilities: engine::EngineCapabilities {
                streaming: true,
                embeddings: false,
                max_context_length: 8192,
                hardware: vec![engine::EngineGpuInfo {
                    name: "RTX 5880".into(),
                    vram_gb: 47,
                }],
                input_modalities: vec![
                    engine::EngineModality::Text,
                    engine::EngineModality::Audio,
                    engine::EngineModality::Sensor,
                ],
                output_modalities: vec![engine::EngineModality::Text],
                content_types: vec!["audio/wav".into(), "application/json".into()],
                methods: vec!["audio.transcribe".into(), "signal.classify".into()],
            },
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["engine"]["language"], "cpp");
        assert!(json.get("token").is_none());
        let back: engine::EngineHandshakeParams = serde_json::from_value(json).unwrap();
        assert_eq!(back.capabilities.max_context_length, 8192);
        assert_eq!(back.capabilities.hardware[0].vram_gb, 47);
    }

    #[test]
    fn engine_capabilities_defaults_are_lenient() {
        let parsed: engine::EngineCapabilities =
            serde_json::from_str(r#"{"streaming":false}"#).unwrap();
        assert!(!parsed.streaming);
        assert!(!parsed.embeddings);
        assert_eq!(parsed.max_context_length, 128_000);
        assert!(parsed.hardware.is_empty());
        // v2: modality/capability fields default to empty (text-only).
        assert!(parsed.input_modalities.is_empty());
        assert!(parsed.output_modalities.is_empty());
        assert!(parsed.content_types.is_empty());
        assert!(parsed.methods.is_empty());
    }

    #[test]
    fn engine_chat_chunk_serializes_stream_shape() {
        let chunk = engine::EngineChatChunk {
            stream_id: "s1".into(),
            token: "hello".into(),
            is_complete: false,
            usage: None,
        };
        let json = serde_json::to_value(&chunk).unwrap();
        assert_eq!(json["stream_id"], "s1");
        assert_eq!(json["token"], "hello");
        assert_eq!(json["is_complete"], false);
    }

    // ── CEP v2: multimodal content + generic invocation ────────────

    #[test]
    fn engine_message_carries_multimodal_parts() {
        let msg = engine::EngineMessage {
            role: "user".into(),
            content: vec![
                engine::EngineContentPart::text("transcribe this"),
                engine::EngineContentPart::base64("audio/wav", "UklGRg=="),
                engine::EngineContentPart::json(
                    "application/json",
                    json!({
                        "sensor": "accel-x", "samples": [1.0, -2.0, 3.5],
                    }),
                ),
            ],
        };
        let json = serde_json::to_value(&msg).unwrap();
        let parts = json["content"].as_array().unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0]["mime"], "text/plain");
        assert_eq!(parts[1]["encoding"], "base64");
        assert_eq!(parts[2]["data"]["sensor"], "accel-x");
        let back: engine::EngineMessage = serde_json::from_value(json).unwrap();
        assert_eq!(back.content[1].mime, "audio/wav");
    }

    #[test]
    fn engine_binary_frame_part_needs_no_data() {
        let part = engine::EngineContentPart::binary_frame("audio/wav");
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["encoding"], "binary-frame");
        assert!(json.get("data").is_none());
        let back: engine::EngineContentPart = serde_json::from_value(json).unwrap();
        assert_eq!(back.mime, "audio/wav");
    }

    #[test]
    fn engine_content_part_carries_position_marker() {
        let tail = engine::EngineContentPart::base64("image/jpeg", "AQID")
            .with_position(engine::EngineContentPosition::Tail);
        let json = serde_json::to_value(&tail).unwrap();
        assert_eq!(json["position"]["kind"], "tail");

        let frame = engine::EngineContentPart::base64("image/jpeg", "AQID")
            .with_position(engine::EngineContentPosition::Frame { index: 1 });
        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["position"]["kind"], "frame");
        assert_eq!(json["position"]["index"], 1);

        let back: engine::EngineContentPart = serde_json::from_value(json).unwrap();
        assert_eq!(
            back.position,
            Some(engine::EngineContentPosition::Frame { index: 1 })
        );
    }

    #[test]
    fn engine_handshake_result_optional_capabilities_round_trips() {
        let caps = engine::EngineCapabilities {
            streaming: true,
            embeddings: true,
            max_context_length: 32_000,
            hardware: vec![],
            input_modalities: vec![engine::EngineModality::Audio],
            output_modalities: vec![engine::EngineModality::Text],
            content_types: vec!["audio/wav".into()],
            methods: vec!["audio.transcribe".into()],
        };
        let result = engine::EngineHandshakeResult {
            ok: true,
            error: None,
            protocol_version: 2,
            capabilities: Some(caps),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["capabilities"]["input_modalities"], json!(["Audio"]));
        assert_eq!(json["protocol_version"], 2);
        let back: engine::EngineHandshakeResult = serde_json::from_value(json).unwrap();
        assert_eq!(
            back.capabilities.unwrap().methods,
            vec!["audio.transcribe".to_string()]
        );
    }

    #[test]
    fn engine_handshake_result_without_capabilities_parses() {
        // Server that doesn't declare capabilities (old engines).
        let back: engine::EngineHandshakeResult =
            serde_json::from_str(r#"{"ok":true,"protocol_version":1}"#).unwrap();
        assert!(back.capabilities.is_none());
    }

    #[test]
    fn engine_invoke_round_trips_free_form_payload() {
        let params = engine::EngineInvokeParams {
            method: "signal.filter".into(),
            params: json!({
                "filter": "lowpass", "cutoff_hz": 800,
                "channel": 3,
            }),
            messages: Some(vec![engine::EngineMessage::text(
                "user",
                "keep the pump vibration band",
            )]),
            stream_id: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["method"], "signal.filter");
        assert_eq!(json["params"]["cutoff_hz"], 800);
        let back: engine::EngineInvokeParams = serde_json::from_value(json).unwrap();
        assert_eq!(
            back.messages.as_ref().unwrap()[0].content[0].mime,
            "text/plain"
        );
    }

    #[test]
    fn engine_stream_chunk_describes_audio_block() {
        let chunk = engine::EngineStreamChunk {
            stream_id: "s2".into(),
            mime: "audio/wav".into(),
            encoding: "base64".into(),
            data: Some(json!("UklGRgAAAA==")),
            shape: Some(vec![1, 16000]),
            is_complete: true,
            usage: None,
        };
        let json = serde_json::to_value(&chunk).unwrap();
        assert_eq!(json["mime"], "audio/wav");
        assert_eq!(json["shape"], json!([1, 16000]));
        assert_eq!(json["is_complete"], true);
    }

    #[test]
    fn engine_binary_start_announce_round_trips() {
        let params = engine::EngineBinaryStartParams {
            transfer_id: "t-42".into(),
            mime: "audio/wav".into(),
            total_bytes: 1_000_000,
            chunk_count: 4,
            checksum: Some("deadbeef".into()),
            stream_id: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["mime"], "audio/wav");
        assert_eq!(json["chunk_count"], 4);
        assert_eq!(json["checksum"], "deadbeef");
        let back: engine::EngineBinaryStartParams = serde_json::from_value(json).unwrap();
        assert_eq!(back.total_bytes, 1_000_000);
    }

    #[test]
    fn engine_binary_end_validates_receipt() {
        let end = engine::EngineBinaryEndParams {
            transfer_id: "t-42".into(),
            bytes_received: 1_000_000,
            checksum_ok: Some(true),
        };
        let json = serde_json::to_value(&end).unwrap();
        assert_eq!(json["bytes_received"], 1_000_000);
        assert_eq!(json["checksum_ok"], true);
        let abort = engine::EngineBinaryAbortParams {
            transfer_id: "t-42".into(),
            reason: "client cancelled".into(),
        };
        let j2 = serde_json::to_value(&abort).unwrap();
        assert_eq!(j2["reason"], "client cancelled");
        // checksum_ok is optional on the wire.
        let bare: engine::EngineBinaryEndParams =
            serde_json::from_str(r#"{"transfer_id":"t","bytes_received":0}"#).unwrap();
        assert!(bare.checksum_ok.is_none());
    }

    #[test]
    fn engine_binary_chunk_size_stays_under_frame_bound() {
        // A 1 MiB payload with the 256 KiB frame bound needs >= 4 frames.
        let payload: usize = 1024 * 1024;
        let frames = payload.div_ceil(engine::ENGINE_BINARY_MAX_FRAME_BYTES);
        assert_eq!(frames, 4);
    }

    #[test]
    fn engine_modality_serde_uses_pascal_case() {
        assert_eq!(
            serde_json::to_string(&engine::EngineModality::Sensor).unwrap(),
            r#""Sensor""#
        );
        assert_eq!(
            serde_json::to_string(&engine::EngineModality::Tensor).unwrap(),
            r#""Tensor""#
        );
    }

    // ── AgentErrorCode ─────────────────────────────────────────────

    #[test]
    fn agent_error_code_no_capable_model_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&AgentErrorCode::ModelNoCapableModel).unwrap(),
            r#""model_no_capable_model""#
        );
        let back: AgentErrorCode = serde_json::from_str(r#""model_no_capable_model""#).unwrap();
        assert_eq!(back, AgentErrorCode::ModelNoCapableModel);
    }

    #[test]
    fn agent_error_code_no_capable_model_displays_capability_and_tier() {
        let msg = AgentErrorCode::ModelNoCapableModel.to_string();
        assert_eq!(
            msg,
            "no model with the required capability is available in the tier"
        );
        assert!(msg.contains("capability"));
        assert!(msg.contains("tier"));
    }

    #[test]
    fn agent_error_code_no_capable_model_is_model_selection_error() {
        assert!(AgentErrorCode::ModelNoCapableModel.is_model_selection_error());
        assert!(!AgentErrorCode::ModelNoCapableModel.is_llm_error());
    }
}
