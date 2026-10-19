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
    // Scepter (entelecheia#571) chunks the system prompt into multiple
    // consecutive system messages; the whole first run of them is folded into
    // the first message's content with the same "\n\n" join the anthropic
    // adapter applies to its `system_parts` (see `anthropic/request.rs
    // convert_messages`). A system message that reappears after the run is NOT
    // re-absorbed: Gemini `contents` only carries user/model roles, so it is
    // dropped, exactly as before the run-merge existed.
    let system_run_end = messages
        .iter()
        .position(|m| m.role == MessageRole::System)
        .map_or(0, |start| {
            start
                + messages[start..]
                    .iter()
                    .position(|m| m.role != MessageRole::System)
                    .unwrap_or(messages.len())
        });
    let contents: Vec<GeminiRequestContent> = messages
        .iter()
        .enumerate()
        .filter_map(|(idx, m)| match m.role {
            MessageRole::System => {
                if system.is_none() {
                    system = m.content.clone();
                } else if idx < system_run_end
                    && let (Some(sys), Some(content)) = (system.as_mut(), m.content.as_deref())
                {
                    sys.push_str("\n\n");
                    sys.push_str(content);
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

#[cfg(test)]
mod tests {
    use super::*;

    // Scepter (entelecheia#571) chunks the system prompt into multiple
    // CONSECUTIVE system messages. The whole leading run is folded into the
    // first system message's content, joined with "\n\n" in order — the same
    // join the anthropic adapter applies to its `system_parts`
    // (anthropic/request.rs `convert_messages`). Non-system messages pass
    // through untouched.
    #[test]
    fn merges_consecutive_system_run_with_blank_line_join() {
        let messages = vec![
            LlmMessage::system("identity"),
            LlmMessage::system("contract"),
            LlmMessage::system("persona"),
            LlmMessage::user("hello"),
        ];
        let (system, contents) = convert_messages(&messages);
        assert_eq!(system.as_deref(), Some("identity\n\ncontract\n\npersona"));
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].role, ROLE_USER);
        match contents[0].parts.as_slice() {
            [GeminiRequestPart::Text { text }] => assert_eq!(text, "hello"),
            other => panic!("expected a single text part, got {other:?}"),
        }
    }

    // A single system message stays byte-identical: no separator is glued on
    // when there is nothing to merge.
    #[test]
    fn single_system_message_stays_byte_identical() {
        let messages = vec![LlmMessage::system("identity"), LlmMessage::user("hello")];
        let (system, contents) = convert_messages(&messages);
        assert_eq!(system.as_deref(), Some("identity"));
        assert_eq!(contents.len(), 1);
    }

    // Only the run of consecutive system messages is merged. A system message
    // sandwiched after a user turn is NOT re-absorbed into the system
    // instruction — Gemini `contents` carries user/model roles only, so the
    // late run is dropped, the same fate it had before the run-merge existed.
    // (The anthropic adapter differs here on purpose: its `system` field
    // collects every system part regardless of position.)
    #[test]
    fn system_after_user_is_not_re_absorbed() {
        let messages = vec![
            LlmMessage::system("identity"),
            LlmMessage::user("hello"),
            LlmMessage::system("late"),
            LlmMessage::system("later"),
        ];
        let (system, contents) = convert_messages(&messages);
        assert_eq!(system.as_deref(), Some("identity"));
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].role, ROLE_USER);
        match contents[0].parts.as_slice() {
            [GeminiRequestPart::Text { text }] => assert_eq!(text, "hello"),
            other => panic!("expected a single text part, got {other:?}"),
        }
    }

    // No system messages at all → no system instruction, contents unchanged.
    #[test]
    fn no_system_message_leaves_instruction_empty() {
        let messages = vec![LlmMessage::user("hello"), LlmMessage::assistant("hi")];
        let (system, contents) = convert_messages(&messages);
        assert_eq!(system, None);
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0].role, ROLE_USER);
        assert_eq!(contents[1].role, ROLE_MODEL);
    }
}
