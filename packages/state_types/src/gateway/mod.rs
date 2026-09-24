//! Gateway wire DTOs for the agent control plane (the `plana_state_sync`
//! message catalog).
//!
//! `Message` is the only envelope: an adjacently tagged
//! `{"type": ..., "data": ...}` frame whose `type` selects one of the action
//! enums below and whose `data` is that enum serde-tagged payload. Through
//! the JSON-RPC bridge the two tags become the method string
//! `{type}.{action}`.
//!
//! The payloads are pure data — no field here triggers behavior — and optional
//! fields are `Option`, which a missing key deserializes to `None`, so a peer
//! that predates a field still parses.

/// Agent-registry and node-discovery action envelopes (`Agent.*`, `Node.*`).
pub mod agent_messages;
/// Agent-to-agent consultation envelopes (`Conversation.*`) about one file.
pub mod conversation_messages;
/// Metrics-sampling envelopes and their payloads (`Monitor.*`).
pub mod monitor;
/// Tool and skill invocation and listing envelopes (`Tool.*`, `Skill.*`).
pub mod tool_messages;
/// The dashboard-facing payload catalog, including the large `SyncMessage`
/// action enum re-exported at the bottom of this module.
pub mod tui_types;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use agent_messages::{AgentMessage, NodeInfo, NodeMessage};
pub use conversation_messages::ConversationMessage;
pub use monitor::{CosmosContainerInfo, CosmosOperationLogEntry, MetricsData, MonitorMessage};
pub use tool_messages::{SkillMessage, ToolMessage};
pub use tui_types::{
    ActorClaims, AgentPatch, AgentSnapshot, AgentUpdateParams, AuthUserInfo, ClientCapability,
    ClientNodeInfo, CompletionOutcome, ConfiguredProvider, ContainerInfo, ContainerPatch,
    ContainerSnapshot, CustomAgentInfo, EntrypointApiConfigInfo, EntrypointConfigInfo,
    EntrypointDefaultsInfo, FilePayload, GlobalSnapshot, HistoryMessage, IndustrialAlarmEvent,
    IndustrialAlarmHistory, IndustrialAlarmHistoryEntry, IndustrialAlarmLevel,
    IndustrialDiscoveryPhase, IndustrialSensorReading, KeyInfo, KeyMetadata, KnowledgeBaseFilters,
    KnowledgeBaseInfo, KnowledgeBaseStatus, Layer2AgentInfo, Layer2SkillInfo, Layer2ToolInfo,
    LogEntryData, MaxConcurrentInfo, MessagesPage, ModelFsInfo, ModelFsPricing, ModelInfo,
    NoaEvent, PeriodType, PolemosDeviceInfo, ProviderCapabilitiesInfo, ProviderFsInfo,
    ProviderInfo, ProviderLimitsInfo, QuotaInfo, RateRuleInfo, RequestState, SearchHit,
    SearchResponse, SyncMessage, TaskInfo, TaskPatch, TasksSnapshot, TuiAgentInfo, UsagePeriodData,
    UserInfo,
    knowledge_base::{
        AddDocumentRequest, AddDocumentResponse, CreateKnowledgeBaseRequest,
        CreateKnowledgeBaseResponse, CreateSubscriptionRequest, CreateSubscriptionResponse,
        DeleteKnowledgeBaseResponse, DeleteSubscriptionResponse, DocumentStatus, EmbeddingModel,
        QueryKnowledgeBaseRequest, QueryKnowledgeBaseResponse, SubscriptionStatus,
        SubscriptionType, SyncSubscriptionRequest, SyncSubscriptionResponse,
    },
    message::SyncMessage as SyncGatewayMessage,
};

/// Gateway message envelope. The `Sync` payload is boxed because
/// `SyncMessage` is by far the largest variant (>= 488 bytes inline), which
/// would otherwise inflate every `Message` value on the hot dispatch path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    /// Wire `type` = `Base`: liveness and transport notices.
    Base(BaseMessage),
    /// Wire `type` = `Agent`: agent-registry actions; the bridge has no constant
    /// and no parse arm for this namespace.
    Agent(AgentMessage),
    /// Wire `type` = `Tool`: tool catalog and invocation.
    Tool(ToolMessage),
    /// Wire `type` = `Skill`: skill catalog and invocation.
    Skill(SkillMessage),
    /// Wire `type` = `Node`: peer-node discovery; also absent from the bridge
    /// catalog.
    Node(NodeMessage),
    /// Wire `type` = `Monitor`: metrics sampling; absent from the bridge catalog
    /// as well.
    Monitor(MonitorMessage),
    /// Wire `type` = `Sync`: the state-sync dialect. Its `action` tag is the
    /// second half of the wire method (`Sync.Ping`, `Sync.AgentPatch`, ...).
    Sync(Box<SyncMessage>),
    /// Wire `type` = `Conversation`: consultations between agents.
    Conversation(ConversationMessage),
}

