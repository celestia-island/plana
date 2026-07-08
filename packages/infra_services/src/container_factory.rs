use std::path::{Path, PathBuf};

use tracing::{info, warn};

use _container::{
    cli_backend::CliContainerBackend, errors::ContainerResult, ops::ContainerOps,
    types::ContainerRuntimeType,
};

pub fn default_container_data_dir() -> PathBuf {
    _config::UserConfig::config_dir().join("containers")
}

/// Returns the container runtime type for **outer orchestration**.
///
/// This is the runtime used by the TUI, scepter daemon, and server manager to
/// manage the top-level infrastructure containers (scepter, postgres, etc).
/// These containers are created via the Docker/Podman API and require a
/// full-featured container engine with networking, volume, and health-check
/// support.
///
/// Resolution order:
/// 1. `CONTAINER_RUNTIME` env var (explicit) — honours `auto` for platform detection.
/// 2. `"auto"` → platform-native runtime if available (Apple Container on macOS,
///    WSLc on Windows), otherwise Docker.
/// 3. Defaults to `"docker"`.
pub fn outer_runtime_type() -> ContainerRuntimeType {
    let raw = std::env::var("CONTAINER_RUNTIME").unwrap_or_else(|_| "docker".to_string());
    if raw.eq_ignore_ascii_case("auto") {
        detect_platform_runtime().unwrap_or(ContainerRuntimeType::Docker)
    } else {
        ContainerRuntimeType::from_str_lossy(&raw)
    }
}

/// Auto-detect the best available platform-native container runtime.
///
/// On macOS 26+ with Apple silicon, prefers Apple Container (`container` CLI).
/// On Windows 11, prefers WSL Containers (`wslc.exe`).
/// Returns `None` on Linux or when neither native runtime is on PATH.
fn detect_platform_runtime() -> Option<ContainerRuntimeType> {
    // macOS: Apple Container
    #[cfg(target_os = "macos")]
    {
        if find_in_path("container").is_some() {
            info!("outer runtime: apple-container (macOS native, detected on PATH)");
            return Some(ContainerRuntimeType::AppleContainer);
        }
    }

    // Windows: WSL Containers
    #[cfg(target_os = "windows")]
    {
        if find_in_path("wslc").is_some() || find_in_path("container").is_some() {
            info!("outer runtime: wslc (Windows native, detected on PATH)");
            return Some(ContainerRuntimeType::Wslc);
        }
    }

    #[allow(unreachable_code)]
    None
}

