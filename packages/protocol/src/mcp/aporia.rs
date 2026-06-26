use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RagDbWriteResult {
    pub doc_id: Uuid,
    pub embedding_dim: usize,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RagDocResult {
    pub doc_id: Uuid,
    pub similarity: f64,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RagDbReadResult {
    pub count: usize,
    pub results: Vec<RagDocResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RagDbDeleteResult {
    pub doc_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LlmChatResult {
    pub model: String,
    pub tokens: String,
    pub response: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslateReportResult {
    pub target_language: String,
    pub original_length: usize,
    pub translated_length: usize,
    pub translation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MediaAssetRegisterResult {
    pub asset_id: Uuid,
    pub asset_type: String,
    pub source_url: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MediaAssetItem {
    pub asset_id: Uuid,
    pub asset_type: String,
    pub source_url: String,
    pub metadata: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MediaAssetRetrieveResult {
    pub count: usize,
    pub assets: Vec<MediaAssetItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WorkspaceIndexResult {
    pub total_files: usize,
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WorkspaceSearchDoc {
    pub doc_id: Uuid,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: String,
    pub similarity: f64,
    pub snippet: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WorkspaceSearchResult {
    pub count: usize,
    pub results: Vec<WorkspaceSearchDoc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WorkspaceStatusResult {
    pub total_files: usize,
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub last_indexed: Option<String>,
    pub is_indexing: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RagDbStatsResult {
    pub total_documents: usize,
    pub total_media_assets: usize,
    pub embedding_dimensions: Option<usize>,
    pub storage_backend: String,
}

// ── Tool result structs (analysis) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CorrelationInfo {
    pub variable: String,
    pub correlation: f64,
    pub lag: usize,
    pub direction: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Hypothesis {
    pub cause: String,
    pub effect: String,
    pub strength: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CausalReasonResult {
    pub correlations: Vec<CorrelationInfo>,
    pub hypotheses: Vec<Hypothesis>,
    pub recommended_actions: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AnomalyInfo {
    pub index: usize,
    pub value: f64,
    pub expected: f64,
    pub deviation: f64,
    pub timestamp: Option<i64>,
    pub severity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AnomalyResult {
    pub anomalies: Vec<AnomalyInfo>,
    pub total_points: usize,
    pub anomaly_count: usize,
    pub anomaly_ratio: f64,
    pub method: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct RagDbWriteParams {
    pub content: String,
    #[serde(default)]
    pub embedding: Option<Vec<f64>>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct RagDbReadParams {
    pub query_embedding: Vec<f64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct RagDbDeleteParams {
    pub id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AnomalyDetectParams {
    pub values: Vec<f64>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub threshold: Option<f64>,
    #[serde(default)]
    pub timestamps: Option<Vec<i64>>,
    #[serde(default)]
    pub window_size: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct CausalReasonParams {
    pub target: String,
    pub target_values: Vec<f64>,
    #[serde(default)]
    pub candidates: Option<std::collections::HashMap<String, Vec<f64>>>,
    #[serde(default)]
    pub max_lag: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WorkspaceIndexParams {
    pub workspace_root: String,
    #[serde(default)]
    pub full_rebuild: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WorkspaceSearchParams {
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct LlmChatParams {
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct TranslateReportParams {
    pub content: String,
    #[serde(default)]
    pub target_language: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct RagDbStatsParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WorkspaceStatusParams {}
