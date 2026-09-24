//! Index of the gateway/TUI wire DTOs.
//!
//! Each `pub mod` below owns one slice of the vocabulary; the `pub use`
//! groups re-export the same types so downstream crates can import them as
//! `plana_state_sync::gateway::<Type>` without spelling the module path.
/// Per-agent telemetry: update params, request/completion state and list rows.
pub mod agent;
/// Config-filesystem mirrors: providers, entry points, models, user config and
/// key metadata.
pub mod config_fs;
/// Paginated conversation-history rows and the page wrapper around them.
pub mod history;
/// Knowledge-base, document and subscription DTOs for the RAG service.
pub mod knowledge_base;
/// Layer-2 marketplace descriptors: custom agents plus their tool and skill
/// inventories.
pub mod layer2;
/// The sync wire vocabulary: `SyncMessage` and the payload modules nested under
/// it.
pub mod message;
/// Configured-provider rows, usage-period kinds and per-period usage
/// counters.
pub mod provider;
/// Semantic-search hits and the response envelope around them.
pub mod search;
/// Snapshot and patch types for agents, containers, tasks, models and
/// providers.
pub mod snapshot;
/// YOLO Cruise Control tiers, their task configuration and their run status.
pub mod yolo;

// ── Re-exports ────────────────────────────────────────
/// Agent update params, request/completion state and TUI agent rows.
pub use agent::{AgentUpdateParams, CompletionOutcome, RequestState, TuiAgentInfo};
/// Config-filesystem mirrors: providers, entry points, models, user config and
/// key metadata.
pub use config_fs::{
    EntrypointApiConfigInfo, EntrypointConfigInfo, EntrypointDefaultsInfo, KeyInfo, KeyMetadata,
    MaxConcurrentInfo, ModelFsInfo, ModelFsPricing, ProviderCapabilitiesInfo, ProviderFsInfo,
    ProviderLimitsInfo, QuotaInfo, RateRuleInfo, UserInfo,
};
/// Conversation-history rows and their page wrapper.
pub use history::{HistoryMessage, MessagesPage};
/// Knowledge-base, document and subscription DTOs.
pub use knowledge_base::{
    AddDocumentRequest, AddDocumentResponse, CreateKnowledgeBaseRequest,
    CreateKnowledgeBaseResponse, CreateSubscriptionRequest, CreateSubscriptionResponse,
    DeleteKnowledgeBaseResponse, DeleteSubscriptionResponse, DocumentStatus, EmbeddingModel,
    KnowledgeBaseFilters, KnowledgeBaseInfo, KnowledgeBaseStatus, QueryKnowledgeBaseRequest,
    QueryKnowledgeBaseResponse, QueryResultChunk, SubscriptionStatus, SubscriptionType,
    SyncSubscriptionRequest, SyncSubscriptionResponse,
};
/// Layer-2 agents and their tool/skill descriptors.
pub use layer2::{CustomAgentInfo, Layer2AgentInfo, Layer2SkillInfo, Layer2ToolInfo};
/// Identity, file, event and client-info payloads used by `SyncMessage`.
pub use message::{
    ActorClaims, AuthUserInfo, ClientCapability, ClientNodeInfo, FilePayload, NoaEvent,
    PolemosDeviceInfo, SyncMessage,
};

// Industrial wire types (telemetry / alarm / discovery / write-approval /
// station topology / alarm history) live in `message::types` but are
// re-exported here so downstream crates can import them via the shorter
// `plana_state_sync::gateway::IndustrialAlarmEvent` path.
/// Industrial telemetry, alarm, discovery and write-approval types (defined in
/// `message::types`, re-exported here for the shorter `gateway::` import path).
pub use message::{
    IndustrialAlarmEvent, IndustrialAlarmHistory, IndustrialAlarmHistoryEntry,
    IndustrialAlarmLevel, IndustrialAlarmThresholds, IndustrialDiscoveryPhase,
    IndustrialDiscoveryProgress, IndustrialSensorReading, IndustrialStationField,
    IndustrialStationInfo, WriteApprovalRequest, WriteApprovalRisk,
};
/// Configured-provider rows, usage-period kinds and usage counters.
pub use provider::{ConfiguredProvider, PeriodType, UsagePeriodData};
/// Semantic-search hits and the response envelope.
pub use search::{SearchHit, SearchResponse};
/// Snapshot and patch types for agents, containers, tasks, models and
/// providers.
pub use snapshot::{
    AgentPatch, AgentSnapshot, ContainerInfo, ContainerPatch, ContainerSnapshot, GlobalSnapshot,
    LogEntryData, ModelInfo, ProviderInfo, TaskInfo, TaskPatch, TasksSnapshot,
};
