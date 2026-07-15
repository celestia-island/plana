use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EgressModeConfig {
    #[default]
    DenyAll,
    AllowAll,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressHostConfig {
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressPolicyConfig {
    #[serde(default)]
    pub mode: EgressModeConfig,
    #[serde(default)]
    pub allowed_hosts: Vec<EgressHostConfig>,
    #[serde(default)]
    pub allowed_networks: Vec<String>,
    #[serde(default)]
    pub dns_servers: Vec<String>,
}

impl Default for EgressPolicyConfig {
    fn default() -> Self {
        Self {
            mode: EgressModeConfig::DenyAll,
            allowed_hosts: Vec::new(),
            allowed_networks: Vec::new(),
            dns_servers: Vec::new(),
        }
    }
}

impl EgressPolicyConfig {
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn entelecheia_default() -> Self {
        let hosts = vec![
            "api.openai.com",
            "api.anthropic.com",
            "generativelanguage.googleapis.com",
            "api.groq.com",
            "api.mistral.ai",
            "api.together.xyz",
            "api.fireworks.ai",
            "openrouter.ai",
            "localhost",
        ];

        Self {
            mode: EgressModeConfig::Whitelist,
            allowed_hosts: hosts
                .into_iter()
                .map(|h| EgressHostConfig {
                    host: h.to_string(),
                    port: None,
                    description: None,
                })
                .collect(),
            allowed_networks: vec![
                "127.0.0.0/8".into(),
                "172.16.0.0/12".into(),
                "10.0.0.0/8".into(),
            ],
            dns_servers: vec!["1.1.1.1".into(), "8.8.8.8".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn default_is_deny_all() -> Result<()> {
        let config = EgressPolicyConfig::default();
        assert_eq!(config.mode, EgressModeConfig::DenyAll);
        Ok(())
    }

    #[test]
    fn entelecheia_default_is_whitelist() -> Result<()> {
        let config = EgressPolicyConfig::entelecheia_default();
        assert_eq!(config.mode, EgressModeConfig::Whitelist);
        assert!(config.allowed_hosts.len() >= 8);
        Ok(())
    }

    #[test]
    fn toml_roundtrip() -> Result<()> {
        let config = EgressPolicyConfig::entelecheia_default();
        let toml_str = config.to_toml_string().context("test precondition")?;
        let parsed = EgressPolicyConfig::from_toml_str(&toml_str).context("test precondition")?;
        assert_eq!(parsed.mode, config.mode);
        assert_eq!(parsed.allowed_hosts.len(), config.allowed_hosts.len());
        Ok(())
    }

    #[test]
    fn parses_minimal_config() -> Result<()> {
        let toml_str = r#"
mode = "Whitelist"
"#;
        let config = EgressPolicyConfig::from_toml_str(toml_str).context("test precondition")?;
        assert_eq!(config.mode, EgressModeConfig::Whitelist);
        assert!(config.allowed_hosts.is_empty());
        Ok(())
    }

    #[test]
    fn parses_full_config() -> Result<()> {
        let toml_str = r#"
mode = "Whitelist"

[[allowed_hosts]]
host = "api.openai.com"
port = 443

[[allowed_hosts]]
host = "api.anthropic.com"
"#;
        let config = EgressPolicyConfig::from_toml_str(toml_str).context("test precondition")?;
        assert_eq!(config.allowed_hosts.len(), 2);
        assert_eq!(config.allowed_hosts[0].port, Some(443));
        Ok(())
    }
}
