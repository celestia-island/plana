mod defaults;
mod limits;
pub mod strings;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use defaults::{DEFAULT_NETWORK, RuntimeTuningConfig, StorageLifecycleConfig};
pub use limits::CONFIG;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = UnknownLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(UnknownLogLevelError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnknownLogLevelError(pub String);

impl std::fmt::Display for UnknownLogLevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown log level: {}", self.0)
    }
}

impl std::error::Error for UnknownLogLevelError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Azure,
    Local,
}

impl LlmProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmProvider::OpenAI => "openai",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Azure => "azure",
            LlmProvider::Local => "local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Base,
    Agent,
    Mcp,
    Skill,
    Node,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseAction {
    Heartbeat,
    Error,
    Ack,
}

impl BaseAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseAction::Heartbeat => "heartbeat",
            BaseAction::Error => "error",
            BaseAction::Ack => "ack",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    Register,
    Unregister,
    ListAgents,
    AgentListResponse,
    GetAgentInfo,
    AgentInfoResponse,
}

impl AgentAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentAction::Register => "register",
            AgentAction::Unregister => "unregister",
            AgentAction::ListAgents => "list_agents",
            AgentAction::AgentListResponse => "agent_list_response",
            AgentAction::GetAgentInfo => "get_agent_info",
            AgentAction::AgentInfoResponse => "agent_info_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum McpAction {
    CallTool,
    ToolResponse,
    ListTools,
    ToolsListResponse,
}

impl McpAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            McpAction::CallTool => "call_tool",
            McpAction::ToolResponse => "tool_response",
            McpAction::ListTools => "list_tools",
            McpAction::ToolsListResponse => "tools_list_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillAction {
    CallSkill,
    SkillResponse,
    ListSkills,
    SkillsListResponse,
}

impl SkillAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkillAction::CallSkill => "call_skill",
            SkillAction::SkillResponse => "skill_response",
            SkillAction::ListSkills => "list_skills",
            SkillAction::SkillsListResponse => "skills_list_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeAction {
    DiscoverNodes,
    NodeListResponse,
    GetNodeInfo,
    NodeInfoResponse,
}

impl NodeAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeAction::DiscoverNodes => "discover_nodes",
            NodeAction::NodeListResponse => "node_list_response",
            NodeAction::GetNodeInfo => "get_node_info",
            NodeAction::NodeInfoResponse => "node_info_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonitorAction {
    GetMetrics,
    MetricsResponse,
    SubscribeMetrics,
    UnsubscribeMetrics,
}

impl MonitorAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            MonitorAction::GetMetrics => "get_metrics",
            MonitorAction::MetricsResponse => "metrics_response",
            MonitorAction::SubscribeMetrics => "subscribe_metrics",
            MonitorAction::UnsubscribeMetrics => "unsubscribe_metrics",
        }
    }
}

pub fn is_invalid_api_key(api_key: &str) -> bool {
    api_key.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_invalid_api_key() -> Result<()> {
        assert!(is_invalid_api_key(""));
        assert!(is_invalid_api_key("   "));
        assert!(is_invalid_api_key("\t\n"));
        assert!(!is_invalid_api_key("sk-abc123"));
        assert!(!is_invalid_api_key(" key "));
        Ok(())
    }
}
