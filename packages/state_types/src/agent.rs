use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::mcp::{McpToolInfo, SkillInfo};
use _core::{AgentBadge, AgentId};
use _domain_agent::AgentKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CustomAgentId(pub String);

impl CustomAgentId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CustomAgentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CustomAgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    SimpleTool,
    ComplexTool {
        agent_number: Option<u16>,
        #[serde(default)]
        preassigned_badge: Option<String>,
    },
}

impl Display for AgentCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCategory::SimpleTool => write!(f, "simple_tool"),
            AgentCategory::ComplexTool {
                agent_number,
                preassigned_badge,
            } => {
                if let Some(badge) = preassigned_badge {
                    write!(f, "complex_tool#{}", badge)
                } else {
                    match agent_number {
                        Some(n) => write!(f, "complex_tool#{:03}", n),
                        None => write!(f, "complex_tool#pending"),
                    }
                }
            }
        }
    }
}

macro_rules! agent_variants {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
        pub enum Agent {
            $($name,)*
        }

        impl Agent {
            pub fn from_kind(kind: AgentKind) -> Self {
                match kind {
                    $(AgentKind::$name => Agent::$name,)*
                }
            }

            pub fn into_kind(self) -> AgentKind {
                match self {
                    $(Agent::$name => AgentKind::$name,)*
                }
            }

            pub fn as_kind(&self) -> AgentKind {
                match self {
                    $(Agent::$name => AgentKind::$name,)*
                }
            }

            pub fn folder_name(&self) -> &'static str {
                self.as_kind().folder_name()
            }

            pub fn friendly_name(&self) -> &'static str {
                self.as_kind().friendly_name()
            }

            pub fn description(&self) -> &'static str {
                self.as_kind().description()
            }

            pub fn all() -> Vec<Self> {
                vec![$(Agent::$name,)*]
            }
        }

        impl Display for Agent {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_kind().folder_name())
            }
        }

        impl std::str::FromStr for Agent {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                AgentKind::from_str(s)
                    .map(Agent::from_kind)
                    .map_err(|_| format!("unknown agent: {}", s))
            }
        }

        impl From<AgentKind> for Agent {
            fn from(kind: AgentKind) -> Self {
                Agent::from_kind(kind)
            }
        }

        impl From<Agent> for AgentKind {
            fn from(agent: Agent) -> Self {
                agent.into_kind()
            }
        }
    };
}

agent_variants!(
    HapLotes,
    SkoPeo,
    HubRis,
    KaLos,
    NeiKos,
    SkeMma,
    ApoRia,
    EleOs,
    EpieiKeia,
    OreXis,
    PhiLia,
    PoleMos,
    WebAutomation,
    ClassicSoftwareEngineering,
    WebUiPanel,
    IndustrialIoT,
    RemoteOperations,
);

impl Agent {
    pub fn display_name(&self) -> &str {
        self.friendly_name()
    }

    pub fn formatted_name(&self, agent_id: &str) -> String {
        AgentBadge::new(agent_id)
            .map(|b| b.to_string())
            .unwrap_or_else(|| self.friendly_name().to_string())
    }

    pub fn is_layer2(&self) -> bool {
        self.as_kind().is_layer2()
    }
}

/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Initializing,
    Online,
    Busy,
    Offline,
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Initializing => write!(f, "Initializing"),
            AgentStatus::Online => write!(f, "Online"),
            AgentStatus::Busy => write!(f, "Busy"),
            AgentStatus::Offline => write!(f, "Offline"),
            AgentStatus::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStatus {
    Thinking,
    StreamingResponse,
    Executing { skill_name: String },
    Retrying { retry_count: u32, max_retries: u32 },
    Nudging,
    Completed,
    RequestFailed,
    Failed,
    ToolLoopTerminated,
    CallingTool,
}

impl std::fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkStatus::Thinking => write!(f, "Thinking"),
            WorkStatus::StreamingResponse => write!(f, "Streaming response"),
            WorkStatus::Executing { .. } => write!(f, "Executing"),
            WorkStatus::Retrying {
                retry_count,
                max_retries,
            } => {
                write!(f, "Retrying ({}/{})", retry_count, max_retries)
            }
            WorkStatus::Nudging => write!(f, "Nudging for report"),
            WorkStatus::Completed => write!(f, "Completed"),
            WorkStatus::RequestFailed => write!(f, "Request Failed"),
            WorkStatus::Failed => write!(f, "Failed"),
            WorkStatus::ToolLoopTerminated => write!(f, "Tool loop terminated"),
            WorkStatus::CallingTool => write!(f, "Calling tool"),
        }
    }
}

impl WorkStatus {
    pub fn display_tag(&self, agent_type: &Agent, tool_name: Option<&str>) -> String {
        match self {
            WorkStatus::Executing { skill_name } => skill_name.clone(),
            WorkStatus::CallingTool => tool_name
                .map(|t| format!("{}::{}", agent_type, t))
                .unwrap_or_else(|| agent_type.to_string()),
            _ => agent_type.to_string(),
        }
    }
}

/// Agent info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_type: Agent,
    pub agent_id: AgentId,
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    pub status: AgentStatus,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub agent_port: Option<u16>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub platform_version: Option<String>,
    pub mcp_tools: Vec<McpToolInfo>,
    pub skills: Vec<SkillInfo>,
    #[serde(default)]
    pub parent_agent_id: Option<AgentId>,
}

/// Agent register request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegisterRequest {
    pub agent_type: Agent,
    pub agent_id: AgentId,
    pub category: AgentCategory,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub agent_port: Option<u16>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub platform_version: Option<String>,
    pub mcp_tools: Vec<McpToolInfo>,
    pub skills: Vec<SkillInfo>,
    #[serde(default)]
    pub parent_agent_id: Option<AgentId>,
}

/// Agent unregister request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUnregisterRequest {
    /// Agent type
    pub agent_type: Agent,
    /// Agent ID
    pub agent_id: AgentId,
}
