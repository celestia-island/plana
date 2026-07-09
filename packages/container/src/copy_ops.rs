use futures::StreamExt;
use std::io::Read;

use bollard::{
    models::FilesystemChange,
    query_parameters::{DownloadFromContainerOptions, UploadToContainerOptions},
};
use tracing::warn;

use super::{
    errors::{ContainerError, ContainerResult},
    manager::ContainerManager,
};

impl ContainerManager {
    pub async fn download_archive(
        &self,
        container_id: &str,
        path: &str,
    ) -> ContainerResult<Vec<u8>> {
        let options = DownloadFromContainerOptions {
            path: path.to_string(),
        };
        let mut stream = self
            .docker
            .download_from_container(container_id, Some(options));
        let mut buf = Vec::new();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(ContainerError::from)?;
            buf.extend_from_slice(&bytes);
        }
        Ok(buf)
    }

    pub async fn container_filesystem_changes(
        &self,
        container_id: &str,
    ) -> ContainerResult<Vec<FilesystemChange>> {
        self.docker
            .container_changes(container_id)
            .await
            .map_err(ContainerError::from)
            .map(|opt| opt.unwrap_or_default())
    }

    pub async fn upload_archive(
        &self,
        container_id: &str,
        path: &str,
        data: Vec<u8>,
    ) -> ContainerResult<()> {
        let options = UploadToContainerOptions {
            path: path.to_string(),
            ..Default::default()
        };
        self.docker
            .upload_to_container(container_id, Some(options), bollard::body_full(data.into()))
            .await
            .map_err(ContainerError::from)
    }
}

pub fn changed_home_paths(changes: &[FilesystemChange]) -> std::collections::HashSet<String> {
    changed_paths_in_prefixes(changes, &["/home/"])
}

pub fn changed_paths_in_prefixes(
    changes: &[FilesystemChange],
    prefixes: &[&str],
) -> std::collections::HashSet<String> {
    changes
        .iter()
        .filter(|c| prefixes.iter().any(|p| c.path.starts_with(p)))
        .map(|c| c.path.clone())
        .collect()
}

pub struct HomeChange {
    pub path: String,
    pub is_modified: bool,
}

pub fn changed_home_paths_with_kind(changes: &[FilesystemChange]) -> Vec<HomeChange> {
    changed_paths_with_kind_in_prefixes(changes, &["/home/"])
}

pub fn changed_paths_with_kind_in_prefixes(
    changes: &[FilesystemChange],
    prefixes: &[&str],
) -> Vec<HomeChange> {
    changes
        .iter()
        .filter(|c| prefixes.iter().any(|p| c.path.starts_with(p)))
        .map(|c| HomeChange {
            path: c.path.clone(),
            is_modified: matches!(c.kind, bollard::models::ChangeType::_0),
        })
        .collect()
}

pub fn changed_home_paths_from_diff(
    changes: &[super::types::PathChange],
) -> std::collections::HashSet<String> {
    changed_paths_from_diff_in_prefixes(changes, &["/home/"])
}

pub fn changed_paths_from_diff_in_prefixes(
    changes: &[super::types::PathChange],
    prefixes: &[&str],
) -> std::collections::HashSet<String> {
    changes
        .iter()
        .filter(|c| prefixes.iter().any(|p| c.path.starts_with(p)))
        .map(|c| c.path.to_string_lossy().to_string())
        .collect()
}

pub struct DiffHomeChange {
    pub path: String,
    pub is_modified: bool,
}

pub fn changed_home_paths_with_kind_from_diff(
    changes: &[super::types::PathChange],
) -> Vec<DiffHomeChange> {
    changed_paths_with_kind_from_diff_in_prefixes(changes, &["/home/"])
}

pub fn changed_paths_with_kind_from_diff_in_prefixes(
    changes: &[super::types::PathChange],
    prefixes: &[&str],
) -> Vec<DiffHomeChange> {
    changes
        .iter()
        .filter(|c| prefixes.iter().any(|p| c.path.starts_with(p)))
        .map(|c| DiffHomeChange {
            path: c.path.to_string_lossy().to_string(),
            is_modified: matches!(c.kind, super::types::ChangeKind::Modified),
        })
        .collect()
}

