#[cfg(test)]
pub mod groups;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::super::{
    AskAnswerSource, ReportSelection, ReportType, RouteInfo, SkillStage, SystemNotification,
    monitor::{CosmosContainerInfo, CosmosOperationLogEntry},
};
use crate::{agent::Agent, agent_error::StructuredAgentError};
use arona_config::GenProtocol;
use arona_core::AgentBadge;
use arona_text::{LlmStream, StreamChunkKind};

fn default_search_limit() -> u64 {
    10
}

fn default_history_limit() -> u64 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolemosDeviceInfo {
    pub node_id: Uuid,
    pub name: String,
    pub address: String,
    pub status: String,
    pub workspace_path: Option<String>,
}

// ── Industrial telemetry / alarm / discovery / write-approval types ──
//
// These are the wire types pushed by scepter → shittim-chest's webui
// (and pulled via the `topology.*` / `industrial.*` JSON-RPC family).
// They mirror the local mirrors in `shittim-chest/packages/webui/src/
// stores/industrial.ts` so both sides of the WebSocket stay in sync.
//
// Field naming uses snake_case end-to-end (matches serde defaults and
// the existing TuiMessage variants); the webui's TS mirrors use the
// same shape so no remapping is required.

/// Severity ordering matches ISA-18.2 alarm severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialAlarmLevel {
    Log,
    LowLow,
    Low,
    High,
    HighHigh,
    RateOfChange,
    Emergency,
}

/// A single live reading from an industrial field (e.g. pressure cell on
/// a Modbus register, S7 DBX bit). Pushed by scepter at the scan cycle
/// of the underlying transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialSensorReading {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub name: String,
    pub raw_value: f64,
    pub scaled_value: f64,
    pub unit: String,
    pub quality: String,
    pub timestamp: String,
}

/// Fired on threshold breach (breached=true) or clear (breached=false).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmEvent {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
}

/// Phases of an evernight discovery scan. Ordered by typical progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialDiscoveryPhase {
    TransportScan,
    ProtocolIdentify,
    DataModelScan,
    SemanticInference,
    ManifestGeneration,
    ManifestValidation,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialDiscoveryProgress {
    pub session_id: String,
    pub phase: IndustrialDiscoveryPhase,
    pub message: String,
    pub found_devices: u64,
    pub progress_percent: u32,
    #[serde(default)]
    pub raw_findings: Option<serde_json::Value>,
}

