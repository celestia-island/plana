mod request;
mod response;

use futures::stream;
use std::sync::atomic::{AtomicU64, Ordering};

static CALL_COUNTER: AtomicU64 = AtomicU64::new(0);

use async_trait::async_trait;
use request::{
    GeminiFunctionCallingConfig, GeminiGenerationConfig, GeminiRequest, GeminiRequestPart,
    GeminiSystemInstruction, GeminiToolConfig, convert_messages, convert_tools,
};
use response::GeminiResponse;
use tracing::warn;

use super::{
    EXPECTED_VALID_JSON, FinishReason, LlmChatRequest, LlmChatResponse, LlmProvider,
    LlmStreamChunk, LlmUsage, MODE_AUTO, ProviderConfig, StreamResult, ToolCall, ToolChoice,
};
use crate::errors::ProviderError;

fn convert_tool_choice(choice: Option<&ToolChoice>, has_tools: bool) -> Option<GeminiToolConfig> {
    if !has_tools {
        return None;
    }
    match choice {
        Some(ToolChoice::Auto) | None => Some(GeminiToolConfig {
            function_calling_config: GeminiFunctionCallingConfig {
                mode: MODE_AUTO.into(),
                allowed_function_names: None,
            },
        }),
        Some(ToolChoice::Required) => Some(GeminiToolConfig {
            function_calling_config: GeminiFunctionCallingConfig {
                mode: "ANY".into(),
                allowed_function_names: None,
            },
        }),
        Some(ToolChoice::Named(name)) => Some(GeminiToolConfig {
            function_calling_config: GeminiFunctionCallingConfig {
                mode: "ANY".into(),
                allowed_function_names: Some(vec![name.clone()]),
            },
        }),
        Some(ToolChoice::None) => Some(GeminiToolConfig {
            function_calling_config: GeminiFunctionCallingConfig {
                mode: "NONE".into(),
                allowed_function_names: None,
            },
        }),
    }
}

#[derive(Clone)]
pub struct GeminiProvider {
    base_url: String,
    default_model: String,
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            base_url: String::new(),
            default_model: String::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn provider_name(&self) -> &str {
        "gemini"
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
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| self.base_url.clone());
        let model = self.resolve_model(&request, config);

        let (system, contents) = convert_messages(&request.messages);
        let tools_json = request.tools.as_ref().map(|t| convert_tools(t));
        let tool_config = convert_tool_choice(request.tool_choice.as_ref(), tools_json.is_some());

