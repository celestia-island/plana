//! Unified configuration event bus
//!
//! Converges Provider config system (P2-06) with Agent config system (P2-03).
//! Provides a unified config update event bus, sharing config watching infrastructure.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use tracing::{debug, info, warn};

use super::provider_config_watcher::ProviderConfigEvent;

/// Unified configuration event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEvent {
    /// Provider configuration change
    Provider(ProviderConfigEvent),
    /// Agent configuration change
    Agent {
        agent_name: String,
        config_path: String,
    },
    /// System configuration change
    System { key: String, value: String },
}

impl ConfigEvent {
    /// Get event type name
    pub fn event_type(&self) -> &str {
        match self {
            ConfigEvent::Provider(_) => "provider",
            ConfigEvent::Agent { .. } => "agent",
            ConfigEvent::System { .. } => "system",
        }
    }
}

impl From<ProviderConfigEvent> for ConfigEvent {
    fn from(event: ProviderConfigEvent) -> Self {
        ConfigEvent::Provider(event)
    }
}

/// Configuration event listener trait
pub trait ConfigEventListener {
    /// Handle configuration change event
    fn on_config_change(&self, event: &ConfigEvent);
}

/// Configuration change audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeAudit {
    pub timestamp: DateTime<Utc>,
    pub event_type: super::provider_config_watcher::ConfigEventType,
    pub source: String,
    pub details: String,
}

impl ConfigChangeAudit {
    pub fn new(event: &ConfigEvent) -> Self {
        let timestamp = Utc::now();
        use super::provider_config_watcher::ConfigEventType as CET;
        let (event_type, source, details) = match event {
            ConfigEvent::Provider(provider_event) => {
                let (event_type, details) = match provider_event {
                    ProviderConfigEvent::Updated {
                        providers_count,
                        models_count,
                    } => (
                        CET::Updated,
                        format!(
                            "Provider config updated: {} providers, {} models",
                            providers_count, models_count
                        ),
                    ),
                    ProviderConfigEvent::ValidationFailed { errors } => (
                        CET::ValidationFailed,
                        format!("Provider config validation failed: {:?}", errors),
                    ),
                    ProviderConfigEvent::LoadFailed { error } => (
                        CET::LoadFailed,
                        format!("Provider config load failed: {}", error),
                    ),
                };
                (event_type, "provider".to_string(), details)
            }
            ConfigEvent::Agent {
                agent_name,
                config_path,
            } => (
                CET::Updated,
                "agent".to_string(),
                format!("Agent config updated: {} at {}", agent_name, config_path),
            ),
            ConfigEvent::System { key, value } => (
                CET::Updated,
                "system".to_string(),
                format!("System config updated: {} = {}", key, value),
            ),
        };

        Self {
            timestamp,
            event_type,
            source,
            details,
        }
    }
}

/// Unified configuration event bus
#[derive(Clone)]
pub struct ConfigEventBus {
    /// Event sender
    event_tx: broadcast::Sender<ConfigEvent>,
    /// Event listener list
    listeners: Arc<RwLock<Vec<Box<dyn ConfigEventListener + Send + Sync>>>>,
    /// Audit log records
    audit_log: Arc<RwLock<Vec<ConfigChangeAudit>>>,
    /// Maximum audit log entries
    max_audit_entries: usize,
}

impl ConfigEventBus {
    /// Create a new configuration event bus
    pub fn new() -> Self {
        Self::with_max_audit_entries(1000)
    }

    /// Create a configuration event bus with specified max audit entries
    pub fn with_max_audit_entries(max_entries: usize) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            event_tx,
            listeners: Arc::new(RwLock::new(Vec::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            max_audit_entries: max_entries,
        }
    }

    /// Create a new configuration event bus (alias with limit, for testing)
    #[cfg(test)]
    pub fn new_with_limit(max_entries: usize) -> Self {
        Self::with_max_audit_entries(max_entries)
    }

    /// Publish a configuration change event
    pub fn publish(&self, event: ConfigEvent) {
        debug!("[ConfigEventBus] Publishing event: {}", event.event_type());

        // Record audit log
        let audit = ConfigChangeAudit::new(&event);
        self.add_audit_entry(audit);

        // Notify all listeners
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener.on_config_change(&event);
        }
        drop(listeners);

        // Send event via broadcast channel
        if self.event_tx.send(event.clone()).is_err() {
            warn!("[ConfigEventBus] Failed to send event: no active receivers");
        }

        info!("[ConfigEventBus] Event published: {}", event.event_type());
    }

    /// Subscribe to configuration change events
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigEvent> {
        self.event_tx.subscribe()
    }

    /// Add an event listener
    pub fn add_listener(&self, listener: Box<dyn ConfigEventListener + Send + Sync>) {
        let mut listeners = self.listeners.write();
        listeners.push(listener);
        debug!(
            "[ConfigEventBus] Listener added, total listeners: {}",
            listeners.len()
        );
    }

    /// Remove an event listener (by index)
    pub fn remove_listener(&self, index: usize) {
        let mut listeners = self.listeners.write();
        if index < listeners.len() {
            listeners.remove(index);
            debug!(
                "[ConfigEventBus] Listener at index {} removed, total listeners: {}",
                index,
                listeners.len()
            );
        }
    }

    /// Add an audit log entry
    fn add_audit_entry(&self, entry: ConfigChangeAudit) {
        let mut log = self.audit_log.write();
        log.push(entry);

        // Keep audit log within max entry limit
        if log.len() > self.max_audit_entries {
            log.remove(0);
        }
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> Vec<ConfigChangeAudit> {
        self.audit_log.read().clone()
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) {
        let mut log = self.audit_log.write();
        log.clear();
        info!("[ConfigEventBus] Audit log cleared");
    }

    /// Get audit log entries since a given time
    pub fn get_audit_log_since(&self, since: DateTime<Utc>) -> Vec<ConfigChangeAudit> {
        self.audit_log
            .read()
            .iter()
            .filter(|entry| entry.timestamp > since)
            .cloned()
            .collect()
    }
}

