//! Core state types — Agent, AgentInfo, MCP, gateway messages.
//!
//! This crate defines the shared data structures that flow between every layer
//! of the Entelecheia platform: agent lifecycle, tool configuration, gateway
//! communication, and TUI dashboard state.
//!
//! Key abstractions:
//! - [`Agent`] / [`AgentInfo`] — agent registration, status, and metadata;
//!   [`AgentCategory`] distinguishes simple tools from complex multi-instance
//!   agents with badge assignment.
//! - [`Message`] — tagged union of all gateway message variants (Base, Agent,
//!   Mcp, Skill, Node, Monitor, Tui, Conversation), forming the wire protocol
//!   between agents and the control plane.
//! - [`McpToolInfo`] / [`McpToolCallRequest`] / [`McpToolCallResponse`] —
//!   MCP tool metadata, invocation contracts, and response envelopes with
//!   visibility, maturity, and injection-policy settings.
//! - TUI types (`tui_types` module) — comprehensive snapshot/patch model for
//!   the admin dashboard: agents, containers, providers, knowledge bases,
//!   quotas, and tasks.
//! - [`CoreError`] / [`CoreResult`] — error taxonomy for core operations.
//!
//! Design philosophy: these types are pure data (Serialize/Deserialize) with
//! no business logic, serving as the lingua franca that decouples the agent
//! runtime, gateway, and TUI from each other.
#![allow(clippy::type_complexity)]

pub mod agent;
pub mod agent_context;
pub mod agent_error;
pub mod doc_loader;
pub mod gateway;
pub mod mcp;
pub mod types;

pub use agent::{
    Agent, AgentCategory, AgentInfo, AgentRegisterRequest, AgentStatus, AgentUnregisterRequest,
    CustomAgentId, WorkStatus,
};
pub use agent_context::AgentContext;
pub use agent_error::{AgentErrorCode, StructuredAgentError};
pub use plana;
pub use doc_loader::{McpToolDoc, McpToolDocLoader};
pub use gateway::{
    AgentMessage, AskAnswerSource, BaseMessage, ClientCapability, ClientNodeInfo,
    ConversationMessage, CosmosContainerInfo, CosmosOperationLogEntry, FilePayload, McpMessage,
    Message, MetricsData, MonitorMessage, NodeInfo, NodeMessage, PolemosDeviceInfo,
    ReportSelection, ReportType, RetryReason, RouteInfo, SkillMessage, SkillStage,
    SystemNotification, TuiMessage,
    tui_types::{
        AgentPatch, AgentSnapshot, AgentUpdateParams, AuthUserInfo, CompletionOutcome,
        ConfiguredProvider, ContainerInfo, ContainerPatch, ContainerSnapshot, CustomAgentInfo,
        EntrypointApiConfigInfo, EntrypointConfigInfo, EntrypointDefaultsInfo, GlobalSnapshot,
        HistoryMessage, KeyInfo, KeyMetadata, KnowledgeBaseFilters, KnowledgeBaseInfo,
        KnowledgeBaseStatus, Layer2AgentInfo, Layer2McpToolInfo, Layer2SkillInfo, LogEntryData,
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
        message::TuiMessage as GatewayTuiMessage,
        yolo::{
            YoloFullConfig, YoloTaskResult, YoloTaskStatus, YoloTaskTier, YoloTierConfig,
            YoloTierStatus, YoloTierTaskConfig,
        },
    },
};
pub use mcp::{
    CompressedContext, MarkedTodoItem, MarkerStrategy, McpPromptInjector, McpToolCallMode,
    McpToolCallRequest, McpToolCallResponse, McpToolConfig, McpToolInfo, McpToolParameters,
    PreserveState, PromptInjectionPolicy, SkillInfo, SkillLocation, TodoMarker, ToolLocation,
    ToolMaturity, ToolVisibility,
};
pub use types::{ModelTier, TaskStatus, UnknownTaskStatusError};
