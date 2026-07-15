mod request;
mod response;

use futures::stream;
use std::time::Duration;

use async_trait::async_trait;
use request::{AnthropicRequest, AnthropicToolChoice, convert_messages, convert_tools};
use response::{
    AnthropicResponse, AnthropicStreamContentBlockStart, AnthropicStreamDelta, AnthropicStreamEvent,
};
use tracing::warn;

use super::{
    CHOICE_AUTO, DEFAULT_ANTHROPIC_TIMEOUT_SECS, EXPECTED_VALID_JSON, FinishReason, LlmChatRequest,
    LlmChatResponse, LlmProvider, LlmStreamChunk, LlmUsage, ProviderConfig, StreamResult, ToolCall,
    ToolCallDelta, ToolChoice,
};
use crate::errors::ProviderError;

fn convert_tool_choice(
    choice: Option<&ToolChoice>,
    has_tools: bool,
) -> Option<AnthropicToolChoice> {
    if !has_tools {
        return None;
    }
    match choice {
        Some(ToolChoice::Auto) | None => Some(AnthropicToolChoice {
            choice_type: CHOICE_AUTO.into(),
            name: None,
        }),
        Some(ToolChoice::Required) => Some(AnthropicToolChoice {
            choice_type: "any".into(),
            name: None,
        }),
        Some(ToolChoice::Named(name)) => Some(AnthropicToolChoice {
            choice_type: "tool".into(),
            name: Some(name.clone()),
        }),
        Some(ToolChoice::None) => None,
    }
}

#[derive(Clone)]
pub struct AnthropicProvider {
    base_url: String,
    default_model: String,
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            base_url: String::new(),
            default_model: String::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_name(&self) -> &str {
        "anthropic"
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
        let timeout = Duration::from_secs(
            config
                .request_timeout_secs
                .unwrap_or(DEFAULT_ANTHROPIC_TIMEOUT_SECS),
        );
        let client = self.create_http_client(timeout)?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| self.base_url.clone());
        let url = format!("{}/v1/messages", base_url);
        let model = self.resolve_model(&request, config);

        let (system, messages) = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let tool_choice =
            convert_tool_choice(request.tool_choice.as_ref(), tools.as_ref().is_some());

        let anthropic_request = AnthropicRequest {
            model,
            messages,
            max_tokens: request.max_tokens.unwrap_or(16384),
            system,
            temperature: request.temperature,
            stream: None,
            tools,
            tool_choice,
        };

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", config.api_key_str())
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
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

        let anthropic_response: AnthropicResponse =
            response
                .json()
                .await
                .map_err(|e| ProviderError::InvalidResponse {
                    expected: EXPECTED_VALID_JSON.into(),
                    got: format!("parse error: {}", e),
                })?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for block in anthropic_response.content {
            match block {
                response::AnthropicContent::Text { text: Some(t) } => {
                    content.push_str(&t);
                }
                response::AnthropicContent::Text { text: None } => {
                    warn!(
                        "Anthropic returned Text block with None content: {:?}",
                        block
                    );
                }
                response::AnthropicContent::Thinking { thinking, .. } => {
                    if let Some(t) = thinking {
                        content.push_str(&t);
                    } else {
                        warn!("Anthropic returned Thinking block with None content");
                    }
                }
                response::AnthropicContent::ToolUse { id, name, input } => {
                    if let (Some(id), Some(name), Some(input)) = (id, name, input) {
                        tool_calls.push(ToolCall {
                            id,
                            name,
                            arguments: serde_json::to_string(&input).unwrap_or_else(|e| {
                                warn!(error=%e, "Anthropic tool input serialization failed");
                                "{}".to_string()
                            }),
                            integrity: None,
                        });
                    }
                }
            }
        }

