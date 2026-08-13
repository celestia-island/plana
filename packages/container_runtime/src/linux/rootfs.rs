use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use nix::unistd::{self, Gid, Uid};
use tracing::{debug, error, info, warn};

use _container::errors::{ContainerError, ContainerResult};

const ROOTFS_CACHE_DIR: &str = "rootfs";

const HOST_EXCLUDED_DIRS: &[&str] = &["proc", "sys", "dev", "run", "tmp"];

/// Environment opt-in for the host-rootfs (`lowerdir=/`) overlay mode.
///
/// Mounting the container rootfs as an overlay over the host `/` would let the
/// container read the entire host filesystem (a read escape), so this mode is
/// fail-closed by default. It only runs when this flag is explicitly set — a
/// local-development escape hatch, never for production.
const HOST_ROOTFS_ALLOW_ENV: &str = "ENTELECHEIA_ALLOW_HOST_ROOTFS";

/// Return true when the host-rootfs overlay escape hatch is explicitly enabled.
///
/// Defaults to `false` (fail-closed) so a container can never silently read the
/// host filesystem via `lowerdir=/`.
fn host_rootfs_allowed() -> bool {
    std::env::var(HOST_ROOTFS_ALLOW_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct RootfsManager {
    base_dir: PathBuf,
}

impl RootfsManager {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            base_dir: base_dir.join(ROOTFS_CACHE_DIR),
        }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn base_rootfs_path(&self, image: &str) -> PathBuf {
        let safe_name = image.replace(['/', ':', '@'], "_");
        self.base_dir.join(safe_name)
    }

    pub fn container_rootfs_path(&self, container_id: &str) -> PathBuf {
        self.base_dir.join("containers").join(container_id)
    }

    pub fn snapshot_path(&self, snapshot_id: &str) -> PathBuf {
        self.base_dir.join("snapshots").join(snapshot_id)
    }

    pub async fn ensure_cache_dir(&self) -> ContainerResult<()> {
        tokio::fs::create_dir_all(&self.base_dir)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("failed to create rootfs cache dir: {}", e),
            })?;
        tokio::fs::create_dir_all(self.base_dir.join("containers"))
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("failed to create containers dir: {}", e),
            })?;
        tokio::fs::create_dir_all(self.base_dir.join("snapshots"))
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("failed to create snapshots dir: {}", e),
            })?;
        self.normalize_ownership(&self.base_dir).await?;
        Ok(())
    }

    /// Infer the `(uid, gid)` that rootfs artifacts *should* belong to.
    ///
    /// When the runtime runs with elevated privileges — most notably the
    /// `e-scepter` container, which executes as root while the host user's
    /// `~/.config/entelecheia` is bind-mounted in — every file we create is
    /// implicitly root-owned and becomes impossible for the real user to clean
    /// up afterwards (the very situation that left root-owned `proc`/`dev`/
    /// `sys` residue stuck in the config directory).
    ///
    /// We resolve the intended owner by walking up from `base_dir` and taking
    /// the first ancestor owned by a non-root account — normally the user's
    /// config directory. Returns `None` when no non-root owner can be inferred
    /// (i.e. we are legitimately running for a root user).
    fn intended_owner(&self) -> Option<(u32, u32)> {
        let mut current = self.base_dir.clone();
        loop {
            if let Ok(meta) = std::fs::metadata(&current) {
                let (uid, gid) = (meta.uid(), meta.gid());
                if uid != 0 {
                    return Some((uid, gid));
                }
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Recursively hand ownership of `path` back to the real user whenever the
    /// process is running as root but a non-root owner can be inferred.
    ///
    /// This is a no-op for the common rootless path (the intended design) and
    /// only compensates for the privileged fallback so that no root-owned
    /// artifact ever leaks into the user's config directory — guaranteeing that
    /// cleanup never requires root.
    async fn normalize_ownership(&self, path: &Path) -> ContainerResult<()> {
        if !unistd::geteuid().is_root() {
            return Ok(());
        }
        let Some((uid, gid)) = self.intended_owner() else {
            return Ok(());
        };
        if uid == 0 {
            return Ok(());
        }

        let target = path.to_path_buf();
        let cid_tag = "rootfs".to_string();
        tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            fn chown_tree(path: &Path, uid: u32, gid: u32) -> std::io::Result<()> {
                unistd::chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid))).map_err(
                    |e| std::io::Error::other(format!("chown {}: {}", path.display(), e)),
                )?;
                let meta = std::fs::metadata(path)?;
                if meta.is_dir() {
                    for entry in std::fs::read_dir(path)? {
                        let entry = entry?;
                        chown_tree(&entry.path(), uid, gid)?;
                    }
                }
                Ok(())
            }
            chown_tree(&target, uid, gid)
        })
        .await
        .map_err(|e| ContainerError::OperationFailed {
            container_id: cid_tag.clone(),
            message: format!("ownership normalization join failed: {}", e),
        })?
        .map_err(|e| ContainerError::OperationFailed {
            container_id: cid_tag,
            message: format!("failed to normalize ownership of {}: {}", path.display(), e),
        })?;
        debug!(
            path = %path.display(),
            uid, gid,
            "normalized rootfs ownership back to invoking user (was running as root)"
        );
        Ok(())
    }

    pub fn is_host_rootfs_image(image: &str) -> bool {
        matches!(image, "host" | "entelecheia" | "cosmos" | "cosmos-host")
            || image.starts_with("host:")
    }

    pub async fn prepare_container_rootfs(
        &self,
        base_image: &str,
        container_id: &str,
    ) -> ContainerResult<PathBuf> {
        if Self::is_host_rootfs_image(base_image) {
            self.prepare_overlay_rootfs(container_id).await
        } else {
            let base_path = self.base_rootfs_path(base_image);
            if base_path.is_dir() {
                let container_path = self.container_rootfs_path(container_id);
                if !container_path.exists() {
                    self.copy_rootfs(&base_path, &container_path).await?;
                }
                self.normalize_ownership(&container_path).await?;
                Ok(container_path)
            } else {
                self.prepare_overlay_rootfs(container_id).await
            }
        }
    }

    async fn prepare_overlay_rootfs(&self, container_id: &str) -> ContainerResult<PathBuf> {
        // Fail-closed: an overlay with `lowerdir=/` exposes the entire host
        // filesystem to the container (a read escape). Refuse to build such a
        // rootfs unless the operator explicitly opts in via the escape-hatch
        // flag (local development only).
        if !host_rootfs_allowed() {
            return Err(ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!(
                    "host rootfs overlay (lowerdir=/) is disabled: it would expose the entire \
                     host filesystem to the container. Set {}=1 to opt in for local \
                     development only.",
                    HOST_ROOTFS_ALLOW_ENV
                ),
            });
        }

        let container_path = self.container_rootfs_path(container_id);
        let upperdir = container_path.join("upper");
        let workdir = container_path.join("work");
        let merged = container_path.join("merged");

        if merged.exists() {
            return Ok(merged);
        }

        tokio::fs::create_dir_all(&upperdir).await.map_err(|e| {
            ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("mkdir upper: {}", e),
            }
        })?;
        tokio::fs::create_dir_all(&workdir)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("mkdir work: {}", e),
            })?;
        tokio::fs::create_dir_all(&merged)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("mkdir merged: {}", e),
            })?;

        // Hand the freshly created upper/work/merged dirs back to the real user
        // before mounting so the overlay artifacts are never root-owned.
        self.normalize_ownership(&container_path).await?;

        let options = format!(
            "lowerdir=/,upperdir={},workdir={}",
            upperdir.display(),
            workdir.display()
        );

        let merged_str = merged.to_string_lossy().to_string();

        let fuse_bin =
            std::env::var("FUSE_OVERLAYFS_BIN").unwrap_or_else(|_| "fuse-overlayfs".to_string());

        let fuse_available = tokio::process::Command::new(&fuse_bin)
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        if fuse_available {
            let status = tokio::process::Command::new(&fuse_bin)
                .args([
                    "-o",
                    &format!(
                        "lowerdir=/,upperdir={},workdir={}",
                        upperdir.display(),
                        workdir.display()
                    ),
                    &merged_str,
                ])
                .status()
                .await;

            if status.map(|s| s.success()).unwrap_or(false) {
                for excluded in HOST_EXCLUDED_DIRS {
                    let dir = merged.join(excluded);
                    if dir.is_dir() {
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        let _ = tokio::fs::create_dir(&dir).await;
                    }
                }
                info!(
                    container_id = %container_id,
                    merged = %merged_str,
                    "host rootfs overlay mounted via fuse-overlayfs (user-space, no CAP_SYS_ADMIN needed)"
                );
                return Ok(merged);
            }

            let err_output = tokio::process::Command::new(&fuse_bin)
                .args(["-o", &options, &merged_str])
                .output()
                .await;
            let stderr = err_output
                .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
                .unwrap_or_default();
            error!(
                container_id,
                stderr = %stderr.trim(),
                "fuse-overlayfs mount failed"
            );
        }

        let output = tokio::process::Command::new("mount")
            .args(["-t", "overlay", "overlay", "-o", &options, &merged_str])
            .output()
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!("failed to execute mount: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = tokio::fs::remove_dir_all(&container_path).await;
            error!(
                container_id,
                exit_code = %output.status,
                stderr = %stderr.trim(),
                "overlayfs mount FAILED — rootfs isolation unavailable"
            );
            return Err(ContainerError::OperationFailed {
                container_id: container_id.to_string(),
                message: format!(
                    "overlayfs mount failed (exit {}). stderr: {}. \
                     Install fuse-overlayfs or grant CAP_SYS_ADMIN. \
                     Rootfs isolation is MANDATORY for WRITE mode.",
                    output.status,
                    stderr.trim()
                ),
            });
        }

        for excluded in HOST_EXCLUDED_DIRS {
            let dir = merged.join(excluded);
            if dir.is_dir() {
                let _ = tokio::fs::remove_dir_all(&dir).await;
                let _ = tokio::fs::create_dir(&dir).await;
            }
        }

        info!(
            container_id = %container_id,
            merged = %merged_str,
            "host rootfs overlay mounted (sandbox mode)"
        );

        Ok(merged)
    }

    pub async fn cleanup_container_rootfs(&self, container_id: &str) -> ContainerResult<()> {
        let container_path = self.container_rootfs_path(container_id);
        let merged = container_path.join("merged");

        if merged.is_dir() {
            let merged_str = merged.to_string_lossy().to_string();
            Self::force_unmount(&merged_str).await;
        }

        if container_path.exists() {
            let _ = self.normalize_ownership(&container_path).await;
            match tokio::fs::remove_dir_all(&container_path).await {
                Ok(()) => {}
                Err(e) => {
                    warn!(
                        path = %container_path.display(),
                        error = %e,
                        "remove_dir_all failed after unmount, retrying with lazy unmount"
                    );
                    let merged_str = container_path.join("merged").to_string_lossy().to_string();
                    Self::force_unmount(&merged_str).await;
                    let _ = tokio::fs::remove_dir_all(&container_path).await;
                }
            }
        }
        Ok(())
    }

    /// Scan the `containers/` directory and force-clean every stale FUSE
    /// mount left behind by a crash, restart, or failed cleanup.
    ///
    /// Called during startup (`YoukiManager::initialize`) **before**
    /// `reconcile` so that stale mounts never accumulate across server
    /// restarts.
    pub async fn cleanup_stale_mounts(&self) -> usize {
        let containers_dir = self.base_dir.join("containers");
        let mut cleaned = 0usize;

        let mut entries = match tokio::fs::read_dir(&containers_dir).await {
            Ok(rd) => rd,
            Err(_) => return 0,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let container_path = entry.path();
            let merged = container_path.join("merged");

            if merged.is_dir() {
                let merged_str = merged.to_string_lossy().to_string();
                Self::force_unmount(&merged_str).await;
                cleaned += 1;
            }

            let _ = self.normalize_ownership(&container_path).await;
            let _ = tokio::fs::remove_dir_all(&container_path).await;
        }

        if cleaned > 0 {
            info!(
                cleaned,
                "startup sweep: cleaned {} stale container rootfs mount(s)", cleaned
            );
        }

        cleaned
    }

    /// Escalating unmount strategy for FUSE / overlay mounts.
    ///
    /// `fusermount -u` is the *only* method that reliably terminates the
    /// userspace `fuse-overlayfs` daemon. Plain `umount` detaches the VFS
    /// entry but often leaves the daemon alive, which is the root cause of
    /// the fuse-overlayfs process leak. We escalate through:
    ///
    /// 1. `fusermount3 -u`   — modern FUSE unmount (kills daemon)
    /// 2. `fusermount  -u`   — older FUSE unmount  (kills daemon)
    /// 3. `umount -l`        — lazy kernel unmount (VFS detach, daemon dies
    ///    when last file descriptor closes)
    /// 4. `umount`           — last-resort synchronous unmount
    async fn force_unmount(path: &str) {
        for (cmd, args) in [
            ("fusermount3", vec!["-u", path]),
            ("fusermount", vec!["-u", path]),
            ("umount", vec!["-l", path]),
            ("umount", vec![path]),
        ] {
            match tokio::process::Command::new(cmd).args(&args).status().await {
                Ok(s) if s.success() => {
                    debug!(path, unmounter = cmd, "unmount succeeded");
                    return;
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
        warn!(
            path,
            "all unmount methods failed — fuse-overlayfs daemon may linger"
        );
    }

    pub async fn snapshot_rootfs(
        &self,
        container_id: &str,
        snapshot_id: &str,
    ) -> ContainerResult<PathBuf> {
        let src = self.container_rootfs_path(container_id);
        let dest = self.snapshot_path(snapshot_id);

        if !src.exists() {
            return Err(ContainerError::NotFound(format!(
                "container rootfs not found: {}",
                src.display()
            )));
        }

        let upperdir = src.join("upper");
        if upperdir.is_dir() && Self::dir_is_nonempty(&upperdir).await {
            self.copy_rootfs(&upperdir, &dest).await?;
        } else {
            tokio::fs::create_dir_all(&dest).await.map_err(|e| {
                ContainerError::OperationFailed {
                    container_id: container_id.to_string(),
                    message: format!("mkdir snapshot: {}", e),
                }
            })?;
        }

        Ok(dest)
    }

    pub async fn extract_rootfs(&self, tarball: &Path, dest: &Path) -> ContainerResult<()> {
        if dest.exists() {
            return Ok(());
        }

        tokio::fs::create_dir_all(dest)
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("mkdir {}: {}", dest.display(), e),
            })?;

        let dest_str = dest.to_string_lossy().to_string();
        let tarball_str = tarball.to_string_lossy().to_string();

        let status = tokio::process::Command::new("tar")
            .args([
                "-xzf",
                &tarball_str,
                "-C",
                &dest_str,
                "--numeric-owner",
                "--preserve-permissions",
            ])
            .status()
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("tar: {}", e),
            })?;

        if !status.success() {
            return Err(ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("tar failed: {}", status),
            });
        }

        Ok(())
    }

    pub async fn copy_rootfs(&self, src: &Path, dest: &Path) -> ContainerResult<()> {
        if dest.exists() {
            return Ok(());
        }

        let src_str = src.to_string_lossy().to_string();
        let dest_str = dest.to_string_lossy().to_string();

        let status = tokio::process::Command::new("cp")
            .args(["-a", "--reflink=auto", &src_str, &dest_str])
            .status()
            .await
            .map_err(|e| ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("cp: {}", e),
            })?;

        if !status.success() {
            return Err(ContainerError::OperationFailed {
                container_id: "rootfs".to_string(),
                message: format!("cp failed: {}", status),
            });
        }

        Ok(())
    }

    async fn dir_is_nonempty(path: &Path) -> bool {
        if let Ok(mut entries) = tokio::fs::read_dir(path).await {
            entries
                .next_entry()
                .await
                .map(|e| e.is_some())
                .unwrap_or(false)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Restore `HOST_ROOTFS_ALLOW_ENV` to its prior value on drop, mirroring
    /// the env-guard pattern used elsewhere in this crate.
    struct EnvGuard {
        prev: Option<String>,
    }

    impl EnvGuard {
        fn clear() -> Self {
            let prev = std::env::var(HOST_ROOTFS_ALLOW_ENV).ok();
            unsafe { std::env::remove_var(HOST_ROOTFS_ALLOW_ENV) };
            Self { prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => unsafe { std::env::set_var(HOST_ROOTFS_ALLOW_ENV, v) },
                None => unsafe { std::env::remove_var(HOST_ROOTFS_ALLOW_ENV) },
            }
        }
    }

    #[test]
    fn host_rootfs_opt_in_flag_is_fail_closed_by_default() {
        let _guard = EnvGuard::clear();
        assert!(!host_rootfs_allowed(), "default must be fail-closed");
        unsafe { std::env::set_var(HOST_ROOTFS_ALLOW_ENV, "1") };
        assert!(host_rootfs_allowed(), "=1 must enable the escape hatch");
        unsafe { std::env::set_var(HOST_ROOTFS_ALLOW_ENV, "true") };
        assert!(host_rootfs_allowed(), "=true must enable the escape hatch");
        unsafe { std::env::set_var(HOST_ROOTFS_ALLOW_ENV, "0") };
        assert!(!host_rootfs_allowed(), "=0 must stay disabled");
        unsafe { std::env::set_var(HOST_ROOTFS_ALLOW_ENV, "no") };
        assert!(!host_rootfs_allowed(), "garbage must stay disabled");
    }

    #[tokio::test]
    async fn host_rootfs_overlay_fails_closed_by_default() {
        let _guard = EnvGuard::clear();
        let tmp = tempfile::tempdir().expect("tempdir");
        let mgr = RootfsManager::new(tmp.path());

        let result = mgr
            .prepare_container_rootfs("host", "fail-closed-test")
            .await;

        let err = result.expect_err("host rootfs overlay must fail closed by default");
        let msg = format!("{err}");
        assert!(
            msg.contains(HOST_ROOTFS_ALLOW_ENV),
            "error must document the opt-in flag, got: {msg}"
        );
        assert!(
            msg.contains("host filesystem"),
            "error must explain the read-escape risk, got: {msg}"
        );
    }

    #[tokio::test]
    async fn non_host_image_without_cache_also_fails_closed() {
        // The fallback path (image not pulled -> overlay) also uses the host `/`
        // lowerdir and must fail closed rather than silently exposing the host.
        let _guard = EnvGuard::clear();
        let tmp = tempfile::tempdir().expect("tempdir");
        let mgr = RootfsManager::new(tmp.path());

        let result = mgr
            .prepare_container_rootfs("some-image:latest", "fallback-test")
            .await;

        assert!(
            result.is_err(),
            "unpulled-image overlay fallback must fail closed by default"
        );
    }
}
