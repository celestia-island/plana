use serde::{Deserialize, Serialize};

/// A persisted message row returned by the paginated history RPCs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    /// Topic grouping: the conversation this row belongs to (one
    /// conversation per user turn). Optional so legacy rows/payloads
    /// without the column still deserialize.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesPage {
    pub messages: Vec<HistoryMessage>,
    /// Whether older messages exist beyond this page (for "load more on scroll
    /// to top").
    pub has_more: bool,
    /// ISO-8601 timestamp of the oldest message in this page (cursor for the
    /// next `RequestOlderMessages` call).
    pub oldest_created_at: Option<String>,
}