        let system_instruction = system.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiRequestPart::Text { text: s }],
        });

        let generation_config = GeminiGenerationConfig {
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
        };

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(generation_config),
            tools: tools_json,
            tool_config,
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            base_url,
            model,
            config.api_key_str()
        );

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&gemini_request)
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

        let gemini_response: GeminiResponse =
            response
                .json()
                .await
                .map_err(|e| ProviderError::InvalidResponse {
                    expected: EXPECTED_VALID_JSON.into(),
                    got: format!("parse error: {}", e),
                })?;

        let candidate = gemini_response
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::InvalidResponse {
                expected: "at least one candidate".into(),
                got: "no candidates in response".into(),
            })?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for part in candidate.content.parts {
            if let Some(text) = part.text {
                content.push_str(&text);
            }
            if let Some(fc) = part.function_call {
                let args = fc
                    .args
                    .map(|v| {
                        serde_json::to_string(&v).unwrap_or_else(|e| {
                            warn!(error=%e, "Gemini tool input serialization failed");
                            "{}".to_string()
                        })
                    })
                    .unwrap_or_default();
                tool_calls.push(ToolCall {
                    id: format!(
                        "gemini-call-{:016x}",
                        CALL_COUNTER.fetch_add(1, Ordering::Relaxed)
                    ),
                    name: fc.name,
                    arguments: args,
                    integrity: None,
                });
            }
        }

        let usage = gemini_response.usage_metadata.map(|u| LlmUsage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
            cached_tokens: None,
        });

        Ok(LlmChatResponse {
            content,
            tool_calls,
            finish_reason: candidate
                .finish_reason
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
        let client = self.create_http_client(config.timeout_duration())?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| self.base_url.clone());
        let model = self.resolve_model(&request, config);

        let (system, contents) = convert_messages(&request.messages);
        let tools_json = request.tools.as_ref().map(|t| convert_tools(t));
        let tool_config = convert_tool_choice(request.tool_choice.as_ref(), tools_json.is_some());

        let system_instruction = system.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiRequestPart::Text { text: s }],
        });

        let generation_config = GeminiGenerationConfig {
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
        };

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config: Some(generation_config),
            tools: tools_json,
            tool_config,
        };

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}&alt=sse",
            base_url,
            model,
            config.api_key_str()
        );

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&gemini_request)
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

        let s = stream::try_unfold(
            SseState {
                byte_stream,
                buffer: String::new(),
                pending: std::collections::VecDeque::new(),
            },
            |mut state: SseState<_>| async move {
                if let Some(chunk) = state.pending.pop_front() {
                    return Ok(Some((chunk, state)));
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
                            if let Some(data) = data {
                                if data == "[DONE]" {
                                    return Ok(None);
                                }
                                let gemini_response: GeminiResponse =
                                    serde_json::from_str(data).unwrap_or_default();

                                let candidate = gemini_response.candidates.into_iter().next();
                                if let Some(cand) = candidate {
                                    let mut text_parts = Vec::new();
                                    let mut fc_parts = Vec::new();
                                    for part in cand.content.parts {
                                        if let Some(text) = part.text {
                                            text_parts.push(text);
                                        }
                                        if let Some(fc) = part.function_call {
                                            fc_parts.push(fc);
                                        }
                                    }
                                    let text_content = if text_parts.is_empty() {
                                        None
                                    } else {
                                        Some(text_parts.join(""))
                                    };
                                    let usage = gemini_response.usage_metadata.map(|u| LlmUsage {
                                        prompt_tokens: u.prompt_token_count,
                                        completion_tokens: u.candidates_token_count,
                                        total_tokens: u.total_token_count,
                                        cached_tokens: None,
                                    });
                                    if text_content.is_some() || usage.is_some() {
                                        state.pending.push_back(LlmStreamChunk {
                                            content: text_content,
                                            tool_call: None,
                                            finish_reason: None,
                                            usage,
                                        });
                                    }
                                    for fc in fc_parts {
                                        state.pending.push_back(LlmStreamChunk {
                                            content: None,
                                            tool_call: Some(super::ToolCallDelta {
                                                id: Some(fc.name.clone()),
                                                name: Some(fc.name),
                                                arguments: fc.args.map(|v| {
                                                    serde_json::to_string(&v).unwrap_or_else(|e| {
                                                        warn!(error=%e, "Gemini streaming tool input serialization failed");
                                                        "{}".to_string()
                                                    })
                                                }),
                                                index: None,
                                                integrity: None,
                                            }),
                                            finish_reason: None,
                                            usage: None,
                                        });
                                    }
                                    if let Some(fr) = cand.finish_reason {
                                        state.pending.push_back(LlmStreamChunk {
                                            content: None,
                                            tool_call: None,
                                            finish_reason: Some(FinishReason::from(fr.as_str())),
                                            usage: None,
                                        });
                                    }
                                }
                            }
                        }

                        if let Some(chunk) = state.pending.pop_front() {
                            return Ok(Some((chunk, state)));
                        }
                    }

                    match state.byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            state.buffer.push_str(&String::from_utf8_lossy(&bytes));
                        },
                        Some(Err(e)) => {
                            return Err(ProviderError::NetworkError(format!(
                                "Stream error: {}",
                                e
                            )));
                        },
                        None => return Ok(None),
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
