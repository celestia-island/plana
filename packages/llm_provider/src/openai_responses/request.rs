use serde::Serialize;
use serde_json::Value;

use super::super::{ContentBlock, LlmMessage, MessageRole, ProviderConfig, ToolDefinition};

const TYPE_FUNCTION_CALL: &str = "function_call";
const TYPE_FUNCTION_CALL_OUTPUT: &str = "function_call_output";
const TYPE_FUNCTION: &str = "function";
const KEY_TYPE: &str = "type";
const KEY_ADDITIONAL_PROPERTIES: &str = "additionalProperties";

#[derive(Serialize, Debug)]
pub struct ResponsesRequest {
    pub model: String,
    pub input: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Serialize)]
struct RoleContentItem {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RoleMultimodalItem {
    role: String,
    content: Vec<Value>,
}

#[derive(Serialize)]
struct InputTextPart {
    #[serde(rename = "type")]
    part_type: &'static str,
    text: String,
}

#[derive(Serialize)]
struct InputImageUrlPart {
    #[serde(rename = "type")]
    part_type: &'static str,
    image_url: String,
}

#[derive(Serialize)]
struct FunctionCallItem {
    #[serde(rename = "type")]
    item_type: &'static str,
    call_id: String,
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct FunctionCallOutputItem {
    #[serde(rename = "type")]
    item_type: &'static str,
    call_id: String,
    output: String,
}

#[derive(Serialize)]
struct ToolItem {
    #[serde(rename = "type")]
    item_type: &'static str,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Value>,
}

fn content_block_to_part(block: &ContentBlock) -> Option<Value> {
    match block {
        ContentBlock::Text { text } => Some(
            serde_json::to_value(InputTextPart {
                part_type: "input_text",
                text: text.clone(),
            })
            .unwrap_or(Value::Null),
        ),
        ContentBlock::ImageUrl { image_url } => Some(
            serde_json::to_value(InputImageUrlPart {
                part_type: "input_image",
                image_url: image_url.url.clone(),
            })
            .unwrap_or(Value::Null),
        ),
        ContentBlock::ImageBase64 { media_type, data } => Some(
            serde_json::to_value(InputImageUrlPart {
                part_type: "input_image",
                image_url: format!("data:{};base64,{}", media_type, data),
            })
            .unwrap_or(Value::Null),
        ),
    }
}

pub fn convert_input_items(messages: Vec<LlmMessage>) -> Vec<Value> {
    let mut items: Vec<Value> = Vec::new();
    for msg in messages {
        match msg.role {
            MessageRole::System | MessageRole::User => {
                if let Some(ref blocks) = msg.content_blocks {
                    let content: Vec<Value> =
                        blocks.iter().filter_map(content_block_to_part).collect();
                    if !content.is_empty() {
                        let item = RoleMultimodalItem {
                            role: msg.role.to_string(),
                            content,
                        };
                        items.push(serde_json::to_value(item).unwrap_or(Value::Null));
                    }
                } else {
                    let item = RoleContentItem {
                        role: msg.role.to_string(),
                        content: msg.content.unwrap_or_default(),
                    };
                    items.push(serde_json::to_value(item).unwrap_or(Value::Null));
                }
            }
            MessageRole::Assistant => {
                if let Some(tool_calls) = &msg.tool_calls {
                    for tc in tool_calls {
                        let item = FunctionCallItem {
                            item_type: TYPE_FUNCTION_CALL,
                            call_id: tc.id.clone(),
                            name: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                        };
                        items.push(serde_json::to_value(item).unwrap_or(Value::Null));
                    }
                } else {
                    let item = RoleContentItem {
                        role: msg.role.to_string(),
                        content: msg.content.unwrap_or_default(),
                    };
                    items.push(serde_json::to_value(item).unwrap_or(Value::Null));
                }
            }
            MessageRole::Tool => {
                if let Some(tool_call_id) = &msg.tool_call_id {
                    let item = FunctionCallOutputItem {
                        item_type: TYPE_FUNCTION_CALL_OUTPUT,
                        call_id: tool_call_id.clone(),
                        output: msg.content.unwrap_or_default(),
                    };
                    items.push(serde_json::to_value(item).unwrap_or(Value::Null));
                }
            }
        }
    }
    items
}

pub fn ensure_additional_properties_false(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        if obj.get(KEY_TYPE).and_then(|t| t.as_str()) == Some("object") {
            obj.entry(KEY_ADDITIONAL_PROPERTIES.to_string())
                .or_insert(Value::Bool(false));
            for (_key, v) in obj.iter_mut() {
                if v.is_object() || v.is_array() {
                    ensure_additional_properties_false(v);
                }
            }
        } else if let Some(arr) = obj.get_mut("anyOf").and_then(|v| v.as_array_mut()) {
            for item in arr.iter_mut() {
                ensure_additional_properties_false(item);
            }
        }
    } else if let Some(arr) = value.as_array_mut() {
        for item in arr.iter_mut() {
            ensure_additional_properties_false(item);
        }
    }
}

pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<Value> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for t in tools {
        if seen.contains(&t.name) {
            continue;
        }
        seen.insert(t.name.clone());
        let mut params = t.parameters.clone();
        if !params.is_null() {
            ensure_additional_properties_false(&mut params);
        }
        let has_params = !params.is_null();
        if has_params {
            if let Some(obj) = params.as_object_mut()
                && obj
                    .get(KEY_TYPE)
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .is_empty()
            {
                obj.insert(KEY_TYPE.to_string(), Value::String("object".into()));
            }
            ensure_additional_properties_false(&mut params);
        }
        let item = ToolItem {
            item_type: TYPE_FUNCTION,
            name: t.name.clone(),
            description: t.description.clone(),
            parameters: if has_params { Some(params) } else { None },
        };
        result.push(serde_json::to_value(item).unwrap_or(Value::Null));
    }
    result
}

pub fn build_request(
    endpoint: &str,
    client: &reqwest::Client,
    config: &ProviderConfig,
    req_body: &ResponsesRequest,
) -> reqwest::RequestBuilder {
    config
        .build_authenticated_post(client, endpoint)
        .json(req_body)
}
