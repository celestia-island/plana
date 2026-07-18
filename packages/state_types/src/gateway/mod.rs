pub mod agent_messages;
pub mod conversation_messages;
pub mod mcp_messages;
pub mod monitor;
pub mod tui_types;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use agent_messages::{AgentMessage, NodeInfo, NodeMessage};
pub use conversation_messages::ConversationMessage;
pub use mcp_messages::{McpMessage, SkillMessage};
pub use monitor::{CosmosContainerInfo, CosmosOperationLogEntry, MetricsData, MonitorMessage};
pub use tui_types::{
    AgentPatch, AgentSnapshot, AgentUpdateParams, AuthUserInfo, ClientCapability, ClientNodeInfo,
    CompletionOutcome, ConfiguredProvider, ContainerInfo, ContainerPatch, ContainerSnapshot,
    CustomAgentInfo, EntrypointApiConfigInfo, EntrypointConfigInfo, EntrypointDefaultsInfo,
    FilePayload, GlobalSnapshot, HistoryMessage, IndustrialAlarmEvent, IndustrialAlarmHistory,
    IndustrialAlarmHistoryEntry, IndustrialAlarmLevel, IndustrialDiscoveryPhase,
    IndustrialSensorReading, KeyInfo, KeyMetadata, KnowledgeBaseFilters, KnowledgeBaseInfo,
    KnowledgeBaseStatus, Layer2AgentInfo, Layer2McpToolInfo, Layer2SkillInfo, LogEntryData,
    MaxConcurrentInfo, MessagesPage, ModelFsInfo, ModelFsPricing, ModelInfo, NoaEvent, PeriodType,
    PolemosDeviceInfo, ProviderCapabilitiesInfo, ProviderFsInfo, ProviderInfo, ProviderLimitsInfo,
    QuotaInfo, RateRuleInfo, RequestState, SearchHit, SearchResponse, TaskInfo, TaskPatch,
    TasksSnapshot, TuiAgentInfo, TuiMessage, UsagePeriodData, UserInfo,
    knowledge_base::{
        AddDocumentRequest, AddDocumentResponse, CreateKnowledgeBaseRequest,
        CreateKnowledgeBaseResponse, CreateSubscriptionRequest, CreateSubscriptionResponse,
        DeleteKnowledgeBaseResponse, DeleteSubscriptionResponse, DocumentStatus, EmbeddingModel,
        QueryKnowledgeBaseRequest, QueryKnowledgeBaseResponse, SubscriptionStatus,
        SubscriptionType, SyncSubscriptionRequest, SyncSubscriptionResponse,
    },
    message::TuiMessage as TuiGatewayMessage,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Base(BaseMessage),
    Agent(AgentMessage),
    Mcp(McpMessage),
    Skill(SkillMessage),
    Node(NodeMessage),
    Monitor(MonitorMessage),
    Tui(TuiMessage),
    Conversation(ConversationMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum BaseMessage {
    Heartbeat { timestamp: i64 },
    Error { code: String, message: String },
    Ack { message_id: Uuid },
}

/// Re-export the canonical connection-topology enum so downstream
/// entelecheia modules can reference it without each taking a direct
/// `arona` dependency at the use-site. Values: `local` (Windows-native or
/// same-host WSL2 peer, trust established by shared-secret handshake),
/// `remote_lan` (RFC1918 without that secret), `remote_internet` (else).
pub use plana::ConnectionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub direction: String,
    pub target: String,
    pub target_token: Option<String>,
    /// Topology of the link this route arrived over. Populated by evernight
    /// at session-creation time (ConnectionType::from_ip) and forwarded
    /// unchanged. entelecheia reads it for display/routing; it never
    /// influences evernight's own behaviour. Absent on legacy messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<ConnectionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetryReason {
    EmptyOutput,
    ReportNotCaptured,
    LlmError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl SkillStage {
    pub fn started(name: &str) -> Self {
        Self::Started(name.to_string())
    }

    pub fn done(name: &str) -> Self {
        Self::Done(name.to_string())
    }

    pub fn complete(name: &str) -> Self {
        Self::Complete(name.to_string())
    }

    pub fn failed(name: &str) -> Self {
        Self::Failed(name.to_string())
    }

    pub fn tool_call(name: &str) -> Self {
        Self::ToolCall(name.to_string())
    }

    pub fn retrying(
        name: &str,
        attempt: usize,
        max_retries: usize,
        reason: Option<RetryReason>,
    ) -> Self {
        Self::Retrying(name.to_string(), attempt, max_retries, reason)
    }

    pub fn trying_model(skill_name: &str, model_name: &str) -> Self {
        Self::TryingModel(skill_name.to_string(), model_name.to_string())
    }

    pub fn model_failed(skill_name: &str, model_name: &str, error: &str) -> Self {
        Self::ModelFailed(
            skill_name.to_string(),
            model_name.to_string(),
            error.to_string(),
        )
    }

    pub fn nudging(skill_name: &str) -> Self {
        Self::Nudging(skill_name.to_string())
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Started(n)
            | Self::Done(n)
            | Self::Complete(n)
            | Self::Failed(n)
            | Self::ToolCall(n)
            | Self::Retrying(n, ..)
            | Self::TryingModel(n, _)
            | Self::ModelFailed(n, _, _)
            | Self::Nudging(n) => Some(n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AskAnswerSource {
    Human,
    Auto,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Selection semantics for an inquiry (`Query`) report's `preset_options`.
/// Mirrors `arona::ReportSelection`. Defaults to `Single` when omitted.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportSelection {
    #[default]
    Single,
    Multiple,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    AlarmTriggered {
        station: u8,
        register: String,
        level: String,
        value: String,
        topic: String,
    },
}

impl SystemNotification {
    pub fn i18n_key(&self) -> &str {
        match self {
            Self::WebUiStarted => "system.webui_started",
            Self::WebUiStopped => "system.webui_stopped",
            Self::WebUiRestarted => "system.webui_restarted",
            Self::WebUiError { .. } => "system.webui_error",
            Self::WebUiUrl { .. } => "system.webui_url",
            Self::ContainerError { .. } => "system.container_error",
            Self::CosmosError { .. } => "system.cosmos_error",
            Self::ServerError { .. } => "system.server_error",
            Self::AutoModeChanged { .. } => "system.auto_mode_changed",
            Self::AutoModeUsage => "system.auto_mode_usage",
            Self::WorkspaceOpened { .. } => "system.workspace_opened",
            Self::WorkspaceError { .. } => "system.workspace_error",
            Self::Generic { key, .. } => key,
            Self::SecurityPolicyChanged { .. } => "system.security_policy_changed",
            Self::SecurityToolBlocked { .. } => "system.security_tool_blocked",
            Self::AlarmTriggered { .. } => "system.alarm_triggered",
        }
    }

    pub fn i18n_params(&self) -> Vec<String> {
        match self {
            Self::WebUiStarted
            | Self::WebUiStopped
            | Self::WebUiRestarted
            | Self::AutoModeUsage => vec![],
            Self::WebUiError { error }
            | Self::ServerError { error }
            | Self::WorkspaceError { error } => vec![error.clone()],
            Self::WebUiUrl { url } => vec![url.clone()],
            Self::ContainerError { container, error } => vec![container.clone(), error.clone()],
            Self::CosmosError { agent, error } => vec![agent.clone(), error.clone()],
            Self::AutoModeChanged {
                enabled,
                timeout_secs,
            } => {
                let mut p = vec![if *enabled {
                    "on".to_string()
                } else {
                    "off".to_string()
                }];
                if let Some(t) = timeout_secs {
                    p.push("with_timeout".to_string());
                    p.push(t.to_string());
                }
                p
            }
            Self::WorkspaceOpened { repo_url, branch } => {
                let mut p = vec![repo_url.clone()];
                if let Some(b) = branch {
                    p.push(b.clone());
                }
                p
            }
            Self::Generic { params, .. } => params.clone(),
            Self::SecurityPolicyChanged {
                action,
                details,
                changed_by,
            } => {
                vec![action.clone(), details.clone(), changed_by.clone()]
            }
            Self::SecurityToolBlocked {
                agent,
                tool,
                reason,
            } => {
                vec![agent.clone(), tool.clone(), reason.clone()]
            }
            Self::AlarmTriggered {
                station,
                register,
                level,
                value,
                topic,
            } => {
                vec![
                    station.to_string(),
                    register.clone(),
                    level.clone(),
                    value.clone(),
                    topic.clone(),
                ]
            }
        }
    }
}
