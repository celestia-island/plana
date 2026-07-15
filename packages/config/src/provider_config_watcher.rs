//! Provider configuration hot-reload system
//!
//! Watches provider configuration file changes and enables runtime hot-reload.
//! Converges with Agent configuration system (P2-03), sharing config update event bus.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::broadcast;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, error, info, warn};

use super::{
    provider_config::{ModelEntry, ProviderConfigData, ProviderEntry, load_provider_config},
    provider_config_validator::{
        AvailableModel, AvailableProvider, ProviderValidationResult, validate_and_log_providers,
    },
};

const PROVIDER_CONFIG_FILENAME: &str = "provider_config.toml";

/// Convert ProviderEntry to AvailableProvider
fn provider_entry_to_available(entry: &ProviderEntry) -> AvailableProvider {
    AvailableProvider {
        id: entry.id.clone(),
        name: entry.name.clone(),
        uuid: entry.uuid,
        api_key: entry.api_key.clone(),
        base_url: entry.base_url.clone(),
        is_validated: entry.is_validated,
        is_custom: entry.is_custom,
        entry_point_id: entry.entry_point_id.clone(),
        period_billing_configs: entry.period_billing_configs.clone(),
        auth_type: entry.auth_type.clone(),
        auth_header: entry.auth_header.clone(),
    }
}

/// Convert ModelEntry to AvailableModel
fn model_entry_to_available(entry: &ModelEntry) -> AvailableModel {
    AvailableModel {
        id: entry.id.clone(),
        name: entry.name.clone(),
        provider_id: entry.provider_id.clone(),
        api_model: entry.api_model.clone(),
        is_enabled: entry.is_enabled,
        is_custom: entry.is_custom,
        priority: entry.priority,
        tier: entry.tier.clone(),
        context_window: entry.context_window,
        compression_threshold: entry.compression_threshold,
        has_per_usage: entry.has_per_usage,
        has_periodic: entry.has_periodic,
        price_input: entry.price_input,
        price_cache_input: entry.price_cache_input,
        price_output: entry.price_output,
        rate_multiplier: entry.rate_multiplier,
        supports_image: entry.supports_image,
        supports_audio: entry.supports_audio,
        supports_video: entry.supports_video,
        can_reason: entry.can_reason,
        max_concurrent: entry.max_concurrent,
        category: entry.category.clone(),
        generation: entry.generation.clone(),
    }
}

/// Provider configuration change event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProviderConfigEvent {
    /// Configuration updated
    Updated {
        providers_count: usize,
        models_count: usize,
    },
    /// Configuration validation failed
    ValidationFailed { errors: Vec<String> },
    /// Configuration load failed
    LoadFailed { error: String },
}

/// Provider configuration hot-reload watcher
pub struct ProviderConfigWatcher {
    /// Config file path
    config_path: PathBuf,
    /// Current config cache
    config_cache: Arc<RwLock<ProviderConfigData>>,
    /// Event sender
    event_tx: broadcast::Sender<ProviderConfigEvent>,
    /// Enabled flag
    enabled: Arc<RwLock<bool>>,
}

impl ProviderConfigWatcher {
    /// Create a new config watcher
    pub fn new() -> Result<Self> {
        let config_path =
            Self::find_config_path().ok_or_else(|| anyhow!("Config path not found"))?;

        let (event_tx, _) = broadcast::channel(100);

        // Initial config load
        let initial_config = load_provider_config();

        info!(
            "[ProviderConfigWatcher] Initialized with config path: {:?}",
            config_path
        );

        Ok(Self {
            config_path,
            config_cache: Arc::new(RwLock::new(initial_config)),
            event_tx,
            enabled: Arc::new(RwLock::new(true)),
        })
    }

    /// Find config file path — delegates to the shared chain resolver.
    fn find_config_path() -> Option<PathBuf> {
        super::provider_config::find_provider_config_path()
    }

    /// Get current configuration
    pub fn get_config(&self) -> ProviderConfigData {
        self.config_cache.read().clone()
    }

    /// Subscribe to config change events
    pub fn subscribe(&self) -> broadcast::Receiver<ProviderConfigEvent> {
        self.event_tx.subscribe()
    }

