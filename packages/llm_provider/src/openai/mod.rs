mod request;
mod response;

use futures::stream;
use serde::Serialize;

use async_trait::async_trait;
use request::build_tools;
pub use request::{
    OpenAiContent, OpenAiContentPart, OpenAiFunctionDef, OpenAiImageUrl, OpenAiMessage,
    OpenAiMessageToolCall, OpenAiRequest, OpenAiTool,
};
pub use response::{
    OpenAiChoice, OpenAiFunction, OpenAiMessageResponse, OpenAiPromptTokensDetails, OpenAiResponse,
    OpenAiToolCall, OpenAiUsageResponse,
};
use response::{OpenAiStreamChunkRaw, parse_openai_stream};

use super::{
    EXPECTED_VALID_JSON, FinishReason, LlmChatRequest, LlmChatResponse, LlmProvider,
    LlmStreamChunk, LlmUsage, ProviderConfig, StreamResult, ToolCall, ToolChoice,
};
use crate::errors::ProviderError;

#[derive(Serialize)]
struct ToolChoiceFunctionNamed<'a> {
    r#type: &'a str,
    function: FunctionNameRef<'a>,
}

#[derive(Serialize)]
struct FunctionNameRef<'a> {
    name: &'a str,
}

fn convert_tool_choice(choice: Option<&ToolChoice>) -> Option<serde_json::Value> {
    match choice {
        Some(ToolChoice::Auto) | None => None,
        Some(ToolChoice::None) => Some(serde_json::Value::String("none".into())),
        Some(ToolChoice::Required) => Some(serde_json::Value::String("required".into())),
        Some(ToolChoice::Named(name)) => Some(
            serde_json::to_value(ToolChoiceFunctionNamed {
                r#type: "function",
                function: FunctionNameRef { name },
            })
            .unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Default)]
pub struct OpenAiCompatibleProvider {
    pub name: String,
    pub base_url: String,
    pub default_model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            base_url: String::new(),
            default_model: String::new(),
        }
    }

    fn get_base_url(&self, config: &ProviderConfig) -> String {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| self.base_url.clone())
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_name(&self) -> &str {
        &self.name
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

    async fn chat(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<LlmChatResponse, ProviderError> {
        let client = self.create_http_client(config.timeout_duration())?;
        let base_url = self.get_base_url(config);
        let url = format!("{}/chat/completions", base_url);
        let model = self.resolve_model(&request, config);

        let openai_request = OpenAiRequest {
            model,
            messages: request
                .messages
                .into_iter()
                .map(OpenAiMessage::from_llm_message)
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: None,
            tools: build_tools(&request.tools),
            tool_choice: convert_tool_choice(request.tool_choice.as_ref()),
            thinking: maybe_disable_thinking(),
        };

        let req = config
            .build_authenticated_post(&client, &url)
            .json(&openai_request);

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_http_error(status.as_u16(), body, retry_after));
        }

        let openai_response: OpenAiResponse =
            response
                .json()
                .await
                .map_err(|e| ProviderError::InvalidResponse {
                    expected: EXPECTED_VALID_JSON.into(),
                    got: format!("parse error: {}", e),
                })?;

        let choice =
            openai_response
                .choices
                .first()
                .ok_or_else(|| ProviderError::InvalidResponse {
                    expected: "at least one choice".into(),
                    got: "no choices in response".into(),
                })?;

        let tool_calls = choice
            .message
            .tool_calls
            .as_ref()
            .map(|tc| {
                tc.iter()
                    .map(|t| ToolCall {
                        id: t.id.clone(),
                        name: t.function.name.clone(),
                        arguments: t.function.arguments.clone(),
                        integrity: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(LlmChatResponse {
            content: choice.message.content.clone().unwrap_or_default(),
            tool_calls,
            finish_reason: choice
                .finish_reason
                .as_deref()
                .map(FinishReason::from)
                .unwrap_or_default(),
            usage: openai_response.usage.map(|u| LlmUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
                cached_tokens: u.prompt_tokens_details.map(|d| d.cached_tokens),
            }),
            images: None,
        })
    }

    async fn chat_stream(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<StreamResult, ProviderError> {
        let client = self.create_http_client(config.timeout_duration())?;
        let base_url = self.get_base_url(config);
        let url = format!("{}/chat/completions", base_url);
        let model = self.resolve_model(&request, config);

        let openai_request = OpenAiRequest {
            model,
            messages: request
                .messages
                .into_iter()
                .map(OpenAiMessage::from_llm_message)
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(true),
            tools: build_tools(&request.tools),
            tool_choice: convert_tool_choice(request.tool_choice.as_ref()),
            thinking: maybe_disable_thinking(),
        };

        tracing::debug!(
            url = %url,
            request_body = %serde_json::to_string(&openai_request).unwrap_or_default(),
            "OpenAI-compatible chat_stream request"
        );

        // Send as a normal POST and parse SSE manually.
        // reqwest-eventsource 0.6 drops the POST body, causing 400 on providers
        // like BigModel that require a body. Manual SSE parsing avoids this.
        let req = config
            .build_authenticated_post(&client, &url)
            .json(&openai_request);

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_http_error(status.as_u16(), body, retry_after));
        }

        use futures::StreamExt as _;
        let byte_stream = response.bytes_stream();

        struct SseState<S> {
            byte_stream: S,
            buffer: String,
            pending: std::collections::VecDeque<LlmStreamChunk>,
        }

        let s = stream::unfold(
            SseState {
                byte_stream,
                buffer: String::new(),
                pending: std::collections::VecDeque::new(),
            },
            |mut state| async move {
                if let Some(chunk) = state.pending.pop_front() {
                    return Some((Ok(chunk), state));
                }

                loop {
                    let (events, remaining) = crate::sse_util::extract_sse_events(&state.buffer);
                    let consumed = state.buffer.len() - remaining.len();
                    state.buffer.drain(..consumed);

                    for data in events {
                        if data == crate::sse_util::DONE {
                            return None;
                        }
                        if let Ok(chunk_raw) = serde_json::from_str::<OpenAiStreamChunkRaw>(&data) {
                            for c in parse_openai_stream(&chunk_raw) {
                                state.pending.push_back(c);
                            }
                        }
                    }

                    if let Some(chunk) = state.pending.pop_front() {
                        return Some((Ok(chunk), state));
                    }

                    // Need more bytes from the response stream
                    match state.byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            state.buffer.push_str(&String::from_utf8_lossy(&bytes));
                        }
                        Some(Err(e)) => {
                            return Some((
                                Err(ProviderError::NetworkError(format!("Stream error: {}", e))),
                                state,
                            ));
                        }
                        None => return None,
                    }
                }
            },
        );

        Ok(Box::pin(s))
    }

    fn list_models(&self) -> Vec<String> {
        vec![self.default_model.clone()]
    }
}

fn maybe_disable_thinking() -> Option<serde_json::Value> {
    // Thinking is controlled per-provider. For GLM coding models that
    // support tool use, thinking should NOT be disabled — the model needs
    // reasoning to decide which tools to call. Only disable when explicitly
    // set via env var.
    let disabled = std::env::var("LLM_DISABLE_THINKING")
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);
    if disabled {
        Some(serde_json::json!({"type": "disabled"}))
    } else {
        None
    }
}