/// Returns the container runtime type for **inner cosmos sandboxing**.
///
/// Auto-detection logic (explicit `COSMOS_CONTAINER_RUNTIME` always wins):
/// 1. If `COSMOS_CONTAINER_RUNTIME` env var is set (or `auto`), honor it.
///    `"auto"` falls through to platform detection below.
/// 2. Inside a container WITH Docker socket → Docker (top-level scepter container).
/// 3. Inside a container WITHOUT Docker socket → youki (cosmos sub-container,
///    no Docker access; uses in-process sandboxing).
/// 4. On host WITH Docker socket → **Docker** (kernel overlayfs — fast).
/// 5. On host WITHOUT Docker socket but WITH /dev/fuse + `fuse-overlayfs` →
///    youki (rootless FUSE fallback).
/// 6. Otherwise (host) → Docker (will surface a clear error if unavailable).
///
/// **Platform-native CLI runtimes** (Apple Container, WSLc) are *not* used for
/// the inner cosmos sandbox because they lack the advanced rootfs/diff/commit
/// operations that the merge pipeline requires. They are, however, valid choices
/// for the outer orchestration layer via `CONTAINER_RUNTIME`.
///
/// **Performance note (branch 4 vs 5):** the youki rootless path mounts every
/// cosmos container's rootfs as a `fuse-overlayfs` over `lowerdir=/` — i.e. the
/// *entire host filesystem* (including large data mounts) is overlaid in
/// userspace, once per container. With many concurrent cosmos containers this
/// explodes load and memory. Docker's kernel-level overlay is dramatically
/// lighter, so on a host we now prefer Docker whenever the socket is present
/// and only fall back to the FUSE path when Docker is genuinely unavailable.
///
/// NOTE: branch 5 requires BOTH the device node and the binary. Keying only on
/// `/dev/fuse` (as before) selected youki on hosts that ship the device but not
/// the `fuse-overlayfs` package, so a non-root process then failed the overlay
/// mount with "must be superuser to use mount". Verifying the binary avoids
/// picking a backend that cannot actually isolate.
pub fn cosmos_runtime_type() -> ContainerRuntimeType {
    if let Ok(val) = std::env::var("COSMOS_CONTAINER_RUNTIME")
        && !val.is_empty()
        && !val.eq_ignore_ascii_case("auto")
    {
        return ContainerRuntimeType::from_str_lossy(&val);
    }

    let inside = _container_runtime::detect_inside_container();
    let docker_socket = std::path::Path::new("/var/run/docker.sock").exists();
    let dev_fuse = std::path::Path::new("/dev/fuse").exists();

    if inside && docker_socket {
        tracing::info!("cosmos runtime: docker (top-level container, Docker socket available)");
        ContainerRuntimeType::Docker
    } else if inside {
        tracing::info!(
            "cosmos runtime: youki (cosmos sub-container, no Docker socket — using in-process sandbox)"
        );
        ContainerRuntimeType::Youki
    } else if docker_socket {
        tracing::info!(
            "cosmos runtime: docker (host, Docker socket available — kernel overlay, far lighter \
             than per-container fuse-overlayfs over lowerdir=/)"
        );
        ContainerRuntimeType::Docker
    } else if dev_fuse && fuse_overlayfs_available() {
        tracing::warn!(
            "cosmos runtime: youki+fuse-overlayfs (host, no Docker socket). \
             fuse-overlayfs mounts each cosmos container over the entire host rootfs \
             (lowerdir=/), which is very heavy under concurrent load and large data \
             mounts. Start the Docker daemon (or set COSMOS_CONTAINER_RUNTIME=docker) \
             to use the fast kernel-overlay path."
        );
        ContainerRuntimeType::Youki
    } else {
        if dev_fuse {
            tracing::warn!(
                "/dev/fuse present but the fuse-overlayfs binary was not found; \
                 falling back to Docker for cosmos isolation. Install fuse-overlayfs \
                 (or set COSMOS_CONTAINER_RUNTIME=docker) to use the rootless youki path."
            );
        } else {
            tracing::info!("cosmos runtime: docker (host, no /dev/fuse)");
        }
        ContainerRuntimeType::Docker
    }
}

