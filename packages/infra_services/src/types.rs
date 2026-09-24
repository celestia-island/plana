use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    #[serde(default)]
    pub tools: Vec<plana_state_sync::ToolInfo>,
    #[serde(default)]
    pub skills: Vec<plana_state_sync::SkillInfo>,
}

impl AgentMetadata {
    pub fn new(
        tools: Vec<plana_state_sync::ToolInfo>,
        skills: Vec<plana_state_sync::SkillInfo>,
    ) -> Self {
        Self { tools, skills }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogContext {
    pub tools: Option<Vec<plana_state_sync::ToolInfo>>,
    pub skills: Option<Vec<plana_state_sync::SkillInfo>>,
}

impl LogContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tools(mut self, tools: Vec<plana_state_sync::ToolInfo>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_skills(mut self, skills: Vec<plana_state_sync::SkillInfo>) -> Self {
        self.skills = Some(skills);
        self
    }
}
