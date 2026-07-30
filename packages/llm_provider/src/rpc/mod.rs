use async_trait::async_trait;
use futures::stream;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};

use crate::errors::ProviderError;
use crate::{
    FinishReason, LlmChatRequest, LlmChatResponse, LlmProvider, LlmStreamChunk, LlmUsage,
    ProviderConfig, StreamResult,
};

const JSONRPC_VERSION: &str = "2.0";
const CHAT_SEND: &str = "chat.send";
const CHAT_STREAM: &str = "chat.stream";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatSendParams {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_memory_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_memory_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_blocks: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatSendResult {
    stream_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum RpcMessage {
    Response {
        jsonrpc: String,
        id: serde_json::Value,
        #[serde(default)]
        result: Option<Value>,
        #[serde(default)]
        error: Option<Value>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default)]
        params: Option<Value>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct ChatStreamParams {
    stream_id: String,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    is_complete: bool,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    tool_calls: Option<Value>,
    #[serde(default)]
    usage: Option<Value>,
}

#[derive(Clone, Default)]
pub struct RpcProvider {
    pub default_model: String,
}

impl RpcProvider {
    pub fn new() -> Self {
        Self {
            default_model: String::new(),
        }
    }

    fn get_base_url(&self, config: &ProviderConfig) -> String {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:8420".to_string())
    }
}

#[async_trait]
impl LlmProvider for RpcProvider {
    fn provider_name(&self) -> &str {
        "rpc"
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        true
    }

    fn supports_multimodal(&self) -> bool {
        true
    }

    async fn chat(
        &self,
        _request: LlmChatRequest,
        _config: &ProviderConfig,
    ) -> Result<LlmChatResponse, ProviderError> {
        Err(ProviderError::ConfigError {
            key: "rpc".into(),
            reason: "Non-streaming chat not supported for RPC provider".into(),
        })
    }

    async fn chat_stream(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<StreamResult, ProviderError> {
        let base_url = self.get_base_url(config);
        let model = self.resolve_model(&request, config);

        let url = format!("{}/api/rpc", base_url)
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let mut ws_request = url
            .into_client_request()
            .map_err(|e| {
                ProviderError::NetworkError(format!("Invalid WS URL: {}", e))
            })?;
        let headers = ws_request.headers_mut();
        headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&format!("Bearer {}", config.api_key_str()))
                .map_err(|e| ProviderError::NetworkError(format!("Invalid auth header: {}", e)))?,
        );

        let (mut ws_stream, _) = connect_async(ws_request)
            .await
            .map_err(|e| ProviderError::NetworkError(format!("WebSocket connect failed: {}", e)))?;

        let messages: Vec<ChatMessage> = request
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role.as_str().to_string(),
                content: m.content.clone(),
                tool_call_id: m.tool_call_id.clone(),
                tool_calls: m
                    .tool_calls
                    .as_ref()
                    .map(|tc| serde_json::to_value(tc).unwrap_or_default()),
                images: m
                    .images
                    .as_ref()
                    .map(|imgs| serde_json::to_value(imgs).unwrap_or_default()),
                content_blocks: m
                    .content_blocks
                    .as_ref()
                    .map(|cb| serde_json::to_value(cb).unwrap_or_default()),
            })
            .collect();

        let tool_choice = match &request.tool_choice {
            Some(crate::ToolChoice::Auto) => None,
            Some(crate::ToolChoice::None) => Some(Value::String("none".into())),
            Some(crate::ToolChoice::Required) => Some(Value::String("required".into())),
            Some(crate::ToolChoice::Named(name)) => Some(serde_json::json!({
                "type": "function",
                "function": { "name": name }
            })),
            None => None,
        };

        let params = ChatSendParams {
            model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: request
                .tools
                .as_ref()
                .map(|t| serde_json::to_value(t).unwrap_or_default()),
            tool_choice,
            user_memory_id: request.user_memory_id.clone(),
            workspace_memory_id: request.workspace_memory_id.clone(),
        };

        let rpc_request = serde_json::json!({
            "jsonrpc": JSONRPC_VERSION,
            "method": CHAT_SEND,
            "params": serde_json::to_value(&params).unwrap_or_default(),
            "id": "1",
        });

        let req_text =
            serde_json::to_string(&rpc_request).map_err(|e| ProviderError::InvalidResponse {
                expected: "valid JSON-RPC request".into(),
                got: format!("serialization error: {}", e),
            })?;

        ws_stream
            .send(Message::Text(req_text.into()))
            .await
            .map_err(|e| ProviderError::NetworkError(format!("WS send failed: {}", e)))?;

        let (tx, rx) = mpsc::channel::<Result<LlmStreamChunk, ProviderError>>(64);

        tokio::spawn(async move {
            let mut stream_id: Option<String> = None;

            loop {
                match ws_stream.next().await {
                    Some(Ok(Message::Text(text))) => {
                        debug!(?text, "RPC WS message received");
                        let rpc_msg: RpcMessage = match serde_json::from_str(&text) {
                            Ok(msg) => msg,
                            Err(e) => {
                                warn!(error=%e, text=%text, "Failed to parse RPC message");
                                continue;
                            }
                        };

                        match rpc_msg {
                            RpcMessage::Response {
                                result: Some(result),
                                ..
                            } => {
                                if stream_id.is_none() {
                                    if let Ok(chat_result) =
                                        serde_json::from_value::<ChatSendResult>(result)
                                    {
                                        stream_id = Some(chat_result.stream_id);
                                    }
                                }
                            }
                            RpcMessage::Response {
                                error: Some(err), ..
                            } => {
                                let err_msg = err
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown RPC error");
                                let _ = tx
                                    .send(Err(ProviderError::InvalidResponse {
                                        expected: "successful RPC response".into(),
                                        got: err_msg.to_string(),
                                    }))
                                    .await;
                                return;
                            }
                            RpcMessage::Notification {
                                method,
                                params: Some(params),
                                ..
                            } if method == CHAT_STREAM => {
                                if let Ok(sp) = serde_json::from_value::<ChatStreamParams>(params) {
                                    if stream_id.as_ref().is_none_or(|s| s != &sp.stream_id) {
                                        continue;
                                    }

                                    let content = sp.token.clone();
                                    let tool_call = sp.tool_calls.as_ref().and_then(|tc| {
                                        if let Some(arr) = tc.as_array() {
                                            arr.first().and_then(|t| {
                                                let id = t
                                                    .get("id")
                                                    .and_then(|v| v.as_str())
                                                    .map(String::from);
                                                let name = t
                                                    .get("name")
                                                    .and_then(|v| v.as_str())
                                                    .map(String::from);
                                                let arguments = t
                                                    .get("arguments")
                                                    .and_then(|v| v.as_str())
                                                    .map(String::from);
                                                Some(crate::ToolCallDelta {
                                                    id,
                                                    name,
                                                    arguments,
                                                    index: t
                                                        .get("index")
                                                        .and_then(|v| v.as_u64())
                                                        .map(|i| i as u32),
                                                    integrity: None,
                                                })
                                            })
                                        } else {
                                            None
                                        }
                                    });

                                    let finish_reason =
                                        sp.finish_reason.as_deref().map(FinishReason::from);

                                    let usage = sp.usage.as_ref().and_then(|u| {
                                        serde_json::from_value::<LlmUsage>(u.clone()).ok()
                                    });

                                    if sp.is_complete
                                        && content.is_none()
                                        && tool_call.is_none()
                                        && finish_reason.is_none()
                                        && usage.is_none()
                                    {
                                        let _ = tx
                                            .send(Ok(LlmStreamChunk {
                                                content: None,
                                                tool_call: None,
                                                finish_reason: Some(FinishReason::Stop),
                                                usage: None,
                                            }))
                                            .await;
                                        return;
                                    }

                                    if tx
                                        .send(Ok(LlmStreamChunk {
                                            content,
                                            tool_call,
                                            finish_reason,
                                            usage,
                                        }))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }

                                    if sp.is_complete {
                                        if tx
                                            .send(Ok(LlmStreamChunk {
                                                content: None,
                                                tool_call: None,
                                                finish_reason: Some(FinishReason::Stop),
                                                usage: sp.usage.as_ref().and_then(|u| {
                                                    serde_json::from_value::<LlmUsage>(u.clone()).ok()
                                                }),
                                            }))
                                            .await
                                            .is_err()
                                        {}
                                        return;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        return;
                    }
                    Some(Err(e)) => {
                        let _ = tx
                            .send(Err(ProviderError::NetworkError(format!("WS error: {}", e))))
                            .await;
                        return;
                    }
                    None => {
                        return;
                    }
                    _ => {}
                }
            }
        });

        let s = stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        });
        Ok(Box::pin(s))
    }

    fn list_models(&self) -> Vec<String> {
        vec![self.default_model.clone()]
    }
}
