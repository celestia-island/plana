pub mod schema;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::ModelTier;
use _core::ToolDefinition;

fn default_param_type() -> String {
    schema::default_param_type()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolLocation {
    #[default]
    Scepter,
    Cosmos,
}

pub type SkillLocation = ToolLocation;

pub use _core::ToolCallMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolVisibility {
    #[default]
    Always,
    SkillWhitelist,
    McpExplore,
}

impl ToolVisibility {
    pub fn is_visible_with(
        &self,
        allow_always: bool,
        allow_skill_whitelist: bool,
        allow_mcp_explore: bool,
    ) -> bool {
        match self {
            Self::Always => allow_always,
            Self::SkillWhitelist => allow_skill_whitelist,
            Self::McpExplore => allow_mcp_explore,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolMaturity {
    #[default]
    Stable,
    Experimental,
    Stub,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub agent_type: super::agent::Agent,
    pub parameters: ToolParameters,
    #[serde(default)]
    pub tier: Option<ModelTier>,
    #[serde(default)]
    pub location: ToolLocation,
    #[serde(default)]
    pub call_mode: ToolCallMode,
    #[serde(default)]
    pub visibility: ToolVisibility,
    #[serde(default)]
    pub is_async: bool,
    #[serde(default)]
    pub maturity: ToolMaturity,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type", default = "default_param_type")]
    pub param_type: String,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub separate_call_keys: Vec<String>,
}

impl ToolParameters {
    pub fn new(
        param_type: &str,
        required: Vec<String>,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            param_type: param_type.to_string(),
            required,
            properties,
            separate_call_keys: Vec::new(),
        }
    }
}

impl ToolInfo {
    pub fn simple(
        name: &str,
        description: &str,
        agent_type: super::agent::Agent,
        required: Vec<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            agent_type,
            parameters: ToolParameters {
                param_type: "object".to_string(),
                required: required.iter().map(|s| s.to_string()).collect(),
                properties: HashMap::new(),
                separate_call_keys: Vec::new(),
            },
            tier: None,
            location: ToolLocation::default(),
            call_mode: ToolCallMode::default(),
            visibility: ToolVisibility::default(),
            is_async: false,
            maturity: ToolMaturity::default(),
        }
    }

    pub fn with_params(mut self, parameters: ToolParameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn with_location(mut self, location: ToolLocation) -> Self {
        self.location = location;
        self
    }

    pub fn with_call_mode(mut self, mode: ToolCallMode) -> Self {
        self.call_mode = mode;
        self
    }

    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = Some(tier);
        self
    }

    pub fn with_visibility(mut self, visibility: ToolVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn async_tool(mut self) -> Self {
        self.is_async = true;
        self
    }

    pub fn with_maturity(mut self, maturity: ToolMaturity) -> Self {
        self.maturity = maturity;
        self
    }

    pub fn to_tool_definition(&self) -> ToolDefinition {
        let properties = self.parameters.properties.clone();
        let separate_keys = &self.parameters.separate_call_keys;
        let has_separate = !separate_keys.is_empty();

        let mut params_map = serde_json::Map::new();
        params_map.insert(
            "type".to_string(),
            serde_json::Value::String(self.parameters.param_type.clone()),
        );
        params_map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties.into_iter().collect()),
        );
        let mut params = serde_json::Value::Object(params_map);

        if has_separate {
            let required: Vec<serde_json::Value> = self
                .parameters
                .required
                .iter()
                .map(|r| serde_json::Value::String(r.clone()))
                .collect();
            if !required.is_empty()
                && let Some(obj) = params.as_object_mut()
            {
                obj.insert("required".to_string(), serde_json::Value::Array(required));
            }
            if let Some(obj) = params.as_object_mut() {
                obj.insert(
                    "additionalProperties".to_string(),
                    serde_json::Value::Bool(false),
                );
            }
        } else {
            schema::normalize_schema_for_strict(&mut params);
        }

        let mut description = self.description.clone();
        if has_separate {
            let fields: Vec<String> = separate_keys
                .iter()
                .map(|k| format!("{}.{}(\"...\")", self.name, k))
                .collect();
            description.push_str(&format!(
                "\n\nParameters [{}] can be provided either in the JSON arguments or via a separate follow-up call using the syntax tool_name.param_name(\"value\").",
                fields.join(", ")
            ));
        }

        ToolDefinition {
            name: self.name.clone(),
            description,
            parameters: params,
            call_mode: self.call_mode,
            strict: Some(!has_separate),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub agent_type: super::agent::Agent,
    pub parameters: serde_json::Value,
    pub call_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub call_id: Uuid,
    pub result: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: HashMap<String, String>,
    pub agent_type: super::agent::Agent,
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub tier: Option<ModelTier>,
    #[serde(default)]
    pub location: SkillLocation,
}

impl Default for SkillInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: HashMap::new(),
            agent_type: super::agent::Agent::ApoRia,
            required_tools: Vec::new(),
            tier: None,
            location: SkillLocation::default(),
        }
    }
}

