mod request;
mod response;

pub mod sse_events {
    pub const OUTPUT_TEXT_DELTA: &str = "response.output_text.delta";
    pub const OUTPUT_ITEM_DONE: &str = "response.output_item.done";
    pub const RESPONSE_COMPLETED: &str = "response.completed";
    pub const RESPONSE_INCOMPLETE: &str = "response.incomplete";
    pub const ITEM_TYPE_FUNCTION_CALL: &str = "function_call";
    pub const ITEM_TYPE_MESSAGE: &str = "message";
}

pub mod sse_signals {
    pub const DONE: &str = "[DONE]";
}

use sse_events::*;
use sse_signals::DONE;

use futures::stream;
use serde::Serialize;
use serde_json::Value;

use async_trait::async_trait;
use request::{ResponsesRequest, build_request, convert_input_items, convert_tools};
use response::ResponsesApiResponse;
use tracing::{debug, instrument, trace, warn};

use super::{
    EXPECTED_VALID_JSON, FinishReason, LlmChatRequest, LlmChatResponse, LlmProvider,
    LlmStreamChunk, LlmUsage, ProviderConfig, StreamResult, ToolCallDelta, ToolChoice,
};
use crate::errors::ProviderError;

#[derive(Serialize)]
struct ResponsesToolChoiceNamed<'a> {
    r#type: &'a str,
    name: &'a str,
}

fn convert_tool_choice(choice: Option<&ToolChoice>) -> Option<Value> {
    match choice {
        Some(ToolChoice::Auto) | None => None,
        Some(ToolChoice::None) => Some(Value::String("none".into())),
        Some(ToolChoice::Required) => Some(Value::String("required".into())),
        Some(ToolChoice::Named(name)) => Some(
            serde_json::to_value(ResponsesToolChoiceNamed {
                r#type: "function",
                name,
            })
            .unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Default)]
pub struct OpenAiResponsesProvider {
    pub name: String,
}

impl OpenAiResponsesProvider {
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }

    fn get_endpoint(&self, config: &ProviderConfig) -> String {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string())
    }
}

