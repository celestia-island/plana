//! Knowledge-base (document / RAG) wire types.
//!
//! Requests travel client to server as `Sync.CreateKnowledgeBase`,
//! `Sync.AddDocument`, `Sync.QueryKnowledgeBase`, the base CRUD and the
//! subscription CRUD; every `*Response` is the server's one-way reply
//! (`groups::knowledge_base`, `message/types/groups.rs:497`, lists the names and
//! directions).
//!
//! Field names and enum variant names are the JSON keys and values verbatim:
//! none of these types carries a `rename` or `rename_all`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Indexing lifecycle of one knowledge base, reported as `KnowledgeBaseInfo.status`.
///
/// Wire form: unit variants serialize as their exact name (`"Ready"`, no
/// `rename_all`), and the TS binding pins the same four literals
/// (`packages/celestia-types/bindings/ws/core.ts:17`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeBaseStatus {
    /// No index build has run for this base yet.
    Uninitialized,
    /// An index build is running.
    Indexing,
    /// The index is built and the base is queryable.
    Ready,
    /// The last index build failed; no error text is carried, the payload has no
    /// field for it.
    Error,
}

/// Embedding backend of a knowledge base, chosen in `CreateKnowledgeBaseRequest`
/// and echoed by `KnowledgeBaseInfo`.
///
/// Wire form: exact variant names (`"OpenAiSmall"`,
/// `packages/celestia-types/bindings/ws/core.ts:15`). The concrete hosted model
/// id is not part of the protocol; only `Custom` names an endpoint, through
/// `custom_embedding_endpoint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// Small hosted OpenAI embedding model.
    OpenAiSmall,
    /// Large hosted OpenAI embedding model.
    OpenAiLarge,
    /// Ada-generation OpenAI embedding model.
    OpenAiAda,
    /// No built-in model: the base calls `custom_embedding_endpoint` instead.
    Custom,
}

/// Kind of source a subscription pulls documents from, sent in
/// `CreateSubscriptionRequest`.
///
/// Wire form: exact variant names (`"GitHubRepo"` and so on); the variants
/// decide how the request's `url` is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionType {
    /// A GitHub repository, as opposed to a generic git remote.
    GitHubRepo,
    /// A git remote hosted somewhere other than GitHub.
    GitRepo,
    /// A web page or site crawled into documents.
    Website,
    /// An RSS or Atom feed.
    Rss,
    /// A directory on the server's filesystem.
    LocalDirectory,
}

/// Sync state of one subscription.
///
/// No `SyncMessage` variant in this crate carries it yet: bases report only
/// `KnowledgeBaseStatus` and `subscription_count`. Wire form: exact variant
/// names (`"NotSynced"` and so on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    /// Never synced since it was created.
    NotSynced,
    /// A sync run is in flight.
    Syncing,
    /// The last sync run finished successfully.
    Synced,
    /// The last sync run failed.
    Error,
    /// Automatic syncing is switched off until the subscription is resumed.
    Paused,
}

/// Indexing state of a single document.
///
/// As with `SubscriptionStatus`, nothing in this crate's `SyncMessage` carries
/// it yet: `AddDocumentResponse` only reports the new document id. Wire form:
/// exact variant names (`"Pending"` and so on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStatus {
    /// Stored, but not handed to the indexer yet.
    Pending,
    /// Embedding and indexing are in flight.
    Indexing,
    /// Indexed and therefore retrievable.
    Indexed,
    /// Indexing failed for this document.
    Error,
}

/// Body of `Sync.CreateKnowledgeBase` (client to server); answered by
/// `CreateKnowledgeBaseResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    /// Label of the new base; identity is the generated `knowledge_base_id`, not this
    /// name.
    pub name: String,
    /// Free-text description for the dashboard; absent means none.
    #[serde(default)]
    pub description: Option<String>,
    /// Backend that builds the index; absent means the server chooses one.
    #[serde(default)]
    pub embedding_model: Option<EmbeddingModel>,
    /// Endpoint to call when `embedding_model` is `Custom`; not used by the hosted
    /// choices.
    #[serde(default)]
    pub custom_embedding_endpoint: Option<String>,
    /// Tags matched by `KnowledgeBaseFilters.tags`; absent means no tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Reply to `Sync.CreateKnowledgeBase` (server to client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseResponse {
    /// Id of the created base - the handle every later request uses.
    pub knowledge_base_id: Uuid,
    /// Whether the base was created; on `false` only `error` is meaningful.
    pub success: bool,
    /// Failure reason; optional, so senders omit it on success.
    #[serde(default)]
    pub error: Option<String>,
}

/// Body of `Sync.AddDocument` (client to server): one document to store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    /// Base the document is added to.
    pub knowledge_base_id: Uuid,
    /// Raw document text - this is what the base embeds for retrieval.
    pub content: String,
    /// Title shown with the document and echoed per result chunk; absent means
    /// untitled.
    #[serde(default)]
    pub title: Option<String>,
    /// Origin URL when the document came from the web; absent means no source.
    #[serde(default)]
    pub source_url: Option<String>,
    /// Caller string metadata stored with the document; arbitrary keys, absent means
    /// none.
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Reply to `Sync.AddDocument` (server to client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentResponse {
    /// Id assigned to the stored document.
    pub document_id: Uuid,
    /// Whether the document was accepted for indexing.
    pub success: bool,
    /// Failure reason; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
}

/// Body of `Sync.QueryKnowledgeBase` (client to server): a retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryKnowledgeBaseRequest {
    /// Base to search; absent means the choice is left to the server and is not
    /// pinned by this type.
    #[serde(default)]
    pub knowledge_base_id: Option<Uuid>,
    /// Query text, embedded and matched against the base's index.
    pub query: String,
    /// Upper bound on returned chunks; absent means the server default.
    #[serde(default)]
    pub top_k: Option<usize>,
    /// Lower score bound for returned chunks; absent means the server default (the
    /// score scale is not pinned here, see `QueryResultChunk.score`).
    #[serde(default)]
    pub score_threshold: Option<f64>,
}

