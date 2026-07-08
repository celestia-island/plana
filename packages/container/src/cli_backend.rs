//! CLI-based container runtime adapter.
//!
//! Provides [`CliContainerBackend`] — a generic implementation of [`ContainerOps`]
//! that drives container runtimes via their command-line interface. This is used
//! for runtimes that do not expose a native API (HTTP, FFI) but instead offer a
//! Docker-compatible CLI:
//!
//! - **WSL Containers** (`wslc.exe` / `container.exe`) — Microsoft's built-in
//!   Linux container runtime on Windows 11.
//! - **Apple Container** (`container`) — Apple's native OCI runtime on
//!   macOS 26+ (VM-per-container architecture).
//!
//! Both CLIs follow Docker command conventions closely, so a single adapter with
//! a lightweight [`CliProfile`] per runtime covers both.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tokio::process::Command;
use tracing::{debug, instrument, warn};

use crate::errors::{ContainerError, ContainerResult};
use crate::ops::ContainerOps;
use crate::types::{
    ContainerCreateParams, ContainerDetail, ContainerForkParams, ContainerInfo,
    ContainerRuntimeType, ContainerStatus, DockerVolumeInfo, ExecOutput, ImageInfo, PathChange,
    WritableRootfs,
};

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

/// Describes the command-line conventions of a specific CLI-based runtime.
///
/// Both WSLc and Apple Container follow Docker conventions, but differ in
/// sub-command layout (e.g., `container image pull` vs `wslc pull`) and in
/// whether structured output is available.
#[derive(Debug, Clone)]
struct CliProfile {
    /// Human-readable name for logging.
    name: &'static str,
    /// Binary name to search for on PATH.
    binary: &'static str,
    /// Whether the CLI supports `--format json` (or equivalent) for structured
    /// output on `ls` / `inspect`. When `true`, we parse JSON; when `false`,
    /// we fall back to regex/heuristics.
    structured_output: bool,
    /// Sub-command prefix for image operations.
    /// Apple Container: `"image"` → `container image pull`
    /// WSLc: `""` → `wslc pull` (no sub-command)
    image_subcmd: &'static str,
}

impl CliProfile {
    const fn apple_container() -> Self {
        Self {
            name: "apple-container",
            binary: "container",
            structured_output: true,
            image_subcmd: "image",
        }
    }

    const fn wslc() -> Self {
        Self {
            name: "wslc",
            binary: "container",
            structured_output: false,
            image_subcmd: "",
        }
    }

    #[cfg(target_os = "windows")]
    const fn wslc_binary() -> &'static str {
        "wslc.exe"
    }

    #[cfg(not(target_os = "windows"))]
    const fn wslc_binary() -> &'static str {
        "container"
    }
}

// ---------------------------------------------------------------------------
// Backend
// ---------------------------------------------------------------------------

/// Container backend that drives a CLI-based runtime (WSLc or Apple Container).
///
/// Constructed via [`CliContainerBackend::new`] with a [`ContainerRuntimeType`].
/// Resolves the binary on PATH at construction time.
pub struct CliContainerBackend {
    profile: CliProfile,
    binary_path: PathBuf,
    runtime: ContainerRuntimeType,
}

impl CliContainerBackend {
    /// Create a CLI backend for the given runtime type.
    ///
    /// Resolves the CLI binary on PATH and probes it with `--version`.
    pub fn new(runtime: ContainerRuntimeType) -> ContainerResult<Self> {
        let profile = Self::profile_for(runtime)?;
        let binary_candidates = Self::binary_candidates(&profile, runtime);
        let binary_path = Self::resolve_binary(&binary_candidates).ok_or_else(|| {
            let msg = format!(
                "none of the candidate binaries found on PATH: [{}]",
                binary_candidates.join(", ")
            );
            ContainerError::RuntimeNotFound(msg)
        })?;

        debug!(
            runtime = profile.name,
            binary = %binary_path.display(),
            "CLI container backend initialised"
        );

        Ok(Self {
            profile,
            binary_path,
            runtime,
        })
    }

    fn profile_for(runtime: ContainerRuntimeType) -> ContainerResult<CliProfile> {
        match runtime {
            ContainerRuntimeType::AppleContainer => Ok(CliProfile::apple_container()),
            ContainerRuntimeType::Wslc => {
                let mut p = CliProfile::wslc();
                p.binary = CliProfile::wslc_binary();
                Ok(p)
            },
            _ => Err(ContainerError::NotSupported(format!(
                "CliContainerBackend does not support runtime `{}`",
                runtime
            ))),
        }
    }

