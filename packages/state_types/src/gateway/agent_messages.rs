//! Agent-registry and node-discovery DTOs (`Agent.*`, `Node.*`).
//!
//! Neither namespace has a constant or a parse arm in the JSON-RPC bridge: the
//! generic helpers still derive `Agent.Register` / `Node.DiscoverNodes` from
//! the envelope tags, but parsing such a string back yields
//! `GatewayMethod::Extension` instead of the namespace variant.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::{Agent, AgentInfo, AgentRegisterRequest, AgentUnregisterRequest};

/// One `Agent`-namespace action, serde-tagged by `action`.
///
/// Wire shape: `{"type": "Agent", "data": {"action": "Register", ...}}`.
/// The agent side sends `Register` and `Unregister`, a client or operator sends
/// the queries, the registry answers with the `*Response` verbs and `Ack`, and
/// both sides describe an agent with the same `AgentInfo` record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AgentMessage {
    /// `Agent.Register` — announces this agent with its category, endpoint, tools
    /// and skills (`AgentRegisterRequest`); the registry answers `Ack` once the
    /// record is stored.
    Register { request: AgentRegisterRequest },
    /// `Agent.Unregister` — drops the agent named by `AgentUnregisterRequest`
    /// (type plus id), so a stale registration can be cleared without waiting for
    /// a liveness timeout.
    Unregister { request: AgentUnregisterRequest },
    /// `Agent.ListAgents` — roster request without payload, answered by
    /// `AgentListResponse`. The state-sync dialect spells the same request
    /// `Sync.ListAgents`, which is the one the bridge has a constant for.
    ListAgents,
    /// `Agent.AgentListResponse` — the roster as `AgentInfo` records; a response
    /// describes the full set, not a delta against a previous one.
    AgentListResponse { agents: Vec<AgentInfo> },
    /// `Agent.GetAgentInfo` — asks for one agent, addressed by `agent_type`;
    /// answered by `AgentInfoResponse`.
    GetAgentInfo { agent_type: Agent },
    /// `Agent.AgentInfoResponse` — the single requested `AgentInfo`.
    AgentInfoResponse { info: AgentInfo },
    /// `Agent.Ack` — confirms one earlier message by `message_id`; as with
    /// `Base.Ack`, the id comes from the original frame rather than this payload.
    Ack { message_id: Uuid },
}

/// One `Node`-namespace action, serde-tagged by `action`.
///
/// Discovery describes a peer by hostname, address and port (`NodeInfo`)
/// instead of the device identity used elsewhere in this crate, so the two
/// views of one machine are not interchangeable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum NodeMessage {
    /// `Node.DiscoverNodes` — asks every reachable peer to announce itself;
    /// answered by `NodeListResponse`.
    DiscoverNodes,
    /// `Node.NodeListResponse` — the peers that answered, as `NodeInfo` records.
    NodeListResponse { nodes: Vec<NodeInfo> },
    /// `Node.GetNodeInfo` — asks for one node by `node_id`; answered by
    /// `NodeInfoResponse`.
    GetNodeInfo { node_id: String },
    /// `Node.NodeInfoResponse` — the single requested `NodeInfo`.
    NodeInfoResponse { info: NodeInfo },
}

/// Description of one peer host, mixing identity (`node_id`, `node_type`),
/// address (`hostname`, `ip_address`, `port`) and liveness (`online`,
/// `last_online`).
///
/// The address block is a record of what the peer announced, not a probe
/// result; only `online` and `last_online` speak to reachability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Id the node is addressed by, and the key `GetNodeInfo` expects.
    pub node_id: String,
    /// Role label the node reported; free-form, so a consumer must not switch on
    /// a closed set of values.
    pub node_type: String,
    /// Host name the node announced.
    pub hostname: String,
    /// Address the node announced, kept as an unparsed string.
    pub ip_address: String,
    /// Port the node serves its gateway on.
    pub port: u16,
    /// Liveness as of the last probe, so it is a snapshot value rather than a
    /// live check; `last_online` says when that probe saw the node.
    pub online: bool,
    /// Platform label the node reported; free-form display data, not a
    /// capability statement. `None` when the node reported none.
    #[serde(default)]
    pub platform: Option<String>,
    /// Version string of that platform; `None` when the node reported none.
    #[serde(default)]
    pub platform_version: Option<String>,
    /// When the node was last seen alive, in UTC.
    pub last_online: DateTime<Utc>,
}
