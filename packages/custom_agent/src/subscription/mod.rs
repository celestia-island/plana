mod events;
mod manager;
mod signature;
pub(crate) use signature::{pubkey_for, subscription_owner, verify_agent_package};

use serde::{Deserialize, Serialize};

pub use events::{
    SubscriptionSyncReport, SyncAction, clone_repository, copy_dir_recursive, find_agent_root,
    install_subscription_agent, parse_check_interval, subscription_repo_url,
    validate_subscription_entry,
};
pub use manager::CustomAgentManager;

fn default_subscribe_version() -> String {
    "1.0".to_string()
}

fn default_check_interval() -> String {
    "daily".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeConfig {
    #[serde(default = "default_subscribe_version")]
    pub version: String,
    #[serde(default)]
    pub subscriptions: Vec<SubscriptionEntry>,
    #[serde(default)]
    pub settings: SubscribeSettings,
}

impl Default for SubscribeConfig {
    fn default() -> Self {
        Self {
            version: default_subscribe_version(),
            subscriptions: Vec::new(),
            settings: SubscribeSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeSettings {
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_check_interval")]
    pub check_interval: String,
    #[serde(default)]
    pub verify_signature: bool,
    #[serde(default)]
    pub trusted_sources: Vec<String>,
}

impl Default for SubscribeSettings {
    fn default() -> Self {
        Self {
            auto_update: false,
            check_interval: default_check_interval(),
            verify_signature: true,
            trusted_sources: vec!["official".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEntry {
    pub name: String,
    pub source: SubscriptionSource,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub enabled_tools: Option<Vec<String>>,
    #[serde(default)]
    pub enabled_skills: Option<Vec<String>>,
    #[serde(default)]
    pub granted_permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionSource {
    Official,
    Github,
    Url,
}

impl SubscriptionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Github => "github",
            Self::Url => "url",
        }
    }
}
