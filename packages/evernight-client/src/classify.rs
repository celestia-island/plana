//! Error classification for the terminal route (spec §4.2 taxonomy).
//!
//! Transport-level faults and broker JSON-RPC errors are normalized into one
//! [`ClassifiedError`] shape so the entelecheia adapter (and the OreXis audit
//! sink) can branch on [`ErrorKind`] instead of matching fragile error text.

use std::fmt;

use plana_jsonrpc::{error_codes, JsonRpcError};
use serde_json::Value;

use crate::transport::TransportError;

/// Stable taxonomy of dispatch failures (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Envelope/params missing or malformed — broker `-32602`.
    InvalidParams,
    /// Unknown method / mistyped verb — broker `-32601`.
    MethodNotFound,
    /// Broker unreachable at the transport layer (connect, write, read,
    /// read-timeout, parse, closed) — adapter-normalized `-32603` with
    /// `data.reason = "peer_unreachable"`.
    PeerUnreachable,
    /// Server-side execution timeout, signalled structurally via
    /// `data.timeout == true` (never by matching message text, per §4.2).
    ExecutionTimeout,
    /// Authorization/authentication refusal — broker `-32005 AUTH_ERROR`.
    Auth,
    /// `target_scope` validation failure — `-32005` with
    /// `data.reason = "target_out_of_scope"`.
    TargetOutOfScope,
    /// The `request_id` idempotency key was already seen; the request was
    /// rejected locally without any transport call.
    DuplicateRequest,
    /// Anything else (original code/message/data preserved).
    Other,
}

