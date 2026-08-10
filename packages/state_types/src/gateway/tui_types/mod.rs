pub mod agent;
pub mod config_fs;
pub mod history;
pub mod knowledge_base;
pub mod layer2;
pub mod message;
pub mod provider;
pub mod search;
pub mod snapshot;
pub mod yolo;

pub use agent::{AgentUpdateParams, CompletionOutcome, RequestState, TuiAgentInfo};
pub use config_fs::{
    EntrypointApiConfigInfo, EntrypointConfigInfo, EntrypointDefaultsInfo, KeyInfo, KeyMetadata,
    MaxConcurrentInfo, ModelFsInfo, ModelFsPricing, ProviderCapabilitiesInfo, ProviderFsInfo,
    ProviderLimitsInfo, QuotaInfo, RateRuleInfo, UserInfo,
};
pub use history::{HistoryMessage, MessagesPage};
pub use knowledge_base::{
    AddDocumentRequest, AddDocumentResponse, CreateKnowledgeBaseRequest,
    CreateKnowledgeBaseResponse, CreateSubscriptionRequest, CreateSubscriptionResponse,
    DeleteKnowledgeBaseResponse, DeleteSubscriptionResponse, DocumentStatus, EmbeddingModel,
    KnowledgeBaseFilters, KnowledgeBaseInfo, KnowledgeBaseStatus, QueryKnowledgeBaseRequest,
    QueryKnowledgeBaseResponse, QueryResultChunk, SubscriptionStatus, SubscriptionType,
    SyncSubscriptionRequest, SyncSubscriptionResponse,
};
pub use layer2::{CustomAgentInfo, Layer2AgentInfo, Layer2SkillInfo, Layer2ToolInfo};
pub use message::{
    AuthUserInfo, ClientCapability, ClientNodeInfo, FilePayload, NoaEvent, PolemosDeviceInfo,
    SyncMessage,
};

// Industrial wire types (telemetry / alarm / discovery / write-approval /
// station topology / alarm history) live in `message::types` but are
// re-exported here so downstream crates can import them via the shorter
// `_state_sync::gateway::IndustrialAlarmEvent` path.
pub use message::{
    IndustrialAlarmEvent, IndustrialAlarmHistory, IndustrialAlarmHistoryEntry,
    IndustrialAlarmLevel, IndustrialAlarmThresholds, IndustrialDiscoveryPhase,
    IndustrialDiscoveryProgress, IndustrialSensorReading, IndustrialStationField,
    IndustrialStationInfo, WriteApprovalRequest, WriteApprovalRisk,
};
pub use provider::{ConfiguredProvider, PeriodType, UsagePeriodData};
pub use search::{SearchHit, SearchResponse};
pub use snapshot::{
    AgentPatch, AgentSnapshot, ContainerInfo, ContainerPatch, ContainerSnapshot, GlobalSnapshot,
    LogEntryData, ModelInfo, ProviderInfo, TaskInfo, TaskPatch, TasksSnapshot,
};
