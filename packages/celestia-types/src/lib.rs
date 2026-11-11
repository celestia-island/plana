//! Celestia platform domain profile for the PLANA protocol.
//!
//! `plana-celestia-types` is the celestia-island platform's domain profile:
//! the agent, task, panel, industrial and tool domain messages, plus the
//! scepter-flavored handshake and client-capability payloads. Every type is
//! `Serialize`/`Deserialize` with JSON Schema and TypeScript bindings
//! generation support.
//!
//! The generic protocol core (base messages, handshake primitives,
//! health/network descriptors, RBAC, region policy, identity) and the JSON-RPC
//! 2.0 envelope now live in the `plana` protocol foundation
//! (`plana::protocol_core` and `plana::jsonrpc`); this crate depends on
//! `plana` and is not re-exported by it.
//!
//! A type belongs here only when it is defined in this crate as the
//! canonical source of truth and consumed on both sides of a wire protocol.
//! Anything else stays out.
//!
//! > **Migration note:** the `tracing-helpers` feature that previously lived
//! > in this crate moved to the foundation, where it enables the
//! > `plana::protocol_core::tracing_helpers` module (re-exported as
//! > `plana::tracing_helpers`). Consumers enabling `tracing-helpers` on this
//! > crate's old versions should enable it on `plana`; the former
//! > `plana-protocol-core` shim was removed after its consumers migrated.

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
pub mod error_paths;
pub mod evernight;
pub mod external_mcp;
pub mod gateway;
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

/// Generic health/network descriptors, re-exported here so this crate's root
/// surface carries the same generic HTTP types as the foundation
/// (`plana::http::*`).
///
/// NOTE: this explicit re-export also pins `HealthResponse` at the crate
/// root to the generic HTTP type, shadowing the unrelated supervision
/// `malkuth::WorkerHealthResponse` (reachable at
/// `plana_celestia_types::malkuth::WorkerHealthResponse`).
pub use plana::protocol_core::http::{
    BackendKind, HealthResponse, NetworkInfo, ServiceStatus, VersionReport, SERVICE_INFO_METHOD,
};

/// Serde default helper for `bool` fields that default to `true`.
pub(crate) fn default_true() -> bool {
    true
}

/// Serde `skip_serializing_if` helper for `bool` fields that default to
/// `false`: an absent flag and an explicit `false` mean the same thing, so
/// the flag stays off the wire unless it is set.
pub(crate) fn is_false(value: &bool) -> bool {
    !*value
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
    // Lowercase serde aliases mirror plana_core::ModelTier (PR #394): a
    // hand-edited config writing `deep`/`normal`/`basic` must not fail on
    // the wire twin while the core enum accepts it. Serialization keeps
    // the PascalCase variant names.
    #[serde(alias = "deep")]
    Deep,
    #[serde(alias = "normal")]
    Normal,
    #[serde(alias = "basic")]
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
    /// The prompt-change notice chunk (2026-10-22): unlike the
    /// caller-recorded SEGMENT arms the wire mirror of `StreamSegment`
    /// rejects, this kind exists precisely to stream live — the chunk
    /// payload is the notice's SOURCE, and consumers must render it as a
    /// provenance row, never as model prose.
    Notice,
}

