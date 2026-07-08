// MCP tool implementation

use serde_json::Value;
use std::collections::HashMap;

use tracing::{debug, error, info};

use _domain_skills_permissions::ToolCapability;
use _state_sync::{Agent, McpToolInfo};

/// Validate that all required parameters are present and non-empty.
/// Returns `None` if valid, or a failure `McpToolResult` if any required param is missing/empty.
pub fn validate_required_params(
    parameters: &Value,
    required: &[&str],
    tool_name: &str,
) -> Option<McpToolResult> {
    let empty_params: Vec<&str> = required
        .iter()
        .filter(|&&key| {
            parameters
                .get(key)
                .map(|v| match v {
                    Value::String(s) => s.is_empty(),
                    Value::Null => true,
                    _ => false,
                })
                .unwrap_or(true)
        })
        .copied()
        .collect();

    if empty_params.is_empty() {
        None
    } else {
        Some(McpToolResult::failure(format!(
            "Missing required parameter(s) for {}: {}",
            tool_name,
            empty_params.join(", ")
        )))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SnapshotPolicy {
    Never,
    Always,
    Conditional,
}

/// MCP tool invocation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolResult {
    pub success: bool,
    pub data: Value,
    pub error: Vec<String>,
    #[serde(default)]
    pub model_name: Option<String>,
    #[serde(default)]
    pub token_usage: Option<(u32, u32)>,
}

impl McpToolResult {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data,
            error: Vec::new(),
            model_name: None,
            token_usage: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: Value::Null,
            error: vec![error],
            model_name: None,
            token_usage: None,
        }
    }

    pub fn success_struct<T: serde::Serialize>(data: &T) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).unwrap_or(Value::Null),
            error: Vec::new(),
            model_name: None,
            token_usage: None,
        }
    }

    pub fn success_text(text: String) -> Self {
        Self {
            success: true,
            data: Value::String(text),
            error: Vec::new(),
            model_name: None,
            token_usage: None,
        }
    }

    pub fn success_with_usage(
        text: String,
        model_name: Option<String>,
        token_usage: Option<(u32, u32)>,
    ) -> Self {
        Self {
            success: true,
            data: Value::String(text),
            error: Vec::new(),
            model_name,
            token_usage,
        }
    }

    pub fn failure_text(error: String) -> Self {
        Self {
            success: false,
            data: Value::Null,
            error: vec![error],
            model_name: None,
            token_usage: None,
        }
    }

    pub fn failure_lines(errors: Vec<String>) -> Self {
        Self {
            success: false,
            data: Value::Null,
            error: errors,
            model_name: None,
            token_usage: None,
        }
    }
}

/// Natural language formatter
pub struct NaturalLanguageFormatter;

impl NaturalLanguageFormatter {
    /// Format key-value pairs as natural language
    pub fn format_key_value_pairs(pairs: &[(&str, &str)]) -> String {
        pairs
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format a list as natural language
    pub fn format_list(title: &str, items: &[String]) -> String {
        if items.is_empty() {
            return format!("{}: (none)", title);
        }
        let items_text = items
            .iter()
            .map(|item| format!("  - {}", item))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}:\n{}", title, items_text)
    }

    /// Format execution result
    pub fn format_execution_result(
        title: &str,
        fields: &[(&str, &str)],
        sections: &[(&str, &str)],
    ) -> String {
        let mut result = String::new();

        result.push_str(&format!("{}\n\n", title));

        for (key, value) in fields {
            result.push_str(&format!("{}: {}\n", key, value));
        }

        if !sections.is_empty() {
            result.push('\n');
            for (section_title, content) in sections {
                result.push_str(&format!("\n{}:\n{}\n", section_title, content));
            }
        }

        result
    }

    /// Format error message
    pub fn format_error(error_type: &str, details: &str, suggestions: &[&str]) -> String {
        let mut result = format!("{}\n\nError: {}\n", error_type, details);

        if !suggestions.is_empty() {
            result.push_str("\nSuggestions:\n");
            for suggestion in suggestions {
                result.push_str(&format!("  - {}\n", suggestion));
            }
        }

        result
    }
}

/// MCP tool invoker trait
#[async_trait::async_trait]
pub trait McpToolInvoker: Send + Sync {
    async fn invoke(&self, tool_name: &str, parameters: Value) -> McpToolResult;

    async fn get_tools(&self) -> Vec<McpToolInfo>;

    fn get_tool_capabilities(&self) -> HashMap<String, ToolCapability> {
        HashMap::new()
    }

    fn snapshot_policy(&self) -> SnapshotPolicy {
        SnapshotPolicy::Always
    }

    async fn verify(&self, _tool_name: &str, _parameters: &Value) -> bool {
        false
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

/// MCP tool registry
pub struct McpToolRegistry {
    /// Agent type
    agent_type: Agent,
    /// Tool registry
    tools: HashMap<String, Box<dyn McpToolInvoker + Send + Sync>>,
}

impl McpToolRegistry {
    /// Create a new MCP tool registry
    pub fn new(agent_type: Agent) -> Self {
        Self {
            agent_type,
            tools: HashMap::new(),
        }
    }

    /// Register tool
    pub fn register_tool<T: McpToolInvoker + 'static>(&mut self, tool_name: String, invoker: T) {
        info!("registered MCP tool: {} ({})", tool_name, self.agent_type);
        self.tools.insert(tool_name, Box::new(invoker));
    }

    /// Invoke tool
    pub async fn invoke(&self, tool_name: &str, parameters: Value) -> McpToolResult {
        debug!("invoking MCP tool: {} ({})", tool_name, self.agent_type);

        if let Some(invoker) = self.tools.get(tool_name) {
            invoker.invoke(tool_name, parameters).await
        } else {
            error!("tool not found: {}", tool_name);
            McpToolResult::failure(format!("Tool not found: {}", tool_name))
        }
    }

    /// Get all tools
    pub async fn get_tools(&self) -> Vec<McpToolInfo> {
        let mut all = Vec::new();
        for invoker in self.tools.values() {
            all.extend(invoker.get_tools().await);
        }
        all
    }
}
