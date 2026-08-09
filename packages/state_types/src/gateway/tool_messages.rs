use serde::{Deserialize, Serialize};

use crate::{
    agent::Agent,
    tools::{SkillInfo, ToolInfo},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ToolMessage {
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
        tools: Vec<ToolInfo>,
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
