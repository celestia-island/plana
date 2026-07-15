//! Configuration module
//!
//! Provides application configuration, database config, LLM config

use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use _core::{DEFAULT_NETWORK, ModelTier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UiMode {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

impl UiMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown ui mode: {0}")]
pub struct UnknownUiModeError(pub String);

impl FromStr for UiMode {
    type Err = UnknownUiModeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(UnknownUiModeError(s.to_string())),
        }
    }
}

impl std::fmt::Display for UiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// User configuration (persisted to ~/.config/entelecheia/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_user_id")]
    pub user_id: uuid::Uuid,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub container_backend: ContainerBackendConfig,
}

fn default_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| String::new())
}

fn default_user_id() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub mode: UiMode,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_cluster_prefix")]
    pub cluster_prefix: String,
    #[serde(default = "default_fallback_badge_delay_ms")]
    pub fallback_badge_delay_ms: u64,
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
    #[serde(default = "default_geo_probe")]
    pub geo_probe: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            mode: UiMode::default(),
            language: default_language(),
            cluster_prefix: default_cluster_prefix(),
            fallback_badge_delay_ms: default_fallback_badge_delay_ms(),
            latitude: default_latitude(),
            longitude: default_longitude(),
            geo_probe: default_geo_probe(),
        }
    }
}

fn default_theme() -> String {
    "synthwave84".to_string()
}

fn default_language() -> String {
    _res::Language::default().code().to_string()
}

fn default_cluster_prefix() -> String {
    // Multi-instance: when the bootloader/evernight sets CONTAINER_PREFIX
    // (e.g. "e-042-"), use it as the default so this instance's containers
    // don't collide with other celestia instances on the same machine.
    std::env::var("CONTAINER_PREFIX")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "e-".to_string())
}

fn default_fallback_badge_delay_ms() -> u64 {
    5_000
}

impl UiConfig {
    pub fn is_default_location(&self) -> bool {
        self.latitude == 0.0 && self.longitude == 0.0
    }
}

fn default_latitude() -> f64 {
    0.0
}

fn default_longitude() -> f64 {
    0.0
}

fn default_geo_probe() -> bool {
    true
}

impl UserConfig {
    /// Get the config directory path
    ///
    /// # Returns
    /// PathBuf of the config directory
    pub fn config_dir() -> PathBuf {
        // [1] ENV override — only when the allow-env-overrides feature is
        // enabled (e.g. mock/dev builds). In production this is OFF so
        // ENTELECHEIA_CONFIG_DIR is never read from the environment,
        // preventing remote attackers from redirecting config paths.
        #[cfg(feature = "allow-env-overrides")]
        if let Ok(dir) = std::env::var("ENTELECHEIA_CONFIG_DIR") {
            return PathBuf::from(dir);
        }
        // [2] XDG / HOME — standard user config directory. Always absolute;
        // if HOME is somehow relative or unset, fall back to a safe temp path
        // rather than polluting CWD (which may be a bind-mounted workspace).
        if let Some(config_dir) = dirs::config_dir()
            && config_dir.is_absolute()
        {
            return config_dir.join("entelecheia");
        }
        PathBuf::from("/tmp/entelecheia-config")
    }

    pub fn discover_config_root() -> PathBuf {
        // Same resolution as config_dir() — no CWD walk-up, no project-layer
        // probing. The config location must be explicitly communicated via
        // environment (XDG_CONFIG_HOME, HOME, or ENTELECHEIA_CONFIG_DIR),
        // never guessed from the working directory. Inside containers /home
        // is a workspace bind-mount, not a place for config files.
        Self::config_dir()
    }

    /// Get the config file path
    ///
    /// # Returns
    /// PathBuf of the config file
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Get custom agent storage root directory path
    ///
    /// Layout: custom_agents/
    ///   ├── subscriptions.toml
    ///   ├── subscriptions.state.toml
    ///   └── git/
    ///       ├── agent-a/
    ///       │   ├── agent.toml
    ///       │   ├── skills/
    ///       │   └── soul/
    ///       └── agent-b/
    pub fn custom_agents_dir() -> PathBuf {
        Self::config_dir().join("custom_agents")
    }

