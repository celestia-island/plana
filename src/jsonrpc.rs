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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new_uuid() -> Self {
        Self::String(uuid::Uuid::now_v7().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let notif = JsonRpcNotification::new(
        method,
        Some(serde_json::to_value(&params).unwrap_or_default()),
    );
    serde_json::to_string(&notif)
        .unwrap_or_else(|_| format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":{{}}}}"#))
}

pub fn build_notification_value(method: &str, params: impl serde::Serialize) -> serde_json::Value {
    let notif = JsonRpcNotification::new(
        method,
        Some(serde_json::to_value(&params).unwrap_or_default()),
    );
    serde_json::to_value(notif).unwrap_or_else(|_| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {}
        })
    })
}
