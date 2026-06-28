//! Layer-2 / custom agent registry types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentInfo {
    pub name: String,
    pub description: String,
    pub mcp_count: usize,
    pub skills_count: usize,
    pub languages: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentListResponseParams {
    pub agents: Vec<Layer2AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2McpToolInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub references_layer1: Vec<String>,
    pub references_layer2: Vec<String>,
    pub related_items: Vec<String>,
    pub referenced_by_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentMcpResponseParams {
    pub agent_name: String,
    pub tools: Vec<Layer2McpToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2SkillInfo {
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub references_layer1: Vec<String>,
    pub references_layer2: Vec<String>,
    pub related_items: Vec<String>,
    pub referenced_by_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2AgentSkillsResponseParams {
    pub agent_name: String,
    pub skills: Vec<Layer2SkillInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2McpPromptResponseParams {
    pub agent_name: String,
    pub tool: String,
    pub lang: String,
    pub content: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct Layer2SkillPromptResponseParams {
    pub agent_name: String,
    pub skill: String,
    pub lang: String,
    pub content: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CustomAgentInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub skills_count: usize,
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub version: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct CustomAgentListResponseParams {
    pub agents: Vec<CustomAgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SubscribeCustomAgentResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub agent: Option<CustomAgentInfo>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UnsubscribeCustomAgentResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}
