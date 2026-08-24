//! JSON-RPC 2.0 envelope types — the single canonical definition.
//!
//! This module is the one and only source of the generic JSON-RPC 2.0
//! envelope (`JsonRpcRequest` / `JsonRpcNotification` / `JsonRpcResponse` /
//! `JsonRpcError` / `Id` / `JsonRpcMessage`) plus the standard error codes.
//! A former second copy in `plana-protocol-core` was removed after the two
//! drifted apart (the copy carried the R9 null-id classification fix that
//! never landed here); its spec-compliant behaviour and tests are merged
//! into this file. Platform-specific domain params and the `UnixMethod`
//! vocabulary live alongside the envelope below.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use tracing::warn;

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub tool_name: String,
    pub parameters: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplExecParams {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplSnapshotParams {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosSetAllowedToolsParams {
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_modes: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_badge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_skill: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosSetRagContextParams {
    pub rag_context: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatusParams {
    pub delegator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthLockdownParams {
    pub delegator_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRestoreParams {
    pub delegator_id: String,
    pub target_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeCallParams {
    pub tool_name: String,
    pub parameters: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_uri: Option<String>,
}

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
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
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
    /// UUID v7 is the canonical wire format for this protocol; all
    /// implementations MUST agree on this format. TypeScript consumers should
    /// parse the string value with a UUID v7 library or treat it as an opaque
    /// string that sorts lexicographically.
    pub fn new_uuid() -> Self {
        Self::String(Uuid::now_v7().to_string())
    }

    /// Creates a new unique request `Id` using UUID v4 (random).
    ///
    /// Provided for consumers that prefer random UUIDs over time-ordered v7.
    /// Default preference in this platform is [`Id::new_uuid`] (v7).
    pub fn new_uuid_v4() -> Self {
        Self::String(Uuid::new_v4().to_string())
    }
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
        let value = Value::deserialize(deserializer)?;

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
    pub fn new(method: UnixMethod, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(Id::new_uuid()),
            method: method.method_str().to_string(),
            params,
        }
    }

    pub fn new_raw(method: impl Into<String>, params: Option<Value>) -> Self {
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

    pub fn method_enum(&self) -> Option<UnixMethod> {
        UnixMethod::from_method_str(&self.method)
    }

    pub fn tool_call(tool_name: impl Into<String>, parameters: Value) -> Self {
        Self::new(
            UnixMethod::ToolCall,
            Some(
                serde_json::to_value(ToolCallParams {
                    tool_name: tool_name.into(),
                    parameters,
                    workspace_uri: None,
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn tool_call_with_workspace(
        tool_name: impl Into<String>,
        parameters: Value,
        workspace_uri: Option<&str>,
    ) -> Self {
        Self::new(
            UnixMethod::ToolCall,
            Some(
                serde_json::to_value(ToolCallParams {
                    tool_name: tool_name.into(),
                    parameters,
                    workspace_uri: workspace_uri.map(|s| s.to_string()),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn tool_list_tools() -> Self {
        Self::new(UnixMethod::ToolListTools, None)
    }

    pub fn repl_exec(code: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::ReplExec,
            Some(
                serde_json::to_value(ReplExecParams { code: code.into() }).unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn repl_snapshot(path: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::ReplSnapshot,
            Some(
                serde_json::to_value(ReplSnapshotParams { path: path.into() }).unwrap_or_else(
                    |e| {
                        warn!(error=%e, "jsonrpc params serialization failed");
                        serde_json::Value::Null
                    },
                ),
            ),
        )
    }

    pub fn repl_snapshot_memory() -> Self {
        Self::new(UnixMethod::ReplSnapshotMemory, None)
    }

    pub fn repl_restore(path: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::ReplRestore,
            Some(
                serde_json::to_value(ReplSnapshotParams { path: path.into() }).unwrap_or_else(
                    |e| {
                        warn!(error=%e, "jsonrpc params serialization failed");
                        serde_json::Value::Null
                    },
                ),
            ),
        )
    }

    pub fn cosmos_status() -> Self {
        Self::new(UnixMethod::CosmosStatus, None)
    }

    pub fn cosmos_cleanup_state() -> Self {
        Self::new(UnixMethod::CosmosCleanupState, None)
    }

    pub fn repl_dump_all_bindings() -> Self {
        Self::new(UnixMethod::ReplDumpAllBindings, None)
    }

    pub fn cosmos_set_allowed_tools(tools: Vec<String>) -> Self {
        Self::new(
            UnixMethod::CosmosSetAllowedTools,
            Some(
                serde_json::to_value(CosmosSetAllowedToolsParams {
                    tools,
                    access_modes: None,
                    session_badge: None,
                    current_skill: None,
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn cosmos_set_allowed_tools_with_access(
        tools: Vec<String>,
        access_modes: std::collections::HashMap<String, String>,
        session_badge: Option<&str>,
        current_skill: Option<&str>,
    ) -> Self {
        Self::new(
            UnixMethod::CosmosSetAllowedTools,
            Some(
                serde_json::to_value(CosmosSetAllowedToolsParams {
                    tools,
                    access_modes: Some(access_modes),
                    session_badge: session_badge.map(|s| s.to_string()),
                    current_skill: current_skill.map(|s| s.to_string()),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn cosmos_set_rag_context(rag_context: serde_json::Value) -> Self {
        Self::new(
            UnixMethod::CosmosSetRagContext,
            Some(
                serde_json::to_value(CosmosSetRagContextParams { rag_context }).unwrap_or_else(
                    |e| {
                        warn!(error=%e, "jsonrpc params serialization failed");
                        serde_json::Value::Null
                    },
                ),
            ),
        )
    }

    pub fn auth_status(delegator_id: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::AuthStatus,
            Some(
                serde_json::to_value(AuthStatusParams {
                    delegator_id: delegator_id.into(),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn auth_lockdown(delegator_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::AuthLockdown,
            Some(
                serde_json::to_value(AuthLockdownParams {
                    delegator_id: delegator_id.into(),
                    reason: reason.into(),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn auth_restore(delegator_id: impl Into<String>, target_level: impl Into<String>) -> Self {
        Self::new(
            UnixMethod::AuthRestore,
            Some(
                serde_json::to_value(AuthRestoreParams {
                    delegator_id: delegator_id.into(),
                    target_level: target_level.into(),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn bridge_call(tool_name: impl Into<String>, parameters: Value) -> Self {
        Self::new(
            UnixMethod::BridgeCall,
            Some(
                serde_json::to_value(BridgeCallParams {
                    tool_name: tool_name.into(),
                    parameters,
                    workspace_uri: None,
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn bridge_call_with_workspace(
        tool_name: impl Into<String>,
        parameters: Value,
        workspace_uri: Option<&str>,
    ) -> Self {
        Self::new(
            UnixMethod::BridgeCall,
            Some(
                serde_json::to_value(BridgeCallParams {
                    tool_name: tool_name.into(),
                    parameters,
                    workspace_uri: workspace_uri.map(|s| s.to_string()),
                })
                .unwrap_or_else(|e| {
                    warn!(error=%e, "jsonrpc params serialization failed");
                    serde_json::Value::Null
                }),
            ),
        )
    }

    pub fn bridge_list_tools() -> Self {
        Self::new(UnixMethod::BridgeListTools, None)
    }
}

impl JsonRpcNotification {
    pub fn new(method: UnixMethod, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.method_str().to_string(),
            params,
        }
    }

    pub fn new_raw(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(id: Id, result: Value) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnixMethod {
    ToolCall,
    ToolListTools,
    ReplExec,
    ReplSnapshot,
    ReplSnapshotMemory,
    ReplRestore,
    ReplDumpAllBindings,
    CosmosCleanupState,
    CosmosStatus,
    CosmosSetAllowedTools,
    CosmosSetRagContext,
    BridgeCall,
    BridgeListTools,
    SyncUserMessage,
    SyncAgentResponse,
    AuthStatus,
    AuthLockdown,
    AuthRestore,
}

impl UnixMethod {
    pub const fn method_str(self) -> &'static str {
        match self {
            Self::ToolCall => "tool.call",
            Self::ToolListTools => "tool.list_tools",
            Self::ReplExec => "repl.exec",
            Self::ReplSnapshot => "repl.snapshot",
            Self::ReplSnapshotMemory => "repl.snapshot_memory",
            Self::ReplRestore => "repl.restore",
            Self::ReplDumpAllBindings => "repl.dump_all_bindings",
            Self::CosmosCleanupState => "cosmos.cleanup_state",
            Self::CosmosStatus => "cosmos.status",
            Self::CosmosSetAllowedTools => "cosmos.set_allowed_tools",
            Self::CosmosSetRagContext => "cosmos.set_rag_context",
            Self::BridgeCall => "bridge.call",
            Self::BridgeListTools => "bridge.list_tools",
            Self::SyncUserMessage => "tui.user_message",
            Self::SyncAgentResponse => "tui.agent_response",
            Self::AuthStatus => "auth.status",
            Self::AuthLockdown => "auth.lockdown",
            Self::AuthRestore => "auth.restore",
        }
    }

    pub fn from_method_str(s: &str) -> Option<Self> {
        match s {
            "tool.call" => Some(Self::ToolCall),
            "tool.list_tools" => Some(Self::ToolListTools),
            "repl.exec" => Some(Self::ReplExec),
            "repl.snapshot" => Some(Self::ReplSnapshot),
            "repl.snapshot_memory" => Some(Self::ReplSnapshotMemory),
            "repl.restore" => Some(Self::ReplRestore),
            "repl.dump_all_bindings" => Some(Self::ReplDumpAllBindings),
            "cosmos.cleanup_state" => Some(Self::CosmosCleanupState),
            "cosmos.status" => Some(Self::CosmosStatus),
            "cosmos.set_allowed_tools" => Some(Self::CosmosSetAllowedTools),
            "cosmos.set_rag_context" => Some(Self::CosmosSetRagContext),
            "bridge.call" => Some(Self::BridgeCall),
            "bridge.list_tools" => Some(Self::BridgeListTools),
            "tui.user_message" => Some(Self::SyncUserMessage),
            "tui.agent_response" => Some(Self::SyncAgentResponse),
            "auth.status" => Some(Self::AuthStatus),
            "auth.lockdown" => Some(Self::AuthLockdown),
            "auth.restore" => Some(Self::AuthRestore),
            _ => None,
        }
    }
}

impl std::fmt::Display for UnixMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.method_str())
    }
}

pub mod methods {
    use super::UnixMethod;

    pub const TOOL_CALL: UnixMethod = UnixMethod::ToolCall;
    pub const TOOL_LIST_TOOLS: UnixMethod = UnixMethod::ToolListTools;
    pub const REPL_EXEC: UnixMethod = UnixMethod::ReplExec;
    pub const REPL_SNAPSHOT: UnixMethod = UnixMethod::ReplSnapshot;
    pub const REPL_SNAPSHOT_MEMORY: UnixMethod = UnixMethod::ReplSnapshotMemory;
    pub const REPL_RESTORE: UnixMethod = UnixMethod::ReplRestore;
    pub const COSMOS_STATUS: UnixMethod = UnixMethod::CosmosStatus;
    pub const COSMOS_SET_ALLOWED_TOOLS: UnixMethod = UnixMethod::CosmosSetAllowedTools;
    pub const COSMOS_SET_RAG_CONTEXT: UnixMethod = UnixMethod::CosmosSetRagContext;
    pub const BRIDGE_CALL: UnixMethod = UnixMethod::BridgeCall;
    pub const BRIDGE_LIST_TOOLS: UnixMethod = UnixMethod::BridgeListTools;
    pub const SYNC_USER_MESSAGE: UnixMethod = UnixMethod::SyncUserMessage;
    pub const SYNC_AGENT_RESPONSE: UnixMethod = UnixMethod::SyncAgentResponse;
}

/// Build a JSON-RPC notification string from a method name and serializable params.
/// Falls back to `internal.fallback` on empty method or serialization failure.
pub fn build_notification(method: &str, params: impl serde::Serialize) -> String {
    let method = if method.trim().is_empty() {
        "internal.fallback"
    } else {
        method
    };
    let params = serde_json::to_value(&params)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
    let notif = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION.to_string(),
        method: method.to_string(),
        params: Some(params),
    };
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

/// Build a JSON-RPC notification `Value` from a method name and serializable params.
/// Falls back to `internal.fallback` on empty method or serialization failure.
pub fn build_notification_value(method: &str, params: impl serde::Serialize) -> Value {
    let method = if method.trim().is_empty() {
        "internal.fallback"
    } else {
        method
    };
    let params = serde_json::to_value(&params)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
    let notif = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION.to_string(),
        method: method.to_string(),
        params: Some(params),
    };
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
    use anyhow::{bail, Context, Result};

    #[test]
    fn request_serializes_correctly() -> Result<()> {
        let req = JsonRpcRequest::new_raw("tool.call", Some(serde_json::json!({"tool": "test"})));
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tool.call\""));
        assert!(json.contains("\"tool\":\"test\""));
        Ok(())
    }

    #[test]
    fn response_success_serializes() -> Result<()> {
        let resp =
            JsonRpcResponse::success(Id::String("1".into()), serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&resp)?;
        assert!(json.contains("\"result\":"));
        assert!(!json.contains("\"error\":"));
        Ok(())
    }

    #[test]
    fn response_error_serializes() -> Result<()> {
        let resp = JsonRpcResponse::error(
            Id::String("1".into()),
            JsonRpcError::method_not_found("test"),
        );
        let json = serde_json::to_string(&resp)?;
        assert!(json.contains("\"error\":"));
        assert!(!json.contains("\"result\":"));
        assert!(json.contains("-32601"));
        Ok(())
    }

    #[test]
    fn notification_has_no_id() -> Result<()> {
        let notif =
            JsonRpcNotification::new_raw("event.log", Some(serde_json::json!({"msg": "hi"})));
        let json = serde_json::to_string(&notif)?;
        assert!(!json.contains("\"id\":"));
        assert!(json.contains("\"method\":\"event.log\""));
        Ok(())
    }

    #[test]
    fn parse_roundtrip() -> Result<()> {
        let original = JsonRpcRequest::new_raw("test.method", Some(serde_json::json!({"a": 1})));
        let json = serde_json::to_string(&original)?;
        let parsed: JsonRpcMessage = serde_json::from_str(&json)?;
        match parsed {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.method, "test.method");
            }
            _ => bail!("Expected Request"),
        }
        Ok(())
    }

    #[test]
    fn typed_constructors_use_correct_methods() -> Result<()> {
        let req = JsonRpcRequest::tool_call("test_tool", serde_json::Value::Null);
        assert_eq!(req.method, "tool.call");
        assert!(req.params.is_some());

        let req = JsonRpcRequest::tool_list_tools();
        assert_eq!(req.method, "tool.list_tools");
        assert!(req.params.is_none());

        let req = JsonRpcRequest::bridge_call("my_tool", serde_json::json!({"x": 1}));
        assert_eq!(req.method, "bridge.call");

        let req = JsonRpcRequest::bridge_list_tools();
        assert_eq!(req.method, "bridge.list_tools");

        let req = JsonRpcRequest::repl_exec("1+1");
        assert_eq!(req.method, "repl.exec");

        Ok(())
    }

    #[test]
    fn unix_method_roundtrip() -> Result<()> {
        for method in [
            UnixMethod::ToolCall,
            UnixMethod::ToolListTools,
            UnixMethod::ReplExec,
            UnixMethod::ReplSnapshot,
            UnixMethod::ReplSnapshotMemory,
            UnixMethod::ReplRestore,
            UnixMethod::CosmosStatus,
            UnixMethod::CosmosSetAllowedTools,
            UnixMethod::CosmosSetRagContext,
            UnixMethod::BridgeCall,
            UnixMethod::BridgeListTools,
            UnixMethod::SyncUserMessage,
            UnixMethod::SyncAgentResponse,
            UnixMethod::AuthStatus,
            UnixMethod::AuthLockdown,
            UnixMethod::AuthRestore,
        ] {
            let s = method.method_str();
            let parsed = UnixMethod::from_method_str(s);
            assert_eq!(parsed, Some(method), "roundtrip failed for {:?}", method);
        }
        assert_eq!(UnixMethod::from_method_str("unknown"), None);
        Ok(())
    }

    #[test]
    fn auth_request_constructors() -> Result<()> {
        let status = JsonRpcRequest::auth_status("agent-1");
        assert_eq!(status.method, "auth.status");
        assert_eq!(
            status.params.as_ref().context("test precondition")?["delegator_id"],
            "agent-1"
        );

        let lockdown = JsonRpcRequest::auth_lockdown("agent-1", "breach");
        assert_eq!(lockdown.method, "auth.lockdown");
        assert_eq!(
            lockdown.params.as_ref().context("test precondition")?["delegator_id"],
            "agent-1"
        );
        assert_eq!(
            lockdown.params.as_ref().context("test precondition")?["reason"],
            "breach"
        );

        let restore = JsonRpcRequest::auth_restore("agent-1", "L3");
        assert_eq!(restore.method, "auth.restore");
        assert_eq!(
            restore.params.as_ref().context("test precondition")?["delegator_id"],
            "agent-1"
        );
        assert_eq!(
            restore.params.as_ref().context("test precondition")?["target_level"],
            "L3"
        );
        Ok(())
    }

    // ── Envelope conformance tests (ported from the retired ──
    // ── plana-protocol-core copy; guard against future drift) ──

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
        assert!(s.starts_with('"') && s.ends_with('"'));
        let inner = &s[1..s.len() - 1];
        let bytes = inner.as_bytes();
        assert_eq!(bytes.len(), 36, "expected hyphenated UUID, got {inner}");
        assert_eq!(bytes[14], b'7', "expected UUID v7, got {inner}");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_new_uuid_v4_round_trip() {
        let id = Id::new_uuid_v4();
        let s = serde_json::to_string(&id).unwrap();
        let inner = &s[1..s.len() - 1];
        let bytes = inner.as_bytes();
        assert_eq!(bytes.len(), 36);
        assert_eq!(bytes[14], b'4', "expected UUID v4, got {inner}");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_number_zero_round_trip() {
        let id = Id::Number(0);
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "0");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_number_negative_round_trip() {
        let id = Id::Number(-42);
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "-42");
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_number_large_round_trip() {
        let id = Id::Number(i64::MAX);
        let s = serde_json::to_string(&id).unwrap();
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_empty_string_round_trip() {
        let id = Id::String(String::new());
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, r#""""#);
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_unicode_string_round_trip() {
        let id = Id::String("请求-42".into());
        let s = serde_json::to_string(&id).unwrap();
        let back: Id = serde_json::from_str(&s).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn id_ordering_null_lt_string_lt_number() {
        // Id derives Ord — Null < String < Number (enum variant order).
        let n = Id::Null;
        let s = Id::String("z".into());
        let num = Id::Number(0);
        assert!(n < s);
        assert!(s < num);
    }

    #[test]
    fn id_equality_and_hash() {
        use std::collections::HashSet;
        let a = Id::String("x".into());
        let b = Id::String("x".into());
        assert_eq!(a, b);
        let set: HashSet<Id> = [a, b, Id::Number(1)].into_iter().collect();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn error_new_without_data() {
        let e = JsonRpcError::new(-32700, "Parse error");
        assert_eq!(e.code, -32700);
        assert_eq!(e.message, "Parse error");
        assert!(e.data.is_none());
    }

    #[test]
    fn error_with_data_builder() {
        let e = JsonRpcError::new(-1, "x").with_data(serde_json::json!({"k": "v"}));
        assert_eq!(e.data, Some(serde_json::json!({"k": "v"})));
    }

    #[test]
    fn error_constructors_use_correct_codes() {
        assert_eq!(JsonRpcError::parse_error().code, error_codes::PARSE_ERROR);
        assert_eq!(
            JsonRpcError::invalid_request().code,
            error_codes::INVALID_REQUEST
        );
        assert_eq!(
            JsonRpcError::method_not_found("test").code,
            error_codes::METHOD_NOT_FOUND
        );
        assert_eq!(
            JsonRpcError::invalid_params("bad").code,
            error_codes::INVALID_PARAMS
        );
        assert_eq!(
            JsonRpcError::internal_error("boom").code,
            error_codes::INTERNAL_ERROR
        );
    }

    #[test]
    fn error_method_not_found_includes_method_name() {
        let e = JsonRpcError::method_not_found("my.method");
        assert!(e.message.contains("my.method"));
    }

    #[test]
    fn error_invalid_params_includes_detail() {
        let e = JsonRpcError::invalid_params("missing field `token`");
        assert!(e.message.contains("missing field"));
    }

    #[test]
    fn error_data_omitted_when_none() {
        let e = JsonRpcError::new(-32603, "err");
        let s = serde_json::to_string(&e).unwrap();
        assert!(!s.contains(r#""data""#), "data must be omitted: {s}");
    }

    #[test]
    fn error_data_present_when_some() {
        let e = JsonRpcError::new(-32603, "err").with_data(serde_json::json!(42));
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains(r#""data":42"#));
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

    #[test]
    fn request_round_trip_with_numeric_id() {
        let req = JsonRpcRequest::new_raw(
            "agent.run",
            Some(serde_json::json!({"agent": "example_agent"})),
        )
        .with_id(Id::Number(7));
        let s = serde_json::to_string(&req).unwrap();
        // Exact-shape assertion guards against protocol drift.
        assert_eq!(
            s,
            r#"{"jsonrpc":"2.0","id":7,"method":"agent.run","params":{"agent":"example_agent"}}"#
        );
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.jsonrpc, "2.0");
        assert_eq!(back.id, Some(Id::Number(7)));
        assert_eq!(back.method, "agent.run");
        assert_eq!(
            back.params,
            Some(serde_json::json!({"agent": "example_agent"}))
        );
    }

    #[test]
    fn request_without_params_omits_field() {
        let req = JsonRpcRequest::new_raw("ping", None).with_id(Id::String("s1".into()));
        let v = serde_json::to_value(&req).unwrap();
        // `params` must be absent (skip_serializing_if = "Option::is_none"),
        // not serialized as null.
        assert!(v.get("params").is_none());
        assert_eq!(v["id"], "s1");
        let back: JsonRpcRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back.params, None);
        assert_eq!(back.id, Some(Id::String("s1".into())));
    }

    #[test]
    fn request_with_none_id_serializes_without_id() {
        let mut req = JsonRpcRequest::new_raw("m", None);
        req.id = None;
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("id").is_none());
    }

    #[test]
    fn request_with_complex_params_round_trip() {
        let params = serde_json::json!({
            "nested": {"deep": [1, 2, {"x": true}]},
            "str": "hello",
            "null_val": null
        });
        let req = JsonRpcRequest::new_raw("complex", Some(params.clone())).with_id(Id::Number(99));
        let s = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.params, Some(params));
    }

    #[test]
    fn request_with_large_params_round_trip() {
        // Deeply nested JSON to verify no stack overflow or truncation.
        let mut params = serde_json::Value::Object(serde_json::Map::new());
        for i in 0..100 {
            params[&format!("key_{i}")] = serde_json::json!({
                "data": vec![i; 100],
                "meta": {"index": i, "tag": format!("item-{i}")}
            });
        }
        let req = JsonRpcRequest::new_raw("bulk", Some(params.clone()))
            .with_id(Id::String("bulk-1".into()));
        let s = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.params, Some(params));
    }

    #[test]
    fn notification_has_no_id_field() {
        let notif = JsonRpcNotification::new_raw(
            "agent.status",
            Some(serde_json::json!({"status": "online"})),
        );
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

    #[test]
    fn message_serialize_request_has_no_result_or_error() {
        let req =
            JsonRpcRequest::new_raw("m", Some(serde_json::json!({"x": 1}))).with_id(Id::Number(1));
        let msg = JsonRpcMessage::Request(req);
        let v = serde_json::to_value(&msg).unwrap();
        assert!(v.get("result").is_none(), "Request must not carry result");
        assert!(v.get("error").is_none(), "Request must not carry error");
        assert_eq!(v["method"], "m");
    }

    #[test]
    fn message_serialize_notification_has_no_id() {
        let notif = JsonRpcNotification::new_raw("ev", None);
        let msg = JsonRpcMessage::Notification(notif);
        let v = serde_json::to_value(&msg).unwrap();
        assert!(v.get("id").is_none(), "Notification must not carry id");
    }

    #[test]
    fn message_serialize_response_has_no_method() {
        let resp = JsonRpcResponse::success(Id::Number(1), serde_json::json!({"ok": true}));
        let msg = JsonRpcMessage::Response(resp);
        let v = serde_json::to_value(&msg).unwrap();
        assert!(
            v.get("method").is_none(),
            "Response must not carry method field"
        );
    }

    #[test]
    fn message_round_trip_request() {
        let req = JsonRpcRequest::new_raw("agent.run", Some(serde_json::json!({"k": "v"})))
            .with_id(Id::String("rid".into()));
        let s = serde_json::to_string(&JsonRpcMessage::Request(req)).unwrap();
        let msg: JsonRpcMessage = serde_json::from_str(&s).unwrap();
        match msg {
            JsonRpcMessage::Request(r) => {
                assert_eq!(r.method, "agent.run");
                assert_eq!(r.id, Some(Id::String("rid".into())));
            }
            other => panic!("expected Request, got {other:?}"),
        }
    }

    #[test]
    fn message_round_trip_notification() {
        let notif = JsonRpcNotification::new_raw("ev.push", Some(serde_json::json!({"n": 1})));
        let s = serde_json::to_string(&JsonRpcMessage::Notification(notif)).unwrap();
        let msg: JsonRpcMessage = serde_json::from_str(&s).unwrap();
        match msg {
            JsonRpcMessage::Notification(n) => {
                assert_eq!(n.method, "ev.push");
            }
            other => panic!("expected Notification, got {other:?}"),
        }
    }

    #[test]
    fn message_round_trip_response() {
        let resp = JsonRpcResponse::success(Id::Number(42), serde_json::json!({"done": true}));
        let s = serde_json::to_string(&JsonRpcMessage::Response(resp)).unwrap();
        let msg: JsonRpcMessage = serde_json::from_str(&s).unwrap();
        match msg {
            JsonRpcMessage::Response(r) => {
                assert_eq!(r.id, Id::Number(42));
                assert!(r.result.is_some());
            }
            other => panic!("expected Response, got {other:?}"),
        }
    }

    #[test]
    fn response_success_round_trip_preserves_shape() {
        let resp = JsonRpcResponse::success(Id::Number(3), serde_json::json!({"value": 42}));
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(s, r#"{"jsonrpc":"2.0","id":3,"result":{"value":42}}"#);
        // `error` is skipped when None.
        assert!(!s.contains(r#""error""#));
        let back: JsonRpcResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, Id::Number(3));
        assert_eq!(back.result, Some(serde_json::json!({"value": 42})));
        assert!(back.error.is_none());
    }

    #[test]
    fn response_error_round_trip_with_data() {
        let err = JsonRpcError::new(error_codes::PARSE_ERROR, "Parse error")
            .with_data(serde_json::json!({"offset": 12}));
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
        assert_eq!(err.data, Some(serde_json::json!({"offset": 12})));
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
    fn response_success_with_null_result() {
        let resp = JsonRpcResponse::success(Id::Number(1), serde_json::Value::Null);
        let v = serde_json::to_value(&resp).unwrap();
        // result is Some(Null) — not skipped because skip_serializing_if only
        // checks Option::is_none, not the inner value.
        assert_eq!(v["result"], serde_json::Value::Null);
        assert!(v.get("error").is_none());
    }

    #[test]
    fn response_error_with_null_id() {
        let resp = JsonRpcResponse::error(Id::Null, JsonRpcError::parse_error());
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["id"], serde_json::Value::Null);
        assert_eq!(v["error"]["code"], -32700);
        assert!(v.get("result").is_none());
    }

    #[test]
    fn build_notification_value_shape() {
        let v = build_notification_value("ev.tick", serde_json::json!({"n": 1}));
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "ev.tick");
        assert_eq!(v["params"]["n"], 1);
        assert!(v.get("id").is_none());
    }

    #[test]
    fn build_notification_string_is_valid_json() {
        let s = build_notification("ev.tick", serde_json::json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "ev.tick");
    }

    #[test]
    fn build_notification_empty_method_produces_fallback() {
        let s = build_notification("", serde_json::json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_whitespace_method_produces_fallback() {
        let s = build_notification("  \t", serde_json::json!({"n": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_value_empty_method_produces_fallback() {
        let v = build_notification_value("", serde_json::json!({"n": 1}));
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_value_whitespace_method_produces_fallback() {
        let v = build_notification_value(" \n ", serde_json::json!({"n": 1}));
        assert_eq!(v["method"], "internal.fallback");
    }

    #[test]
    fn build_notification_unicode_method() {
        let s = build_notification("智能体.运行", serde_json::json!({"a": 1}));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["method"], "智能体.运行");
        assert_eq!(v["params"]["a"], 1);
    }

    #[test]
    fn build_notification_value_unicode_method() {
        let v = build_notification_value("通知.推送", serde_json::json!({"x": 0}));
        assert_eq!(v["method"], "通知.推送");
    }

    #[test]
    fn jsonrpc_version_is_two_point_zero() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }
}