/// Payloads of the `Base` namespace: keepalive and transport-level notices.
///
/// Wire shape: `{"type": "Base", "data": {"action": "Heartbeat", ...}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum BaseMessage {
    /// `Base.Heartbeat` — keepalive; `timestamp` is the sender clock reading.
    Heartbeat { timestamp: i64 },
    /// `Base.Error` — a notice, not an answer: `code` is the machine-readable
    /// identifier and `message` the human text. Nothing replies to it.
    Error { code: String, message: String },
    /// `Base.Ack` — acknowledges one earlier message by `message_id`. The
    /// envelope carries no id of its own, so the id comes from the original
    /// frame, not from this payload.
    Ack { message_id: Uuid },
}

/// Re-export the canonical connection-topology enum so downstream
/// entelecheia modules can reference it without each taking a direct
/// `arona` dependency at the use-site. Values: `local` (Windows-native or
/// same-host WSL2 peer, trust established by shared-secret handshake),
/// `remote_lan` (RFC1918 without that secret), `remote_internet` (else).
pub use plana::ConnectionType;

/// Routing descriptor for a report, attached as `AgentReport.next_route`.
///
/// Written by the hop that handled the turn and forwarded verbatim; a peer
/// must not infer routing decisions from it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// Hop direction label as the routing layer wrote it, stored verbatim.
    pub direction: String,
    /// Routing target this report is addressed to.
    pub target: String,
    /// Token to present to that target. `None` — also the value of a missing
    /// key — means the route needs none.
    pub target_token: Option<String>,
    /// Topology of the link this route arrived over. Populated by evernight
    /// at session-creation time (ConnectionType::from_ip) and forwarded
    /// unchanged. entelecheia reads it for display/routing; it never
    /// influences evernight's own behaviour. Absent on legacy messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<ConnectionType>,
}

/// Why a skill attempt was retried. Carried as the fourth element of
/// `SkillStage::Retrying`, and recorded on the timeline group of the skill so
/// a later reader can still explain the retry once the stage itself is gone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetryReason {
    /// The attempt returned no output at all — distinct from `ReportNotCaptured`,
    /// which is output that produced no report.
    EmptyOutput,
    /// The attempt produced output, but the chain captured no report from it and
    /// therefore had no result to continue from.
    ReportNotCaptured,
    /// The model call itself failed; `message` is the error text of that attempt.
    LlmError { message: String },
}

/// One stage of a running skill chain, sent as the `stage` field of
/// `Sync.OrchestrationStatus`.
///
/// The first tuple element of every variant is the skill name; later elements
/// add attempt and model detail. Stages are progress only — the outcome a
/// client acts on arrives as a `Sync.AgentReport`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillStage {
    /// The chain is about to call the skill.
    Started(String),
    /// The skill call has returned; the chain still has to capture its report.
    Done(String),
    /// Final stage of this skill contribution to the chain.
    Complete(String),
    /// The skill call ended in failure; the chain decides what to do next.
    Failed(String),
    /// The skill issued a tool call that is now in flight.
    ToolCall(String),
    /// The skill is being re-run: attempt number, retry ceiling, and why. The
    /// reason is optional because a producer may retry without recording one.
    Retrying(String, usize, usize, Option<RetryReason>),
    /// Retry against another model; the second element names that model.
    TryingModel(String, String),
    /// The model tried in `TryingModel` failed; the third element is its error.
    ModelFailed(String, String, String),
    /// A reminder was sent to the skill because its report was still missing.
    Nudging(String),
}

impl SkillStage {
    /// Builds `Started`, copying the borrowed skill name into the payload so the
    /// stage can outlive the caller.
    pub fn started(name: &str) -> Self {
        Self::Started(name.to_string())
    }

    /// Builds `Done` (skill call returned), copying the skill name.
    pub fn done(name: &str) -> Self {
        Self::Done(name.to_string())
    }

