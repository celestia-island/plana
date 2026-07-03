use serde::{Deserialize, Serialize};

pub const JSONRPC_VERSION: &str = "2.0";

pub mod error_codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
    pub const SNAPSHOT_FAILED: i64 = -32001;
    pub const AGENT_UNAVAILABLE: i64 = -32002;
    pub const CONTAINER_ERROR: i64 = -32003;
    pub const REPL_ERROR: i64 = -32004;
    pub const AUTH_ERROR: i64 = -32005;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn parse_error() -> Self {
        Self::new(error_codes::PARSE_ERROR, "Parse error")
    }

    pub fn invalid_request() -> Self {
        Self::new(error_codes::INVALID_REQUEST, "Invalid Request")
    }

    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", method),
        )
    }

    pub fn invalid_params(msg: &str) -> Self {
        Self::new(
            error_codes::INVALID_PARAMS,
            format!("Invalid params: {}", msg),
        )
    }

    pub fn internal_error(msg: &str) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, msg)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Id {
    #[default]
    Null,
    String(String),
    Number(i64),
}

impl Id {
    /// Creates a new unique request `Id` using UUID v7 (time-ordered, sortable).
    ///
    /// UUID v7 is the canonical wire format for the celestia-island platform.
    /// Both entelecheia (Rust) and shittim-chest (TypeScript) MUST agree on
    /// this format — TypeScript consumers should parse the string value with a
    /// UUID v7 library or treat it as an opaque string that sorts
    /// lexicographically.
    pub fn new_uuid() -> Self {
        Self::String(uuid::Uuid::now_v7().to_string())
    }

    /// Creates a new unique request `Id` using UUID v4 (random).
    ///
    /// Provided for consumers that prefer random UUIDs over time-ordered v7.
    /// Default preference in this platform is [`Id::new_uuid`] (v7).
    pub fn new_uuid_v4() -> Self {
        Self::String(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
    Response(JsonRpcResponse),
}

impl<'de> serde::Deserialize<'de> for JsonRpcMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let has_id = value.get("id").map(|v| !v.is_null()).unwrap_or(false);
        let has_method = value.get("method").is_some();
        let has_result = value.get("result").is_some();
        let has_error = value.get("error").is_some();

        // A Response is identified by `result`/`error`. Its `id` may be null:
        // JSON-RPC 2.0 mandates `"id": null` on error responses to requests
        // whose id could not be detected (Parse error / Invalid Request).
        // R9 redefined null id as "no id" only for *request/notification*
        // discrimination below; responses must still accept it.
        if has_result || has_error {
            let resp: JsonRpcResponse = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid response: {}", e)))?;
            Ok(JsonRpcMessage::Response(resp))
        } else if has_method && has_id {
            let req: JsonRpcRequest = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid request: {}", e)))?;
            Ok(JsonRpcMessage::Request(req))
        } else if has_method && !has_id {
            let notif: JsonRpcNotification = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid notification: {}", e)))?;
            Ok(JsonRpcMessage::Notification(notif))
        } else {
            Err(serde::de::Error::custom(
                "cannot classify JSON-RPC message: must have (method + id) for request, (method) for notification, or (id + result/error) for response",
            ))
        }
    }
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(Id::new_uuid()),
            method: method.into(),
            params,
        }
    }

    pub fn with_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(id: Id, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Id, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

pub fn build_notification(method: &str, params: impl serde::Serialize) -> String {
    let method = if method.trim().is_empty() {
        "internal.fallback"
    } else {
        method
    };
    let params = serde_json::to_value(&params)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
    let notif = JsonRpcNotification::new(method, Some(params));
    serde_json::to_string(&notif).unwrap_or_else(|_| {
        let fallback = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "internal.fallback",
            "params": {}
        });
        serde_json::to_string(&fallback).unwrap_or_else(|_| {
            String::from(r#"{"jsonrpc":"2.0","method":"internal.fallback","params":{}}"#)
        })
    })
}

