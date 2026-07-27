use anyhow::{Context, Result, anyhow};
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddrV4},
};

use tracing::{debug, error, info};

use _config::UserConfig;
use _config::ensure_provider_config_from_env;
use _container::{ServerStatus as DomainServerStatus, ops::ContainerOps};
use _infra_utils::async_bridge;

const SERVER_CONTAINER_SUFFIX: &str = "scepter";
const SERVER_PORT: u16 = 8424;

static CLUSTER_PREFIX: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn set_cluster_prefix(prefix: &str) {
    let mut guard = CLUSTER_PREFIX.lock().unwrap_or_else(|e| e.into_inner());
    *guard = if prefix.is_empty() {
        None
    } else {
        Some(prefix.to_string())
    };
}

fn server_container_name() -> String {
    let guard = CLUSTER_PREFIX.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_deref() {
        Some(prefix) => format!("{}{}", prefix, SERVER_CONTAINER_SUFFIX),
        None => format!("e-{}", SERVER_CONTAINER_SUFFIX),
    }
}

pub use _container::types::ServerStatus;

pub fn inject_docker_client(_docker: bollard::Docker) {
    debug!("[ServerManager] inject_docker_client called (no-op, using factory)");
}

fn container_ops() -> Option<&'static dyn ContainerOps> {
    static CM: std::sync::LazyLock<Option<Box<dyn ContainerOps>>> =
        std::sync::LazyLock::new(|| {
            info!("[ServerManager] Initializing container backend");
            let data_dir = crate::container_factory::default_container_data_dir();
            let runtime = crate::container_factory::outer_runtime_type();

            let backend_result = async_bridge::block_on(
                crate::container_factory::create_container_backend(runtime, &data_dir),
            )
            .ok()
            .and_then(|r| r.ok());

            match backend_result {
                Some(mgr) => {
                    info!(
                        "[ServerManager] Container backend initialized ({})",
                        runtime
                    );
                    Some(mgr)
                }
                None => {
                    error!("[ServerManager] Failed to create container backend");
                    None
                }
            }
        });
    CM.as_ref().map(|v| v.as_ref())
}

fn map_domain_status(s: DomainServerStatus) -> ServerStatus {
    match s {
        DomainServerStatus::Running => ServerStatus::Running,
        DomainServerStatus::Stopped => ServerStatus::Stopped,
        DomainServerStatus::NotExists => ServerStatus::NotExists,
        DomainServerStatus::NotBuilt => ServerStatus::NotBuilt,
        DomainServerStatus::Unknown => ServerStatus::Unknown,
    }
}

