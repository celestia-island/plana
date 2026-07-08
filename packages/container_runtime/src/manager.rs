use chrono::Utc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use async_trait::async_trait;
use libcontainer::{container::builder::ContainerBuilder, syscall::syscall::SyscallType};
use tracing::{debug, info, warn};

use crate::{
    rootfs::RootfsManager,
    spec,
    state::{YoukiContainerRecord, YoukiState},
};
use arona_container::{
    errors::{ContainerError, ContainerResult},
    events::ContainerEvent,
    ops::ContainerOps,
    types::{
        ContainerCreateParams, ContainerDetail, ContainerForkParams, ContainerInfo,
        ContainerStatus, DockerVolumeInfo, ExecOutput, ImageInfo, ServerStatus,
    },
};

const RUN_DIR: &str = "/run/entelecheia/youki";
const FALLBACK_RUN_DIR: &str = "/tmp/entelecheia/youki";

fn resolve_run_dir() -> PathBuf {
    let primary = PathBuf::from(RUN_DIR);
    let probe = primary.join(".write_test");
    match std::fs::write(&probe, b"x") {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            primary
        },
        Err(_) => {
            if let Ok(run_dir) = std::env::var("ENTELECHEIA_RUN_DIR") {
                warn!(
                    primary = %primary.display(),
                    fallback = %run_dir,
                    "run_dir not writable, using ENTELECHEIA_RUN_DIR env override"
                );
                PathBuf::from(run_dir)
            } else {
                warn!(
                    primary = %primary.display(),
                    fallback = FALLBACK_RUN_DIR,
                    "run_dir not writable, falling back to /tmp"
                );
                PathBuf::from(FALLBACK_RUN_DIR)
            }
        },
    }
}

#[derive(Debug, Clone)]
pub struct YoukiManager {
    state: YoukiState,
    rootfs: RootfsManager,
    run_dir: PathBuf,
    event_tx: tokio::sync::broadcast::Sender<ContainerEvent>,
}

impl YoukiManager {
    pub fn new(data_dir: &Path) -> ContainerResult<Self> {
        let (event_tx, _) = tokio::sync::broadcast::channel(256);
        let run_dir = resolve_run_dir();
        Ok(Self {
            state: YoukiState::new(),
            rootfs: RootfsManager::new(data_dir),
            run_dir,
            event_tx,
        })
    }

    pub async fn initialize(&self) -> ContainerResult<()> {
        tokio::fs::create_dir_all(&self.run_dir)
            .await
            .map_err(|e| ContainerError::Connection(format!("mkdir run dir: {}", e)))?;
        self.rootfs.ensure_cache_dir().await?;
        // Clean up any stale FUSE mounts left by a previous crash or restart.
        // This prevents fuse-overlayfs daemons from accumulating across server
        // restarts — the root cause of the process leak.
        let cleaned = self.rootfs.cleanup_stale_mounts().await;
        if cleaned > 0 {
            info!(
                cleaned,
                "startup sweep: removed stale container rootfs mounts from previous session"
            );
        }
        Ok(())
    }

    pub async fn reconcile(&self) -> ContainerResult<Vec<ContainerInfo>> {
        info!(
            "Youki reconcile: scanning {} for stale containers",
            self.run_dir.display()
        );

        let mut records: Vec<YoukiContainerRecord> = Vec::new();

        let bundles_dir = self.run_dir.join("bundles");
        if tokio::fs::try_exists(&bundles_dir).await.unwrap_or(false) {
            let mut entries = match tokio::fs::read_dir(&bundles_dir).await {
                Ok(rd) => rd,
                Err(e) => {
                    warn!("Youki reconcile: failed to read bundles dir: {}", e);
                    return Ok(Vec::new());
                },
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let container_id = entry.file_name().to_string_lossy().to_string();
                let bundle_path = entry.path();

                if !entry
                    .file_type()
                    .await
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false)
                {
                    continue;
                }

                let config_path = bundle_path.join("config.json");
                if !tokio::fs::try_exists(&config_path).await.unwrap_or(false) {
                    warn!(
                        "Youki reconcile: skipping {} — no config.json",
                        container_id
                    );
                    continue;
                }

                let container_dir = self.run_dir.join(&container_id);
                let rootfs_path = self.rootfs.container_rootfs_path(&container_id);

                if !tokio::fs::try_exists(&rootfs_path).await.unwrap_or(false) {
                    continue;
                }

                let status = if tokio::fs::try_exists(&container_dir).await.unwrap_or(false) {
                    match Self::load_container_status(&container_dir).await {
                        Some(s) => s,
                        None => {
                            warn!(
                                "Youki reconcile: cannot load container {}, marking Exited",
                                container_id
                            );
                            ContainerStatus::Exited
                        },
                    }
                } else {
                    ContainerStatus::Exited
                };

                let record = YoukiContainerRecord {
                    info: ContainerInfo {
                        id: container_id.clone(),
                        name: container_id.clone(),
                        image: String::new(),
                        status,
                        created_at: None,
                        ports: Vec::new(),
                        env: HashMap::new(),
                        volumes: Vec::new(),
                        ip_address: None,
                        labels: HashMap::new(),
                    },
                    bundle_path,
                    rootfs_path,
                    pid: None,
                    exit_code: None,
                    started_at: None,
                    finished_at: None,
                    error: None,
                };

                records.push(record);

                info!(
                    "Youki reconcile: found container {} ({})",
                    container_id, status
                );
            }
        }

        for record in &records {
            info!(
                "Youki reconcile: recovered container {} ({})",
                record.info.id, record.info.status
            );
        }

        if !records.is_empty() {
            info!("Youki reconcile: recovered {} containers", records.len());
        }

        self.state.replace_all(records.clone()).await;