pub fn build_notification_value(method: &str, params: impl serde::Serialize) -> serde_json::Value {
    let method = if method.trim().is_empty() {
        "internal.fallback"
    } else {
        method
    };
    let params = serde_json::to_value(&params)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
    let notif = JsonRpcNotification::new(method, Some(params));
    serde_json::to_value(notif).unwrap_or_else(|_| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "internal.fallback",
            "params": {}
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Id handling ──────────────────────────────────────────

    #[test]
    fn id_numeric_round_trip() {
        let id = Id::Number(42);
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "42");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_string_round_trip() {
        let id = Id::String("abc-123".to_string());
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, r#""abc-123""#);
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_null_default_round_trip() {
        let id = Id::Null;
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "null");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
        assert_eq!(Id::default(), Id::Null);
    }

    #[test]
    fn id_new_uuid_is_v7_shaped_string() {
        let id = Id::new_uuid();
        let s = serde_json::to_string(&id).unwrap();
        // new_uuid() produces Id::String(_); serialized as a JSON string.
        assert!(s.starts_with('"') && s.ends_with('"'));
        let inner = &s[1..s.len() - 1];
        // UUID v7: version nibble at position 14 must be '7'.
        let bytes = inner.as_bytes();
        assert_eq!(bytes.len(), 36, "expected hyphenated UUID, got {inner}");
        assert_eq!(bytes[14], b'7', "expected UUID v7, got {inner}");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    // ── Request round-trip & exact JSON shape ────────────────

    #[test]
    fn request_round_trip_with_numeric_id() {
        let req = JsonRpcRequest::new("agent.run", Some(json!({"agent": "haplotes"})))
            .with_id(Id::Number(7));
        let s = serde_json::to_string(&req).unwrap();
        // Exact-shape assertion guards against protocol drift.
        assert_eq!(
            s,
            r#"{"jsonrpc":"2.0","id":7,"method":"agent.run","params":{"agent":"haplotes"}}"#
        );
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.jsonrpc, "2.0");
        assert_eq!(back.id, Some(Id::Number(7)));
        assert_eq!(back.method, "agent.run");
        assert_eq!(back.params, Some(json!({"agent": "haplotes"})));
    }

    #[test]
    fn request_without_params_omits_field() {
        let req = JsonRpcRequest::new("ping", None).with_id(Id::String("s1".into()));
        let v = serde_json::to_value(&req).unwrap();
        // `params` must be absent (skip_serializing_if = "Option::is_none"),
        // not serialized as null.
        assert!(v.get("params").is_none());
        assert_eq!(v["id"], "s1");
        let back: JsonRpcRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back.params, None);
        assert_eq!(back.id, Some(Id::String("s1".into())));
    }

    // ── Notification round-trip & discrimination ─────────────

    #[test]
    fn notification_has_no_id_field() {
        let notif = JsonRpcNotification::new("agent.status", Some(json!({"status": "online"})));
        let v = serde_json::to_value(&notif).unwrap();
        assert!(v.get("id").is_none(), "notifications must not carry an id");
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "agent.status");
        let back: JsonRpcNotification = serde_json::from_value(v).unwrap();
        assert_eq!(back.method, "agent.status");
    }

    #[test]
    fn message_classifies_request_vs_notification() {
        // No `id` + `method` → notification.
        let notif_json = r#"{"jsonrpc":"2.0","method":"ev.push","params":{"x":1}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(notif_json).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Notification(_)));

        // `id` + `method` → request.
        let req_json = r#"{"jsonrpc":"2.0","id":99,"method":"agent.run","params":{}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(req_json).unwrap();
        match msg {
            JsonRpcMessage::Request(r) => assert_eq!(r.id, Some(Id::Number(99))),
            other => panic!("expected Request, got {other:?}"),
        }
    }

    #[test]
    fn message_classifies_response_success_and_error() {
        let ok_json = r#"{"jsonrpc":"2.0","id":5,"result":{"ok":true}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(ok_json).unwrap();
        match msg {
            JsonRpcMessage::Response(r) => {
                assert_eq!(r.id, Id::Number(5));
                assert!(r.result.is_some());
                assert!(r.error.is_none());
            }
            other => panic!("expected Response, got {other:?}"),
        }

        let err_json =
            r#"{"jsonrpc":"2.0","id":"e1","error":{"code":-32601,"message":"not found"}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(err_json).unwrap();
        match msg {
            JsonRpcMessage::Response(r) => {
                assert_eq!(r.id, Id::String("e1".into()));
                let err = r.error.expect("error field");
                assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
                assert_eq!(err.message, "not found");
                assert!(r.result.is_none());
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn message_classifies_explicit_null_id_as_notification() {
        // JSON-RPC 2.0 allows id to be null. A payload with `"id": null`
        // semantically means "no response expected", i.e. notification.
        let json = r#"{"jsonrpc":"2.0","method":"ev.push","id":null,"params":{"x":1}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(json).unwrap();
        assert!(
            matches!(msg, JsonRpcMessage::Notification(_)),
            "explicit null id must be classified as Notification"
        );
    }

    #[test]
    fn message_classifies_null_id_error_response() {
        // JSON-RPC 2.0: a response to a request whose id could not be
        // detected (Parse error / Invalid Request) MUST carry `"id": null`.
        // R9 made `"id": null` mean "no id" for *request/notification*
        // discrimination, but a *response* (identified by result/error) must
        // still deserialize with a null id.
        let raw = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(raw).unwrap();
        match msg {
            JsonRpcMessage::Response(r) => {
                assert_eq!(r.id, Id::Null);
                assert_eq!(r.error.unwrap().code, error_codes::PARSE_ERROR);
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn message_rejects_unclassifiable_payload() {
        // Neither method nor result/error.
        let bad = r#"{"jsonrpc":"2.0","id":1}"#;
        assert!(serde_json::from_str::<JsonRpcMessage>(bad).is_err());
    }

    #[test]
    fn message_ambiguous_payload_classified_as_response() {
        // A malformed payload carrying BOTH `method` and `result` must be
        // classified as a Response (result/error takes priority per JSON-RPC
        // spec), NOT a Request — otherwise `result` is silently discarded.
        let ambig = r#"{"jsonrpc":"2.0","id":7,"method":"x","result":{"ok":true}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(ambig).unwrap();
        match msg {
            JsonRpcMessage::Response(r) => {
                assert_eq!(r.id, Id::Number(7));
                assert!(r.result.is_some());
            }
            other => panic!("expected Response for ambiguous payload, got {other:?}"),
        }

        // Same with `error` instead of `result`.
        let ambig_err =
            r#"{"jsonrpc":"2.0","id":9,"method":"x","error":{"code":-1,"message":"e"}}"#;
        let msg: JsonRpcMessage = serde_json::from_str(ambig_err).unwrap();
        assert!(matches!(msg, JsonRpcMessage::Response(_)));
    }

    // ── Response round-trip & error codes ────────────────────

    #[test]
    fn response_success_round_trip_preserves_shape() {
        let resp = JsonRpcResponse::success(Id::Number(3), json!({"value": 42}));
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(s, r#"{"jsonrpc":"2.0","id":3,"result":{"value":42}}"#);
        // `error` is skipped when None.
        assert!(!s.contains(r#""error""#));
        let back: JsonRpcResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, Id::Number(3));
        assert_eq!(back.result, Some(json!({"value": 42})));
        assert!(back.error.is_none());
    }

    #[test]
    fn response_error_round_trip_with_data() {
        let err = JsonRpcError::new(error_codes::PARSE_ERROR, "Parse error")
            .with_data(json!({"offset": 12}));
        let resp = JsonRpcResponse::error(Id::String("p".into()), err);
        let v = serde_json::to_value(&resp).unwrap();
        // `result` skipped on error path; `error.data` present.
        assert!(v.get("result").is_none());
        assert_eq!(v["error"]["code"], -32700);
        assert_eq!(v["error"]["message"], "Parse error");
        assert_eq!(v["error"]["data"]["offset"], 12);
        let back: JsonRpcResponse = serde_json::from_value(v).unwrap();
        let err = back.error.unwrap();
        assert_eq!(err.code, error_codes::PARSE_ERROR);
        assert_eq!(err.data, Some(json!({"offset": 12})));
    }

    #[test]
    fn response_error_without_data_omits_field() {
        let err = JsonRpcError::new(error_codes::INTERNAL_ERROR, "boom");
        let resp = JsonRpcResponse::error(Id::Number(0), err);
        let s = serde_json::to_string(&resp).unwrap();
        assert!(
            !s.contains(r#""data""#),
            "`data` must be absent when None, got: {s}"
        );
        let back: JsonRpcResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back.error.unwrap().data, None);
    }

    #[test]
    fn error_code_constants_unchanged() {
        // Guard against accidental drift in well-known JSON-RPC error codes.
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    // ── Notification helpers ─────────────────────────────────

    #[test]
    fn build_notification_value_shape() {
        let v = build_notification_value("ev.tick", json!({"n": 1}));
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "ev.tick");
        assert_eq!(v["params"]["n"], 1);
        assert!(v.get("id").is_none());
    }

    #[test]
    fn build_notification_string_is_valid_json() {
        let s = build_notification("ev.tick", json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "ev.tick");
    }

    #[test]
    fn build_notification_empty_method_produces_fallback() {
        let s = build_notification("", json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_whitespace_method_produces_fallback() {
        let s = build_notification("  \t", json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_value_empty_method_produces_fallback() {
        let v = build_notification_value("", json!({"n": 1}));
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_value_whitespace_method_produces_fallback() {
        let v = build_notification_value(" \n ", json!({"n": 1}));
        assert_eq!(v["method"], "internal.fallback");
    }
}
