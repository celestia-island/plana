use serde::{Deserialize, Serialize};

use super::super::{FinishReason, LlmStreamChunk, LlmUsage, ToolCallDelta};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAiUsageResponse>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAiChoice {
    pub message: OpenAiMessageResponse,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAiMessageResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAiToolCall {
    pub id: String,
    pub function: OpenAiFunction,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAiFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct OpenAiUsageResponse {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub prompt_tokens_details: Option<OpenAiPromptTokensDetails>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct OpenAiPromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: u64,
}

impl OpenAiResponse {
    pub fn extract_content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.content.as_deref())
    }

    pub fn extract_usage(&self) -> Option<(u64, u64)> {
        self.usage
            .as_ref()
            .map(|u| (u.prompt_tokens, u.completion_tokens))
    }
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamChunkRaw {
    pub choices: Vec<OpenAiStreamChoiceRaw>,
    pub usage: Option<OpenAiStreamUsageRaw>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamChoiceRaw {
    pub delta: OpenAiStreamDeltaRaw,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamDeltaRaw {
    pub content: Option<String>,
    /// Reasoning model output (e.g. GLM-4.7-Flash, DeepSeek V4 thinking mode).
    /// Forwarded as regular content so the skill chain can process it.
    #[serde(default)]
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<OpenAiStreamToolCallRaw>>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamToolCallRaw {
    pub index: Option<u64>,
    pub id: Option<String>,
    pub function: Option<OpenAiStreamFunctionRaw>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamFunctionRaw {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamUsageRaw {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub prompt_tokens_details: Option<OpenAiStreamUsageDetailsRaw>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAiStreamUsageDetailsRaw {
    pub cached_tokens: Option<u64>,
}

pub fn parse_openai_stream(value: &OpenAiStreamChunkRaw) -> Vec<LlmStreamChunk> {
    let choice = match value.choices.first() {
        Some(c) => c,
        None => return Vec::new(),
    };

    // Newer GLM flash models (4.5+, 4.7+) return output in reasoning_content
    // while leaving content as Some("").  Fall back to reasoning_content when
    // content is None OR empty.
    let content = {
        let c = choice.delta.content.as_deref();
        if c.is_some_and(|s| !s.is_empty()) {
            choice.delta.content.clone()
        } else {
            choice.delta.reasoning_content.clone()
        }
    };
    let finish_reason = choice.finish_reason.as_deref().map(FinishReason::from);

    let usage = value.usage.as_ref().map(|u| LlmUsage {
        prompt_tokens: u.prompt_tokens,
        completion_tokens: u.completion_tokens,
        total_tokens: u.total_tokens,
        cached_tokens: u
            .prompt_tokens_details
            .as_ref()
            .and_then(|d| d.cached_tokens),
    });

    let tool_call_deltas: Vec<ToolCallDelta> = choice
        .delta
        .tool_calls
        .as_ref()
        .map(|arr| {
            arr.iter()
                .filter_map(|tc_delta| {
                    let index = tc_delta.index.map(|v| v as u32);
                    let id = tc_delta.id.clone();
                    let name = tc_delta.function.as_ref().and_then(|f| f.name.clone());
                    let arguments = tc_delta.function.as_ref().and_then(|f| f.arguments.clone());
                    if id.is_some() || name.is_some() || arguments.is_some() || index.is_some() {
                        Some(ToolCallDelta {
                            id,
                            name,
                            arguments,
                            index,
                            integrity: None,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut chunks = Vec::new();

    if !tool_call_deltas.is_empty() {
        for tc_delta in tool_call_deltas {
            chunks.push(LlmStreamChunk {
                content: None,
                tool_call: Some(tc_delta),
                finish_reason: None,
                usage: None,
            });
        }
    }

    let has_non_tool_content = content.is_some() || finish_reason.is_some() || usage.is_some();
    if has_non_tool_content {
        chunks.push(LlmStreamChunk {
            content,
            tool_call: None,
            finish_reason,
            usage,
        });
    }

    chunks
}
