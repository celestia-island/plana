//! WebSocket connection handshake & client-capability types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Handshake wire-protocol version. Bumped on incompatible changes to the
/// handshake payload itself. OLD clients that omit `protocol_version` still
/// deserialize via [`default_protocol_version`] and are treated as v1.
pub const PROTOCOL_VERSION: u32 = 1;

fn default_protocol_version() -> u32 {
    PROTOCOL_VERSION
}

// ═══════════════════════════════════════════════════════════════
// Connection / Handshake
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/handshake.ts")]
pub struct HandshakeAckParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/handshake.ts")]
pub struct ScepterIdentityParams {
    #[ts(type = "string")]
    pub device_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/handshake.ts")]
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
#[ts(export, export_to = "ws/handshake.ts")]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    FileRelay,
    Terminal,
    ScreenCapture,
    NoaWorkspace,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/handshake.ts")]
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
#[ts(export, export_to = "ws/handshake.ts")]
pub struct ConnectHandshakeParams {
    /// Handshake wire-protocol version advertised by the client. Defaults to
    /// [`PROTOCOL_VERSION`] when absent (backward-compatible with old clients).
    #[serde(default = "default_protocol_version")]
    pub protocol_version: u32,
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
