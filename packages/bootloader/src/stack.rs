//! Container stack — bring up the entelecheia infrastructure (postgres,
//! scepter, registry) on the host.
//!
//! This module is the host-side responsibility lifted out of
//! `entelecheia-cli`'s `service.rs`: creating the shared docker network, the
//! ext4-backed socket directories (NOT tmpfs — see [`SOCKET_DIR_EXT4`]), and
//! the three managed containers. The CLI now delegates here, and evernight's
//! `supervise` command calls the same path so both share one implementation.
//!
//! Everything is parameterised by [`StackConfig`] — image names, ports, the
//! container-name prefix, the workspace path — so callers (the CLI doing a
//! source-build, evernight doing a release pull) can drive the same code
//! without entelecheia-specific assumptions baked in.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use bollard::config::NetworkCreateRequest;
use tracing::{info, warn};

use _container::{
    ContainerCreateParams, HealthcheckParams, VolumeMount,
    ops::ContainerOps,
    security_profile,
    types::{ContainerRuntimeType, DeviceMapping, PortMapping},
};

/// The shared bridge network all three containers attach to.
pub const NETWORK_NAME: &str = "entelecheia-network";

/// How long to wait for a container healthcheck to turn green.
pub const HEALTH_TIMEOUT: Duration = Duration::from_secs(120);

/// Socket directory on ext4. Unix domain sockets on tmpfs bind mounts are not
/// connectable from Docker containers (ECONNREFUSED despite the file being
/// visible), so this MUST live on ext4 — never `/run/entelecheia` (tmpfs).
///
/// Overridable via `EVERNIGHT_SOCKET_DIR` env for environments where `/tmp` is
/// not visible to rootless containers (e.g. podman-machine).
///
/// (Original evidence: entelecheia service.rs:299-301, evernight_daemon.rs.)
pub fn socket_dir_ext4() -> String {
    std::env::var("EVERNIGHT_SOCKET_DIR")
        .unwrap_or_else(|_| "/tmp/entelecheia-unix-socket".to_string())
}

/// tmpfs socket dir — used by the TUI unix socket and bind-mounted into the
/// scepter container, but NOT for evernight's IPC socket (see socket_dir_ext4).
///
/// Overridable via `ENTELECHEIA_SOCKET_DIR` env.
pub fn socket_dir_tmpfs() -> String {
    std::env::var("ENTELECHEIA_SOCKET_DIR")
        .unwrap_or_else(|_| "/run/entelecheia".to_string())
}

/// Configuration for a stack bring-up. Every field has a sensible default so a
/// caller can override only what it cares about.
#[derive(Debug, Clone)]
pub struct StackConfig {
    /// Container-name prefix, e.g. `e-042-`. Yields `e-042-postgres` etc.
    pub prefix: String,
    /// Host port for postgres (mapped to container 5432).
    pub postgres_port: u16,
    /// Host port for scepter's HTTP/WS API.
    pub scepter_port: u16,
    /// Postgres credentials.
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    /// Image refs.
    pub postgres_image: String,
    pub scepter_image: String,
    pub registry_image: String,
    /// Container backend runtime: `docker` / `youki` / `wslc` /
    /// `apple-container` (resolved via [`ContainerRuntimeType`]).
    pub container_backend: String,
    /// When true, mount the host-built `target/debug/scepter` binary and the
    /// current workspace into the scepter container (dev mode).
    pub source_build: bool,
    /// Optional read-only model-cache directory mounted at `/models`.
    pub model_cache_dir: Option<PathBuf>,
    /// Optional host workspace to bind-mount at `/workspace` (dev mode).
    pub workspace_dir: Option<PathBuf>,
    /// Optional host directory holding the entelecheia config; bind-mounted
    /// read-write into `/home/entelecheia/.config/entelecheia`.
    pub config_dir: Option<PathBuf>,
}

impl Default for StackConfig {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("entelecheia");
        Self {
            prefix: "e-".to_string(),
            postgres_port: 5432,
            scepter_port: 8424,
            postgres_user: "entelecheia".to_string(),
            postgres_password: String::new(),
            postgres_db: "entelecheia".to_string(),
            postgres_image: "pgvector/pgvector:pg18-bookworm".to_string(),
            scepter_image: "entelecheia:latest".to_string(),
            registry_image: "registry:2".to_string(),
            container_backend: "docker".to_string(),
            source_build: false,
            model_cache_dir: None,
            workspace_dir: None,
            config_dir: Some(config_dir),
        }
    }
}

