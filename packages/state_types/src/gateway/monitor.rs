//! Metrics sampling for the `Monitor.*` namespace: on-demand and periodic
//! host or agent samples, plus the per-container runtime view carried by
//! `Sync.VmSnapshot`.
//!
//! Nothing in this workspace produces or consumes these payloads yet, and the
//! JSON-RPC bridge defines no constant for the namespace, so the method
//! strings come from the envelope tags (`Monitor.GetMetrics`, ...) or from a
//! peer that writes them out itself.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::Agent;
use plana_core::AgentBadge;

/// One `Monitor`-namespace frame, serde-tagged by `action`.
///
/// Wire shape: `{"type": "Monitor", "data": {"action": "GetMetrics", ...}}`.
/// The bridge defines no `MONITOR_*` constant, so a caller either builds the
/// method string through `core_message_to_method_and_params` or writes it out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum MonitorMessage {
    /// `Monitor.GetMetrics` — one-shot sample request. `agent_type` selects one
    /// agent; `None`, which is also the value of a missing key, asks for the host
    /// as a whole. Answered by `MetricsResponse`.
    GetMetrics {
        /// Agent to sample; `None` (also the value of a missing key) asks for
        /// the host as a whole.
        agent_type: Option<Agent>,
    },
    /// `Monitor.MetricsResponse` — the sample itself, sent both as the answer to
    /// `GetMetrics` and once per `SubscribeMetrics` tick. It names no agent, so a
    /// subscription covering several agents cannot attribute a sample to one.
    MetricsResponse {
        /// The sampled figures, at one instant.
        metrics: MetricsData,
    },
    /// `Monitor.SubscribeMetrics` — starts a periodic push for the listed
    /// `agent_types`. `interval` is the cadence as a bare integer: the unit is
    /// not on the wire and nothing in this workspace produces or consumes the
    /// field, so the peers have to agree on it out of band.
    SubscribeMetrics {
        agent_types: Vec<Agent>,
        interval: u64,
    },
    /// `Monitor.UnsubscribeMetrics` — stops the periodic push. It carries no
    /// payload, so it cannot name which subscription to end.
    UnsubscribeMetrics,
}

/// One metrics sample: the host or agent counters read at `timestamp`.
///
/// Every value is a bare `f64` with no unit on the wire (the sibling
/// `ProxySystemInfo` in `plana-celestia-types` repeats the same three names
/// with the same ambiguity), so a consumer must take the unit from the
/// producer rather than assume a scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// When the counters were read, in UTC. It is the only ordering key: the
    /// sample has no sequence number and no agent id.
    pub timestamp: DateTime<Utc>,
    /// CPU utilisation sample; fraction and percent are both consistent with the
    /// field, which fixes no scale.
    pub cpu_usage: f64,
    /// Memory utilisation sample, subject to the same unstated scale as
    /// `cpu_usage`.
    pub memory_usage: f64,
    /// Disk utilisation sample, subject to the same unstated scale as
    /// `cpu_usage`.
    pub disk_usage: f64,
    /// Network throughput sample in an unstated unit — neither the rate basis
    /// nor bytes-versus-bits is on the wire.
    pub network_throughput: f64,
    /// Producer-defined extras keyed by metric name; anything the fixed fields
    /// cannot express lands here, so a consumer must tolerate unknown keys.
    pub custom_metrics: std::collections::HashMap<String, serde_json::Value>,
}

/// One cosmos (agent sandbox) container as the control plane sees it, carried
/// as `Sync.VmSnapshot.container_info`.
///
/// It identifies a container and its IPC endpoint only; live state such as
/// status or resource use travels in `ContainerInfo` and `Sync.ContainerPatch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosContainerInfo {
    /// Runtime container id; distinct from `instance_uuid` below, which is the
    /// per-instance key used for IPC.
    pub container_id: String,
    /// Runtime container name (the agent-scoped name the container was created
    /// with).
    pub container_name: String,
    /// Agent kind this container hosts, as a raw string rather than the `Agent`
    /// enum the rest of this crate uses.
    pub agent_type: String,
    /// Per-instance uuid that keys the container IPC socket: the cosmos socket
    /// helper resolves it to a file under the cosmos socket directory.
    pub instance_uuid: Uuid,
    /// Socket the control plane dials for this container, as a filesystem path
    /// (not a URL); the cosmos socket helper derives it from `instance_uuid`.
    pub socket_path: String,
    /// Image reference the sandbox was created from.
    pub image: String,
    /// Cosmos branch label of this container; `None` — also the value of a
    /// missing key — when the container sits on no branch.
    #[serde(default)]
    pub branch: Option<String>,
    /// Instance badge (`AgentBadge`: a three-digit number, a `parent.NNN`
    /// sub-badge, or `demiurge`); `None` when no badge is assigned.
    #[serde(default)]
    pub badge: Option<AgentBadge>,
}

/// One line of a container tool-invocation log, carried as an entry of
/// `Sync.VmSnapshot.op_log`.
///
/// The two previews are already truncated by the producer: they are display
/// strings, not re-parsable payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosOperationLogEntry {
    /// When the call happened, formatted by the producer as a string rather than
    /// a typed timestamp.
    pub timestamp: String,
    /// Name of the invoked tool. It is the row's only identifier: the entry
    /// carries no call id and the previews are not attributable.
    pub tool_name: String,
    /// Truncated rendering of the call arguments, for display only.
    pub params_preview: String,
    /// Whether the call succeeded. It is the only machine-readable outcome on the
    /// row; `result_preview` and `error` are free text.
    pub success: bool,
    /// Truncated rendering of the result, for display only.
    pub result_preview: String,
    /// Error text of a failed call, split into one string per line.
    pub error: Vec<String>,
}
