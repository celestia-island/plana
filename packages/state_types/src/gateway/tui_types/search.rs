use serde::{Deserialize, Serialize};

/// A single semantic-search hit returned by the vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    /// Compact one-line preview of the matched content.
    pub snippet: String,
    /// Full matched content (used when opening the detail view).
    pub content: String,
    pub score: f32,
    /// Vector-store source tag, e.g. `"report"`, `"knowledge"`,
    /// `"workspace_indexer"`.
    pub source: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub total: u64,
    pub results: Vec<SearchHit>,
}
