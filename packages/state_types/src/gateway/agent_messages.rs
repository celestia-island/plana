use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::{Agent, AgentInfo, AgentRegisterRequest, AgentUnregisterRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AgentMessage {
    Register { request: AgentRegisterRequest },
    Unregister { request: AgentUnregisterRequest },
    ListAgents,
    AgentListResponse { agents: Vec<AgentInfo> },
    GetAgentInfo { agent_type: Agent },
    AgentInfoResponse { info: AgentInfo },
    Ack { message_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum NodeMessage {
    DiscoverNodes,
    NodeListResponse { nodes: Vec<NodeInfo> },
    GetNodeInfo { node_id: String },
    NodeInfoResponse { info: NodeInfo },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_type: String,
    pub hostname: String,
    pub ip_address: String,
    pub port: u16,
    pub online: bool,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub platform_version: Option<String>,
    pub last_online: DateTime<Utc>,
}
