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
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(unix)]
use anyhow::Context;
use anyhow::{Result, anyhow};
use bollard::config::NetworkCreateRequest;
use tracing::{info, warn};

use cherino::{
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
    std::env::var("ENTELECHEIA_SOCKET_DIR").unwrap_or_else(|_| "/run/entelecheia".to_string())
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
    /// Optional host directory containing the celestia monorepo (all project
    /// repositories). When set, bind-mounts as `/celestia` (read-only) into
    /// the scepter container so agents can read PLAN.md / source files and
    /// iterate on project code.
    pub host_repo_root: Option<PathBuf>,
    /// Network mode: None (default) uses the `entelecheia-network` bridge;
    /// `Some("host")` uses host networking (no port mapping, useful when the
    /// container runtime's bridge network is unavailable, e.g. podman+nftables).
    pub network_mode: Option<String>,
    /// Evernight IPC lane — host path of the evernight broker's unix socket
    /// (e.g. `/tmp/entelecheia-unix-socket/evernight-e-042.sock`). When set,
    /// injected into the scepter container env as `EVERNIGHT_SOCK`. The ext4
    /// socket dir is already bind-mounted at the same path inside the
    /// container, so the host path works verbatim. Set together with
    /// [`StackConfig::evernight_token`].
    pub evernight_sock: Option<String>,
    /// Evernight IPC lane — the dispatch token scepter presents as
    /// `params.auth` when calling the broker. When set, injected into the
    /// scepter container env as `EVERNIGHT_TOKEN`. This is a credential: it
    /// travels spec → container env only and is deliberately NOT read from
    /// the host process env here (unlike the HTTP_PROXY passthrough above).
    /// Set together with [`StackConfig::evernight_sock`].
    pub evernight_token: Option<String>,
    /// Opt-in: mount the host docker socket into the scepter container.
    ///
    /// The socket is root-equivalent on the host (a container escape or code
    /// execution inside scepter can spawn privileged containers and read any
    /// host path). The 2026-11-06 security audit (R-01) therefore made it
    /// **default off**: a stack built without this flag has no docker socket
    /// and no docker GID, and scepter's cosmos sub-container spawn fails
    /// loudly instead of silently holding host root. Set
    /// `EVERNIGHT_ALLOW_DOCKER_SOCKET=1` (supervise) or the field directly
    /// only when the host-root tradeoff is accepted.
    pub allow_docker_socket: bool,
    /// Opt-in: mount a **proxy** docker socket instead of the raw host one.
    ///
    /// The value is a host path (for example the unix socket exposed by a
    /// docker-socket-proxy bridge) and it is mounted at the container's default
    /// `/var/run/docker.sock`, so bollard finds it with no `DOCKER_HOST` and no
    /// image or client change.
    ///
    /// This is the **sanctioned alternative** to [`StackConfig::allow_docker_socket`]:
    /// the raw socket stays default-off (R-01, 2026-11-06 audit: it is
    /// root-equivalent on the host), while a proxy socket only reaches the API
    /// surface the proxy allows. [`bring_up_stack`] refuses the ambiguous and the
    /// unsafe shapes rather than ranking them: both sources set ("both" means
    /// nobody knows which daemon the stack talks to), a non-absolute path (a
    /// relative value is a *named volume* to the container runtime, not a bind —
    /// the stack would come up pointing at a socket that does not exist), a path
    /// that *resolves* to the raw host socket (that is the R-01 opt-in's job, under
    /// its own name and its own tradeoff), and a directory (mounting one at the
    /// socket path exposes every socket beneath it).
    ///
    /// "Resolves" is answered two ways, so read this before trusting it: lexically,
    /// which covers `..`, `.` and doubled separators and works before the socket
    /// exists; and, once both paths exist, through `canonicalize`, which is what
    /// catches a symlinked parent — where `/var/run` is a symlink, `/run/docker.sock`
    /// *is* the raw socket under another spelling. What still gets through is a
    /// dangling symlink that only resolves after this validation runs. So this is a
    /// guard against operator mistakes, not against someone who can already write
    /// the config.
    ///
    /// Rollout — read this before restarting anything: [`bring_up_stack`] is
    /// **idempotent**. A running container is left alone, and a stopped one is
    /// merely started with the spec it already has, so editing the supervisor
    /// environment and restarting the unit applies **nothing** to a stack that is
    /// already up and warns about nothing. The operator must also free the old
    /// container's published host port (a rename alone keeps it bound, and the new
    /// container's start then fails with "port is already allocated"): stop the
    /// old container, rename it aside as the rollback body, then start the
    /// supervisor. This paragraph gives the *shape*; the PR's rollout note gives
    /// the literal commands. If they ever disagree, the note wins — and the socket
    /// you publish must be connectable by the container's user (a `0600` root socket
    /// is not: set `0666`, or `root:<non-zero gid> 0660` so the gid arm can inject
    /// that group).
    pub docker_socket_path: Option<PathBuf>,
    /// Opt-in: the host directory scepter keeps its per-container workspace roots
    /// under, bind-mounted at the **same path** inside the scepter container.
    ///
    /// Why identity, and not a fixed container path: scepter puts
    /// `COSMOS_WORKSPACE_HOST_DIR` straight into the Docker API as a *bind
    /// source*, and the daemon resolves that string on the **host**. A
    /// containerised scepter whose data home is its own filesystem therefore names
    /// a path the daemon cannot see — live on node-1, 2026-11-15: `400 invalid
    /// mount config for type "bind": bind source path does not exist:
    /// /home/entelecheia/.local/share/snowflake-ws/<uuid>`. Mounting the host
    /// directory at the identical path makes the two sides agree by construction.
    ///
    /// `None` keeps today's behaviour exactly (scepter's own data home), which is
    /// correct only when scepter runs on the host.
    ///
    /// [`bring_up_stack`] refuses a relative path, one containing `..`, a symlink,
    /// the filesystem root, the docker socket itself, a directory **containing** that
    /// socket, a non-UTF-8 path, the same path as
    /// [`StackConfig::docker_socket_path`], and anything that is not a directory once
    /// created. The mount is **read-write** on purpose: scepter creates
    /// `<root>/<uuid>` through it and reclaim deletes through it.
    ///
    /// The directory is set to `0o1777` **unconditionally** — world-writable and
    /// sticky like the socket directories, but with the sticky bit and the
    /// unconditional normalisation added here on purpose): world-writable
    /// because scepter creates its per-container subdirectories through the bind as
    /// whatever uid the image runs as — the bootloader takes no uid parameter and
    /// should not hardcode one — and **sticky** because those subdirectories become
    /// the agent containers' `/workspace` roots, so no peer uid may rename or
    /// substitute another user's entry. An existing directory is normalised too: an
    /// operator who pre-creates it as root would otherwise leave the fix's own
    /// precondition unmet, and scepter only warns when its `create_dir_all` fails.
    ///
    /// Rollout — the same caveat as [`StackConfig::docker_socket_path`]:
    /// [`bring_up_stack`] is idempotent, so a container that already exists keeps
    /// the spec it was created with. Editing the supervisor environment and
    /// restarting the unit applies **neither** this mount **nor** the env var, and
    /// says nothing; the original 400 survives to the first write-mode skill. Stop
    /// the supervisor, **stop** the old container (a rename alone keeps its
    /// published port bound, and the new container's start then fails with "port is
    /// already allocated", so the mount never applies and the doc looks wrong),
    /// rename it aside, then start the supervisor — that move is also the rollback.
    pub snowflake_ws_dir: Option<PathBuf>,
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
            host_repo_root: None,
            network_mode: None,
            evernight_sock: None,
            evernight_token: None,
            allow_docker_socket: false,
            docker_socket_path: None,
            snowflake_ws_dir: None,
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
    let data_dir = plana_infra_services::container_factory::default_container_data_dir();
    plana_infra_services::container_factory::create_container_backend(runtime, &data_dir)
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
            let p = Path::new(dir.as_str());
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

    let network = cfg
        .network_mode
        .clone()
        .unwrap_or_else(|| NETWORK_NAME.into());
    let is_host = network == "host";
    ContainerCreateParams {
        name,
        image: cfg.postgres_image.clone(),
        network: Some(network),
        env,
        ports: if is_host {
            vec![]
        } else {
            vec![PortMapping {
                host_port: cfg.postgres_port,
                container_port: 5432,
                protocol: "tcp".into(),
            }]
        },
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
        network: Some(
            cfg.network_mode
                .clone()
                .unwrap_or_else(|| NETWORK_NAME.into()),
        ),
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
    let is_host = cfg.network_mode.as_deref() == Some("host");
    let db_host = if is_host {
        "127.0.0.1".to_string()
    } else {
        format!("{}postgres", cfg.prefix)
    };
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
    // Read lane: advertise the monorepo's HOST path so scepter's cosmos spawn
    // path (snowflake_manager) can mount it READ-ONLY into every agent
    // container at the same /celestia path mounted below. Without this the
    // main scepter container sees /celestia but the cosmos agents that
    // actually execute skills do not (2026-10-08: the node-1 self-iteration
    // engine could not read the workspace at all).
    if let Some(ref repo_root) = cfg.host_repo_root {
        env.insert(
            "COSMOS_REPO_READONLY".into(),
            repo_root.display().to_string(),
        );
    }
    // Authorization is unconditional: the stack ships no RBAC off-switch.
    // Proxy passthrough — evernight provides host proxy access via polemos.
    if let Ok(proxy) = std::env::var("HTTP_PROXY").or(std::env::var("http_proxy")) {
        env.insert("HTTP_PROXY".into(), proxy.clone());
        env.insert("HTTPS_PROXY".into(), proxy);
    }

    // Evernight IPC lane (2026-10-23 loop-pilot follow-up): scepter — and the
    // cosmos sub-containers that inherit scepter's env via snowflake_manager —
    // need EVERNIGHT_SOCK / EVERNIGHT_TOKEN to reach the host evernight
    // broker. Before this, the pair only existed in hand-rebuilt containers
    // and was silently lost on every buffer/fork or stack rebuild. Values
    // come from explicit StackConfig fields, NOT a host-env passthrough: the
    // token is a credential whose only delivery path into the container is
    // the spec, and the caller (evernight supervise) owns how it is sourced.
    if let Some(ref sock) = cfg.evernight_sock {
        env.insert("EVERNIGHT_SOCK".into(), sock.clone());
    }
    if let Some(ref token) = cfg.evernight_token {
        env.insert("EVERNIGHT_TOKEN".into(), token.clone());
    }
    if cfg.evernight_sock.is_some() != cfg.evernight_token.is_some() {
        warn!(
            sock_set = cfg.evernight_sock.is_some(),
            token_set = cfg.evernight_token.is_some(),
            "evernight lane is half-configured — the broker will be unreachable \
             or reject auth inside the scepter container"
        );
    }

    let tmpfs_dir = socket_dir_tmpfs();
    let ext4_dir = socket_dir_ext4();
    let mut volumes = vec![];
    if let Some(proxy_socket) = cfg.docker_socket_path.as_ref() {
        // Proxy socket (docker-socket-proxy): mounted at the container's default
        // docker path so bollard needs no DOCKER_HOST. The proxy is the security
        // boundary; the raw socket stays unmounted (R-01 default).
        info!(
            proxy_socket = %proxy_socket.display(),
            "docker socket mounted from a proxy path (allowlisted surface; raw host socket stays unmounted)"
        );
        volumes.push(VolumeMount::rw(
            proxy_socket.display().to_string(),
            "/var/run/docker.sock",
        ));
    } else if cfg.allow_docker_socket {
        // Docker socket — scepter spawns cosmos sub-containers through it.
        // Opt-in (R-01): root-equivalent on the host; see StackConfig.
        volumes.push(VolumeMount::rw(RAW_DOCKER_SOCKET, "/var/run/docker.sock"));
    } else {
        info!(
            "docker socket NOT mounted (R-01 default): cosmos sub-container spawn is disabled until EVERNIGHT_ALLOW_DOCKER_SOCKET=1"
        );
    }
    volumes.push(
        // tmpfs socket dir (TUI unix socket lives here).
        VolumeMount::rw(&tmpfs_dir, &tmpfs_dir),
    );
    volumes.push(
        // ext4 socket dir (evernight IPC — must be ext4).
        VolumeMount::rw(&ext4_dir, &ext4_dir),
    );

    if let Some(ref dir) = cfg.config_dir {
        volumes.push(VolumeMount::rw(
            dir.to_string_lossy(),
            "/home/entelecheia/.config/entelecheia",
        ));
    }
    if let Some(ref repo_root) = cfg.host_repo_root {
        info!(
            path = %repo_root.display(),
            "bind-mounting celestia monorepo into scepter container"
        );
        volumes.push(VolumeMount::ro(repo_root.to_string_lossy(), "/celestia"));
    }
    // ALSO mount the monorepo at its OWN host path (read-only) inside the
    // main scepter container: scepter's cosmos spawn path guards on
    // `Path::is_dir(COSMOS_REPO_READONLY)` evaluated INSIDE this container,
    // so without this same-path view the guard is silently false in a
    // production stack (the host path does not otherwise exist here) and no
    // agent container gets the read lane (R3 review F1, 2026-10-08).
    if let Some(ref repo_root) = cfg.host_repo_root {
        volumes.push(VolumeMount::ro(
            repo_root.to_string_lossy(),
            repo_root.display().to_string(),
        ));
    }
    if let Some(ref dir) = cfg.model_cache_dir {
        volumes.push(VolumeMount::ro(dir.to_string_lossy(), "/models"));
    }
    if cfg.source_build
        && let Some(ref ws) = cfg.workspace_dir
    {
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

    if let Some(ref ws) = cfg.snowflake_ws_dir {
        // Identity bind: host path == container path, and that is the whole fix.
        // Scepter hands `COSMOS_WORKSPACE_HOST_DIR` to the Docker API as a bind
        // *source*; the daemon resolves it on the HOST. A containerised scepter
        // whose data home is its own filesystem therefore names a path the daemon
        // cannot see (node-1, 2026-11-15: `400 ... bind source path does not
        // exist`). The contract is identity **plus a mountpoint the container can
        // write** — Docker creates the target, and scepter then does
        // `create_dir_all(<root>/<uuid>)` through it, so a path whose mountpoint
        // the container cannot write fails just as surely as no mount at all.
        let ws = ws.to_string_lossy().to_string();
        info!(
            path = %ws,
            "snowflake workspace root mounted identity (host path == container path)"
        );
        volumes.push(VolumeMount::rw(&ws, &ws));
        env.insert("COSMOS_WORKSPACE_HOST_DIR".into(), ws);
    }

    // Inject the socket's GID so the scepter container can talk to the daemon —
    // only when a socket is mounted, and from the source that is actually mounted.
    // A proxy socket owned by a non-root group is the normal shape, and without
    // this the container's user simply cannot connect to it (no knob for that
    // group); reading the raw path while a proxy path is mounted would inject the
    // wrong GID and look like it worked.
    let mounted_socket: Option<&Path> = match cfg.docker_socket_path.as_deref() {
        Some(proxy) => Some(proxy),
        None if cfg.allow_docker_socket => Some(Path::new(RAW_DOCKER_SOCKET)),
        None => None,
    };
    let group_add: Option<Vec<String>> = mounted_socket.and_then(|socket| {
        let meta = match std::fs::metadata(socket) {
            Ok(meta) => meta,
            Err(error) => {
                // Not fatal — there is simply no gid to inject — but staying silent
                // here is how the old wrong-source bug looked like success.
                warn!(
                    socket = %socket.display(),
                    %error,
                    "cannot stat the mounted docker socket; no GID injected — the container user may be unable to connect to it"
                );
                return None;
            }
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let gid = meta.gid();
            if gid != 0 {
                Some(vec![gid.to_string()])
            } else {
                None
            }
        }
        #[cfg(not(unix))]
        {
            let _ = meta;
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

    let net = cfg
        .network_mode
        .clone()
        .unwrap_or_else(|| NETWORK_NAME.into());
    let is_host = net == "host";

    ContainerCreateParams {
        name,
        image: cfg.scepter_image.clone(),
        network: Some(net),
        env,
        ports: if is_host {
            vec![]
        } else {
            vec![PortMapping {
                host_port: cfg.scepter_port,
                container_port: cfg.scepter_port,
                protocol: "tcp".into(),
            }]
        },
        volumes,
        devices: vec![DeviceMapping {
            host_path: "/dev/fuse".into(),
            container_path: "/dev/fuse".into(),
            permissions: "rwm".into(),
        }],
        healthcheck: Some(HealthcheckParams {
            test: vec![
                "CMD-SHELL".into(),
                format!(
                    "curl -sf http://localhost:{}/health 2>/dev/null || curl -sf http://localhost:{}/",
                    cfg.scepter_port, cfg.scepter_port
                ),
            ],
            interval_ns: secs_to_ns(5),
            timeout_ns: secs_to_ns(5),
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

/// Validate — and, when configured, create — the snowflake workspace root.
///
/// Separate from [`socket_source_error`] because this one has a side effect: the
/// host directory must exist before the identity bind is created, and it must be
/// writable by the uid the scepter image runs as. That uid is `1000` today and is
/// recorded elsewhere in this file, but the bootloader takes no uid parameter and
/// should not hardcode one image's value, so the directory is made `0o1777` —
/// world-writable for that uid, sticky so no *other* uid can rename or substitute
/// an entry. The mode is applied **unconditionally**, including to a directory the
/// operator pre-created: scepter only warns when its `create_dir_all` fails, so an
/// unwritable root would resurface as the original 400 at the first write-mode
/// skill rather than at bring-up.
fn prepare_snowflake_ws_dir(cfg: &StackConfig) -> Result<Option<PathBuf>> {
    let Some(dir) = cfg.snowflake_ws_dir.as_ref() else {
        return Ok(None);
    };
    if dir.as_os_str().is_empty() {
        return Err(anyhow!(
            "snowflake_ws_dir is empty; leave the field unset to keep scepter's own data home"
        ));
    }
    if !dir.is_absolute() {
        return Err(anyhow!(
            "snowflake_ws_dir must be an absolute host path, got {:?} — scepter passes it \
             to the Docker API as a bind source, which the daemon resolves on the host",
            dir.display()
        ));
    }
    if dir
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(anyhow!(
            "snowflake_ws_dir must not contain `..` ({:?}); the path is used verbatim on both \
             sides of the bind, so an escape makes the identity ambiguous",
            dir.display()
        ));
    }
    if dir.to_str().is_none() {
        return Err(anyhow!(
            "snowflake_ws_dir is not valid UTF-8 ({:?}); it is passed to the Docker API as a \
             string, so a lossy rendering would name a different path",
            dir.display()
        ));
    }
    // Everything below runs on the lexically normalised spelling so that a trailing
    // separator cannot hide the leaf (`symlink_metadata("/var/run/")` follows it).
    let leaf = normalize_lexically(dir);
    if leaf == Path::new("/") {
        return Err(anyhow!(
            "snowflake_ws_dir must not be the filesystem root ({:?}) — it is mounted \
             read-write at the same path inside scepter, so `/` would mask the container's \
             whole rootfs",
            leaf.display()
        ));
    }
    if std::fs::symlink_metadata(&leaf)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(anyhow!(
            "snowflake_ws_dir {:?} is a symlink; `chmod` and the Docker daemon both follow it, \
             so the path this guard validated is not the directory that would be mounted — \
             point the field at the real directory (and if that directory contains the \
             docker socket, it is refused on its own account)",
            leaf.display()
        ));
    }
    // Identity refusals run **before** creation: creating a path that is then refused
    // would leave a directory behind, and if the path is the daemon's socket path the
    // daemon could no longer bind it.
    if let Some(problem) = socket_bearing_error(&leaf) {
        return Err(anyhow!(problem));
    }
    if cfg
        .docker_socket_path
        .as_ref()
        .is_some_and(|sock| names_the_same_socket(sock, &leaf))
    {
        return Err(anyhow!(
            "snowflake_ws_dir and docker_socket_path are the same path ({:?}); one is a \
             socket, the other a directory that scepter writes into",
            leaf.display()
        ));
    }
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|error| {
            anyhow!(
                "snowflake_ws_dir {:?} could not be created: {error} — the identity bind needs a \
                 host directory, and a bind whose source is missing is refused by the daemon",
                dir.display()
            )
        })?;
    }
    if !dir.is_dir() {
        return Err(anyhow!(
            "snowflake_ws_dir {:?} is not a directory",
            dir.display()
        ));
    }
    // A symlinked **ancestor** has the same failure mode as a symlinked leaf: the
    // spelling passed every check above while `mount` and `chmod` act on the target.
    let resolved = std::fs::canonicalize(dir).unwrap_or_else(|_| leaf.clone());
    if let Some(problem) = socket_bearing_error(&resolved) {
        return Err(anyhow!(problem));
    }
    if cfg
        .docker_socket_path
        .as_ref()
        .is_some_and(|sock| names_the_same_socket(sock, &resolved))
    {
        return Err(anyhow!(
            "snowflake_ws_dir {:?} resolves to docker_socket_path; one is a socket, the \
             other a directory scepter writes into",
            resolved.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Unconditionally, unlike `ensure_socket_dirs`'s plain `0o777` — this one
        // adds the sticky bit and normalises an existing directory too:
        // an operator who pre-created this as root would otherwise leave the container
        // unable to write, and scepter only *warns* when its `create_dir_all` fails —
        // the failure would resurface as the original 400 at the first write-mode skill.
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o1777)).with_context(
            || {
                format!(
                    "cannot make snowflake_ws_dir {:?} container-writable (0o1777)",
                    dir.display()
                )
            },
        )?;
    }
    Ok(Some(dir.clone()))
}

/// The reason `candidate` is unusable because it contains the daemon's socket, if it
/// does. Both spellings are tested and both are resolved through their **parent**, so
/// the answer does not depend on which of `/var/run` and `/run` this host has, nor on
/// the socket itself existing.
fn socket_bearing_error(candidate: &Path) -> Option<String> {
    for spelling in RAW_DOCKER_SOCKET_SPELLINGS {
        let literal = Path::new(spelling);
        let resolved = literal
            .parent()
            .and_then(|parent| std::fs::canonicalize(parent).ok())
            .and_then(|parent| literal.file_name().map(|leaf| parent.join(leaf)))
            .unwrap_or_else(|| literal.to_path_buf());
        if literal.starts_with(candidate) || resolved.starts_with(candidate) {
            return Some(format!(
                "snowflake_ws_dir {:?} contains the raw docker socket ({spelling}); mounting it \
                 read-write inside scepter would hand the container the daemon's root-equivalent \
                 socket whatever `allow_docker_socket` says",
                candidate.display()
            ));
        }
    }
    None
}

/// The reason the configured socket sources are unusable, if they are.
///
/// Sits beside the mount it constrains (see `scepter_params`) but is enforced by
/// the fallible entry point, so the params builder is never handed a genuinely
/// ambiguous configuration. It still applies a fixed precedence for the
/// two-sources case (proxy wins) because it cannot refuse — that ranking has its
/// own test, and the guard is what keeps it unreachable in practice. Four shapes are refused: two sources at once; a non-absolute path (the
/// container runtime reads a relative value as a *named volume*, so the stack
/// would come up with an empty socket); the raw host socket named through this
/// field, which belongs to `allow_docker_socket` (that option names the R-01
/// tradeoff instead of smuggling root-equivalence in under the "proxy" label); and
/// a directory, which would hand over every socket beneath it.
fn socket_source_error(cfg: &StackConfig) -> Option<String> {
    if cfg.allow_docker_socket && cfg.docker_socket_path.is_some() {
        return Some(
            "StackConfig sets both allow_docker_socket and docker_socket_path; \
             choose one — the raw host socket (R-01 opt-in) or a proxy socket path"
                .to_string(),
        );
    }
    let path = cfg.docker_socket_path.as_ref()?;
    if !path.is_absolute() {
        return Some(format!(
            "docker_socket_path must be an absolute host path, got {:?} — a relative \
             value is a named volume to the container runtime, not a socket bind",
            path.display()
        ));
    }
    // `Path` equality is component-wise: it drops `.` and redundant separators but
    // does **not** resolve `..`, and no lexical rule can see a symlinked parent —
    // on a host where `/var/run -> /run`, `/run/docker.sock` is the same socket
    // under a different spelling. Compare lexically first, then by filesystem
    // identity when the path exists (a socket that does not exist cannot be
    // canonicalized — and cannot be mounted either).
    let raw = Path::new(RAW_DOCKER_SOCKET);
    if names_the_same_socket(path, raw) {
        return Some(format!(
            "docker_socket_path resolves to the raw host socket ({RAW_DOCKER_SOCKET}); that is \
             the root-equivalent R-01 opt-in — set allow_docker_socket and accept that \
             tradeoff, or point this field at the proxy socket instead"
        ));
    }
    // Mounting a *directory* at the container's socket path hands over every socket
    // beneath it — `/var/run` would smuggle the raw socket in through the side door
    // and the branch would still log that the raw socket stays unmounted.
    if path.is_dir() {
        return Some(format!(
            "docker_socket_path is a directory ({:?}); the mount target is a socket path, \
             and mounting a directory there exposes every socket beneath it",
            path.display()
        ));
    }
    None
}

/// The host path of the raw docker socket (the R-01 opt-in's source).
const RAW_DOCKER_SOCKET: &str = "/var/run/docker.sock";

/// The same socket under both spellings. `/var/run` is a symlink to `/run` on the
/// common Debian shape, so a directory that contains the socket may be named either
/// way, and a host can legitimately have `/run` without `/var/run`. Checking both
/// keeps the containment refusal independent of which of the two exists.
const RAW_DOCKER_SOCKET_SPELLINGS: [&str; 2] = [RAW_DOCKER_SOCKET, "/run/docker.sock"];

/// True when `path` names the same file as `other`: lexically (`..`, `.`, doubled
/// separators) or, when both exist, through symlinks. The filesystem half is what
/// catches a symlinked parent — `/run/docker.sock` on a host whose `/var/run` is a
/// symlink to `/run` — which no lexical rule can see. Both halves are needed: the
/// lexical one works before the socket exists, the canonical one after.
fn names_the_same_socket(path: &Path, other: &Path) -> bool {
    if normalize_lexically(path) == normalize_lexically(other) {
        return true;
    }
    match (path.canonicalize(), other.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Lexical normalization: drop `.` and redundant separators, resolve `..` against
/// the segments already seen, never above the root. No filesystem access — the
/// socket this path names may not exist yet when the config is validated.
fn normalize_lexically(path: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                // pop() is false at the root; `/..` is `/`, so drop it there.
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Bring up the full stack: sockets → network → pull images → create/start
/// containers → wait healthy. Returns a handle that keeps the backend alive.
///
/// Idempotent: already-running containers are left alone, stopped ones are
/// started, missing ones are created. Safe to call on every boot.
///
/// Because of that idempotency, changing `docker_socket_path` does **not** reach a
/// container that already exists; see [`StackConfig::docker_socket_path`] for what
/// the operator has to do instead of restarting the unit.
pub async fn bring_up_stack(cfg: &StackConfig) -> Result<StackHandle> {
    // Refuse rather than pick when the socket sources are ambiguous or unsafe.
    // Checked here (the fallible entry point) so `scepter_params` can stay
    // infallible and the container runtime never receives a shape it would
    // silently misread (a relative bind becomes a named volume).
    if let Some(problem) = socket_source_error(cfg) {
        return Err(anyhow!(problem));
    }
    prepare_snowflake_ws_dir(cfg)?;
    if cfg.postgres_password.is_empty() {
        return Err(anyhow!(
            "postgres_password is empty — set POSTGRES_PASSWORD or the bootloader config"
        ));
    }

    let backend = create_container_ops(&cfg.container_backend).await?;

    ensure_socket_dirs().await?;
    if cfg.network_mode.as_deref() != Some("host") {
        ensure_network().await?;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_readonly_is_advertised_only_when_repo_root_is_set() {
        // NB: 不碰 HTTP_PROXY —— 断言只看 COSMOS_REPO_READONLY 这一个键，
        // 代理镜像（若设置）不影响本判据；edition 2024 里 env::remove_var 是 unsafe。

        let mut cfg = StackConfig::default();
        assert!(
            !scepter_params(&cfg)
                .env
                .contains_key("COSMOS_REPO_READONLY"),
            "no repo_root configured => no advertisement"
        );

        cfg.host_repo_root = Some(PathBuf::from("/mnt/codespace"));
        let params = scepter_params(&cfg);
        assert_eq!(
            params.env.get("COSMOS_REPO_READONLY").map(String::as_str),
            Some("/mnt/codespace"),
            "repo_root configured => the HOST path is advertised verbatim"
        );
        // F1 钉子：主容器必须同时看到宿主同路径的只读视图，否则 scepter 侧
        // is_dir 守卫在生产栈里恒 false（读通道静默 no-op）。
        let host_path = "/mnt/codespace";
        let mounts: Vec<&VolumeMount> = params
            .volumes
            .iter()
            .filter(|v| v.host_path == host_path)
            .collect();
        assert!(
            mounts
                .iter()
                .any(|v| v.container_path == "/celestia" && v.read_only),
            "repo_root must stay mounted read-only at /celestia"
        );
        assert!(
            mounts
                .iter()
                .any(|v| v.container_path == host_path && v.read_only),
            "repo_root must ALSO be visible at its own host path (F1 guard)"
        );
    }

    #[test]
    fn evernight_lane_is_injected_into_scepter_when_configured() {
        let cfg = StackConfig {
            evernight_sock: Some("/tmp/entelecheia-unix-socket/evernight-e-042.sock".to_string()),
            evernight_token: Some("spec-sourced-token".to_string()),
            ..StackConfig::default()
        };

        let scepter = scepter_params(&cfg);
        assert_eq!(
            scepter.env.get("EVERNIGHT_SOCK").map(String::as_str),
            Some("/tmp/entelecheia-unix-socket/evernight-e-042.sock"),
            "configured sock must land verbatim in the scepter container env"
        );
        assert_eq!(
            scepter.env.get("EVERNIGHT_TOKEN").map(String::as_str),
            Some("spec-sourced-token"),
            "configured token must land verbatim in the scepter container env"
        );

        // The broker lane is scepter-only: postgres / registry must not
        // sprout evernight credentials they have no use for.
        for (name, env) in [
            ("postgres", &postgres_params(&cfg).env),
            ("registry", &registry_params(&cfg).env),
        ] {
            assert!(
                !env.contains_key("EVERNIGHT_SOCK") && !env.contains_key("EVERNIGHT_TOKEN"),
                "{name} container env must stay free of the evernight lane"
            );
        }
    }

    #[test]
    fn docker_socket_is_absent_by_default() {
        // R-01: the docker socket is root-equivalent on the host. The stack
        // must NOT mount it (and must not add the docker GID) unless the
        // operator explicitly opted in.
        let cfg = StackConfig::default();
        let scepter = scepter_params(&cfg);
        assert!(
            scepter
                .volumes
                .iter()
                .all(|v| v.host_path != "/var/run/docker.sock"),
            "default stack must not mount the docker socket"
        );
        assert!(
            scepter.group_add.is_none(),
            "default stack must not receive the docker GID"
        );
    }

    #[test]
    fn docker_socket_mounts_only_when_allowed() {
        let cfg = StackConfig {
            allow_docker_socket: true,
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        assert!(
            scepter
                .volumes
                .iter()
                .any(|v| v.host_path == "/var/run/docker.sock"
                    && v.container_path == "/var/run/docker.sock"),
            "explicit opt-in must mount the docker socket"
        );
        // The other two socket dirs stay mounted either way.
        assert!(
            scepter
                .volumes
                .iter()
                .any(|v| v.container_path == socket_dir_tmpfs()),
            "tmpfs socket dir must stay mounted"
        );
    }

    #[test]
    fn evernight_lane_is_absent_by_default() {
        // Unset spec fields must mean no injection — no panic, no host-env
        // fallback, no half-derived defaults (edition 2024 forbids touching
        // process env here anyway: env::remove_var is unsafe).
        let cfg = StackConfig::default();
        let scepter = scepter_params(&cfg);
        assert!(
            !scepter.env.contains_key("EVERNIGHT_SOCK"),
            "no evernight_sock in the spec => no EVERNIGHT_SOCK in the container env"
        );
        assert!(
            !scepter.env.contains_key("EVERNIGHT_TOKEN"),
            "no evernight_token in the spec => no EVERNIGHT_TOKEN in the container env"
        );
    }

    /// R-01 regression: by default the scepter container must get **no** docker
    /// socket at all. If this ever starts failing, the stack silently widens the
    /// 2026-11-06 security posture.
    #[test]
    fn docker_socket_path_none_mounts_nothing() {
        let cfg = StackConfig::default();
        let scepter = scepter_params(&cfg);
        assert!(
            !scepter
                .volumes
                .iter()
                .any(|v| v.container_path == "/var/run/docker.sock"),
            "default config must not mount any docker socket"
        );
    }

    /// The sanctioned alternative: a proxy socket is mounted at the container's
    /// default docker path, so bollard needs no DOCKER_HOST and the raw host socket
    /// stays unmounted.
    #[test]
    fn docker_socket_path_mounts_the_proxy_at_the_default_path() {
        let cfg = StackConfig {
            docker_socket_path: Some(std::path::PathBuf::from("/srv/proxy/docker.sock")),
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        let socket_mounts: Vec<_> = scepter
            .volumes
            .iter()
            .filter(|v| v.container_path == "/var/run/docker.sock")
            .collect();
        assert_eq!(socket_mounts.len(), 1, "exactly one docker socket mount");
        assert_eq!(socket_mounts[0].host_path, "/srv/proxy/docker.sock");
    }

    /// R-01 opt-in must keep working byte-identically: `allow_docker_socket` alone
    /// still mounts the **raw** host socket at the container's default path.
    #[test]
    fn allow_docker_socket_alone_still_mounts_the_raw_socket() {
        let cfg = StackConfig {
            allow_docker_socket: true,
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        let socket_mounts: Vec<_> = scepter
            .volumes
            .iter()
            .filter(|v| v.container_path == "/var/run/docker.sock")
            .collect();
        assert_eq!(socket_mounts.len(), 1, "exactly one docker socket mount");
        assert_eq!(
            socket_mounts[0].host_path, "/var/run/docker.sock",
            "the R-01 opt-in must still mount the RAW host socket"
        );
    }

    /// Spelling the raw host socket in `docker_socket_path` must be refused: the
    /// field is documented as the *proxy* alternative, and mounting the raw socket
    /// through it would carry root-equivalence in under the safe label while the
    /// branch's log line claims the raw socket stays unmounted.
    #[tokio::test]
    async fn docker_socket_path_cannot_be_the_raw_host_socket() {
        for raw in ["/var/run/docker.sock", "/var/run/docker.sock/"] {
            let cfg = StackConfig {
                docker_socket_path: Some(std::path::PathBuf::from(raw)),
                ..StackConfig::default()
            };
            let err = match bring_up_stack(&cfg).await {
                Ok(_) => panic!("{raw} must be refused"),
                Err(error) => error,
            };
            let text = err.to_string();
            assert!(
                text.contains("allow_docker_socket"),
                "the refusal must point at the opt-in that owns the tradeoff: {text}"
            );
        }
    }

    /// A relative (or empty) path is not a bind source: the container runtime reads
    /// it as a named volume, so the stack would start with an empty socket and the
    /// failure would surface much later as a dead docker client.
    #[tokio::test]
    async fn docker_socket_path_must_be_absolute() {
        for bad in ["proxy.sock", "", "../run/proxy.sock"] {
            let cfg = StackConfig {
                docker_socket_path: Some(std::path::PathBuf::from(bad)),
                ..StackConfig::default()
            };
            let err = match bring_up_stack(&cfg).await {
                Ok(_) => panic!("{bad:?} must be refused"),
                Err(error) => error,
            };
            assert!(
                err.to_string().contains("absolute"),
                "the refusal must say the path has to be absolute: {err}"
            );
        }
    }

    /// `Path` equality drops `.` but does not resolve `..`, so a lexical alias of
    /// the raw socket must be refused on the normalized form — otherwise
    /// `/var/run/docker.sock/../docker.sock` slips through the guard and the raw
    /// socket rides in while the branch logs that it stays unmounted.
    #[tokio::test]
    async fn docker_socket_path_aliases_of_the_raw_socket_are_refused() {
        for alias in [
            "//var/run/docker.sock",
            "/var/run/./docker.sock",
            "/var/run/docker.sock/../docker.sock",
        ] {
            let cfg = StackConfig {
                docker_socket_path: Some(std::path::PathBuf::from(alias)),
                ..StackConfig::default()
            };
            let err = match bring_up_stack(&cfg).await {
                Ok(_) => panic!("{alias} must be refused"),
                Err(error) => error,
            };
            assert!(
                err.to_string().contains("resolves to the raw host socket"),
                "{alias} must be refused as the raw socket: {err}"
            );
        }
    }

    /// No lexical rule can see a symlinked **parent**: on a host whose `/var/run`
    /// is a symlink to `/run`, `/run/docker.sock` is the same socket under a
    /// different spelling, and `is_dir()` is false for a socket. The filesystem half
    /// of `names_the_same_socket` is what catches it, so it is tested here on paths
    /// a test can create. (An end-to-end case for `/run/docker.sock` itself is not
    /// written: it would pass or skip depending on whether the lane has a docker
    /// socket, which is exactly the kind of vacuous test this repo counts as a
    /// finding.)
    #[cfg(unix)]
    #[test]
    fn names_the_same_socket_resolves_a_symlinked_parent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let real_dir = dir.path().join("real");
        std::fs::create_dir(&real_dir).expect("mkdir");
        let real = real_dir.join("docker.sock");
        std::fs::write(&real, b"").expect("touch");
        let alias_dir = dir.path().join("alias");
        std::os::unix::fs::symlink(&real_dir, &alias_dir).expect("symlink");
        assert!(
            names_the_same_socket(&alias_dir.join("docker.sock"), &real),
            "a path reached through a symlinked parent must resolve to the same file"
        );
        // The lexical half still has to work before the socket exists.
        assert!(names_the_same_socket(
            Path::new("/nonexistent/x/../y.sock"),
            Path::new("/nonexistent/y.sock")
        ));
        // `..` above the root must clamp to the root, not leave the path changed:
        // without the clamp this compares unequal and only the filesystem half
        // could save it — and that half needs the socket to exist.
        assert!(names_the_same_socket(
            Path::new("/../var/run/docker.sock"),
            Path::new("/var/run/docker.sock")
        ));
        assert!(!names_the_same_socket(&real, &real_dir.join("other.sock")));
    }

    /// A directory mounted at the container's socket path hands over everything
    /// beneath it — `/var/run` would smuggle the raw socket in through the side
    /// door while the log line says the raw socket stays unmounted.
    #[tokio::test]
    async fn docker_socket_path_directory_is_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = StackConfig {
            docker_socket_path: Some(dir.path().to_path_buf()),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("a directory source must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("is a directory"),
            "the refusal must say why a directory is wrong: {err}"
        );
    }

    /// `bring_up_stack` refuses both-set, but `scepter_params` stays infallible and
    /// therefore still *ranks* the two sources. That ranking is a second layer and
    /// needs its own test: without one, turning the `else if` into an `if` mounts
    /// both sockets and nothing notices (it survived a mutation round).
    #[test]
    fn scepter_params_mounts_only_the_proxy_when_both_are_set() {
        let cfg = StackConfig {
            allow_docker_socket: true,
            docker_socket_path: Some(std::path::PathBuf::from("/srv/proxy/docker.sock")),
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        let socket_mounts: Vec<_> = scepter
            .volumes
            .iter()
            .filter(|v| v.container_path == "/var/run/docker.sock")
            .collect();
        assert_eq!(
            socket_mounts.len(),
            1,
            "the params layer must never mount two docker sockets, even unreachable-via-guard"
        );
        assert_eq!(socket_mounts[0].host_path, "/srv/proxy/docker.sock");
    }

    /// The GID that reaches the container must come from the socket that is
    /// actually mounted. A proxy socket owned by a non-root group is the normal
    /// shape; reading the raw path here would inject the wrong group and look fine.
    #[test]
    fn proxy_socket_gid_is_read_from_the_mounted_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let proxy = dir.path().join("docker.sock");
        std::fs::write(&proxy, b"").expect("touch the fake socket");
        let cfg = StackConfig {
            docker_socket_path: Some(proxy.clone()),
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let gid = std::fs::metadata(&proxy).expect("stat").gid();
            if gid != 0 {
                assert_eq!(
                    scepter.group_add,
                    Some(vec![gid.to_string()]),
                    "the proxy socket's own GID must be injected, not the raw socket's"
                );
            } else {
                // root-owned file: the code documents that gid 0 needs no injection
                assert!(scepter.group_add.is_none());
            }
        }
        #[cfg(not(unix))]
        assert!(scepter.group_add.is_none());
    }

    /// Default stays byte-identical to before: no mount, no env var. Back-compat
    /// for any caller that does not know the field, and the shape that is correct
    /// only when scepter runs on the host.
    #[test]
    fn snowflake_ws_dir_is_absent_by_default() {
        let scepter = scepter_params(&StackConfig::default());
        assert!(
            !scepter.env.contains_key("COSMOS_WORKSPACE_HOST_DIR"),
            "no snowflake_ws_dir in the spec => no COSMOS_WORKSPACE_HOST_DIR in the env"
        );
        assert!(
            !scepter
                .volumes
                .iter()
                .any(|v| v.host_path.contains("snowflake-ws")),
            "default config must not mount a snowflake workspace root"
        );
    }

    /// The fix, in one test: the mount is an **identity** bind (host path ==
    /// container path) and the env var carries that same string, because scepter
    /// passes it to the Docker API as a bind source that the daemon resolves on
    /// the host. Any difference between the two sides reintroduces the bug.
    #[test]
    fn snowflake_ws_dir_mounts_identically_and_sets_the_env() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ws = dir.path().join("snowflake-ws");
        let ws_str = ws.to_string_lossy().to_string();
        let cfg = StackConfig {
            snowflake_ws_dir: Some(ws),
            ..StackConfig::default()
        };
        let scepter = scepter_params(&cfg);
        let mounts: Vec<_> = scepter
            .volumes
            .iter()
            .filter(|v| v.container_path == ws_str)
            .collect();
        assert_eq!(mounts.len(), 1, "exactly one identity mount");
        assert_eq!(
            mounts[0].host_path, ws_str,
            "host and container path must be the same string"
        );
        assert!(
            !mounts[0].read_only,
            "the bind must be read-write: scepter creates <root>/<uuid> through it and reclaim deletes it"
        );
        assert_eq!(
            scepter.env.get("COSMOS_WORKSPACE_HOST_DIR"),
            Some(&ws_str),
            "the env var must carry exactly the mounted path"
        );
    }

    /// The directory is created when missing, world-writable on purpose: scepter
    /// creates `<root>/<uuid>` through this bind as the image's uid, which the host
    /// cannot know.
    #[cfg(unix)]
    #[test]
    fn snowflake_ws_dir_is_created_writable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let ws = dir.path().join("new-snowflake-ws");
        let cfg = StackConfig {
            snowflake_ws_dir: Some(ws.clone()),
            ..StackConfig::default()
        };
        let prepared = prepare_snowflake_ws_dir(&cfg)
            .expect("prepare must succeed")
            .expect("Some in, Some out");
        assert_eq!(prepared, ws);
        assert!(
            ws.is_dir(),
            "the bind needs a directory that already exists"
        );
        let mode = std::fs::metadata(&ws).expect("stat").permissions().mode() & 0o7777;
        assert_eq!(
            mode, 0o1777,
            "world-writable so the image uid can create subdirectories, sticky so no peer uid can rename or substitute another user's entry"
        );
    }

    /// Relative and escaping paths are refused: the string is used verbatim on
    /// both sides, so an escape makes the identity ambiguous.
    /// Each escape is refused by its own branch, not incidentally by the
    /// create-directory failure — that message also names the field, so a deleted
    /// guard would still look covered. The `..` cases stay rooted in a tempdir so a
    /// surviving mutant cannot litter the crate directory.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_relative_and_escaping_paths() {
        let base = tempfile::tempdir().expect("tempdir");
        let dotdot = base.path().join("a/../b");
        let cases = [
            (
                std::path::PathBuf::from("snowflake-ws"),
                "must be an absolute host path",
            ),
            (dotdot, "must not contain `..`"),
        ];
        for (bad, expected) in cases {
            let cfg = StackConfig {
                snowflake_ws_dir: Some(bad.clone()),
                ..StackConfig::default()
            };
            let err = match bring_up_stack(&cfg).await {
                Ok(_) => panic!("{bad:?} must be refused"),
                Err(error) => error,
            };
            assert!(
                err.to_string().contains(expected),
                "{bad:?} must be refused with {expected:?}, got: {err}"
            );
        }
    }

    /// The filesystem root would be mounted read-write at the same path inside
    /// scepter, masking the container's whole rootfs.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_the_filesystem_root() {
        let cfg = StackConfig {
            snowflake_ws_dir: Some(std::path::PathBuf::from("/")),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("`/` must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("filesystem root"),
            "refusal must say why: {err}"
        );
    }

    /// One field is a socket, the other a directory scepter writes into.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_the_same_path_as_the_docker_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shared = dir.path().join("shared");
        let cfg = StackConfig {
            docker_socket_path: Some(shared.clone()),
            snowflake_ws_dir: Some(shared),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("the same path for both must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("same path"),
            "refusal must say why: {err}"
        );
    }

    /// A pre-created root (the operator spelling) is normalised too: leaving it
    /// 0755 root would reinstate the original 400 at the first write-mode skill.
    #[cfg(unix)]
    #[test]
    fn snowflake_ws_dir_normalises_an_existing_directory() {
        use std::os::unix::fs::PermissionsExt;
        let base = tempfile::tempdir().expect("tempdir");
        let ws = base.path().join("pre-created");
        std::fs::create_dir(&ws).expect("pre-create");
        std::fs::set_permissions(&ws, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        let cfg = StackConfig {
            snowflake_ws_dir: Some(ws.clone()),
            ..StackConfig::default()
        };
        prepare_snowflake_ws_dir(&cfg)
            .expect("prepare must succeed")
            .expect("Some in, Some out");
        let mode = std::fs::metadata(&ws).expect("stat").permissions().mode() & 0o7777;
        assert_eq!(
            mode, 0o1777,
            "an existing directory must be normalised, or a root-owned one silently breaks the bind"
        );
    }

    /// Non-UTF-8 cannot survive the trip to the Docker API, which takes a string.
    #[cfg(unix)]
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_non_utf8() {
        use std::os::unix::ffi::OsStrExt;
        let bad = std::path::PathBuf::from(std::ffi::OsStr::from_bytes(b"/tmp/ws-\xff"));
        let cfg = StackConfig {
            snowflake_ws_dir: Some(bad),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("non-UTF-8 must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("not valid UTF-8"),
            "refusal must say why: {err}"
        );
    }

    /// The raw docker socket is a socket, not a workspace root. Mounting it
    /// read-write as a directory would be both wrong and dangerous.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_the_raw_docker_socket() {
        let cfg = StackConfig {
            snowflake_ws_dir: Some(std::path::PathBuf::from("/var/run/docker.sock")),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("the raw socket must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("raw docker socket"),
            "refusal must name the reason: {err}"
        );
    }

    /// `chmod` and the daemon both follow a symlink leaf, so every spelling check
    /// above would have validated a path the mount never uses. Refused outright.
    #[cfg(unix)]
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_a_symlink() {
        let base = tempfile::tempdir().expect("tempdir");
        let target = base.path().join("real");
        std::fs::create_dir(&target).expect("mkdir");
        let link = base.path().join("link");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let cfg = StackConfig {
            snowflake_ws_dir: Some(link),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("a symlink must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("is a symlink"),
            "refusal must say why: {err}"
        );
    }

    /// Mounting a directory that *contains* the raw docker socket read-write at the
    /// same path would hand scepter a root-equivalent socket at bollard's default
    /// location with `allow_docker_socket` still false — R-01 default-off bypassed.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_a_directory_containing_the_docker_socket() {
        let cfg = StackConfig {
            snowflake_ws_dir: Some(std::path::PathBuf::from("/var")),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("an ancestor of the raw socket must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("contains the raw docker socket"),
            "refusal must say why: {err}"
        );
    }

    /// The canonical spelling of the socket's own directory. `/var/run` is a symlink
    /// to `/run` on the common Debian shape, so a guard that compares a canonicalised
    /// directory against the *literal* socket path misses it — and `/run` mounted
    /// read-write at `/run` still puts the socket where bollard looks for it.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_the_socket_directory_canonical_spelling() {
        let cfg = StackConfig {
            snowflake_ws_dir: Some(std::path::PathBuf::from("/run")),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("/run must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("contains the raw docker socket"),
            "refusal must say why: {err}"
        );
    }

    /// A trailing separator is the same leaf, and `symlink_metadata` follows the final
    /// component when the path ends in `/`. Socket-free on purpose: with a socket-bearing
    /// path the containment guard would refuse it anyway, so such a case passes with the
    /// leaf normalisation deleted (it did, in round 4).
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_a_symlink_with_a_trailing_slash() {
        let base = tempfile::tempdir().expect("tempdir");
        let target = base.path().join("real");
        std::fs::create_dir(&target).expect("mkdir");
        let link = base.path().join("link");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let with_slash = std::path::PathBuf::from(format!("{}/", link.display()));
        let cfg = StackConfig {
            snowflake_ws_dir: Some(with_slash),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("a symlink with a trailing slash must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("is a symlink"),
            "the leaf guard must fire on the normalised spelling, got: {err}"
        );
    }

    /// A path that exists but is a file is refused rather than mounted: the bind
    /// target is a directory in the container.
    #[tokio::test]
    async fn snowflake_ws_dir_refuses_a_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("ws-file");
        std::fs::write(&file, b"x").expect("write");
        let cfg = StackConfig {
            snowflake_ws_dir: Some(file),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("a file must be refused"),
            Err(error) => error,
        };
        assert!(
            err.to_string().contains("not a directory"),
            "refusal must say why: {err}"
        );
    }

    /// The acceptance case. Every other test in this module asserts a **refusal**,
    /// and a guard that refuses every proxy path passes all of them — the feature
    /// would ship dead and no gate would notice. This is the test that says a
    /// legitimate configuration gets through.
    #[test]
    fn socket_source_error_accepts_every_legitimate_source() {
        for good in ["/srv/proxy/docker.sock", "/mnt/work/r1-sock/docker.sock"] {
            let cfg = StackConfig {
                docker_socket_path: Some(std::path::PathBuf::from(good)),
                ..StackConfig::default()
            };
            assert_eq!(socket_source_error(&cfg), None, "{good} must be accepted");
        }
        // Neither source (the R-01 default) is legitimate too.
        assert_eq!(socket_source_error(&StackConfig::default()), None);
        // So is the raw opt-in on its own.
        let raw = StackConfig {
            allow_docker_socket: true,
            ..StackConfig::default()
        };
        assert_eq!(socket_source_error(&raw), None);
    }

    /// Two socket sources must be refused, not silently ranked: "both" means nobody
    /// knows which daemon the stack actually talks to. The refusal sits ahead of the
    /// postgres_password check, so this test never reaches docker.
    #[tokio::test]
    async fn both_socket_sources_are_refused() {
        let cfg = StackConfig {
            allow_docker_socket: true,
            docker_socket_path: Some(std::path::PathBuf::from("/srv/proxy/docker.sock")),
            ..StackConfig::default()
        };
        let err = match bring_up_stack(&cfg).await {
            Ok(_) => panic!("both socket sources must be refused"),
            Err(error) => error,
        };
        let text = err.to_string();
        assert!(
            text.contains("docker_socket_path") && text.contains("allow_docker_socket"),
            "refusal must name both fields so the operator knows what to drop: {text}"
        );
    }
}