    /// Builds `Complete`, the last stage of one skill contribution.
    pub fn complete(name: &str) -> Self {
        Self::Complete(name.to_string())
    }

    /// Builds `Failed`, copying the skill name of the failed call.
    pub fn failed(name: &str) -> Self {
        Self::Failed(name.to_string())
    }

    /// Builds `ToolCall`; the name is the skill that issued the call, not the
    /// tool.
    pub fn tool_call(name: &str) -> Self {
        Self::ToolCall(name.to_string())
    }

    /// Builds `Retrying` from the current attempt number, the configured ceiling
    /// and the trigger, if the producer recorded one.
    pub fn retrying(
        name: &str,
        attempt: usize,
        max_retries: usize,
        reason: Option<RetryReason>,
    ) -> Self {
        Self::Retrying(name.to_string(), attempt, max_retries, reason)
    }

    /// Builds `TryingModel`: the skill and the alternative model about to be
    /// attempted.
    pub fn trying_model(skill_name: &str, model_name: &str) -> Self {
        Self::TryingModel(skill_name.to_string(), model_name.to_string())
    }

    /// Builds `ModelFailed` for the model that just failed, with its error text.
    pub fn model_failed(skill_name: &str, model_name: &str, error: &str) -> Self {
        Self::ModelFailed(
            skill_name.to_string(),
            model_name.to_string(),
            error.to_string(),
        )
    }

    /// Builds `Nudging` for a skill whose report has not arrived yet.
    pub fn nudging(skill_name: &str) -> Self {
        Self::Nudging(skill_name.to_string())
    }

    /// Skill name carried by the stage. Every variant currently matches, so the
    /// result is never `None`; a variant without a name would have to change this
    /// signature.
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

/// Who produced the answer recorded in an `AskHumanReply`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AskAnswerSource {
    /// A person answered, as opposed to the auto-mode responder or the deadline
    /// path below.
    Human,
    /// The server auto-mode responder answered instead of a person; see
    /// `SystemNotification::AutoModeChanged`.
    Auto,
    /// No answer arrived before the deadline, so the reply holds whatever the
    /// timeout path recorded for `selected_options` and `custom_answer`.
    Timeout,
}

/// What kind of report one `Sync.AgentReport` is, which decides how a client
/// treats it. Serialized `snake_case` (`"query"`, `"skill_failed"`, ...).
///
/// Three groups matter to consumers: `Query` is an inquiry answered with
/// `Sync.AgentReportReply`; `Error`, `ChainMaxDepth`, `ChainCycle`,
/// `SkillFailed`, `SkillEmptyOutput` and `SkillMissingReport` describe a
/// failure; `Reply`, `SkillTerminal`, `Error`, `System` and
/// `NextActionFallback` end the turn. The sibling profile copy
/// (`plana-celestia-types`) spells these groups out as `is_query`, `is_error`,
/// `is_pending` and `is_terminal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// The agent needs an answer: `preset_options` are shown as an inquiry and the
    /// reply comes back as `Sync.AgentReportReply`. Also the only type that reads
    /// `selection_mode` and `allow_custom_reply`.
    Query,
    /// A human-origin report; the profile helpers classify it as neither an error
    /// nor a terminal report.
    Human,
    /// The reply to the current user turn; terminal.
    Reply,
    /// A skill final report for the turn; terminal.
    SkillTerminal,
    /// Progress from inside a skill; not terminal.
    SkillStep,
    /// Terminal fallback used when the chain could not pick a next action.
    NextActionFallback,
    /// Failure: the chain reached its depth limit.
    ChainMaxDepth,
    /// Failure: the chain revisited a skill and was cut off.
    ChainCycle,
    /// Failure: a skill ended in error.
    SkillFailed,
    /// Failure: a skill finished without producing output.
    SkillEmptyOutput,
    /// Failure: a skill finished without producing its required report.
    SkillMissingReport,
    /// Failure: a generic chain or server error report.
    Error,
    /// A system-authored report rather than an agent turn; terminal.
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
    /// Exactly one of `preset_options` may be picked; the default.
    Single,
    /// Any number of `preset_options` may be picked.
    Multiple,
}