    /// Start file watching
    pub async fn spawn_watcher(&self) -> Result<()> {
        let config_cache = self.config_cache.clone();
        let event_tx = self.event_tx.clone();
        let enabled = self.enabled.clone();

        // Create file watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    debug!("[ProviderConfigWatcher] File event: {:?}", event.kind);

                    if event.kind.is_modify() || event.kind.is_create() {
                        // Check if this is our config file
                        for path in &event.paths {
                            if path.ends_with(PROVIDER_CONFIG_FILENAME) {
                                debug!("[ProviderConfigWatcher] Config file changed: {:?}", path);

                                // Reload config
                                let new_config = load_provider_config();

                                // Convert to validator types
                                let providers: Vec<AvailableProvider> = new_config
                                    .providers
                                    .iter()
                                    .map(provider_entry_to_available)
                                    .collect();
                                let models: Vec<AvailableModel> = new_config
                                    .models
                                    .iter()
                                    .map(model_entry_to_available)
                                    .collect();

                                // Validate config
                                let validation_result =
                                    validate_and_log_providers(&providers, &models);

                                if validation_result.is_valid {
                                    // Update cache
                                    *config_cache.write() = new_config.clone();

                                    // Send update event
                                    if let Err(e) = event_tx.send(ProviderConfigEvent::Updated {
                                        providers_count: new_config.providers.len(),
                                        models_count: new_config.models.len(),
                                    }) {
                                        warn!(error = %e, "provider config Updated event send failed");
                                    }

                                    info!(
                                        "[ProviderConfigWatcher] Config reloaded successfully: {} providers, {} models",
                                        new_config.providers.len(),
                                        new_config.models.len()
                                    );
                                } else {
                                    // Validation failed
                                    let errors: Vec<String> = validation_result
                                        .errors
                                        .iter()
                                        .map(|e| e.message.clone())
                                        .collect();

                                    if let Err(e) =
                                        event_tx.send(ProviderConfigEvent::ValidationFailed {
                                            errors: errors.clone(),
                                        })
                                    {
                                        warn!(error = %e, "provider config ValidationFailed event send failed");
                                    }

                                    error!(
                                        "[ProviderConfigWatcher] Config validation failed: {:?}",
                                        errors
                                    );
                                }
                                break;
                            }
                        }
                    }
                }
            },
            notify::Config::default(),
        )?;

        // Watch the config file directory — and any other directories in the
        // config chain that currently exist, so changes at any priority layer
        // are picked up.
        let chain = super::provider_config::config_chain_public();
        let mut watched: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
        if let Some(dir) = self.config_path.parent() {
            watched.insert(dir.to_path_buf());
        }
        for candidate in &chain {
            if candidate.exists()
                && let Some(dir) = candidate.parent()
                && dir.exists()
            {
                watched.insert(dir.to_path_buf());
            }
        }
        for dir in &watched {
            if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
                warn!("[ProviderConfigWatcher] Failed to watch {:?}: {}", dir, e);
            }
        }

        info!(
            "[ProviderConfigWatcher] Watching {} director(ies) for provider_config.toml changes",
            watched.len()
        );

        // Keep watcher alive
        tokio::spawn(async move {
            let _watcher = watcher;
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if !*enabled.read() {
                    debug!("[ProviderConfigWatcher] Watcher disabled, stopping");
                    break;
                }
            }
        });

        Ok(())
    }

    /// Manually reload configuration
    pub fn reload(&self) -> Result<ProviderValidationResult> {
        let new_config = load_provider_config();

        // Convert to validator types
        let providers: Vec<AvailableProvider> = new_config
            .providers
            .iter()
            .map(provider_entry_to_available)
            .collect();
        let models: Vec<AvailableModel> = new_config
            .models
            .iter()
            .map(model_entry_to_available)
            .collect();

        // Validate config
        let validation_result = validate_and_log_providers(&providers, &models);

        if validation_result.is_valid {
            // Update cache
            *self.config_cache.write() = new_config.clone();

            // Send update event
            let _ = self.event_tx.send(ProviderConfigEvent::Updated {
                providers_count: new_config.providers.len(),
                models_count: new_config.models.len(),
            });

            info!(
                "[ProviderConfigWatcher] Config reloaded manually: {} providers, {} models",
                new_config.providers.len(),
                new_config.models.len()
            );
        } else {
            let errors: Vec<String> = validation_result
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect();

            let _ = self
                .event_tx
                .send(ProviderConfigEvent::ValidationFailed { errors });

            error!("[ProviderConfigWatcher] Manual config reload validation failed");
        }

        Ok(validation_result)
    }

    /// Enable watcher
    pub fn enable(&self) {
        *self.enabled.write() = true;
        info!("[ProviderConfigWatcher] Watcher enabled");
    }

    /// Disable watcher
    pub fn disable(&self) {
        *self.enabled.write() = false;
        info!("[ProviderConfigWatcher] Watcher disabled");
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }
}

