//! WebSocket connection handshake primitives — the generic part of the
//! connection handshake shared by every platform profile.
//!
//! Handshake wire-protocol version negotiation ([`HANDSHAKE_VERSION`],
//! [`HandshakeAckParams`]) and keepalive ([`PingParams`]) are protocol-core
//! concepts. The client capability vocabulary and the concrete connection
//! payload that references it live in the domain profile crates.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Handshake wire-protocol version. Bumped on incompatible changes to the
/// handshake payload itself. OLD clients that omit `protocol_version` still
/// deserialize via the serde default and are treated as v1.
pub const HANDSHAKE_VERSION: u32 = 1;

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
pub struct PingParams {
    pub timestamp: u64,
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

    // ── HANDSHAKE_VERSION constant ──────────────────────────────────

    #[test]
    fn handshake_version_is_one() {
        assert_eq!(HANDSHAKE_VERSION, 1);
    }
}
