//! Canonical `SyncMessage` wire enum of the TUI/gateway sync protocol and
//! the payload structs it carries.
//!
//! `SyncMessage` is internally tagged (`action`) and rides inside the gateway
//! envelope `Message::Sync`, so one frame is
//! `{"type": "Sync", "data": {"action": "<VariantName>", …}}`; the JSON-RPC
//! bridge additionally addresses each variant as method `Sync.<VariantName>`.
//! Field names stay snake_case end-to-end so the shittim-chest webui mirrors
//! in `plana-celestia-types` (and their generated TS bindings) need no
//! remapping. Request/response pairing and one-way-ness are declared
//! centrally in `plana::jsonrpc::pending`.

/// Domain-grouped index of `SyncMessage` variant names, compiled for tests
/// only; it documents which variants belong to which functional area.
#[cfg(test)]
pub mod groups;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::super::{
    AskAnswerSource, ReportSelection, ReportType, RouteInfo, SkillStage, SystemNotification,
    monitor::{CosmosContainerInfo, CosmosOperationLogEntry},
};
use crate::{agent::Agent, agent_error::StructuredAgentError};
use plana_config::GenProtocol;
use plana_core::AgentBadge;
use plana_text::{LlmStream, StreamChunkKind};

fn default_search_limit() -> u64 {
    10
}

fn default_history_limit() -> u64 {
    50
}

/// Deserialize a wire value that may arrive either as a JSON string or as a
/// JSON number into its string form.
///
/// Wire-compat shim for `Sync.UserMessage`: shittim-chest has always
/// serialized `timestamp` as epoch-millis (i64) while this enum declared the
/// field as `String`. A strict serde parse rejects the integer and the whole
/// message is then silently dropped downstream, so the receiver now accepts
/// both shapes. Serialization still emits a `String`, keeping the existing
/// TS bindings and legacy peers unchanged.
fn deserialize_string_or_number<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<String, D::Error> {
    struct StringOrNumberVisitor;

    impl<'de> serde::de::Visitor<'de> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            Ok(value.to_owned())
        }

        fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<String, E> {
            Ok(value.to_string())
        }

        fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<String, E> {
            Ok(value.to_string())
        }
    }

    d.deserialize_any(StringOrNumberVisitor)
}

/// A bridge ("Polemos") host registered with the gateway: a machine that can
/// serve workspaces and be browsed over the file browser. scepter pushes the
/// roster as `PolemosDeviceList`; the client store keys entries by `node_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolemosDeviceInfo {
    /// Device uuid; the client state tree keys the roster under `state.devices.<node_id>`.
    pub node_id: Uuid,
    /// Display label for the host, shown in the Bridge Network host list.
    pub name: String,
    /// Host address the gateway reaches the device on (host name or literal address).
    pub address: String,
    /// Liveness string reported by the device; observed values: "online", "offline", "busy".
    pub status: String,
    /// Default workspace root on that host when advertised; `None` = not reported.
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
// the existing SyncMessage variants); the webui's TS mirrors use the
// same shape so no remapping is required.

/// Severity ordering matches ISA-18.2 alarm severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialAlarmLevel {
    /// Journal entry that is not an alarm (wire tag "Log").
    Log,
    /// Low-low alarm level; its configured threshold is `IndustrialAlarmThresholds::ll`.
    LowLow,
    /// Low alarm level; its configured threshold is `IndustrialAlarmThresholds::l`.
    Low,
    /// High alarm level; its configured threshold is `IndustrialAlarmThresholds::h`.
    High,
    /// High-high alarm level; its configured threshold is `IndustrialAlarmThresholds::hh`.
    HighHigh,
    /// Trip on rate of change rather than on an absolute level (wire tag "RateOfChange").
    RateOfChange,
    /// Highest declared severity, above the threshold levels (wire tag "Emergency").
    Emergency,
}

/// A single live reading from an industrial field (e.g. pressure cell on
/// a Modbus register, S7 DBX bit). Pushed by scepter at the scan cycle
/// of the underlying transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialSensorReading {
    /// Id of the station the point belongs to; matches `IndustrialStationInfo::station_id`.
    pub station_id: String,
    /// Transport that produced the reading (producer vocabulary, e.g. "modbus_rtu").
    pub protocol: String,
    /// Producer-side register/bit address, e.g. "HR:16" or "DB1.DBW4".
    pub address: String,
    /// Station-local point name; with `protocol` + `station_id` it forms the ingest key.
    pub name: String,
    /// Value exactly as read from the wire, before scaling.
    pub raw_value: f64,
    /// Engineering value after the station scaling is applied.
    pub scaled_value: f64,
    /// Unit symbol of `scaled_value`, e.g. "MPa".
    pub unit: String,
    /// Transport quality flag; world-state ingestion maps it onto its own `Quality`.
    pub quality: String,
    /// Producer event time (RFC 3339); ingestion uses it as the entity wall clock.
    pub timestamp: String,
}

/// Fired on threshold breach (breached=true) or clear (breached=false).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmEvent {
    /// Id of the station whose point raised the event.
    pub station_id: String,
    /// Transport the point was read over; same vocabulary as `IndustrialSensorReading`.
    pub protocol: String,
    /// Register/bit address that changed state, e.g. "HR:16".
    pub address: String,
    /// Station-local point name; with `protocol` + `address` it identifies the point.
    pub field_name: String,
    /// Severity of the transition, ordered per ISA-18.2.
    pub level: IndustrialAlarmLevel,
    /// Reading that triggered the event, in engineering units.
    pub value: f64,
    /// Configured threshold the reading crossed, in the same unit as `value`.
    pub threshold: f64,
    /// Unit shared by `value` and `threshold`.
    pub unit: String,
    /// `true` on breach, `false` on clear; the same point re-emits with `false` when in range.
    pub breached: bool,
    /// Wall-clock time of the transition (RFC 3339).
    pub timestamp: String,
}

/// Phases of an evernight discovery scan. Ordered by typical progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialDiscoveryPhase {
    /// Probing transports/ports for live devices.
    TransportScan,
    /// Identifying which protocol each responding device speaks.
    ProtocolIdentify,
    /// Enumerating the device data model (registers, DB blocks, tags).
    DataModelScan,
    /// Inferring field semantics (names, units, roles) for the scanned points.
    SemanticInference,
    /// Building the device manifest from the scan result.
    ManifestGeneration,
    /// Validating the generated manifest before it is offered for import.
    ManifestValidation,
    /// Terminal phase; the scan finished and produced its final manifest.
    Complete,
}

/// Progress event of one evernight discovery scan, pushed to the operator UI as
/// the scan advances through `IndustrialDiscoveryPhase`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialDiscoveryProgress {
    /// Producer-assigned scan id; ties every progress event of one scan together.
    pub session_id: String,
    /// Phase the scan is currently in.
    pub phase: IndustrialDiscoveryPhase,
    /// Human-readable detail line for the current phase.
    pub message: String,
    /// Devices found so far in this scan.
    pub found_devices: u64,
    /// Completion in percent (0-100) over the whole scan.
    pub progress_percent: u32,
    /// Raw findings of the current phase for debugging; absent = not published.
    #[serde(default)]
    pub raw_findings: Option<serde_json::Value>,
}

/// Operator confirmation gate for safety-critical PLC writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteApprovalRisk {
    /// Low-risk setpoint change (wire tag "safe").
    Safe,
    /// Change needing operator attention before it goes through (wire tag "caution").
    Caution,
    /// Safety-critical write; the operator gate must confirm it (wire tag "critical").
    Critical,
}

/// A pending confirmation for a safety-critical PLC write, pushed to the operator
/// UI; the UI answers via `industrial.approveWrite` with the same `request_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteApprovalRequest {
    /// Unique id assigned by the producer (orexis). The operator UI echoes
    /// it back in `industrial.approveWrite` so scepter's resolver can match
    /// the response to the pending oneshot (Phase D.2, A.2.4.1→A.2.4.2).
    /// `#[serde(default)]` keeps the wire format backward-compatible with
    /// older push events that predate this field.
    #[serde(default)]
    pub request_id: String,
    /// Id of the station holding the target field.
    pub station_id: String,
    /// Transport of the target field; same vocabulary as `IndustrialSensorReading`.
    pub protocol: String,
    /// Register/bit address to be written, e.g. "HR:16".
    pub address: String,
    /// Station-local name of the field being written.
    pub field_name: String,
    /// Value read when the approval was requested, in engineering units.
    pub current_value: f64,
    /// Value the agent proposes to write, in the same unit as `current_value`.
    pub proposed_value: f64,
    /// Unit shared by `current_value` and `proposed_value`.
    pub unit: String,
    /// Agent-authored justification shown to the operator.
    pub reason: String,
    /// Agent that requested the write (free-form agent name/id).
    pub agent: String,
    /// Risk class that decides how loudly the operator is asked.
    pub risk_level: WriteApprovalRisk,
}

