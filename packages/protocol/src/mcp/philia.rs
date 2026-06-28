use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AgentRegistryEntry {
    pub agent_type: String,
    pub status: String,
    pub mcp_tool_count: usize,
    pub skill_count: usize,
    pub mcp_tools: Vec<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AgentRegistryListResult {
    pub agents: Vec<AgentRegistryEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct McpToolDetail {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SkillDetail {
    pub name: String,
    pub description: String,
    pub related_tools: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AgentRegistryGetResult {
    pub agent_type: String,
    pub mcp_tools: Vec<McpToolDetail>,
    pub skills: Vec<SkillDetail>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DataStoreSaveResult {
    pub key: String,
    pub namespace: String,
    pub store_key: String,
    pub saved_at: String,
    pub value: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DataStoreLoadResult {
    pub key: String,
    pub namespace: String,
    pub store_key: String,
    pub loaded_at: String,
    pub value: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContextPrepareResult {
    pub episode_count: usize,
    pub entity_count: usize,
    pub relevant_nodes: usize,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryStoreResult {
    pub node_id: String,
    pub node_type: String,
    pub text_preview: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryQueryItem {
    pub node_type: String,
    pub text: String,
    pub score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryQueryResult {
    pub query: String,
    pub total: usize,
    pub results: Vec<MemoryQueryItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryConsolidateResult {
    pub episode_id: String,
    pub linked_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct MemorySubgraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f64,
    pub metadata: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct MemoryNodeFull {
    pub id: String,
    pub node_type: String,
    pub text: String,
    pub score: f64,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub source: Option<String>,
    pub metadata: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct MemorySubgraphResult {
    pub query: String,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub nodes: Vec<MemoryNodeFull>,
    pub edges: Vec<MemorySubgraphEdge>,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryStoreParams {
    pub text: String,
    pub node_type: String,
    pub entity_type: Option<String>,
    pub source_episode_id: Option<String>,
    pub related_node_ids: Option<Vec<String>>,
    pub properties: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryQueryParams {
    pub query: String,
    pub limit: Option<u64>,
    pub graph_depth: Option<u64>,
    pub node_type_filter: Option<String>,
    pub subgraph: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MemoryConsolidateParams {
    pub episode_focus: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContextPrepareParams {
    pub query: String,
    pub max_nodes: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct TimeseriesQueryParams {
    pub metric: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub include_stats: Option<bool>,
    pub limit: Option<u64>,
    pub tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DataQualityCheckParams {
    pub metric: String,
    pub expected_interval_ms: Option<u64>,
    pub stale_threshold_ms: Option<u64>,
    pub z_score_threshold: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ToolSchemaGetParams {
    pub agent_type: String,
    pub tool_name: String,
}

// ── Tool result structs for timeseries / registry tools ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct ToolSchemaGetResult {
    pub agent_type: String,
    pub tool_name: String,
    pub declaration: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct TimeseriesPointResult {
    pub timestamp: i64,
    pub value: f64,
    pub tags: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct QueryStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct TimeseriesQueryResult {
    pub metric: String,
    pub points: Vec<TimeseriesPointResult>,
    pub stats: Option<QueryStats>,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct GapInfo {
    pub start_time: i64,
    pub end_time: i64,
    pub estimated_missing: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/philia.ts")]
pub struct QualityReport {
    pub metric: String,
    pub total_points: usize,
    pub completeness: f64,
    pub expected_points: usize,
    pub missing_points: usize,
    pub duplicate_points: usize,
    pub stale_points: usize,
    pub outlier_count: usize,
    pub outlier_ratio: f64,
    pub gaps: Vec<GapInfo>,
    pub score: f64,
}
