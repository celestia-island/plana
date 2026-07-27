use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub call_mode: super::McpToolCallMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