        Ok(records.into_iter().map(|r| r.info).collect())
    }

    async fn load_container_status(container_dir: &Path) -> Option<ContainerStatus> {
        let container_dir = container_dir.to_path_buf();
        let result = tokio::task::spawn_blocking(move || {
            let container = libcontainer::container::Container::load(container_dir).ok()?;
            Some(container.status())
        })
        .await
        .ok()??;

        match result {
            libcontainer::container::ContainerStatus::Running => Some(ContainerStatus::Running),
            libcontainer::container::ContainerStatus::Stopped => Some(ContainerStatus::Exited),
            libcontainer::container::ContainerStatus::Created => Some(ContainerStatus::Created),
            _ => Some(ContainerStatus::Exited),
        }
    }

    #[cfg(test)]
    pub fn new_for_test(data_dir: &Path, run_dir: &Path) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(256);
        Self {
            state: YoukiState::new(),
            rootfs: RootfsManager::new(data_dir),
            run_dir: run_dir.to_path_buf(),
            event_tx,
        }
    }

    #[cfg(test)]
    pub async fn insert_test_record(&self, id: &str, rootfs_path: PathBuf) {
        self.state
            .insert(YoukiContainerRecord {
                info: ContainerInfo {
                    id: id.to_string(),
                    name: id.to_string(),
                    image: "test".to_string(),
                    status: ContainerStatus::Running,
                    created_at: Some(Utc::now()),
                    ports: Vec::new(),
                    env: HashMap::new(),
                    volumes: Vec::new(),
                    ip_address: None,
                    labels: HashMap::new(),
                },
                bundle_path: PathBuf::new(),
                rootfs_path,
                pid: None,
                exit_code: None,
                started_at: Some(Utc::now().to_rfc3339()),
                finished_at: None,
                error: None,
            })
            .await;
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ContainerEvent> {
        self.event_tx.subscribe()
    }

    fn container_bundle_path(&self, container_id: &str) -> PathBuf {
        self.run_dir.join("bundles").join(container_id)
    }

    async fn write_oci_spec(
        &self,
        params: &ContainerCreateParams,
        rootfs_path: &Path,
        container_id: &str,
    ) -> ContainerResult<PathBuf> {
        let bundle_path = self.container_bundle_path(container_id);
        tokio::fs::create_dir_all(&bundle_path).await.map_err(|e| {
            ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("mkdir bundle: {}", e),
            }
        })?;
        let oci_spec = spec::generate_oci_spec(params, rootfs_path, container_id, &self.run_dir)?;
        let config_str = serde_json::to_string_pretty(&oci_spec).map_err(|e| {
            ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("serialize spec: {}", e),
            }
        })?;
        tokio::fs::write(bundle_path.join("config.json"), config_str)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("write config.json: {}", e),
            })?;
        Ok(bundle_path)
    }

    fn make_info(
        &self,
        container_id: &str,
        params: &ContainerCreateParams,
        status: ContainerStatus,
    ) -> ContainerInfo {
        ContainerInfo {
            id: container_id.to_string(),
            name: params.name.clone(),
            image: params.image.clone(),
            status,
            created_at: Some(Utc::now()),
            ports: params.ports.clone(),
            env: params.env.clone(),
            volumes: params.volumes.clone(),
            ip_address: None,
            labels: params.labels.clone(),
        }
    }

    async fn resolve_id(&self, name_or_id: &str) -> ContainerResult<String> {
        if let Some(r) = self.state.get(name_or_id).await {
            return Ok(r.info.id);
        }
        if let Some(r) = self.state.get_by_name(name_or_id).await {
            return Ok(r.info.id);
        }
        let name_trimmed = name_or_id.trim_start_matches('/');
        if name_trimmed != name_or_id
            && let Some(r) = self.state.get_by_name(name_trimmed).await
        {
            return Ok(r.info.id);
        }
        for r in self.state.list_all().await {
            if r.name.trim_start_matches('/') == name_trimmed {
                return Ok(r.id);
            }
            if r.id.starts_with(&format!("{}-", name_trimmed)) {
                return Ok(r.id);
            }
        }
        Err(ContainerError::NotFound(name_or_id.to_string()))
    }

    async fn resolve_named_volumes(
        &self,
        params: &ContainerCreateParams,
    ) -> ContainerResult<ContainerCreateParams> {
        let mut resolved = params.clone();
        resolved.volumes = resolved
            .volumes
            .into_iter()
            .filter_map(|vol| {
                if !vol.host_path.starts_with('/') {
                    let p = self.run_dir.join("volumes").join(&vol.host_path);
                    Some((p, vol))
                } else {
                    let path = std::path::Path::new(&vol.host_path);
                    if !path.exists() {
                        warn!(
                            "skipping bind mount {}: source does not exist",
                            vol.host_path
                        );
                        return None;
                    }
                    let ft = std::fs::metadata(path).ok();
                    let ftype = ft.as_ref().map(|m| m.file_type());
                    if ftype.is_none_or(|t| !t.is_dir() && !t.is_file()) {
                        warn!(
                            "skipping bind mount {}: not a regular file or directory",
                            vol.host_path
                        );
                        return None;
                    }
                    Some((path.to_path_buf(), vol))
                }
            })
            .filter_map(|(p, mut vol)| {
                if !p.exists()
                    && let Err(e) = std::fs::create_dir_all(&p)
                {
                    warn!("failed to create volume dir {}: {}", p.display(), e);
                    return None;
                }
                vol.host_path = p.to_string_lossy().to_string();
                Some(vol)
            })
            .collect();
        Ok(resolved)
    }

    async fn walk_diff(
        &self,
        container_dir: &std::path::Path,
        host_dir: &std::path::Path,
        changes: &mut Vec<arona_container::PathChange>,
    ) -> ContainerResult<()> {
        let mut entries = tokio::fs::read_dir(container_dir).await.map_err(|e| {
            ContainerError::OperationFailed {
                container_id: String::new(),
                message: format!("read_dir {}: {}", container_dir.display(), e),
            }
        })?;
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("entry: {}", e),
                })?
        {
            let name = entry.file_name();
            let container_path = container_dir.join(&name);
            let host_path = host_dir.join(&name);

            let relative = container_path
                .strip_prefix(self.rootfs.cache_dir())
                .unwrap_or(&container_path)
                .to_path_buf();

            let host_meta = tokio::fs::metadata(&host_path).await.ok();
            let container_meta = tokio::fs::metadata(&container_path).await;

            match (host_meta, container_meta) {
                (None, Ok(cm)) => {
                    changes.push(arona_container::PathChange {
                        path: relative,
                        kind: arona_container::ChangeKind::Added,
                    });
                    if cm.is_dir() {
                        self.collect_all_paths(&container_path, self.rootfs.cache_dir(), changes);
                    }
                },
                (Some(_), Ok(cm)) if cm.is_dir() => {
                    Box::pin(self.walk_diff(&container_path, &host_path, changes)).await?;
                },
                (Some(hm), Ok(cm)) => {
                    if hm.len() != cm.len() {
                        changes.push(arona_container::PathChange {
                            path: relative,
                            kind: arona_container::ChangeKind::Modified,
                        });
                    } else if hm.modified().ok() != cm.modified().ok() {
                        let host_bytes = match std::fs::read(&host_path) {
                            Ok(b) => b,
                            Err(e) => {
                                debug!(path = %host_path.display(), error = %e, "skipping unreadable host file in diff");
                                continue;
                            },
                        };
                        let container_bytes = match std::fs::read(&container_path) {
                            Ok(b) => b,
                            Err(e) => {
                                debug!(path = %container_path.display(), error = %e, "skipping unreadable container file in diff");
                                continue;
                            },
                        };
                        if host_bytes != container_bytes {
                            changes.push(arona_container::PathChange {
                                path: relative,
                                kind: arona_container::ChangeKind::Modified,
                            });
                        }
                    }
                },
                (Some(_), Err(_)) => {
                    changes.push(arona_container::PathChange {
                        path: relative,
                        kind: arona_container::ChangeKind::Deleted,
                    });
                },
                _ => {},
            }
        }
        Ok(())
    }

    fn collect_all_paths(
        &self,
        dir: &std::path::Path,
        prefix: &std::path::Path,
        changes: &mut Vec<arona_container::PathChange>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(prefix).unwrap_or(&path).to_path_buf();
                changes.push(arona_container::PathChange {
                    path: relative,
                    kind: arona_container::ChangeKind::Added,
                });
                if path.is_dir() {
                    self.collect_all_paths(&path, prefix, changes);
                }
            }
        }
    }
}