/// One retrieved passage inside `QueryKnowledgeBaseResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultChunk {
    /// Document the passage came from, as returned by `AddDocumentResponse`.
    pub document_id: Uuid,
    /// Title of that document when it had one.
    #[serde(default)]
    pub document_title: Option<String>,
    /// The retrieved passage text.
    pub content: String,
    /// Ranking score of the passage; the scale is not pinned by this crate
    /// (`score_threshold` filters on the same value).
    pub score: f64,
    /// Origin URL of the source document when it had one.
    #[serde(default)]
    pub source_url: Option<String>,
}

/// Reply to `Sync.QueryKnowledgeBase` (server to client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryKnowledgeBaseResponse {
    /// Retrieved passages; empty when nothing passed the threshold.
    pub results: Vec<QueryResultChunk>,
    /// Whether the query executed; on `false` only `error` is meaningful.
    pub success: bool,
    /// Failure reason; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
}

/// Body of `Sync.CreateSubscription` (client to server): attach a sync source to a
/// base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    /// Base the synced documents are added to.
    pub knowledge_base_id: Uuid,
    /// Kind of source being subscribed; decides how `url` is read.
    pub subscription_type: SubscriptionType,
    /// Address of the source (repository, site or feed) as the server should fetch
    /// it.
    pub url: String,
    /// Filesystem path the sync reads, for source kinds that live on disk; absent
    /// means not a local source.
    #[serde(default)]
    pub sync_path: Option<String>,
    /// Hours between automatic sync runs; absent means the server's schedule.
    #[serde(default)]
    pub sync_interval_hours: Option<u64>,
    /// Display name of the subscription; absent means server-derived.
    #[serde(default)]
    pub name: Option<String>,
}

/// Reply to `Sync.CreateSubscription` (server to client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionResponse {
    /// Id of the created subscription, used by the sync and delete requests.
    pub subscription_id: Uuid,
    /// Whether the subscription was created.
    pub success: bool,
    /// Failure reason; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
}

/// Body of `Sync.SyncSubscription` (client to server): run a sync now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSubscriptionRequest {
    /// Subscription to run now; its base, source kind and schedule are already stored
    /// server-side.
    pub subscription_id: Uuid,
    /// `Some(true)` asks for a full re-fetch instead of the usual incremental update;
    /// absent means the sender left the choice to the server.
    #[serde(default)]
    pub force_full_sync: Option<bool>,
}

/// Reply to `Sync.SyncSubscription` (server to client): what the run changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSubscriptionResponse {
    /// Whether the sync run completed.
    pub success: bool,
    /// Failure reason; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
    /// Documents the run added; absent means the sender reported no count.
    #[serde(default)]
    pub added_documents: Option<usize>,
    /// Documents the run rewrote; absent means the sender reported no count.
    #[serde(default)]
    pub updated_documents: Option<usize>,
    /// Documents the run dropped because the source no longer has them; absent means
    /// the sender reported no count.
    #[serde(default)]
    pub deleted_documents: Option<usize>,
}

/// Reply to `Sync.DeleteSubscription`; the request itself carries only a
/// `subscription_id` (`SyncMessage::DeleteSubscription`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSubscriptionResponse {
    /// Whether the subscription was removed.
    pub success: bool,
    /// Failure reason, for example an unknown id; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
}

/// Optional filter set on `Sync.ListKnowledgeBases`. Both fields are optional and
/// the type derives `Default`, so the default value filters nothing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBaseFilters {
    /// Only list bases in this lifecycle state; absent means any status.
    #[serde(default)]
    pub status: Option<KnowledgeBaseStatus>,
    /// Only list bases carrying these tags; absent means no tag filter.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// One knowledge base as returned by `Sync.GetKnowledgeBase` and
/// `Sync.ListKnowledgeBases`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseInfo {
    /// Id every request that targets this base uses.
    pub id: Uuid,
    /// Display name given at creation.
    pub name: String,
    /// Free-text description; absent means none was set.
    #[serde(default)]
    pub description: Option<String>,
    /// Index lifecycle state of this base.
    pub status: KnowledgeBaseStatus,
    /// Backend the base was created with; absent means the server default is in use.
    #[serde(default)]
    pub embedding_model: Option<EmbeddingModel>,
    /// Endpoint of the `Custom` backend; absent for the hosted choices.
    #[serde(default)]
    pub custom_embedding_endpoint: Option<String>,
    /// Number of documents indexed in the base.
    pub document_count: usize,
    /// Number of sync subscriptions attached to the base.
    pub subscription_count: usize,
    /// Tags set at creation; absent means the empty list.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation time, RFC 3339 on the wire (`DateTime<Utc>`).
    pub created_at: DateTime<Utc>,
    /// Last modification time in the same format; the celestia-types mirror sends
    /// both stamps as plain strings
    /// (`packages/celestia-types/src/ws/services/knowledge_base.rs:42`).
    pub updated_at: DateTime<Utc>,
}

/// Reply to `Sync.DeleteKnowledgeBase`; the request itself carries only a
/// `knowledge_base_id` (`SyncMessage::DeleteKnowledgeBase`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteKnowledgeBaseResponse {
    /// Whether the base and its documents were removed.
    pub success: bool,
    /// Failure reason, for example an unknown id; optional, omitted on success.
    #[serde(default)]
    pub error: Option<String>,
}
