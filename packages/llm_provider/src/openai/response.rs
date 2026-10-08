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
    // 2026-10-08: reasoning_content rides its OWN field — the parser no
    // longer guesses whether it is thinking or output (that is per-model
    // registry knowledge: GLM-flash emits output there, DeepSeek emits
    // thinking). Consumers fold `thinking` into content for the GLM-flash
    // class; nothing is lost and thinking-capable models keep the two
    // streams distinct.
    let content = choice.delta.content.clone().filter(|s| !s.is_empty());
    let thinking = choice
        .delta
        .reasoning_content
        .clone()
        .filter(|s| !s.is_empty());
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
                thinking: None,
                tool_call: Some(tc_delta),
                finish_reason: None,
                usage: None,
            });
        }
    }

    let has_non_tool_content =
        content.is_some() || thinking.is_some() || finish_reason.is_some() || usage.is_some();
    if has_non_tool_content {
        chunks.push(LlmStreamChunk {
            content,
            thinking,
            tool_call: None,
            finish_reason,
            usage,
        });
    }

    chunks
}

#[cfg(test)]
mod reasoning_split_tests {
    use super::*;

    fn delta(content: Option<&str>, reasoning: Option<&str>) -> OpenAiStreamChunkRaw {
        serde_json::from_value(serde_json::json!({
            "choices": [{
                "delta": {
                    "content": content,
                    "reasoning_content": reasoning,
                },
                "finish_reason": null,
            }],
        }))
        .expect("raw chunk parses")
    }

    /// The 2026-10-08 split: reasoning deltas ride their OWN field — the
    /// parser never guesses model semantics (GLM-flash output vs
    /// DeepSeek thinking). Consumers route by the registry's can_reason.
    #[test]
    fn reasoning_delta_stays_separate_from_content() {
        let out = parse_openai_stream(&delta(Some("answer"), Some("pondering")));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content.as_deref(), Some("answer"));
        assert_eq!(out[0].thinking.as_deref(), Some("pondering"));
    }

    #[test]
    fn reasoning_only_delta_has_empty_content() {
        let out = parse_openai_stream(&delta(None, Some("pure thought")));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content, None);
        assert_eq!(out[0].thinking.as_deref(), Some("pure thought"));
    }

    #[test]
    fn content_only_delta_has_no_thinking() {
        let out = parse_openai_stream(&delta(Some("plain"), None));
        assert_eq!(out[0].content.as_deref(), Some("plain"));
        assert_eq!(out[0].thinking, None);
    }
}
