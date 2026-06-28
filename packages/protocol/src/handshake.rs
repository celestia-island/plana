//! WebSocket connection handshake & client-capability types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Connection / Handshake
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HandshakeAckParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ScepterIdentityParams {
    #[ts(type = "string")]
    pub device_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PingParams {
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════
// Client capability + handshake payload
//
// Mirrors `entelecheia/packages/shared/state_types/src/gateway/
// tui_types/message/types/mod.rs`. The webui declares capabilities
// in its `Tui.ConnectHandshake` so scepter's `client_node_registry`
// can route capability-scoped requests back to it (e.g. NOA
// handshakes are only sent to sessions that declared
// `ClientCapability::NoaWorkspace`).
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    FileRelay,
    Terminal,
    ScreenCapture,
    NoaWorkspace,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ClientNodeInfo {
    pub hostname: String,
    pub os: String,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ConnectHandshakeParams {
    pub token: String,
    #[serde(default)]
    #[ts(optional)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<ClientCapability>,
    #[serde(default)]
    #[ts(optional)]
    pub node_info: Option<ClientNodeInfo>,
    /// Stringified UUID — kept as a string on the wire so consumers
    /// don't need a UUID parser to round-trip the JSON-RPC payload.
    #[serde(default)]
    #[ts(optional)]
    pub workspace_id: Option<String>,
}