impl SkillInfo {
    pub fn desc_from_str(s: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("en".to_string(), s.to_string());
        m
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptInjectionPolicy {
    Always,
    #[default]
    OnFirstUse,
    OnEveryUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool_name: String,
    pub description: String,
    pub mandatory_prompt: String,
    #[serde(default)]
    pub injection_policy: PromptInjectionPolicy,
    pub agent_type: super::agent::Agent,
    #[serde(default)]
    pub parameters: ToolParameters,
    #[serde(default)]
    pub tier: Option<ModelTier>,
    #[serde(default)]
    pub location: ToolLocation,
}

impl ToolConfig {
    pub fn new(tool_name: &str, description: &str, mandatory_prompt: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            description: description.to_string(),
            mandatory_prompt: mandatory_prompt.to_string(),
            injection_policy: PromptInjectionPolicy::default(),
            agent_type: super::agent::Agent::ApoRia,
            parameters: ToolParameters {
                param_type: "object".to_string(),
                required: vec![],
                properties: std::collections::HashMap::new(),
                separate_call_keys: Vec::new(),
            },
            tier: None,
            location: ToolLocation::default(),
        }
    }

    pub fn with_injection_policy(mut self, policy: PromptInjectionPolicy) -> Self {
        self.injection_policy = policy;
        self
    }

    pub fn with_agent_type(mut self, agent_type: super::agent::Agent) -> Self {
        self.agent_type = agent_type;
        self
    }

    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = Some(tier);
        self
    }

