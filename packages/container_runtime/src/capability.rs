use std::path::Path;

use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootfsCapability {
    FuseOverlay,
    KernelOverlay,
    Docker,
    None,
}

impl RootfsCapability {
    pub fn supports_isolation(self) -> bool {
        !matches!(self, RootfsCapability::None)
    }

    pub fn description(self) -> &'static str {
        match self {
            RootfsCapability::FuseOverlay => "fuse-overlayfs (user-space, no CAP_SYS_ADMIN needed)",
            RootfsCapability::KernelOverlay => "kernel overlay mount (requires CAP_SYS_ADMIN)",
            RootfsCapability::Docker => "Docker daemon (cosmos containers via Docker API)",
            RootfsCapability::None => "none — rootfs isolation unavailable",
        }
    }
}

pub fn detect_inside_container() -> bool {
    Path::new("/.dockerenv").exists()
        || std::fs::read_to_string("/proc/1/cgroup")
            .map(|c| c.contains("docker") || c.contains("containerd"))
            .unwrap_or(false)
}

pub async fn detect_rootfs_capability() -> RootfsCapability {
    if std::env::var("COSMOS_CONTAINER_RUNTIME").as_deref() == Ok("docker") {
        debug!("COSMOS_CONTAINER_RUNTIME=docker, cosmos containers will use Docker API");
        return RootfsCapability::Docker;
    }

    let fuse_bin =
        std::env::var("FUSE_OVERLAYFS_BIN").unwrap_or_else(|_| "fuse-overlayfs".to_string());

    let dev_fuse_ok = Path::new("/dev/fuse").exists();
    let fuse_bin_ok = tokio::process::Command::new(&fuse_bin)
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if dev_fuse_ok && fuse_bin_ok {
        match try_fuse_mount().await {
            true => {
                info!(
                    "rootfs capability: fuse-overlayfs (device=/dev/fuse, binary={})",
                    fuse_bin
                );
                return RootfsCapability::FuseOverlay;
            },
            false => {
                warn!(
                    "fuse-overlayfs binary and /dev/fuse present but mount test failed — check apparmor/seccomp/capabilities"
                );
            },
        }
    }

    if try_kernel_overlay().await {
        info!("rootfs capability: kernel overlay mount");
        return RootfsCapability::KernelOverlay;
    }

    // On the host (not inside a container), this process only orchestrates the
    // top-level infrastructure containers via the Docker API. Cosmos
    // sub-container isolation is delegated to the scepter container, which is
    // started with the Docker socket mounted (so cosmos uses the Docker API)
    // and, on Linux, with /dev/fuse passed through. The host process itself
    // therefore does not need fuse-overlayfs or CAP_SYS_ADMIN — treat this as
    // Docker-backed rather than a hard failure. Only when running inside a
    // container (where this process must perform isolation directly) does the
    // lack of any overlay mechanism become a real error.
    if !detect_inside_container() {
        info!(
            "rootfs capability: host mode — cosmos isolation delegated to scepter container (Docker API)"
        );
        return RootfsCapability::Docker;
    }

    RootfsCapability::None
}

async fn try_fuse_mount() -> bool {
    let tmp = std::env::temp_dir().join("entelecheia-rootfs-probe");
    let _ = tokio::fs::create_dir_all(tmp.join("upper")).await;
    let _ = tokio::fs::create_dir_all(tmp.join("work")).await;
    let _ = tokio::fs::create_dir_all(tmp.join("merged")).await;

    let fuse_bin =
        std::env::var("FUSE_OVERLAYFS_BIN").unwrap_or_else(|_| "fuse-overlayfs".to_string());
    let merged = tmp.join("merged").to_string_lossy().to_string();
    let upper = tmp.join("upper").to_string_lossy().to_string();
    let work = tmp.join("work").to_string_lossy().to_string();

    let ok = tokio::process::Command::new(&fuse_bin)
        .args([
            "-o",
            &format!("lowerdir=/,upperdir={},workdir={}", upper, work),
            &merged,
        ])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if ok {
        let _ = tokio::process::Command::new("fusermount3")
            .arg("-u")
            .arg(&merged)
            .status()
            .await;
        let _ = tokio::process::Command::new("fusermount")
            .arg("-u")
            .arg(&merged)
            .status()
            .await;
    }

    let _ = tokio::fs::remove_dir_all(&tmp).await;
    ok
}

async fn try_kernel_overlay() -> bool {
    let tmp = std::env::temp_dir().join("entelecheia-kernel-probe");
    let _ = tokio::fs::create_dir_all(tmp.join("upper")).await;
    let _ = tokio::fs::create_dir_all(tmp.join("work")).await;
    let _ = tokio::fs::create_dir_all(tmp.join("merged")).await;

    let merged = tmp.join("merged").to_string_lossy().to_string();
    let upper = tmp.join("upper").to_string_lossy().to_string();
    let work = tmp.join("work").to_string_lossy().to_string();
    let opts = format!("lowerdir=/,upperdir={},workdir={}", upper, work);

    let ok = tokio::process::Command::new("mount")
        .args(["-t", "overlay", "overlay", "-o", &opts, &merged])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    if ok {
        let _ = tokio::process::Command::new("umount")
            .arg(&merged)
            .status()
            .await;
    }

    let _ = tokio::fs::remove_dir_all(&tmp).await;
    ok
}