/// The LIVE-STREAM wire mirror of `plana_text::StreamSegment` — the shape
/// the WS event plane serializes (see `ws/core.ts`).
///
/// ⚠️ Deliberately NOT identical to the runtime enum: the runtime one
/// gained `Input` (plana #420) for run-record capture — the composed
/// prompt side of a skill invocation — and `PromptNotice` (2026-10-22)
/// for the one-line mid-run prompt-change provenance. BOTH are
/// CALLER-recorded at record time and never stream live, so this wire
/// enum has neither arm.
/// (One smaller typing difference is intentional too: the runtime types
/// `call_id` as `Uuid` where this wire shape carries `call_id: String`;
/// the optional `ts` stamp is the same on both.) Do
/// NOT use this type to deserialize a stored run-record stream
/// (`task_run_records.stream`): it hard-fails on the `Input` tag. Parse
/// those with `plana_text::StreamSegment` (or the consumer's own adapter,
/// as the chest webui does): a run-record stream may carry either caller
/// arm, and an older reader that enumerates variants hard-fails on the
/// whole stream rather than skipping one row. The divergence is pinned by
/// `wire_stream_segment_mirrors_live_events_only` below — adding `Input`
/// here must be a deliberate decision that flips that test.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub enum StreamSegment {
    Text {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
        /// Wall-clock ms epoch at the segment's last activity — mirrors
        /// plana_text's segment stamp (plana #407) so the two shapes stay
        /// aligned; absent on legacy/hand-built values.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        #[ts(type = "number")]
        ts: Option<u64>,
    },
    Thinking {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        #[ts(type = "number")]
        ts: Option<u64>,
    },
    DeepThinking {
        text: String,
        #[serde(default)]
        #[ts(optional)]
        #[ts(type = "string")]
        message_id: Option<uuid::Uuid>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        #[ts(type = "number")]
        ts: Option<u64>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        #[ts(type = "number")]
        ts: Option<u64>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        #[ts(type = "number")]
        ts: Option<u64>,
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

// Declared through the yuuka `derive_enum!` house form (the same shape as
// `error_paths::UsageErrors`): the error identity the wire carries must be an
// enum, declared with the workspace's enum DSL, in the contract crate — not a
// hand-rolled `serde` enum. The derives come FIRST inside the macro (a
// `derive_enum!` parser constraint); the `ts` / `serde` attributes and the
// per-variant `#[error(...)]` prose pass through unchanged, so the wire form
// (`snake_case` union in `bindings/ws/core.ts`) is byte-identical to the
// pre-macro declaration.
yuuka::derive_enum!(
    #[derive(JsonSchema, PartialEq, Eq, Hash, Serialize, Deserialize, TS, thiserror::Error)]
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
        /// An infrastructure dependency (IPC peer, upstream service, database) is
        /// unreachable or unhealthy. See [`StructuredAgentError`]'s `context`
        /// field for the canonical keys carried alongside this code.
        #[error("infrastructure dependency failure")]
        DependencyDown,
        /// The request was refused because the engine is already processing
        /// another user message (single-flight guard), not because of a fault.
        #[error("agent is busy processing another request")]
        AgentBusy,
        /// The request was refused because the task's budget (steps, tokens or
        /// wall-clock allowance) is exhausted, not because of a fault.
        #[error("task budget is exhausted")]
        BudgetExhausted,
    }
);

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

    pub fn is_dependency_error(&self) -> bool {
        matches!(self, Self::DependencyDown)
    }

    /// Engine-flow refusals: the request was declined by the engine's own
    /// flow control (single-flight guard, budget accounting) rather than
    /// failing inside a model call, tool run or chain step.
    pub fn is_engine_flow_error(&self) -> bool {
        matches!(self, Self::AgentBusy | Self::BudgetExhausted)
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

#[derive(JsonSchema, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/core.ts")]
pub struct StructuredAgentError {
    pub code: AgentErrorCode,
    #[serde(default)]
    #[ts(optional)]
    pub detail: Option<String>,
    /// Free-form context bag; absent on the wire deserializes as an empty
    /// map (serde default), so payloads written by newer producers stay
    /// readable by older consumers and vice versa.
    ///
    /// Identity-bearing values are closed vocabularies declared below, so
    /// consumers key display catalogs off enum wire forms instead of
    /// matching free-form strings:
    /// - `reason`: the wire form of [`AgentErrorReason`] — carried on
    ///   engine-flow refusals (`AgentBusy` → `single_flight`,
    ///   `BudgetExhausted` → `quota_exceeded`), the chain deadline guard
    ///   (`ChainFailed` → `chain_deadline`), and infrastructure dependency
    ///   failures (`DependencyDown` → the dispatch taxonomy labels such as
    ///   `peer_unreachable`);
    /// - `interrupted`: the wire form of [`AgentInterruptReason`] — why an
    ///   in-flight chain was interrupted (`ChainFailed` → `restart`);
    /// - `dependency` (DependencyDown): stable identifier of the failed
    ///   dependency (e.g. `evernight-ipc`, `arona`);
    /// - `hint`: optional operator-facing remediation hint — deliberately
    ///   free prose, NOT part of a closed vocabulary.
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,
}

// The two context vocabularies below live in the same file as
// [`AgentErrorCode`] for the same reason that enum lives behind
// `derive_enum!`: they are error identity, and the contract crate is their
// home. Wire forms are snake_case strings identical to the tokens producers
// write today — the enums close the vocabulary without changing the wire.
// Doc comments sit INSIDE each macro invocation: rustdoc does not attach
// docs to items through a macro invocation from the outside.
yuuka::derive_enum!(
    #[derive(JsonSchema, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
    /// Closed vocabulary for the structured agent error's `reason` context
    /// value. Union of every token the engine writes today: the engine-flow
    /// refusal reasons, the chain deadline guard, and the dispatch/dependency
    /// taxonomy labels that flow in from evernight broker failures
    /// (`ClassifiedError` / `ErrorKind::label`). Unknown values stay readable
    /// on the wire (context is a string map); a new producer token must land
    /// here as a variant, not bypass the vocabulary.
    #[ts(export, export_to = "ws/core.ts")]
    #[serde(rename_all = "snake_case")]
    pub enum AgentErrorReason {
        /// The engine was already processing another user message
        /// (single-flight guard) — pairs with `AgentBusy`.
        SingleFlight,
        /// The task's quota/budget was exhausted — pairs with
        /// `BudgetExhausted`.
        QuotaExceeded,
        /// A chain step missed its deadline guard — pairs with
        /// `ChainFailed`.
        ChainDeadline,
        /// Broker rejected the envelope/params (`-32602`).
        InvalidParams,
        /// Unknown broker method / mistyped verb (`-32601`).
        MethodNotFound,
        /// Broker unreachable at the transport layer.
        PeerUnreachable,
        /// Server-side execution timeout.
        ExecutionTimeout,
        /// Authorization/authentication refusal (`AUTH_ERROR`).
        Auth,
        /// `target_scope` validation failure.
        TargetOutOfScope,
        /// The idempotency key was already seen; rejected locally.
        DuplicateRequest,
        /// Anything else (original code/message/data preserved).
        Other,
    }
);

yuuka::derive_enum!(
    #[derive(JsonSchema, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
    /// Closed vocabulary for the structured agent error's `interrupted`
    /// context value: why an in-flight chain was interrupted. `Restart` is
    /// the restart-sweep discriminator chest renders the aggregated
    /// interruption card from (ente #821); future interruption causes extend
    /// this enum instead of minting new free-form tokens.
    #[ts(export, export_to = "ws/core.ts")]
    #[serde(rename_all = "snake_case")]
    pub enum AgentInterruptReason {
        /// The engine restart interrupted the in-flight chain.
        Restart,
    }
);

impl AgentErrorReason {
    /// The exact wire form (`context` value spelling). Producers write
    /// vocabularies into the string-map context through this — the enum
    /// stays the only source of the token, and the wire-pin test asserts
    /// this never drifts from the serde form.
    pub fn wire_form(&self) -> &'static str {
        match self {
            Self::SingleFlight => "single_flight",
            Self::QuotaExceeded => "quota_exceeded",
            Self::ChainDeadline => "chain_deadline",
            Self::InvalidParams => "invalid_params",
            Self::MethodNotFound => "method_not_found",
            Self::PeerUnreachable => "peer_unreachable",
            Self::ExecutionTimeout => "execution_timeout",
            Self::Auth => "auth",
            Self::TargetOutOfScope => "target_out_of_scope",
            Self::DuplicateRequest => "duplicate_request",
            Self::Other => "other",
        }
    }
}