#[async_trait]
impl LlmProvider for OpenAiResponsesProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn default_model(&self) -> &str {
        "gpt-4o"
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    #[instrument(skip(self, request, config), fields(model = %self.resolve_model(&request, config), endpoint = %self.get_endpoint(config)))]
    async fn chat(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<LlmChatResponse, ProviderError> {
        let client = self.create_http_client(config.timeout_duration())?;
        let endpoint = self.get_endpoint(config);
        let model = self.resolve_model(&request, config);

        let input = convert_input_items(request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let req_body = ResponsesRequest {
            model,
            input,
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
            stream: None,
            tools,
            tool_choice: convert_tool_choice(request.tool_choice.as_ref()),
        };

        let req = build_request(&endpoint, &client, config, &req_body);
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

        let api_response: ResponsesApiResponse =
            response
                .json()
                .await
                .map_err(|e| ProviderError::InvalidResponse {
                    expected: EXPECTED_VALID_JSON.into(),
                    got: format!("parse error: {}", e),
                })?;

        let (content, tool_calls) = api_response.extract_text_and_tool_calls();
        let is_incomplete = api_response.status.as_deref() == Some("incomplete");
        let finish_reason = if is_incomplete {
            FinishReason::Length
        } else if !tool_calls.is_empty() {
            FinishReason::ToolUse
        } else {
            FinishReason::Stop
        };
        let usage = api_response.usage.map(|u| LlmUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.total_tokens,
            cached_tokens: None,
        });

        debug!(
            input_tokens = usage.as_ref().map_or(0, |u| u.prompt_tokens),
            output_tokens = usage.as_ref().map_or(0, |u| u.completion_tokens),
            content_len = content.len(),
            tool_calls = tool_calls.len(),
            "Non-streaming response completed"
        );

        Ok(LlmChatResponse {
            content,
            tool_calls,
            finish_reason,
            usage,
            images: None,
        })
    }

    #[instrument(skip(self, request, config), fields(model = %self.resolve_model(&request, config), endpoint = %self.get_endpoint(config)))]
    async fn chat_stream(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<StreamResult, ProviderError> {
        let client = self.create_http_client(config.timeout_duration())?;
        let endpoint = self.get_endpoint(config);
        let model = self.resolve_model(&request, config);

        let input = convert_input_items(request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let req_body = ResponsesRequest {
            model: model.clone(),
            input: input.clone(),
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
            stream: Some(true),
            tools: tools.clone(),
            tool_choice: convert_tool_choice(request.tool_choice.as_ref()),
        };

        debug!(
            input_count = input.len(),
            tools_count = tools.as_ref().map_or(0, |t| t.len()),
            tools_json = %serde_json::to_string(&tools).unwrap_or_default(),
            input_json = %serde_json::to_string(&input).unwrap_or_default(),
            "Starting streaming request"
        );

        let req = build_request(&endpoint, &client, config, &req_body);

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

        debug!("SSE stream started");

        use futures::StreamExt as _;
        let byte_stream = response.bytes_stream();

        struct SseState<S> {
            byte_stream: S,
            buffer: String,
            pending: std::collections::VecDeque<LlmStreamChunk>,
            delta_count: u64,
            tool_call_index: u32,
        }

        let s = stream::unfold(
            SseState {
                byte_stream,
                buffer: String::new(),
                pending: std::collections::VecDeque::new(),
                delta_count: 0,
                tool_call_index: 0,
            },
            |mut state: SseState<_>| async move {
                if let Some(chunk) = state.pending.pop_front() {
                    return Some((Ok(chunk), state));
                }
                loop {
                    while let Some(pos) = state.buffer.find("\n\n") {
                        let event = state.buffer[..pos].to_string();
                        let keep_from = pos + 2;
                        state.buffer.drain(..keep_from);

                        for line in event.lines() {
                            let data = line
                                .strip_prefix("data: ")
                                .or_else(|| line.strip_prefix("data:"))
                                .map(str::trim);
                            let data = match data {
                                Some(d) => d,
                                None => continue,
                            };
                            if data == DONE {
                                debug!(state.delta_count, "SSE stream finished ([DONE])");
                                return None;
                            }

                            let event_val: Value = match serde_json::from_str(data) {
                                Ok(v) => v,
                                Err(e) => {
                                    warn!(error = %e, data_len = data.len(), "Failed to parse SSE JSON");
                                    continue;
                                },
                            };

                            let event_type =
                                event_val.get("type").and_then(|t| t.as_str()).unwrap_or("");

                            match event_type {
                                OUTPUT_TEXT_DELTA => {
                                    let delta = event_val
                                        .get("delta")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("");
                                    if !delta.is_empty() {
                                        state.delta_count += 1;
                                        if state.delta_count <= 3 || state.delta_count % 20 == 0 {
                                            trace!(
                                                delta_chars = delta.len(),
                                                delta_preview = &delta[..delta
                                                    .floor_char_boundary(delta.len().min(80))],
                                                "SSE delta"
                                            );
                                        }
                                        state.pending.push_back(LlmStreamChunk {
                                            content: Some(delta.to_string()),
                                            tool_call: None,
                                            finish_reason: None,
                                            usage: None,
                                        });
                                    }
                                },
                                OUTPUT_ITEM_DONE => {
                                    if let Some(item) = event_val.get("item") {
                                        let item_type =
                                            item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                        if item_type == ITEM_TYPE_FUNCTION_CALL {
                                            let call_id = item
                                                .get("call_id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            let name = item
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            let arguments = item
                                                .get("arguments")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("{}")
                                                .to_string();
                                            let idx = state.tool_call_index;
                                            state.tool_call_index += 1;
                                            debug!(call_id = %call_id, name = %name, idx, "function_call done");
                                            state.pending.push_back(LlmStreamChunk {
                                                content: None,
                                                tool_call: Some(ToolCallDelta {
                                                    id: Some(call_id),
                                                    name: Some(name),
                                                    arguments: Some(arguments),
                                                    index: Some(idx),
                                                    integrity: None,
                                                }),
                                                finish_reason: None,
                                                usage: None,
                                            });
                                        }
                                    }
                                },
                                RESPONSE_COMPLETED => {
                                    let usage = event_val.get("response").and_then(|r| {
                                        r.get("usage").map(|u| LlmUsage {
                                            prompt_tokens: u
                                                .get("input_tokens")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0),
                                            completion_tokens: u
                                                .get("output_tokens")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0),
                                            total_tokens: u
                                                .get("total_tokens")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0),
                                            cached_tokens: None,
                                        })
                                    });
                                    let finish = if state.tool_call_index > 0 {
                                        FinishReason::ToolUse
                                    } else {
                                        FinishReason::Stop
                                    };
                                    debug!(
                                        state.delta_count,
                                        has_usage = usage.is_some(),
                                        tool_calls = state.tool_call_index,
                                        "response.completed"
                                    );
                                    state.pending.push_back(LlmStreamChunk {
                                        content: None,
                                        tool_call: None,
                                        finish_reason: Some(finish),
                                        usage,
                                    });
                                },
                                RESPONSE_INCOMPLETE => {
                                    let reason = event_val
                                        .get("response")
                                        .and_then(|r| r.get("incomplete_details"))
                                        .and_then(|d| d.get("reason"))
                                        .and_then(|r| r.as_str())
                                        .unwrap_or("unknown");
                                    warn!(
                                        event = "openai_responses_incomplete",
                                        reason,
                                        tool_calls = state.tool_call_index,
                                        "OpenAI Responses API returned incomplete response"
                                    );
                                    state.pending.push_back(LlmStreamChunk {
                                        content: None,
                                        tool_call: None,
                                        finish_reason: Some(FinishReason::Length),
                                        usage: None,
                                    });
                                },
                                other => {
                                    trace!(event_type = other, "SSE event ignored");
                                },
                            }
                        }

                        if let Some(chunk) = state.pending.pop_front() {
                            return Some((Ok(chunk), state));
                        }
                    }

                    match state.byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            state.buffer.push_str(&String::from_utf8_lossy(&bytes));
                        },
                        Some(Err(e)) => {
                            warn!(error = %e, "SSE stream error");
                            return Some((
                                Err(ProviderError::NetworkError(format!("Stream error: {}", e))),
                                state,
                            ));
                        },
                        None => {
                            debug!(state.delta_count, "SSE byte stream exhausted");
                            return None;
                        },
                    }
                }
            },
        );

        Ok(Box::pin(s))
    }

    fn list_models(&self) -> Vec<String> {
        Vec::new()
    }
}
