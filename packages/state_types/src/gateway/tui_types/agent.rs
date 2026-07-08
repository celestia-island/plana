use serde::{Deserialize, Serialize};

use crate::agent::{Agent, AgentStatus, WorkStatus};
use _core::{AgentBadge, AgentId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RequestState {
    #[default]
    Idle,
    Waiting,
    Streaming,
    Retrying,
    WaitingTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompletionOutcome {
    #[default]
    None,
    Reported,
    Failed,
}

#[derive(Debug, Clone)]
pub struct AgentUpdateParams {
    pub agent_id: AgentId,
    pub agent_number: Option<String>,
    pub agent_type: Option<Agent>,
    pub llm_working: bool,
    pub work_status: Option<WorkStatus>,
    pub current_model: Option<String>,
    pub model_tier: Option<crate::types::ModelTier>,
    pub llm_handle: Option<String>,
    pub token_usage: Option<(u32, u32)>,
    pub request_state: RequestState,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    pub retry_count: Option<u32>,
    pub max_retries: Option<u32>,
    pub parent_id: Option<String>,
}

impl Default for AgentUpdateParams {
    fn default() -> Self {
        Self {
            agent_id: AgentId::system(),
            agent_number: None,
            agent_type: None,
            llm_working: false,
            work_status: None,
            current_model: None,
            model_tier: None,
            llm_handle: None,
            token_usage: None,
            request_state: RequestState::Idle,
            cpu_usage: 0.0,
            memory_mb: 0,
            retry_count: None,
            max_retries: None,
            parent_id: None,
        }
    }
}

impl AgentUpdateParams {
    pub fn new(agent_id: impl AsRef<str>, agent_type: Agent) -> Self {
        Self {
            agent_id: AgentId::from_raw(agent_id.as_ref()),
            agent_type: Some(agent_type),
            ..Default::default()
        }
    }

    pub fn for_skill(panel_id: impl AsRef<str>, agent_type: Agent) -> Self {
        let panel_id = AgentId::from_raw(panel_id.as_ref());
        Self {
            agent_number: Some(panel_id.to_string()),
            agent_type: Some(agent_type),
            agent_id: panel_id,
            ..Default::default()
        }
    }

    pub fn llm_working(mut self, working: bool) -> Self {
        self.llm_working = working;
        self
    }

    pub fn request_state(mut self, state: RequestState) -> Self {
        self.request_state = state;
        self
    }

    pub fn current_model(mut self, model: impl Into<String>) -> Self {
        self.current_model = Some(model.into());
        self
    }

    pub fn maybe_current_model(mut self, model: Option<String>) -> Self {
        self.current_model = model;
        self
    }

    pub fn work_status(mut self, status: WorkStatus) -> Self {
        self.work_status = Some(status);
        self
    }

    pub fn maybe_work_status(mut self, status: Option<WorkStatus>) -> Self {
        self.work_status = status;
        self
    }

    pub fn maybe_token_usage(mut self, usage: Option<(u32, u32)>) -> Self {
        self.token_usage = usage;
        self
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    pub fn maybe_parent_id(mut self, id: Option<String>) -> Self {
        self.parent_id = id;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiAgentInfo {
    pub agent_type: Agent,
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    #[serde(default)]
    pub agent_uuid: Option<AgentId>,
    pub agent_id: String,
    pub status: AgentStatus,
    pub llm_working: bool,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub work_status: Option<WorkStatus>,
    #[serde(default)]
    pub current_model: Option<String>,
    #[serde(default)]
    pub model_tier: Option<crate::types::ModelTier>,
    #[serde(default)]
    pub llm_handle: Option<String>,
    #[serde(default)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    pub mcp_tool_calls: u32,
    #[serde(default)]
    pub request_state: RequestState,
    #[serde(default)]
    pub completion_outcome: CompletionOutcome,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default)]
    pub max_retries: u32,
}
