//! State and wire types for the entelecheia platform.
//!
//! This crate owns the plain data that flows between the agent runtime, the
//! gateway and the TUI dashboard: agent lifecycle, tool configuration, gateway
//! messages and the snapshot/patch model behind the dashboard. Every type here
//! is `Serialize`/`Deserialize` data with no business logic, which is what lets
//! those components share one definition instead of translating between three.
//!
//! What lives here, module by module:
//! - `agent`: agent kinds and categories, plus per-instance state and the two
//!   status axes (`AgentInfo`, `AgentStatus`, `WorkStatus`).
//! - `tools`: tool metadata and invocation contracts (`ToolInfo`,
//!   `ToolCallRequest`, `ToolCallResponse`), mandatory-prompt injection, and
//!   the todo/compression payloads.
//! - `gateway`: the `Message` union (Base, Agent, Tool, Skill, Node, Monitor,
//!   Sync, Conversation) and the `tui_types` snapshot/patch model for agents,
//!   containers, providers, knowledge bases and tasks.
//! - `types`: task status; `doc_loader`: per-tool markdown docs;
//!   `agent_context`: the in-process agent handle; `agent_error`: a re-export
//!   shim over the error taxonomy defined in `plana_core`.
//!
//! What does not live here: the primitive id, tier and error types come from
//! `plana_core`, the canonical agent list from `plana_domain_agent`, and the
//! generic protocol foundation is re-exported wholesale as `plana`.
#![allow(clippy::type_complexity)]

/// Agent kinds, categories and per-instance state; see `AgentInfo` and `Agent`.
pub mod agent;
/// The in-process agent handle (`AgentContext`); deliberately not a wire type.
pub mod agent_context;
/// Re-export shim: `AgentErrorCode` and `StructuredAgentError` live in `plana_core`.
pub mod agent_error;
/// Reads per-tool markdown docs from disk and folds them into `ToolInfo`.
pub mod doc_loader;
/// The gateway `Message` union and the `tui_types` snapshot/patch model.
pub mod gateway;
/// Tool definitions, invocation envelopes, prompt injection and todo markers.
pub mod tools;
/// Task status and its parse error; `ModelTier` is re-exported from `plana_core`.
pub mod types;

pub use agent::{
    Agent, AgentCategory, AgentInfo, AgentRegisterRequest, AgentStatus, AgentUnregisterRequest,
    CustomAgentId, WorkStatus,
};
pub use agent_context::AgentContext;
pub use agent_error::{AgentErrorCode, StructuredAgentError};
pub use doc_loader::{ToolDoc, ToolDocLoader};
pub use gateway::{
    AgentMessage, AskAnswerSource, BaseMessage, ClientCapability, ClientNodeInfo,
    ConversationMessage, CosmosContainerInfo, CosmosOperationLogEntry, FilePayload, Message,
    MetricsData, MonitorMessage, NodeInfo, NodeMessage, PolemosDeviceInfo, ReportSelection,
    ReportType, RetryReason, RouteInfo, SkillMessage, SkillStage, SyncMessage, SystemNotification,
    ToolMessage,
    tui_types::{
        ActorClaims, AgentPatch, AgentSnapshot, AgentUpdateParams, AuthUserInfo, CompletionOutcome,
        ConfiguredProvider, ContainerInfo, ContainerPatch, ContainerSnapshot, CustomAgentInfo,
        EntrypointApiConfigInfo, EntrypointConfigInfo, EntrypointDefaultsInfo, GlobalSnapshot,
        HistoryMessage, KeyInfo, KeyMetadata, KnowledgeBaseFilters, KnowledgeBaseInfo,
        KnowledgeBaseStatus, Layer2AgentInfo, Layer2SkillInfo, Layer2ToolInfo, LogEntryData,
        MaxConcurrentInfo, MessagesPage, ModelFsInfo, ModelFsPricing, ModelInfo, NoaEvent,
        PeriodType, ProviderCapabilitiesInfo, ProviderFsInfo, ProviderInfo, ProviderLimitsInfo,
        QuotaInfo, RateRuleInfo, RequestState, SearchHit, SearchResponse, TaskInfo, TaskPatch,
        TasksSnapshot, TuiAgentInfo, UsagePeriodData, UserInfo,
        knowledge_base::{
            AddDocumentRequest, AddDocumentResponse, CreateKnowledgeBaseRequest,
            CreateKnowledgeBaseResponse, CreateSubscriptionRequest, CreateSubscriptionResponse,
            DeleteKnowledgeBaseResponse, DeleteSubscriptionResponse, DocumentStatus,
            EmbeddingModel, QueryKnowledgeBaseRequest, QueryKnowledgeBaseResponse,
            QueryResultChunk, SubscriptionStatus, SubscriptionType, SyncSubscriptionRequest,
            SyncSubscriptionResponse,
        },
        message::SyncMessage as GatewaySyncMessage,
        yolo::{
            YoloFullConfig, YoloTaskResult, YoloTaskStatus, YoloTaskTier, YoloTierConfig,
            YoloTierStatus, YoloTierTaskConfig,
        },
    },
};
/// Re-export of the `plana` protocol foundation this crate is built on, so a
/// consumer can name both through `plana_state_sync` alone.
pub use plana;
pub use tools::{
    CompressedContext, MarkedTodoItem, MarkerStrategy, PreserveState, PromptInjectionPolicy,
    SkillInfo, SkillLocation, TodoMarker, ToolCallMode, ToolCallRequest, ToolCallResponse,
    ToolConfig, ToolInfo, ToolLocation, ToolMaturity, ToolParameters, ToolPromptInjector,
    ToolVisibility,
};
pub use types::{ModelTier, TaskStatus, UnknownTaskStatusError};
