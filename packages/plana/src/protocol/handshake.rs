//! WebSocket connection handshake & client-capability types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Handshake wire-protocol version. Bumped on incompatible changes to the
/// handshake payload itself. OLD clients that omit `protocol_version` still
/// deserialize via the serde default and are treated as v1.
pub const HANDSHAKE_VERSION: u32 = 1;

fn default_protocol_version() -> u32 {
    HANDSHAKE_VERSION
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
    /// [`HANDSHAKE_VERSION`] when absent (backward-compatible with old clients).
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── HandshakeAckParams ──────────────────────────────────────────

    #[test]
    fn handshake_ack_success_round_trip() {
        let ack = HandshakeAckParams {
            ok: true,
            error: None,
        };
        let v = serde_json::to_value(&ack).unwrap();
        assert_eq!(v["ok"], true);
        // error is optional + #[ts(optional)] — serialized as null when None.
        // (No skip_serializing_if, so null appears on the wire.)
        assert_eq!(v["error"], serde_json::Value::Null);
        let back: HandshakeAckParams = serde_json::from_value(v).unwrap();
        assert!(back.ok);
        assert!(back.error.is_none());
    }

    #[test]
    fn handshake_ack_failure_with_message() {
        let ack = HandshakeAckParams {
            ok: false,
            error: Some("bad token".into()),
        };
        let v = serde_json::to_value(&ack).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"], "bad token");
        let back: HandshakeAckParams = serde_json::from_value(v).unwrap();
        assert!(!back.ok);
        assert_eq!(back.error.as_deref(), Some("bad token"));
    }

    #[test]
    fn handshake_ack_accepts_missing_error_field() {
        // #[serde(default)] allows omitting error on deserialize.
        let raw = json!({"ok": true});
        let ack: HandshakeAckParams = serde_json::from_value(raw).unwrap();
        assert!(ack.ok);
        assert!(ack.error.is_none());
    }

    // ── ScepterIdentityParams ───────────────────────────────────────

    #[test]
    fn scepter_identity_uuid_serializes_as_string() {
        let uid = uuid::Uuid::new_v4();
        let p = ScepterIdentityParams { device_id: uid };
        let v = serde_json::to_value(&p).unwrap();
        // Uuid serializes as a JSON string (hyphenated).
        assert_eq!(v["device_id"], uid.to_string());
        let back: ScepterIdentityParams = serde_json::from_value(v).unwrap();
        assert_eq!(back.device_id, uid);
    }

    // ── PingParams ──────────────────────────────────────────────────

    #[test]
    fn ping_params_round_trip() {
        let p = PingParams {
            timestamp: 1700000000,
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["timestamp"], 1700000000);
        // No extra fields.
        assert_eq!(v.as_object().unwrap().len(), 1);
        let back: PingParams = serde_json::from_value(v).unwrap();
        assert_eq!(back.timestamp, 1700000000);
    }

    #[test]
    fn ping_params_zero_timestamp() {
        let p = PingParams { timestamp: 0 };
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(s, r#"{"timestamp":0}"#);
    }

    // ── ClientCapability enum ───────────────────────────────────────

    #[test]
    fn client_capability_snake_case_serialization() {
        let caps = vec![
            ClientCapability::FileRelay,
            ClientCapability::Terminal,
            ClientCapability::ScreenCapture,
            ClientCapability::NoaWorkspace,
        ];
        let v = serde_json::to_value(&caps).unwrap();
        assert_eq!(
            v,
            json!(["file_relay", "terminal", "screen_capture", "noa_workspace"])
        );
        let back: Vec<ClientCapability> = serde_json::from_value(v).unwrap();
        assert_eq!(back, caps);
    }

    #[test]
    fn client_capability_round_trip_each_variant() {
        for cap in [
            ClientCapability::FileRelay,
            ClientCapability::Terminal,
            ClientCapability::ScreenCapture,
            ClientCapability::NoaWorkspace,
        ] {
            let s = serde_json::to_string(&cap).unwrap();
            let back: ClientCapability = serde_json::from_str(&s).unwrap();
            assert_eq!(back, cap);
        }
    }

    #[test]
    fn client_capability_rejects_unknown_variant() {
        assert!(serde_json::from_str::<ClientCapability>("\"nonexistent\"").is_err());
    }

    // ── ClientNodeInfo ──────────────────────────────────────────────

    #[test]
    fn client_node_info_full_round_trip() {
        let info = ClientNodeInfo {
            hostname: "gpu-box-01".into(),
            os: "linux".into(),
            workspace_root: Some("/home/user/proj".into()),
            user_id: Some("u42".into()),
        };
        let v = serde_json::to_value(&info).unwrap();
        assert_eq!(v["hostname"], "gpu-box-01");
        assert_eq!(v["os"], "linux");
        assert_eq!(v["workspace_root"], "/home/user/proj");
        assert_eq!(v["user_id"], "u42");
        let back: ClientNodeInfo = serde_json::from_value(v).unwrap();
        assert_eq!(back.hostname, "gpu-box-01");
        assert_eq!(back.workspace_root.as_deref(), Some("/home/user/proj"));
    }

    #[test]
    fn client_node_info_optional_fields_serialize_as_null() {
        let info = ClientNodeInfo {
            hostname: "h".into(),
            os: "linux".into(),
            workspace_root: None,
            user_id: None,
        };
        let v = serde_json::to_value(&info).unwrap();
        // #[ts(optional)] without skip_serializing_if → null on wire.
        assert_eq!(v["workspace_root"], serde_json::Value::Null);
        assert_eq!(v["user_id"], serde_json::Value::Null);
    }

    #[test]
    fn client_node_info_accepts_missing_optional_fields() {
        let raw = json!({"hostname": "h", "os": "win"});
        let info: ClientNodeInfo = serde_json::from_value(raw).unwrap();
        assert!(info.workspace_root.is_none());
        assert!(info.user_id.is_none());
    }

    // ── ConnectHandshakeParams ──────────────────────────────────────

    #[test]
    fn connect_handshake_full_round_trip() {
        let p = ConnectHandshakeParams {
            protocol_version: 2,
            token: "tok-abc".into(),
            session_id: Some("sess-1".into()),
            capabilities: vec![ClientCapability::Terminal, ClientCapability::NoaWorkspace],
            node_info: Some(ClientNodeInfo {
                hostname: "host".into(),
                os: "linux".into(),
                workspace_root: None,
                user_id: None,
            }),
            workspace_id: Some("550e8400-e29b-41d4-a716-446655440000".into()),
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["protocol_version"], 2);
        assert_eq!(v["token"], "tok-abc");
        assert_eq!(v["session_id"], "sess-1");
        assert_eq!(v["capabilities"][0], "terminal");
        assert_eq!(v["capabilities"][1], "noa_workspace");
        assert_eq!(v["node_info"]["hostname"], "host");
        assert_eq!(v["workspace_id"], "550e8400-e29b-41d4-a716-446655440000");
        let back: ConnectHandshakeParams = serde_json::from_value(v).unwrap();
        assert_eq!(back.protocol_version, 2);
        assert_eq!(back.token, "tok-abc");
        assert_eq!(back.capabilities.len(), 2);
    }

    #[test]
    fn connect_handshake_defaults_protocol_version_when_absent() {
        // Old clients that omit protocol_version must default to v1.
        let raw = json!({"token": "tok"});
        let p: ConnectHandshakeParams = serde_json::from_value(raw).unwrap();
        assert_eq!(p.protocol_version, HANDSHAKE_VERSION);
        assert_eq!(p.protocol_version, 1);
        assert!(p.session_id.is_none());
        assert!(p.capabilities.is_empty());
        assert!(p.node_info.is_none());
        assert!(p.workspace_id.is_none());
    }

    #[test]
    fn connect_handshake_explicit_v1() {
        let raw = json!({"protocol_version": 1, "token": "tok"});
        let p: ConnectHandshakeParams = serde_json::from_value(raw).unwrap();
        assert_eq!(p.protocol_version, 1);
    }

    #[test]
    fn connect_handshake_empty_capabilities() {
        let raw = json!({"token": "tok", "capabilities": []});
        let p: ConnectHandshakeParams = serde_json::from_value(raw).unwrap();
        assert!(p.capabilities.is_empty());
    }

    #[test]
    fn connect_handshake_capabilities_default_when_absent() {
        let raw = json!({"token": "tok"});
        let p: ConnectHandshakeParams = serde_json::from_value(raw).unwrap();
        assert!(
            p.capabilities.is_empty(),
            "capabilities must default to empty vec"
        );
    }

    #[test]
    fn connect_handshake_serialized_includes_protocol_version() {
        let p = ConnectHandshakeParams {
            protocol_version: 1,
            token: "t".into(),
            session_id: None,
            capabilities: vec![],
            node_info: None,
            workspace_id: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        // protocol_version always appears (no skip_serializing_if).
        assert!(v.get("protocol_version").is_some());
        assert_eq!(v["protocol_version"], 1);
    }

    #[test]
    fn connect_handshake_missing_token_rejected() {
        let raw = json!({"protocol_version": 1});
        assert!(serde_json::from_value::<ConnectHandshakeParams>(raw).is_err());
    }

    #[test]
    fn connect_handshake_accepts_unknown_fields() {
        // Forward-compat: new fields from a newer client should not break deserialization.
        let raw = json!({"token": "t", "future_field": 42});
        let p: ConnectHandshakeParams = serde_json::from_value(raw).unwrap();
        assert_eq!(p.token, "t");
    }

    // ── HANDSHAKE_VERSION constant ──────────────────────────────────

    #[test]
    fn handshake_version_is_one() {
        assert_eq!(HANDSHAKE_VERSION, 1);
    }
}