/// Opaque handle returned by [`bring_up_stack`]. Keeps the backend alive and
/// remembers the container names so [`teardown_stack`] can stop them in order.
pub struct StackHandle {
    backend: Box<dyn ContainerOps>,
    pub scepter_port: u16,
    pub postgres_port: u16,
    /// Container names in dependency order (postgres first, scepter last).
    pub container_names: Vec<String>,
}

impl StackHandle {
    /// Borrow the underlying backend (for health probes etc).
    pub fn backend(&self) -> &dyn ContainerOps {
        self.backend.as_ref()
    }
}

fn secs_to_ns(secs: i64) -> Option<i64> {
    Some(secs * 1_000_000_000)
}

/// Build a container backend from the configured runtime string.
async fn create_container_ops(backend: &str) -> Result<Box<dyn ContainerOps>> {
    let runtime = ContainerRuntimeType::from_str_lossy(backend);
    let data_dir = _infra_services::container_factory::default_container_data_dir();
    _infra_services::container_factory::create_container_backend(runtime, &data_dir)
        .await
        .map_err(|e| anyhow!("failed to initialize {runtime} backend: {e}"))
}

/// Create `/run/entelecheia` (tmpfs) and `/tmp/entelecheia-unix-socket` (ext4),
/// world-writable so uid 1000 inside the scepter container can bind to them.
///
/// On non-Unix (Windows) this is a no-op: the containers run inside the
/// podman-machine / WSL2 VM, so these Linux paths exist inside the VM, not on
/// the Windows host. Podman creates the mountpoints automatically when the
/// container is created with a bind mount.
pub async fn ensure_socket_dirs() -> Result<()> {
    #[cfg(unix)]
    {
        for dir in [socket_dir_tmpfs(), socket_dir_ext4()] {
            let p = Path::new(dir);
            if !p.exists() {
                std::fs::create_dir_all(p)
                    .with_context(|| format!("failed to create socket dir {dir}"))?;
            }
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o777));
        }
    }
    #[cfg(not(unix))]
    {
        // Windows: socket dirs live inside the podman-machine VM. Podman creates
        // bind-mount mountpoints on container creation. Nothing to do on the host.
        tracing::debug!("ensure_socket_dirs: skipped on non-Unix (containers run in VM)");
    }
    Ok(())
}