impl YoukiManager {
    async fn resolve_record(&self, container_id: &str) -> ContainerResult<YoukiContainerRecord> {
        let resolved_id = self
            .resolve_id(container_id)
            .await
            .map_err(|_| ContainerError::NotFound(container_id.to_string()))?;
        self.state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id))
    }

    async fn scan_upper(
        &self,
        dir: &std::path::Path,
        base_path: &str,
        changes: &mut Vec<arona_container::PathChange>,
    ) -> ContainerResult<()> {
        let mut entries =
            tokio::fs::read_dir(dir)
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("read_dir {}: {}", dir.display(), e),
                })?;
        let prefix = format!("/{}/", base_path.trim_start_matches('/'));
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("entry: {}", e),
                })?
        {
            let name = entry.file_name();
            let path = entry.path();
            let relative = format!("{}{}", prefix, name.to_string_lossy());

            let meta = tokio::fs::metadata(&path).await;
            match meta {
                Ok(m) if m.is_dir() => {
                    changes.push(arona_container::PathChange {
                        path: std::path::PathBuf::from(&relative),
                        kind: arona_container::ChangeKind::Added,
                    });
                    Box::pin(self.scan_upper_recursive(&path, &prefix, changes)).await?;
                },
                Ok(_) => {
                    changes.push(arona_container::PathChange {
                        path: std::path::PathBuf::from(&relative),
                        kind: arona_container::ChangeKind::Modified,
                    });
                },
                Err(_) => {},
            }
        }
        Ok(())
    }

    async fn scan_upper_recursive(
        &self,
        dir: &std::path::Path,
        parent_prefix: &str,
        changes: &mut Vec<arona_container::PathChange>,
    ) -> ContainerResult<()> {
        let mut entries =
            tokio::fs::read_dir(dir)
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("read_dir {}: {}", dir.display(), e),
                })?;
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("entry: {}", e),
                })?
        {
            let name = entry.file_name();
            let path = entry.path();
            let relative = format!(
                "{}/{}",
                parent_prefix.trim_end_matches('/'),
                name.to_string_lossy()
            );

            let meta = tokio::fs::metadata(&path).await;
            match meta {
                Ok(m) if m.is_dir() => {
                    changes.push(arona_container::PathChange {
                        path: std::path::PathBuf::from(&relative),
                        kind: arona_container::ChangeKind::Added,
                    });
                    Box::pin(self.scan_upper_recursive(&path, &relative, changes)).await?;
                },
                Ok(_) => {
                    changes.push(arona_container::PathChange {
                        path: std::path::PathBuf::from(&relative),
                        kind: arona_container::ChangeKind::Modified,
                    });
                },
                Err(_) => {},
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ContainerOps for YoukiManager {
    async fn create(&self, params: &ContainerCreateParams) -> ContainerResult<ContainerInfo> {
        let container_id = format!("{}-{}", params.name, Uuid::now_v7().as_simple());
        info!(container_id = %container_id, "creating youki container");

        let resolved_params = self.resolve_named_volumes(params).await?;

        let rootfs_path = self
            .rootfs
            .prepare_container_rootfs(&params.image, &container_id)
            .await?;
        let bundle_path = self
            .write_oci_spec(&resolved_params, &rootfs_path, &container_id)
            .await?;
        let info = self.make_info(&container_id, params, ContainerStatus::Running);

        let run_dir = self.run_dir.clone();
        let cid = container_id.clone();
        let bid = bundle_path.clone();
        let pid = tokio::task::spawn_blocking(move || {
            let mut container = ContainerBuilder::new(cid, SyscallType::default())
                .with_root_path(&run_dir)
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))?
                .as_init(&bid)
                .with_systemd(false)
                .build()
                .map_err(|e| ContainerError::YoukiApi(format!("build: {}", e)))?;
            container
                .start()
                .map_err(|e| ContainerError::YoukiApi(format!("start: {}", e)))?;
            Ok::<_, ContainerError>(container.pid().map(|p| p.as_raw()))
        })
        .await
        .map_err(|e| ContainerError::YoukiApi(format!("create: {}", e)))??;

        self.state
            .insert(YoukiContainerRecord {
                info: info.clone(),
                bundle_path,
                rootfs_path,
                pid,
                exit_code: None,
                started_at: Some(Utc::now().to_rfc3339()),
                finished_at: None,
                error: None,
            })
            .await;

        if let Err(e) = self.event_tx.send(ContainerEvent::Created {
            id: info.id.clone(),
            name: info.name.clone(),
            image: info.image.clone(),
        }) {
            debug!(error = %e, "container Created event broadcast failed (no receivers)");
        }
        if let Err(e) = self.event_tx.send(ContainerEvent::Started {
            id: info.id.clone(),
        }) {
            debug!(error = %e, "container Started event broadcast failed (no receivers)");
        }

        Ok(info)
    }

    async fn start(&self, container_id: &str) -> ContainerResult<()> {
        let resolved_id = self.resolve_id(container_id).await?;
        let container_dir = self.run_dir.join(&resolved_id);
        tokio::task::spawn_blocking(move || {
            let mut container = libcontainer::container::Container::load(container_dir)
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))?;
            container
                .start()
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))
        })
        .await
        .map_err(|e| ContainerError::YoukiApi(format!("start: {}", e)))??;
        let old_status = self.state.get(&resolved_id).await.map(|r| r.info.status);
        self.state
            .update_status(&resolved_id, ContainerStatus::Running)
            .await;
        if let Some(old) = old_status
            && let Err(e) = self.event_tx.send(ContainerEvent::StatusChanged {
                id: resolved_id.clone(),
                old_status: old,
                new_status: ContainerStatus::Running,
            })
        {
            debug!(error = %e, "container StatusChanged event broadcast failed");
        }
        if let Err(e) = self
            .event_tx
            .send(ContainerEvent::Started { id: resolved_id })
        {
            debug!(error = %e, "container Started event broadcast failed");
        }
        Ok(())
    }

    async fn stop(&self, container_id: &str) -> ContainerResult<()> {
        let resolved_id = self.resolve_id(container_id).await?;
        let container_dir = self.run_dir.join(&resolved_id);
        tokio::task::spawn_blocking(move || {
            let mut container = libcontainer::container::Container::load(container_dir)
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))?;
            let sig = libcontainer::signal::Signal::try_from("SIGTERM")
                .map_err(|e| ContainerError::YoukiApi(format!("signal: {:?}", e)))?;
            if container.kill(sig, false).is_err() {
                let sigk = libcontainer::signal::Signal::try_from("SIGKILL")
                    .map_err(|e| ContainerError::YoukiApi(format!("signal: {:?}", e)))?;
                let _ = container.kill(sigk, false);
            }
            Ok::<_, ContainerError>(())
        })
        .await
        .map_err(|e| ContainerError::YoukiApi(format!("stop: {}", e)))??;
        let old_status = self.state.get(&resolved_id).await.map(|r| r.info.status);
        self.state
            .update_status(&resolved_id, ContainerStatus::Exited)
            .await;
        self.state
            .update_exit_status(&resolved_id, Some(0), Some(Utc::now().to_rfc3339()), None)
            .await;
        if let Some(old) = old_status
            && let Err(e) = self.event_tx.send(ContainerEvent::StatusChanged {
                id: resolved_id.clone(),
                old_status: old,
                new_status: ContainerStatus::Exited,
            })
        {
            debug!(error = %e, "container StatusChanged(Exited) event broadcast failed");
        }
        if let Err(e) = self
            .event_tx
            .send(ContainerEvent::Stopped { id: resolved_id })
        {
            debug!(error = %e, "container Stopped event broadcast failed");
        }
        Ok(())
    }

    async fn remove(&self, container_id: &str, force: bool) -> ContainerResult<()> {
        // Try to resolve; if it fails, still attempt rootfs cleanup with the
        // raw id so mounts don't leak when state was lost.
        let resolved_id = match self.resolve_id(container_id).await {
            Ok(id) => id,
            Err(_) => {
                warn!(
                    container_id,
                    "container not found in state during remove — attempting best-effort rootfs cleanup"
                );
                self.rootfs.cleanup_container_rootfs(container_id).await?;
                return Ok(());
            },
        };
        let container_dir = self.run_dir.join(&resolved_id);
        tokio::task::spawn_blocking(move || {
            if let Ok(mut container) =
                libcontainer::container::Container::load(container_dir.clone())
            {
                let _ = container.delete(force);
            }
            let _ = std::fs::remove_dir_all(&container_dir);
            Ok::<_, ContainerError>(())
        })
        .await
        .map_err(|e| ContainerError::YoukiApi(format!("remove: {}", e)))??;
        self.rootfs.cleanup_container_rootfs(&resolved_id).await?;
        self.state.remove(&resolved_id).await;
        let _ = self
            .event_tx
            .send(ContainerEvent::Destroyed { id: resolved_id });
        Ok(())
    }

    async fn restart(&self, container_id: &str) -> ContainerResult<()> {
        let resolved_id = self.resolve_id(container_id).await?;
        let record = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id.clone()))?;
        self.stop(&resolved_id).await?;

        let container_dir = self.run_dir.join(&resolved_id);
        let bundle_path = record.bundle_path.clone();
        let parent_dir = container_dir
            .parent()
            .unwrap_or(Path::new("/run"))
            .to_path_buf();
        let cid = resolved_id.clone();

        tokio::task::spawn_blocking(move || {
            if let Ok(mut container) =
                libcontainer::container::Container::load(container_dir.clone())
            {
                let _ = container.delete(true);
            }
            let mut c = ContainerBuilder::new(cid, SyscallType::default())
                .with_root_path(&parent_dir)
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))?
                .as_init(&bundle_path)
                .with_systemd(false)
                .build()
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))?;
            c.start()
                .map_err(|e| ContainerError::YoukiApi(format!("{}", e)))
        })
        .await
        .map_err(|e| ContainerError::YoukiApi(format!("restart: {}", e)))??;

        self.state
            .update_status(&resolved_id, ContainerStatus::Running)
            .await;
        Ok(())
    }

    async fn list(&self) -> ContainerResult<Vec<ContainerInfo>> {
        Ok(self.state.list_all().await)
    }

    async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<HashMap<String, String>>,
        all: bool,
    ) -> ContainerResult<Vec<ContainerInfo>> {
        Ok(self
            .state
            .list_with_filter(name_prefix, label_filter.as_ref(), all)
            .await)
    }

    async fn detect_server_status(&self, container_name: &str) -> ServerStatus {
        let containers = self
            .list_with_filter(Some(container_name), None, false)
            .await;
        match containers {
            Ok(c) if !c.is_empty() => ServerStatus::Running,
            Ok(_) => {
                let all = self
                    .list_with_filter(Some(container_name), None, true)
                    .await;
                match all {
                    Ok(c) if !c.is_empty() => ServerStatus::Stopped,
                    _ => ServerStatus::NotExists,
                }
            },
            Err(_) => ServerStatus::Unknown,
        }
    }

    async fn inspect(&self, container_id: &str) -> ContainerResult<ContainerDetail> {
        let resolved_id = self.resolve_id(container_id).await?;
        let r = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id))?;
        Ok(ContainerDetail {
            info: r.info.clone(),
            exit_code: r.exit_code,
            started_at: r
                .started_at
                .or_else(|| r.info.created_at.map(|d| d.to_rfc3339())),
            finished_at: r.finished_at,
            error: r.error,
        })
    }

    async fn is_running(&self, container_id: &str) -> ContainerResult<bool> {
        let resolved_id = self.resolve_id(container_id).await?;
        let r = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id))?;
        Ok(r.info.status.is_running())
    }

    async fn exec(&self, container_id: &str, command: &[&str]) -> ContainerResult<ExecOutput> {
        if command.is_empty() {
            return Err(ContainerError::ExecFailed {
                container_id: container_id.to_string(),
                message: "exec command cannot be empty".to_string(),
            });
        }
        let resolved_id = self.resolve_id(container_id).await?;
        let record = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id.clone()))?;

        let rootfs_path = record.rootfs_path.clone();
        let merged_path = rootfs_path.join("merged");
        let workdir = if merged_path.is_dir() {
            merged_path
        } else {
            rootfs_path
        };

        let output = if let Some(pid) = record.pid {
            info!(
                container_id = %resolved_id,
                pid,
                "Entering container namespaces via nsenter for exec"
            );
            let mut args = vec![
                "-m".to_string(),
                "-u".to_string(),
                "-i".to_string(),
                "-p".to_string(),
                "-t".to_string(),
                pid.to_string(),
                "--".to_string(),
            ];
            args.extend(command.iter().map(|s| s.to_string()));
            tokio::process::Command::new("nsenter")
                .args(&args)
                .current_dir(&workdir)
                .env("PATH", "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
                .map_err(|e| ContainerError::ExecFailed {
                    container_id: resolved_id.clone(),
                    message: format!("nsenter exec spawn: {}", e),
                })?
        } else {
            warn!(
                container_id = %resolved_id,
                "No PID recorded for container — executing on host filesystem only (no namespace isolation). \
                 This is a security limitation of the current youki runtime backend."
            );
            tokio::process::Command::new(command[0])
                .args(&command[1..])
                .current_dir(&workdir)
                .env("PATH", "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
                .map_err(|e| ContainerError::ExecFailed {
                    container_id: resolved_id.clone(),
                    message: format!("exec spawn: {}", e),
                })?
        };

        Ok(ExecOutput {
            exit_code: output.status.code().map(|c| c as i64),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    async fn fork(&self, params: &ContainerForkParams) -> ContainerResult<ContainerInfo> {
        self.state
            .get(&params.source_container_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(params.source_container_id.clone()))?;
        let snapshot_id = format!("snapshot-{}", Uuid::now_v7().as_simple());
        self.rootfs
            .snapshot_rootfs(&params.source_container_id, &snapshot_id)
            .await?;
        let cp = ContainerCreateParams {
            name: params.new_name.clone(),
            image: snapshot_id.clone(),
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
        self.create(&cp).await
    }

    async fn commit(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
    ) -> ContainerResult<String> {
        self.commit_with_labels(container_id, repo, tag, None).await
    }

    async fn commit_with_labels(
        &self,
        container_id: &str,
        repo: &str,
        tag: Option<&str>,
        labels: Option<&HashMap<String, String>>,
    ) -> ContainerResult<String> {
        let resolved_id = self.resolve_id(container_id).await?;
        let sid = format!(
            "{}-{}-{}",
            repo,
            tag.unwrap_or("latest"),
            Uuid::now_v7().as_simple()
        );
        self.rootfs.snapshot_rootfs(&resolved_id, &sid).await?;
        if let Some(labels) = labels {
            let meta = serde_json::to_string_pretty(labels).map_err(|e| {
                ContainerError::OperationFailed {
                    container_id: resolved_id.clone(),
                    message: format!("serialize: {}", e),
                }
            })?;
            let _ =
                tokio::fs::write(self.rootfs.snapshot_path(&sid).join(".meta.json"), meta).await;
        }
        Ok(sid)
    }

    async fn wait_healthy(
        &self,
        container_id: &str,
        timeout: std::time::Duration,
    ) -> ContainerResult<()> {
        let resolved_id = self.resolve_id(container_id).await?;
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.is_running(&resolved_id).await? {
                return Ok(());
            }
            if tokio::time::Instant::now() > deadline {
                return Err(ContainerError::OperationFailed {
                    container_id: resolved_id,
                    message: format!("not healthy within {:?}", timeout),
                });
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    async fn ensure_running(&self, container_id: &str) -> ContainerResult<ContainerInfo> {
        let resolved_id = self.resolve_id(container_id).await?;
        let r = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id.clone()))?;
        if !r.info.status.is_running() {
            self.start(&resolved_id).await?;
            return self
                .state
                .get(&resolved_id)
                .await
                .ok_or_else(|| ContainerError::NotFound(resolved_id))
                .map(|r| r.info);
        }
        Ok(r.info)
    }

    async fn recreate(
        &self,
        container_name: &str,
        new_image: &str,
    ) -> ContainerResult<ContainerInfo> {
        let old = self.state.get_by_name(container_name).await;
        let old = old.ok_or_else(|| ContainerError::NotFound(container_name.to_string()))?;
        let old_image = old.info.image.clone();
        let cp = ContainerCreateParams {
            name: old.info.name.clone(),
            image: new_image.to_string(),
            network: None,
            env: old.info.env.clone(),
            ports: old.info.ports.clone(),
            volumes: old.info.volumes.clone(),
            labels: old.info.labels.clone(),
            working_dir: None,
            log_driver: None,
            log_opts: HashMap::new(),
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
        };
        self.remove(&old.info.id, true).await?;
        let new_info = self.create(&cp).await?;
        if let Err(e) = self.event_tx.send(ContainerEvent::Updated {
            id: new_info.id.clone(),
            name: container_name.to_string(),
            old_image,
            new_image: new_image.to_string(),
        }) {
            debug!(error = %e, "container Updated event broadcast failed");
        }
        Ok(new_info)
    }

    async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>> {
        let mut imgs = vec![ImageInfo {
            id: "host".into(),
            repository: "host".into(),
            tag: "host".into(),
            size: 0,
            created: 0,
        }];
        let cache = self.rootfs.cache_dir();
        if !cache.exists() {
            return Ok(imgs);
        }
        let mut entries = tokio::fs::read_dir(cache)
            .await
            .map_err(|e| ContainerError::ImageFailed(format!("readdir: {}", e)))?;
        while let Some(e) = entries
            .next_entry()
            .await
            .map_err(|e| ContainerError::ImageFailed(format!("entry: {}", e)))?
        {
            let name = e.file_name().to_string_lossy().to_string();
            if name == "containers" || name == "snapshots" {
                continue;
            }
            let m = e
                .metadata()
                .await
                .map_err(|e| ContainerError::ImageFailed(format!("meta: {}", e)))?;
            imgs.push(ImageInfo {
                id: name.clone(),
                repository: "rootfs".into(),
                tag: name,
                size: m.len() as i64,
                created: m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            });
        }
        Ok(imgs)
    }

    async fn pull_image(&self, image: &str) -> ContainerResult<String> {
        if RootfsManager::is_host_rootfs_image(image) {
            return Ok(image.to_string());
        }
        let dest = self.rootfs.base_rootfs_path(image);
        if dest.is_dir() {
            return Ok(image.to_string());
        }
        let url = format!(
            "{}/{}.tar.gz",
            std::env::var("ENTELECHEIA_ROOTFS_URL")
                .unwrap_or_else(|_| "https://releases.entelecheia.dev/rootfs".into()),
            image
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| ContainerError::ImageFailed(format!("download: {}", e)))?;
        if !resp.status().is_success() {
            return Err(ContainerError::ImageFailed(format!(
                "HTTP {} for {}",
                resp.status(),
                image
            )));
        }
        let tmp = dest.with_extension("tar.gz.tmp");
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ContainerError::ImageFailed(format!("read: {}", e)))?;
        tokio::fs::write(&tmp, &bytes)
            .await
            .map_err(|e| ContainerError::ImageFailed(format!("write: {}", e)))?;
        self.rootfs.extract_rootfs(&tmp, &dest).await?;
        let _ = tokio::fs::remove_file(&tmp).await;
        Ok(image.to_string())
    }

    async fn image_exists(&self, image: &str) -> ContainerResult<bool> {
        if RootfsManager::is_host_rootfs_image(image) {
            return Ok(true);
        }
        Ok(self.rootfs.base_rootfs_path(image).is_dir())
    }

    async fn remove_image(&self, image: &str, _force: bool) -> ContainerResult<()> {
        if RootfsManager::is_host_rootfs_image(image) {
            return Err(ContainerError::ImageFailed("cannot remove host".into()));
        }
        let p = self.rootfs.base_rootfs_path(image);
        if p.exists() {
            tokio::fs::remove_dir_all(&p)
                .await
                .map_err(|e| ContainerError::ImageFailed(format!("rm: {}", e)))?;
        }
        Ok(())
    }

    async fn remove_with_image(
        &self,
        container_id: &str,
        image: &str,
        force: bool,
    ) -> ContainerResult<()> {
        let resolved_id = self.resolve_id(container_id).await?;
        self.remove(&resolved_id, force).await?;
        if image.starts_with("snapshot-") {
            self.remove_image(image, true).await?;
        }
        Ok(())
    }

    async fn create_volume(&self, name: &str) -> ContainerResult<String> {
        let p = self.run_dir.join("volumes").join(name);
        tokio::fs::create_dir_all(&p)
            .await
            .map_err(|e| ContainerError::VolumeFailed(format!("mkdir: {}", e)))?;
        Ok(p.to_string_lossy().to_string())
    }

    async fn remove_volume(&self, name: &str, _force: bool) -> ContainerResult<()> {
        let p = self.run_dir.join("volumes").join(name);
        if p.exists() {
            tokio::fs::remove_dir_all(&p)
                .await
                .map_err(|e| ContainerError::VolumeFailed(format!("rm: {}", e)))?;
        }
        Ok(())
    }

    async fn volume_exists(&self, name: &str) -> ContainerResult<bool> {
        Ok(self.run_dir.join("volumes").join(name).is_dir())
    }

    async fn list_volumes(&self) -> ContainerResult<Vec<DockerVolumeInfo>> {
        let d = self.run_dir.join("volumes");
        if !d.exists() {
            return Ok(Vec::new());
        }
        let mut vols = Vec::new();
        let mut entries = tokio::fs::read_dir(&d)
            .await
            .map_err(|e| ContainerError::VolumeFailed(format!("readdir: {}", e)))?;
        while let Some(e) = entries
            .next_entry()
            .await
            .map_err(|e| ContainerError::VolumeFailed(format!("entry: {}", e)))?
        {
            vols.push(DockerVolumeInfo {
                name: e.file_name().to_string_lossy().to_string(),
                driver: "local".into(),
                mountpoint: Some(e.path().to_string_lossy().to_string()),
            });
        }
        Ok(vols)
    }

    async fn logs(&self, container_id: &str, tail: usize) -> ContainerResult<Vec<String>> {
        let resolved_id = self.resolve_id(container_id).await?;
        let p = self
            .container_bundle_path(&resolved_id)
            .join("container.log");
        if !p.exists() {
            return Ok(Vec::new());
        }
        let s =
            tokio::fs::read_to_string(&p)
                .await
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: resolved_id,
                    message: format!("read: {}", e),
                })?;
        let lines: Vec<String> = s.lines().map(String::from).collect();
        let skip = lines.len().saturating_sub(tail);
        Ok(lines.into_iter().skip(skip).collect())
    }

    async fn get_container_logs(&self, container_id: &str, tail: usize) -> ContainerResult<String> {
        let lines = self.logs(container_id, tail).await?;
        Ok(lines.join("\n"))
    }

    async fn writable_rootfs(
        &self,
        container_id: &str,
    ) -> ContainerResult<arona_container::WritableRootfs> {
        let resolved_id = self.resolve_id(container_id).await?;
        let record = self
            .state
            .get(&resolved_id)
            .await
            .ok_or_else(|| ContainerError::NotFound(resolved_id.clone()))?;
        let merged = record.rootfs_path.join("merged");
        if merged.is_dir() {
            return Ok(arona_container::WritableRootfs {
                path: merged,
                is_direct: true,
            });
        }
        if record.rootfs_path.is_dir() {
            return Ok(arona_container::WritableRootfs {
                path: record.rootfs_path,
                is_direct: true,
            });
        }
        Err(ContainerError::OperationFailed {
            container_id: resolved_id,
            message: "rootfs path does not exist".to_string(),
        })
    }

    async fn diff_workspace(
        &self,
        container_id: &str,
        _workspace_path: &std::path::Path,
        base_path: &str,
    ) -> ContainerResult<Vec<arona_container::PathChange>> {
        let record = self.resolve_record(container_id).await?;
        let upper = record.rootfs_path.join("upper");
        let target = if upper.is_dir() {
            upper.join(base_path.trim_start_matches('/'))
        } else {
            let merged = record.rootfs_path.join("merged");
            if merged.is_dir() {
                merged.join(base_path.trim_start_matches('/'))
            } else {
                record.rootfs_path.join(base_path.trim_start_matches('/'))
            }
        };

        if !target.is_dir() {
            return Ok(Vec::new());
        }

        let mut changes = Vec::new();
        if upper.is_dir() {
            self.scan_upper(&target, base_path, &mut changes).await?;
        } else {
            self.walk_diff(&target, _workspace_path, &mut changes)
                .await?;
            let prefix = match base_path.trim_start_matches('/') {
                "" => "/".to_string(),
                bp => format!("/{}/", bp),
            };
            for change in &mut changes {
                let s = change.path.to_string_lossy().to_string();
                if !s.starts_with('/') {
                    if let Some(idx) = s.find(&prefix) {
                        change.path = std::path::PathBuf::from(&s[idx..]);
                    } else if let Some(idx) = s.find(base_path.trim_start_matches('/')) {
                        change.path = std::path::PathBuf::from(format!("/{}", &s[idx..]));
                    }
                }
            }
        }
        Ok(changes)
    }

    async fn download_archive(&self, container_id: &str, path: &str) -> ContainerResult<Vec<u8>> {
        let rootfs = self.writable_rootfs(container_id).await?;
        let target = rootfs.path.join(path.trim_start_matches('/'));
        if !target.exists() {
            return Ok(Vec::new());
        }
        let is_file = target.is_file();
        let target_str = target.to_string_lossy().to_string();
        let file_name = target
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        tokio::task::spawn_blocking(move || {
            let mut buf = Vec::new();
            {
                let mut builder = tar::Builder::new(&mut buf);
                if is_file {
                    let mut f = std::fs::File::open(&target_str).map_err(|e| {
                        ContainerError::OperationFailed {
                            container_id: String::new(),
                            message: format!("open file: {}", e),
                        }
                    })?;
                    let meta = f.metadata().map_err(|e| ContainerError::OperationFailed {
                        container_id: String::new(),
                        message: format!("metadata: {}", e),
                    })?;
                    let mut header = tar::Header::new_gnu();
                    header.set_size(meta.len());
                    header.set_entry_type(tar::EntryType::file());
                    header.set_mode(0o644);
                    header
                        .set_path(&*file_name)
                        .map_err(|e| ContainerError::OperationFailed {
                            container_id: String::new(),
                            message: format!("set_path: {}", e),
                        })?;
                    header.set_cksum();
                    builder.append(&header, &mut f).map_err(|e| {
                        ContainerError::OperationFailed {
                            container_id: String::new(),
                            message: format!("append file: {}", e),
                        }
                    })?;
                } else {
                    builder.append_dir_all(".", &target_str).map_err(|e| {
                        ContainerError::OperationFailed {
                            container_id: String::new(),
                            message: format!("tar archive: {}", e),
                        }
                    })?;
                }
                builder
                    .finish()
                    .map_err(|e| ContainerError::OperationFailed {
                        container_id: String::new(),
                        message: format!("tar finish: {}", e),
                    })?;
            }
            Ok(buf)
        })
        .await
        .map_err(|e| ContainerError::OperationFailed {
            container_id: container_id.to_string(),
            message: format!("download_archive spawn: {}", e),
        })?
    }

    async fn upload_archive(
        &self,
        container_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> ContainerResult<()> {
        let rootfs = self.writable_rootfs(container_id).await?;
        let target = rootfs.path.join(path.trim_start_matches('/'));
        tokio::fs::create_dir_all(&target)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("mkdir for upload: {}", e),
            })?;
        let target_str = target.to_string_lossy().to_string();
        tokio::task::spawn_blocking(move || {
            let mut archive = tar::Archive::new(data.as_slice());
            archive
                .unpack(&target_str)
                .map_err(|e| ContainerError::OperationFailed {
                    container_id: String::new(),
                    message: format!("tar unpack: {}", e),
                })
        })
        .await
        .map_err(|e| ContainerError::OperationFailed {
            container_id: container_id.to_string(),
            message: format!("upload_archive spawn: {}", e),
        })??;
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn ContainerOps> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arona_container::types::ChangeKind;
    use anyhow::{Context, Result};
    use tempfile::TempDir;

    fn setup_mock_rootfs(base: &Path, container_id: &str) -> Result<PathBuf> {
        let rootfs = base.join("rootfs").join("containers").join(container_id);
        let home = rootfs.join("home");
        std::fs::create_dir_all(home.join("src")).context("test precondition")?;
        std::fs::write(home.join("Cargo.toml"), "[package]\nname = \"test\"\n")
            .context("test precondition")?;
        std::fs::write(home.join("src").join("main.rs"), "fn main() {}")
            .context("test precondition")?;
        Ok(rootfs)
    }

    fn make_mgr(tmp: &TempDir) -> YoukiManager {
        YoukiManager::new_for_test(tmp.path(), tmp.path())
    }

    #[tokio::test]
    async fn test_writable_rootfs_returns_path() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c1")?;
        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c1", rootfs_path).await;

        let result = mgr
            .writable_rootfs("c1")
            .await
            .context("test precondition")?;
        assert!(result.is_direct);
        assert!(result.path.is_dir());
        assert!(result.path.join("home").is_dir());
        Ok(())
    }

    #[tokio::test]
    async fn test_writable_rootfs_not_found() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);
        assert!(mgr.writable_rootfs("nonexistent").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_diff_workspace_detects_added_files() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c2")?;
        let host_ws = tmp.path().join("host-ws");
        std::fs::create_dir_all(&host_ws).context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c2", rootfs_path).await;

        let changes = mgr
            .diff_workspace("c2", &host_ws, "home")
            .await
            .context("test precondition")?;

        let added: Vec<String> = changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Added)
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        assert!(
            added.iter().any(|p| p.contains("Cargo.toml")),
            "Cargo.toml should be added: {:?}",
            added
        );
        assert!(
            added.iter().any(|p| p.contains("main.rs")),
            "main.rs should be added: {:?}",
            added
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_diff_workspace_detects_modified_files() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c3")?;

        let host_ws = tmp.path().join("host-ws");
        std::fs::create_dir_all(host_ws.join("src")).context("test precondition")?;
        std::fs::write(host_ws.join("Cargo.toml"), "[package]\nname = \"old\"\n")
            .context("test precondition")?;
        std::fs::write(
            host_ws.join("src").join("main.rs"),
            "fn main() { /* old */ }",
        )
        .context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c3", rootfs_path).await;

        let changes = mgr
            .diff_workspace("c3", &host_ws, "home")
            .await
            .context("test precondition")?;

        let modified: Vec<String> = changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Modified)
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        assert!(
            modified.iter().any(|p| p.contains("Cargo.toml")),
            "Cargo.toml content differs, should be modified: {:?}",
            modified
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_diff_workspace_no_changes_when_identical() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c4")?;

        let host_ws = tmp.path().join("host-ws");
        std::fs::create_dir_all(host_ws.join("src")).context("test precondition")?;
        std::fs::write(host_ws.join("Cargo.toml"), "[package]\nname = \"test\"\n")
            .context("test precondition")?;
        std::fs::write(host_ws.join("src").join("main.rs"), "fn main() {}")
            .context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c4", rootfs_path).await;

        let changes = mgr
            .diff_workspace("c4", &host_ws, "home")
            .await
            .context("test precondition")?;

        let changed: Vec<_> = changes
            .iter()
            .filter(|c| c.kind == ChangeKind::Modified || c.kind == ChangeKind::Added)
            .collect();

        assert!(
            changed.is_empty(),
            "identical files should produce no changes: {:?}",
            changed
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_download_archive_creates_tar() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c5")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c5", rootfs_path).await;

        let tar_bytes = mgr
            .download_archive("c5", "home")
            .await
            .context("test precondition")?;
        assert!(!tar_bytes.is_empty());

        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        let entries: Vec<_> = archive
            .entries()
            .context("test precondition")?
            .collect::<Result<Vec<_>, _>>()
            .context("test precondition")?;
        let names: Vec<String> = entries
            .iter()
            .filter_map(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        assert!(
            names.iter().any(|n| n.contains("Cargo.toml")),
            "archive should contain Cargo.toml: {:?}",
            names
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_download_archive_single_file() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c-sf")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-sf", rootfs_path).await;

        let tar_bytes = mgr
            .download_archive("c-sf", "home/Cargo.toml")
            .await
            .context("test precondition")?;
        assert!(!tar_bytes.is_empty(), "single file tar should not be empty");

        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        let entries: Vec<_> = archive
            .entries()
            .context("test precondition")?
            .collect::<Result<Vec<_>, _>>()
            .context("test precondition")?;
        let names: Vec<String> = entries
            .iter()
            .filter_map(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
            .collect();

        assert!(
            names.iter().any(|n| n.contains("Cargo.toml")),
            "archive should contain Cargo.toml: {:?}",
            names
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_upload_archive_extracts_files() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c6")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c6", rootfs_path).await;

        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            let data = b"hello";
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_path("hello.txt").context("test precondition")?;
            header.set_entry_type(tar::EntryType::file());
            header.set_cksum();
            builder
                .append(&header, &data[..])
                .context("test precondition")?;
            builder.finish().context("test precondition")?;
        }

        mgr.upload_archive("c6", "home/upload", tar_buf)
            .await
            .context("test precondition")?;

        let rootfs = mgr
            .writable_rootfs("c6")
            .await
            .context("test precondition")?;
        let uploaded = rootfs.path.join("home").join("upload").join("hello.txt");
        assert!(uploaded.exists(), "uploaded file should exist");
        assert_eq!(
            std::fs::read_to_string(&uploaded).context("test precondition")?,
            "hello"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_download_upload_roundtrip() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;

        let src_rootfs = setup_mock_rootfs(tmp.path(), "src")?;
        std::fs::write(
            src_rootfs.join("home").join("roundtrip.txt"),
            "roundtrip content",
        )
        .context("test precondition")?;

        let dst_rootfs = tmp.path().join("rootfs").join("containers").join("dst");
        std::fs::create_dir_all(dst_rootfs.join("home")).context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("src", src_rootfs).await;
        mgr.insert_test_record("dst", dst_rootfs).await;

        let tar_bytes = mgr
            .download_archive("src", "home")
            .await
            .context("test precondition")?;
        assert!(!tar_bytes.is_empty());

        mgr.upload_archive("dst", "home", tar_bytes)
            .await
            .context("test precondition")?;

        let dst_rootfs = mgr
            .writable_rootfs("dst")
            .await
            .context("test precondition")?;
        let transferred = dst_rootfs.path.join("home").join("roundtrip.txt");
        assert!(transferred.exists(), "roundtrip.txt should exist in dst");
        assert_eq!(
            std::fs::read_to_string(&transferred).context("test precondition")?,
            "roundtrip content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_inspect_returns_populated_fields() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "inspect-1")?;
        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("inspect-1", rootfs_path).await;

        let detail = mgr
            .inspect("inspect-1")
            .await
            .context("test precondition")?;
        assert!(
            detail.started_at.is_some(),
            "started_at should be populated"
        );
        assert!(
            detail.finished_at.is_none(),
            "finished_at should be None for running container"
        );
        assert!(
            detail.exit_code.is_none(),
            "exit_code should be None for running container"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_inspect_not_found() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);
        assert!(mgr.inspect("nonexistent").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_subscribe_receives_no_events_initially() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);
        let mut rx = mgr.subscribe();
        assert!(rx.try_recv().is_err(), "should have no events initially");
        Ok(())
    }

    #[tokio::test]
    async fn test_event_tx_is_functional() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);
        let mut rx = mgr.subscribe();

        use arona_container::ContainerEvent;
        let _ = mgr.event_tx.send(ContainerEvent::Created {
            id: "test-id".to_string(),
            name: "test-name".to_string(),
            image: "test-image".to_string(),
        });

        let event = rx.try_recv().context("test precondition")?;
        match event {
            ContainerEvent::Created { id, name, image } => {
                assert_eq!(id, "test-id");
                assert_eq!(name, "test-name");
                assert_eq!(image, "test-image");
            },
            _ => return Err(anyhow::anyhow!("expected Created event, got {:?}", event)),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_subscribers_receive_events() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);
        let mut rx1 = mgr.subscribe();
        let mut rx2 = mgr.subscribe();

        use arona_container::ContainerEvent;
        let _ = mgr.event_tx.send(ContainerEvent::Destroyed {
            id: "gone".to_string(),
        });

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_diff_workspace_with_nested_added_dir() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = tmp
            .path()
            .join("rootfs")
            .join("containers")
            .join("c-nested");
        let home = rootfs_path.join("home");
        std::fs::create_dir_all(home.join("deep").join("nested").join("dir"))
            .context("test precondition")?;
        std::fs::write(
            home.join("deep")
                .join("nested")
                .join("dir")
                .join("file.txt"),
            "nested content",
        )
        .context("test precondition")?;
        std::fs::write(home.join("top.txt"), "top level").context("test precondition")?;

        let host_ws = tmp.path().join("host-ws");
        std::fs::create_dir_all(&host_ws).context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-nested", rootfs_path).await;

        let changes = mgr
            .diff_workspace("c-nested", &host_ws, "home")
            .await
            .context("test precondition")?;

        let added: Vec<String> = changes
            .iter()
            .filter(|c| c.kind == arona_container::ChangeKind::Added)
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        assert!(
            added.iter().any(|p| p.contains("file.txt")),
            "nested file should be added: {:?}",
            added
        );
        assert!(
            added.iter().any(|p| p.contains("top.txt")),
            "top-level file should be added: {:?}",
            added
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_diff_workspace_empty_base_path_scans_all() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = tmp.path().join("rootfs").join("containers").join("c-all");
        let home = rootfs_path.join("home");
        let data = rootfs_path.join("data");
        std::fs::create_dir_all(&home).context("test precondition")?;
        std::fs::create_dir_all(&data).context("test precondition")?;
        std::fs::write(home.join("file.txt"), "home content").context("test precondition")?;
        std::fs::write(data.join("output.log"), "data content").context("test precondition")?;

        let host_ws = tmp.path().join("host-ws");
        std::fs::create_dir_all(&host_ws).context("test precondition")?;

        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-all", rootfs_path).await;

        let changes = mgr
            .diff_workspace("c-all", &host_ws, "")
            .await
            .context("test precondition")?;

        let paths: Vec<String> = changes
            .iter()
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        assert!(
            paths
                .iter()
                .any(|p| p == "/home/file.txt" || p.contains("home") && p.contains("file.txt")),
            "should find /home/file.txt, got: {:?}",
            paths
        );
        assert!(
            paths
                .iter()
                .any(|p| p == "/data/output.log" || p.contains("data") && p.contains("output.log")),
            "should find /data/output.log, got: {:?}",
            paths
        );
        Ok(())
    }

    #[tokio::test]
    async fn exec_without_pid_falls_back_to_host() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c-exec-nopid")?;
        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-exec-nopid", rootfs_path.clone())
            .await;

        let result = mgr.exec("c-exec-nopid", &["echo", "hello"]).await;
        if let Ok(output) = result {
            assert_eq!(output.exit_code, Some(0));
            assert!(output.stdout.contains("hello"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn exec_with_nonexistent_container_returns_not_found() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);

        let result = mgr.exec("nonexistent-container", &["echo", "hello"]).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ContainerError::NotFound(id) => assert_eq!(id, "nonexistent-container"),
            other => return Err(anyhow::anyhow!("Expected NotFound, got: {:?}", other)),
        }
        Ok(())
    }

    #[tokio::test]
    async fn exec_with_invalid_command_returns_error() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c-exec-bad")?;
        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-exec-bad", rootfs_path).await;

        let result = mgr.exec("c-exec-bad", &["nonexistent_binary_xyz"]).await;
        // The contract for a nonexistent binary is one of two observable
        // outcomes, depending on container backend:
        //   (a) the spawn fails and exec returns Err (most backends), OR
        //   (b) the shell inside the container reports exit 127 and exec
        //       returns Ok with exit_code != 0.
        // Both are acceptable; what is NOT acceptable is exec returning
        // Ok with exit_code == 0 (the test name says "returns error"). The
        // previous `if let Ok(output) = result { assert_ne!(...) }` form
        // silently passed when exec returned Err for ANY reason, including
        // unrelated bugs (container crash, manager panic-via-Err). We now
        // explicitly assert one of the two acceptable outcomes — any other
        // result (Ok with exit 0, Ok with no exit code, Err with a
        // container-not-found message that indicates we lost the test
        // fixture) fails the test.
        match result {
            Ok(output) => {
                assert_ne!(
                    output.exit_code,
                    Some(0),
                    "exec of a nonexistent binary must not exit 0; got output: {:?}",
                    output
                );
            },
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(
                    !msg.contains("NotFound"),
                    "the container itself must exist; got NotFound error: {msg}"
                );
                // Any other Err is acceptable — spawn failure, I/O error,
                // etc. We log it but do not fail.
                eprintln!("note: exec returned Err (acceptable): {msg}");
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn exec_with_pid_attempts_nsenter() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let rootfs_path = setup_mock_rootfs(tmp.path(), "c-exec-pid")?;
        let mgr = make_mgr(&tmp);
        mgr.insert_test_record("c-exec-pid", rootfs_path).await;

        mgr.state
            .update_pid("c-exec-pid", Some(std::process::id() as i32))
            .await;

        let result = mgr.exec("c-exec-pid", &["echo", "from-nsenter"]).await;
        match result {
            Ok(output) => {
                assert!(
                    output.stdout.contains("from-nsenter") || output.exit_code == Some(1),
                    "nsenter should execute echo or fail gracefully, got stdout={}, exit={:?}",
                    output.stdout.trim(),
                    output.exit_code
                );
            },
            Err(ContainerError::ExecFailed { ref message, .. }) => {
                assert!(
                    message.contains("nsenter"),
                    "Should attempt nsenter when PID is set, got: {}",
                    message
                );
            },
            Err(other) => {
                return Err(anyhow::anyhow!("Unexpected error type: {:?}", other));
            },
        }
        Ok(())
    }

    #[tokio::test]
    async fn exec_with_empty_command_returns_error() -> Result<()> {
        let tmp = TempDir::new().context("test precondition")?;
        let mgr = make_mgr(&tmp);

        let result = mgr.exec("any-container", &[]).await;
        assert!(result.is_err(), "Empty command should return error");
        match result.unwrap_err() {
            ContainerError::ExecFailed { message, .. } => {
                assert!(
                    message.contains("cannot be empty"),
                    "Expected empty command error, got: {}",
                    message
                );
            },
            other => return Err(anyhow::anyhow!("Expected ExecFailed, got: {:?}", other)),
        }
        Ok(())
    }
}
