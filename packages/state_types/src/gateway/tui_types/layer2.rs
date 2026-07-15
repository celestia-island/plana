use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2AgentInfo {
    pub name: String,
    pub description: String,
    pub mcp_count: usize,
    pub skills_count: usize,
    pub languages: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub skills_count: usize,
    pub source: String,
    pub version: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2McpToolInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    #[serde(default)]
    pub references_layer1: Vec<String>,
    #[serde(default)]
    pub references_layer2: Vec<String>,
    #[serde(default)]
    pub related_items: Vec<String>,
    #[serde(default)]
    pub referenced_by_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2SkillInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    #[serde(default)]
    pub references_layer1: Vec<String>,
    #[serde(default)]
    pub references_layer2: Vec<String>,
    #[serde(default)]
    pub related_items: Vec<String>,
    #[serde(default)]
    pub referenced_by_items: Vec<String>,
}