    /// Load config from file
    ///
    /// # Returns
    /// Loaded config, or defaults if file does not exist
    pub fn load() -> Self {
        let config_file = Self::config_file();

        if config_file.exists() {
            let content = match std::fs::read_to_string(&config_file) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        "Config file {:?} exists but unreadable: {}. Attempting repair.",
                        config_file,
                        e
                    );
                    Self::attempt_config_repair(&config_file);
                    return Self::default();
                }
            };
            if let Ok(mut config) = toml::from_str::<Self>(&content) {
                if config.user_id == uuid::Uuid::nil() {
                    config.user_id = uuid::Uuid::now_v7();
                    if let Err(e) = config.save() {
                        tracing::warn!("Failed to save config: {}", e);
                    }
                }
                return config;
            }
            tracing::warn!(
                "Failed to parse config file {:?}, using defaults without overwriting",
                config_file
            );
            return Self::default();
        }

        let config = Self::default();
        if let Err(e) = config.save() {
            tracing::warn!("Failed to save config: {}", e);
        }
        config
    }

    #[cfg(unix)]
    fn attempt_config_repair(config_file: &Path) {
        use std::fs;
        let mode = fs::metadata(config_file)
            .map(|m| m.permissions().mode())
            .unwrap_or(0);
        if (mode & 0o600) != 0o600
            && fs::set_permissions(config_file, fs::Permissions::from_mode(0o644)).is_ok()
        {
            tracing::info!("Repaired permissions on {:?}", config_file);
        }
        if let Some(dir) = config_file.parent() {
            let dir_mode = fs::metadata(dir)
                .map(|m| m.permissions().mode())
                .unwrap_or(0);
            if (dir_mode & 0o700) != 0o700
                && let Err(e) = fs::set_permissions(dir, fs::Permissions::from_mode(0o755))
            {
                tracing::warn!(path = %dir.display(), error = %e, "failed to set permissions on config directory");
            }
        }
    }

    #[cfg(not(unix))]
    fn attempt_config_repair(_config_file: &Path) {}

    /// Save config to file
    ///
    /// # Returns
    /// Ok(()) on success, IO error on failure
    pub fn save(&self) -> std::io::Result<()> {
        let config_dir = Self::config_dir();

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
            Self::set_user_writable(&config_dir);
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;

        let config_file = Self::config_file();
        std::fs::write(&config_file, &content)?;
        Self::set_user_writable(&config_file);
        Ok(())
    }

    #[cfg(unix)]
    fn set_user_writable(path: &Path) {
        use std::fs;
        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.permissions().mode();
            let new_mode = mode | 0o600;
            if mode != new_mode
                && let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(new_mode))
            {
                tracing::warn!(path = %path.display(), error = %e, "failed to set permissions on config file");
            }
        }
    }

    #[cfg(not(unix))]
    fn set_user_writable(_path: &Path) {}

    /// Set theme
    ///
    /// # Arguments
    /// - `theme`: theme name
    pub fn set_theme(&mut self, theme: &str) {
        self.ui.theme = theme.to_string();
    }

    /// Set mode
    ///
    /// # Arguments
    /// - `mode`: mode
    pub fn set_mode(&mut self, mode: &str) {
        if let Ok(m) = UiMode::from_str(mode) {
            self.ui.mode = m;
        }
    }

    /// Set language
    ///
    /// # Arguments
    /// - `language`: language code
    pub fn set_language(&mut self, language: &str) {
        self.ui.language = language.to_string();
    }

    /// Set cluster prefix
    ///
    /// # Arguments
    /// - `prefix`: cluster prefix
    pub fn set_cluster_prefix(&mut self, prefix: &str) {
        self.ui.cluster_prefix = prefix.to_string();
    }

    pub fn set_location(&mut self, lat: f64, lon: f64) {
        self.ui.latitude = lat;
        self.ui.longitude = lon;
    }

    /// Get cluster prefix
    ///
    /// # Returns
    /// Cluster prefix string
    pub fn get_cluster_prefix(&self) -> &str {
        &self.ui.cluster_prefix
    }

    /// Get fallback badge display delay (ms)
    pub fn get_fallback_badge_delay_ms(&self) -> u64 {
        self.ui.fallback_badge_delay_ms
    }

    /// Set database URL
    ///
    /// # Arguments
    /// - `url`: database URL
    pub fn set_database_url(&mut self, url: &str) {
        self.database.url = url.to_string();
    }
}