impl AgentInterruptReason {
    /// See [`AgentErrorReason::wire_form`].
    pub fn wire_form(&self) -> &'static str {
        match self {
            Self::Restart => "restart",
        }
    }
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
// the foundation's `plana::protocol_core`, which re-exports it at the
// `plana` crate root.
// ═══════════════════════════════════════════════════════════════

// The client-capability handshake payload types (scepter-flavored) stay at
// the crate root. The generic handshake primitives (HandshakeAckParams,
// PingParams, HANDSHAKE_VERSION) live in `plana::protocol_core`.
pub use protocol::base_messages::*;
pub use protocol::handshake::*;
// The platform-specific JSON-RPC error codes stay reachable at the crate
// root (`plana_celestia_types::jsonrpc::error_codes`) as a re-export of the
// single canonical definition in the foundation's `plana::jsonrpc::types`,
// which `plana` exposes as `plana::jsonrpc`.
pub use protocol::jsonrpc;

// enums/ — foundational shared enums (Agent, WorkStatus, etc.)
pub use enums::*;

// malkuth/ — supervision protocol types (restart authorization gate)
pub use malkuth::*;

// model/ — unified model management (re-export key types to crate root
// for ergonomic access: `arona::ModelCapability` not `arona::model::…`)
pub use model::{DeploymentMode, GenerationTier, HardwareRequirements, ModelCapability, ModelType};

