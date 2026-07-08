use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use tracing::warn;

pub const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCallParams {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new_uuid() -> Self {
        Self::String(Uuid::now_v7().to_string())
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

        let has_id = value.get("id").is_some();
        let has_method = value.get("method").is_some();
        let has_result = value.get("result").is_some();
        let has_error = value.get("error").is_some();

        if has_method && !has_id {
            let notif: JsonRpcNotification = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid notification: {}", e)))?;
            Ok(JsonRpcMessage::Notification(notif))
        } else if has_method && has_id {
            let req: JsonRpcRequest = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid request: {}", e)))?;
            Ok(JsonRpcMessage::Request(req))
        } else if (has_result || has_error) && has_id {
            let resp: JsonRpcResponse = serde_json::from_value(value)
                .map_err(|e| serde::de::Error::custom(format!("invalid response: {}", e)))?;
            Ok(JsonRpcMessage::Response(resp))
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

    pub fn mcp_call(tool_name: impl Into<String>, parameters: Value) -> Self {
        Self::new(
            UnixMethod::McpCall,
            Some(
                serde_json::to_value(McpCallParams {
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

    pub fn mcp_call_with_workspace(
        tool_name: impl Into<String>,
        parameters: Value,
        workspace_uri: Option<&str>,
    ) -> Self {
        Self::new(
            UnixMethod::McpCall,
            Some(
                serde_json::to_value(McpCallParams {
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

    pub fn mcp_list_tools() -> Self {
        Self::new(UnixMethod::McpListTools, None)
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
    McpCall,
    McpListTools,
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
    TuiUserMessage,
    TuiAgentResponse,
    AuthStatus,
    AuthLockdown,
    AuthRestore,
}

impl UnixMethod {
    pub const fn method_str(self) -> &'static str {
        match self {
            Self::McpCall => "mcp.call",
            Self::McpListTools => "mcp.list_tools",
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
            Self::TuiUserMessage => "tui.user_message",
            Self::TuiAgentResponse => "tui.agent_response",
            Self::AuthStatus => "auth.status",
            Self::AuthLockdown => "auth.lockdown",
            Self::AuthRestore => "auth.restore",
        }
    }

    pub fn from_method_str(s: &str) -> Option<Self> {
        match s {
            "mcp.call" => Some(Self::McpCall),
            "mcp.list_tools" => Some(Self::McpListTools),
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
            "tui.user_message" => Some(Self::TuiUserMessage),
            "tui.agent_response" => Some(Self::TuiAgentResponse),
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

    pub const MCP_CALL: UnixMethod = UnixMethod::McpCall;
    pub const MCP_LIST_TOOLS: UnixMethod = UnixMethod::McpListTools;
    pub const REPL_EXEC: UnixMethod = UnixMethod::ReplExec;
    pub const REPL_SNAPSHOT: UnixMethod = UnixMethod::ReplSnapshot;
    pub const REPL_SNAPSHOT_MEMORY: UnixMethod = UnixMethod::ReplSnapshotMemory;
    pub const REPL_RESTORE: UnixMethod = UnixMethod::ReplRestore;
    pub const COSMOS_STATUS: UnixMethod = UnixMethod::CosmosStatus;
    pub const COSMOS_SET_ALLOWED_TOOLS: UnixMethod = UnixMethod::CosmosSetAllowedTools;
    pub const COSMOS_SET_RAG_CONTEXT: UnixMethod = UnixMethod::CosmosSetRagContext;
    pub const BRIDGE_CALL: UnixMethod = UnixMethod::BridgeCall;
    pub const BRIDGE_LIST_TOOLS: UnixMethod = UnixMethod::BridgeListTools;
    pub const TUI_USER_MESSAGE: UnixMethod = UnixMethod::TuiUserMessage;
    pub const TUI_AGENT_RESPONSE: UnixMethod = UnixMethod::TuiAgentResponse;
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
    use anyhow::{Context, Result, bail};

    #[test]
    fn request_serializes_correctly() -> Result<()> {
        let req = JsonRpcRequest::new_raw("mcp.call", Some(serde_json::json!({"tool": "test"})));
        let json = serde_json::to_string(&req)?;
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"mcp.call\""));
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
            },
            _ => bail!("Expected Request"),
        }
        Ok(())
    }

    #[test]
    fn typed_constructors_use_correct_methods() -> Result<()> {
        let req = JsonRpcRequest::mcp_call("test_tool", serde_json::Value::Null);
        assert_eq!(req.method, "mcp.call");
        assert!(req.params.is_some());

        let req = JsonRpcRequest::mcp_list_tools();
        assert_eq!(req.method, "mcp.list_tools");
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
            UnixMethod::McpCall,
            UnixMethod::McpListTools,
            UnixMethod::ReplExec,
            UnixMethod::ReplSnapshot,
            UnixMethod::ReplSnapshotMemory,
            UnixMethod::ReplRestore,
            UnixMethod::CosmosStatus,
            UnixMethod::CosmosSetAllowedTools,
            UnixMethod::CosmosSetRagContext,
            UnixMethod::BridgeCall,
            UnixMethod::BridgeListTools,
            UnixMethod::TuiUserMessage,
            UnixMethod::TuiAgentResponse,
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
}
