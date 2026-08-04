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
pub struct SidecarDeliverResult {
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
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct NewContainerToolParams {
    pub image: String,
    pub name: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub volumes: Option<Vec<NewContainerVolumeMount>>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerStartParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerStopParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerRemoveParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerForkParams {
    pub container_id: String,
    pub name: Option<String>,
    pub namespace_volume: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
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
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<ContainerFilterCriteria>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ContainerInfoParams {
    pub container_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ExecOnContainerParams {
    pub command: String,
    pub container_id: Option<String>,
    pub target_badge: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct GitPushBranchParams {
    pub container_id: String,
    pub commit_message: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
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
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct SidecarSendParams {
    pub name: String,
    pub method: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub params: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct SidecarKillParams {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainListParams {
    pub amphoreus_dir: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct ToolchainEnsureParams {
    pub profile_id: String,
    pub amphoreus_dir: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
pub struct WaitParams {
    pub seconds: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/neikos.ts")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ContainerOpStatus;
    use serde_json::json;

    #[test]
    fn container_list_result_round_trip() {
        let r = ContainerListResult {
            total_count: 2,
            containers: vec![
                ContainerListItem {
                    name: "web".into(),
                    image: "nginx:latest".into(),
                    status: "running".into(),
                    id: "abc123".into(),
                },
                ContainerListItem {
                    name: "db".into(),
                    image: "postgres:16".into(),
                    status: "exited".into(),
                    id: "def456".into(),
                },
            ],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["total_count"], 2);
        assert_eq!(v["containers"][0]["name"], "web");
        let back: ContainerListResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.total_count, 2);
        assert_eq!(back.containers[1].name, "db");
    }

    #[test]
    fn container_list_result_empty() {
        let r = ContainerListResult {
            total_count: 0,
            containers: vec![],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["containers"], json!([]));
    }

    #[test]
    fn container_info_result_with_exit_code() {
        let r = ContainerInfoResult {
            container_id: "cid".into(),
            name: "test".into(),
            image: "img".into(),
            status: "exited".into(),
            running: false,
            exit_code: Some(0),
            ip_address: "172.17.0.2".into(),
            started_at: "2026-01-01T00:00:00Z".into(),
            ports: vec!["8080:80".into()],
            env: vec!["FOO=bar".into()],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["exit_code"], 0);
        assert_eq!(v["running"], false);
        let back: ContainerInfoResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.exit_code, Some(0));
    }

    #[test]
    fn container_info_result_no_exit_code() {
        let r = ContainerInfoResult {
            container_id: "cid".into(),
            name: "test".into(),
            image: "img".into(),
            status: "running".into(),
            running: true,
            exit_code: None,
            ip_address: "".into(),
            started_at: "".into(),
            ports: vec![],
            env: vec![],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["exit_code"], serde_json::Value::Null);
    }

    #[test]
    fn container_op_status_enum_round_trip() {
        for s in [
            ContainerOpStatus::Created,
            ContainerOpStatus::Running,
            ContainerOpStatus::Stopped,
            ContainerOpStatus::Removed,
            ContainerOpStatus::Forked,
        ] {
            let ser = serde_json::to_string(&s).unwrap();
            let de: ContainerOpStatus = serde_json::from_str(&ser).unwrap();
            assert_eq!(de, s);
        }
    }

    #[test]
    fn container_start_result_round_trip() {
        let r = ContainerStartResult {
            container_id: "cid".into(),
            status: ContainerOpStatus::Running,
        };
        let v = serde_json::to_value(&r).unwrap();
        // ContainerOpStatus serializes as PascalCase variant name.
        assert_eq!(v["status"], "Running");
        let back: ContainerStartResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.status, ContainerOpStatus::Running);
    }

    #[test]
    fn volume_info_round_trip() {
        let v = VolumeInfo {
            host_path: "/host/data".into(),
            container_path: "/data".into(),
            read_only: true,
        };
        let val = serde_json::to_value(&v).unwrap();
        assert_eq!(val["read_only"], true);
        let back: VolumeInfo = serde_json::from_value(val).unwrap();
        assert!(back.read_only);
    }

    #[test]
    fn exec_result_round_trip() {
        let r = ExecResult {
            container_id: "cid".into(),
            command: "ls -la".into(),
            exit_code: Some(0),
            output: "total 0\n".into(),
            error: String::new(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["exit_code"], 0);
        let back: ExecResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.exit_code, Some(0));
    }

    #[test]
    fn git_push_result_round_trip() {
        let r = GitPushResult {
            container_id: "cid".into(),
            branch: "feature".into(),
            remote: "origin".into(),
            commit_hash: Some("abc123".into()),
            pushed: true,
            output: "Everything up-to-date".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["pushed"], true);
        assert_eq!(v["commit_hash"], "abc123");
        let back: GitPushResult = serde_json::from_value(v).unwrap();
        assert!(back.pushed);
    }

    #[test]
    fn container_create_result_seccomp_default() {
        let r = ContainerCreateResult {
            image: "img".into(),
            container_id: "cid".into(),
            name: "test".into(),
            network: "bridge".into(),
            status: ContainerOpStatus::Created,
            volumes: vec![],
            seccomp_enabled: false,
        };
        let v = serde_json::to_value(&r).unwrap();
        // seccomp_enabled has #[serde(default)] on deserialize but always
        // serializes (no skip_serializing_if).
        assert_eq!(v["seccomp_enabled"], false);
    }

    #[test]
    fn container_filter_criteria_all_optional_skip() {
        let c = ContainerFilterCriteria {
            label: None,
            name: None,
            status: None,
        };
        let v = serde_json::to_value(&c).unwrap();
        // All fields use skip_serializing_if = "Option::is_none".
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn container_filter_criteria_with_values() {
        let mut labels = std::collections::HashMap::new();
        labels.insert("app".into(), "web".into());
        let c = ContainerFilterCriteria {
            label: Some(labels),
            name: Some("web-*".into()),
            status: Some(vec!["running".into()]),
        };
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["name"], "web-*");
        assert_eq!(v["label"]["app"], "web");
    }

    #[test]
    fn toolchain_profile_info_round_trip() {
        let r = ToolchainProfileInfo {
            id: "rust-full".into(),
            display_name: "Rust Toolchain".into(),
            source_image: "rust:1.85".into(),
            image_pulled: true,
            volume_ready: true,
            available_tools: vec!["cargo".into(), "rustc".into()],
            supported_languages: vec!["rust".into()],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["available_tools"][0], "cargo");
        let back: ToolchainProfileInfo = serde_json::from_value(v).unwrap();
        assert_eq!(back.available_tools.len(), 2);
    }
}