impl ErrorKind {
    /// Stable snake_case label, used in Display output and audit fields.
    pub fn label(self) -> &'static str {
        match self {
            ErrorKind::InvalidParams => "invalid_params",
            ErrorKind::MethodNotFound => "method_not_found",
            ErrorKind::PeerUnreachable => "peer_unreachable",
            ErrorKind::ExecutionTimeout => "execution_timeout",
            ErrorKind::Auth => "auth",
            ErrorKind::TargetOutOfScope => "target_out_of_scope",
            ErrorKind::DuplicateRequest => "duplicate_request",
            ErrorKind::Other => "other",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A normalized dispatch failure: taxonomy kind plus the original JSON-RPC
/// code/message/data (or the adapter-normalized substitutes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedError {
    /// Taxonomy bucket (spec §4.2).
    pub kind: ErrorKind,
    /// JSON-RPC error code carried by (or normalized for) this failure.
    pub code: i64,
    /// Human-readable failure message (safe for logs; never contains
    /// credentials or command output).
    pub message: String,
    /// Structured error data when present (`data.reason`, `data.timeout`, …).
    pub data: Option<Value>,
}

impl ClassifiedError {
    /// Builds a classified error without structured data.
    pub fn new(kind: ErrorKind, code: i64, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Sets/merges a `data.reason` marker (e.g. `"peer_unreachable"`,
    /// `"target_out_of_scope"`).
    pub fn with_reason(mut self, reason: &str) -> Self {
        let mut data = match self.data {
            Some(Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };
        data.insert("reason".to_string(), Value::String(reason.to_string()));
        self.data = Some(Value::Object(data));
        self
    }
}

impl fmt::Display for ClassifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "evernight dispatch failed [{} code {}]: {}",
            self.kind, self.code, self.message
        )
    }
}

impl std::error::Error for ClassifiedError {}

/// Normalizes a transport-layer failure.
///
/// Every [`TransportError`] variant means "the broker could not be reached
/// or the exchange could not complete", so all of them classify as
/// `PeerUnreachable` with the internal-error code `-32603` and the
/// structured `data.reason = "peer_unreachable"` marker (spec §4.2).
pub fn classify_transport(error: TransportError) -> ClassifiedError {
    ClassifiedError::new(
        ErrorKind::PeerUnreachable,
        error_codes::INTERNAL_ERROR,
        error.to_string(),
    )
    .with_reason("peer_unreachable")
}

/// Classifies a broker JSON-RPC error object per the §4.2 mapping.
///
/// Priority order:
/// 1. `data.timeout == true` → [`ErrorKind::ExecutionTimeout`] (structural
///    marker; message text matching is explicitly forbidden);
/// 2. code `-32005` → [`ErrorKind::Auth`];
/// 3. code `-32602` → [`ErrorKind::InvalidParams`];
/// 4. code `-32601` → [`ErrorKind::MethodNotFound`];
/// 5. anything else → [`ErrorKind::Other`].
///
/// In every branch the original `code`/`message`/`data` are preserved.
pub fn classify_rpc(error: JsonRpcError) -> ClassifiedError {
    let timeout_marker = error
        .data
        .as_ref()
        .and_then(|data| data.get("timeout"))
        .is_some_and(|marker| marker == &Value::Bool(true));
    let kind = if timeout_marker {
        ErrorKind::ExecutionTimeout
    } else {
        match error.code {
            error_codes::AUTH_ERROR => ErrorKind::Auth,
            error_codes::INVALID_PARAMS => ErrorKind::InvalidParams,
            error_codes::METHOD_NOT_FOUND => ErrorKind::MethodNotFound,
            _ => ErrorKind::Other,
        }
    };
    ClassifiedError {
        kind,
        code: error.code,
        message: error.message,
        data: error.data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn assert_reason(error: &ClassifiedError, reason: &str) {
        assert_eq!(
            error.data.as_ref().and_then(|d| d.get("reason")),
            Some(&json!(reason)),
            "expected data.reason = {reason}"
        );
    }

    #[test]
    fn every_transport_variant_is_peer_unreachable() {
        let samples = [
            TransportError::Connect("connection refused".to_string()),
            TransportError::Write("broken pipe".to_string()),
            TransportError::ReadTimeout(90),
            TransportError::Read("reset by peer".to_string()),
            TransportError::Parse("expected value".to_string()),
            TransportError::Closed,
        ];
        for sample in samples {
            let classified = classify_transport(sample);
            assert_eq!(classified.kind, ErrorKind::PeerUnreachable);
            assert_eq!(classified.code, error_codes::INTERNAL_ERROR);
            assert_eq!(classified.code, -32603);
            assert_reason(&classified, "peer_unreachable");
            assert!(!classified.message.is_empty());
        }
    }

    #[test]
    fn rpc_timeout_marker_wins_over_the_code() {
        let error = JsonRpcError::internal_error("Command timed out after 60s")
            .with_data(json!({"timeout": true}));
        let classified = classify_rpc(error);
        assert_eq!(classified.kind, ErrorKind::ExecutionTimeout);
        assert_eq!(classified.code, -32603);
        assert_eq!(classified.message, "Command timed out after 60s");
        assert_eq!(classified.data, Some(json!({"timeout": true})));
    }

    #[test]
    fn rpc_timeout_marker_requires_exact_boolean_true() {
        // A numeric or string "timeout" value is NOT the marker.
        let error = JsonRpcError::internal_error("boom").with_data(json!({"timeout": 60}));
        assert_eq!(classify_rpc(error).kind, ErrorKind::Other);
        let error = JsonRpcError::internal_error("boom").with_data(json!({"timeout": "true"}));
        assert_eq!(classify_rpc(error).kind, ErrorKind::Other);
    }

    #[test]
    fn rpc_auth_error_maps_to_auth() {
        let error = JsonRpcError::new(error_codes::AUTH_ERROR, "static token rejected");
        let classified = classify_rpc(error);
        assert_eq!(classified.kind, ErrorKind::Auth);
        assert_eq!(classified.code, -32005);
        assert_eq!(classified.data, None);
    }

    #[test]
    fn rpc_invalid_params_maps_to_invalid_params() {
        let error = JsonRpcError::invalid_params("missing field `command`");
        let classified = classify_rpc(error);
        assert_eq!(classified.kind, ErrorKind::InvalidParams);
        assert_eq!(classified.code, -32602);
    }

    #[test]
    fn rpc_method_not_found_maps_to_method_not_found() {
        let error = JsonRpcError::method_not_found("Command.Exec");
        let classified = classify_rpc(error);
        assert_eq!(classified.kind, ErrorKind::MethodNotFound);
        assert_eq!(classified.code, -32601);
    }

    #[test]
    fn rpc_unknown_code_maps_to_other_and_preserves_everything() {
        let error =
            JsonRpcError::new(-32003, "container exploded").with_data(json!({"container": "demo"}));
        let classified = classify_rpc(error);
        assert_eq!(classified.kind, ErrorKind::Other);
        assert_eq!(classified.code, -32003);
        assert_eq!(classified.message, "container exploded");
        assert_eq!(classified.data, Some(json!({"container": "demo"})));
    }

    #[test]
    fn display_contains_kind_label_code_and_message() {
        let classified = ClassifiedError::new(
            ErrorKind::TargetOutOfScope,
            error_codes::AUTH_ERROR,
            "scope rejected",
        )
        .with_reason("target_out_of_scope");
        let rendered = classified.to_string();
        assert!(rendered.contains("target_out_of_scope"));
        assert!(rendered.contains("-32005"));
        assert!(rendered.contains("scope rejected"));
    }
}