/// Ensure the shared bridge network exists (create if absent, idempotent).
pub async fn ensure_network() -> Result<()> {
    let docker = bollard::Docker::connect_with_local_defaults()
        .map_err(|e| anyhow!("failed to connect to Docker: {e}"))?;

    let networks = docker
        .list_networks(None::<bollard::query_parameters::ListNetworksOptions>)
        .await
        .map_err(|e| anyhow!("failed to list networks: {e}"))?;

    let exists = networks
        .iter()
        .any(|n| n.name.as_deref() == Some(NETWORK_NAME));

    if !exists {
        info!("creating network '{NETWORK_NAME}'");
        docker
            .create_network(NetworkCreateRequest {
                name: NETWORK_NAME.to_string(),
                driver: Some("bridge".to_string()),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow!("failed to create network: {e}"))?;
    }
    Ok(())
}

fn postgres_params(cfg: &StackConfig) -> ContainerCreateParams {
    let name = format!("{}postgres", cfg.prefix);
    let mut env = HashMap::new();
    env.insert("POSTGRES_USER".into(), cfg.postgres_user.clone());
    env.insert("POSTGRES_PASSWORD".into(), cfg.postgres_password.clone());
    env.insert("POSTGRES_DB".into(), cfg.postgres_db.clone());

    let sec = security_profile::postgres();

    ContainerCreateParams {
        name,
        image: cfg.postgres_image.clone(),
        network: Some(NETWORK_NAME.into()),
        env,
        ports: vec![PortMapping {
            host_port: cfg.postgres_port,
            container_port: 5432,
            protocol: "tcp".into(),
        }],
        volumes: vec![VolumeMount::rw(
            format!("{}postgres-data", cfg.prefix),
            "/var/lib/postgresql",
        )],
        healthcheck: Some(HealthcheckParams {
            test: vec![
                "CMD-SHELL".into(),
                format!("pg_isready -U {}", cfg.postgres_user),
            ],
            interval_ns: secs_to_ns(5),
            timeout_ns: secs_to_ns(3),
            retries: Some(5),
            start_period_ns: secs_to_ns(10),
        }),
        labels: HashMap::from([("managed-by".into(), "celestia-bootloader".into())]),
        cap_drop: sec.cap_drop,
        cap_add: sec.cap_add,
        security_opt: sec.security_opt,
        egress_policy: sec.egress_policy,
        ..ContainerCreateParams::simple("", "")
    }
}

fn registry_params(cfg: &StackConfig) -> ContainerCreateParams {
    ContainerCreateParams {
        name: format!("{}registry", cfg.prefix),
        image: cfg.registry_image.clone(),
        network: Some(NETWORK_NAME.into()),
        env: HashMap::new(),
        ports: vec![],
        volumes: vec![],
        devices: vec![],
        healthcheck: Some(HealthcheckParams {
            test: vec![
                "CMD".into(),
                "wget".into(),
                "-q".into(),
                "-O-".into(),
                "http://localhost:5000/".into(),
            ],
            interval_ns: secs_to_ns(10),
            timeout_ns: secs_to_ns(5),
            retries: Some(10),
            start_period_ns: secs_to_ns(5),
        }),
        labels: HashMap::from([("managed-by".into(), "celestia-bootloader".into())]),
        ..ContainerCreateParams::simple(format!("{}registry", cfg.prefix), &cfg.registry_image)
    }
}

fn scepter_params(cfg: &StackConfig) -> ContainerCreateParams {
    let name = format!("{}scepter", cfg.prefix);
    let db_host = format!("{}postgres", cfg.prefix);
    let db_url = format!(
        "postgresql://{}:{}@{}/{}",
        cfg.postgres_user, cfg.postgres_password, db_host, cfg.postgres_db
    );

    let mut env = HashMap::new();
    env.insert("DATABASE_URL".into(), db_url);
    env.insert("RUST_LOG".into(), "debug".into());
    env.insert("HOME".into(), "/home/entelecheia".into());
    env.insert(
        "SERVER_BIND_ADDRESS".into(),
        format!("0.0.0.0:{}", cfg.scepter_port),
    );
    env.insert("CONTAINER_PREFIX".into(), cfg.prefix.clone());
    // Cosmos containers use the docker socket we mount below — not youki/fuse.
    env.insert("COSMOS_CONTAINER_RUNTIME".into(), "docker".into());
    // Local image names (no registry prefix) for cosmos sub-containers.
    env.insert("CONTAINER_REGISTRY".into(), String::new());
    env.insert("RBAC_ENABLED".into(), "false".into());
    // Proxy passthrough — evernight provides host proxy access via polemos.
    if let Ok(proxy) = std::env::var("HTTP_PROXY").or(std::env::var("http_proxy")) {
        env.insert("HTTP_PROXY".into(), proxy.clone());
        env.insert("HTTPS_PROXY".into(), proxy);
    }

    let tmpfs_dir = socket_dir_tmpfs();
    let ext4_dir = socket_dir_ext4();
    let mut volumes = vec![
        // Docker socket — scepter spawns cosmos sub-containers through it.
        VolumeMount::rw("/var/run/docker.sock", "/var/run/docker.sock"),
        // tmpfs socket dir (TUI unix socket lives here).
        VolumeMount::rw(&tmpfs_dir, &tmpfs_dir),
        // ext4 socket dir (evernight IPC — must be ext4).
        VolumeMount::rw(&ext4_dir, &ext4_dir),
    ];

    if let Some(ref dir) = cfg.config_dir {
        volumes.push(VolumeMount::rw(
            dir.to_string_lossy(),
            "/home/entelecheia/.config/entelecheia",
        ));
    }
    if let Some(ref dir) = cfg.model_cache_dir {
        volumes.push(VolumeMount::ro(dir.to_string_lossy(), "/models"));
    }
    if cfg.source_build {
        if let Some(ref ws) = cfg.workspace_dir {
            let bin = ws.join("target/debug/scepter");
            if bin.exists() {
                info!(
                    path = %bin.display(),
                    "mounting dev scepter binary into container"
                );
                volumes.push(VolumeMount::ro(
                    bin.to_string_lossy(),
                    "/usr/local/bin/scepter",
                ));
            }
            let ws_str = ws.to_string_lossy().to_string();
            env.insert("HOST_WORKSPACE_PATH".into(), ws_str.clone());
            volumes.push(VolumeMount::rw(ws_str, "/workspace"));
        }
    }

    // Inject the docker socket GID so the scepter container can talk to dockerd.
    let group_add: Option<Vec<String>> = std::fs::metadata("/var/run/docker.sock")
        .ok()
        .and_then(|m| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let gid = m.gid();
                if gid != 0 {
                    Some(vec![gid.to_string()])
                } else {
                    None
                }
            }
            #[cfg(not(unix))]
            {
                let _ = m;
                None
            }
        });
    if let Some(ref groups) = group_add {
        info!(
            gid = %groups.first().unwrap_or(&"0".to_string()),
            "adding docker socket GID to scepter group_add"
        );
    }

    let sec = security_profile::scepter();

    ContainerCreateParams {
        name,
        image: cfg.scepter_image.clone(),
        network: Some(NETWORK_NAME.into()),
        env,
        ports: vec![PortMapping {
            host_port: cfg.scepter_port,
            container_port: cfg.scepter_port,
            protocol: "tcp".into(),
        }],
        volumes,
        devices: vec![DeviceMapping {
            host_path: "/dev/fuse".into(),
            container_path: "/dev/fuse".into(),
            permissions: "rwm".into(),
        }],
        healthcheck: Some(HealthcheckParams {
            test: vec![
                "CMD-SHELL".into(),
                format!("test -S {}/entelecheia-tui.sock", socket_dir_tmpfs()),
            ],
            interval_ns: secs_to_ns(5),
            timeout_ns: secs_to_ns(3),
            retries: Some(30),
            start_period_ns: secs_to_ns(10),
        }),
        labels: HashMap::from([("managed-by".into(), "celestia-bootloader".into())]),
        cap_drop: sec.cap_drop,
        cap_add: sec.cap_add,
        security_opt: sec.security_opt,
        egress_policy: sec.egress_policy,
        group_add,
        ..ContainerCreateParams::simple("", "")
    }
}

