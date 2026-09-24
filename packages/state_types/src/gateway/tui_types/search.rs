//! Semantic-search payloads served by the gateway's vector store
//! (`Sync.SearchRequest` / `Sync.SearchResponse`).
use serde::{Deserialize, Serialize};

/// A single semantic-search hit returned by the vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// Identifier of the indexed chunk inside the producing vector store;
    /// opaque to clients.
    pub id: String,
    /// Compact one-line preview of the matched content.
    pub snippet: String,
    /// Full matched content (used when opening the detail view).
    pub content: String,
    /// Relevance score assigned by the vector store — higher is more relevant,
    /// and `Sync.SearchRequest.min_score` drops hits below a caller threshold.
    pub score: f32,
    /// Vector-store source tag, e.g. `"report"`, `"knowledge"`,
    /// `"workspace_indexer"`.
    pub source: String,
    /// Per-source metadata attached by the indexer; the key set depends on
    /// `source` and is not fixed by this crate.
    pub metadata: serde_json::Value,
}

/// Answer to one semantic-search request: the echoed query, the match total
/// and the page of hits the caller receives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// The query string this response answers.
    pub query: String,
    /// Matches the store counted for the query before the request's `limit` was
    /// applied, so it can exceed `results.len()`.
    pub total: u64,
    /// Hits carried by this response, capped by the request's `limit`
    /// (10 when the caller omits it).
    pub results: Vec<SearchHit>,
}