/// One addressable field of a station in the topology snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialStationField {
    /// Register/bit address of the field, e.g. "HR:16".
    pub address: String,
    /// Station-local field name; becomes the point name on ingestion.
    pub name: String,
    /// Data type the device declares (producer vocabulary, not normalized).
    pub data_type: String,
    /// Engineering unit when the device declares one; absent = unitless or unknown.
    #[serde(default)]
    pub unit: Option<String>,
    /// Alarm thresholds configured for this field; absent = none configured.
    #[serde(default)]
    pub alarm: Option<IndustrialAlarmThresholds>,
    /// Latest value known to the producer; absent = the field was never read.
    #[serde(default)]
    pub current_value: Option<f64>,
}

/// Alarm thresholds of one field, in the field unit. Every level is optional and
/// an unset level can never trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmThresholds {
    /// Low-low threshold; breaches report as `IndustrialAlarmLevel::LowLow`.
    #[serde(default)]
    pub ll: Option<f64>,
    /// Low threshold; breaches report as `IndustrialAlarmLevel::Low`.
    #[serde(default)]
    pub l: Option<f64>,
    /// High threshold; breaches report as `IndustrialAlarmLevel::High`.
    #[serde(default)]
    pub h: Option<f64>,
    /// High-high threshold; breaches report as `IndustrialAlarmLevel::HighHigh`.
    #[serde(default)]
    pub hh: Option<f64>,
}

/// One station of the industrial topology, served by the `topology.*` /
/// `industrial.*` JSON-RPC family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialStationInfo {
    /// Stable station id; joins telemetry, alarms, history and topology entries.
    pub station_id: String,
    /// Transport the station is reached over (producer vocabulary).
    pub protocol: String,
    /// Connection descriptor the station was configured with (endpoint or bus string).
    pub connection: String,
    /// Device class reported by discovery; selects the applicable field schema.
    pub device_class: String,
    /// Vendor reported by discovery; absent = unknown.
    #[serde(default)]
    pub vendor: Option<String>,
    /// Model reported by discovery; absent = unknown.
    #[serde(default)]
    pub model: Option<String>,
    /// Firmware/version reported by discovery; absent = unknown.
    #[serde(default)]
    pub firmware: Option<String>,
    /// Connection/health state of the station (producer vocabulary).
    pub status: String,
    /// Field inventory; empty when the station has not been scanned yet.
    #[serde(default)]
    pub fields: Vec<IndustrialStationField>,
}

/// One entry in the historical alarm log (last N days).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmHistoryEntry {
    /// Station the logged alarm belongs to.
    pub station_id: String,
    /// Transport the point was read over.
    pub protocol: String,
    /// Register/bit address of the point, e.g. "HR:16".
    pub address: String,
    /// Station-local point name.
    pub field_name: String,
    /// Severity the entry was logged with.
    pub level: IndustrialAlarmLevel,
    /// Reading at the time of the transition, in engineering units.
    pub value: f64,
    /// Threshold that was crossed, in the same unit as `value`.
    pub threshold: f64,
    /// Unit shared by `value` and `threshold`.
    pub unit: String,
    /// `true` for a breach entry, `false` for a clear entry.
    pub breached: bool,
    /// Wall-clock time of the transition (RFC 3339).
    pub timestamp: String,
    /// Whether an operator acknowledged the alarm, and when.
    #[serde(default)]
    pub acknowledged: bool,
    /// Acknowledgement time (RFC 3339); absent while `acknowledged` is false.
    #[serde(default)]
    pub acknowledged_at: Option<String>,
    /// Operator identity that acknowledged; absent while unacknowledged.
    #[serde(default)]
    pub acknowledged_by: Option<String>,
}

/// Historical alarm log slice plus the pre-paging total.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialAlarmHistory {
    /// Entries in this page.
    pub entries: Vec<IndustrialAlarmHistoryEntry>,
    /// Total matching entries before paging.
    pub total: u64,
}

/// Capability a client declares in `ConnectHandshake`, so scepter's client
/// node registry only routes capability-scoped work to sessions that have it
/// (NOA handshakes, for example, go only to `NoaWorkspace` clients).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    /// Client can relay files over the bridge (wire tag "file_relay").
    FileRelay,
    /// Client can run or attach a terminal (wire tag "terminal").
    Terminal,
    /// Client can capture its screen (wire tag "screen_capture").
    ScreenCapture,
    /// Client can drive a NOA workspace (wire tag "noa_workspace").
    NoaWorkspace,
}

/// Self-description a client attaches to `ConnectHandshake` so the gateway can
/// register the session node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientNodeInfo {
    /// Machine host name of the connecting client.
    pub hostname: String,
    /// OS description string (producer vocabulary, e.g. "linux").
    pub os: String,
    /// Default workspace root on the client; absent = not advertised.
    #[serde(default)]
    pub workspace_root: Option<String>,
    /// Authenticated user the session belongs to; absent on pre-auth handshakes.
    #[serde(default)]
    pub user_id: Option<Uuid>,
}

/// One file carried inside a `PushWorkspaceFiles` batch; paths are relative to
/// that message's `base_path`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePayload {
    /// Path relative to the batch `base_path`.
    pub relative_path: String,
    /// File body, transferred inline in the message.
    pub content: String,
    /// Size of `content` in bytes.
    pub size: u64,
    /// Source modification time as formatted by the sender; absent = not reported.
    #[serde(default)]
    pub last_modified: Option<String>,
}

/// One NOA event exchanged in the bidirectional `NoaEventSync` pair that runs
/// after `NoaReady`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoaEvent {
    /// Producer event id; `NoaEventSyncAck::last_event_id` acknowledges up to it.
    pub event_id: String,
    /// Producer-defined event kind (e.g. the file-change classification).
    pub event_type: String,
    /// Wall-clock time the event was produced.
    pub timestamp: String,
    /// Path the event concerns, for file-oriented events; absent otherwise.
    pub file_path: Option<String>,
    /// Content hash at event time when the producer computes one; absent otherwise.
    pub content_hash: Option<String>,
    /// Free-form producer metadata; absent or `null` when the event carries none.
    pub metadata: Option<serde_json::Value>,
}

// ═══ File Browsing ═══
// Mirrors arona's `FileTarget` / `FileTreeEntry` / file-browse params. Used by
// the node-list container cards, the Bridge Network host/workspace cards, and
// the workspace file browser to list/read files inside a container, on a host,
// or in a workspace checkout.

/// Which filesystem a file-browsing operation targets; it decides how
/// `FileTarget::id` is read (wire tags are snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileTargetKind {
    /// Container of the agent runtime, addressed by its badge (`#demiurge`, `#001`).
    Container,
    /// Host machine, addressed by its host id.
    Host,
    /// Workspace checkout, addressed by its workspace id.
    Workspace,
}

/// A file-operation target: the filesystem kind plus the id that names a member
/// of that kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTarget {
    /// Which filesystem `id` addresses.
    pub kind: FileTargetKind,
    /// Container badge (`#demiurge` / `#001`), host id, or workspace id.
    pub id: String,
    /// Workspace the container slot belongs to (container slots are per-workspace);
    /// absent = target is not workspace-scoped.
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

/// One entry of a directory listing returned by `FileTree`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeEntry {
    /// Base name of the entry, without path separators.
    pub name: String,
    /// `"file"` | `"dir"` | `"symlink"`.
    pub kind: String,
    /// Size in bytes; directory entries carry the size the filesystem reports.
    pub size: u64,
}

// ═══ Bridge Network ═══
// Mirrors arona's host / workspace-node / git-status / token-usage structs.

/// Live performance snapshot of one host machine, shown in the Bridge Network
/// page next to its workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    /// Host id; joins `WorkspaceNode::host_id` and addresses the file browser target.
    pub host_id: String,
    /// Display host name.
    pub hostname: String,
    /// OS description string reported by the host.
    pub os: String,
    /// CPU utilization at sample time, in percent (0-100).
    pub cpu_usage_percent: f64,
    /// Number of logical CPU cores.
    pub cpu_cores: u32,
    /// Memory in use, in bytes.
    pub mem_used_bytes: u64,
    /// Installed memory, in bytes.
    pub mem_total_bytes: u64,
    /// Uplink rate in bits per second; absent = not sampled.
    #[serde(default)]
    pub net_up_bps: Option<u64>,
    /// Downlink rate in bits per second; absent = not sampled.
    #[serde(default)]
    pub net_down_bps: Option<u64>,
}

/// Git summary of one workspace checkout, as shown on its workspace card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceGitStatus {
    /// Name of the checked-out branch.
    pub branch: String,
    /// Count of files with uncommitted changes.
    #[serde(default)]
    pub modified: u32,
    /// Commits the branch is ahead of its upstream.
    #[serde(default)]
    pub ahead: u32,
    /// Commits the branch is behind its upstream.
    #[serde(default)]
    pub behind: u32,
    /// `true` when the checkout has any uncommitted change.
    #[serde(default)]
    pub dirty: bool,
}

/// Token usage of one agent inside a workspace; the workspace card lists the
/// top entries only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTokenUsage {
    /// Agent name/id the usage is attributed to.
    pub agent: String,
    /// Prompt (input) tokens consumed.
    pub input: u64,
    /// Completion (output) tokens produced.
    pub output: u64,
}

