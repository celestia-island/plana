use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeBaseStatus {
    Uninitialized,
    Indexing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingModel {
    OpenAiSmall,
    OpenAiLarge,
    OpenAiAda,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionType {
    GitHubRepo,
    GitRepo,
    Website,
    Rss,
    LocalDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    NotSynced,
    Syncing,
    Synced,
    Error,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStatus {
    Pending,
    Indexing,
    Indexed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub embedding_model: Option<EmbeddingModel>,
    #[serde(default)]
    pub custom_embedding_endpoint: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgeBaseResponse {
    pub knowledge_base_id: Uuid,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub knowledge_base_id: Uuid,
    pub content: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentResponse {
    pub document_id: Uuid,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryKnowledgeBaseRequest {
    #[serde(default)]
    pub knowledge_base_id: Option<Uuid>,
    pub query: String,
    #[serde(default)]
    pub top_k: Option<usize>,
    #[serde(default)]
    pub score_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultChunk {
    pub document_id: Uuid,
    #[serde(default)]
    pub document_title: Option<String>,
    pub content: String,
    pub score: f64,
    #[serde(default)]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryKnowledgeBaseResponse {
    pub results: Vec<QueryResultChunk>,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub knowledge_base_id: Uuid,
    pub subscription_type: SubscriptionType,
    pub url: String,
    #[serde(default)]
    pub sync_path: Option<String>,
    #[serde(default)]
    pub sync_interval_hours: Option<u64>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionResponse {
    pub subscription_id: Uuid,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSubscriptionRequest {
    pub subscription_id: Uuid,
    #[serde(default)]
    pub force_full_sync: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSubscriptionResponse {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub added_documents: Option<usize>,
    #[serde(default)]
    pub updated_documents: Option<usize>,
    #[serde(default)]
    pub deleted_documents: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSubscriptionResponse {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBaseFilters {
    #[serde(default)]
    pub status: Option<KnowledgeBaseStatus>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: KnowledgeBaseStatus,
    #[serde(default)]
    pub embedding_model: Option<EmbeddingModel>,
    #[serde(default)]
    pub custom_embedding_endpoint: Option<String>,
    pub document_count: usize,
    pub subscription_count: usize,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteKnowledgeBaseResponse {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}