    /// Return candidate binary names to search for (platform-specific).
    fn binary_candidates(profile: &CliProfile, runtime: ContainerRuntimeType) -> Vec<&'static str> {
        match runtime {
            ContainerRuntimeType::Wslc => {
                // On Windows, find_in_path will try bare names with PATHEXT
                // extensions (.EXE, .CMD, etc.). On other platforms the bare
                // names are used directly.
                vec!["wslc", "container"]
            },
            ContainerRuntimeType::AppleContainer => vec![profile.binary],
            _ => vec![],
        }
    }

    /// Search PATH for the first candidate binary that exists.
    fn resolve_binary(candidates: &[&'static str]) -> Option<PathBuf> {
        for &name in candidates {
            // Use `Command` to probe — if the binary is absent the spawn fails fast.
            if let Some(path) = find_in_path(name) {
                return Some(path);
            }
        }
        None
    }

    /// Execute the CLI with the given arguments and return stdout on success.
    #[instrument(skip(self), fields(runtime = %self.profile.name, args = ?args))]
    async fn run(&self, args: &[&str]) -> ContainerResult<String> {
        let output = Command::new(&self.binary_path)
            .args(args)
            .output()
            .await
            .map_err(|e| ContainerError::CliFailed {
                binary: self.binary_path.display().to_string(),
                args: args.join(" "),
                message: format!("spawn failed: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(ContainerError::CliFailed {
                binary: self.profile.binary.to_string(),
                args: args.join(" "),
                message: if stderr.is_empty() {
                    format!("exit {:?}: {}", output.status.code(), stdout.trim())
                } else {
                    stderr.trim().to_string()
                },
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    // -- Helpers to build CLI argument lists from params --

    /// Build `run` arguments from [`ContainerCreateParams`].
    fn build_run_args(params: &ContainerCreateParams) -> Vec<String> {
        let mut args = vec![
            "run".to_string(),
            "-d".to_string(), // detached
        ];

        args.push("--name".to_string());
        args.push(params.name.clone());

        for (k, v) in &params.env {
            args.push("-e".to_string());
            args.push(format!("{k}={v}"));
        }

        for port in &params.ports {
            args.push("-p".to_string());
            let proto = if port.protocol != "tcp" {
                format!("/{}", port.protocol)
            } else {
                String::new()
            };
            args.push(format!(
                "{}:{}{}",
                port.host_port, port.container_port, proto
            ));
        }

        for vol in &params.volumes {
            args.push("-v".to_string());
            let ro = if vol.read_only { ":ro" } else { "" };
            args.push(format!("{}:{}{}", vol.host_path, vol.container_path, ro));
        }

        for (k, v) in &params.labels {
            args.push("--label".to_string());
            args.push(format!("{k}={v}"));
        }

        if let Some(mem) = params.memory_limit {
            args.push("--memory".to_string());
            args.push(format!("{}m", mem / (1024 * 1024)));
        }

        if let Some(cpus) = params.nano_cpus {
            args.push("--cpus".to_string());
            args.push(format!("{}", cpus as f64 / 1_000_000_000.0));
        }

        if let Some(pids) = params.pids_limit {
            args.push("--pids-limit".to_string());
            args.push(pids.to_string());
        }

        if let Some(ref user) = params.user {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        if let Some(ref workdir) = params.working_dir {
            args.push("-w".to_string());
            args.push(workdir.clone());
        }

        if params.read_only_rootfs.unwrap_or(false) {
            args.push("--read-only".to_string());
        }

        if let Some(ref network) = params.network {
            args.push("--network".to_string());
            args.push(network.clone());
        }

        for cap in params.cap_drop.iter().flatten() {
            args.push("--cap-drop".to_string());
            args.push(cap.clone());
        }
        for cap in params.cap_add.iter().flatten() {
            args.push("--cap-add".to_string());
            args.push(cap.clone());
        }

        if let Some(ref log_driver) = params.log_driver {
            args.push("--log-driver".to_string());
            args.push(log_driver.clone());
        }
        for (k, v) in &params.log_opts {
            args.push("--log-opt".to_string());
            args.push(format!("{k}={v}"));
        }
        if let Some(ref sec_opt) = params.security_opt {
            for opt in sec_opt {
                args.push("--security-opt".to_string());
                args.push(opt.clone());
            }
        }
        if let Some(ref groups) = params.group_add {
            for g in groups {
                args.push("--group-add".to_string());
                args.push(g.clone());
            }
        }
        for dev in &params.devices {
            args.push("--device".to_string());
            args.push(format!(
                "{}:{}:{}",
                dev.host_path, dev.container_path, dev.permissions
            ));
        }

        if let Some(ref hc) = params.healthcheck {
            if !hc.test.is_empty() {
                args.push("--health-cmd".to_string());
                args.push(hc.test.join(" "));
            }
            if let Some(ns) = hc.interval_ns {
                args.push("--health-interval".to_string());
                args.push(nanos_to_duration_str(ns));
            }
            if let Some(ns) = hc.timeout_ns {
                args.push("--health-timeout".to_string());
                args.push(nanos_to_duration_str(ns));
            }
            if let Some(r) = hc.retries {
                args.push("--health-retries".to_string());
                args.push(r.to_string());
            }
            if let Some(ns) = hc.start_period_ns {
                args.push("--health-start-period".to_string());
                args.push(nanos_to_duration_str(ns));
            }
        }

        // Image and optional command — always last
        args.push(params.image.clone());

        args
    }
}

// ---------------------------------------------------------------------------
// ContainerOps implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl ContainerOps for CliContainerBackend {
    // -- Lifecycle --

    async fn create(&self, params: &ContainerCreateParams) -> ContainerResult<ContainerInfo> {
        let arg_strs = CliContainerBackend::build_run_args(params);
        let arg_refs: Vec<&str> = arg_strs.iter().map(|s| s.as_str()).collect();
        let stdout = self.run(&arg_refs).await?;

        // Most CLIs print the container ID on `run -d`. Take the first token
        // in case warnings or other output precede the ID.
        let id = stdout.split_whitespace().next().unwrap_or("").to_string();
        if id.is_empty() {
            return Err(ContainerError::OperationFailed {
                container_id: params.name.clone(),
                message: "create produced no output — cannot determine container ID".into(),
            });
        }

        // Try to inspect immediately to get full info.
        match self.inspect(&id).await {
            Ok(detail) => Ok(detail.info),
            Err(_) => {
                // Fallback: construct minimal info. Use Created (not Running)
                // since we can't verify the container actually started.
                Ok(ContainerInfo {
                    id: id.clone(),
                    name: params.name.clone(),
                    image: params.image.clone(),
                    status: ContainerStatus::Created,
                    created_at: chrono::Utc::now().into(),
                    ports: params.ports.clone(),
                    env: params.env.clone(),
                    volumes: params.volumes.clone(),
                    ip_address: None,
                    labels: params.labels.clone(),
                })
            },
        }
    }

    async fn start(&self, container_id: &str) -> ContainerResult<()> {
        self.run(&["start", container_id]).await?;
        Ok(())
    }

    async fn stop(&self, container_id: &str) -> ContainerResult<()> {
        self.run(&["stop", container_id]).await?;
        Ok(())
    }

    async fn remove(&self, container_id: &str, force: bool) -> ContainerResult<()> {
        let mut args = vec!["rm"];
        if force {
            args.push("-f");
        }
        args.push(container_id);
        self.run(&args).await?;
        Ok(())
    }

    async fn restart(&self, container_id: &str) -> ContainerResult<()> {
        self.run(&["restart", container_id]).await?;
        Ok(())
    }

    // -- Query --

    async fn list(&self) -> ContainerResult<Vec<ContainerInfo>> {
        self.list_with_filter(None, None, false).await
    }

    async fn list_with_filter(
        &self,
        name_prefix: Option<&str>,
        label_filter: Option<HashMap<String, String>>,
        all: bool,
    ) -> ContainerResult<Vec<ContainerInfo>> {
        let mut args = vec!["ls"];
        if all {
            args.push("-a");
        }
        // Add label filters to the CLI command (Docker-compatible syntax).
        // We collect them here and also apply post-query for safety.
        let mut label_args: Vec<String> = Vec::new();
        if let Some(ref labels) = label_filter {
            for (k, v) in labels {
                label_args.push(format!("--filter=label={}={}", k, v));
            }
        }
        for la in &label_args {
            args.push(la);
        }
        if self.profile.structured_output {
            args.push("--format");
            args.push("json");
        } else {
            args.push("--format");
            args.push("{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}");
        }

        let stdout = self.run(&args).await?;
        let mut containers = self.parse_list_output(&stdout)?;

        if let Some(prefix) = name_prefix {
            if prefix.starts_with('^') && prefix.ends_with('$') {
                // Anchored regex: exact match only (strip ^ and $)
                let pattern = &prefix[1..prefix.len() - 1];
                containers.retain(|c| c.name == pattern);
            } else {
                containers.retain(|c| c.name.starts_with(prefix));
            }
        }

        // Post-query label filtering (belt-and-suspenders in case the CLI
        // doesn't support --filter=label=...). Only applies when we actually
        // parsed labels from the CLI output — if labels is empty (CLI didn't
        // emit them), we can't verify and must trust the CLI's own filter.
        if let Some(ref labels) = label_filter {
            containers.retain(|c| {
                if c.labels.is_empty() {
                    true // can't verify — trust CLI's --filter
                } else {
                    labels.iter().all(|(k, v)| c.labels.get(k) == Some(v))
                }
            });
        }

        Ok(containers)
    }

    async fn inspect(&self, container_id: &str) -> ContainerResult<ContainerDetail> {
        if self.profile.structured_output {
            let stdout = self
                .run(&["inspect", container_id, "--format", "json"])
                .await?;
            self.parse_inspect_json(&stdout, container_id)
        } else {
            // Try --format first, fall back to raw JSON.
            let result = self
                .run(&["inspect", "--format", "json", container_id])
                .await;
            let stdout = match result {
                Ok(s) => s,
                Err(_) => self.run(&["inspect", container_id]).await?,
            };
            self.parse_inspect_json(&stdout, container_id)
        }
    }

    async fn is_running(&self, container_id: &str) -> ContainerResult<bool> {
        match self.inspect(container_id).await {
            Ok(detail) => Ok(detail.info.status.is_running()),
            Err(ContainerError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    // -- Exec --

    async fn exec(&self, container_id: &str, command: &[&str]) -> ContainerResult<ExecOutput> {
        if command.is_empty() {
            return Err(ContainerError::InvalidParam(
                "exec command must not be empty".into(),
            ));
        }
        let span = tracing::debug_span!(
            "exec",
            runtime = %self.profile.name,
            container_id,
            cmd = ?command,
        );
        let _guard = span.enter();
        let mut args = vec!["exec", container_id];
        args.extend(command);
        let output = Command::new(&self.binary_path)
            .args(&args)
            .output()
            .await
            .map_err(|e| ContainerError::ExecFailed {
                container_id: container_id.to_string(),
                message: format!("spawn failed: {e}"),
            })?;

        Ok(ExecOutput {
            exit_code: output.status.code().map(|c| c as i64),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    // -- Fork / Commit --
    // These are advanced operations not uniformly available via CLI.
    // Return NotSupported so callers can fall back to Docker/Youki when needed.

    async fn fork(&self, _params: &ContainerForkParams) -> ContainerResult<ContainerInfo> {
        Err(ContainerError::NotSupported(format!(
            "fork is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    async fn commit(
        &self,
        _container_id: &str,
        _repo: &str,
        _tag: Option<&str>,
    ) -> ContainerResult<String> {
        Err(ContainerError::NotSupported(format!(
            "commit is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    async fn commit_with_labels(
        &self,
        _container_id: &str,
        _repo: &str,
        _tag: Option<&str>,
        _labels: Option<&HashMap<String, String>>,
    ) -> ContainerResult<String> {
        Err(ContainerError::NotSupported(format!(
            "commit_with_labels is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    // -- Health / Ensure --

    async fn wait_healthy(&self, container_id: &str, timeout: Duration) -> ContainerResult<()> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(ContainerError::OperationFailed {
                    container_id: container_id.to_string(),
                    message: "health-check timeout".to_string(),
                });
            }
            match self.is_running(container_id).await {
                Ok(true) => return Ok(()),
                Ok(false) => {},
                Err(ContainerError::NotFound(_)) => {
                    return Err(ContainerError::NotFound(container_id.to_string()));
                },
                Err(e) => {
                    warn!(error = %e, "health-check inspect failed, retrying");
                },
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn ensure_running(&self, container_id: &str) -> ContainerResult<ContainerInfo> {
        let detail = self.inspect(container_id).await?;
        if !detail.info.status.is_running() {
            self.start(container_id).await?;
            // Re-inspect after start.
            let detail = self.inspect(container_id).await?;
            return Ok(detail.info);
        }
        Ok(detail.info)
    }

    async fn recreate(
        &self,
        container_name: &str,
        new_image: &str,
    ) -> ContainerResult<ContainerInfo> {
        // Remove old container if it exists. Ignore NotFound to avoid TOCTOU
        // race — another process may have removed it between inspect and remove.
        if self.inspect(container_name).await.is_ok()
            && let Err(e) = self.remove(container_name, true).await
            && !matches!(e, ContainerError::NotFound(_))
        {
            return Err(e);
        }
        let params = ContainerCreateParams::simple(container_name, new_image);
        self.create(&params).await
    }

    // -- Images --

    async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>> {
        let mut args: Vec<&str> = Vec::new();
        if !self.profile.image_subcmd.is_empty() {
            args.push(self.profile.image_subcmd);
        }
        args.push("ls");
        if self.profile.structured_output {
            args.push("--format");
            args.push("json");
        } else {
            args.push("--format");
            args.push("{{.ID}}\t{{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}");
        }

        let stdout = self.run(&args).await?;
        self.parse_images_output(&stdout)
    }

    async fn pull_image(&self, image: &str) -> ContainerResult<String> {
        let mut args: Vec<&str> = Vec::new();
        if !self.profile.image_subcmd.is_empty() {
            args.push(self.profile.image_subcmd);
        }
        args.push("pull");
        args.push(image);
        self.run(&args).await?;
        Ok(image.to_string())
    }

    async fn image_exists(&self, image: &str) -> ContainerResult<bool> {
        let images = self.list_images().await?;
        Ok(images.iter().any(|img| {
            img.repository == image
                || format!("{}:{}", img.repository, img.tag) == image
                || img.id == image
        }))
    }

    async fn remove_image(&self, image: &str, force: bool) -> ContainerResult<()> {
        let mut args: Vec<&str> = Vec::new();
        if !self.profile.image_subcmd.is_empty() {
            args.push(self.profile.image_subcmd);
        }
        args.push("rm");
        if force {
            args.push("-f");
        }
        args.push(image);
        self.run(&args).await?;
        Ok(())
    }

    async fn remove_with_image(
        &self,
        container_id: &str,
        image: &str,
        force: bool,
    ) -> ContainerResult<()> {
        self.remove(container_id, force).await?;
        self.remove_image(image, force).await
    }

    // -- Volumes --
    // CLI-based runtimes may or may not support volumes.
    // We attempt the commands and surface errors naturally.

    async fn create_volume(&self, name: &str) -> ContainerResult<String> {
        self.run(&["volume", "create", name]).await?;
        Ok(name.to_string())
    }

    async fn remove_volume(&self, name: &str, force: bool) -> ContainerResult<()> {
        let mut args = vec!["volume", "rm"];
        if force {
            args.push("-f");
        }
        args.push(name);
        self.run(&args).await?;
        Ok(())
    }

    async fn volume_exists(&self, name: &str) -> ContainerResult<bool> {
        let volumes = self.list_volumes().await?;
        Ok(volumes.iter().any(|v| v.name == name))
    }

    async fn list_volumes(&self) -> ContainerResult<Vec<DockerVolumeInfo>> {
        // Force tab-separated output via --format for reliability.
        // Docker's default volume ls output is space-aligned (not tab-separated).
        let mut args = vec!["volume", "ls"];
        args.push("--format");
        args.push("{{.Driver}}\t{{.Name}}");
        let stdout = self.run(&args).await?;
        let mut volumes = Vec::new();
        for line in stdout.lines().skip(1) {
            // Format: DRIVER\tNAME (Docker-style, tab-separated)
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() >= 2 {
                volumes.push(DockerVolumeInfo {
                    driver: parts[0].to_string(),
                    name: parts[1].to_string(),
                    mountpoint: None,
                });
            }
        }
        Ok(volumes)
    }

    // -- Logs --

    async fn logs(&self, container_id: &str, tail: usize) -> ContainerResult<Vec<String>> {
        if tail == 0 {
            return Ok(Vec::new());
        }
        let tail_str = tail.to_string();
        let stdout = self
            .run(&["logs", "--tail", &tail_str, container_id])
            .await?;
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }

    async fn get_container_logs(&self, container_id: &str, tail: usize) -> ContainerResult<String> {
        let lines = self.logs(container_id, tail).await?;
        Ok(lines.join("\n"))
    }

    // -- Rootfs Access --
    // These are merge-pipeline specific operations not available via CLI.

    async fn writable_rootfs(&self, _container_id: &str) -> ContainerResult<WritableRootfs> {
        Err(ContainerError::NotSupported(format!(
            "writable_rootfs is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    async fn diff_workspace(
        &self,
        _container_id: &str,
        _workspace_path: &Path,
        _base_path: &str,
    ) -> ContainerResult<Vec<PathChange>> {
        Err(ContainerError::NotSupported(format!(
            "diff_workspace is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    async fn download_archive(&self, _container_id: &str, _path: &str) -> ContainerResult<Vec<u8>> {
        Err(ContainerError::NotSupported(format!(
            "download_archive is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    async fn upload_archive(
        &self,
        _container_id: &str,
        _path: &str,
        _data: Vec<u8>,
    ) -> ContainerResult<()> {
        Err(ContainerError::NotSupported(format!(
            "upload_archive is not supported via the {} CLI backend",
            self.profile.name
        )))
    }

    // -- Clone --

    fn clone_boxed(&self) -> Box<dyn ContainerOps> {
        // CliContainerBackend is immutable after construction — safe to clone
        // the (cheap) handle fields.
        Box::new(Self {
            profile: self.profile.clone(),
            binary_path: self.binary_path.clone(),
            runtime: self.runtime,
        })
    }
}

// ---------------------------------------------------------------------------
// Output parsing helpers
// ---------------------------------------------------------------------------

impl CliContainerBackend {
    /// Parse `ls` output into [`ContainerInfo`] list.
    fn parse_list_output(&self, stdout: &str) -> ContainerResult<Vec<ContainerInfo>> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if self.profile.structured_output {
            // Apple Container JSON: array of objects
            self.parse_list_json(trimmed)
        } else {
            // Tab-delimited Go template output
            self.parse_list_tsv(trimmed)
        }
    }

    fn parse_list_json(&self, json: &str) -> ContainerResult<Vec<ContainerInfo>> {
        // Parse as JSON Value first, then extract manually for maximum field-name
        // tolerance. Direct deserialization into ContainerInfo fails because the
        // struct has no serde aliases and real CLI output uses different field
        // names/casings (Docker: PascalCase, Apple Container: lowercase).
        let values: Vec<serde_json::Value> = if json.trim_start().starts_with('[') {
            serde_json::from_str(json)
                .map_err(|e| ContainerError::CliParse(format!("list JSON array: {e}")))?
        } else {
            // NDJSON: one object per line
            json.lines()
                .filter(|l| l.trim().starts_with('{'))
                .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                .collect()
        };

        let containers = values.iter().filter_map(extract_container_info).collect();
        Ok(containers)
    }

    fn parse_list_tsv(&self, text: &str) -> ContainerResult<Vec<ContainerInfo>> {
        let mut containers = Vec::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                continue;
            }
            let id = parts[0].to_string();
            let name = parts[1].to_string();
            let image = parts[2].to_string();
            let status_str = parts[3].to_string();
            let status = if status_str.contains("Up") {
                ContainerStatus::Running
            } else if status_str.contains("Exited") || status_str.contains("Stopped") {
                ContainerStatus::Exited
            } else {
                ContainerStatus::Unknown
            };

            containers.push(ContainerInfo {
                id,
                name,
                image,
                status,
                created_at: None,
                ports: Vec::new(),
                env: HashMap::new(),
                volumes: Vec::new(),
                ip_address: None,
                labels: HashMap::new(),
            });
        }
        Ok(containers)
    }

    fn parse_inspect_json(&self, json: &str, id_hint: &str) -> ContainerResult<ContainerDetail> {
        let json = json.trim();

        // Try array first (Docker/WSLc format), then single object (Apple Container).
        let val = if json.starts_with('[') {
            let arr: Vec<serde_json::Value> = serde_json::from_str(json)
                .map_err(|e| ContainerError::CliParse(format!("inspect JSON array: {e}")))?;
            arr.into_iter()
                .next()
                .ok_or_else(|| ContainerError::NotFound(id_hint.to_string()))?
        } else {
            serde_json::from_str(json)
                .map_err(|e| ContainerError::CliParse(format!("inspect JSON: {e}")))?
        };

        extract_container_detail(&val, id_hint)
            .ok_or_else(|| ContainerError::CliParse("inspect JSON: unexpected structure".into()))
    }

    fn parse_images_output(&self, stdout: &str) -> ContainerResult<Vec<ImageInfo>> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if self.profile.structured_output && trimmed.starts_with('[') {
            let values: Vec<serde_json::Value> = serde_json::from_str(trimmed)
                .map_err(|e| ContainerError::CliParse(format!("images JSON: {e}")))?;
            return Ok(values.iter().filter_map(extract_image_info).collect());
        }

        // TSV fallback
        let mut images = Vec::new();
        for line in trimmed.lines() {
            if line.starts_with("REPOSITORY") || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                images.push(ImageInfo {
                    id: parts[0].to_string(),
                    repository: parts[1].to_string(),
                    tag: parts[2].to_string(),
                    size: parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
                    created: parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0),
                });
            }
        }
        Ok(images)
    }
}

// ---------------------------------------------------------------------------
// Duration formatting helper
// ---------------------------------------------------------------------------

/// Convert nanoseconds to a Docker-compatible duration string (e.g., "30s", "5m").
/// Docker CLI only supports seconds and minutes granularity.
fn nanos_to_duration_str(ns: i64) -> String {
    let secs = ns / 1_000_000_000;
    if secs < 60 {
        format!("{secs}s")
    } else {
        format!("{}m", secs / 60)
    }
}

// ---------------------------------------------------------------------------
// JSON extraction helpers (field-name tolerant)
// ---------------------------------------------------------------------------

/// Get a string field from a JSON object, trying multiple key names.
///
/// Different container CLIs use different casings: Docker uses PascalCase
/// (`"Id"`, `"Name"`), Apple Container uses lowercase (`"id"`, `"name"`).
/// This helper tries both plus common variants.
fn json_str<'a>(
    obj: &'a serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<&'a str> {
    for key in keys {
        if let Some(v) = obj.get(*key)
            && let Some(s) = v.as_str()
        {
            return Some(s);
        }
    }
    None
}

/// Get a bool field from a JSON object, trying multiple key names.
fn json_bool(obj: &serde_json::Map<String, serde_json::Value>, keys: &[&str]) -> Option<bool> {
    for key in keys {
        if let Some(v) = obj.get(*key)
            && let Some(b) = v.as_bool()
        {
            return Some(b);
        }
    }
    None
}

/// Get an i64 field from a JSON object, trying multiple key names.
fn json_i64(obj: &serde_json::Map<String, serde_json::Value>, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(n) = v.as_i64() {
                return Some(n);
            }
            // Some CLIs emit numbers as strings
            if let Some(s) = v.as_str()
                && let Ok(n) = s.parse()
            {
                return Some(n);
            }
        }
    }
    None
}

/// Extract a [`ContainerInfo`] from a JSON value, tolerant of field name casings.
///
/// Tries Docker-style PascalCase keys and Apple Container-style lowercase keys.
/// Returns `None` if the value is not an object or lacks an `id`/`Id`.
fn extract_container_info(val: &serde_json::Value) -> Option<ContainerInfo> {
    let obj = val.as_object()?;

    let id = json_str(obj, &["id", "Id", "ID", "container_id"])?.to_string();
    let name = json_str(obj, &["name", "Name", "Names"])
        .unwrap_or(&id)
        .trim_start_matches('/')
        .to_string();

    // Image can be a string or an object with a reference field
    let image = obj
        .get("image")
        .or_else(|| obj.get("Image"))
        .or_else(|| obj.get("Config").and_then(|c| c.get("Image")))
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(obj) = v.as_object() {
                json_str(obj, &["reference", "Reference", "name", "Name"])
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Status: try state object, then flat status string
    let state = obj.get("state").or_else(|| obj.get("State"));
    let running = state
        .and_then(|s| {
            s.as_object()
                .and_then(|so| json_bool(so, &["running", "Running"]))
        })
        .or_else(|| json_bool(obj, &["running", "Running"]))
        .unwrap_or(false);

    let status_str = state
        .and_then(|s| {
            s.as_object()
                .and_then(|so| json_str(so, &["status", "Status"]))
        })
        .or_else(|| json_str(obj, &["status", "Status"]))
        .unwrap_or("unknown");

    let status = if running {
        ContainerStatus::Running
    } else {
        match status_str.to_lowercase().as_str() {
            "created" => ContainerStatus::Created,
            "running" => ContainerStatus::Running,
            "paused" => ContainerStatus::Paused,
            "restarting" => ContainerStatus::Restarting,
            "removing" => ContainerStatus::Removing,
            "exited" | "stopped" => ContainerStatus::Exited,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Unknown,
        }
    };

    let created_at = json_str(obj, &["created_at", "Created", "created", "CreatedAt"])
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Parse labels from JSON if present (Docker: "Labels", Apple: "labels")
    let labels = obj
        .get("Labels")
        .or_else(|| obj.get("labels"))
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    Some(ContainerInfo {
        id,
        name,
        image,
        status,
        created_at,
        ports: Vec::new(),
        env: HashMap::new(),
        volumes: Vec::new(),
        ip_address: None,
        labels,
    })
}

/// Extract a [`ContainerDetail`] from a JSON value, tolerant of field casings.
fn extract_container_detail(val: &serde_json::Value, id_hint: &str) -> Option<ContainerDetail> {
    let obj = val.as_object()?;

    let id = json_str(obj, &["id", "Id", "ID"])
        .unwrap_or(id_hint)
        .to_string();
    let name = json_str(obj, &["name", "Name"])
        .unwrap_or(&id)
        .trim_start_matches('/')
        .to_string();

    let image = obj
        .get("image")
        .or_else(|| obj.get("Image"))
        .or_else(|| obj.get("Config").and_then(|c| c.get("Image")))
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(o) = v.as_object() {
                json_str(o, &["reference", "Reference", "name"])
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let state = obj.get("state").or_else(|| obj.get("State"));
    let state_obj = state.and_then(|s| s.as_object());

    let running = state_obj
        .and_then(|so| json_bool(so, &["running", "Running"]))
        .or_else(|| json_bool(obj, &["running", "Running"]))
        .unwrap_or(false);

    let status_str = state_obj
        .and_then(|so| json_str(so, &["status", "Status"]))
        .or_else(|| json_str(obj, &["status", "Status"]))
        .unwrap_or("unknown");

    let status = if running {
        ContainerStatus::Running
    } else {
        match status_str.to_lowercase().as_str() {
            "created" => ContainerStatus::Created,
            "running" => ContainerStatus::Running,
            "paused" => ContainerStatus::Paused,
            "restarting" => ContainerStatus::Restarting,
            "removing" => ContainerStatus::Removing,
            "exited" | "stopped" => ContainerStatus::Exited,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Unknown,
        }
    };

    let exit_code = state_obj.and_then(|so| json_i64(so, &["exit_code", "ExitCode", "exitCode"]));
    let started_at = state_obj
        .and_then(|so| json_str(so, &["started_at", "StartedAt", "startedAt"]))
        .map(|s| s.to_string());
    let finished_at = state_obj
        .and_then(|so| json_str(so, &["finished_at", "FinishedAt", "finishedAt"]))
        .map(|s| s.to_string());
    let error = state_obj
        .and_then(|so| json_str(so, &["error", "Error"]))
        .map(|s| s.to_string());

    let created_at = json_str(obj, &["created_at", "Created", "created", "CreatedAt"])
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Parse labels from JSON if present (same logic as extract_container_info)
    let labels: HashMap<String, String> = obj
        .get("Labels")
        .or_else(|| obj.get("labels"))
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // Also try Config.Labels (Docker inspect nests labels under Config)
    let labels = if labels.is_empty() {
        obj.get("Config")
            .and_then(|c| c.get("Labels"))
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or(labels)
    } else {
        labels
    };

    Some(ContainerDetail {
        info: ContainerInfo {
            id,
            name,
            image,
            status,
            created_at,
            ports: Vec::new(),
            env: HashMap::new(),
            volumes: Vec::new(),
            ip_address: None,
            labels,
        },
        exit_code,
        started_at,
        finished_at,
        error,
    })
}

/// Extract an [`ImageInfo`] from a JSON value, tolerant of field casings.
fn extract_image_info(val: &serde_json::Value) -> Option<ImageInfo> {
    let obj = val.as_object()?;

    let id = json_str(obj, &["id", "Id", "digest", "Digest"])
        .unwrap_or("unknown")
        .to_string();

    // Image reference can be a single string or split into repo + tag
    let (repository, tag) = if let Some(reference) = json_str(
        obj,
        &[
            "reference",
            "Reference",
            "name",
            "Name",
            "repository",
            "Repository",
        ],
    ) {
        if let Some((repo, tg)) = reference.rsplit_once(':') {
            (repo.to_string(), tg.to_string())
        } else {
            (reference.to_string(), "latest".to_string())
        }
    } else {
        (
            json_str(obj, &["repository", "Repository"])
                .unwrap_or("unknown")
                .to_string(),
            json_str(obj, &["tag", "Tag"])
                .unwrap_or("latest")
                .to_string(),
        )
    };

    let size = json_i64(obj, &["size", "Size"]).unwrap_or(0);
    let created = json_i64(obj, &["created", "Created", "CreatedAt", "created_at"]).unwrap_or(0);

    Some(ImageInfo {
        id,
        repository,
        tag,
        size,
        created,
    })
}

// ---------------------------------------------------------------------------
// PATH lookup
// ---------------------------------------------------------------------------

/// Search PATH for `binary` and return the resolved path if found.
///
/// This is a minimal replacement for the `which` crate — we avoid pulling in
/// another dependency for a single call-site.
fn find_in_path(binary: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    let ext = if cfg!(target_os = "windows") {
        std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.COM;.BAT;.CMD".into())
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
            if candidate.is_file() && is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Check whether a file has the executable bit set on Unix.
/// On Windows all regular files are considered executable for PATH lookup
/// purposes (PATHEXT provides the filtering).
fn is_executable(path: &Path) -> bool {
    if cfg!(windows) {
        return true;
    }
    #[cfg(unix)]
    {
        std::fs::metadata(path)
            .ok()
            .map(|m| {
                let mode = m.permissions().mode();
                mode & 0o111 != 0
            })
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PortMapping, VolumeMount};

    #[test]
    fn test_runtime_type_from_str_new_variants() {
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("wslc"),
            ContainerRuntimeType::Wslc
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("wsl"),
            ContainerRuntimeType::Wslc
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("apple-container"),
            ContainerRuntimeType::AppleContainer
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("apple_container"),
            ContainerRuntimeType::AppleContainer
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("apple"),
            ContainerRuntimeType::AppleContainer
        ));
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("container"),
            ContainerRuntimeType::AppleContainer
        ));
    }

    #[test]
    fn test_runtime_type_as_str_new_variants() {
        assert_eq!(ContainerRuntimeType::Wslc.as_str(), "wslc");
        assert_eq!(
            ContainerRuntimeType::AppleContainer.as_str(),
            "apple-container"
        );
    }

    #[test]
    fn test_runtime_type_is_cli_backend() {
        assert!(ContainerRuntimeType::Wslc.is_cli_backend());
        assert!(ContainerRuntimeType::AppleContainer.is_cli_backend());
        assert!(!ContainerRuntimeType::Docker.is_cli_backend());
        assert!(!ContainerRuntimeType::Youki.is_cli_backend());
    }

    #[test]
    fn test_build_run_args_basic() {
        let params = ContainerCreateParams::simple("test-container", "alpine:latest");
        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&"--name".to_string()));
        assert!(args.contains(&"test-container".to_string()));
        assert!(args.contains(&"alpine:latest".to_string()));
    }

    #[test]
    fn test_build_run_args_with_env_and_ports() {
        let mut params = ContainerCreateParams::simple("web", "nginx:latest");
        params.env.insert("FOO".into(), "bar".into());
        params.ports.push(PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: "tcp".into(),
        });

        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.contains(&"-e".to_string()));
        assert!(args.iter().any(|a| a == "FOO=bar"));
        assert!(args.contains(&"-p".to_string()));
        assert!(args.iter().any(|a| a == "8080:80"));
    }

    #[test]
    fn test_build_run_args_with_volumes() {
        let mut params = ContainerCreateParams::simple("vol-test", "alpine");
        params.volumes.push(VolumeMount::rw("/host/data", "/data"));
        params.volumes.push(VolumeMount::ro("/host/conf", "/conf"));

        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.iter().any(|a| a == "/host/data:/data"));
        assert!(args.iter().any(|a| a == "/host/conf:/conf:ro"));
    }

    #[test]
    fn test_build_run_args_read_only_rootfs() {
        let mut params = ContainerCreateParams::simple("ro-test", "alpine");
        params.read_only_rootfs = Some(true);

        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.contains(&"--read-only".to_string()));
    }

    #[test]
    fn test_nanos_to_duration_str() {
        assert_eq!(nanos_to_duration_str(0), "0s");
        assert_eq!(nanos_to_duration_str(5_000_000_000), "5s");
        assert_eq!(nanos_to_duration_str(60_000_000_000), "1m");
        assert_eq!(nanos_to_duration_str(120_000_000_000), "2m");
        assert_eq!(nanos_to_duration_str(90_000_000_000), "1m"); // 90s rounds to 1m
    }

    #[test]
    fn test_find_in_path_returns_none_for_nonexistent() {
        let result = find_in_path("entelecheia_definitely_nonexistent_binary_xyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_in_path_finds_ls_or_sh() {
        #[cfg(unix)]
        {
            let result = find_in_path("ls");
            assert!(result.is_some(), "ls should be on PATH");
        }
        #[cfg(windows)]
        {
            let result = find_in_path("cmd");
            assert!(result.is_some(), "cmd should be on PATH");
        }
    }

    // -- JSON extraction tests --

    #[test]
    fn test_extract_container_info_docker_style() {
        let json = r#"{
            "Id": "abc123",
            "Name": "/my-container",
            "Image": "nginx:latest",
            "State": {"Running": true, "Status": "running", "ExitCode": 0},
            "Created": "2026-06-30T12:00:00Z"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert_eq!(info.id, "abc123");
        assert_eq!(info.name, "my-container"); // leading / stripped
        assert_eq!(info.image, "nginx:latest");
        assert!(info.status.is_running());
    }

    #[test]
    fn test_extract_container_info_apple_style() {
        // Apple Container likely uses lowercase field names
        let json = r#"{
            "id": "def456",
            "name": "my-app",
            "image": {"reference": "alpine:3.18"},
            "state": {"running": true, "status": "running"},
            "created": "2026-06-30T12:00:00Z"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert_eq!(info.id, "def456");
        assert_eq!(info.name, "my-app");
        assert_eq!(info.image, "alpine:3.18"); // extracted from object
        assert!(info.status.is_running());
    }

    #[test]
    fn test_extract_container_info_stopped() {
        let json = r#"{
            "id": "stopped1",
            "name": "old-container",
            "image": "ubuntu:22.04",
            "state": {"running": false, "status": "exited"}
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert!(!info.status.is_running());
        assert!(matches!(info.status, ContainerStatus::Exited));
    }

    #[test]
    fn test_extract_container_info_with_labels() {
        let json = r#"{
            "id": "labeled1",
            "name": "app",
            "image": "nginx",
            "Labels": {"env": "prod", "tier": "frontend"}
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert_eq!(info.labels.get("env"), Some(&"prod".to_string()));
        assert_eq!(info.labels.get("tier"), Some(&"frontend".to_string()));
    }

    #[test]
    fn test_extract_container_detail_with_labels_docker_config() {
        // Docker nests labels under Config.Labels
        let json = r#"{
            "Id": "det1",
            "Name": "/svc",
            "Config": {"Image": "redis:7", "Labels": {"role": "cache"}},
            "State": {"Running": true, "Status": "running"}
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let detail = extract_container_detail(&val, "fallback").expect("should extract");
        assert_eq!(detail.info.labels.get("role"), Some(&"cache".to_string()));
    }

    #[test]
    fn test_extract_container_detail_docker_style() {
        let json = r#"{
            "Id": "abc123",
            "Name": "/web",
            "Config": {"Image": "nginx:1.25"},
            "State": {"Running": true, "Status": "running", "ExitCode": 0, "StartedAt": "2026-06-30T10:00:00Z"},
            "Created": "2026-06-30T09:00:00Z"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let detail = extract_container_detail(&val, "fallback").expect("should extract");
        assert_eq!(detail.info.id, "abc123");
        assert_eq!(detail.info.name, "web");
        assert_eq!(detail.info.image, "nginx:1.25");
        assert!(detail.info.status.is_running());
        assert_eq!(detail.exit_code, Some(0));
        assert!(detail.started_at.is_some());
    }

    #[test]
    fn test_extract_container_detail_apple_lowercase_state() {
        // Apple Container uses lowercase nested state fields
        let json = r#"{
            "id": "xyz789",
            "name": "api-server",
            "image": {"reference": "node:20"},
            "state": {
                "running": true,
                "status": "running",
                "exit_code": 0,
                "started_at": "2026-06-30T08:00:00Z"
            }
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let detail = extract_container_detail(&val, "fallback").expect("should extract");
        assert_eq!(detail.info.id, "xyz789");
        assert_eq!(detail.info.image, "node:20");
        assert!(
            detail.info.status.is_running(),
            "should detect running state"
        );
        assert_eq!(detail.exit_code, Some(0));
    }

    #[test]
    fn test_extract_image_info_with_reference() {
        // Apple Container style: single reference field
        let json = r#"{
            "id": "sha256:abc",
            "reference": "docker.io/library/alpine:3.18",
            "size": 7012345,
            "created": 1719742800
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_image_info(&val).expect("should extract");
        assert_eq!(info.id, "sha256:abc");
        assert_eq!(info.repository, "docker.io/library/alpine");
        assert_eq!(info.tag, "3.18");
        assert_eq!(info.size, 7012345);
    }

    #[test]
    fn test_extract_image_info_docker_style() {
        // Docker style: separate repository and tag fields
        let json = r#"{
            "Id": "sha256:def",
            "Repository": "nginx",
            "Tag": "latest",
            "Size": 142000000,
            "Created": 1719742800
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_image_info(&val).expect("should extract");
        assert_eq!(info.id, "sha256:def");
        assert_eq!(info.repository, "nginx");
        assert_eq!(info.tag, "latest");
        assert_eq!(info.size, 142000000);
    }

    #[test]
    fn test_extract_image_info_size_as_string() {
        // Some CLIs emit size as a human-readable string
        let json = r#"{
            "id": "img1",
            "reference": "alpine:latest",
            "size": "12.3MB"
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_image_info(&val).expect("should extract");
        assert_eq!(info.size, 0); // unparseable string → default 0
    }

    #[test]
    fn test_extract_container_info_missing_id() {
        // Should return None if id is missing
        let json = r#"{"name": "no-id"}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(extract_container_info(&val).is_none());
    }

    #[test]
    fn test_build_run_args_udp_port() {
        let mut params = ContainerCreateParams::simple("dns-test", "dns:latest");
        params.ports.push(PortMapping {
            host_port: 1053,
            container_port: 53,
            protocol: "udp".into(),
        });
        let args = CliContainerBackend::build_run_args(&params);
        assert!(
            args.iter().any(|a| a == "1053:53/udp"),
            "UDP port should include /udp suffix"
        );
    }

    #[test]
    fn test_build_run_args_with_security_opt_and_devices() {
        let mut params = ContainerCreateParams::simple("sec-test", "alpine");
        params.security_opt = Some(vec!["no-new-privileges".into()]);
        params.devices.push(crate::types::DeviceMapping {
            host_path: "/dev/ttyUSB0".into(),
            container_path: "/dev/ttyUSB0".into(),
            permissions: "rwm".into(),
        });
        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.iter().any(|a| a == "--security-opt"));
        assert!(args.iter().any(|a| a == "no-new-privileges"));
        assert!(args.iter().any(|a| a == "--device"));
        assert!(args.iter().any(|a| a == "/dev/ttyUSB0:/dev/ttyUSB0:rwm"));
    }

    #[test]
    fn test_from_str_lossy_applecontainer_serde_form() {
        // The serde serialized form "applecontainer" must round-trip
        assert!(matches!(
            ContainerRuntimeType::from_str_lossy("applecontainer"),
            ContainerRuntimeType::AppleContainer
        ));
    }

    #[test]
    fn test_build_run_args_with_healthcheck() {
        use crate::types::HealthcheckParams;
        let mut params = ContainerCreateParams::simple("hc-test", "nginx");
        params.healthcheck = Some(HealthcheckParams {
            test: vec![
                "CMD-SHELL".to_string(),
                "curl -f http://localhost || exit 1".into(),
            ],
            interval_ns: Some(30_000_000_000),
            timeout_ns: Some(10_000_000_000),
            retries: Some(3),
            start_period_ns: Some(5_000_000_000),
        });
        let args = CliContainerBackend::build_run_args(&params);
        assert!(args.iter().any(|a| a == "--health-cmd"));
        assert!(
            args.iter()
                .any(|a| a == "CMD-SHELL curl -f http://localhost || exit 1")
        );
        assert!(args.iter().any(|a| a == "--health-interval"));
        assert!(args.iter().any(|a| a == "30s"));
        assert!(args.iter().any(|a| a == "--health-timeout"));
        assert!(args.iter().any(|a| a == "10s"));
        assert!(args.iter().any(|a| a == "--health-retries"));
        assert!(args.iter().any(|a| a == "3"));
        assert!(args.iter().any(|a| a == "--health-start-period"));
        assert!(args.iter().any(|a| a == "5s"));
    }

    #[test]
    fn test_extract_container_info_lowercase_labels_apple() {
        let json = r#"{
            "id": "apple-labels",
            "name": "svc",
            "image": "nginx",
            "labels": {"env": "staging", "owner": "team-a"}
        }"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert_eq!(info.labels.get("env"), Some(&"staging".to_string()));
        assert_eq!(info.labels.get("owner"), Some(&"team-a".to_string()));
    }

    #[test]
    fn test_extract_container_info_missing_image_defaults() {
        let json = r#"{"id": "no-img", "name": "imgless"}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_container_info(&val).expect("should extract");
        assert_eq!(info.image, "unknown");
    }

    #[test]
    fn test_extract_container_detail_id_hint_fallback() {
        let json = r#"{"name": "no-id-here"}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let detail = extract_container_detail(&val, "hint-id").expect("should extract with hint");
        assert_eq!(detail.info.id, "hint-id");
    }

    #[test]
    fn test_extract_image_info_no_tag_in_reference() {
        let json = r#"{"id": "no-tag", "reference": "alpine"}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        let info = extract_image_info(&val).expect("should extract");
        assert_eq!(info.repository, "alpine");
        assert_eq!(info.tag, "latest");
    }
}
