use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    #[serde(default)]
    pub mcp_tools: Vec<arona_state_sync::McpToolInfo>,
    #[serde(default)]
    pub skills: Vec<arona_state_sync::SkillInfo>,
}

impl AgentMetadata {
    pub fn new(
        mcp_tools: Vec<arona_state_sync::McpToolInfo>,
        skills: Vec<arona_state_sync::SkillInfo>,
    ) -> Self {
        Self { mcp_tools, skills }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogContext {
    pub mcp_tools: Option<Vec<arona_state_sync::McpToolInfo>>,
    pub skills: Option<Vec<arona_state_sync::SkillInfo>>,
}

impl LogContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mcp_tools(mut self, tools: Vec<arona_state_sync::McpToolInfo>) -> Self {
        self.mcp_tools = Some(tools);
        self
    }

    pub fn with_skills(mut self, skills: Vec<arona_state_sync::SkillInfo>) -> Self {
        self.skills = Some(skills);
        self
    }
}
