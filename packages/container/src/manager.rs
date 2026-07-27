use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tracing::warn;

use bollard::Docker;

use super::{
    errors::{ContainerError, ContainerResult},
    events::ContainerEvent,
    types::{ContainerDetail, ContainerInfo, ExecOutput},
};

#[derive(Debug, Clone)]
pub struct ContainerManager {
    pub(crate) docker: Docker,
    pub(crate) state: Arc<RwLock<HashMap<String, ContainerInfo>>>,
    pub(crate) event_tx: broadcast::Sender<ContainerEvent>,
}

impl ContainerManager {
    pub fn new() -> ContainerResult<Self> {
        let docker = super::docker_client::connect_local()?;

        let (event_tx, _) = broadcast::channel(256);

        Ok(Self {
            docker,
            state: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        })
    }

    pub fn new_with_socket(socket_path: &str) -> ContainerResult<Self> {
        let docker = super::docker_client::connect_socket(socket_path)?;

        let (event_tx, _) = broadcast::channel(256);

        Ok(Self {
            docker,
            state: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        })
    }

    pub fn from_docker(docker: Docker) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            docker,
            state: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ContainerEvent> {
        self.event_tx.subscribe()
    }

    pub async fn reconcile(&self) -> ContainerResult<Vec<ContainerInfo>> {
        let options = bollard::query_parameters::ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;

        let mut state = self.state.write().await;
        state.clear();

        let mut result = Vec::with_capacity(containers.len());
        for c in &containers {
            let info = self.summary_to_info(c);
            state.insert(info.id.clone(), info.clone());
            result.push(info);
        }

        Ok(result)
    }

    pub async fn list(&self) -> ContainerResult<Vec<ContainerInfo>> {
        let options = bollard::query_parameters::ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;
        Ok(containers.iter().map(|c| self.summary_to_info(c)).collect())
    }

    pub async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<HashMap<String, String>>,
        all: bool,
    ) -> ContainerResult<Vec<ContainerInfo>> {
        let mut filters = HashMap::new();

        if let Some(prefix) = name_prefix {
            filters.insert("name".to_string(), vec![prefix.to_string()]);
        }

        if let Some(labels) = label_filter {
            let label_filters: Vec<String> = labels
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        k.clone()
                    } else {
                        format!("{}={}", k, v)
                    }
                })
                .collect();
            filters.insert("label".to_string(), label_filters);
        }

        let options = bollard::query_parameters::ListContainersOptions {
            all,
            filters: Some(filters),
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;
        Ok(containers.iter().map(|c| self.summary_to_info(c)).collect())
    }

    pub async fn inspect(&self, container_id: &str) -> ContainerResult<ContainerDetail> {
        let response = self
            .docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        let info = self.inspect_to_info(&response);

        let detail = ContainerDetail {
            info: info.clone(),
            exit_code: response.state.as_ref().and_then(|s| s.exit_code),
            started_at: response.state.as_ref().and_then(|s| s.started_at.clone()),
            finished_at: response.state.as_ref().and_then(|s| s.finished_at.clone()),
            error: response.state.as_ref().and_then(|s| s.error.clone()),
        };

        let mut state = self.state.write().await;
        state.insert(info.id.clone(), info);

        Ok(detail)
    }

    pub async fn is_running(&self, container_id: &str) -> ContainerResult<bool> {
        let response = self
            .docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| {
                if matches!(
                    e,
                    bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }
                ) {
                    ContainerError::NotFound(container_id.to_string())
                } else {
                    ContainerError::from(e)
                }
            })?;

        Ok(response
            .state
            .as_ref()
            .map(|s| s.running.unwrap_or(false))
            .unwrap_or(false))
    }

    pub async fn get_cached(&self, container_id: &str) -> Option<ContainerInfo> {
        let state = self.state.read().await;
        state.get(container_id).cloned()
    }

    pub async fn get_cached_by_name(&self, name: &str) -> Option<ContainerInfo> {
        let state = self.state.read().await;
        state
            .values()
            .find(|c| c.name == name || c.name == format!("/{}", name))
            .cloned()
    }
}

