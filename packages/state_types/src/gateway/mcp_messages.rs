use serde::{Deserialize, Serialize};

use crate::{
    agent::Agent,
    mcp::{McpToolInfo, SkillInfo},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum McpMessage {
    CallTool {
        tool_name: String,
        agent_type: Agent,
        parameters: serde_json::Value,
    },
    ToolResponse {
        result: serde_json::Value,
    },
    ListTools {
        agent_type: Option<Agent>,
    },
    ToolsListResponse {
        tools: Vec<McpToolInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SkillMessage {
    CallSkill {
        skill_name: String,
        agent_type: Agent,
        parameters: serde_json::Value,
    },
    SkillResponse {
        result: serde_json::Value,
    },
    ListSkills {
        agent_type: Option<Agent>,
    },
    SkillsListResponse {
        skills: Vec<SkillInfo>,
    },
}
