use super::enums::WebSearchEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct WebSearchItem {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct WebSearchResult {
    pub query: String,
    pub engine: WebSearchEngine,
    pub count: usize,
    pub results: Vec<WebSearchItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct WebFetchResult {
    pub url: String,
    pub title: String,
    pub status_code: u16,
    pub headers: String,
    pub content: String,
    pub content_preview: String,
    pub content_length: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct RemoteRefEntry {
    pub ref_id: String,
    pub url: String,
    pub title: String,
    pub ref_type: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct QueryRemoteRefsResult {
    pub count: usize,
    pub refs: Vec<RemoteRefEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/eleos.ts")]
pub struct RegisterRemoteRefsResult {
    pub ref_id: String,
    pub url: String,
    pub registered: bool,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WebFetchParams {
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WebSearchParams {
    pub query: String,
    pub engine: Option<String>,
    pub limit: Option<u64>,
}