/// Bring up the full stack: sockets → network → pull images → create/start
/// containers → wait healthy. Returns a handle that keeps the backend alive.
///
/// Idempotent: already-running containers are left alone, stopped ones are
/// started, missing ones are created. Safe to call on every boot.
pub async fn bring_up_stack(cfg: &StackConfig) -> Result<StackHandle> {
    if cfg.postgres_password.is_empty() {
        return Err(anyhow!(
            "postgres_password is empty — set POSTGRES_PASSWORD or the bootloader config"
        ));
    }

    let backend = create_container_ops(&cfg.container_backend).await?;

    ensure_socket_dirs().await?;
    ensure_network().await?;

    // Pull images first (source-build callers handle this themselves).
    if !cfg.source_build {
        for image in [&cfg.postgres_image, &cfg.scepter_image, &cfg.registry_image] {
            if backend.image_exists(image).await.unwrap_or(false) {
                info!("{image} — already available locally");
            } else {
                info!("pulling {image} ...");
                if let Err(e) = backend.pull_image(image).await {
                    warn!("pull {image} — failed: {e}");
                } else {
                    info!("pulled {image}");
                }
            }
        }
    }

    let specs = vec![
        postgres_params(cfg),
        scepter_params(cfg),
        registry_params(cfg),
    ];

    let mut container_names = Vec::with_capacity(specs.len());
    info!("creating / starting containers...");
    for spec in &specs {
        let name = spec.name.clone();
        container_names.push(name.clone());
        match backend.inspect(&name).await {
            Ok(existing) => {
                if existing.info.status.is_running() {
                    info!("{name} — already running");
                } else {
                    info!("{name} — starting");
                    backend
                        .start(&name)
                        .await
                        .map_err(|e| anyhow!("failed to start {name}: {e}"))?;
                }
            }
            Err(_) => {
                info!("{name} — creating + starting");
                backend
                    .create(spec)
                    .await
                    .map_err(|e| anyhow!("failed to create {name}: {e}"))?;
            }
        }
    }

    info!("waiting for health checks...");
    for name in &container_names {
        match backend.wait_healthy(name, HEALTH_TIMEOUT).await {
            Ok(_) => info!("{name} — healthy ✓"),
            Err(e) => warn!("{name} health check failed: {e}"),
        }
    }

    Ok(StackHandle {
        backend,
        scepter_port: cfg.scepter_port,
        postgres_port: cfg.postgres_port,
        container_names,
    })
}

/// Stop the three containers in reverse dependency order (scepter first,
/// postgres last). Does not remove them — a subsequent [`bring_up_stack`] will
/// start them again.
pub async fn teardown_stack(handle: &StackHandle) -> Result<()> {
    info!("stopping entelecheia stack...");
    for name in handle.container_names.iter().rev() {
        match handle.backend.stop(name).await {
            Ok(_) => info!("{name} — stopped"),
            Err(e) => warn!("{name} — {e}"),
        }
    }
    info!("stack stopped.");
    Ok(())
}
