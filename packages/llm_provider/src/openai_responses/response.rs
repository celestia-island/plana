use serde::Deserialize;

use super::{
    super::ToolCall,
    sse_events::{ITEM_TYPE_FUNCTION_CALL, ITEM_TYPE_MESSAGE},
};

#[derive(Deserialize, Debug)]
pub struct ResponsesApiResponse {
    pub status: Option<String>,
    pub output: Vec<serde_json::Value>,
    pub usage: Option<ResponsesUsage>,
}

#[derive(Deserialize, Debug)]
pub struct ResponsesUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl ResponsesApiResponse {
    pub fn extract_text_and_tool_calls(&self) -> (String, Vec<ToolCall>) {
        let mut text = String::new();
        let mut tool_calls = Vec::new();

        for item in &self.output {
            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");

            match item_type {
                ITEM_TYPE_MESSAGE => {
                    if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                        for part in content {
                            if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                if !text.is_empty() {
                                    text.push('\n');
                                }
                                text.push_str(t);
                            }
                        }
                    }
                },
                ITEM_TYPE_FUNCTION_CALL => {
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
                    tool_calls.push(ToolCall {
                        id: call_id,
                        name,
                        arguments,
                        integrity: None,
                    });
                },
                _ => {},
            }
        }
        (text, tool_calls)
    }
}