    pub fn with_location(mut self, location: ToolLocation) -> Self {
        self.location = location;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolPromptInjector {
    used_tools: HashSet<String>,
    current_prompt_section: Option<String>,
}

impl ToolPromptInjector {
    pub fn new() -> Self {
        Self {
            used_tools: HashSet::new(),
            current_prompt_section: None,
        }
    }

    pub fn should_inject(&self, config: &ToolConfig) -> bool {
        match config.injection_policy {
            PromptInjectionPolicy::Always => true,
            PromptInjectionPolicy::OnFirstUse => !self.used_tools.contains(&config.tool_name),
            PromptInjectionPolicy::OnEveryUse => true,
        }
    }

    pub fn prepare_tool_context(&mut self, config: &ToolConfig) -> Option<String> {
        if self.should_inject(config) {
            self.mark_tool_used(&config.tool_name);
            Some(config.mandatory_prompt.clone())
        } else {
            None
        }
    }

    pub fn mark_tool_used(&mut self, tool_name: &str) {
        self.used_tools.insert(tool_name.to_string());
    }

    pub fn has_used_tool(&self, tool_name: &str) -> bool {
        self.used_tools.contains(tool_name)
    }

    pub fn reset(&mut self) {
        self.used_tools.clear();
        self.current_prompt_section = None;
    }

    pub fn inject_to_system_prompt(&self, system_prompt: &mut String, mandatory_prompt: &str) {
        let section_name = "tool_constraints";
        let section_marker_start = format!("<{}>", section_name);
        let section_marker_end = format!("</{}>", section_name);

        let new_section = format!(
            "\n{}\n{}\n{}\n",
            section_marker_start, mandatory_prompt, section_marker_end
        );

        if let Some(start) = system_prompt.find(&section_marker_start) {
            if let Some(end) = system_prompt.find(&section_marker_end) {
                system_prompt.replace_range(start..end + section_marker_end.len(), &new_section);
            }
        } else {
            system_prompt.push_str(&new_section);
        }
    }

    pub fn replace_tool_section(&self, system_prompt: &str, mandatory_prompt: &str) -> String {
        let section_name = "tool_constraints";
        let section_marker_start = format!("<{}>", section_name);
        let section_marker_end = format!("</{}>", section_name);

        let start_idx = system_prompt.find(&section_marker_start);
        let end_idx = system_prompt.find(&section_marker_end);

        match (start_idx, end_idx) {
            (Some(start), Some(end)) => {
                let new_section = format!(
                    "{}\n{}\n{}",
                    section_marker_start, mandatory_prompt, section_marker_end
                );
                format!(
                    "{}{}{}",
                    &system_prompt[..start],
                    new_section,
                    &system_prompt[end + section_marker_end.len()..]
                )
            }
            _ => {
                let mut result = system_prompt.to_string();
                self.inject_to_system_prompt(&mut result, mandatory_prompt);
                result
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkedTodoItem {
    pub todo_id: Uuid,
    #[serde(default = "default_include_depth")]
    pub include_depth: u32,
    #[serde(default)]
    pub include_ancestors: bool,
    #[serde(default)]
    pub include_artifacts: bool,
}

fn default_include_depth() -> u32 {
    1
}

impl MarkedTodoItem {
    pub fn new(todo_id: &Uuid) -> Self {
        Self {
            todo_id: *todo_id,
            include_depth: 1,
            include_ancestors: false,
            include_artifacts: false,
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.include_depth = depth;
        self
    }

    pub fn with_ancestors(mut self) -> Self {
        self.include_ancestors = true;
        self
    }

    pub fn with_artifacts(mut self) -> Self {
        self.include_artifacts = true;
        self
    }

    pub fn simple(todo_id: &Uuid) -> Self {
        Self::new(todo_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarkerStrategy {
    Manual,
    AutoCritical,
    AutoUnfinished,
    #[default]
    Hybrid,
}

#[derive(Debug, Clone, Default)]
pub struct TodoMarker {
    marked_items: Vec<MarkedTodoItem>,
    marker_strategy: MarkerStrategy,
}

impl TodoMarker {
    pub fn new(strategy: MarkerStrategy) -> Self {
        Self {
            marked_items: Vec::new(),
            marker_strategy: strategy,
        }
    }

    pub fn mark(&mut self, item: MarkedTodoItem) {
        self.marked_items.push(item);
    }

    pub fn mark_multiple(&mut self, items: Vec<MarkedTodoItem>) {
        self.marked_items.extend(items);
    }

    pub fn get_marked_items(&self) -> &[MarkedTodoItem] {
        &self.marked_items
    }

    pub fn clear(&mut self) {
        self.marked_items.clear();
    }

    pub fn get_strategy(&self) -> MarkerStrategy {
        self.marker_strategy
    }

    pub fn set_strategy(&mut self, strategy: MarkerStrategy) {
        self.marker_strategy = strategy;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedContext {
    pub soul_prompt: String,
    pub skill_prompt: String,
    pub initial_user_input: String,
    pub preserved_state: PreserveState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreserveState {
    pub todo_tree_root: Option<String>,
    pub critical_decisions: Vec<String>,
}

impl Default for PreserveState {
    fn default() -> Self {
        Self::new()
    }
}

impl PreserveState {
    pub fn new() -> Self {
        Self {
            todo_tree_root: None,
            critical_decisions: Vec::new(),
        }
    }

    pub fn with_todo_root(mut self, root: &str) -> Self {
        self.todo_tree_root = Some(root.to_string());
        self
    }

    pub fn add_decision(mut self, decision: &str) -> Self {
        self.critical_decisions.push(decision.to_string());
        self
    }
}
