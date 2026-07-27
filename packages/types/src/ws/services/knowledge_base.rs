//! Knowledge Base — RAG store info & lifecycle responses.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{EmbeddingModel, KnowledgeBaseStatus};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/knowledgeBase.ts")]
pub struct KbGenericResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/knowledgeBase.ts")]
pub struct KnowledgeBaseInfo {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: KnowledgeBaseStatus,
    #[serde(default)]
    #[ts(optional)]
    pub embedding_model: Option<EmbeddingModel>,
    #[serde(default)]
    #[ts(optional)]
    pub custom_embedding_endpoint: Option<String>,
    pub document_count: usize,
    #[serde(default)]
    pub subscription_count: usize,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/knowledgeBase.ts")]
pub struct ListKnowledgeBasesResponseParams {
    pub knowledge_bases: Vec<KnowledgeBaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/knowledgeBase.ts")]
pub struct GetKnowledgeBaseResponseParams {
    #[serde(default)]
    #[ts(optional)]
    pub knowledge_base: Option<KnowledgeBaseInfo>,
}