impl Default for ConfigEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Global configuration event bus instance
static GLOBAL_BUS: std::sync::LazyLock<ConfigEventBus> =
    std::sync::LazyLock::new(ConfigEventBus::default);

/// Publish a configuration change event to the global bus
pub fn publish_global_event(event: ConfigEvent) {
    GLOBAL_BUS.publish(event);
}

/// Subscribe to global config change events
pub fn subscribe_global_events() -> broadcast::Receiver<ConfigEvent> {
    GLOBAL_BUS.subscribe()
}

/// Add a global event listener
pub fn add_global_listener(listener: Box<dyn ConfigEventListener + Send + Sync>) {
    GLOBAL_BUS.add_listener(listener);
}

/// Get global audit log
pub fn get_global_audit_log() -> Vec<ConfigChangeAudit> {
    GLOBAL_BUS.get_audit_log()
}

/// Configuration hot-reload manager
pub struct ConfigHotReloadManager {
    event_bus: Arc<ConfigEventBus>,
    enabled: Arc<RwLock<bool>>,
}

impl ConfigHotReloadManager {
    pub fn new() -> Self {
        Self {
            event_bus: Arc::new(ConfigEventBus::new()),
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub fn from_bus(bus: Arc<ConfigEventBus>) -> Self {
        Self {
            event_bus: bus,
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Enable hot reload
    pub fn enable(&self) {
        *self.enabled.write() = true;
        info!("[ConfigHotReloadManager] Hot reload enabled");
    }

    /// Disable hot reload
    pub fn disable(&self) {
        *self.enabled.write() = false;
        info!("[ConfigHotReloadManager] Hot reload disabled");
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Handle configuration change
    pub fn handle_config_change(&self, event: ConfigEvent) {
        if !self.is_enabled() {
            debug!("[ConfigHotReloadManager] Hot reload disabled, ignoring event");
            return;
        }

        debug!(
            "[ConfigHotReloadManager] Handling config change: {}",
            event.event_type()
        );

        // Publish event to bus
        self.event_bus.publish(event);
    }
}

impl Default for ConfigHotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigEventListener for ConfigHotReloadManager {
    fn on_config_change(&self, event: &ConfigEvent) {
        if self.is_enabled() {
            debug!(
                "[ConfigHotReloadManager] Processing config change: {}",
                event.event_type()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::provider_config_watcher::ConfigEventType;
    use super::*;
    use anyhow::Result;

    struct TestListener {
        name: String,
        events_received: Arc<RwLock<Vec<String>>>,
    }

    impl ConfigEventListener for TestListener {
        fn on_config_change(&self, event: &ConfigEvent) {
            let mut events = self.events_received.write();
            events.push(format!("{}:{}", self.name, event.event_type()));
        }
    }

    #[tokio::test]
    async fn test_config_event_bus_publish() -> Result<()> {
        let bus = ConfigEventBus::new();
        let mut rx = bus.subscribe();

        let event = ConfigEvent::System {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };

        bus.publish(event.clone());

        let received = rx.recv().await?;
        assert_eq!(received.event_type(), "system");
        Ok(())
    }

    #[test]
    fn test_config_event_audit() -> Result<()> {
        let bus = ConfigEventBus::new();

        let event = ConfigEvent::Provider(ProviderConfigEvent::Updated {
            providers_count: 5,
            models_count: 10,
        });

        bus.publish(event);

        let audit_log = bus.get_audit_log();
        assert_eq!(audit_log.len(), 1);
        assert_eq!(audit_log[0].source, "provider");
        assert_eq!(audit_log[0].event_type, ConfigEventType::Updated);
        Ok(())
    }

    #[test]
    fn test_config_event_listener() -> Result<()> {
        let bus = ConfigEventBus::new();
        let events_received = Arc::new(RwLock::new(Vec::new()));

        let listener = TestListener {
            name: "test_listener".to_string(),
            events_received: events_received.clone(),
        };

        bus.add_listener(Box::new(listener));

        let event = ConfigEvent::System {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };

        bus.publish(event);

        let received = events_received.read();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], "test_listener:system");
        Ok(())
    }

    #[test]
    fn test_hot_reload_manager() -> Result<()> {
        let manager = ConfigHotReloadManager::new();

        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
        Ok(())
    }

    #[test]
    fn test_config_change_audit() -> Result<()> {
        let event = ConfigEvent::Provider(ProviderConfigEvent::Updated {
            providers_count: 3,
            models_count: 7,
        });

        let audit = ConfigChangeAudit::new(&event);

        assert_eq!(audit.source, "provider");
        assert_eq!(audit.event_type, ConfigEventType::Updated);
        assert!(audit.details.contains("3 providers"));
        assert!(audit.details.contains("7 models"));
        Ok(())
    }

    #[test]
    fn test_audit_log_limit() -> Result<()> {
        let bus = ConfigEventBus::new_with_limit(5);

        for i in 0..10 {
            let event = ConfigEvent::System {
                key: format!("key_{}", i),
                value: format!("value_{}", i),
            };
            bus.publish(event);
        }

        let audit_log = bus.get_audit_log();
        assert_eq!(audit_log.len(), 5);
        // Should keep the last 5 entries
        assert_eq!(
            audit_log[0].details,
            "System config updated: key_5 = value_5"
        );
        Ok(())
    }
}