/// A workspace attached to a host, with its git and token-usage summary, as
/// listed by `BridgeNetwork`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceNode {
    /// Workspace uuid; the same id is used by workspace-scoped sync messages
    /// (`PushWorkspaceFiles`, `RequestWorkspaceFiles`, `SwitchWorkspace`).
    pub workspace_id: Uuid,
    /// Host the workspace runs on; joins `HostMetrics::host_id`.
    pub host_id: String,
    /// On-disk path of the checkout as seen by that host.
    pub path: String,
    /// User-facing alias; absent = show `path`.
    #[serde(default)]
    pub alias: Option<String>,
    /// Git summary; absent when the checkout was not sampled or is not a repository.
    #[serde(default)]
    pub git: Option<WorkspaceGitStatus>,
    /// Per-agent token usage; empty when none was recorded.
    #[serde(default)]
    pub token_usage: Vec<WorkspaceTokenUsage>,
}

/// A user account as returned by the `AuthListUsers` / `AuthGetUser` responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserInfo {
    /// User id in the string form the auth service stores.
    pub user_id: String,
    /// Login name.
    pub username: String,
    /// Display name; absent = fall back to `username`.
    pub display_name: Option<String>,
    /// Whether the account is enabled for login.
    pub is_active: bool,
}

/// A single reasoning step rendered in an agent's "thinking" timeline.
///
/// Carried by `Tui.AgentThinkingStep` (`params.step`). Emitted by the mock
/// simulator and by real agents; the webui appends it to the streaming store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStepEntry {
    /// Producer-assigned step id; the client timeline keys steps by it.
    pub id: String,
    /// Text of the reasoning step as rendered in the thinking timeline.
    pub content: String,
    /// `"running"` | `"completed"`.
    pub status: String,
    /// Producer wall-clock time of the step.
    pub timestamp: String,
}

/// The acting end user a forwarded conversational frame acts for, plus
/// the tool-execution dimension of their RBAC standing. The panel
/// gateway asserts it under the shared service credential; the
/// execution plane gates tool runs on `agent_execute`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorClaims {
    /// End user the forwarded frame acts for.
    pub user_id: Uuid,
    /// Whether that user holds the `agent_execute` permission the tool gate checks.
    pub agent_execute: bool,
    /// The acting user's group identifiers (builtin keys for builtins,
    /// names for custom groups), resolved fresh by the gateway at send
    /// time. Empty for legacy senders that predate the field — group
    /// gated policies treat an empty list as "on no group" (fail-closed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