#[async_trait::async_trait]
impl super::ops::ContainerOps for ContainerManager {
    async fn create(
        &self,
        params: &super::types::ContainerCreateParams,
    ) -> ContainerResult<ContainerInfo> {
        self.create(params).await
    }

    async fn start(&self, container_id: &str) -> ContainerResult<()> {
        self.start(container_id).await
    }

    async fn stop(&self, container_id: &str) -> ContainerResult<()> {
        self.stop(container_id).await
    }

    async fn remove(&self, container_id: &str, force: bool) -> ContainerResult<()> {
        self.remove(container_id, force).await
    }

    async fn restart(&self, container_id: &str) -> ContainerResult<()> {
        self.restart(container_id).await
    }

    async fn list(&self) -> ContainerResult<Vec<ContainerInfo>> {
        self.list().await
    }

    async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<std::collections::HashMap<String, String>>,
        all: bool,
    ) -> ContainerResult<Vec<ContainerInfo>> {
        self.list_with_filter(name_prefix, label_filter, all).await
    }

    async fn inspect(&self, container_id: &str) -> ContainerResult<ContainerDetail> {
        self.inspect(container_id).await
    }

    async fn is_running(&self, container_id: &str) -> ContainerResult<bool> {
        self.is_running(container_id).await
    }

    async fn exec(&self, container_id: &str, command: &[&str]) -> ContainerResult<ExecOutput> {
        self.exec(container_id, command).await
    }

    async fn fork(
        &self,
        params: &super::types::ContainerForkParams,
    ) -> ContainerResult<ContainerInfo> {
        self.fork(params).await
    }

    async fn commit(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
    ) -> ContainerResult<String> {
        self.commit(container_id, repo, tag).await
    }

    async fn commit_with_labels(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
        labels: Option<&std::collections::HashMap<String, String>>,
    ) -> ContainerResult<String> {
        self.commit_with_labels(container_id, repo, tag, labels)
            .await
    }

    async fn wait_healthy(
        &self,
        container_id: &str,
        timeout: std::time::Duration,
    ) -> ContainerResult<()> {
        self.wait_healthy(container_id, timeout).await
    }

    async fn ensure_running(&self, container_id: &str) -> ContainerResult<ContainerInfo> {
        self.ensure_running(container_id).await
    }

    async fn recreate(
        &self,
        container_name: &str,
        new_image: &str,
    ) -> ContainerResult<ContainerInfo> {
        self.recreate(container_name, new_image).await
    }

    async fn list_images(&self) -> ContainerResult<Vec<super::types::ImageInfo>> {
        self.list_images().await
    }

    async fn pull_image(&self, image: &str) -> ContainerResult<String> {
        self.pull_image(image).await
    }

    async fn image_exists(&self, image: &str) -> ContainerResult<bool> {
        self.image_exists(image).await
    }

    async fn remove_image(&self, image: &str, force: bool) -> ContainerResult<()> {
        self.remove_image(image, force).await
    }

    async fn remove_with_image(
        &self,
        container_id: &str,
        image: &str,
        force: bool,
    ) -> ContainerResult<()> {
        self.remove_with_image(container_id, image, force).await
    }

    async fn create_volume(&self, name: &str) -> ContainerResult<String> {
        self.create_volume(name).await
    }

    async fn remove_volume(&self, name: &str, force: bool) -> ContainerResult<()> {
        self.remove_volume(name, force).await
    }

    async fn volume_exists(&self, name: &str) -> ContainerResult<bool> {
        self.volume_exists(name).await
    }

    async fn list_volumes(&self) -> ContainerResult<Vec<super::types::DockerVolumeInfo>> {
        self.list_volumes().await
    }

    async fn logs(&self, container_id: &str, tail: usize) -> ContainerResult<Vec<String>> {
        self.logs(container_id, tail).await
    }

    async fn writable_rootfs(
        &self,
        container_id: &str,
    ) -> ContainerResult<super::types::WritableRootfs> {
        use super::types::WritableRootfs;
        let dir =
            tempfile::tempdir().map_err(|e| super::errors::ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("create temp dir for writable_rootfs: {}", e),
            })?;
        let tar_bytes = self.download_archive(container_id, "/home").await?;
        if !tar_bytes.is_empty() {
            let dir_path = dir.keep();
            let mut archive = tar::Archive::new(tar_bytes.as_slice());
            for entry in
                archive
                    .entries()
                    .map_err(|e| super::errors::ContainerError::OperationFailed {
                        container_id: container_id.to_string(),
                        message: format!("read tar entries: {}", e),
                    })?
            {
                let mut entry =
                    entry.map_err(|e| super::errors::ContainerError::OperationFailed {
                        container_id: container_id.to_string(),
                        message: format!("read tar entry: {}", e),
                    })?;
                let path = entry
                    .path()
                    .map_err(|e| super::errors::ContainerError::OperationFailed {
                        container_id: container_id.to_string(),
                        message: format!("read tar entry path: {}", e),
                    })?
                    .to_path_buf();
                if path.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir | std::path::Component::RootDir
                    )
                }) {
                    warn!(?path, "skipping tar entry with path traversal");
                    continue;
                }
                if let Err(e) = entry.unpack_in(&dir_path) {
                    warn!(?path, error = %e, "failed to unpack tar entry");
                }
            }
            return Ok(WritableRootfs {
                path: dir_path,
                is_direct: false,
            });
        }
        Ok(WritableRootfs {
            path: std::path::PathBuf::new(),
            is_direct: false,
        })
    }

    async fn diff_workspace(
        &self,
        container_id: &str,
        _workspace_path: &std::path::Path,
        base_path: &str,
    ) -> ContainerResult<Vec<super::types::PathChange>> {
        let changes = self.container_filesystem_changes(container_id).await?;
        Ok(changes
            .into_iter()
            .filter(|c| c.path.starts_with(base_path))
            .map(|c| super::types::PathChange {
                path: std::path::PathBuf::from(&c.path),
                kind: match c.kind {
                    bollard::models::ChangeType::_0 => super::types::ChangeKind::Modified,
                    bollard::models::ChangeType::_1 => super::types::ChangeKind::Added,
                    _ => super::types::ChangeKind::Deleted,
                },
            })
            .collect())
    }

    async fn download_archive(&self, container_id: &str, path: &str) -> ContainerResult<Vec<u8>> {
        self.download_archive(container_id, path).await
    }

    async fn upload_archive(
        &self,
        container_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> ContainerResult<()> {
        self.upload_archive(container_id, path, data).await
    }

    async fn get_container_logs(&self, container_id: &str, tail: usize) -> ContainerResult<String> {
        self.get_container_logs(container_id, tail).await
    }

    fn clone_boxed(&self) -> Box<dyn super::ops::ContainerOps> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_connect_to_docker() -> Result<()> {
        ContainerManager::new()?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_list_containers() -> Result<()> {
        let mgr = ContainerManager::new()?;
        let containers = mgr.list().await?;
        println!("Found {} containers", containers.len());
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_list_with_filter() -> Result<()> {
        let mgr = ContainerManager::new()?;
        let filtered = mgr.list_with_filter(Some("e-"), None, true).await?;
        println!("Found {} filtered containers", filtered.len());
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Docker daemon and running e-scepter container"]
    async fn test_inspect_container() -> Result<()> {
        let mgr = ContainerManager::new()?;
        let d = mgr.inspect("e-scepter").await?;
        assert!(d.info.name.contains("scepter"));
        assert!(d.info.status.is_running());
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires Docker daemon and running e-scepter container"]
    async fn test_is_running() -> Result<()> {
        let mgr = ContainerManager::new()?;
        let running = mgr.is_running("e-scepter").await?;
        assert!(running, "e-scepter should be running");
        Ok(())
    }
}