/// Container backend connection config (persisted, cross-platform)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContainerBackendConfig {
    #[serde(default)]
    pub scheme: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_runtime")]
    pub runtime: String,
}

fn default_runtime() -> String {
    "youki".to_string()
}

impl Default for ContainerBackendConfig {
    fn default() -> Self {
        Self {
            scheme: String::new(),
            target: String::new(),
            label: String::new(),
            runtime: default_runtime(),
        }
    }
}

impl ContainerBackendConfig {
    pub fn is_configured(&self) -> bool {
        !self.scheme.is_empty() && !self.target.is_empty()
    }

    pub fn set(&mut self, scheme: &str, target: &str, label: &str) {
        self.scheme = scheme.to_string();
        self.target = target.to_string();
        self.label = label.to_string();
    }

    pub fn clear(&mut self) {
        self.scheme.clear();
        self.target.clear();
        self.label.clear();
        self.runtime = default_runtime();
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            username: default_username(),
            user_id: default_user_id(),
            ui: UiConfig::default(),
            database: DatabaseConfig::from_env(),
            container_backend: ContainerBackendConfig::default(),
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// LLM configuration
    pub llm: LlmConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
    /// Container configuration
    pub container: ContainerConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Database URL.
    ///
    /// **SECURITY WARNING**: The hardcoded default is `postgresql://entelecheia:password@localhost:5432/entelecheia`.
    /// This default password (`password`) is publicly known and MUST be changed before any production or
    /// internet-facing deployment. Always set the `DATABASE_URL` environment variable to a strong credential.
    #[serde(default = "default_database_url")]
    pub url: String,
    /// Maximum connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://entelecheia:password@localhost:5432/entelecheia".to_string()
    })
}

fn default_max_connections() -> u32 {
    std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

impl DatabaseConfig {
    /// Load configuration from environment variables
    ///
    /// # Returns
    /// Configuration loaded from environment variables
    pub fn from_env() -> Self {
        Self {
            url: default_database_url(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    #[serde(default = "default_tier")]
    pub tier: ModelTier,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub quota_limit: Option<u64>,
    #[serde(default)]
    pub quota_used: u64,
}

impl std::fmt::Debug for LlmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmConfig")
            .field("provider", &self.provider)
            .field("api_key", &"<REDACTED>")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("tier", &self.tier)
            .field("priority", &self.priority)
            .finish()
    }
}

fn default_tier() -> ModelTier {
    ModelTier::Normal
}

fn default_priority() -> u8 {
    5
}

impl LlmConfig {
    /// Load configuration from environment variables
    ///
    /// # Returns
    /// Configuration loaded from environment variables
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            api_key: {
                let key = std::env::var("LLM_API_KEY").unwrap_or_default();
                if key.is_empty() {
                    tracing::debug!(
                        "LLM_API_KEY not set — LLM calls will fail without a valid API key"
                    );
                }
                key
            },
            base_url: std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            model: std::env::var("LLM_MODEL")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(|v| v.trim().to_string())
                .unwrap_or_default(),
            tier: ModelTier::Normal,
            priority: 5,
            quota_limit: None,
            quota_used: 0,
        }
    }
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Bind address
    pub bind_address: String,
    /// Maximum connections
    pub max_connections: usize,
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// Network name
    pub network: String,
    /// Registry address
    pub registry: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    ///
    /// # Returns
    /// Configuration loaded from environment variables
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig::from_env(),
            llm: LlmConfig::from_env(),
            websocket: WebSocketConfig {
                bind_address: std::env::var("WS_BIND_ADDRESS")
                    .unwrap_or_else(|_| "127.0.0.1:8424".to_string()),
                max_connections: std::env::var("WS_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
            },
            container: ContainerConfig {
                network: std::env::var("CONTAINER_NETWORK")
                    .unwrap_or_else(|_| DEFAULT_NETWORK.to_string()),
                registry: std::env::var("CONTAINER_REGISTRY")
                    .unwrap_or_else(|_| "127.0.0.1:5000".to_string()),
            },
        }
    }

    /// Load .env file and configure from environment variables
    ///
    /// # Arguments
    /// - `path`: .env file path
    ///
    /// # Returns
    /// Loaded configuration
    pub fn load_with_env<P: AsRef<Path>>(path: P) -> Self {
        let _ = dotenvy::from_path(path);
        Self::from_env()
    }
}
