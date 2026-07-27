use serde::{Deserialize, Serialize};

use super::super::{ContentBlock, LlmMessage, TYPE_FUNCTION, ToolDefinition};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// ZhiPu GLM-5+ "thinking" (reasoning) mode control.
    /// When set to `{"type":"disabled"}`, the model skips the reasoning
    /// phase and returns content directly — critical for tool-use prompts
    /// where reasoning tokens exhaust the max_tokens budget before the
    /// model can emit a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAiMessageToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OpenAiContent {
    Text(String),
    Parts(Vec<OpenAiContentPart>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAiContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAiImageUrl },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiImageUrl {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiMessageToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: super::response::OpenAiFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiTool {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: OpenAiFunctionDef,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAiFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl OpenAiMessage {
    pub fn from_llm_message(m: LlmMessage) -> Self {
        let tool_calls = m.tool_calls.map(|tc| {
            tc.into_iter()
                .map(|t| OpenAiMessageToolCall {
                    id: t.id,
                    r#type: TYPE_FUNCTION.into(),
                    function: super::response::OpenAiFunction {
                        name: t.name,
                        arguments: t.arguments,
                    },
                })
                .collect::<Vec<_>>()
        });

        let content = if let Some(blocks) = &m.content_blocks {
            let parts: Vec<OpenAiContentPart> = blocks
                .iter()
                .map(|b| match b {
                    ContentBlock::Text { text } => OpenAiContentPart::Text { text: text.clone() },
                    ContentBlock::ImageUrl { image_url } => OpenAiContentPart::ImageUrl {
                        image_url: OpenAiImageUrl {
                            url: image_url.url.clone(),
                        },
                    },
                    ContentBlock::ImageBase64 { media_type, data } => OpenAiContentPart::ImageUrl {
                        image_url: OpenAiImageUrl {
                            url: format!("data:{};base64,{}", media_type, data),
                        },
                    },
                })
                .collect();
            Some(OpenAiContent::Parts(parts))
        } else if let Some(images) = &m.images {
            let mut parts: Vec<OpenAiContentPart> = Vec::new();
            if let Some(text) = &m.content {
                parts.push(OpenAiContentPart::Text { text: text.clone() });
            }
            for img in images {
                parts.push(OpenAiContentPart::ImageUrl {
                    image_url: OpenAiImageUrl {
                        url: format!("data:{};base64,{}", img.media_type, img.to_base64()),
                    },
                });
            }
            Some(OpenAiContent::Parts(parts))
        } else {
            // For assistant messages with tool_calls but no content,
            // GLM and some other providers require an explicit empty string
            // instead of a missing field. Omitting content entirely can cause
            // the model to return empty responses in subsequent turns.
            Some(OpenAiContent::Text(m.content.clone().unwrap_or_default()))
        };

        OpenAiMessage {
            role: m.role.to_string(),
            content,
            tool_call_id: m.tool_call_id,
            tool_calls,
        }
    }
}

pub fn build_tools(tools: &Option<Vec<ToolDefinition>>) -> Option<Vec<OpenAiTool>> {
    tools.as_ref().map(|defs| {
        defs.iter()
            .map(|td| {
                let strict = if td.strict.unwrap_or(false) {
                    Some(true)
                } else {
                    None
                };
                OpenAiTool {
                    r#type: TYPE_FUNCTION.into(),
                    function: OpenAiFunctionDef {
                        name: td.name.clone(),
                        description: td.description.clone(),
                        parameters: td.parameters.clone(),
                        strict,
                    },
                }
            })
            .collect()
    })
}