        let usage = anthropic_response.usage.map(|u| LlmUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
            cached_tokens: u.cache_read_input_tokens,
        });

        Ok(LlmChatResponse {
            content,
            tool_calls,
            finish_reason: anthropic_response
                .stop_reason
                .as_deref()
                .map(FinishReason::from)
                .unwrap_or_default(),
            usage,
            images: None,
        })
    }

    async fn chat_stream(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<StreamResult, ProviderError> {
        let timeout = Duration::from_secs(
            config
                .request_timeout_secs
                .unwrap_or(DEFAULT_ANTHROPIC_TIMEOUT_SECS),
        );
        let client = self.create_http_client(timeout)?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| self.base_url.clone());
        let url = format!("{}/v1/messages", base_url);
        let model = self.resolve_model(&request, config);

        let (system, messages) = convert_messages(&request.messages);
        let tools = request.tools.as_ref().map(|t| convert_tools(t));

        let tool_choice =
            convert_tool_choice(request.tool_choice.as_ref(), tools.as_ref().is_some());

        let anthropic_request = AnthropicRequest {
            model,
            messages,
            max_tokens: request.max_tokens.unwrap_or(16384),
            system,
            temperature: request.temperature,
            stream: Some(true),
            tools,
            tool_choice,
        };

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", config.api_key_str())
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
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

        let s = stream::unfold(
            (byte_stream, String::new()),
            |(mut byte_stream, mut buffer)| async move {
                loop {
                    while let Some(pos) = buffer.find("\n\n") {
                        let event = buffer[..pos].to_string();
                        let keep_from = pos + 2;
                        buffer.drain(..keep_from);

                        for line in event.lines() {
                            let data = line
                                .strip_prefix("data: ")
                                .or_else(|| line.strip_prefix("data:"))
                                .map(str::trim);
                            let data = match data {
                                Some(d) => d,
                                None => continue,
                            };

                            let event: AnthropicStreamEvent = match serde_json::from_str(data) {
                                Ok(e) => e,
                                Err(_) => continue,
                            };

                            let chunk: Option<LlmStreamChunk> = match event {
                                AnthropicStreamEvent::ContentBlockDelta { delta } => match delta {
                                    AnthropicStreamDelta::Text { text } => Some(LlmStreamChunk {
                                        content: Some(text),
                                        tool_call: None,
                                        finish_reason: None,
                                        usage: None,
                                    }),
                                    AnthropicStreamDelta::InputJson { partial_json } => {
                                        Some(LlmStreamChunk {
                                            content: None,
                                            tool_call: Some(ToolCallDelta {
                                                id: None,
                                                name: None,
                                                arguments: Some(partial_json),
                                                index: None,
                                                integrity: None,
                                            }),
                                            finish_reason: None,
                                            usage: None,
                                        })
                                    }
                                    _ => None,
                                },
                                AnthropicStreamEvent::ContentBlockStart { content_block } => {
                                    match content_block {
                                        AnthropicStreamContentBlockStart::ToolUse { id, name } => {
                                            Some(LlmStreamChunk {
                                                content: None,
                                                tool_call: Some(ToolCallDelta {
                                                    id: Some(id),
                                                    name: Some(name),
                                                    arguments: None,
                                                    index: None,
                                                    integrity: None,
                                                }),
                                                finish_reason: None,
                                                usage: None,
                                            })
                                        }
                                        _ => None,
                                    }
                                }
                                AnthropicStreamEvent::MessageStart { message } => {
                                    Some(LlmStreamChunk {
                                        content: None,
                                        tool_call: None,
                                        finish_reason: None,
                                        usage: Some(LlmUsage {
                                            prompt_tokens: message.usage.input_tokens,
                                            completion_tokens: 0,
                                            total_tokens: message.usage.input_tokens,
                                            cached_tokens: message.usage.cache_read_input_tokens,
                                        }),
                                    })
                                }
                                AnthropicStreamEvent::MessageDelta { delta, usage } => {
                                    Some(LlmStreamChunk {
                                        content: None,
                                        tool_call: None,
                                        finish_reason: delta
                                            .stop_reason
                                            .map(|s| FinishReason::from(s.as_str())),
                                        usage: Some(LlmUsage {
                                            prompt_tokens: 0,
                                            completion_tokens: usage.output_tokens,
                                            total_tokens: usage.output_tokens,
                                            cached_tokens: None,
                                        }),
                                    })
                                }
                                AnthropicStreamEvent::MessageStop => {
                                    return None;
                                }
                            };
                            if let Some(c) = chunk {
                                return Some((
                                    Ok(c) as Result<LlmStreamChunk, ProviderError>,
                                    (byte_stream, buffer),
                                ));
                            }
                        }
                    }

                    match byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                        }
                        Some(Err(e)) => {
                            return Some((
                                Err(ProviderError::NetworkError(format!("Stream error: {}", e))),
                                (byte_stream, buffer),
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