/// Operator confirmation gate for safety-critical PLC writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteApprovalRisk {
    Safe,
    Caution,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteApprovalRequest {
    /// Unique id assigned by the producer (orexis). The operator UI echoes
    /// it back in `industrial.approveWrite` so scepter's resolver can match
    /// the response to the pending oneshot (Phase D.2, A.2.4.1→A.2.4.2).
    /// `#[serde(default)]` keeps the wire format backward-compatible with
    /// older push events that predate this field.
    #[serde(default)]
    pub request_id: String,
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub current_value: f64,
    pub proposed_value: f64,
    pub unit: String,
    pub reason: String,
    pub agent: String,
    pub risk_level: WriteApprovalRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialStationField {
    pub address: String,
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub alarm: Option<IndustrialAlarmThresholds>,
    #[serde(default)]
    pub current_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmThresholds {
    #[serde(default)]
    pub ll: Option<f64>,
    #[serde(default)]
    pub l: Option<f64>,
    #[serde(default)]
    pub h: Option<f64>,
    #[serde(default)]
    pub hh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialStationInfo {
    pub station_id: String,
    pub protocol: String,
    pub connection: String,
    pub device_class: String,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub firmware: Option<String>,
    pub status: String,
    #[serde(default)]
    pub fields: Vec<IndustrialStationField>,
}

/// One entry in the historical alarm log (last N days).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmHistoryEntry {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
    /// Whether an operator acknowledged the alarm, and when.
    #[serde(default)]
    pub acknowledged: bool,
    #[serde(default)]
    pub acknowledged_at: Option<String>,
    #[serde(default)]
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmHistory {
    pub entries: Vec<IndustrialAlarmHistoryEntry>,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    FileRelay,
    Terminal,
    ScreenCapture,
    NoaWorkspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientNodeInfo {
    pub hostname: String,
    pub os: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePayload {
    pub relative_path: String,
    pub content: String,
    pub size: u64,
    #[serde(default)]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoaEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub file_path: Option<String>,
    pub content_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ═══ File Browsing ═══
// Mirrors arona's `FileTarget` / `FileTreeEntry` / file-browse params. Used by
// the node-list container cards, the Bridge Network host/workspace cards, and
// the workspace file browser to list/read files inside a container, on a host,
// or in a workspace checkout.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileTargetKind {
    Container,
    Host,
    Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTarget {
    pub kind: FileTargetKind,
    /// Container badge (`#demiurge` / `#001`), host id, or workspace id.
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeEntry {
    pub name: String,
    /// `"file"` | `"dir"` | `"symlink"`.
    pub kind: String,
    pub size: u64,
}

// ═══ Bridge Network ═══
// Mirrors arona's host / workspace-node / git-status / token-usage structs.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub host_id: String,
    pub hostname: String,
    pub os: String,
    pub cpu_usage_percent: f64,
    pub cpu_cores: u32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    #[serde(default)]
    pub net_up_bps: Option<u64>,
    #[serde(default)]
    pub net_down_bps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceGitStatus {
    pub branch: String,
    #[serde(default)]
    pub modified: u32,
    #[serde(default)]
    pub ahead: u32,
    #[serde(default)]
    pub behind: u32,
    #[serde(default)]
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTokenUsage {
    pub agent: String,
    pub input: u64,
    pub output: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceNode {
    pub workspace_id: Uuid,
    pub host_id: String,
    pub path: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub git: Option<WorkspaceGitStatus>,
    #[serde(default)]
    pub token_usage: Vec<WorkspaceTokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserInfo {
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub is_active: bool,
}

/// A single reasoning step rendered in an agent's "thinking" timeline.
///
/// Carried by `Tui.AgentThinkingStep` (`params.step`). Emitted by the mock
/// simulator and by real agents; the webui appends it to the streaming store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStepEntry {
    pub id: String,
    pub content: String,
    /// `"running"` | `"completed"`.
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum TuiMessage {
    // ═══ Protocol / Connection ═══
    Ping {
        timestamp: u64,
    },
    // ═══ Layer-2 / Custom Agents ═══
    Layer2AgentList,
    Layer2AgentListResponse {
        agents: Vec<super::super::layer2::Layer2AgentInfo>,
    },
    Layer2AgentMcpTools {
        agent_name: String,
    },
    Layer2AgentMcpResponse {
        agent_name: String,
        tools: Vec<super::super::layer2::Layer2McpToolInfo>,
    },
    Layer2AgentSkills {
        agent_name: String,
    },
    Layer2AgentSkillsResponse {
        agent_name: String,
        skills: Vec<super::super::layer2::Layer2SkillInfo>,
    },
    Layer2AgentMcpPrompt {
        agent_name: String,
        tool: String,
        lang: Option<String>,
    },
    Layer2AgentMcpPromptResponse {
        agent_name: String,
        tool: String,
        lang: String,
        content: String,
        name: String,
    },
    Layer2AgentSkillPrompt {
        agent_name: String,
        skill: String,
        lang: Option<String>,
    },
    Layer2AgentSkillPromptResponse {
        agent_name: String,
        skill: String,
        lang: String,
        content: String,
        name: String,
    },
    CustomAgentList,
    CustomAgentListResponse {
        agents: Vec<super::super::layer2::CustomAgentInfo>,
    },
    SubscribeCustomAgent {
        source: String,
        repository: Option<String>,
        url: Option<String>,
    },
    SubscribeCustomAgentResponse {
        success: bool,
        error: Option<String>,
        agent: Option<super::super::layer2::CustomAgentInfo>,
        skills: Vec<String>,
        permissions: Vec<String>,
    },
    UnsubscribeCustomAgent {
        name: String,
    },
    UnsubscribeCustomAgentResponse {
        success: bool,
        error: Option<String>,
    },
    // ═══ Protocol / Connection (continued) ═══
    ServerVersion {
        version: String,
        build_info: String,
    },
    ConnectHandshake {
        token: String,
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        capabilities: Vec<ClientCapability>,
        #[serde(default)]
        node_info: Option<ClientNodeInfo>,
        #[serde(default)]
        workspace_id: Option<Uuid>,
        /// Identifies the client type: "cli" for request/response CLI,
        /// "tui" for the full-featured TUI. When "cli", the server
        /// skips PubSub bridge and filters background-agent broadcasts.
        #[serde(default)]
        client_type: Option<String>,
    },
    HandshakeAck {
        ok: bool,
        #[serde(default)]
        error: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        reconnect: bool,
    },
    VersionMismatch {
        server_version: String,
        client_version: String,
    },
    // ═══ Agent Lifecycle ═══
    UserMessage {
        sender_id: String,
        content: String,
        timestamp: String,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        images: Option<Vec<arona_core::LlmImageContent>>,
        #[serde(default)]
        workspace_id: Option<Uuid>,
    },
    AgentResponse {
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        content: String,
        timestamp: String,
        parent_id: Option<String>,
        #[serde(default)]
        workspace_id: Option<Uuid>,
    },
    AgentStreamingChunk {
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        chunk: String,
        is_done: bool,
        timestamp: String,
        #[serde(default)]
        chunk_kind: Option<StreamChunkKind>,
        #[serde(default)]
        workspace_id: Option<Uuid>,
    },
    AgentThinkingStep {
        agent_type: Agent,
        agent_id: String,
        step: ThinkingStepEntry,
    },
    AgentReport {
        report_type: ReportType,
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        title: String,
        content: String,
        summary: Option<String>,
        timestamp: String,
        preset_options: Vec<String>,
        /// For `report_type: "query"`: whether `preset_options` are mutually
        /// exclusive (single) or pick-any (multiple). Omit ⇒ single.
        #[serde(default)]
        selection_mode: Option<ReportSelection>,
        /// For `report_type: "query"`: whether the recipient may type a
        /// free-form answer in addition to (or instead of) picking presets.
        /// Omit ⇒ treated as `true` when `report_type == Query`.
        #[serde(default)]
        allow_custom_reply: Option<bool>,
        /// Subset of `preset_options` the agent suggests.
        #[serde(default)]
        recommended_options: Vec<String>,
        model_name: Option<String>,
        token_usage: Option<(u32, u32)>,
        #[serde(default)]
        skill_count: Option<u32>,
        #[serde(default)]
        mcp_count: Option<u32>,
        #[serde(default)]
        next_route: Option<RouteInfo>,
        #[serde(default)]
        stream: Option<LlmStream>,
        #[serde(default)]
        error: Option<StructuredAgentError>,
    },
    /// Server-bound reply to an inquiry (`AgentReport { report_type: Query }`)
    /// sent earlier. `report_id` mirrors the original `agent_id`.
    AgentReportReply {
        report_id: String,
        #[serde(default)]
        selected_options: Vec<String>,
        #[serde(default)]
        custom_answer: Option<String>,
        timestamp: String,
    },
    AgentTransfer {
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        from_skill: String,
        to_skill: String,
        #[serde(default)]
        stream: Option<LlmStream>,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        model_name: Option<String>,
        #[serde(default)]
        token_usage: Option<(u32, u32)>,
    },
    OrchestrationStatus {
        stage: SkillStage,
        agent: String,
        #[serde(default)]
        agent_type: Option<Agent>,
        #[serde(default)]
        tool_name: Option<String>,
        #[serde(default)]
        call_id: Option<Uuid>,
        #[serde(default)]
        parent_agent: Option<Uuid>,
        #[serde(default)]
        parameters_summary: Option<String>,
    },
    McpToolResult {
        tool_name: String,
        call_id: Uuid,
        #[serde(default)]
        parameters_summary: Option<String>,
        result: String,
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        success: bool,
        #[serde(default)]
        duration_ms: Option<u64>,
    },
    AgentToolCall {
        agent_type: Agent,
        agent_id: String,
        tool: String,
        #[serde(default)]
        params: Option<serde_json::Value>,
        #[serde(default)]
        result: Option<String>,
        status: String,
    },
    StreamingTail {
        agent_id: String,
        tail: String,
    },
    HumanReviewRequest {
        review_id: String,
        agent_type: Agent,
        agent_id: String,
        title: String,
        content: String,
        timestamp: String,
    },
    HumanReviewResponse {
        review_id: String,
        choice: String,
        comment: String,
    },
    AskHumanRequest {
        consultation_id: String,
        agent_type: Agent,
        agent_id: String,
        #[serde(default)]
        agent_number: Option<AgentBadge>,
        question: String,
        question_localized: String,
        context: Option<String>,
        options: Vec<String>,
        recommended: Option<String>,
        timestamp: String,
    },
    AskHumanReply {
        consultation_id: String,
        selected_options: Vec<String>,
        custom_answer: Option<String>,
        answered_by: AskAnswerSource,
        timestamp: String,
    },
    AutoModeUpdate {
        enabled: bool,
        timeout_secs: Option<u64>,
    },
    ScepterIdentity {
        device_id: Uuid,
    },
    ListAgents,
    AgentListResponse {
        agents: Vec<super::super::agent::TuiAgentInfo>,
    },
    AgentUpdate {
        agent: super::super::agent::TuiAgentInfo,
    },
    // ═══ Task Management ═══
    TaskCreated {
        task_id: Uuid,
        issue_id: Uuid,
        title: String,
        #[serde(default)]
        description: Option<String>,
        assigned_agent: Option<Agent>,
        #[serde(default)]
        parent_task_id: Option<Uuid>,
        #[serde(default)]
        badge: Option<AgentBadge>,
    },
    TaskStatusUpdate {
        task_id: Uuid,
        status: crate::TaskStatus,
        progress: u8,
    },
    // ═══ LLM Provider Configuration ═══
    ConfigureLlmProvider {
        provider_name: String,
        api_key: String,
        api_endpoint: Option<String>,
        default_model: String,
        provider_type: String,
    },
    LlmProviderConfigured {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    RenameProvider {
        provider_name: String,
        new_display_name: String,
    },
    ProviderRenamed {
        provider_name: String,
        new_display_name: String,
        success: bool,
        error: Option<String>,
    },
    EditProvider {
        provider_name: String,
        api_key: Option<String>,
        api_endpoint: Option<String>,
    },
    ProviderEdited {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    DeleteProvider {
        provider_name: String,
    },
    ProviderDeleted {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    ListConfiguredProviders,
    ConfiguredProvidersList {
        providers: Vec<super::super::provider::ConfiguredProvider>,
    },
    UpdateModelProviderConfig {
        provider_name: String,
        display_name: Option<String>,
        endpoint_url: Option<String>,
        model_id: Option<String>,
        context_window: Option<u64>,
        compression_threshold: Option<u64>,
        usage_type: Option<String>,
        price_input: Option<f64>,
        price_cache_input: Option<f64>,
        price_output: Option<f64>,
        period_reset_hours: Option<u64>,
        period_data_limit: Option<u64>,
        period_request_limit: Option<u64>,
        supports_image: Option<bool>,
        supports_audio: Option<bool>,
        supports_video: Option<bool>,
        can_reason: Option<bool>,
    },
    ModelProviderConfigUpdated {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    ValidateEndpoint {
        provider_name: String,
        api_endpoint: String,
        api_key: Option<String>,
        protocol: GenProtocol,
    },
    EndpointValidated {
        provider_name: String,
        is_reachable: bool,
        latency_ms: Option<u64>,
        error: Option<String>,
    },
    UsagePeriodQuery {
        user_id: Option<String>,
        period_types: Vec<super::super::provider::PeriodType>,
    },
    UsagePeriodResponse {
        data: Vec<super::super::provider::UsagePeriodData>,
    },
    UsagePeriodUpdate {
        data: Vec<super::super::provider::UsagePeriodData>,
    },
    // ═══ State Sync / Snapshots ═══
    RequestFullSnapshot,
    AgentPatch {
        patches: Vec<super::super::snapshot::AgentPatch>,
    },
    AgentSnapshot {
        snapshot: super::super::snapshot::AgentSnapshot,
    },
    RequestGlobalSnapshot,
    GlobalSnapshot {
        snapshot: super::super::snapshot::GlobalSnapshot,
    },
    ModelsSnapshot {
        models: Vec<super::super::snapshot::ModelInfo>,
    },
    ProvidersSnapshot {
        providers: Vec<super::super::snapshot::ProviderInfo>,
    },
    ContainerPatch {
        patches: Vec<super::super::snapshot::ContainerPatch>,
    },
    RequestContainerSnapshot,
    ContainerSnapshot {
        snapshot: super::super::snapshot::ContainerSnapshot,
    },
    TaskPatch {
        patches: Vec<super::super::snapshot::TaskPatch>,
    },
    RequestTasksSnapshot,
    TasksSnapshot {
        snapshot: super::super::snapshot::TasksSnapshot,
    },
    RequestVmSnapshot {
        agent_type: Agent,
        agent_id: String,
    },
    VmSnapshot {
        agent_id: String,
        globals: serde_json::Value,
        container_info: Option<CosmosContainerInfo>,
        tool_list: Vec<String>,
        op_log: Vec<CosmosOperationLogEntry>,
    },
    // ═══ Agent Lifecycle (continued) ═══
    RetryAgentRequest {
        agent_id: String,
        attempt: u32,
    },
    UndoRequest,
    // ═══ Config Filesystem ═══
    GetProvidersFromFs,
    ProvidersFromFsResponse {
        providers: Vec<super::super::config_fs::ProviderFsInfo>,
    },
    GetModelsFromFs,
    ModelsFromFsResponse {
        models: Vec<super::super::config_fs::ModelFsInfo>,
    },
    ReloadProviderConfig {
        provider_id: String,
    },
    ReloadModelConfig {
        provider_id: String,
        model_id: String,
    },
    GetUserConfig,
    UserConfigResponse {
        config: super::super::config_fs::UserInfo,
    },
    UpdateUserConfig {
        config: super::super::config_fs::UserInfo,
    },
    ReloadUserConfig,
    ReloadAllAgentConfigs,
    ReloadAgentConfig {
        agent_name: String,
    },
    ListKeys,
    KeysListResponse {
        keys: Vec<super::super::config_fs::KeyInfo>,
    },
    SaveApiKey {
        provider: String,
        api_key: String,
        metadata: Option<super::super::config_fs::KeyMetadata>,
    },
    DeleteApiKey {
        provider: String,
    },
    GetApiKeyInfo {
        provider: String,
    },
    ApiKeyInfoResponse {
        info: super::super::config_fs::KeyInfo,
    },
    // ═══ Semantic Search ═══
    SearchRequest {
        query: String,
        #[serde(default = "default_search_limit")]
        limit: u64,
        /// Optional source filter (e.g. "report", "knowledge").
        source: Option<String>,
        #[serde(default)]
        min_score: f64,
    },
    SearchResponse {
        response: super::super::search::SearchResponse,
    },
    // ═══ Conversation History (lazy-loaded reports) ═══
    /// Emitted once when the server creates a conversation for the active task,
    /// so the client can later request paginated message history.
    ConversationStarted {
        conversation_id: Uuid,
    },
    RequestRecentMessages {
        conversation_id: Uuid,
        #[serde(default = "default_history_limit")]
        limit: u64,
    },
    RequestOlderMessages {
        conversation_id: Uuid,
        /// ISO-8601 cursor: load messages created strictly before this time.
        before_created_at: String,
        #[serde(default = "default_history_limit")]
        limit: u64,
    },
    MessagesResponse {
        conversation_id: Uuid,
        page: super::super::history::MessagesPage,
    },
    // ═══ Knowledge Base ═══
    CreateKnowledgeBase {
        request: super::super::knowledge_base::CreateKnowledgeBaseRequest,
    },
    CreateKnowledgeBaseResponse {
        response: super::super::knowledge_base::CreateKnowledgeBaseResponse,
    },
    AddDocument {
        request: super::super::knowledge_base::AddDocumentRequest,
    },
    AddDocumentResponse {
        response: super::super::knowledge_base::AddDocumentResponse,
    },
    QueryKnowledgeBase {
        request: super::super::knowledge_base::QueryKnowledgeBaseRequest,
    },
    QueryKnowledgeBaseResponse {
        response: super::super::knowledge_base::QueryKnowledgeBaseResponse,
    },
    CreateSubscription {
        request: super::super::knowledge_base::CreateSubscriptionRequest,
    },
    CreateSubscriptionResponse {
        response: super::super::knowledge_base::CreateSubscriptionResponse,
    },
    SyncSubscription {
        request: super::super::knowledge_base::SyncSubscriptionRequest,
    },
    SyncSubscriptionResponse {
        response: super::super::knowledge_base::SyncSubscriptionResponse,
    },
    DeleteSubscription {
        subscription_id: Uuid,
    },
    DeleteSubscriptionResponse {
        response: super::super::knowledge_base::DeleteSubscriptionResponse,
    },
    GetKnowledgeBase {
        knowledge_base_id: Uuid,
    },
    GetKnowledgeBaseResponse {
        knowledge_base: Option<super::super::knowledge_base::KnowledgeBaseInfo>,
    },
    ListKnowledgeBases {
        filters: Option<super::super::knowledge_base::KnowledgeBaseFilters>,
    },
    ListKnowledgeBasesResponse {
        knowledge_bases: Vec<super::super::knowledge_base::KnowledgeBaseInfo>,
    },
    DeleteKnowledgeBase {
        knowledge_base_id: Uuid,
    },
    DeleteKnowledgeBaseResponse {
        response: super::super::knowledge_base::DeleteKnowledgeBaseResponse,
    },
    // ═══ Workspace ═══
    OpenWorkspace {
        uri: String,
    },
    OpenWorkspaceResponse {
        success: bool,
        workspace_id: Option<Uuid>,
        #[serde(default)]
        error: Option<String>,
    },
    WorkspaceStatus {
        workspace_id: Uuid,
        display_name: Option<String>,
        connection_kind: String,
        resolved_path: Option<String>,
        remote_url: Option<String>,
        branch: Option<String>,
        host_id: Option<String>,
    },
    RequestWorkspaceStatus,
    ListPolemosDevices,
    PolemosDeviceList {
        devices: Vec<PolemosDeviceInfo>,
    },
    RegisterPolemosDevice {
        host_id: String,
        address: String,
        workspace_path: Option<String>,
    },
    RegisterPolemosDeviceResponse {
        success: bool,
        error: Option<String>,
        device: Option<PolemosDeviceInfo>,
    },
    // ── Industrial push events (scepter → shittim-chest) ──────────
    IndustrialTelemetryPush {
        reading: Option<IndustrialSensorReading>,
        #[serde(default)]
        readings: Option<Vec<IndustrialSensorReading>>,
    },
    IndustrialAlarmPush {
        event: IndustrialAlarmEvent,
    },
    IndustrialDiscoveryPush {
        session_id: String,
        phase: IndustrialDiscoveryPhase,
        message: String,
        found_devices: u64,
        progress_percent: u32,
        #[serde(default)]
        raw_findings: Option<serde_json::Value>,
    },
    IndustrialWriteApprovalPush {
        /// Matches `WriteApprovalRequest::request_id` — the operator UI
        /// must echo this back in `industrial.approveWrite` so the resolver
        /// can wake the awaiting producer. `#[serde(default)]` keeps older
        /// clients parseable.
        #[serde(default)]
        request_id: String,
        station_id: String,
        protocol: String,
        address: String,
        field_name: String,
        current_value: f64,
        proposed_value: f64,
        unit: String,
        reason: String,
        agent: String,
        risk_level: WriteApprovalRisk,
    },
    SwitchWorkspace {
        workspace_id: Uuid,
    },
    SwitchWorkspaceResponse {
        success: bool,
        workspace_id: Uuid,
        error: Option<String>,
    },
    SetClientCwd {
        path: String,
        #[serde(default)]
        device_id: Option<Uuid>,
        #[serde(default)]
        device_type: Option<String>,
    },
    PushWorkspaceFiles {
        workspace_id: Uuid,
        files: Vec<FilePayload>,
        base_path: String,
        #[serde(default)]
        batch_index: u32,
        #[serde(default)]
        batch_total: u32,
    },
    PushWorkspaceFilesAck {
        workspace_id: Uuid,
        batch_index: u32,
        accepted: u32,
        error: Option<String>,
    },
    RequestWorkspaceFiles {
        workspace_id: Uuid,
        glob_patterns: Vec<String>,
        #[serde(default)]
        exclude_patterns: Vec<String>,
    },
    WorkspaceReady {
        workspace_id: Uuid,
        container_id: Option<String>,
    },
    // ═══ Noa Workspace ═══
    RequestNoaHandshake {
        workspace_id: Uuid,
        remote_name: String,
        remote_path: String,
    },
    NoaHandshakeResponse {
        workspace_id: Uuid,
        repo_id: String,
        current_branch: String,
        #[serde(default)]
        noa_initialized: bool,
        #[serde(default)]
        gitignore_updated: bool,
    },
    NoaAuthRequest {
        workspace_id: Uuid,
        branches: Vec<String>,
        suggested_branch: String,
        reason: String,
    },
    NoaAuthResponse {
        workspace_id: Uuid,
        selected_branch: String,
        #[serde(default)]
        branch_base: Option<String>,
        #[serde(default)]
        approved: bool,
    },
    NoaReady {
        workspace_id: Uuid,
        branch: String,
        snapshot_id: String,
    },
    NoaEventSync {
        workspace_id: Uuid,
        events: Vec<NoaEvent>,
        #[serde(default)]
        direction: Option<String>,
    },
    NoaEventSyncAck {
        workspace_id: Uuid,
        last_event_id: String,
    },
    // ═══ File Browsing ═══
    // List/read files inside a container (#demiurge / #NNN), on a host, or in
    // a workspace checkout. The node-list container cards, the Bridge Network
    // cards and the workspace browser all open this same file browser.
    RequestFileTree {
        target: FileTarget,
        #[serde(default)]
        path: String,
    },
    FileTree {
        target: FileTarget,
        path: String,
        entries: Vec<FileTreeEntry>,
    },
    RequestFileRead {
        target: FileTarget,
        path: String,
    },
    FileRead {
        target: FileTarget,
        path: String,
        content: String,
        size: u64,
        #[serde(default)]
        truncated: bool,
    },
    // ═══ Bridge Network ═══
    RequestBridgeNetwork {},
    BridgeNetwork {
        hosts: Vec<HostMetrics>,
        workspaces: Vec<WorkspaceNode>,
    },
    // ═══ System / UI Control ═══
    BadgeTransition {
        previous_llm_session_id: String,
        current_llm_session_id: String,
        previous_container_id: String,
        current_container_id: String,
        transition_uuid: Uuid,
        linked_session_uuid: Option<Uuid>,
    },
    SystemMessage {
        notification: SystemNotification,
        timestamp: String,
    },
    WebUiControl {
        command: String,
    },
    WebUiControlResponse {
        command: String,
        success: bool,
        message: String,
        url: Option<String>,
    },
    WebUiStatus {
        running: bool,
        url: Option<String>,
        container_id: Option<String>,
    },
    RequestWebUiStatus,
    // ═══ Authentication ═══
    AuthLogin {
        username: String,
        password: String,
    },
    AuthLoginResponse {
        ok: bool,
        token: Option<String>,
        session_id: Option<String>,
        user_id: Option<String>,
        username: Option<String>,
        display_name: Option<String>,
        role: Option<String>,
        error: Option<String>,
    },
    AuthRegister {
        username: String,
        password: String,
        display_name: Option<String>,
    },
    AuthRegisterResponse {
        ok: bool,
        user_id: Option<String>,
        username: Option<String>,
        error: Option<String>,
    },
    AuthListUsers,
    AuthListUsersResponse {
        ok: bool,
        users: Option<Vec<AuthUserInfo>>,
        error: Option<String>,
    },
    AuthGetUser {
        user_id: String,
    },
    AuthGetUserResponse {
        ok: bool,
        user: Option<AuthUserInfo>,
        error: Option<String>,
    },
    AuthDeleteUser {
        user_id: String,
    },
    AuthDeleteUserResponse {
        ok: bool,
        error: Option<String>,
    },
    AuthChangePassword {
        user_id: String,
        old_password: String,
        new_password: String,
    },
    AuthChangePasswordResponse {
        ok: bool,
        error: Option<String>,
    },
    // ═══ Log Subscription ═══
    SubscribeContainerLogs {
        instance_uuid: String,
        tail: Option<u32>,
    },
    SubscribeContainerLogsResponse {
        ok: bool,
        error: Option<String>,
        entries: Vec<super::super::snapshot::LogEntryData>,
    },
    UnsubscribeContainerLogs {
        instance_uuid: String,
    },
    UnsubscribeContainerLogsResponse {
        ok: bool,
        error: Option<String>,
    },
    ContainerLogEntry {
        instance_uuid: String,
        entry: super::super::snapshot::LogEntryData,
    },
    SubscribeServerLogs {
        tail: Option<u32>,
    },
    SubscribeServerLogsResponse {
        ok: bool,
        error: Option<String>,
        entries: Vec<super::super::snapshot::LogEntryData>,
    },
    UnsubscribeServerLogs,
    UnsubscribeServerLogsResponse {
        ok: bool,
        error: Option<String>,
    },
    ServerLogEntry {
        entry: super::super::snapshot::LogEntryData,
    },
    // ═══ YOLO Cruise Control ═══
    YoloStart,
    YoloStartResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloStop,
    YoloStopResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloTerminate,
    YoloTerminateResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloStatus,
    YoloStatusResponse {
        active: bool,
        loop_count: u64,
        started_at: Option<String>,
        current_cycle: Option<String>,
        #[serde(default)]
        tiers: Vec<super::super::yolo::YoloTierStatus>,
    },
    YoloGetConfig,
    YoloConfigResponse {
        tiers: Vec<super::super::yolo::YoloTierConfig>,
    },
    YoloUpdateTask {
        tier: String,
        agent: String,
        skill: String,
        enabled: bool,
    },
    YoloUpdateTaskResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloSetTierInterval {
        tier: String,
        interval_secs: u64,
    },
    YoloSetTierIntervalResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloRunTierNow {
        tier: String,
    },
    YoloRunTierNowResponse {
        ok: bool,
        error: Option<String>,
    },
    YoloCycleStep {
        skill: String,
        loop_count: u64,
        status: String,
        #[serde(default)]
        token_usage: Option<(u32, u32)>,
        #[serde(default)]
        model_name: Option<String>,
    },
    YoloCycleComplete {
        loop_count: u64,
        duration_ms: u64,
    },
    SkillChainStart,
    SkillChainStep {
        skill: String,
        status: String,
    },
    SkillChainComplete {
        skill: String,
    },
    YoloTaskStart {
        tier: String,
        agent: String,
        skill: String,
    },
    YoloTaskDone {
        tier: String,
        agent: String,
        skill: String,
        duration_ms: u64,
        #[serde(default)]
        token_usage: Option<(u32, u32)>,
        #[serde(default)]
        model_name: Option<String>,
    },
    YoloTaskError {
        tier: String,
        agent: String,
        skill: String,
        error: String,
    },

    ArbiterStatus {
        instance_uuid: String,
    },
    ArbiterStatusResponse {
        ok: bool,
        status: Option<serde_json::Value>,
        error: Option<String>,
    },
    ArbiterLockdown {
        instance_uuid: String,
        delegator_id: String,
        reason: String,
    },
    ArbiterLockdownResponse {
        ok: bool,
        error: Option<String>,
    },
    ArbiterRestore {
        instance_uuid: String,
        delegator_id: String,
        target_level: String,
    },
    ArbiterRestoreResponse {
        ok: bool,
        error: Option<String>,
    },
}