/// One server-side notification, carried as `Sync.SystemMessage.notification`
/// and published on the `system_notification` push topic. Serialized
/// `snake_case`.
///
/// The client renders it: `i18n_key` names the translation entry and
/// `i18n_params` supplies its positional arguments, so a notification carries
/// no display text of its own.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SystemNotification {
    /// The embedded Web UI came up; no params.
    WebUiStarted,
    /// The Web UI stopped; no params.
    WebUiStopped,
    /// The Web UI was restarted; no params.
    WebUiRestarted,
    /// The Web UI failed; `error` is the only param.
    WebUiError {
        /// Failure text from the Web UI process, passed through verbatim for
        /// the client to display.
        error: String,
    },
    /// The Web UI is reachable; `url` is the only param.
    WebUiUrl {
        /// URL the Web UI became reachable at, as the client should open it.
        url: String,
    },
    /// A container failed; params are the container name and the error text.
    ContainerError {
        /// Name of the container that failed.
        container: String,
        /// Failure text from the container runtime, passed through verbatim.
        error: String,
    },
    /// A cosmos (agent sandbox) failed; params are the agent and the error text.
    CosmosError {
        /// Agent whose cosmos sandbox failed.
        agent: String,
        /// Failure text from the sandbox, passed through verbatim.
        error: String,
    },
    /// Server-level failure; `error` is the only param.
    ServerError {
        /// Server-level failure text, passed through verbatim.
        error: String,
    },
    /// Auto mode was toggled. Params are `"on"`/`"off"`, then `"with_timeout"`
    /// plus the seconds only when `timeout_secs` is set — `None` means the mode
    /// runs without a deadline.
    AutoModeChanged {
        enabled: bool,
        timeout_secs: Option<u64>,
    },
    /// Auto mode handled a turn; no params.
    AutoModeUsage,
    /// A workspace was opened. Params are `repo_url`, then `branch` only when it
    /// is set.
    WorkspaceOpened {
        repo_url: String,
        branch: Option<String>,
    },
    /// Opening a workspace failed; `error` is the only param.
    WorkspaceError {
        /// Why the workspace could not be opened, passed through verbatim.
        error: String,
    },
    /// Caller-defined notification: `key` is used verbatim as the i18n key, so it
    /// must exist in the client catalog, and `params` are its arguments.
    Generic {
        /// i18n key, used verbatim — it must exist in the client catalog.
        key: String,
        /// Positional arguments substituted into the localized string.
        params: Vec<String>,
    },
    /// A security policy changed; params are the action, the details and the user
    /// who made the change, in that order.
    SecurityPolicyChanged {
        action: String,
        details: String,
        changed_by: String,
    },
    /// A tool run was denied; params are the agent, the tool and the reason.
    SecurityToolBlocked {
        agent: String,
        tool: String,
        reason: String,
    },
    /// An industrial alarm fired; params are station, register, level, value and
    /// topic, with the station rendered as a decimal string.
    AlarmTriggered {
        station: u8,
        register: String,
        level: String,
        value: String,
        topic: String,
    },
    /// A Modbus write attempt completed (M5 write-closure audit). Emitted by
    /// scepter when `industrial_iot.modbus_write` returns, regardless of the
    /// write result — blocked writes are reported as failures.
    ModbusWriteAudit {
        /// Target station id.
        station: String,
        /// Target endpoint (host:port or serial device).
        endpoint: String,
        /// Number of write operations attempted.
        writes_attempted: u64,
        /// Whether every write was verified by a read-back.
        all_confirmed: bool,
        /// The agent that issued the write.
        agent: String,
        /// Compact per-write outcome summary (JSON).
        summary: String,
    },
}

impl SystemNotification {
    /// Translation key of the notification (`system.webui_started`, ...).
    /// `Generic` returns its own `key` unchanged, so a caller-defined
    /// notification brings the key with it.
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
            Self::ModbusWriteAudit { .. } => "system.modbus_write_audit",
        }
    }

    /// Positional arguments for that key, in template order. The count is not
    /// fixed per variant: `AutoModeChanged` and `WorkspaceOpened` append their
    /// optional element only when it is present.
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
            Self::ModbusWriteAudit {
                station,
                endpoint,
                writes_attempted,
                all_confirmed,
                agent,
                summary,
            } => {
                vec![
                    station.clone(),
                    endpoint.clone(),
                    writes_attempted.to_string(),
                    all_confirmed.to_string(),
                    agent.clone(),
                    summary.clone(),
                ]
            }
        }
    }
}