/// Wire enum of the sync protocol, internally tagged by `action`.
///
/// Variants are grouped below by domain. Pairing and delivery kind are
/// declared centrally in `plana::jsonrpc::pending`: `SyncReq` waits for the
/// paired reply, `AsyncReq` gets its reply later through the pending-request
/// registry, and `OneWay` is fire-and-forget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SyncMessage {
    // ═══ Protocol / Connection ═══
    /// Liveness probe (`Sync.Ping`, `SyncReq`) answered by the payload-free
    /// `Sync.Pong` method.
    Ping {
        timestamp: u64,
    },
    // ═══ Layer-2 / Custom Agents ═══
    /// Async request for the Layer-2 agent roster; answered by
    /// `Layer2AgentListResponse`.
    Layer2AgentList,
    /// Layer-2 roster reply carrying one `Layer2AgentInfo` per registered agent.
    Layer2AgentListResponse {
        agents: Vec<super::super::layer2::Layer2AgentInfo>,
    },
    /// Async request for the tools one Layer-2 agent exposes (`agent_name`).
    Layer2AgentToolTools {
        agent_name: String,
    },
    /// Reply with the requested agent's tools.
    Layer2AgentToolResponse {
        agent_name: String,
        tools: Vec<super::super::layer2::Layer2ToolInfo>,
    },
    /// Async request for the skills one Layer-2 agent exposes (`agent_name`).
    Layer2AgentSkills {
        agent_name: String,
    },
    /// Reply with the requested agent's skills.
    Layer2AgentSkillsResponse {
        agent_name: String,
        skills: Vec<super::super::layer2::Layer2SkillInfo>,
    },
    /// Async request for one tool's prompt template (`agent_name`, `tool`, optional `lang`).
    Layer2AgentToolPrompt {
        agent_name: String,
        tool: String,
        lang: Option<String>,
    },
    /// Rendered tool prompt plus its display `name` and the `lang` actually served.
    Layer2AgentToolPromptResponse {
        agent_name: String,
        tool: String,
        lang: String,
        content: String,
        name: String,
    },
    /// Async request for one skill's prompt template (`agent_name`, `skill`, optional `lang`).
    Layer2AgentSkillPrompt {
        agent_name: String,
        skill: String,
        lang: Option<String>,
    },
    /// Rendered skill prompt plus its display `name` and the `lang` actually served.
    Layer2AgentSkillPromptResponse {
        agent_name: String,
        skill: String,
        lang: String,
        content: String,
        name: String,
    },
    /// Request for the subscribed custom-agent roster; answered by `CustomAgentListResponse`.
    CustomAgentList,
    /// Roster reply carrying one `CustomAgentInfo` per subscribed agent.
    CustomAgentListResponse {
        agents: Vec<super::super::layer2::CustomAgentInfo>,
    },
    /// Install a custom agent from a marketplace source, repository or URL.
    SubscribeCustomAgent {
        source: String,
        repository: Option<String>,
        url: Option<String>,
    },
    /// Outcome of `SubscribeCustomAgent`: the installed agent plus its skills and permissions.
    SubscribeCustomAgentResponse {
        success: bool,
        error: Option<String>,
        agent: Option<super::super::layer2::CustomAgentInfo>,
        skills: Vec<String>,
        permissions: Vec<String>,
    },
    /// Remove a subscribed custom agent by `name`.
    UnsubscribeCustomAgent {
        name: String,
    },
    /// Outcome of `UnsubscribeCustomAgent`; `error` is set when the name was unknown.
    UnsubscribeCustomAgentResponse {
        success: bool,
        error: Option<String>,
    },
    // ═══ Protocol / Connection (continued) ═══
    /// One-way announcement of the server build: `version` plus free-form `build_info`.
    ServerVersion {
        version: String,
        build_info: String,
    },
    /// Blocking handshake (`SyncReq`): the client presents its token, claimed
    /// capabilities and node description; the server answers `HandshakeAck`.
    ConnectHandshake {
        /// Service credential the session is authenticated with.
        token: String,
        /// Session to resume; absent = the server mints a new one.
        #[serde(default)]
        session_id: Option<String>,
        /// Capabilities the client claims, so the server routes only matching work to it.
        #[serde(default)]
        capabilities: Vec<ClientCapability>,
        /// Self-description registered for this client node; absent = not reported.
        #[serde(default)]
        node_info: Option<ClientNodeInfo>,
        /// Workspace the client wants the session scoped to; absent = left unscoped.
        #[serde(default)]
        workspace_id: Option<Uuid>,
        /// Identifies the client type: "cli" for request/response CLI,
        /// "tui" for the full-featured TUI. When "cli", the server
        /// skips PubSub bridge and filters background-agent broadcasts.
        #[serde(default)]
        client_type: Option<String>,
    },
    /// Handshake result: `ok` plus the session the client is bound to.
    HandshakeAck {
        ok: bool,
        /// Failure reason when `ok` is false; absent on success.
        #[serde(default)]
        error: Option<String>,
        /// Session id assigned (or resumed) for this connection.
        #[serde(default)]
        session_id: Option<String>,
        /// Set by the server when this ack answers a re-connect rather than a fresh session.
        #[serde(default)]
        reconnect: bool,
    },
    /// One-way notice that the two sides disagree on the build, carrying both
    /// version strings so the client can decide whether to continue.
    VersionMismatch {
        server_version: String,
        client_version: String,
    },
    // ═══ Agent Lifecycle ═══
    /// Client → server user turn (one-way): prompt text, optional images and
    /// topic correlation. The server routes it into the agent chain and answers
    /// with `AgentStreamingChunk` / `AgentReport` for the same conversation.
    UserMessage {
        sender_id: String,
        content: String,
        #[serde(deserialize_with = "deserialize_string_or_number")]
        timestamp: String,
        /// Language tag of the prompt (e.g. "en"); absent = unspecified.
        #[serde(default)]
        language: Option<String>,
        /// Images attached to the turn; absent = text-only turn.
        #[serde(default)]
        images: Option<Vec<plana_core::LlmImageContent>>,
        /// Workspace the turn is scoped to, when the sender targets one.
        #[serde(default)]
        workspace_id: Option<Uuid>,
        /// Client-supplied topic correlation id (e.g. the shittim-chest
        /// conversation id). When present the server reuses that
        /// conversation instead of minting a fresh one, so follow-up
        /// replies chain into the same topic thread.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conversation_id: Option<Uuid>,
        /// Acting end-user assertion (tool-execution permission
        /// dimension). The panel gateway stamps it on
        /// service-authenticated forwards so the execution plane can
        /// gate tool runs on the originating user's standing.
        /// Absent on legacy/local senders — no gate applies.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actor: Option<ActorClaims>,
    },
    /// Server → client final answer for one turn; follows the streaming chunks
    /// emitted for the same `agent_id`.
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
    /// Server → client incremental model output (push topic `agent_streaming`);
    /// `is_done` marks the final chunk of the turn.
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
    /// Server → client reasoning step for the agent thinking timeline
    /// (push topic `agent_thinking`).
    AgentThinkingStep {
        agent_type: Agent,
        agent_id: String,
        step: ThinkingStepEntry,
    },
    /// Server → client report card for one turn (push topic `reports`): body,
    /// preset options, usage, routing and the chain correlation ids.
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
        tool_count: Option<u32>,
        #[serde(default)]
        next_route: Option<RouteInfo>,
        #[serde(default)]
        stream: Option<LlmStream>,
        #[serde(default)]
        error: Option<StructuredAgentError>,
        /// Topic correlation: the conversation this report belongs to
        /// (one conversation per user turn). Client-supplied via
        /// `UserMessage.conversation_id` when present, else minted by
        /// the server at skill-chain pre-init.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conversation_id: Option<Uuid>,
        /// The task (skill-chain run) that produced this report.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        task_id: Option<Uuid>,
        /// Workspace scope of the originating chain, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        workspace_id: Option<Uuid>,
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
    /// Server → client notice that the agent handed the turn from `from_skill`
    /// to `to_skill`.
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
    /// Server → client orchestration progress: the `SkillStage` the chain reached
    /// and what it is running.
    OrchestrationStatus {
        /// Stage of the skill chain this frame reports.
        stage: SkillStage,
        agent: String,
        #[serde(default)]
        agent_type: Option<Agent>,
        #[serde(default)]
        tool_name: Option<String>,
        /// Id of the tool invocation being reported; absent outside tool stages.
        #[serde(default)]
        call_id: Option<Uuid>,
        /// Id of the spawning agent; absent for root chains.
        #[serde(default)]
        parent_agent: Option<Uuid>,
        /// Human-readable summary of the tool arguments; absent when not reported.
        #[serde(default)]
        parameters_summary: Option<String>,
    },
    /// Server → client result of one tool run (push topic `tool_result`);
    /// `call_id` matches the tool call reported through `OrchestrationStatus`.
    ToolResult {
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
    /// Server → client tool invocation for the timeline (push topic
    /// `agent_tool_call`), with arguments and the current status.
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
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    StreamingTail {
        agent_id: String,
        tail: String,
    },
    /// Server → client human-in-the-loop review request (push topic
    /// `human_review`): the item to review plus the id to answer with.
    HumanReviewRequest {
        review_id: String,
        agent_type: Agent,
        agent_id: String,
        title: String,
        content: String,
        timestamp: String,
    },
    /// Client → server answer to a `HumanReviewRequest`, matched by `review_id`.
    HumanReviewResponse {
        review_id: String,
        choice: String,
        comment: String,
    },
    /// Server → client consultation question, with preset options and the one the
    /// agent recommends.
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
    /// Client → server answer to an `AskHumanRequest`; the async reply is
    /// `AskHumanReplyResponse`.
    AskHumanReply {
        consultation_id: String,
        selected_options: Vec<String>,
        custom_answer: Option<String>,
        answered_by: AskAnswerSource,
        timestamp: String,
    },
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    AutoModeUpdate {
        enabled: bool,
        timeout_secs: Option<u64>,
    },
    /// One-way identity announcement: a scepter-side connection declares the
    /// device it serves so the gateway can bind the session to that device.
    ScepterIdentity {
        device_id: Uuid,
    },
    /// Blocking request for the agent roster; answered by `AgentListResponse`.
    /// Clients on the state-tree protocol declare the `state.agents` viewport
    /// instead of polling this method.
    ListAgents,
    /// Agent roster reply; the client store upserts each entry under
    /// `state.agents.<agent_id>`.
    AgentListResponse {
        agents: Vec<super::super::agent::TuiAgentInfo>,
    },
    /// One-way update of a single roster entry; the client store re-upserts it
    /// under `state.agents.<agent_id>`.
    AgentUpdate {
        agent: super::super::agent::TuiAgentInfo,
    },
    // ═══ Task Management ═══
    /// One-way notice that a task/issue pair was created (push topic `task`).
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
        /// Topic correlation: the conversation this task was spawned
        /// for, when the spawning chain knows one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        conversation_id: Option<Uuid>,
        /// Task Decompose estimate for this task, in degrees, when the
        /// spawning chain produced one (chest reserves against it).
        /// Absent on senders that do not estimate: old payloads and old
        /// receivers stay wire-compatible in both directions.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        estimated_degrees: Option<i64>,
    },
    /// One-way task progress update (push topic `task`): `status` plus `progress`
    /// in percent.
    TaskStatusUpdate {
        task_id: Uuid,
        status: crate::TaskStatus,
        progress: u8,
    },
    /// A task's estimate was updated after creation (S2c: the skill chain
    /// refines its estimate). Wire consumers use this to adjust reserves;
    /// consumers reading the value must use `!= null` — `Some(0)` is a
    /// legitimate zero-cost estimate, not "no estimate".
    TaskEstimateUpdated {
        task_id: Uuid,
        #[serde(default)]
        estimated_degrees: Option<i64>,
    },
    // ═══ LLM Provider Configuration ═══
    /// Register a new LLM provider (name, credential, endpoint, default model);
    /// answered by `LlmProviderConfigured`.
    ConfigureLlmProvider {
        provider_name: String,
        api_key: String,
        api_endpoint: Option<String>,
        default_model: String,
        provider_type: String,
    },
    /// Outcome of `ConfigureLlmProvider`.
    LlmProviderConfigured {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    /// Change a provider's display name; answered by `ProviderRenamed`.
    RenameProvider {
        provider_name: String,
        new_display_name: String,
    },
    /// Outcome of `RenameProvider`.
    ProviderRenamed {
        provider_name: String,
        new_display_name: String,
        success: bool,
        error: Option<String>,
    },
    /// Patch a provider's credential and/or endpoint; absent members stay
    /// unchanged. Answered by `ProviderEdited`.
    EditProvider {
        provider_name: String,
        api_key: Option<String>,
        api_endpoint: Option<String>,
    },
    /// Outcome of `EditProvider`.
    ProviderEdited {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    /// Remove a provider configuration by name; answered by `ProviderDeleted`.
    DeleteProvider {
        provider_name: String,
    },
    /// Outcome of `DeleteProvider`.
    ProviderDeleted {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    /// Request the configured-provider catalogue; answered by
    /// `ConfiguredProvidersList`.
    ListConfiguredProviders,
    /// The configured-provider catalogue.
    ConfiguredProvidersList {
        providers: Vec<super::super::provider::ConfiguredProvider>,
    },
    /// Update one model's provider configuration (endpoint, context window,
    /// pricing, quotas, modality flags); absent members stay unchanged. Answered
    /// by `ModelProviderConfigUpdated`.
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
    /// Outcome of `UpdateModelProviderConfig`.
    ModelProviderConfigUpdated {
        provider_name: String,
        success: bool,
        error: Option<String>,
    },
    /// Probe an endpoint/credential pair with the given wire `protocol`; answered
    /// by `EndpointValidated`.
    ValidateEndpoint {
        provider_name: String,
        api_endpoint: String,
        api_key: Option<String>,
        protocol: GenProtocol,
    },
    /// Result of `ValidateEndpoint`: reachability plus measured latency or the
    /// probe error.
    EndpointValidated {
        provider_name: String,
        is_reachable: bool,
        latency_ms: Option<u64>,
        error: Option<String>,
    },
    /// Query usage-period accounting for one user (`user_id` absent = all users);
    /// answered by `UsagePeriodResponse`.
    UsagePeriodQuery {
        user_id: Option<String>,
        period_types: Vec<super::super::provider::PeriodType>,
    },
    /// Usage-period rows answering `UsagePeriodQuery`.
    UsagePeriodResponse {
        data: Vec<super::super::provider::UsagePeriodData>,
    },
    /// One-way push of refreshed usage-period rows, so clients need not poll.
    UsagePeriodUpdate {
        data: Vec<super::super::provider::UsagePeriodData>,
    },
    // ═══ State Sync / Snapshots ═══
    /// Blocking request for a full state snapshot; answered by the `FullSnapshot`
    /// method, which has no variant in this enum.
    RequestFullSnapshot,
    /// One-way field-level agent increment from entelecheia: each patch carries
    /// `agent_id` plus only the changed fields, and the client store replaces
    /// them one by one under `state.agents.<agent_id>`.
    AgentPatch {
        patches: Vec<super::super::snapshot::AgentPatch>,
    },
    /// One-way full agent snapshot, applied as the authoritative agent state for
    /// its scope.
    AgentSnapshot {
        snapshot: super::super::snapshot::AgentSnapshot,
    },
    /// Blocking request for the global snapshot; answered by `GlobalSnapshot`.
    RequestGlobalSnapshot,
    /// One-way global snapshot (versioned agents, containers and active tasks).
    GlobalSnapshot {
        snapshot: super::super::snapshot::GlobalSnapshot,
    },
    /// One-way snapshot of the model catalogue the client renders.
    ModelsSnapshot {
        models: Vec<super::super::snapshot::ModelInfo>,
    },
    /// One-way snapshot of the provider entries the client renders.
    ProvidersSnapshot {
        providers: Vec<super::super::snapshot::ProviderInfo>,
    },
    /// One-way container increments applied by the client container store.
    ContainerPatch {
        patches: Vec<super::super::snapshot::ContainerPatch>,
    },
    /// Blocking request for the container snapshot; answered by `ContainerSnapshot`.
    RequestContainerSnapshot,
    /// One-way versioned container snapshot.
    ContainerSnapshot {
        snapshot: super::super::snapshot::ContainerSnapshot,
    },
    /// One-way task increments applied by the client task store.
    TaskPatch {
        patches: Vec<super::super::snapshot::TaskPatch>,
    },
    /// Blocking request for the task snapshot; answered by `TasksSnapshot`.
    RequestTasksSnapshot,
    /// One-way versioned task snapshot.
    TasksSnapshot {
        snapshot: super::super::snapshot::TasksSnapshot,
    },
    /// Blocking request for one agent VM's introspection snapshot; answered by
    /// `VmSnapshot`.
    RequestVmSnapshot {
        agent_type: Agent,
        agent_id: String,
    },
    /// One-way VM snapshot of one agent: interpreter globals, container info, the
    /// available tools and the operation log.
    VmSnapshot {
        agent_id: String,
        globals: serde_json::Value,
        container_info: Option<CosmosContainerInfo>,
        tool_list: Vec<String>,
        op_log: Vec<CosmosOperationLogEntry>,
    },
    // ═══ Agent Lifecycle (continued) ═══
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    RetryAgentRequest {
        agent_id: String,
        attempt: u32,
    },
    /// Semantics TBC — see no caller in this worktree; the payload-free variant is
    /// absent from the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    UndoRequest,
    // ═══ Config Filesystem ═══
    /// Blocking request for the provider configuration read from disk; answered by
    /// `ProvidersFromFsResponse`.
    GetProvidersFromFs,
    /// Provider files discovered in the on-disk config store.
    ProvidersFromFsResponse {
        providers: Vec<super::super::config_fs::ProviderFsInfo>,
    },
    /// Blocking request for the model configuration read from disk; answered by
    /// `ModelsFromFsResponse`.
    GetModelsFromFs,
    /// Model files discovered in the on-disk config store.
    ModelsFromFsResponse {
        models: Vec<super::super::config_fs::ModelFsInfo>,
    },
    /// Ask the server to re-read one provider's configuration from disk
    /// (`provider_id`) without editing it.
    ReloadProviderConfig {
        provider_id: String,
    },
    /// Ask the server to re-read one model's configuration from disk
    /// (`provider_id` + `model_id`).
    ReloadModelConfig {
        provider_id: String,
        model_id: String,
    },
    /// Blocking request for the on-disk user configuration; answered by
    /// `UserConfigResponse`.
    GetUserConfig,
    /// The user configuration as read from disk.
    UserConfigResponse {
        config: super::super::config_fs::UserInfo,
    },
    /// Write the supplied user configuration back to disk; `config` replaces the
    /// stored value.
    UpdateUserConfig {
        config: super::super::config_fs::UserInfo,
    },
    /// Ask the server to re-read the user configuration from disk.
    ReloadUserConfig,
    /// Ask the server to re-read every agent configuration from disk.
    ReloadAllAgentConfigs,
    /// Ask the server to re-read one agent's configuration (`agent_name`).
    ReloadAgentConfig {
        agent_name: String,
    },
    /// Request the stored API-key registry; answered by `KeysListResponse`.
    ListKeys,
    /// The stored API-key entries.
    KeysListResponse {
        keys: Vec<super::super::config_fs::KeyInfo>,
    },
    /// Store or replace one provider's credential, with optional metadata.
    SaveApiKey {
        provider: String,
        api_key: String,
        metadata: Option<super::super::config_fs::KeyMetadata>,
    },
    /// Delete one provider's stored credential.
    DeleteApiKey {
        provider: String,
    },
    /// Request the metadata of one provider's stored credential; answered by
    /// `ApiKeyInfoResponse`.
    GetApiKeyInfo {
        provider: String,
    },
    /// Metadata of the requested stored credential.
    ApiKeyInfoResponse {
        info: super::super::config_fs::KeyInfo,
    },
    // ═══ Semantic Search ═══
    /// Semantic-search query: free text, a source filter and a score floor;
    /// answered by `SearchResponse`.
    SearchRequest {
        query: String,
        #[serde(default = "default_search_limit")]
        limit: u64,
        /// Optional source filter (e.g. "report", "knowledge").
        source: Option<String>,
        #[serde(default)]
        min_score: f64,
    },
    /// Semantic-search hits for the preceding `SearchRequest`.
    SearchResponse {
        response: super::super::search::SearchResponse,
    },
    // ═══ Long-term Memory ═══
    /// Store a node into the long-term memory service (Philia).
    MemoryStoreRequest {
        text: String,
        node_type: String,
        #[serde(default)]
        source_episode_id: Option<String>,
        #[serde(default)]
        related_node_ids: Option<Vec<String>>,
        #[serde(default)]
        properties: Option<std::collections::HashMap<String, String>>,
    },
    /// Id assigned to the stored memory node.
    MemoryStoreResponse {
        node_id: String,
    },
    /// Semantic long-term memory query.
    MemoryQueryRequest {
        query: String,
        #[serde(default = "default_search_limit")]
        limit: u64,
        #[serde(default)]
        graph_depth: Option<u64>,
        #[serde(default)]
        node_type_filter: Option<String>,
        #[serde(default)]
        subgraph: Option<bool>,
    },
    /// Memory query result set plus the total hit count.
    MemoryQueryResponse {
        query: String,
        total: usize,
        results: Vec<plana_celestia_types::tools::philia::MemoryQueryItem>,
    },
    /// Delete a stored memory node by id.
    MemoryDeleteRequest {
        node_id: String,
    },
    /// Whether the node existed and was removed.
    MemoryDeleteResponse {
        deleted: bool,
    },
    // ═══ Conversation History (lazy-loaded reports) ═══
    /// Emitted once when the server creates a conversation for the active task,
    /// so the client can later request paginated message history.
    ConversationStarted {
        conversation_id: Uuid,
    },
    /// Paginated request for the newest messages of one conversation; answered by
    /// `MessagesResponse`.
    RequestRecentMessages {
        conversation_id: Uuid,
        #[serde(default = "default_history_limit")]
        limit: u64,
    },
    /// Paginated request for messages older than the `before_created_at` cursor;
    /// answered by `MessagesResponse`.
    RequestOlderMessages {
        conversation_id: Uuid,
        /// ISO-8601 cursor: load messages created strictly before this time.
        before_created_at: String,
        #[serde(default = "default_history_limit")]
        limit: u64,
    },
    /// One page of conversation history.
    MessagesResponse {
        conversation_id: Uuid,
        page: super::super::history::MessagesPage,
    },
    // ═══ Knowledge Base ═══
    /// Run one knowledge-base creation request; answered by
    /// `CreateKnowledgeBaseResponse`.
    CreateKnowledgeBase {
        request: super::super::knowledge_base::CreateKnowledgeBaseRequest,
    },
    /// Result of the wrapped knowledge-base creation request.
    CreateKnowledgeBaseResponse {
        response: super::super::knowledge_base::CreateKnowledgeBaseResponse,
    },
    /// Add a document to a knowledge base.
    AddDocument {
        request: super::super::knowledge_base::AddDocumentRequest,
    },
    /// Result of the wrapped add-document request.
    AddDocumentResponse {
        response: super::super::knowledge_base::AddDocumentResponse,
    },
    /// Query one knowledge base for relevant chunks.
    QueryKnowledgeBase {
        request: super::super::knowledge_base::QueryKnowledgeBaseRequest,
    },
    /// Retrieved chunks plus the wrapped query result.
    QueryKnowledgeBaseResponse {
        response: super::super::knowledge_base::QueryKnowledgeBaseResponse,
    },
    /// Create a knowledge-base subscription (source plus schedule).
    CreateSubscription {
        request: super::super::knowledge_base::CreateSubscriptionRequest,
    },
    /// Result of the wrapped subscription creation.
    CreateSubscriptionResponse {
        response: super::super::knowledge_base::CreateSubscriptionResponse,
    },
    /// Trigger one manual sync of a knowledge-base subscription.
    SyncSubscription {
        request: super::super::knowledge_base::SyncSubscriptionRequest,
    },
    /// Result of the wrapped subscription sync.
    SyncSubscriptionResponse {
        response: super::super::knowledge_base::SyncSubscriptionResponse,
    },
    /// Remove a knowledge-base subscription by id.
    DeleteSubscription {
        subscription_id: Uuid,
    },
    /// Result of the wrapped subscription deletion.
    DeleteSubscriptionResponse {
        response: super::super::knowledge_base::DeleteSubscriptionResponse,
    },
    /// Fetch one knowledge base by id.
    GetKnowledgeBase {
        knowledge_base_id: Uuid,
    },
    /// The knowledge base when the id is known; `None` otherwise.
    GetKnowledgeBaseResponse {
        knowledge_base: Option<super::super::knowledge_base::KnowledgeBaseInfo>,
    },
    /// List knowledge bases, optionally filtered.
    ListKnowledgeBases {
        filters: Option<super::super::knowledge_base::KnowledgeBaseFilters>,
    },
    /// Knowledge bases matching the filter.
    ListKnowledgeBasesResponse {
        knowledge_bases: Vec<super::super::knowledge_base::KnowledgeBaseInfo>,
    },
    /// Delete a knowledge base by id.
    DeleteKnowledgeBase {
        knowledge_base_id: Uuid,
    },
    /// Result of the wrapped knowledge-base deletion.
    DeleteKnowledgeBaseResponse {
        response: super::super::knowledge_base::DeleteKnowledgeBaseResponse,
    },
    // ═══ Workspace ═══
    /// Open (resolve and attach) a workspace by URI; answered by
    /// `OpenWorkspaceResponse`.
    OpenWorkspace {
        uri: String,
    },
    /// Result of `OpenWorkspace`, carrying the new workspace id on success.
    OpenWorkspaceResponse {
        success: bool,
        workspace_id: Option<Uuid>,
        #[serde(default)]
        error: Option<String>,
    },
    /// One-way workspace status push: display name, connection kind, resolved path,
    /// remote URL, branch and host.
    WorkspaceStatus {
        workspace_id: Uuid,
        display_name: Option<String>,
        connection_kind: String,
        resolved_path: Option<String>,
        remote_url: Option<String>,
        branch: Option<String>,
        host_id: Option<String>,
    },
    /// Blocking request for the workspace status; answered by `WorkspaceStatus`.
    RequestWorkspaceStatus,
    /// Blocking request for the bridge-device roster; answered by
    /// `PolemosDeviceList`. Clients on the state-tree protocol declare the
    /// `state.devices` viewport instead of polling this method.
    ListPolemosDevices,
    /// One-way bridge-device roster push (scepter); the client store keys each
    /// entry under `state.devices.<node_id>`.
    PolemosDeviceList {
        devices: Vec<PolemosDeviceInfo>,
    },
    /// Register a bridge device with the gateway (host id, address, optional
    /// workspace path); answered by `RegisterPolemosDeviceResponse`.
    RegisterPolemosDevice {
        host_id: String,
        address: String,
        workspace_path: Option<String>,
    },
    /// Result of `RegisterPolemosDevice`, echoing the stored device entry.
    RegisterPolemosDeviceResponse {
        success: bool,
        error: Option<String>,
        device: Option<PolemosDeviceInfo>,
    },
    // ── Industrial push events (scepter → shittim-chest) ──────────
    /// One-way telemetry push (scepter → shittim-chest webui), in single-reading
    /// or batch form.
    IndustrialTelemetryPush {
        /// Single-reading form; senders that batch set it to `null` and fill `readings`.
        reading: Option<IndustrialSensorReading>,
        /// Batch form; absent on senders that push one reading at a time.
        #[serde(default)]
        readings: Option<Vec<IndustrialSensorReading>>,
    },
    /// One-way alarm push carrying one breach or clear event.
    IndustrialAlarmPush {
        event: IndustrialAlarmEvent,
    },
    /// One-way discovery progress push, mirroring `IndustrialDiscoveryProgress`.
    IndustrialDiscoveryPush {
        session_id: String,
        phase: IndustrialDiscoveryPhase,
        message: String,
        found_devices: u64,
        progress_percent: u32,
        #[serde(default)]
        raw_findings: Option<serde_json::Value>,
    },
    // ── Dashboard push events (P3#A4: agent → webui panels) ─────────
    // Payloads match the shittim-chest webui handlers
    // (`useWebSocketHandlers.ts`): layout_id (layoutId alias), widget_id
    // (widgetId alias), op ("create" | "update" | "delete"). Layout and
    // widget bodies travel as opaque JSON the agent constructs.
    /// One-way dashboard descriptor push (agent → webui panel): replaces the panel
    /// layout named by `layout_id`.
    DashboardLayoutPush {
        layout_id: String,
        /// Full dashboard descriptor ({title, subtitle?, widgets[]}).
        layout: serde_json::Value,
    },
    /// One-way widget data push for a dashboard layout.
    ViewDataPush {
        layout_id: String,
        widget_id: String,
        /// Per-widget data payload ({columns/rows}, {items}, …).
        data: serde_json::Value,
        /// false = shallow-merge into existing data (incremental).
        #[serde(default)]
        full_replace: bool,
    },
    /// One-way widget create/update/delete on an existing layout (`op`).
    ViewInstancePush {
        layout_id: String,
        /// "create" | "update" | "delete".
        op: String,
        /// Widget descriptor ({id, type, title?, source, span?, …}).
        widget: serde_json::Value,
    },
    // ── Pipeline execution progress (P3#C2 streaming) ────────────────
    // Streamed from the chest backend while a media pipeline runs so the
    // webui can update node states without polling pipeline.history.
    /// One-way media-pipeline progress push from the chest backend, per node, so
    /// the webui need not poll `pipeline.history`.
    PipelineProgress {
        run_id: String,
        /// Node id currently reporting progress.
        node_id: String,
        /// Node status: "running" | "complete" | "error" | "skipped".
        status: String,
        /// 0–1 progress (running nodes).
        #[serde(default)]
        progress: f64,
        /// Error message (status == "error").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// One-way terminal push for a pipeline run: completed and failed node counts.
    PipelineDone {
        run_id: String,
        completed: u32,
        failed: u32,
    },
    /// One-way write-approval request push; the operator UI echoes `request_id`
    /// back through `industrial.approveWrite`.
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
    /// Ask the server to switch the session to another workspace; answered by
    /// `SwitchWorkspaceResponse`.
    SwitchWorkspace {
        workspace_id: Uuid,
    },
    /// Result of `SwitchWorkspace`, echoing the workspace that is now active.
    SwitchWorkspaceResponse {
        success: bool,
        workspace_id: Uuid,
        error: Option<String>,
    },
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    SetClientCwd {
        path: String,
        #[serde(default)]
        device_id: Option<Uuid>,
        #[serde(default)]
        device_type: Option<String>,
    },
    /// Upload one batch of files into a workspace; `batch_index`/`batch_total`
    /// frame the transfer and each batch is answered by `PushWorkspaceFilesAck`.
    PushWorkspaceFiles {
        workspace_id: Uuid,
        files: Vec<FilePayload>,
        base_path: String,
        #[serde(default)]
        batch_index: u32,
        #[serde(default)]
        batch_total: u32,
    },
    /// Per-batch acknowledgement of `PushWorkspaceFiles`: accepted count or error.
    PushWorkspaceFilesAck {
        workspace_id: Uuid,
        batch_index: u32,
        accepted: u32,
        error: Option<String>,
    },
    /// Ask the peer to send workspace files matching the globs (minus the exclude
    /// patterns); the files travel back as `PushWorkspaceFiles` batches.
    RequestWorkspaceFiles {
        workspace_id: Uuid,
        glob_patterns: Vec<String>,
        #[serde(default)]
        exclude_patterns: Vec<String>,
    },
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    WorkspaceReady {
        workspace_id: Uuid,
        container_id: Option<String>,
    },
    // ═══ Noa Workspace ═══
    /// NOA round trip step 1 (scepter → client): open `remote_name`/`remote_path`
    /// as a NOA workspace.
    RequestNoaHandshake {
        workspace_id: Uuid,
        remote_name: String,
        remote_path: String,
    },
    /// NOA round trip step 2 (client → scepter): the client's view of the
    /// workspace — repo id, current branch, whether NOA was initialized.
    NoaHandshakeResponse {
        workspace_id: Uuid,
        repo_id: String,
        current_branch: String,
        #[serde(default)]
        noa_initialized: bool,
        #[serde(default)]
        gitignore_updated: bool,
    },
    /// NOA round trip step 3 (scepter → client): branch picker with the candidate
    /// branches, a suggestion and the reason for it.
    NoaAuthRequest {
        workspace_id: Uuid,
        branches: Vec<String>,
        suggested_branch: String,
        reason: String,
    },
    /// NOA round trip step 4 (client → scepter): the branch the user picked and
    /// whether they approved it.
    NoaAuthResponse {
        workspace_id: Uuid,
        selected_branch: String,
        #[serde(default)]
        branch_base: Option<String>,
        #[serde(default)]
        approved: bool,
    },
    /// NOA terminal event (scepter → client): the workspace is ready on `branch`
    /// at `snapshot_id`.
    NoaReady {
        workspace_id: Uuid,
        branch: String,
        snapshot_id: String,
    },
    /// Bidirectional NOA event batch exchanged after `NoaReady`; `direction` names
    /// the sending side.
    NoaEventSync {
        workspace_id: Uuid,
        events: Vec<NoaEvent>,
        #[serde(default)]
        direction: Option<String>,
    },
    /// Acknowledgement of a `NoaEventSync` batch, naming the last event id the
    /// sender consumed.
    NoaEventSyncAck {
        workspace_id: Uuid,
        last_event_id: String,
    },
    // ═══ File Browsing ═══
    // List/read files inside a container (#demiurge / #NNN), on a host, or in
    // a workspace checkout. The node-list container cards, the Bridge Network
    // cards and the workspace browser all open this same file browser.
    /// List one directory level of a target; answered by `FileTree`.
    RequestFileTree {
        target: FileTarget,
        #[serde(default)]
        path: String,
    },
    /// Directory listing reply for the requested target and path.
    FileTree {
        target: FileTarget,
        path: String,
        entries: Vec<FileTreeEntry>,
    },
    /// Read one text file of a target (capped server-side); answered by `FileRead`.
    RequestFileRead {
        target: FileTarget,
        path: String,
    },
    /// File-content reply; `truncated` marks a body cut at the server cap.
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
    /// Host/workspace roster reply (or push) answering `RequestBridgeNetwork`.
    BridgeNetwork {
        hosts: Vec<HostMetrics>,
        workspaces: Vec<WorkspaceNode>,
    },
    // ═══ System / UI Control ═══
    /// Semantics TBC — see no caller in this worktree; the variant is absent from
    /// the Sync method table in packages/plana/src/jsonrpc/pending.rs.
    BadgeTransition {
        previous_llm_session_id: String,
        current_llm_session_id: String,
        previous_container_id: String,
        current_container_id: String,
        transition_uuid: Uuid,
        linked_session_uuid: Option<Uuid>,
    },
    /// One-way system notification push (push topic `system_notification`).
    SystemMessage {
        notification: SystemNotification,
        timestamp: String,
    },
    /// Ask the server to control the bundled web UI (start/stop/...); answered by
    /// `WebUiControlResponse`.
    WebUiControl {
        command: String,
    },
    /// Result of `WebUiControl`, carrying the UI URL when it is running.
    WebUiControlResponse {
        command: String,
        success: bool,
        message: String,
        url: Option<String>,
    },
    /// One-way web-UI status push: running flag, URL and container id.
    WebUiStatus {
        running: bool,
        url: Option<String>,
        container_id: Option<String>,
    },
    /// Blocking request for the web-UI status; answered by `WebUiStatus`.
    RequestWebUiStatus,
    // ═══ Authentication ═══
    /// Authenticate a user with username and password; answered by
    /// `AuthLoginResponse`.
    AuthLogin {
        username: String,
        password: String,
    },
    /// Login result: token, session and profile on success, `error` otherwise.
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
    /// Create a user account; answered by `AuthRegisterResponse`.
    AuthRegister {
        username: String,
        password: String,
        display_name: Option<String>,
    },
    /// Outcome of `AuthRegister`.
    AuthRegisterResponse {
        ok: bool,
        user_id: Option<String>,
        username: Option<String>,
        error: Option<String>,
    },
    /// Request the user list; answered by `AuthListUsersResponse`.
    AuthListUsers,
    /// User list reply; `users` is absent when the call failed.
    AuthListUsersResponse {
        ok: bool,
        users: Option<Vec<AuthUserInfo>>,
        error: Option<String>,
    },
    /// Fetch one user by id; answered by `AuthGetUserResponse`.
    AuthGetUser {
        user_id: String,
    },
    /// The requested user when found, `error` otherwise.
    AuthGetUserResponse {
        ok: bool,
        user: Option<AuthUserInfo>,
        error: Option<String>,
    },
    /// Delete a user account by id; answered by `AuthDeleteUserResponse`.
    AuthDeleteUser {
        user_id: String,
    },
    /// Outcome of `AuthDeleteUser`.
    AuthDeleteUserResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Change one user's password, presenting the old one for verification;
    /// answered by `AuthChangePasswordResponse`.
    AuthChangePassword {
        user_id: String,
        old_password: String,
        new_password: String,
    },
    /// Outcome of `AuthChangePassword`.
    AuthChangePasswordResponse {
        ok: bool,
        error: Option<String>,
    },
    // ═══ Log Subscription ═══
    /// Subscribe to one container's log stream (`instance_uuid`, optional `tail`
    /// backlog); answered by `SubscribeContainerLogsResponse`.
    SubscribeContainerLogs {
        instance_uuid: String,
        tail: Option<u32>,
    },
    /// Subscription result; `entries` seeds the client with the requested backlog.
    SubscribeContainerLogsResponse {
        ok: bool,
        error: Option<String>,
        entries: Vec<super::super::snapshot::LogEntryData>,
    },
    /// End a container log subscription by instance uuid.
    UnsubscribeContainerLogs {
        instance_uuid: String,
    },
    /// Outcome of `UnsubscribeContainerLogs`.
    UnsubscribeContainerLogsResponse {
        ok: bool,
        error: Option<String>,
    },
    /// One-way container log line (push topic `container_logs`) for subscribers of
    /// `instance_uuid`.
    ContainerLogEntry {
        instance_uuid: String,
        entry: super::super::snapshot::LogEntryData,
    },
    /// Subscribe to the server's own log stream (optional `tail`); answered by
    /// `SubscribeServerLogsResponse`.
    SubscribeServerLogs {
        tail: Option<u32>,
    },
    /// Subscription result; `entries` seeds the requested backlog.
    SubscribeServerLogsResponse {
        ok: bool,
        error: Option<String>,
        entries: Vec<super::super::snapshot::LogEntryData>,
    },
    /// End the server log subscription.
    UnsubscribeServerLogs,
    /// Outcome of `UnsubscribeServerLogs`.
    UnsubscribeServerLogsResponse {
        ok: bool,
        error: Option<String>,
    },
    /// One-way server log line (push topic `server_logs`).
    ServerLogEntry {
        entry: super::super::snapshot::LogEntryData,
    },
    // ═══ YOLO Cruise Control ═══
    /// Start YOLO cruise control (async request); answered by `YoloStartResponse`.
    YoloStart,
    /// Outcome of `YoloStart`.
    YoloStartResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Stop the YOLO loop (async request); answered by `YoloStopResponse`.
    YoloStop,
    /// Outcome of `YoloStop`.
    YoloStopResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Terminate the YOLO loop and its in-flight work (async request); answered by
    /// `YoloTerminateResponse`.
    YoloTerminate,
    /// Outcome of `YoloTerminate`.
    YoloTerminateResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Async request for the YOLO loop status; answered by `YoloStatusResponse`.
    YoloStatus,
    /// Loop status: active flag, cycle count, start time, current cycle and the
    /// per-tier status list.
    YoloStatusResponse {
        active: bool,
        loop_count: u64,
        started_at: Option<String>,
        current_cycle: Option<String>,
        #[serde(default)]
        tiers: Vec<super::super::yolo::YoloTierStatus>,
    },
    /// Async request for the YOLO tier configuration; answered by
    /// `YoloConfigResponse`.
    YoloGetConfig,
    /// Configured YOLO tiers with their intervals and tasks.
    YoloConfigResponse {
        tiers: Vec<super::super::yolo::YoloTierConfig>,
    },
    /// Enable or disable one tier task (`tier`, `agent`, `skill`); answered by
    /// `YoloUpdateTaskResponse`.
    YoloUpdateTask {
        tier: String,
        agent: String,
        skill: String,
        enabled: bool,
    },
    /// Outcome of `YoloUpdateTask`.
    YoloUpdateTaskResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Set one tier's loop interval in seconds; `tier` names a `YoloTaskTier`
    /// (e.g. "periodic"). Answered by `YoloSetTierIntervalResponse`.
    YoloSetTierInterval {
        tier: String,
        interval_secs: u64,
    },
    /// Outcome of `YoloSetTierInterval`.
    YoloSetTierIntervalResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Run one tier's tasks immediately, outside its interval; answered by
    /// `YoloRunTierNowResponse`.
    YoloRunTierNow {
        tier: String,
    },
    /// Outcome of `YoloRunTierNow`.
    YoloRunTierNowResponse {
        ok: bool,
        error: Option<String>,
    },
    /// One-way per-step progress of a YOLO cycle (push topic `yolo_cycle`).
    YoloCycleStep {
        skill: String,
        loop_count: u64,
        status: String,
        #[serde(default)]
        token_usage: Option<(u32, u32)>,
        #[serde(default)]
        model_name: Option<String>,
    },
    /// One-way terminal event of a YOLO cycle (push topic `yolo_cycle`) with its
    /// duration.
    YoloCycleComplete {
        loop_count: u64,
        duration_ms: u64,
    },
    /// One-way notice that a skill chain started (push topic `skill_chain`).
    SkillChainStart,
    /// One-way per-skill progress of a chain (push topic `skill_chain`).
    SkillChainStep {
        skill: String,
        status: String,
    },
    /// One-way notice that a skill chain finished (push topic `skill_chain`).
    SkillChainComplete {
        skill: String,
    },
    /// One-way notice that one YOLO tier task started.
    YoloTaskStart {
        tier: String,
        agent: String,
        skill: String,
    },
    /// One-way notice that one YOLO tier task finished, with duration and usage.
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
    /// One-way notice that one YOLO tier task failed.
    YoloTaskError {
        tier: String,
        agent: String,
        skill: String,
        error: String,
    },

    /// Read the arbiter's standing for one container instance (`instance_uuid`);
    /// answered by `ArbiterStatusResponse`.
    ArbiterStatus {
        instance_uuid: String,
    },
    /// Arbiter status reply; `status` carries the raw authority object.
    ArbiterStatusResponse {
        ok: bool,
        status: Option<serde_json::Value>,
        error: Option<String>,
    },
    /// Revoke a running instance's authority through the arbiter; `delegator_id`
    /// names the requesting authority and `reason` is kept with the lockdown.
    ArbiterLockdown {
        instance_uuid: String,
        delegator_id: String,
        reason: String,
    },
    /// Outcome of `ArbiterLockdown`.
    ArbiterLockdownResponse {
        ok: bool,
        error: Option<String>,
    },
    /// Lift a lockdown and restore the instance to `target_level` (auth level
    /// vocabulary, e.g. "L3").
    ArbiterRestore {
        instance_uuid: String,
        delegator_id: String,
        target_level: String,
    },
    /// Outcome of `ArbiterRestore`.
    ArbiterRestoreResponse {
        ok: bool,
        error: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::SyncMessage;
    use crate::gateway::Message;

    /// shittim-chest serializes `Sync.UserMessage.timestamp` as epoch-millis
    /// (i64); the reconstructed bridge shape must parse with the numeric
    /// form and normalize it into the declared `String` field.
    #[test]
    fn user_message_accepts_numeric_timestamp() {
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "UserMessage",
                "sender_id": "u",
                "content": "c",
                "timestamp": 1770000000000_i64
            }
        }))
        .expect("integer timestamp must deserialize");

        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::UserMessage { timestamp, .. } = &*sync else {
            panic!("expected UserMessage variant");
        };
        assert_eq!(timestamp, "1770000000000");
    }

    /// Legacy senders already emit the string form; it must keep parsing.
    #[test]
    fn user_message_accepts_string_timestamp() {
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "UserMessage",
                "sender_id": "u",
                "content": "c",
                "timestamp": "1770000000000"
            }
        }))
        .expect("string timestamp must deserialize");

        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::UserMessage { timestamp, .. } = &*sync else {
            panic!("expected UserMessage variant");
        };
        assert_eq!(timestamp, "1770000000000");
    }

    /// The real production path: scepter parses inbound JSON-RPC via
    /// `plana::jsonrpc::deserialize_from_jsonrpc`, which rebuilds the
    /// tagged `{"type":..,"data":..}` shape and feeds it to serde. An
    /// integer timestamp used to fail here and silently drop the message.
    #[test]
    fn jsonrpc_path_accepts_numeric_timestamp() {
        let parsed = plana::jsonrpc::deserialize_from_jsonrpc::<Message>(
            r#"{"jsonrpc":"2.0","method":"Sync.UserMessage","params":{"sender_id":"u","content":"c","timestamp":1770000000000}}"#,
        )
        .expect("jsonrpc parse must succeed");

        let Some(Message::Sync(sync)) = parsed else {
            panic!("expected Some(Sync) from the jsonrpc bridge");
        };
        let SyncMessage::UserMessage { timestamp, .. } = &*sync else {
            panic!("expected UserMessage from the jsonrpc bridge");
        };
        assert_eq!(timestamp, "1770000000000");
    }

    /// Serialization must stay unchanged (timestamp as a JSON string) so the
    /// TS bindings and legacy peers see the exact same wire shape as before.
    #[test]
    fn user_message_serializes_timestamp_as_string() {
        let msg = SyncMessage::UserMessage {
            sender_id: "u".to_string(),
            content: "c".to_string(),
            timestamp: "1770000000000".to_string(),
            language: None,
            images: None,
            workspace_id: None,
            conversation_id: None,
            actor: None,
        };

        let json = serde_json::to_value(&msg).expect("serialize UserMessage");
        assert_eq!(json["action"], "UserMessage");
        assert!(json["timestamp"].is_string());
        assert_eq!(json["timestamp"], "1770000000000");
        // The actor claim must stay off the wire when absent — legacy
        // peers see the exact same shape as before this field existed.
        assert!(json.get("actor").is_none());

        // The groups field rides inside the claim: present when non-empty,
        // and a legacy claim (pre-groups) deserializes with an empty list.
        let claim = super::ActorClaims {
            user_id: uuid::Uuid::new_v4(),
            agent_execute: true,
            groups: vec!["registered".to_string()],
        };
        let wire = serde_json::to_value(&claim).expect("serialize claim");
        assert_eq!(wire["groups"], serde_json::json!(["registered"]));
        let legacy: super::ActorClaims = serde_json::from_str(
            r#"{"user_id":"0f8fad5b-d9cb-469f-a165-70867728950e","agent_execute":true}"#,
        )
        .expect("legacy claim without groups must deserialize");
        assert!(legacy.groups.is_empty(), "absent groups = empty list");

        // And the mirror half: an empty group list must stay off the wire
        // (skip_serializing_if), so legacy peers never see "groups": [].
        let empty = super::ActorClaims {
            user_id: uuid::Uuid::new_v4(),
            agent_execute: false,
            groups: Vec::new(),
        };
        let wire = serde_json::to_value(&empty).expect("serialize empty claim");
        assert!(
            wire.get("groups").is_none(),
            "an empty group list must stay off the wire"
        );
    }

    /// `TaskCreated` gained `estimated_degrees` for the degree-ledger
    /// reserve flow. Old senders never emit the key: the message must
    /// deserialize to `None`, and re-serializing must keep the key absent
    /// (skip_serializing_if) so the old wire shape is preserved exactly.
    #[test]
    fn task_created_without_estimate_stays_wire_compatible() {
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "TaskCreated",
                "task_id": "00000000-0000-0000-0000-000000000001",
                "issue_id": "00000000-0000-0000-0000-000000000002",
                "title": "t",
                "sender_id": "u"
            }
        }))
        .expect("payload without estimated_degrees must deserialize");

        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::TaskCreated {
            estimated_degrees, ..
        } = &*sync
        else {
            panic!("expected TaskCreated variant");
        };
        assert!(estimated_degrees.is_none());

        let json = serde_json::to_value(&*sync).expect("re-serialize TaskCreated");
        assert!(
            json.get("estimated_degrees").is_none(),
            "None must not re-emit the key: {json}"
        );
    }

    /// New senders attach the estimate; it must round-trip as i64 through
    /// the same envelope the bridge rebuilds.
    #[test]
    fn task_created_round_trips_estimate() {
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "TaskCreated",
                "task_id": "00000000-0000-0000-0000-000000000001",
                "issue_id": "00000000-0000-0000-0000-000000000002",
                "title": "t",
                "sender_id": "u",
                "estimated_degrees": 4_i64
            }
        }))
        .expect("payload with estimated_degrees must deserialize");

        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::TaskCreated {
            estimated_degrees, ..
        } = &*sync
        else {
            panic!("expected TaskCreated variant");
        };
        assert_eq!(*estimated_degrees, Some(4_i64));

        let json = serde_json::to_value(&*sync).expect("re-serialize TaskCreated");
        assert_eq!(json["estimated_degrees"], 4_i64);
    }

    /// `TaskEstimateUpdated` wire-compat: absent field → None; Some(0) is
    /// a legitimate value distinct from None (consumers must `!= null`).
    #[test]
    fn task_estimate_updated_wire_compat() {
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "TaskEstimateUpdated",
                "task_id": "00000000-0000-0000-0000-000000000001"
            }
        }))
        .expect("payload without estimated_degrees must deserialize");

        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::TaskEstimateUpdated {
            estimated_degrees, ..
        } = &*sync
        else {
            panic!("expected TaskEstimateUpdated variant");
        };
        assert!(estimated_degrees.is_none());

        // Some(0) round-trips — falsy-checking consumers would lose this
        let msg: Message = serde_json::from_value(serde_json::json!({
            "type": "Sync",
            "data": {
                "action": "TaskEstimateUpdated",
                "task_id": "00000000-0000-0000-0000-000000000001",
                "estimated_degrees": 0_i64
            }
        }))
        .expect("payload with estimated_degrees=0 must deserialize");
        let Message::Sync(sync) = msg else {
            panic!("expected Sync variant");
        };
        let SyncMessage::TaskEstimateUpdated {
            estimated_degrees, ..
        } = &*sync
        else {
            panic!("expected TaskEstimateUpdated variant");
        };
        assert_eq!(
            *estimated_degrees,
            Some(0_i64),
            "Some(0) must survive the wire"
        );
    }
}
