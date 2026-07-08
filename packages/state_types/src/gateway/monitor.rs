use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::Agent;
use arona_core::AgentBadge;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum MonitorMessage {
    GetMetrics {
        agent_type: Option<Agent>,
    },
    MetricsResponse {
        metrics: MetricsData,
    },
    SubscribeMetrics {
        agent_types: Vec<Agent>,
        interval: u64,
    },
    UnsubscribeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_throughput: f64,
    pub custom_metrics: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosContainerInfo {
    pub container_id: String,
    pub container_name: String,
    pub agent_type: String,
    pub instance_uuid: Uuid,
    pub socket_path: String,
    pub image: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub badge: Option<AgentBadge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosOperationLogEntry {
    pub timestamp: String,
    pub tool_name: String,
    pub params_preview: String,
    pub success: bool,
    pub result_preview: String,
    pub error: Vec<String>,
}
