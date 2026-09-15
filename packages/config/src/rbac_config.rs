//! RBAC configuration for the services that embed this crate.
//!
//! Authorization here is unconditional: this surface carries no enable/disable
//! switch and no super-admin key list. The retired `RBAC_ENABLED` and
//! `RBAC_SUPER_ADMIN_KEYS` variables are deliberately ignored — the tests below
//! pin that — so no environment can disarm the authorization layer or mint
//! extra super-admin credentials behind the RBAC service's back.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    #[serde(default)]
    pub default_role: Option<String>,
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
}

fn default_strict_mode() -> bool {
    true
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            default_role: None,
            strict_mode: true,
        }
    }
}

impl RbacConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(val) = std::env::var("RBAC_DEFAULT_ROLE") {
            config.default_role = Some(val);
        }
        if let Ok(val) = std::env::var("RBAC_STRICT_MODE") {
            config.strict_mode = val.parse().unwrap_or(true);
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::RbacConfig;

    /// The retired switches are gone from the serialized surface: nothing in
    /// `RbacConfig` may serialize under their names, so a config file or a
    /// container env cannot reintroduce an authorization off-switch.
    #[test]
    fn retired_authorization_switches_are_absent_from_the_config_surface() {
        let value = serde_json::to_value(RbacConfig::default()).expect("RbacConfig serializes");
        let fields = value.as_object().expect("RbacConfig is an object");

        assert!(
            !fields.contains_key("enabled"),
            "`enabled` (RBAC_ENABLED) would let configuration disarm authorization: {value}"
        );
        assert!(
            !fields.contains_key("super_admin_api_keys"),
            "`super_admin_api_keys` (RBAC_SUPER_ADMIN_KEYS) would bypass the RBAC service: {value}"
        );
    }

    /// `from_env` ignores the retired variables: a container launched with
    /// `RBAC_ENABLED=false` and a populated `RBAC_SUPER_ADMIN_KEYS` resolves to
    /// exactly the same config as one launched without them.
    #[test]
    fn retired_authorization_env_switches_are_inert() {
        const RETIRED: [&str; 2] = ["RBAC_ENABLED", "RBAC_SUPER_ADMIN_KEYS"];
        const STILL_SUPPORTED: [&str; 2] = ["RBAC_DEFAULT_ROLE", "RBAC_STRICT_MODE"];

        // SAFETY: test-only mutation of process-local env vars; both arrays name
        // variables read by this module alone.
        unsafe {
            for name in STILL_SUPPORTED {
                std::env::remove_var(name);
            }
            std::env::set_var(RETIRED[0], "false");
            std::env::set_var(RETIRED[1], "placeholder-super-admin-key");
        }

        let with_retired_switches =
            serde_json::to_value(RbacConfig::from_env()).expect("RbacConfig serializes");

        unsafe {
            for name in RETIRED {
                std::env::remove_var(name);
            }
        }

        let without_retired_switches =
            serde_json::to_value(RbacConfig::from_env()).expect("RbacConfig serializes");

        assert_eq!(
            with_retired_switches, without_retired_switches,
            "the retired RBAC_* switches must not change the resolved config"
        );
    }
}