// mdd/ — model deployment descriptor schema v1
pub use mdd::*;

// ws/ — SyncMessage variant params (types at crate root)
pub use ws::agent::{agent_lifecycle::*, layer2::*, state_sync::*, tasks::*, yolo::*};
pub use ws::services::{auth::*, industrial::*, knowledge_base::*, llm_provider::*};
pub use ws::ui::{
    bridge_network::*, file_browsing::*, logs::*, noa::*, realtime::*, system_ui::*,
    view_template::*, views::*, workspace::*,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_tier_accepts_lowercase_aliases() {
        for (text, expected) in [
            ("Deep", ModelTier::Deep),
            ("deep", ModelTier::Deep),
            ("normal", ModelTier::Normal),
            ("Basic", ModelTier::Basic),
        ] {
            let parsed: ModelTier = serde_json::from_str(&format!("{text:?}"))
                .unwrap_or_else(|e| panic!("failed to parse {text:?}: {e}"));
            assert_eq!(parsed, expected);
        }
        // Serialization stays PascalCase (the TS wire type contract).
        assert_eq!(serde_json::to_string(&ModelTier::Deep).unwrap(), "\"Deep\"");
        assert!(serde_json::from_str::<ModelTier>("\"premium\"").is_err());
    }
    use serde_json::json;

    // ── Agent enum ─────────────────────────────────────────────────

    #[test]
    fn agent_all_returns_twelve_unique_variants() {
        let all = Agent::all();
        assert_eq!(
            all.len(),
            12,
            "Agent::all() must return exactly 12 variants"
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
    fn agent_error_code_is_dependency_error() {
        assert!(AgentErrorCode::DependencyDown.is_dependency_error());
        assert!(!AgentErrorCode::ChainFailed.is_dependency_error());
        assert!(!AgentErrorCode::CosmosNoConnection.is_dependency_error());
    }

    #[test]
    fn agent_error_code_is_engine_flow_error() {
        assert!(AgentErrorCode::AgentBusy.is_engine_flow_error());
        assert!(AgentErrorCode::BudgetExhausted.is_engine_flow_error());
        assert!(!AgentErrorCode::DependencyDown.is_engine_flow_error());
        assert!(!AgentErrorCode::LlmRateLimited.is_engine_flow_error());
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
            AgentErrorCode::DependencyDown,
            AgentErrorCode::AgentBusy,
            AgentErrorCode::BudgetExhausted,
        ] {
            let count = [
                code.is_llm_error(),
                code.is_cosmos_error(),
                code.is_chain_error(),
                code.is_skill_error(),
                code.is_model_selection_error(),
                code.is_dependency_error(),
                code.is_engine_flow_error(),
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

    /// The wire string of the dependency-failure code is a cross-service
    /// contract (chest-side error routing reads it verbatim).
    #[test]
    fn dependency_down_wire_string_is_stable() {
        let wire = serde_json::to_string(&AgentErrorCode::DependencyDown).unwrap();
        assert_eq!(wire, r#""dependency_down""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::DependencyDown);
        assert_eq!(
            AgentErrorCode::DependencyDown.to_string(),
            "infrastructure dependency failure"
        );
    }

    /// The wire strings of the two engine-flow codes are cross-service
    /// contracts (chest-side error routing reads them verbatim).
    #[test]
    fn engine_flow_wire_strings_are_stable() {
        let wire = serde_json::to_string(&AgentErrorCode::AgentBusy).unwrap();
        assert_eq!(wire, r#""agent_busy""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::AgentBusy);
        assert_eq!(
            AgentErrorCode::AgentBusy.to_string(),
            "agent is busy processing another request"
        );

        let wire = serde_json::to_string(&AgentErrorCode::BudgetExhausted).unwrap();
        assert_eq!(wire, r#""budget_exhausted""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::BudgetExhausted);
        assert_eq!(
            AgentErrorCode::BudgetExhausted.to_string(),
            "task budget is exhausted"
        );
    }

    /// The `reason` context vocabulary's wire strings are cross-service
    /// contracts: engine producers write them into
    /// `StructuredAgentError.context` and chest keys display catalogs off
    /// them. Every variant is pinned to its exact snake_case wire form — a
    /// rename drifts consumers' catalogs silently, so it must fail here.
    #[test]
    fn agent_error_reason_wire_strings_are_stable() {
        let expected: &[(&str, &str)] = &[
            ("SingleFlight", "single_flight"),
            ("QuotaExceeded", "quota_exceeded"),
            ("ChainDeadline", "chain_deadline"),
            ("InvalidParams", "invalid_params"),
            ("MethodNotFound", "method_not_found"),
            ("PeerUnreachable", "peer_unreachable"),
            ("ExecutionTimeout", "execution_timeout"),
            ("Auth", "auth"),
            ("TargetOutOfScope", "target_out_of_scope"),
            ("DuplicateRequest", "duplicate_request"),
            ("Other", "other"),
        ];
        assert_eq!(expected.len(), 11);
        for (variant, wire) in expected {
            let value = match *variant {
                "SingleFlight" => AgentErrorReason::SingleFlight,
                "QuotaExceeded" => AgentErrorReason::QuotaExceeded,
                "ChainDeadline" => AgentErrorReason::ChainDeadline,
                "InvalidParams" => AgentErrorReason::InvalidParams,
                "MethodNotFound" => AgentErrorReason::MethodNotFound,
                "PeerUnreachable" => AgentErrorReason::PeerUnreachable,
                "ExecutionTimeout" => AgentErrorReason::ExecutionTimeout,
                "Auth" => AgentErrorReason::Auth,
                "TargetOutOfScope" => AgentErrorReason::TargetOutOfScope,
                "DuplicateRequest" => AgentErrorReason::DuplicateRequest,
                "Other" => AgentErrorReason::Other,
                _ => unreachable!(),
            };
            let encoded = serde_json::to_string(&value).unwrap();
            assert_eq!(encoded, format!(r#""{wire}""#), "variant {variant}");
            let back: AgentErrorReason = serde_json::from_str(&encoded).unwrap();
            assert_eq!(back, value, "variant {variant}");
            // `wire_form` is the producer-facing spelling — it must equal
            // the serde form for every variant or the vocabulary splits.
            assert_eq!(value.wire_form(), *wire, "wire_form drift on {variant}");
        }
    }

    /// The `interrupted` context vocabulary's wire string is the
    /// restart-interruption discriminator chest classifies the aggregated
    /// interruption card from (ente #821).
    #[test]
    fn agent_interrupt_reason_wire_strings_are_stable() {
        let encoded = serde_json::to_string(&AgentInterruptReason::Restart).unwrap();
        assert_eq!(encoded, r#""restart""#);
        let back: AgentInterruptReason = serde_json::from_str(&encoded).unwrap();
        assert_eq!(back, AgentInterruptReason::Restart);
        assert_eq!(AgentInterruptReason::Restart.wire_form(), "restart");
    }

    /// Backward-compatibility tolerance for the `context` field, in both
    /// directions: a payload that carries `context` must stay readable by a
    /// consumer whose struct predates the field (unknown fields are ignored),
    /// and the serde-default field must materialize as empty for payloads
    /// that omit it.
    #[test]
    fn structured_agent_error_tolerates_context_field_in_both_directions() {
        // Old-consumer struct shape: `code` + optional `detail`, no `context`.
        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        struct LegacyStructuredAgentError {
            code: AgentErrorCode,
            #[serde(default)]
            detail: Option<String>,
        }

        let payload = json!({
            "code": "dependency_down",
            "detail": "evernight ipc bridge is unreachable",
            "context": {
                "dependency": "evernight-ipc",
                "reason": "peer_unreachable",
                "hint": "check the evernight-ipc service on the host",
            },
        });

        let legacy: LegacyStructuredAgentError = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(legacy.code, AgentErrorCode::DependencyDown);
        assert_eq!(
            legacy.detail.as_deref(),
            Some("evernight ipc bridge is unreachable")
        );

        let full: StructuredAgentError = serde_json::from_value(payload).unwrap();
        assert_eq!(full.code, AgentErrorCode::DependencyDown);
        assert_eq!(
            full.context.get("dependency").map(String::as_str),
            Some("evernight-ipc")
        );
        assert_eq!(
            full.context.get("reason").map(String::as_str),
            Some("peer_unreachable")
        );
        assert_eq!(
            full.context.get("hint").map(String::as_str),
            Some("check the evernight-ipc service on the host")
        );

        // A payload without `context` deserializes with the default empty bag.
        let bare: StructuredAgentError = serde_json::from_value(json!({
            "code": "chain_failed",
        }))
        .unwrap();
        assert_eq!(bare.code, AgentErrorCode::ChainFailed);
        assert_eq!(bare.detail, None);
        assert!(bare.context.is_empty());
    }

    #[test]
    fn structured_agent_error_dependency_context_round_trip() {
        let err = StructuredAgentError {
            code: AgentErrorCode::DependencyDown,
            detail: Some("infrastructure dependency is unreachable".into()),
            context: [
                ("dependency".to_string(), "arona".to_string()),
                ("reason".to_string(), "http_probe_failed".to_string()),
                (
                    "hint".to_string(),
                    "restart the arona container".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        };
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v["code"], "dependency_down");
        assert_eq!(v["context"]["dependency"], "arona");
        assert_eq!(v["context"]["reason"], "http_probe_failed");
        assert_eq!(v["context"]["hint"], "restart the arona container");
        let back: StructuredAgentError = serde_json::from_value(v).unwrap();
        assert_eq!(back, err);
    }

    // ── StreamSegment variants ─────────────────────────────────────

    #[test]
    fn stream_segment_text_round_trip() {
        let seg = StreamSegment::Text {
            text: "hello".into(),
            message_id: None,
            ts: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        // Externally tagged enum: {"Text": {"text": "hello", "message_id": null}}
        assert_eq!(v["Text"]["text"], "hello");
        // The ts half of the #407 alignment: absent stays OFF the wire,
        // present rides as a number, and legacy rows deserialize to None.
        assert!(v["Text"].get("ts").is_none());
        let stamped: StreamSegment = serde_json::from_value(json!({
            "Text": {"text": "hi", "ts": 1790000000123u64}
        }))
        .unwrap();
        match stamped {
            StreamSegment::Text {
                ts: Some(1_790_000_000_123),
                ..
            } => {}
            other => panic!("expected a stamped Text, got {other:?}"),
        }
        let legacy: StreamSegment = serde_json::from_value(json!({
            "Text": {"text": "hi"}
        }))
        .unwrap();
        match legacy {
            StreamSegment::Text { ts: None, .. } => {}
            other => panic!("expected an unstamped Text, got {other:?}"),
        }
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
            ts: None,
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
            ts: None,
        };
        let v = serde_json::to_value(&seg).unwrap();
        // The ts omission is pinned on every variant, not just Text (R2 M1).
        assert!(v["ToolResult"].get("ts").is_none());
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

    /// The wire mirror carries every LIVE-streamed variant and exactly
    /// those: `Input` (the run-record-only arm added by plana #420) must
    /// be rejected here. If a future change adds an `Input` arm — or the
    /// runtime enum grows a new live-streamed variant this mirror does
    /// not know — this test forces the decision into the open.
    #[test]
    fn wire_stream_segment_mirrors_live_events_only() {
        // The five live-streamed shapes parse on the wire type.
        let live = [
            json!({"Text": {"text": "t", "message_id": null}}),
            json!({"Thinking": {"text": "t", "message_id": null}}),
            json!({"DeepThinking": {"text": "t", "message_id": null}}),
            json!({"ToolCall": {
                "tool_name": "kalos.file_read", "call_id": "call-1",
                "params": {"path": "/x"}, "agent_type": null, "message_id": null}}),
            json!({"ToolResult": {
                "tool_name": "kalos.file_read", "call_id": "call-1",
                "success": true, "data": {"ok": true},
                "duration_ms": null, "agent_type": null, "message_id": null}}),
        ];
        for sample in live {
            serde_json::from_value::<StreamSegment>(sample.clone())
                .unwrap_or_else(|e| panic!("live variant must parse: {sample}: {e}"));
        }

        // The run-record-only Input arm is NOT part of the live wire.
        let input = json!({"Input": {
            "messages": [{"role": "system", "content": "composed stack"}],
            "message_id": null}});
        assert!(
            serde_json::from_value::<StreamSegment>(input).is_err(),
            "the wire mirror must NOT accept the run-record-only Input arm — \
             if this fails, a deliberate wire decision changed; see the enum's doc"
        );

        // The caller-recorded prompt-change notice is run-record-only too
        // (2026-10-22): it never streams live, so this wire mirror must
        // reject it exactly like Input. This half of the divergence needs
        // its own pin — adding the arm above would not trip the Input
        // assertion.
        let notice = json!({"PromptNotice": {"source": "tools"}});
        assert!(
            serde_json::from_value::<StreamSegment>(notice).is_err(),
            "the wire mirror must NOT accept the caller-only PromptNotice arm — \
             if this fails, a deliberate wire decision changed; see the enum's doc"
        );
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

    /// The notice chunk kind is part of the LIVE wire: it must serialize
    /// as the exact tag consumers dispatch on, and an old consumer that
    /// bins unknown kinds as text would smear the source into prose — the
    /// paired chest release dispatches it explicitly.
    #[test]
    fn stream_chunk_kind_notice_serializes_on_the_live_wire() {
        assert_eq!(
            serde_json::to_string(&StreamChunkKind::Notice).unwrap(),
            r#""Notice""#
        );
        let back: StreamChunkKind = serde_json::from_str(r#""Notice""#).unwrap();
        assert_eq!(back, StreamChunkKind::Notice);
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