pub fn filter_tar_to_changed_paths(
    tar_bytes: &[u8],
    changed: &std::collections::HashSet<String>,
) -> Vec<u8> {
    let mut archive = tar::Archive::new(tar_bytes);
    let entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "failed to read tar entries, returning original bytes");
            return tar_bytes.to_vec();
        }
    };

    let mut buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut buf);

        for entry_result in entries {
            let mut entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, "skipping unreadable tar entry");
                    continue;
                }
            };
            let entry_path = entry
                .path()
                .unwrap_or_else(|e| {
                    warn!(error = %e, "unreadable tar entry path, using empty");
                    std::borrow::Cow::Borrowed(std::path::Path::new(""))
                })
                .to_string_lossy()
                .into_owned();

            let normalized = if entry_path.starts_with("./") {
                entry_path.clone()
            } else {
                format!("/{}", entry_path)
            };

            let matches = changed.iter().any(|changed_path| {
                let cp = changed_path.trim_start_matches('/');
                normalized.trim_start_matches("./").starts_with(cp)
                    || cp.starts_with(normalized.trim_start_matches('/'))
            });

            if matches {
                let mut header = tar::Header::new_gnu();
                header.set_entry_type(entry.header().entry_type());
                header.set_size(entry.size());
                header.set_mode(entry.header().mode().unwrap_or(0o644));
                if let Ok(mtime) = entry.header().mtime() {
                    header.set_mtime(mtime);
                }
                let path_bytes = entry
                    .path()
                    .unwrap_or_else(|e| {
                        warn!(error = %e, "unreadable tar entry path for header");
                        std::borrow::Cow::Borrowed(std::path::Path::new("unknown"))
                    })
                    .into_owned();
                if let Err(e) = header.set_path(&path_bytes) {
                    warn!(error = %e, ?path_bytes, "failed to set tar header path");
                }
                header.set_cksum();

                let mut data = Vec::new();
                if entry.read_to_end(&mut data).is_ok()
                    && let Err(e) = builder.append(&header, data.as_slice())
                {
                    warn!(error = %e, path = %entry_path, "filter_tar: failed to append entry");
                }
            }
        }

        if let Err(e) = builder.finish() {
            warn!(error = %e, "filter_tar: failed to finalize archive");
        }
    }

    if buf.is_empty() {
        tar_bytes.to_vec()
    } else {
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result, ensure};

    #[cfg(feature = "docker-tests")]
    use bollard::exec::{CreateExecOptions, StartExecResults};
    #[cfg(feature = "docker-tests")]
    use bollard::models::ContainerCreateBody as DockerConfig;
    #[cfg(feature = "docker-tests")]
    use bollard::query_parameters::{CreateContainerOptions, RemoveContainerOptions};

    #[cfg(feature = "docker-tests")]
    fn docker_available() -> bool {
        std::env::var("RUN_DOCKER_TESTS")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    #[cfg(feature = "docker-tests")]
    async fn make_mgr() -> Result<ContainerManager> {
        ContainerManager::new().context("Docker must be available")
    }

    #[cfg(feature = "docker-tests")]
    async fn create_test_container(mgr: &ContainerManager, name: &str) -> Result<String> {
        let _ = mgr
            .docker
            .remove_container(
                name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;

        let container = mgr
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: Some(name.to_string()),
                    ..Default::default()
                }),
                DockerConfig {
                    image: Some("entelecheia:latest".to_string()),
                    entrypoint: Some(vec!["sleep".to_string(), "300".to_string()]),
                    cmd: Some(vec![]),
                    working_dir: Some("/home".to_string()),
                    ..Default::default()
                },
            )
            .await
            .context("create container")?;
        Ok(container.id)
    }

    #[cfg(feature = "docker-tests")]
    async fn exec_in_container(mgr: &ContainerManager, id: &str, cmd: &str) -> Result<()> {
        let exec = mgr
            .docker
            .create_exec(
                id,
                CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .context("create exec")?;

        let result = mgr.docker.start_exec(&exec.id, None).await;
        if let Ok(StartExecResults::Attached { mut output, .. }) = result {
            use futures::StreamExt;
            while output.next().await.is_some() {}
        }
        Ok(())
    }

    #[cfg(feature = "docker-tests")]
    async fn cleanup(mgr: &ContainerManager, id: &str) {
        let _ = mgr
            .docker
            .remove_container(
                id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_container_filesystem_changes_detects_new_file() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let container_name = "test-overlay-changes";
        let id = create_test_container(&mgr, container_name).await?;

        mgr.docker
            .start_container(&id, None)
            .await
            .context("start container")?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(&mgr, &id, "echo 'hello overlay' > /home/test_file.txt").await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let changes = mgr
            .container_filesystem_changes(&id)
            .await
            .context("get filesystem changes")?;

        let home_changes: Vec<_> = changes
            .iter()
            .filter(|c| c.path.starts_with("/home/"))
            .collect();

        ensure!(
            !home_changes.is_empty(),
            "Expected at least one change under /home, got {:?}",
            changes.iter().map(|c| &c.path).collect::<Vec<_>>()
        );

        ensure!(
            home_changes
                .iter()
                .any(|c| c.path.contains("test_file.txt")),
            "Expected test_file.txt in changes, got {:?}",
            home_changes
        );

        cleanup(&mgr, &id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_download_archive_and_extract_single_file() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let container_name = "test-overlay-download";
        let id = create_test_container(&mgr, container_name).await?;

        mgr.docker
            .start_container(&id, None)
            .await
            .context("start container")?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(
            &mgr,
            &id,
            "mkdir -p /home/subdir && echo 'content' > /home/subdir/nested.txt",
        )
        .await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let tar_bytes = mgr
            .download_archive(&id, "/home/subdir")
            .await
            .context("download archive")?;

        ensure!(!tar_bytes.is_empty(), "Expected non-empty tar archive");

        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        let tmp_dir = tempfile::tempdir().context("create temp dir")?;
        archive.unpack(tmp_dir.path()).context("unpack tar")?;

        let extracted = std::fs::read_to_string(tmp_dir.path().join("subdir").join("nested.txt"))
            .with_context(|| {
            let paths = std::fs::read_dir(tmp_dir.path())
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path().display().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            format!("nested.txt not found. Contents: {:?}", paths)
        })?;
        assert_eq!(extracted.trim(), "content");

        cleanup(&mgr, &id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_download_full_home_and_strip_prefix() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let container_name = "test-overlay-fullhome";
        let id = create_test_container(&mgr, container_name).await?;

        mgr.docker
            .start_container(&id, None)
            .await
            .context("start container")?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(&mgr, &id, "echo 'merge test' > /home/merge_test.txt").await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let tar_bytes = mgr
            .download_archive(&id, "/home")
            .await
            .context("download full /home")?;

        ensure!(!tar_bytes.is_empty());

        let tmp_dir = tempfile::tempdir().context("create temp dir")?;
        let mut archive = tar::Archive::new(tar_bytes.as_slice());
        archive.set_preserve_permissions(false);

        let mut found = false;

        for entry_result in archive.entries().context("read entries")? {
            let mut entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Warning: skipping entry: {}", e);
                    continue;
                }
            };
            let path = entry
                .path()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let stripped = path
                .strip_prefix("home/")
                .or_else(|| path.strip_prefix("./home/"))
                .unwrap_or(&path)
                .to_string();

            if stripped.is_empty()
                || stripped == "."
                || stripped == "entelecheia"
                || stripped.ends_with('/')
            {
                continue;
            }

            let target = tmp_dir.path().join(&stripped);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            if stripped == "merge_test.txt" {
                let mut buf = Vec::new();
                if entry.read_to_end(&mut buf).is_ok()
                    && !buf.is_empty()
                    && buf.iter().any(|&b| b != 0)
                {
                    std::fs::write(&target, &buf).context("write")?;
                    let content = std::fs::read_to_string(&target).context("read")?;
                    assert_eq!(content.trim(), "merge test");
                    found = true;
                }
            } else {
                let _ = entry.unpack(&target);
            }
        }

        ensure!(found, "merge_test.txt should be found in archive");

        cleanup(&mgr, &id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_child_to_parent_merge_transfers_files() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let parent_id = create_test_container(&mgr, "test-merge-parent").await?;
        let child_id = create_test_container(&mgr, "test-merge-child").await?;

        mgr.docker
            .start_container(&parent_id, None)
            .await
            .context("start parent")?;
        mgr.docker
            .start_container(&child_id, None)
            .await
            .context("start child")?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(&mgr, &child_id, "mkdir -p /home/src && echo 'child content' > /home/src/lib.rs && echo 'child-only' > /home/child_only.txt").await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let tar_bytes = mgr
            .download_archive(&child_id, "/home")
            .await
            .context("download child /home")?;
        ensure!(!tar_bytes.is_empty(), "child tar should not be empty");

        mgr.upload_archive(&parent_id, "/home", tar_bytes)
            .await
            .context("upload to parent")?;

        let verify_tar = mgr
            .download_archive(&parent_id, "/home/src/lib.rs")
            .await
            .context("download from parent")?;
        let mut archive = tar::Archive::new(verify_tar.as_slice());
        let tmp = tempfile::tempdir().context("temp dir")?;
        archive.unpack(tmp.path()).context("unpack")?;

        let content = std::fs::read_to_string(tmp.path().join("src").join("lib.rs"))
            .context("lib.rs should exist in parent after merge")?;
        assert_eq!(content.trim(), "child content");

        cleanup(&mgr, &parent_id).await;
        cleanup(&mgr, &child_id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_child_to_host_merge_extracts_correctly() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let id = create_test_container(&mgr, "test-merge-to-host").await?;

        mgr.docker
            .start_container(&id, None)
            .await
            .context("start")?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(&mgr, &id, "mkdir -p /home/deep/nested && echo 'layer1' > /home/deep/a.txt && echo 'layer2' > /home/deep/nested/b.txt").await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let tar_bytes = mgr
            .download_archive(&id, "/home")
            .await
            .context("download /home")?;
        ensure!(!tar_bytes.is_empty());

        let ws = tempfile::tempdir().context("workspace temp dir")?;
        extract_overlay_to_workspace(&tar_bytes, ws.path())?;

        let a =
            std::fs::read_to_string(ws.path().join("deep").join("a.txt")).context("deep/a.txt")?;
        let b = std::fs::read_to_string(ws.path().join("deep").join("nested").join("b.txt"))
            .context("deep/nested/b.txt")?;
        assert_eq!(a.trim(), "layer1");
        assert_eq!(b.trim(), "layer2");

        cleanup(&mgr, &id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_parallel_children_conflict_on_same_file() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let parent_id = create_test_container(&mgr, "test-conflict-parent").await?;
        let child_a_id = create_test_container(&mgr, "test-conflict-child-a").await?;
        let child_b_id = create_test_container(&mgr, "test-conflict-child-b").await?;

        for cid in [&parent_id, &child_a_id, &child_b_id] {
            mgr.docker
                .start_container(cid, None)
                .await
                .context("start")?;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(
            &mgr,
            &child_a_id,
            "echo 'from child A' > /home/conflict.txt && echo 'unique-A' > /home/unique_a.txt",
        )
        .await?;
        exec_in_container(
            &mgr,
            &child_b_id,
            "echo 'from child B' > /home/conflict.txt && echo 'unique-B' > /home/unique_b.txt",
        )
        .await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let child_a_changes: std::collections::HashSet<String> = mgr
            .container_filesystem_changes(&child_a_id)
            .await
            .context("child A changes")?
            .iter()
            .filter(|c| c.path.starts_with("/home/"))
            .map(|c| c.path.clone())
            .collect();

        let child_b_changes: std::collections::HashSet<String> = mgr
            .container_filesystem_changes(&child_b_id)
            .await
            .context("child B changes")?
            .iter()
            .filter(|c| c.path.starts_with("/home/"))
            .map(|c| c.path.clone())
            .collect();

        let conflicts: std::collections::HashSet<&str> = child_a_changes
            .iter()
            .chain(child_b_changes.iter())
            .filter(|path| child_a_changes.contains(*path) && child_b_changes.contains(*path))
            .map(|s| s.as_str())
            .collect();

        ensure!(
            conflicts.contains("/home/conflict.txt"),
            "Expected /home/conflict.txt in conflicts, got: {:?}",
            conflicts
        );
        ensure!(
            !conflicts.contains("/home/unique_a.txt"),
            "unique_a.txt should not be a conflict"
        );
        ensure!(
            !conflicts.contains("/home/unique_b.txt"),
            "unique_b.txt should not be a conflict"
        );

        let tar_a = mgr
            .download_archive(&child_a_id, "/home")
            .await
            .context("download A")?;
        mgr.upload_archive(&parent_id, "/home", tar_a)
            .await
            .context("merge A into parent")?;

        let tar_b = mgr
            .download_archive(&child_b_id, "/home")
            .await
            .context("download B")?;
        mgr.upload_archive(&parent_id, "/home", tar_b)
            .await
            .context("merge B into parent (overwrites A)")?;

        let verify = mgr
            .download_archive(&parent_id, "/home/conflict.txt")
            .await
            .context("verify")?;
        let mut archive = tar::Archive::new(verify.as_slice());
        let tmp = tempfile::tempdir().context("temp dir")?;
        archive.unpack(tmp.path()).context("unpack")?;
        let final_content =
            std::fs::read_to_string(tmp.path().join("conflict.txt")).context("read")?;
        ensure!(
            final_content.trim() == "from child B",
            "last-write-wins without conflict resolution"
        );

        cleanup(&mgr, &parent_id).await;
        cleanup(&mgr, &child_a_id).await;
        cleanup(&mgr, &child_b_id).await;
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "docker-tests")]
    async fn test_no_conflict_when_children_write_different_files() -> Result<()> {
        if !docker_available() {
            eprintln!("Skipping: RUN_DOCKER_TESTS=1 not set");
            return Ok(());
        }

        let mgr = make_mgr().await?;
        let child_a = create_test_container(&mgr, "test-noconflict-a").await?;
        let child_b = create_test_container(&mgr, "test-noconflict-b").await?;

        mgr.docker
            .start_container(&child_a, None)
            .await
            .context("start A")?;
        mgr.docker
            .start_container(&child_b, None)
            .await
            .context("start B")?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        exec_in_container(&mgr, &child_a, "echo 'only A' > /home/file_a.txt").await?;
        exec_in_container(&mgr, &child_b, "echo 'only B' > /home/file_b.txt").await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let a_changes: std::collections::HashSet<String> = mgr
            .container_filesystem_changes(&child_a)
            .await
            .context("A changes")?
            .iter()
            .filter(|c| c.path.starts_with("/home/"))
            .map(|c| c.path.clone())
            .collect();

        let b_changes: std::collections::HashSet<String> = mgr
            .container_filesystem_changes(&child_b)
            .await
            .context("B changes")?
            .iter()
            .filter(|c| c.path.starts_with("/home/"))
            .map(|c| c.path.clone())
            .collect();

        let conflicts: Vec<&str> = a_changes
            .iter()
            .filter(|p| b_changes.contains(*p))
            .map(|s| s.as_str())
            .collect();

        ensure!(
            conflicts.is_empty(),
            "No conflicts expected, got: {:?}",
            conflicts
        );

        cleanup(&mgr, &child_a).await;
        cleanup(&mgr, &child_b).await;
        Ok(())
    }

    #[cfg(feature = "docker-tests")]
    fn extract_overlay_to_workspace(tar_bytes: &[u8], workspace: &std::path::Path) -> Result<()> {
        let mut archive = tar::Archive::new(tar_bytes);
        archive.set_preserve_permissions(false);
        for entry in archive.entries().context("read entries")? {
            let mut entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry
                .path()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let stripped = path
                .strip_prefix("home/")
                .or_else(|| path.strip_prefix("./home/"))
                .unwrap_or(&path)
                .to_string();
            if stripped.is_empty() || stripped == "." || stripped == "home" {
                continue;
            }
            let target = workspace.join(&stripped);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = entry.unpack(&target);
        }
        Ok(())
    }

    #[test]
    fn test_changed_home_paths_filters_to_home() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/src/main.rs".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/etc/config.ini".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/home/README.md".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
        ];
        let paths = changed_home_paths(&changes);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains("/home/src/main.rs"));
        assert!(paths.contains("/home/README.md"));
        assert!(!paths.contains("/etc/config.ini"));
        Ok(())
    }

    #[test]
    fn test_filter_tar_keeps_only_changed_files() -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut buf);
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::file());
            header.set_size(5);
            header.set_path("home/src/main.rs")?;
            header.set_cksum();
            builder.append(&header, b"main\n".as_slice())?;

            let mut header2 = tar::Header::new_gnu();
            header2.set_entry_type(tar::EntryType::file());
            header2.set_size(4);
            header2.set_path("home/old.txt")?;
            header2.set_cksum();
            builder.append(&header2, b"old\n".as_slice())?;

            let mut header3 = tar::Header::new_gnu();
            header3.set_entry_type(tar::EntryType::file());
            header3.set_size(3);
            header3.set_path("home/README.md")?;
            header3.set_cksum();
            builder.append(&header3, b"md\n".as_slice())?;

            builder.finish()?;
        }

        let mut changed = std::collections::HashSet::new();
        changed.insert("/home/src/main.rs".to_string());
        changed.insert("/home/README.md".to_string());

        let filtered = filter_tar_to_changed_paths(&buf, &changed);
        let mut archive = tar::Archive::new(filtered.as_slice());
        let entries: Vec<_> = archive
            .entries()
            .context("read archive entries")?
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 2, "should keep only changed files");
        let names: Vec<_> = entries
            .iter()
            .filter_map(|e| e.path().ok())
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().any(|n| n.contains("main.rs")));
        assert!(names.iter().any(|n| n.contains("README.md")));
        assert!(!names.iter().any(|n| n.contains("old.txt")));
        Ok(())
    }

    #[test]
    fn test_filter_tar_empty_changes_returns_original() -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut buf);
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::file());
            header.set_size(5);
            header.set_path("home/test.txt")?;
            header.set_cksum();
            builder.append(&header, b"test\n".as_slice())?;
            builder.finish()?;
        }

        let empty = std::collections::HashSet::new();
        let filtered = filter_tar_to_changed_paths(&buf, &empty);
        let mut archive = tar::Archive::new(filtered.as_slice());
        let entries: Vec<_> = archive
            .entries()
            .context("read archive entries")?
            .filter_map(|e| e.ok())
            .collect();
        ensure!(
            entries.is_empty(),
            "empty changes should produce empty filtered archive, got {} entries",
            entries.len()
        );
        Ok(())
    }

    #[test]
    fn test_changed_home_paths_with_kind_classifies_modified_vs_added() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/Cargo.toml".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/home/NEW_REPORT.md".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/etc/ignore".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
        ];
        let home_changes = changed_home_paths_with_kind(&changes);
        assert_eq!(home_changes.len(), 2);

        let cargo = home_changes
            .iter()
            .find(|c| c.path == "/home/Cargo.toml")
            .context("Cargo.toml not found")?;
        assert!(cargo.is_modified, "Cargo.toml should be Modified (_0)");

        let report = home_changes
            .iter()
            .find(|c| c.path == "/home/NEW_REPORT.md")
            .context("NEW_REPORT.md not found")?;
        assert!(!report.is_modified, "NEW_REPORT.md should be Added (_1)");
        Ok(())
    }

    #[test]
    fn test_init_copy_filter_skips_existing_host_files() -> Result<()> {
        let tmp = tempfile::tempdir().context("temp dir")?;
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]").context("write Cargo.toml")?;
        std::fs::write(tmp.path().join("README.md"), "# readme").context("write README.md")?;

        let changes = vec![
            HomeChange {
                path: "/home/Cargo.toml".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/README.md".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/NEW_FILE.txt".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/modified_existing.rs".to_string(),
                is_modified: true,
            },
        ];

        let ws = tmp.path().to_path_buf();
        let skill_changes: Vec<String> = changes
            .into_iter()
            .filter(|c| {
                if c.is_modified {
                    return true;
                }
                let relative = match c.path.strip_prefix("/home/") {
                    Some(r) => r,
                    None => return true,
                };
                !ws.join(relative).exists()
            })
            .map(|c| c.path)
            .collect();

        assert_eq!(skill_changes.len(), 2);
        assert!(skill_changes.contains(&"/home/NEW_FILE.txt".to_string()));
        assert!(skill_changes.contains(&"/home/modified_existing.rs".to_string()));
        assert!(!skill_changes.contains(&"/home/Cargo.toml".to_string()));
        assert!(!skill_changes.contains(&"/home/README.md".to_string()));
        Ok(())
    }

    #[test]
    fn test_init_copy_filter_all_modified_pass_through() -> Result<()> {
        let tmp = tempfile::tempdir().context("temp dir")?;
        std::fs::write(tmp.path().join("existing.txt"), "old").context("write existing.txt")?;

        let changes = vec![
            HomeChange {
                path: "/home/existing.txt".to_string(),
                is_modified: true,
            },
            HomeChange {
                path: "/home/also_modified.rs".to_string(),
                is_modified: true,
            },
        ];

        let ws = tmp.path().to_path_buf();
        let skill_changes: Vec<String> = changes
            .into_iter()
            .filter(|c| {
                if c.is_modified {
                    return true;
                }
                let relative = match c.path.strip_prefix("/home/") {
                    Some(r) => r,
                    None => return true,
                };
                !ws.join(relative).exists()
            })
            .map(|c| c.path)
            .collect();

        assert_eq!(skill_changes.len(), 2);
        Ok(())
    }

    #[test]
    fn test_init_copy_filter_empty_when_all_init_copy() -> Result<()> {
        let tmp = tempfile::tempdir().context("temp dir")?;
        std::fs::write(tmp.path().join("file_a.txt"), "a").context("write file_a.txt")?;
        std::fs::write(tmp.path().join("file_b.txt"), "b").context("write file_b.txt")?;

        let changes = vec![
            HomeChange {
                path: "/home/file_a.txt".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/file_b.txt".to_string(),
                is_modified: false,
            },
        ];

        let ws = tmp.path().to_path_buf();
        let skill_changes: Vec<String> = changes
            .into_iter()
            .filter(|c| {
                if c.is_modified {
                    return true;
                }
                let relative = match c.path.strip_prefix("/home/") {
                    Some(r) => r,
                    None => return true,
                };
                !ws.join(relative).exists()
            })
            .map(|c| c.path)
            .collect();

        ensure!(
            skill_changes.is_empty(),
            "all Added files exist on host — should be filtered"
        );
        Ok(())
    }

    #[test]
    fn test_multi_container_collects_from_primary_and_fork() -> Result<()> {
        use std::collections::HashSet as HSet;

        let primary_changes = vec![
            FilesystemChange {
                path: "/home/Cargo.toml".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/home/src/main.rs".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
        ];
        let fork_changes = vec![FilesystemChange {
            path: "/home/DIAGNOSTIC.md".to_string(),
            kind: bollard::models::ChangeType::_1,
        }];

        let primary_home = changed_home_paths_with_kind(&primary_changes);
        let fork_home = changed_home_paths_with_kind(&fork_changes);

        let all_paths: HSet<String> = primary_home
            .iter()
            .chain(fork_home.iter())
            .map(|c| c.path.clone())
            .collect();

        assert_eq!(all_paths.len(), 3);
        assert!(all_paths.contains("/home/Cargo.toml"));
        assert!(all_paths.contains("/home/src/main.rs"));
        assert!(all_paths.contains("/home/DIAGNOSTIC.md"));
        Ok(())
    }

    // ── Three-layer merge tests ──

    fn simple_hash(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn detect_three_layer_conflicts(
        upstream_changes: &[HomeChange],
        downstream_changes: &[HomeChange],
    ) -> Vec<String> {
        let upstream_map: std::collections::HashMap<&str, &HomeChange> = upstream_changes
            .iter()
            .map(|c| (c.path.as_str(), c))
            .collect();
        downstream_changes
            .iter()
            .filter(|dc| {
                if let Some(uc) = upstream_map.get(dc.path.as_str()) {
                    dc.path == uc.path
                } else {
                    false
                }
            })
            .map(|c| c.path.clone())
            .collect()
    }

    #[test]
    fn test_three_layer_no_conflict_different_files() -> Result<()> {
        let fork_changes = vec![
            HomeChange {
                path: "/home/NEW_FILE.txt".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/another.rs".to_string(),
                is_modified: true,
            },
        ];
        let demiurge_changes = vec![HomeChange {
            path: "/home/Cargo.toml".to_string(),
            is_modified: true,
        }];

        let conflicts = detect_three_layer_conflicts(&demiurge_changes, &fork_changes);
        assert!(conflicts.is_empty(), "Different files should not conflict");
        Ok(())
    }

    #[test]
    fn test_three_layer_conflict_same_file() -> Result<()> {
        let fork_changes = vec![
            HomeChange {
                path: "/home/main.rs".to_string(),
                is_modified: true,
            },
            HomeChange {
                path: "/home/unique.txt".to_string(),
                is_modified: false,
            },
        ];
        let demiurge_changes = vec![
            HomeChange {
                path: "/home/main.rs".to_string(),
                is_modified: true,
            },
            HomeChange {
                path: "/home/other.txt".to_string(),
                is_modified: false,
            },
        ];

        let conflicts = detect_three_layer_conflicts(&demiurge_changes, &fork_changes);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], "/home/main.rs");
        assert!(!conflicts.contains(&"/home/unique.txt".to_string()));
        Ok(())
    }

    #[test]
    fn test_three_layer_host_vs_demiurge_conflict() -> Result<()> {
        let host_files = vec!["main.rs", "config.toml"];
        let host_dir = tempfile::tempdir().context("create temp dir")?;
        for f in &host_files {
            std::fs::write(host_dir.path().join(f), format!("host version of {}", f))
                .with_context(|| format!("write {}", f))?;
        }

        let demiurge_changes = [
            HomeChange {
                path: "/home/main.rs".to_string(),
                is_modified: true,
            },
            HomeChange {
                path: "/home/new_output.txt".to_string(),
                is_modified: false,
            },
        ];

        let ws = host_dir.path().to_path_buf();
        let skill_changes: Vec<String> = demiurge_changes
            .iter()
            .filter(|c| {
                if c.is_modified {
                    return true;
                }
                let relative = match c.path.strip_prefix("/home/") {
                    Some(r) => r,
                    None => return true,
                };
                !ws.join(relative).exists()
            })
            .map(|c| c.path.clone())
            .collect();

        assert!(
            skill_changes.contains(&"/home/main.rs".to_string()),
            "Modified files always pass"
        );
        assert!(
            skill_changes.contains(&"/home/new_output.txt".to_string()),
            "New file not on host"
        );
        Ok(())
    }

    #[test]
    fn test_three_layer_upstream_priority_model() -> Result<()> {
        let upstream_content = "fn main() { println!(\"upstream\"); }";
        let downstream_content = "fn main() { println!(\"downstream\"); }";

        assert_ne!(
            simple_hash(upstream_content),
            simple_hash(downstream_content)
        );

        let resolved = upstream_content.to_string();
        assert_eq!(resolved, upstream_content, "Upstream wins on conflict");
        Ok(())
    }

    #[test]
    fn test_three_layer_downstream_additions_preserved() -> Result<()> {
        let upstream_content = "fn foo() {}\n";
        let downstream_content = "fn foo() {}\nfn bar() {}\n";

        let upstream_lines: std::collections::HashSet<&str> = upstream_content.lines().collect();
        let downstream_only: Vec<&str> = downstream_content
            .lines()
            .filter(|l| !upstream_lines.contains(l))
            .collect();

        assert_eq!(downstream_only.len(), 1);
        assert_eq!(downstream_only[0], "fn bar() {}");
        Ok(())
    }

    #[test]
    fn test_three_layer_multiple_forks_merge_to_demiurge() -> Result<()> {
        let fork_a = [
            HomeChange {
                path: "/home/file_a.txt".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/shared.txt".to_string(),
                is_modified: true,
            },
        ];
        let fork_b = [
            HomeChange {
                path: "/home/file_b.txt".to_string(),
                is_modified: false,
            },
            HomeChange {
                path: "/home/shared.txt".to_string(),
                is_modified: true,
            },
        ];

        let all_fork_changes: Vec<&HomeChange> = fork_a.iter().chain(fork_b.iter()).collect();
        let shared_count = all_fork_changes
            .iter()
            .filter(|c| c.path == "/home/shared.txt")
            .count();
        assert_eq!(
            shared_count, 2,
            "Two forks both modified shared.txt → conflict"
        );
        Ok(())
    }

    #[test]
    fn test_changed_paths_in_prefixes_single() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/src/main.rs".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/data/output.log".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/etc/config.ini".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
        ];
        let paths = changed_paths_in_prefixes(&changes, &["/home/"]);
        assert_eq!(paths.len(), 1);
        assert!(paths.contains("/home/src/main.rs"));
        Ok(())
    }

    #[test]
    fn test_changed_paths_in_prefixes_multiple() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/src/main.rs".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/data/output.log".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/etc/config.ini".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
        ];
        let paths = changed_paths_in_prefixes(&changes, &["/home/", "/data/"]);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains("/home/src/main.rs"));
        assert!(paths.contains("/data/output.log"));
        Ok(())
    }

    #[test]
    fn test_changed_paths_in_prefixes_root_matches_all() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/src/main.rs".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
            FilesystemChange {
                path: "/data/output.log".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/etc/config.ini".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
        ];
        let paths = changed_paths_in_prefixes(&changes, &["/"]);
        assert_eq!(paths.len(), 3);
        Ok(())
    }

    #[test]
    fn test_changed_paths_with_kind_in_prefixes() -> Result<()> {
        let changes = vec![
            FilesystemChange {
                path: "/home/Cargo.toml".to_string(),
                kind: bollard::models::ChangeType::_0,
            },
            FilesystemChange {
                path: "/data/new_file.txt".to_string(),
                kind: bollard::models::ChangeType::_1,
            },
        ];
        let home_changes = changed_paths_with_kind_in_prefixes(&changes, &["/home/"]);
        assert_eq!(home_changes.len(), 1);
        assert!(home_changes[0].is_modified);

        let data_changes = changed_paths_with_kind_in_prefixes(&changes, &["/data/"]);
        assert_eq!(data_changes.len(), 1);
        assert!(!data_changes[0].is_modified);
        Ok(())
    }

    #[test]
    fn test_changed_paths_from_diff_in_prefixes() -> Result<()> {
        use super::super::types::{ChangeKind, PathChange};
        let changes = vec![
            PathChange {
                path: std::path::PathBuf::from("/home/a.rs"),
                kind: ChangeKind::Modified,
            },
            PathChange {
                path: std::path::PathBuf::from("/data/b.log"),
                kind: ChangeKind::Added,
            },
        ];
        let paths = changed_paths_from_diff_in_prefixes(&changes, &["/home/"]);
        assert_eq!(paths.len(), 1);
        assert!(paths.contains("/home/a.rs"));

        let all = changed_paths_from_diff_in_prefixes(&changes, &["/home/", "/data/"]);
        assert_eq!(all.len(), 2);
        Ok(())
    }

    #[test]
    fn test_changed_paths_with_kind_from_diff_in_prefixes() -> Result<()> {
        use super::super::types::{ChangeKind, PathChange};
        let changes = vec![
            PathChange {
                path: std::path::PathBuf::from("/home/mod.rs"),
                kind: ChangeKind::Modified,
            },
            PathChange {
                path: std::path::PathBuf::from("/home/new.txt"),
                kind: ChangeKind::Added,
            },
        ];
        let results = changed_paths_with_kind_from_diff_in_prefixes(&changes, &["/home/"]);
        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .any(|r| r.path == "/home/mod.rs" && r.is_modified)
        );
        assert!(
            results
                .iter()
                .any(|r| r.path == "/home/new.txt" && !r.is_modified)
        );
        Ok(())
    }
}
