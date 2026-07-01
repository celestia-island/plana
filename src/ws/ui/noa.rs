//! NOA Workspace — TuiMessage variant params.
//!
//! Mirrors `entelecheia/packages/shared/state_types/src/gateway/
//! tui_types/message/types/mod.rs`. The NOA handshake is a 4-message round trip:
//!
//!   scepter → client   RequestNoaHandshake
//!   client  → scepter  NoaHandshakeResponse
//!   scepter → client   NoaAuthRequest   (branch picker)
//!   client  → scepter  NoaAuthResponse  (user's choice)
//!   scepter → client   NoaReady         (terminal event)
//!
//! Plus a bidirectional event-sync pair used after NoaReady.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    #[serde(default)]
    #[ts(optional)]
    pub file_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub content_hash: Option<String>,
    #[serde(default)]
    #[ts(optional, type = "Record<string, unknown>")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct RequestNoaHandshakeParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub remote_name: String,
    pub remote_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaHandshakeResponseParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub repo_id: String,
    pub current_branch: String,
    #[serde(default)]
    pub noa_initialized: bool,
    #[serde(default)]
    pub gitignore_updated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaAuthRequestParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub branches: Vec<String>,
    pub suggested_branch: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaAuthResponseParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub selected_branch: String,
    #[serde(default)]
    #[ts(optional)]
    pub branch_base: Option<String>,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaReadyParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub branch: String,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaEventSyncParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub events: Vec<NoaEvent>,
    #[serde(default)]
    #[ts(optional)]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/noa.ts")]
pub struct NoaEventSyncAckParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub last_event_id: String,
}
