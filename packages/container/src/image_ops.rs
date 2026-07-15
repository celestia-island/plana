use futures::StreamExt;
use std::collections::HashMap;
use uuid::Uuid;

use bollard::{
    container::LogOutput,
    query_parameters::{CommitContainerOptions, RemoveImageOptions},
};
use tracing::{info, warn};

use super::{
    errors::{ContainerError, ContainerResult},
    manager::ContainerManager,
    types::{ContainerForkParams, ContainerInfo, ImageInfo},
};

const COSMOS_FORK_PREFIX: &str = "cosmos-fork-";
const COSMOS_SNAPSHOT_PREFIX: &str = "cosmos-snapshot-";

impl ContainerManager {
    pub async fn remove_with_image(
        &self,
        container_id: &str,
        image: &str,
        force: bool,
    ) -> ContainerResult<()> {
        self.remove(container_id, force).await?;

        if image.starts_with(COSMOS_FORK_PREFIX) || image.starts_with(COSMOS_SNAPSHOT_PREFIX) {
            self.remove_image(image, true).await?;
        }

        Ok(())
    }

    pub async fn commit(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
    ) -> ContainerResult<String> {
        self.commit_with_labels(container_id, repo, tag, None).await
    }

    pub async fn commit_with_labels(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
        labels: Option<&std::collections::HashMap<String, String>>,
    ) -> ContainerResult<String> {
        let options = CommitContainerOptions {
            container: Some(container_id.to_string()),
            repo: Some(repo.to_string()),
            tag: Some(tag.map(|t| t.to_string()).unwrap_or_default()),
            ..Default::default()
        };

        let mut config = bollard::models::ContainerConfig::default();
        if let Some(lbls) = labels {
            config.labels = Some(lbls.clone());
        }

        let response = self
            .docker
            .commit_container(options, config)
            .await
            .map_err(|e| ContainerError::CommitFailed(e.to_string()))?;

        Ok(response.id)
    }

    pub async fn fork(&self, params: &ContainerForkParams) -> ContainerResult<ContainerInfo> {
        let image_tag = params
            .image_tag
            .clone()
            .unwrap_or_else(|| format!("{}{}", COSMOS_FORK_PREFIX, Uuid::now_v7()));

        let image_id = self
            .commit_with_labels(
                &params.source_container_id,
                &image_tag,
                None,
                params.commit_labels.as_ref(),
            )
            .await
            .map_err(|e| {
                ContainerError::CommitFailed(format!(
                    "commit {} failed: {}",
                    params.source_container_id, e
                ))
            })?;

        let create_params = super::types::ContainerCreateParams {
            name: params.new_name.clone(),
            image: image_tag.clone(),
            network: params.network.clone(),
            env: params.env.clone(),
            ports: Vec::new(),
            volumes: params.volumes.clone(),
            labels: params.labels.clone(),
            working_dir: None,
            log_driver: None,
            log_opts: HashMap::new(),
            healthcheck: None,
            user: params.user.clone(),
            memory_limit: params.memory_limit,
            nano_cpus: params.nano_cpus,
            pids_limit: params.pids_limit,
            cap_drop: params.cap_drop.clone(),
            cap_add: params.cap_add.clone(),
            security_opt: params.security_opt.clone(),
            read_only_rootfs: params.read_only_rootfs,
            egress_policy: None,
            group_add: None,
            devices: Vec::new(),
            compile_mode: false,
            egress_domain_allowlist_extra: Vec::new(),
        };

        match self.create(&create_params).await {
            Ok(info) => Ok(info),
            Err(e) => {
                // Clean up the committed image to prevent disk space leaks.
                let _ = self.remove_image(&image_tag, true).await;
                Err(ContainerError::OperationFailed {
                    container_id: params.source_container_id.clone(),
                    message: format!(
                        "commit succeeded (image {}), but create failed: {}",
                        image_id, e
                    ),
                })
            }
        }
    }

    pub async fn remove_image(&self, image: &str, force: bool) -> ContainerResult<()> {
        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };

        self.docker
            .remove_image(image, Some(options), None)
            .await
            .map_err(|e| ContainerError::ImageFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn pull_image(&self, image: &str) -> ContainerResult<String> {
        use bollard::query_parameters::CreateImageOptions;

        let options = CreateImageOptions {
            from_image: Some(image.to_string()),
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);
        let mut digest = String::new();

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    if let Some(id) = info.id {
                        digest = id;
                    }
                }
                Err(e) => {
                    return Err(ContainerError::ImageFailed(format!(
                        "pull {} failed: {}",
                        image, e
                    )));
                }
            }
        }

        info!(image = %image, digest = %digest, "image pulled");

        Ok(digest)
    }

    pub async fn inspect_image_id(&self, image: &str) -> ContainerResult<String> {
        let inspect = self
            .docker
            .inspect_image(image)
            .await
            .map_err(|e| ContainerError::ImageFailed(e.to_string()))?;

        Ok(inspect.id.unwrap_or_default())
    }

    pub async fn image_exists(&self, image: &str) -> ContainerResult<bool> {
        match self.docker.inspect_image(image).await {
            Ok(_) => Ok(true),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => Ok(false),
            Err(e) => Err(ContainerError::ImageFailed(e.to_string())),
        }
    }

    pub async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>> {
        let options = bollard::query_parameters::ListImagesOptions {
            ..Default::default()
        };
        let images = self.docker.list_images(Some(options)).await?;
        let result = images
            .iter()
            .flat_map(|img| {
                if img.repo_tags.is_empty() {
                    vec![ImageInfo {
                        id: img.id.clone().trim_start_matches("sha256:").to_string(),
                        repository: "<none>".to_string(),
                        tag: "<none>".to_string(),
                        size: img.size,
                        created: img.created,
                    }]
                } else {
                    img.repo_tags
                        .iter()
                        .map(|tag_str: &String| {
                            let (repo, t) = tag_str
                                .rsplit_once(':')
                                .unwrap_or((tag_str.as_str(), "latest"));
                            ImageInfo {
                                id: img.id.clone().trim_start_matches("sha256:").to_string(),
                                repository: repo.to_string(),
                                tag: t.to_string(),
                                size: img.size,
                                created: img.created,
                            }
                        })
                        .collect()
                }
            })
            .collect();
        Ok(result)
    }

    pub async fn logs(&self, container_id: &str, tail: usize) -> ContainerResult<Vec<String>> {
        let options = bollard::query_parameters::LogsOptions {
            tail: tail.to_string(),
            stdout: true,
            stderr: true,
            timestamps: false,
            ..Default::default()
        };

        let stream = self.docker.logs(container_id, Some(options));
        let mut lines = Vec::new();

        let mut stream = Box::pin(stream);
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(LogOutput::StdOut { message }) => {
                    lines.push(String::from_utf8_lossy(&message).to_string());
                }
                Ok(LogOutput::StdErr { message }) => {
                    lines.push(String::from_utf8_lossy(&message).to_string());
                }
                Ok(LogOutput::Console { message }) => {
                    lines.push(String::from_utf8_lossy(&message).to_string());
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("log stream error: {}", e);
                    break;
                }
            }
        }

        Ok(lines)
    }

    pub async fn get_container_logs(
        &self,
        container_id: &str,
        tail: usize,
    ) -> ContainerResult<String> {
        let options = bollard::query_parameters::LogsOptions {
            tail: tail.to_string(),
            stdout: true,
            stderr: true,
            timestamps: false,
            ..Default::default()
        };
        let stream = self.docker.logs(container_id, Some(options));
        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut stream = Box::pin(stream);
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(LogOutput::StdOut { message }) => {
                    stdout.push_str(&String::from_utf8_lossy(&message));
                }
                Ok(LogOutput::StdErr { message }) => {
                    stderr.push_str(&String::from_utf8_lossy(&message));
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        Ok(format!("{}\n{}", stdout, stderr))
    }

    pub async fn detect_server_status(&self, container_name: &str) -> super::types::ServerStatus {
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
}