impl Default for ProviderConfigWatcher {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            warn!("[ProviderConfigWatcher] Failed to initialize, using empty config");

            let (event_tx, _) = broadcast::channel(100);

            Self {
                config_path: PathBuf::from("provider_config.toml"),
                config_cache: Arc::new(RwLock::new(ProviderConfigData::default())),
                event_tx,
                enabled: Arc::new(RwLock::new(false)),
            }
        })
    }
}

/// Global provider config watcher instance
static GLOBAL_WATCHER: std::sync::LazyLock<ProviderConfigWatcher> =
    std::sync::LazyLock::new(ProviderConfigWatcher::default);

/// Get global config watcher
pub fn get_global_watcher() -> &'static ProviderConfigWatcher {
    &GLOBAL_WATCHER
}

/// Get current config from global watcher
pub fn get_global_config() -> ProviderConfigData {
    GLOBAL_WATCHER.get_config()
}

/// Subscribe to global config change events
pub fn subscribe_global_events() -> broadcast::Receiver<ProviderConfigEvent> {
    GLOBAL_WATCHER.subscribe()
}

/// Start global config watcher
pub async fn spawn_global_watcher() -> Result<()> {
    GLOBAL_WATCHER.enable();
    GLOBAL_WATCHER.spawn_watcher().await
}

/// Configuration update audit log
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigEventType {
    Updated,
    ValidationFailed,
    LoadFailed,
}

impl std::fmt::Display for ConfigEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigEventType::Updated => write!(f, "updated"),
            ConfigEventType::ValidationFailed => write!(f, "validation_failed"),
            ConfigEventType::LoadFailed => write!(f, "load_failed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigUpdateAuditLog {
    pub timestamp: DateTime<Utc>,
    pub event_type: ConfigEventType,
    pub providers_count: usize,
    pub models_count: usize,
    pub validation_errors: Vec<String>,
}

impl ConfigUpdateAuditLog {
    pub fn from_event(event: &ProviderConfigEvent) -> Self {
        let timestamp = Utc::now();

        match event {
            ProviderConfigEvent::Updated {
                providers_count,
                models_count,
            } => Self {
                timestamp,
                event_type: ConfigEventType::Updated,
                providers_count: *providers_count,
                models_count: *models_count,
                validation_errors: Vec::new(),
            },
            ProviderConfigEvent::ValidationFailed { errors } => Self {
                timestamp,
                event_type: ConfigEventType::ValidationFailed,
                providers_count: 0,
                models_count: 0,
                validation_errors: errors.clone(),
            },
            ProviderConfigEvent::LoadFailed { error } => Self {
                timestamp,
                event_type: ConfigEventType::LoadFailed,
                providers_count: 0,
                models_count: 0,
                validation_errors: vec![error.clone()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_update_audit_log_from_event() -> Result<()> {
        let event = ProviderConfigEvent::Updated {
            providers_count: 5,
            models_count: 10,
        };

        let audit_log = ConfigUpdateAuditLog::from_event(&event);

        assert_eq!(audit_log.event_type, ConfigEventType::Updated);
        assert_eq!(audit_log.providers_count, 5);
        assert_eq!(audit_log.models_count, 10);
        assert!(audit_log.validation_errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_config_update_audit_log_validation_failed() -> Result<()> {
        let event = ProviderConfigEvent::ValidationFailed {
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        let audit_log = ConfigUpdateAuditLog::from_event(&event);

        assert_eq!(audit_log.event_type, ConfigEventType::ValidationFailed);
        assert_eq!(audit_log.validation_errors.len(), 2);
        Ok(())
    }
}