#[derive(Debug, Clone)]
pub struct ServerManager {
    status: ServerStatus,
    last_check: std::time::Instant,
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            status: ServerStatus::Unknown,
            last_check: std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(10))
                .unwrap_or_else(std::time::Instant::now),
        }
    }

    pub fn status(&self) -> ServerStatus {
        self.status
    }

    pub fn is_running(&self) -> bool {
        self.status == ServerStatus::Running
    }

    pub async fn check_and_ensure_running(&mut self) -> Result<bool> {
        let now = std::time::Instant::now();
        let should_refresh = now.duration_since(self.last_check).as_secs() > 5;

        if should_refresh {
            self.status = self.detect_status().await;
            self.last_check = now;
        }

        if self.status == ServerStatus::Running {
            return Ok(true);
        }

        self.start_server().await
    }

    async fn detect_status(&self) -> ServerStatus {
        let name = server_container_name();
        info!(
            "[ServerManager] detect_status: checking container '{}'",
            name
        );

        let mgr = match container_ops() {
            Some(m) => m,
            None => {
                error!("[ServerManager] Container backend not available");
                return ServerStatus::Unknown;
            }
        };

        let result = mgr.detect_server_status(&name).await;
        info!("[ServerManager] detect_status: result = {:?}", result);
        map_domain_status(result)
    }

    pub async fn start_server(&mut self) -> Result<bool> {
        Self::prepare_initial_aporia_config()?;

        let mgr = match container_ops() {
            Some(m) => m,
            None => return Err(anyhow!("Container backend not available")),
        };

        let status = mgr.detect_server_status(&server_container_name()).await;
        let mapped = map_domain_status(status);

        match mapped {
            ServerStatus::Running => {
                self.status = ServerStatus::Running;
                Ok(true)
            }
            ServerStatus::Stopped => {
                mgr.start(&server_container_name())
                    .await
                    .map_err(|e| anyhow!("Failed to start server: {}", e))?;
                Self::wait_for_server_ready().await;
                self.status = ServerStatus::Running;
                Ok(true)
            }
            ServerStatus::NotExists | ServerStatus::NotBuilt => Err(anyhow!(
                "Server container does not exist. Run initial setup first."
            )),
            ServerStatus::Unknown => Err(anyhow!("Cannot determine server status")),
        }
    }

    pub fn stop_server(&mut self) -> Result<bool> {
        let mgr = match container_ops() {
            Some(m) => m,
            None => return Err(anyhow!("Container backend not available")),
        };

        async_bridge::block_on(mgr.stop(&server_container_name()))
            .map_err(|e| anyhow!("Failed to stop server: {}", e))?
            .map_err(|e| anyhow!("Failed to stop server: {}", e))?;
        self.status = ServerStatus::Stopped;
        Ok(true)
    }

    pub async fn restart_server(&mut self) -> Result<bool> {
        let mgr = match container_ops() {
            Some(m) => m,
            None => return Err(anyhow!("Container backend not available")),
        };

        mgr.restart(&server_container_name())
            .await
            .map_err(|e| anyhow!("Failed to restart server: {}", e))?;
        Self::wait_for_server_ready().await;
        self.status = ServerStatus::Running;
        Ok(true)
    }

    async fn wait_for_server_ready() {
        let max_attempts = 30;
        for _ in 0..max_attempts {
            if Self::is_port_open().await {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    async fn is_port_open() -> bool {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT);
        tokio::time::timeout(
            std::time::Duration::from_millis(500),
            tokio::net::TcpStream::connect(addr),
        )
        .await
        .map(|res| res.is_ok())
        .unwrap_or(false)
    }

    pub fn get_logs(&self, lines: usize) -> Result<String> {
        let mgr = match container_ops() {
            Some(m) => m,
            None => return Err(anyhow!("Container backend not available")),
        };

        async_bridge::block_on(mgr.get_container_logs(&server_container_name(), lines))
            .context("block_on failed")?
            .map_err(|e| anyhow!("Failed to get logs: {}", e))
    }

    pub fn refresh_status(&mut self) {
        self.status = async_bridge::block_on(self.detect_status()).unwrap_or(ServerStatus::Unknown);
        self.last_check = std::time::Instant::now();
    }

    fn prepare_initial_aporia_config() -> Result<()> {
        let config_dir = UserConfig::config_dir();
        let config_path = config_dir.join("aporia.toml");

        if config_path.exists() {
            return Ok(());
        }

        let providers = Self::detect_env_providers();
        if providers.is_empty() {
            return Ok(());
        }

        std::fs::create_dir_all(&config_dir).map_err(|e| {
            anyhow!(
                "Failed to create config dir {}: {}",
                config_dir.display(),
                e
            )
        })?;

        let mut toml_content = String::from("[llm_providers]\n");
        for (name, provider_type, api_key, model) in &providers {
            toml_content.push_str(&format!(
                "\n[llm_providers.{}]\n\
                 name = \"{}\"\n\
                 provider_type = \"{}\"\n\
                 api_key = \"{}\"\n\
                 model = \"{}\"\n",
                name.replace('-', "_"),
                name,
                provider_type,
                api_key,
                model
            ));
        }

        let mut file = std::fs::File::create(&config_path)
            .map_err(|e| anyhow!("Failed to create {}: {}", config_path.display(), e))?;
        file.write_all(toml_content.as_bytes())
            .map_err(|e| anyhow!("Failed to write config: {}", e))?;

        info!(
            "[ServerManager] Initialized aporia config at {}",
            config_path.display()
        );

        // Also generate provider_config.toml from env vars so LLM routing
        // can find these providers immediately without requiring the TUI to
        // be opened first. ensure_provider_config_from_env() is a no-op if
        // provider_config.toml already exists.
        if let Err(e) = ensure_provider_config_from_env() {
            info!(
                "[ServerManager] provider_config.toml generation skipped or failed: {}",
                e
            );
        } else {
            info!("[ServerManager] provider_config.toml ensured alongside aporia.toml");
        }

        Ok(())
    }

    fn detect_env_providers() -> Vec<(String, String, String, String)> {
        struct EnvMapping {
            env_var: &'static str,
            name: &'static str,
            provider_type: &'static str,
            model: &'static str,
        }

        const ENV_MAPPINGS: &[EnvMapping] = &[
            EnvMapping {
                env_var: "OPENAI_API_KEY",
                name: "openai",
                provider_type: "openai",
                model: "gpt-4o",
            },
            EnvMapping {
                env_var: "ANTHROPIC_API_KEY",
                name: "anthropic",
                provider_type: "anthropic",
                model: "claude-sonnet-4-20250514",
            },
            EnvMapping {
                env_var: "DEEPSEEK_API_KEY",
                name: "deepseek",
                provider_type: "openai",
                model: "deepseek-chat",
            },
            EnvMapping {
                env_var: "QWEN_API_KEY",
                name: "qwen",
                provider_type: "openai",
                model: "qwen-plus",
            },
            EnvMapping {
                env_var: "GLM_API_KEY",
                name: "zhipu_glm",
                provider_type: "openai",
                model: "glm-4-flash",
            },
            EnvMapping {
                env_var: "CODING_PRO",
                name: "coding",
                provider_type: "openai",
                model: "coding-pro",
            },
            EnvMapping {
                env_var: "CODING_MAX",
                name: "coding",
                provider_type: "openai",
                model: "coding-max",
            },
        ];

        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for mapping in ENV_MAPPINGS {
            if let Ok(key) = std::env::var(mapping.env_var)
                && !key.trim().is_empty()
                && seen.insert(mapping.name)
            {
                result.push((
                    mapping.name.to_string(),
                    mapping.provider_type.to_string(),
                    key,
                    mapping.model.to_string(),
                ));
            }
        }

        result
    }
}
