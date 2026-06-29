use crate::enums::{ConsultationStatus, ContainerOpStatus};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerListItem {
    pub name: String,
    pub image: String,
    pub status: String,
    pub id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerListResult {
    pub total_count: usize,
    pub containers: Vec<ContainerListItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerInfoResult {
    pub container_id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub running: bool,
    pub exit_code: Option<i64>,
    pub ip_address: String,
    pub started_at: String,
    pub ports: Vec<String>,
    pub env: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerStartResult {
    pub container_id: String,
    pub status: ContainerOpStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerStopResult {
    pub container_id: String,
    pub status: ContainerOpStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerRemoveResult {
    pub container_id: String,
    pub status: ContainerOpStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerSnapshotResult {
    pub container_id: String,
    pub snapshot_id: String,
    pub image_id: String,
    pub image_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct VolumeInfo {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerCreateResult {
    pub image: String,
    pub container_id: String,
    pub name: String,
    pub network: String,
    pub status: ContainerOpStatus,
    pub volumes: Vec<VolumeInfo>,
    #[serde(default)]
    pub seccomp_enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerForkResult {
    pub parent_container_id: String,
    pub new_container_id: String,
    pub branch_level: u32,
    pub image: String,
    pub fallback: bool,
    pub status: ContainerOpStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ExecResult {
    pub container_id: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub output: String,
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct DeliverMessageResult {
    pub todo_id: String,
    pub title: String,
    pub target_badge: String,
    pub status: ConsultationStatus,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct GitPushResult {
    pub container_id: String,
    pub branch: String,
    pub remote: String,
    pub commit_hash: Option<String>,
    pub pushed: bool,
    pub output: String,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct NewContainerVolumeMount {
    pub source: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NewContainerToolParams {
    pub image: String,
    pub name: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub volumes: Option<Vec<NewContainerVolumeMount>>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerStartParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerStopParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerRemoveParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerForkParams {
    pub container_id: String,
    pub name: Option<String>,
    pub namespace_volume: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerSnapshotParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerFilterCriteria {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<std::collections::HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<ContainerFilterCriteria>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ContainerInfoParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ExecOnContainerParams {
    pub command: String,
    pub container_id: Option<String>,
    pub target_badge: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct GitPushBranchParams {
    pub container_id: String,
    pub commit_message: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SidecarSpawnParams {
    pub name: String,
    pub cmd: Option<Vec<String>>,
    pub language: Option<String>,
    pub framing: Option<String>,
    pub working_dir: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub idle_timeout_secs: Option<u64>,
    pub ready_pattern: Option<String>,
    pub amphoreus_dir: Option<String>,
    pub agent_folder: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SidecarSendParams {
    pub name: String,
    pub method: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub params: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SidecarKillParams {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ToolchainListParams {
    pub amphoreus_dir: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ToolchainEnsureParams {
    pub profile_id: String,
    pub amphoreus_dir: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct WaitParams {
    pub seconds: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct CheckWaitParams {
    pub handle: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainProfileInfo {
    pub id: String,
    pub display_name: String,
    pub source_image: String,
    pub image_pulled: bool,
    pub volume_ready: bool,
    pub available_tools: Vec<String>,
    pub supported_languages: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainListResult {
    pub profiles: Vec<ToolchainProfileInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainVolumeSpec {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainEnsureResult {
    pub profile_id: String,
    pub source_image: String,
    pub container_image: String,
    pub container_env: std::collections::HashMap<String, String>,
    pub container_volumes: Vec<ToolchainVolumeSpec>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct SidecarSendResult {
    pub name: String,
    pub sent: bool,
}
