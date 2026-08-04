use std::{collections::HashMap, path::Path};

use async_trait::async_trait;

use super::{
    errors::ContainerResult,
    types::{
        ContainerCreateParams, ContainerDetail, ContainerForkParams, ContainerInfo, ExecOutput,
        ImageInfo, PathChange, WritableRootfs,
    },
};

#[async_trait]
pub trait ContainerOps: Send + Sync {
    // -- Lifecycle --

    async fn create(&self, params: &ContainerCreateParams) -> ContainerResult<ContainerInfo>;
    async fn start(&self, container_id: &str) -> ContainerResult<()>;
    async fn stop(&self, container_id: &str) -> ContainerResult<()>;
    async fn remove(&self, container_id: &str, force: bool) -> ContainerResult<()>;
    async fn restart(&self, container_id: &str) -> ContainerResult<()>;

    // -- Query --

    async fn list(&self) -> ContainerResult<Vec<ContainerInfo>>;
    async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<HashMap<String, String>>,
        all: bool,
    ) -> ContainerResult<Vec<ContainerInfo>>;
    async fn inspect(&self, container_id: &str) -> ContainerResult<ContainerDetail>;
    async fn is_running(&self, container_id: &str) -> ContainerResult<bool>;

    // -- Exec --

    async fn exec(&self, container_id: &str, command: &[&str]) -> ContainerResult<ExecOutput>;

    // -- Fork / Commit --

    async fn fork(&self, params: &ContainerForkParams) -> ContainerResult<ContainerInfo>;
    async fn commit(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
    ) -> ContainerResult<String>;
    async fn commit_with_labels(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
        labels: Option<&HashMap<String, String>>,
    ) -> ContainerResult<String>;

    // -- Health / Ensure --

    async fn wait_healthy(
        &self,
        container_id: &str,
        timeout: std::time::Duration,
    ) -> ContainerResult<()>;
    async fn ensure_running(&self, container_id: &str) -> ContainerResult<ContainerInfo>;
    async fn recreate(
        &self,
        container_name: &str,
        new_image: &str,
    ) -> ContainerResult<ContainerInfo>;

    // -- Images --

    async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>>;
    async fn pull_image(&self, image: &str) -> ContainerResult<String>;
    async fn image_exists(&self, image: &str) -> ContainerResult<bool>;
    async fn remove_image(&self, image: &str, force: bool) -> ContainerResult<()>;
    async fn remove_with_image(
        &self,
        container_id: &str,
        image: &str,
        force: bool,
    ) -> ContainerResult<()>;

    // -- Volumes --

    async fn create_volume(&self, name: &str) -> ContainerResult<String>;
    async fn remove_volume(&self, name: &str, force: bool) -> ContainerResult<()>;
    async fn volume_exists(&self, name: &str) -> ContainerResult<bool>;
    async fn list_volumes(&self) -> ContainerResult<Vec<super::types::DockerVolumeInfo>>;

    // -- Logs --

    async fn logs(&self, container_id: &str, tail: usize) -> ContainerResult<Vec<String>>;

    async fn get_container_logs(&self, container_id: &str, tail: usize) -> ContainerResult<String>;

    // -- Server Detection --

    async fn detect_server_status(&self, container_name: &str) -> super::types::ServerStatus {
        let containers = self
            .list_with_filter(Some(&format!("^{}$", container_name)), None, false)
            .await;
        match containers {
            Ok(c) if !c.is_empty() => super::types::ServerStatus::Running,
            Ok(_) => {
                let all = self
                    .list_with_filter(Some(&format!("^{}$", container_name)), None, true)
                    .await;
                match all {
                    Ok(c) if !c.is_empty() => super::types::ServerStatus::Stopped,
                    Ok(_) => super::types::ServerStatus::NotExists,
                    Err(_) => super::types::ServerStatus::Unknown,
                }
            }
            Err(_) => super::types::ServerStatus::Unknown,
        }
    }

    // -- Rootfs Access (merge pipeline replacement) --

    async fn writable_rootfs(&self, container_id: &str) -> ContainerResult<WritableRootfs>;

    async fn diff_workspace(
        &self,
        container_id: &str,
        workspace_path: &Path,
        base_path: &str,
    ) -> ContainerResult<Vec<PathChange>>;

    async fn download_archive(&self, container_id: &str, path: &str) -> ContainerResult<Vec<u8>>;

    async fn upload_archive(
        &self,
        container_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> ContainerResult<()>;

    // -- Clone --

    fn clone_boxed(&self) -> Box<dyn ContainerOps>;
}

impl Clone for Box<dyn ContainerOps> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}
