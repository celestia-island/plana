//! Base protocol messages — heartbeat, error, ack.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/baseMessages.ts")]
pub struct BaseHeartbeatParams {
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/baseMessages.ts")]
pub struct BaseErrorParams {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/baseMessages.ts")]
pub struct BaseAckParams {
    pub message_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── BaseHeartbeatParams ─────────────────────────────────────────

    #[test]
    fn heartbeat_round_trip() {
        let h = BaseHeartbeatParams {
            timestamp: 1700000000,
        };
        let s = serde_json::to_string(&h).unwrap();
        assert_eq!(s, r#"{"timestamp":1700000000}"#);
        let back: BaseHeartbeatParams = serde_json::from_str(&s).unwrap();
        assert_eq!(back.timestamp, 1700000000);
    }

    #[test]
    fn heartbeat_negative_timestamp() {
        // Pre-epoch timestamps should round-trip without error.
        let h = BaseHeartbeatParams { timestamp: -1 };
        let v = serde_json::to_value(&h).unwrap();
        assert_eq!(v["timestamp"], -1);
        let back: BaseHeartbeatParams = serde_json::from_value(v).unwrap();
        assert_eq!(back.timestamp, -1);
    }

    #[test]
    fn heartbeat_no_extra_fields() {
        let h = BaseHeartbeatParams { timestamp: 0 };
        let v = serde_json::to_value(&h).unwrap();
        assert_eq!(
            v.as_object().unwrap().len(),
            1,
            "heartbeat must have exactly 1 field"
        );
    }

    // ── BaseErrorParams ─────────────────────────────────────────────

    #[test]
    fn error_params_round_trip() {
        let e = BaseErrorParams {
            code: "E_TIMEOUT".into(),
            message: "request timed out".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "E_TIMEOUT");
        assert_eq!(v["message"], "request timed out");
        let back: BaseErrorParams = serde_json::from_value(v).unwrap();
        assert_eq!(back.code, "E_TIMEOUT");
        assert_eq!(back.message, "request timed out");
    }

    #[test]
    fn error_params_no_extra_fields() {
        let e = BaseErrorParams {
            code: "X".into(),
            message: "y".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v.as_object().unwrap().len(), 2);
    }

    #[test]
    fn error_params_missing_field_rejected() {
        let raw = serde_json::json!({"code": "X"});
        assert!(serde_json::from_value::<BaseErrorParams>(raw).is_err());
    }

    // ── BaseAckParams ───────────────────────────────────────────────

    #[test]
    fn ack_params_round_trip() {
        let a = BaseAckParams {
            message_id: "msg-001".into(),
        };
        let s = serde_json::to_string(&a).unwrap();
        assert_eq!(s, r#"{"message_id":"msg-001"}"#);
        let back: BaseAckParams = serde_json::from_str(&s).unwrap();
        assert_eq!(back.message_id, "msg-001");
    }

    #[test]
    fn ack_params_empty_message_id() {
        let a = BaseAckParams {
            message_id: String::new(),
        };
        let v = serde_json::to_value(&a).unwrap();
        assert_eq!(v["message_id"], "");
    }

    #[test]
    fn ack_params_no_extra_fields() {
        let a = BaseAckParams {
            message_id: "m".into(),
        };
        let v = serde_json::to_value(&a).unwrap();
        assert_eq!(v.as_object().unwrap().len(), 1);
    }
}
