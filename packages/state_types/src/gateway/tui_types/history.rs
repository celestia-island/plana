//! Paginated conversation-history rows served by `Sync.MessagesResponse`:
//! the recent tail and the scroll-up older pages.
use serde::{Deserialize, Serialize};

/// A persisted message row returned by the paginated history RPCs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    /// Row id of the persisted message.
    pub id: String,
    /// Author role of the message — the `system` / `user` / `assistant` / `tool`
    /// tokens of the family's `MessageRole`; carried through as stored.
    pub role: String,
    /// Message body as stored.
    pub content: String,
    /// Creation time of the row as an ISO-8601 string — the value the older-page
    /// pagination compares against.
    pub created_at: String,
    /// Topic grouping: the conversation this row belongs to (one
    /// conversation per user turn). Optional so legacy rows/payloads
    /// without the column still deserialize.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

/// One page of conversation history: the rows plus the cursor needed to walk
/// further back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesPage {
    /// Rows in this page; the cursor for fetching what precedes them is
    /// `oldest_created_at`.
    pub messages: Vec<HistoryMessage>,
    /// Whether older messages exist beyond this page (for "load more on scroll
    /// to top").
    pub has_more: bool,
    /// ISO-8601 timestamp of the oldest message in this page (cursor for the
    /// next `RequestOlderMessages` call).
    pub oldest_created_at: Option<String>,
}