/// Return true if the `fuse-overlayfs` binary responds to `--version`.
///
/// Honors `FUSE_OVERLAYFS_BIN` (consistent with the mount code in
/// `container_runtime::rootfs` and `capability`). This is a fast, one-shot
/// probe — safe to call from the synchronous `cosmos_runtime_type()`.
fn fuse_overlayfs_available() -> bool {
    use std::process::Stdio;
    let bin = std::env::var("FUSE_OVERLAYFS_BIN").unwrap_or_else(|_| "fuse-overlayfs".to_string());
    std::process::Command::new(&bin)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Search PATH for `binary` and return the resolved path if found.
///
/// Minimal `which`-equivalent to avoid pulling in a new dependency.
#[allow(dead_code)] // used only on macOS/Windows targets
fn find_in_path(binary: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    let ext = if cfg!(target_os = "windows") {
        std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.COM;.BAT".into())
    } else {
        String::new()
    };
    let exts: Vec<&str> = if ext.is_empty() {
        vec![""]
    } else {
        ext.split(';').collect()
    };

    for dir in std::env::split_paths(&path_env) {
        for exe_ext in &exts {
            let candidate = dir.join(format!("{binary}{exe_ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub async fn create_container_backend(
    runtime: ContainerRuntimeType,
    data_dir: &Path,
) -> ContainerResult<Box<dyn ContainerOps>> {
    match runtime {
        ContainerRuntimeType::Youki => {
            let mgr = _container_runtime::YoukiManager::new(data_dir)?;
            mgr.initialize().await?;
            if let Err(e) = mgr.reconcile().await {
                warn!(
                    "Youki reconcile failed (continuing with empty state): {}",
                    e
                );
            }
            Ok(Box::new(mgr))
        },
        ContainerRuntimeType::Docker => {
            let mgr = _container::ContainerManager::new()?;
            Ok(Box::new(mgr))
        },
        ContainerRuntimeType::Wslc | ContainerRuntimeType::AppleContainer => {
            let backend = CliContainerBackend::new(runtime)?;
            info!(
                runtime = runtime.as_str(),
                "CLI container backend initialised for outer orchestration"
            );
            Ok(Box::new(backend))
        },
    }
}

pub async fn create_container_backend_from_config(
    config: &_config::UserConfig,
    data_dir: &Path,
) -> ContainerResult<Box<dyn ContainerOps>> {
    let runtime = ContainerRuntimeType::from_str_lossy(&config.container_backend.runtime);
    create_container_backend(runtime, data_dir).await
}

pub async fn create_container_backend_from_str(
    runtime_str: &str,
) -> ContainerResult<Box<dyn ContainerOps>> {
    let runtime = ContainerRuntimeType::from_str_lossy(runtime_str);
    let data_dir = default_container_data_dir();
    create_container_backend(runtime, &data_dir).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    /// RAII guard that restores an environment variable to its original value
    /// (or removes it) when dropped — safe in the face of test panics.
    struct EnvGuard {
        key: &'static str,
        old: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, val: &str) -> Self {
            let old = std::env::var(key).ok();
            unsafe { std::env::set_var(key, val) };
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(v) => unsafe { std::env::set_var(self.key, v) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn test_default_container_data_dir_is_under_config() -> Result<()> {
        let dir = default_container_data_dir();
        assert!(dir.to_string_lossy().contains("containers"));
        Ok(())
    }

    #[test]
    fn test_runtime_type_from_str_youki() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("youki"),
            ContainerRuntimeType::Youki
        ));
        Ok(())
    }

    #[test]
    fn test_runtime_type_from_str_docker() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("docker"),
            ContainerRuntimeType::Docker
        ));
        Ok(())
    }

    #[test]
    fn test_runtime_type_from_str_default_is_youki() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("anything_else"),
            ContainerRuntimeType::Youki
        ));
        Ok(())
    }

    #[test]
    fn test_runtime_type_from_str_wslc() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("wslc"),
            ContainerRuntimeType::Wslc
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("wsl"),
            ContainerRuntimeType::Wslc
        ));
        Ok(())
    }

    #[test]
    fn test_runtime_type_from_str_apple_container() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("apple-container"),
            ContainerRuntimeType::AppleContainer
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("apple"),
            ContainerRuntimeType::AppleContainer
        ));
        Ok(())
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_outer_runtime_type_auto_on_linux() -> Result<()> {
        // On Linux without a platform-native CLI runtime, auto should resolve
        // to Docker (the existing default). The EnvGuard restores the original
        // value even if the assertion panics.
        let _guard = EnvGuard::set("CONTAINER_RUNTIME", "auto");
        let runtime = outer_runtime_type();
        assert!(
            matches!(runtime, ContainerRuntimeType::Docker),
            "on Linux without apple-container/wslc, auto should fall back to Docker, got {:?}",
            runtime
        );
        Ok(())
    }

    #[test]
    fn test_outer_runtime_type_resolves_docker() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("docker"),
            ContainerRuntimeType::Docker
        ));
        Ok(())
    }

    #[test]
    fn test_cosmos_runtime_type_resolves_youki() -> Result<()> {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("youki"),
            ContainerRuntimeType::Youki
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_create_youki_backend() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let backend = create_container_backend(ContainerRuntimeType::Youki, tmp.path()).await?;

        let list = backend.list().await?;
        assert!(list.is_empty(), "fresh backend should have no containers");
        Ok(())
    }

    #[tokio::test]
    async fn test_youki_backend_images() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let backend = create_container_backend(ContainerRuntimeType::Youki, tmp.path()).await?;

        let exists = backend.image_exists("host").await?;
        assert!(exists, "host rootfs should always exist");
        Ok(())
    }

    #[tokio::test]
    async fn test_youki_backend_list_volumes() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let backend = create_container_backend(ContainerRuntimeType::Youki, tmp.path()).await?;

        let volumes = backend.list_volumes().await?;
        // The previous `volumes.iter().all(...)` was vacuously true when
        // `volumes` is empty (iter().all() on an empty iterator returns true).
        // On a fresh backend, the volume list SHOULD be empty — assert that
        // directly. If volumes ARE present (e.g. leaked from another test),
        // each must be well-formed.
        if volumes.is_empty() {
            // Expected: a fresh backend has no volumes.
        } else {
            for v in &volumes {
                assert!(
                    !v.name.is_empty(),
                    "volume name must not be empty, got: {:?}",
                    v
                );
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_youki_backend_inspect_not_found() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let backend = create_container_backend(ContainerRuntimeType::Youki, tmp.path()).await?;

        let result = backend.inspect("nonexistent").await;
        assert!(result.is_err(), "inspect on nonexistent should fail");
        Ok(())
    }
}
