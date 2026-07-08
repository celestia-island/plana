use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent::{CompletionOutcome, RequestState, TuiAgentInfo};
use crate::agent::{Agent, WorkStatus};
use arona_container::ContainerStatus;
use arona_core::{AgentBadge, AgentId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPatch {
    pub agent_id: AgentId,
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    #[serde(default)]
    pub agent_type: Option<Agent>,
    pub version: u64,
    #[serde(default)]
    pub llm_working_changed: Option<bool>,
    #[serde(default)]
    pub work_status: Option<WorkStatus>,
    #[serde(default)]
    pub current_model: Option<String>,
    #[serde(default)]
    pub model_tier: Option<crate::types::ModelTier>,
    #[serde(default)]
    pub llm_handle: Option<String>,
    #[serde(default)]
    pub token_usage_delta: Option<(u32, u32)>,
    #[serde(default)]
    pub token_usage_absolute: Option<(u32, u32)>,
    #[serde(default)]
    pub request_state: Option<RequestState>,
    #[serde(default)]
    pub mcp_tool_calls_delta: Option<u32>,
    #[serde(default)]
    pub skill_calls_delta: Option<u32>,
    #[serde(default)]
    pub cpu_usage: Option<f64>,
    #[serde(default)]
    pub memory_mb: Option<u64>,
    #[serde(default)]
    pub completion_outcome: Option<CompletionOutcome>,
    #[serde(default)]
    pub retry_count: Option<u32>,
    #[serde(default)]
    pub max_retries: Option<u32>,
    #[serde(default)]
    pub current_stage: Option<String>,
    #[serde(default)]
    pub next_stage: Option<String>,
    #[serde(default)]
    pub current_tool_name: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub containers: Vec<ContainerInfo>,
    pub active_tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider_name: String,
    pub model_type: String,
    pub context_length: Option<u32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub api_endpoint: String,
    pub has_api_key: bool,
    pub default_model: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub title: String,
    pub status: crate::TaskStatus,
    pub progress: u8,
    pub assigned_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPatch {
    pub container_id: String,
    pub version: u64,
    #[serde(default)]
    pub status_changed: Option<ContainerStatus>,
    #[serde(default)]
    pub cpu_usage_changed: Option<f64>,
    #[serde(default)]
    pub memory_usage_changed: Option<u64>,
    #[serde(default)]
    pub branch_changed: Option<String>,
    #[serde(default)]
    pub is_read_only_changed: Option<bool>,
    #[serde(default)]
    pub badge_changed: Option<AgentBadge>,
    #[serde(default)]
    pub current_skill_changed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPatch {
    pub task_id: Uuid,
    pub version: u64,
    #[serde(default)]
    pub status_changed: Option<crate::TaskStatus>,
    #[serde(default)]
    pub progress_changed: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: ContainerStatus,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub branch_level: u32,
    #[serde(default)]
    pub is_cosmos: bool,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    pub badge: Option<AgentBadge>,
    #[serde(default)]
    pub current_skill: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    pub git_remote_url: Option<String>,
    #[serde(default)]
    pub git_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn test_container_info_with_workspace_fields() -> Result<()> {
        let info = ContainerInfo {
            id: "container-123".to_string(),
            name: "e-skemma-abc12345".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 0.5,
            memory_mb: 256,
            image: "127.0.0.1:5000/entelecheia".to_string(),
            agent_type: Some("SkeMma".to_string()),
            parent_id: None,
            branch_level: 0,
            is_cosmos: true,
            branch: Some("cosmos/auto-plan_execute".to_string()),
            is_read_only: false,
            badge: Some(AgentBadge::new("001").context("invalid badge")?),
            current_skill: Some("plan_execute".to_string()),
            workspace_id: Some("ws-uuid-here".to_string()),
            workspace_path: Some("/home/user/project".to_string()),
            git_remote_url: Some("https://github.com/org/repo.git".to_string()),
            git_branch: Some("main".to_string()),
        };

        let json = serde_json::to_string(&info)?;
        let deserialized: ContainerInfo = serde_json::from_str(&json)?;

        assert_eq!(deserialized.workspace_id, Some("ws-uuid-here".to_string()));
        assert_eq!(
            deserialized.workspace_path,
            Some("/home/user/project".to_string())
        );
        assert_eq!(
            deserialized.git_remote_url,
            Some("https://github.com/org/repo.git".to_string())
        );
        assert_eq!(deserialized.git_branch, Some("main".to_string()));
        Ok(())
    }

    #[test]
    fn test_container_info_backward_compatible() -> Result<()> {
        let old_json = r#"{
            "id": "c1",
            "name": "e-test",
            "status": "running",
            "cpu_usage": 0.0,
            "memory_mb": 0,
            "image": "",
            "agent_type": null,
            "parent_id": null,
            "branch_level": 0,
            "is_cosmos": false,
            "branch": null,
            "is_read_only": false,
            "badge": null,
            "current_skill": null
        }"#;
        let info: ContainerInfo = serde_json::from_str(old_json)?;
        assert!(info.workspace_id.is_none());
        assert!(info.workspace_path.is_none());
        assert!(info.git_remote_url.is_none());
        assert!(info.git_branch.is_none());
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryData {
    pub source: String,
    pub instance_uuid: Option<String>,
    pub level: String,
    pub target: Option<String>,
    pub message: String,
    pub fields: serde_json::Value,
    pub created_at: String,
}
