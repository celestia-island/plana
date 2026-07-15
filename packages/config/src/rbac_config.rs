use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub default_role: Option<String>,
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
    #[serde(default)]
    pub super_admin_api_keys: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

fn default_strict_mode() -> bool {
    true
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_role: None,
            strict_mode: true,
            super_admin_api_keys: Vec::new(),
        }
    }
}

impl RbacConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(val) = std::env::var("RBAC_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }
        if let Ok(val) = std::env::var("RBAC_DEFAULT_ROLE") {
            config.default_role = Some(val);
        }
        if let Ok(val) = std::env::var("RBAC_STRICT_MODE") {
            config.strict_mode = val.parse().unwrap_or(true);
        }
        if let Ok(val) = std::env::var("RBAC_SUPER_ADMIN_KEYS") {
            config.super_admin_api_keys = val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        config
    }
}
