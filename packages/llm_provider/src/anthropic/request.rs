use serde::{Deserialize, Serialize};

use tracing::warn;

use super::super::{
    ContentBlock, LlmMessage, MessageRole, ROLE_ASSISTANT, ROLE_USER, SOURCE_TYPE_BASE64,
    ToolDefinition,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicToolChoice {
    #[serde(rename = "type")]
    pub choice_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
    #[serde(rename = "image")]
    Image { source: AnthropicImageSource },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AnthropicMessageContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicMessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub max_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AnthropicToolChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

pub fn convert_messages(messages: &[LlmMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system_parts: Vec<String> = Vec::new();
    let anthropic_messages: Vec<AnthropicMessage> = messages
        .iter()
        .filter_map(|m| {
            if m.role == MessageRole::System {
                if let Some(ref content) = m.content {
                    system_parts.push(content.clone());
                }
                None
            } else if m.role == MessageRole::Tool {
                let blocks = vec![AnthropicContentBlock::ToolResult {
                    tool_use_id: m.tool_call_id.as_deref().unwrap_or("").to_string(),
                    content: m.content.as_deref().unwrap_or("").to_string(),
                }];
                Some(AnthropicMessage {
                    role: ROLE_USER.into(),
                    content: AnthropicMessageContent::Blocks(blocks),
                })
            } else if m.role == MessageRole::Assistant {
                if let Some(ref tool_calls) = m.tool_calls {
                    let mut blocks: Vec<AnthropicContentBlock> = Vec::new();
                    if let Some(ref content) = m.content
                        && !content.is_empty() {
                            blocks.push(AnthropicContentBlock::Text {
                                text: content.clone(),
                            });
                        }
                    for tc in tool_calls {
                        let input: serde_json::Value =
                            serde_json::from_str(&tc.arguments).unwrap_or_else(|e| { warn!(error=%e, args=%tc.arguments, "Anthropic tool call args parse failed"); serde_json::Value::Null });
                        blocks.push(AnthropicContentBlock::ToolUse {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            input,
                        });
                    }
                    Some(AnthropicMessage {
                        role: ROLE_ASSISTANT.into(),
                        content: AnthropicMessageContent::Blocks(blocks),
                    })
                } else {
                    Some(AnthropicMessage {
                        role: ROLE_ASSISTANT.into(),
                        content: AnthropicMessageContent::Text(
                            m.content.as_deref().unwrap_or("").to_string(),
                        ),
                    })
                }
            } else {
                if let Some(ref blocks) = m.content_blocks {
                    let anthropic_blocks: Vec<AnthropicContentBlock> = blocks
                        .iter()
                        .filter_map(|b| match b {
                            ContentBlock::Text { text } => {
                                Some(AnthropicContentBlock::Text { text: text.clone() })
                            },
                            ContentBlock::ImageBase64 { media_type, data } => {
                                Some(AnthropicContentBlock::Image {
                                    source: AnthropicImageSource {
                                        source_type: SOURCE_TYPE_BASE64.into(),
                                        media_type: media_type.clone(),
                                        data: data.clone(),
                                    },
                                })
                            },
                            ContentBlock::ImageUrl { .. } => None,
                        })
                        .collect();
                    Some(AnthropicMessage {
                        role: m.role.to_string(),
                        content: AnthropicMessageContent::Blocks(anthropic_blocks),
                    })
                } else if let Some(ref images) = m.images {
                    let mut blocks: Vec<AnthropicContentBlock> = Vec::new();
                    if let Some(ref text) = m.content {
                        blocks.push(AnthropicContentBlock::Text { text: text.clone() });
                    }
                    for img in images {
                        blocks.push(AnthropicContentBlock::Image {
                            source: AnthropicImageSource {
                                source_type: "base64".to_string(),
                                media_type: img.media_type.clone(),
                                data: img.to_base64(),
                            },
                        });
                    }
                    Some(AnthropicMessage {
                        role: m.role.to_string(),
                        content: AnthropicMessageContent::Blocks(blocks),
                    })
                } else {
                    Some(AnthropicMessage {
                        role: m.role.to_string(),
                        content: AnthropicMessageContent::Text(
                            m.content.as_deref().unwrap_or("").to_string(),
                        ),
                    })
                }
            }
        })
        .collect();
    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n\n"))
    };
    (system, anthropic_messages)
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<AnthropicTool> {
    tools
        .iter()
        .map(|t| AnthropicTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.parameters.clone(),
            strict: if t.strict.unwrap_or(false) {
                Some(true)
            } else {
                None
            },
        })
        .collect()
}
