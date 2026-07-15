use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContainerStatus {
    #[default]
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown,
}

impl ContainerStatus {
    pub fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn from_str(status: &str, running: bool) -> Self {
        if running {
            return Self::Running;
        }
        match status {
            "created" => Self::Created,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "exited" => Self::Exited,
            "dead" => Self::Dead,
            _ => Self::Unknown,
        }
    }

    pub fn from_runtime_status(status: &str, running: bool) -> Self {
        if running {
            return Self::Running;
        }
        match status {
            "created" | "Created" => Self::Created,
            "running" | "Running" => Self::Running,
            "paused" | "Paused" => Self::Paused,
            "restarting" | "Restarting" => Self::Restarting,
            "removing" | "Removing" => Self::Removing,
            "exited" | "Stopped" => Self::Exited,
            "dead" | "Dead" => Self::Dead,
            _ => Self::Unknown,
        }
    }

    pub fn from_docker_state(
        status: &bollard::service::ContainerStateStatusEnum,
        running: bool,
    ) -> Self {
        if running {
            return Self::Running;
        }
        match status {
            bollard::service::ContainerStateStatusEnum::CREATED => Self::Created,
            bollard::service::ContainerStateStatusEnum::RUNNING => Self::Running,
            bollard::service::ContainerStateStatusEnum::PAUSED => Self::Paused,
            bollard::service::ContainerStateStatusEnum::RESTARTING => Self::Restarting,
            bollard::service::ContainerStateStatusEnum::REMOVING => Self::Removing,
            bollard::service::ContainerStateStatusEnum::EXITED => Self::Exited,
            bollard::service::ContainerStateStatusEnum::DEAD => Self::Dead,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Restarting => write!(f, "restarting"),
            Self::Removing => write!(f, "removing"),
            Self::Exited => write!(f, "exited"),
            Self::Dead => write!(f, "dead"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    #[serde(default)]
    pub read_only: bool,
}

impl VolumeMount {
    pub fn new(host: impl Into<String>, container: impl Into<String>, read_only: bool) -> Self {
        Self {
            host_path: host.into(),
            container_path: container.into(),
            read_only,
        }
    }

    pub fn rw(host: impl Into<String>, container: impl Into<String>) -> Self {
        Self::new(host, container, false)
    }

    pub fn ro(host: impl Into<String>, container: impl Into<String>) -> Self {
        Self::new(host, container, true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    pub ports: Vec<PortMapping>,
    pub env: std::collections::HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HealthcheckParams {
    pub test: Vec<String>,
    pub interval_ns: Option<i64>,
    pub timeout_ns: Option<i64>,
    pub retries: Option<i64>,
    pub start_period_ns: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DeviceMapping {
    pub host_path: String,
    pub container_path: String,
    pub permissions: String,
}

#[derive(Debug, Clone)]
pub struct ContainerCreateParams {
    pub name: String,
    pub image: String,
    pub network: Option<String>,
    pub env: std::collections::HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMount>,
    pub labels: std::collections::HashMap<String, String>,
    pub working_dir: Option<String>,
    pub log_driver: Option<String>,
    pub log_opts: std::collections::HashMap<String, String>,
    pub healthcheck: Option<HealthcheckParams>,
    pub user: Option<String>,
    pub memory_limit: Option<i64>,
    pub nano_cpus: Option<i64>,
    pub pids_limit: Option<i64>,
    pub cap_drop: Option<Vec<String>>,
    pub cap_add: Option<Vec<String>>,
    pub security_opt: Option<Vec<String>>,
    pub read_only_rootfs: Option<bool>,
    pub egress_policy: Option<crate::egress::EgressPolicy>,
    pub group_add: Option<Vec<String>>,
    pub devices: Vec<DeviceMapping>,
    pub compile_mode: bool,
    pub egress_domain_allowlist_extra: Vec<String>,
}

impl ContainerCreateParams {
    pub fn simple(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            network: None,
            env: std::collections::HashMap::new(),
            ports: Vec::new(),
            volumes: Vec::new(),
            labels: std::collections::HashMap::new(),
            working_dir: None,
            log_driver: None,
            log_opts: std::collections::HashMap::new(),
            healthcheck: None,
            user: None,
            memory_limit: None,
            nano_cpus: None,
            pids_limit: None,
            cap_drop: None,
            cap_add: None,
            security_opt: None,
            read_only_rootfs: None,
            egress_policy: None,
            group_add: None,
            devices: Vec::new(),
            compile_mode: false,
            egress_domain_allowlist_extra: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerForkParams {
    pub source_container_id: String,
    pub new_name: String,
    pub network: Option<String>,
    pub volumes: Vec<VolumeMount>,
    pub env: std::collections::HashMap<String, String>,
    pub labels: std::collections::HashMap<String, String>,
    pub commit_labels: Option<std::collections::HashMap<String, String>>,
    pub image_tag: Option<String>,
    pub user: Option<String>,
    pub memory_limit: Option<i64>,
    pub nano_cpus: Option<i64>,
    pub pids_limit: Option<i64>,
    pub cap_drop: Option<Vec<String>>,
    pub cap_add: Option<Vec<String>>,
    pub security_opt: Option<Vec<String>>,
    pub read_only_rootfs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct ContainerDetail {
    #[serde(flatten)]
    pub info: ContainerInfo,
    pub exit_code: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct ExecOutput {
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct ImageInfo {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: i64,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, _macros::Getters)]
pub struct DockerVolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    Running,
    Stopped,
    NotExists,
    NotBuilt,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContainerRuntimeType {
    #[default]
    Youki,
    Docker,
    /// WSL Containers (`wslc.exe` / `container.exe`) — Microsoft's built-in Linux
    /// container runtime on Windows 11, available via `wsl --update --pre-release`.
    /// Eliminates the need for Docker Desktop on Windows.
    Wslc,
    /// Apple Container 1.0+ (`container`) — Apple's native Swift OCI runtime on
    /// macOS 26+ with Apple silicon. Each container runs in its own lightweight VM.
    #[serde(rename = "apple-container")]
    AppleContainer,
}

impl ContainerRuntimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Youki => "youki",
            Self::Docker => "docker",
            Self::Wslc => "wslc",
            Self::AppleContainer => "apple-container",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "docker" => Self::Docker,
            "wslc" | "wsl" => Self::Wslc,
            "apple-container" | "apple_container" | "applecontainer" | "apple" | "container" => {
                Self::AppleContainer
            }
            _ => Self::Youki,
        }
    }

    /// Returns true if this runtime is a CLI-based adapter (as opposed to a native
    /// API driver like Docker/bollard or libcontainer).
    pub fn is_cli_backend(self) -> bool {
        matches!(self, Self::Wslc | Self::AppleContainer)
    }
}

impl std::fmt::Display for ContainerRuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathChange {
    pub path: PathBuf,
    pub kind: ChangeKind,
}

pub struct WritableRootfs {
    pub path: PathBuf,
    pub is_direct: bool,
}
