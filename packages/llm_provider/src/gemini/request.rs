use serde::{Deserialize, Serialize};

use tracing::warn;

use super::super::{ContentBlock, LlmMessage, MessageRole, ROLE_MODEL, ROLE_USER, ToolDefinition};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiRequestContent {
    pub role: String,
    pub parts: Vec<GeminiRequestPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum GeminiRequestPart {
    Text {
        text: String,
    },
    FunctionCall {
        function_call: GeminiFunctionCallDef,
    },
    FunctionResponse {
        function_response: GeminiFunctionResponseDef,
    },
    InlineData {
        inline_data: GeminiInlineData,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionCallDef {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionResponseDef {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiSystemInstruction {
    pub parts: Vec<GeminiRequestPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolDef {
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolConfig {
    pub function_calling_config: GeminiFunctionCallingConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionCallingConfig {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_function_names: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRequest {
    pub contents: Vec<GeminiRequestContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiSystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<GeminiToolConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
}

pub fn convert_messages(messages: &[LlmMessage]) -> (Option<String>, Vec<GeminiRequestContent>) {
    let mut system = None;
    let contents: Vec<GeminiRequestContent> = messages
        .iter()
        .filter_map(|m| match m.role {
            MessageRole::System => {
                if system.is_none() {
                    system = m.content.clone();
                }
                None
            },
            MessageRole::Assistant => {
                let mut parts: Vec<GeminiRequestPart> = Vec::new();
                if let Some(ref text) = m.content
                    && !text.is_empty() {
                        parts.push(GeminiRequestPart::Text { text: text.clone() });
                    }
                if let Some(ref tool_calls) = m.tool_calls {
                    for tc in tool_calls {
                        let input: serde_json::Value =
                            serde_json::from_str(&tc.arguments).unwrap_or_else(|e| { warn!(error=%e, args=%tc.arguments, "Gemini tool call args parse failed"); serde_json::Value::Null });
                        parts.push(GeminiRequestPart::FunctionCall {
                            function_call: GeminiFunctionCallDef {
                                name: tc.name.clone(),
                                args: input,
                            },
                        });
                    }
                }
                if parts.is_empty() {
                    None
                } else {
                    Some(GeminiRequestContent {
                        role: ROLE_MODEL.into(),
                        parts,
                    })
                }
            },
            MessageRole::Tool => {
                let fn_name = m.tool_call_id.as_deref().unwrap_or("unknown").to_string();
                let resp = GeminiFunctionResponseDef {
                    name: fn_name,
                    response: serde_json::from_str::<serde_json::Value>(
                        m.content.as_deref().unwrap_or("{}"),
                    )
                    .unwrap_or_default(),
                };
                Some(GeminiRequestContent {
                    role: ROLE_USER.into(),
                    parts: vec![GeminiRequestPart::FunctionResponse {
                        function_response: resp,
                    }],
                })
            },
            MessageRole::User => {
                if let Some(ref blocks) = m.content_blocks {
                    let mut parts: Vec<GeminiRequestPart> = blocks
                        .iter()
                        .filter_map(|b| match b {
                            ContentBlock::Text { text } => {
                                Some(GeminiRequestPart::Text { text: text.clone() })
                            },
                            ContentBlock::ImageBase64 { media_type, data } => {
                                Some(GeminiRequestPart::InlineData {
                                    inline_data: GeminiInlineData {
                                        mime_type: media_type.clone(),
                                        data: data.clone(),
                                    },
                                })
                            },
                            ContentBlock::ImageUrl { .. } => None,
                        })
                        .collect();
                    if parts.is_empty() {
                        parts.push(GeminiRequestPart::Text {
                            text: String::new(),
                        });
                    }
                    Some(GeminiRequestContent {
                        role: ROLE_USER.into(),
                        parts,
                    })
                } else if let Some(ref images) = m.images {
                    let mut parts: Vec<GeminiRequestPart> = Vec::new();
                    if let Some(ref text) = m.content {
                        parts.push(GeminiRequestPart::Text { text: text.clone() });
                    }
                    for img in images {
                        parts.push(GeminiRequestPart::InlineData {
                            inline_data: GeminiInlineData {
                                mime_type: img.media_type.clone(),
                                data: img.to_base64(),
                            },
                        });
                    }
                    Some(GeminiRequestContent {
                        role: ROLE_USER.into(),
                        parts,
                    })
                } else {
                    Some(GeminiRequestContent {
                        role: ROLE_USER.into(),
                        parts: vec![GeminiRequestPart::Text {
                            text: m.content.as_deref().unwrap_or("").to_string(),
                        }],
                    })
                }
            },
        })
        .collect();
    (system, contents)
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<GeminiToolDef> {
    let declarations: Vec<GeminiFunctionDeclaration> = tools
        .iter()
        .map(|td| GeminiFunctionDeclaration {
            name: td.name.clone(),
            description: td.description.clone(),
            parameters: td.parameters.clone(),
        })
        .collect();
    vec![GeminiToolDef {
        function_declarations: declarations,
    }]
}
