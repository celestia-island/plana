use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use tokio::{sync::broadcast, time::interval};

use tracing::{info, warn};

use super::{agent_config::LlmProviderConfig, gen_protocol::GenProtocol};
use _core::is_invalid_api_key;

const URL_PATH_OPENAI_RESPONSES: &str = "/openai/responses";
const URL_PATH_V1_RESPONSES: &str = "/v1/responses";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCrudConfig {
    #[serde(rename = "llm_providers")]
    pub llm_providers: Vec<LlmProviderConfig>,
    #[serde(default)]
    pub preferences: ProviderCrudPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCrudPreferences {
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for ProviderCrudPreferences {
    fn default() -> Self {
        Self {
            language: default_language(),
        }
    }
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Clone, Default)]
pub struct ProviderUpdates {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub provider_type: Option<String>,
}

impl std::fmt::Debug for ProviderUpdates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderUpdates")
            .field("api_key", &self.api_key.as_ref().map(|_| "<REDACTED>"))
            .field("model", &self.model)
            .field("endpoint", &self.endpoint)
            .field("provider_type", &self.provider_type)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredProvider {
    pub provider_name: String,
    pub display_name: String,
    pub api_endpoint: Option<String>,
    pub is_enabled: bool,
    pub default_model: String,
}

impl From<LlmProviderConfig> for ConfiguredProvider {
    fn from(p: LlmProviderConfig) -> Self {
        Self {
            provider_name: p.name.clone(),
            display_name: p.name,
            api_endpoint: p.endpoint,
            is_enabled: !p.api_key.is_empty(),
            default_model: p.model,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigChangeSource {
    AgentConfig,
    ProviderConfig,
}

#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub source: ConfigChangeSource,
}

pub struct ProviderConfigManager {
    config_path: PathBuf,
    cache: Arc<RwLock<Option<ProviderCrudConfig>>>,
}

impl ProviderConfigManager {
    pub fn new() -> Self {
        let config_path = Self::find_config_path();
        Self {
            config_path,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            config_path: path.as_ref().to_path_buf(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load(&self) -> ProviderCrudConfig {
        if let Ok(guard) = self.cache.read()
            && let Some(ref cfg) = *guard
        {
            return cfg.clone();
        }
        let cfg = self.load_from_disk();
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(cfg.clone());
        }
        cfg
    }

    fn load_from_disk(&self) -> ProviderCrudConfig {
        if !self.config_path.exists() {
            info!(
                "Provider config not found at {:?}, using default",
                self.config_path
            );
            return ProviderCrudConfig::default();
        }

        match std::fs::read_to_string(&self.config_path) {
            Ok(content) => match toml::from_str::<ProviderCrudConfig>(&content) {
                Ok(config) => {
                    info!(
                        "Loaded provider config from {:?} with {} providers",
                        self.config_path,
                        config.llm_providers.len()
                    );
                    config
                },
                Err(e) => {
                    warn!("Failed to parse provider config: {}, using default", e);
                    ProviderCrudConfig::default()
                },
            },
            Err(e) => {
                warn!("Failed to read provider config: {}, using default", e);
                ProviderCrudConfig::default()
            },
        }
    }

    pub fn save(&self, config: &ProviderCrudConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        std::fs::write(&self.config_path, content)
            .map_err(|e| anyhow!("Failed to write config: {}", e))?;

        self.invalidate_cache();
        info!("Saved provider config to {:?}", self.config_path);
        Ok(())
    }

    pub fn list_providers(&self) -> Vec<LlmProviderConfig> {
        self.load().llm_providers
    }

    pub fn get_provider(&self, name: &str) -> Option<LlmProviderConfig> {
        let config = self.load();
        config.llm_providers.into_iter().find(|p| p.name == name)
    }

    pub fn get_default_provider(&self) -> Option<LlmProviderConfig> {
        let config = self.load();
        config.llm_providers.into_iter().next()
    }

    pub fn add_provider(&self, provider: LlmProviderConfig) -> Result<()> {
        let mut config = self.load();

        if config.llm_providers.iter().any(|p| p.name == provider.name) {
            return Err(anyhow!("Provider '{}' already exists", provider.name));
        }

        config.llm_providers.push(provider);
        self.save(&config)
    }

    pub fn update_provider(&self, name: &str, updates: ProviderUpdates) -> Result<()> {
        let mut config = self.load();

        let provider = config
            .llm_providers
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow!("Provider '{}' not found", name))?;

        if let Some(api_key) = updates.api_key {
            provider.api_key = api_key;
        }
        if let Some(model) = updates.model {
            provider.model = model;
        }
        if let Some(endpoint) = updates.endpoint {
            provider.endpoint = Some(endpoint);
        }
        if let Some(provider_type) = updates.provider_type {
            provider.provider_type = provider_type;
        }

        self.save(&config)
    }

    pub fn rename_provider(&self, name: &str, new_name: String) -> Result<()> {
        let mut config = self.load();

        let provider = config
            .llm_providers
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow!("Provider '{}' not found", name))?;

        provider.name = new_name;
        self.save(&config)
    }

    pub fn delete_provider(&self, name: &str) -> Result<()> {
        let mut config = self.load();

        let initial_len = config.llm_providers.len();
        config.llm_providers.retain(|p| p.name != name);

        if config.llm_providers.len() == initial_len {
            return Err(anyhow!("Provider '{}' not found", name));
        }

        self.save(&config)
    }

    pub fn set_api_key(&self, name: &str, api_key: &str) -> Result<()> {
        self.update_provider(
            name,
            ProviderUpdates {
                api_key: Some(api_key.to_string()),
                ..Default::default()
            },
        )
    }

    pub fn update_endpoint(&self, name: &str, endpoint: &str) -> Result<()> {
        self.update_provider(
            name,
            ProviderUpdates {
                endpoint: Some(endpoint.to_string()),
                ..Default::default()
            },
        )
    }

    pub fn get_language(&self) -> String {
        self.load().preferences.language.clone()
    }

    pub fn set_language(&self, language: &str) -> Result<()> {
        let mut config = self.load();
        config.preferences.language = language.to_string();
        self.save(&config)
    }

    pub fn bootstrap_from_env(&self) -> bool {
        let env_providers = collect_env_providers();
        if env_providers.is_empty() {
            return false;
        }

        let mut config = self.load_from_disk();
        if config.llm_providers.is_empty() && !self.config_path.exists() {
            config = ProviderCrudConfig::default();
        }

        let mut changed = false;
        for provider in env_providers {
            if config
                .llm_providers
                .iter()
                .any(|p| p.name == provider.name && !is_invalid_api_key(&p.api_key))
            {
                continue;
            }
            if config.llm_providers.iter().any(|p| p.name == provider.name) {
                if let Some(existing) = config
                    .llm_providers
                    .iter_mut()
                    .find(|p| p.name == provider.name)
                {
                    *existing = provider;
                }
            } else {
                config.llm_providers.push(provider);
            }
            changed = true;
        }

        if changed {
            if let Err(e) = self.save(&config) {
                warn!("bootstrap_from_env: failed to save config: {}", e);
            } else {
                info!("bootstrap_from_env: config written from env vars");
                self.invalidate_cache();
            }
        }
        changed
    }

    fn invalidate_cache(&self) {
        if let Ok(mut guard) = self.cache.write() {
            *guard = None;
        }
    }

    pub fn start_watch(
        &self,
        tx: broadcast::Sender<ConfigChangeEvent>,
        mut shutdown_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let config_path = self.config_path.clone();
        let cache = Arc::clone(&self.cache);
        tokio::spawn(async move {
            let mut poll = interval(Duration::from_secs(5));
            let mut last_mtime: Option<SystemTime> = None;
            loop {
                tokio::select! {
                    _ = poll.tick() => {}
                    _ = shutdown_rx.recv() => {
                        info!("Hot reload: provider config watcher received shutdown signal");
                        break;
                    }
                }
                let mtime = tokio::fs::metadata(&config_path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());
                if mtime.is_some() && mtime != last_mtime {
                    if last_mtime.is_some() {
                        match tokio::fs::read_to_string(&config_path).await {
                            Ok(content) => match toml::from_str::<ProviderCrudConfig>(&content) {
                                Ok(new_cfg) => {
                                    info!("Hot reload: provider config reloaded");
                                    if let Ok(mut g) = cache.write() {
                                        *g = Some(new_cfg);
                                    }
                                    let _ = tx.send(ConfigChangeEvent {
                                        source: ConfigChangeSource::AgentConfig,
                                    });
                                },
                                Err(e) => warn!("Hot reload: config parse failed: {}", e),
                            },
                            Err(e) => warn!("Hot reload: config read failed: {}", e),
                        }
                    }
                    last_mtime = mtime;
                }
            }
        })
    }

    fn find_config_path() -> PathBuf {
        let candidates = get_config_candidates();

        for config_path in &candidates {
            if config_path.exists() {
                info!("Found existing provider config at {:?}", config_path);
                return config_path.clone();
            }
        }

        let (_, config_path) = find_writable_config_dir();
        info!("Using provider config path: {:?}", config_path);
        config_path
    }
}

impl Default for ProviderConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

fn get_config_candidates() -> Vec<PathBuf> {
    vec![super::app_config::UserConfig::config_dir().join("aporia.toml")]
}

fn find_writable_config_dir() -> (PathBuf, PathBuf) {
    let dir = super::app_config::UserConfig::config_dir();
    if is_writable_dir(&dir) {
        return (dir.clone(), dir.join("aporia.toml"));
    }

    let fallback = std::env::temp_dir().join("entelecheia-config");
    let _ = std::fs::create_dir_all(&fallback);
    warn!(
        "No standard config directory writable, falling back to: {:?}",
        fallback
    );
    (fallback.clone(), fallback.join("aporia.toml"))
}

fn is_writable_dir(path: &Path) -> bool {
    if path.exists() {
        let test_file = path.join(".write_test");
        if std::fs::write(&test_file, b"").is_ok() {
            let _ = std::fs::remove_file(&test_file);
            return true;
        }
    } else {
        if std::fs::create_dir_all(path).is_ok() {
            return true;
        }
    }
    false
}

fn collect_env_providers() -> Vec<LlmProviderConfig> {
    let mut providers = Vec::new();

    let api_key = std::env::var("LLM_API_KEY")
        .ok()
        .filter(|k| !is_invalid_api_key(k));

    if let Some(api_key) = api_key {
        let provider_name = std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "openai".to_string())
            .trim()
            .to_lowercase();

        let protocol = std::env::var("LLM_PROTOCOL")
            .ok()
            .filter(|p| !p.trim().is_empty())
            .map(|p| p.trim().to_string())
            .unwrap_or_else(|| {
                let base_url = std::env::var("LLM_BASE_URL")
                    .unwrap_or_default()
                    .to_lowercase();
                if base_url.contains(URL_PATH_OPENAI_RESPONSES)
                    || base_url.contains(URL_PATH_V1_RESPONSES)
                {
                    "openai_responses_v1".to_string()
                } else {
                    GenProtocol::resolve(&provider_name).as_str().to_string()
                }
            });

        let model = std::env::var("LLM_MODEL")
            .ok()
            .filter(|m| !m.trim().is_empty())
            .map(|m| m.trim().to_string())
            .unwrap_or_default();

        let endpoint = std::env::var("LLM_BASE_URL")
            .ok()
            .filter(|u| !u.trim().is_empty())
            .map(|u| u.trim().to_string());

        providers.push(LlmProviderConfig {
            name: provider_name,
            provider_type: protocol,
            api_key,
            model,
            endpoint,
            website_domain: String::new(),
        });
    }

    let all_entrypoints = super::provider_config::load_all_entrypoints_from_toml();

    for ep in &all_entrypoints {
        if let Ok(api_key) = std::env::var(&ep.env_var)
            && !is_invalid_api_key(&api_key)
        {
            if providers.iter().any(|p| p.name == ep.entrypoint_id) {
                continue;
            }
            let default_model = ep.normal_models.first().cloned().unwrap_or_default();
            providers.push(LlmProviderConfig {
                name: ep.entrypoint_id.clone(),
                provider_type: ep.protocol.clone(),
                api_key,
                model: default_model,
                endpoint: Some(ep.base_url.clone()),
                website_domain: ep.website_domain.clone(),
            });
        }
    }

    providers
}
