use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EgressMode {
    #[default]
    DenyAll,
    AllowAll,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressRule {
    pub host: String,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressPolicy {
    pub mode: EgressMode,
    #[serde(default)]
    pub allowed_hosts: Vec<EgressRule>,
    #[serde(default)]
    pub allowed_networks: Vec<String>,
    #[serde(default)]
    pub dns_servers: Vec<String>,
}

impl Default for EgressPolicy {
    fn default() -> Self {
        Self {
            mode: EgressMode::DenyAll,
            allowed_hosts: Vec::new(),
            allowed_networks: Vec::new(),
            dns_servers: Vec::new(),
        }
    }
}

impl EgressPolicy {
    pub fn deny_all() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            mode: EgressMode::AllowAll,
            allowed_hosts: Vec::new(),
            allowed_networks: Vec::new(),
            dns_servers: Vec::new(),
        }
    }

    pub fn whitelist() -> Self {
        Self {
            mode: EgressMode::Whitelist,
            ..Default::default()
        }
    }

    pub fn allow_host(mut self, host: &str) -> Self {
        self.allowed_hosts.push(EgressRule {
            host: host.to_string(),
            port: None,
            protocol: None,
            description: None,
        });
        self
    }

    pub fn allow_host_with_port(mut self, host: &str, port: u16) -> Self {
        self.allowed_hosts.push(EgressRule {
            host: host.to_string(),
            port: Some(port),
            protocol: None,
            description: None,
        });
        self
    }

    pub fn allow_network(mut self, cidr: &str) -> Self {
        self.allowed_networks.push(cidr.to_string());
        self
    }

    pub fn with_dns_server(mut self, dns: &str) -> Self {
        self.dns_servers.push(dns.to_string());
        self
    }

    pub fn is_restricted(&self) -> bool {
        self.mode != EgressMode::AllowAll
    }

    pub fn allowed_host_names(&self) -> Vec<&str> {
        self.allowed_hosts.iter().map(|r| r.host.as_str()).collect()
    }

    pub fn to_extra_hosts(&self) -> Vec<String> {
        Vec::new()
    }

    // WARNING: DNS-based egress control is a SOFT restriction only.
    //
    // It prevents name resolution but does NOT block direct IP connections.
    // An attacker who knows target IP addresses can bypass this control entirely.
    //
    // For production hard egress enforcement, use one of:
    // - iptables/nftables rules on the host
    // - Docker network policies with `--internal` flag
    // - Kubernetes NetworkPolicy (if running in K8s)
    // - A transparent DNS proxy that also filters by IP
    //
    // The allowed_hosts/allowed_networks data is preserved here as configuration
    // for external enforcement tools to consume.
    pub fn apply_to_docker_config(&self, host_config: &mut bollard::service::HostConfig) {
        if !self.is_restricted() {
            return;
        }

        match self.mode {
            EgressMode::DenyAll => {
                host_config.dns = Some(vec!["0.0.0.0".to_string()]);
            }
            EgressMode::Whitelist => {
                if !self.dns_servers.is_empty() {
                    host_config.dns = Some(self.dns_servers.clone());
                }
            }
            EgressMode::AllowAll => {}
        }
    }

    pub fn entelecheia_default() -> Self {
        Self::whitelist()
            .allow_host("api.openai.com")
            .allow_host("api.anthropic.com")
            .allow_host("generativelanguage.googleapis.com")
            .allow_host("api.groq.com")
            .allow_host("api.mistral.ai")
            .allow_host("api.together.xyz")
            .allow_host("api.fireworks.ai")
            .allow_host("openrouter.ai")
            .allow_host("localhost")
            .allow_network("127.0.0.0/8")
            .allow_network("172.16.0.0/12")
            .allow_network("10.0.0.0/8")
            .with_dns_server("1.1.1.1")
            .with_dns_server("8.8.8.8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, ensure};

    #[test]
    fn deny_all_is_restricted() -> Result<()> {
        let policy = EgressPolicy::deny_all();
        assert!(policy.is_restricted());
        assert_eq!(policy.mode, EgressMode::DenyAll);
        Ok(())
    }

    #[test]
    fn allow_all_is_not_restricted() -> Result<()> {
        let policy = EgressPolicy::allow_all();
        assert!(!policy.is_restricted());
        assert_eq!(policy.mode, EgressMode::AllowAll);
        Ok(())
    }

    #[test]
    fn whitelist_is_restricted() -> Result<()> {
        let policy = EgressPolicy::whitelist();
        assert!(policy.is_restricted());
        assert_eq!(policy.mode, EgressMode::Whitelist);
        Ok(())
    }

    #[test]
    fn allow_host_adds_to_list() -> Result<()> {
        let policy = EgressPolicy::whitelist()
            .allow_host("api.openai.com")
            .allow_host("api.anthropic.com");
        assert_eq!(policy.allowed_hosts.len(), 2);
        assert_eq!(
            policy.allowed_host_names(),
            vec!["api.openai.com", "api.anthropic.com"]
        );
        Ok(())
    }

    #[test]
    fn allow_host_with_port() -> Result<()> {
        let policy = EgressPolicy::whitelist().allow_host_with_port("api.openai.com", 443);
        assert_eq!(policy.allowed_hosts.len(), 1);
        assert_eq!(policy.allowed_hosts[0].port, Some(443));
        Ok(())
    }

    #[test]
    fn allow_network_adds_cidr() -> Result<()> {
        let policy = EgressPolicy::whitelist().allow_network("10.0.0.0/8");
        assert_eq!(policy.allowed_networks, vec!["10.0.0.0/8"]);
        Ok(())
    }

    #[test]
    fn to_extra_hosts_always_empty() -> Result<()> {
        let policy = EgressPolicy::whitelist().allow_host("api.openai.com");
        assert!(policy.to_extra_hosts().is_empty());
        let deny = EgressPolicy::deny_all();
        assert!(deny.to_extra_hosts().is_empty());
        Ok(())
    }

    #[test]
    fn default_policy_is_deny_all() -> Result<()> {
        let policy = EgressPolicy::default();
        assert_eq!(policy.mode, EgressMode::DenyAll);
        assert!(policy.allowed_hosts.is_empty());
        Ok(())
    }

    #[test]
    fn entelecheia_default_has_providers() -> Result<()> {
        let policy = EgressPolicy::entelecheia_default();
        assert!(policy.is_restricted());
        assert!(policy.allowed_hosts.len() >= 8);
        assert!(policy.allowed_networks.contains(&"127.0.0.0/8".to_string()));
        assert!(policy.dns_servers.contains(&"1.1.1.1".to_string()));
        assert!(policy.allowed_host_names().contains(&"api.openai.com"));
        assert!(policy.allowed_host_names().contains(&"api.anthropic.com"));
        Ok(())
    }

    #[test]
    fn entelecheia_default_includes_localhost() -> Result<()> {
        let policy = EgressPolicy::entelecheia_default();
        assert!(policy.allowed_host_names().contains(&"localhost"));
        assert!(
            policy
                .allowed_networks
                .iter()
                .any(|n| n.starts_with("172."))
        );
        assert!(policy.allowed_networks.iter().any(|n| n.starts_with("10.")));
        Ok(())
    }

    #[test]
    fn dns_servers_configured() -> Result<()> {
        let policy = EgressPolicy::entelecheia_default();
        assert!(policy.dns_servers.len() >= 2);
        Ok(())
    }

    #[test]
    fn apply_to_docker_config_deny_all_sets_dns_blackhole() -> Result<()> {
        let policy = EgressPolicy::deny_all();
        let mut host_config = bollard::service::HostConfig::default();
        policy.apply_to_docker_config(&mut host_config);
        assert_eq!(host_config.dns, Some(vec!["0.0.0.0".to_string()]));
        Ok(())
    }

    #[test]
    fn apply_to_docker_config_whitelist_sets_dns_resolvers() -> Result<()> {
        let policy = EgressPolicy::whitelist()
            .allow_host("api.openai.com")
            .with_dns_server("1.1.1.1");
        let mut host_config = bollard::service::HostConfig::default();
        policy.apply_to_docker_config(&mut host_config);
        assert_eq!(host_config.dns, Some(vec!["1.1.1.1".to_string()]));
        Ok(())
    }

    #[test]
    fn apply_to_docker_config_whitelist_no_dns_if_unset() -> Result<()> {
        let policy = EgressPolicy::whitelist().allow_host("api.openai.com");
        let mut host_config = bollard::service::HostConfig::default();
        policy.apply_to_docker_config(&mut host_config);
        assert!(host_config.dns.is_none());
        Ok(())
    }

    #[test]
    fn serde_roundtrip() -> Result<()> {
        let policy = EgressPolicy::entelecheia_default();
        let json = serde_json::to_string(&policy)?;
        let deserialized: EgressPolicy = serde_json::from_str(&json)?;
        assert_eq!(deserialized.mode, policy.mode);
        assert_eq!(deserialized.allowed_hosts.len(), policy.allowed_hosts.len());
        assert_eq!(
            deserialized.allowed_networks.len(),
            policy.allowed_networks.len()
        );
        Ok(())
    }

    #[test]
    fn entelecheia_default_includes_major_providers() -> Result<()> {
        let policy = EgressPolicy::entelecheia_default();
        let hosts = policy.allowed_host_names();
        ensure!(
            hosts.iter().any(|h| h.contains("openai")),
            "Should allow OpenAI domains"
        );
        ensure!(
            hosts.iter().any(|h| h.contains("anthropic")),
            "Should allow Anthropic domains"
        );
        Ok(())
    }

    #[test]
    fn apply_to_docker_config_allow_all_does_nothing() -> Result<()> {
        let policy = EgressPolicy::allow_all();
        let mut host_config = bollard::service::HostConfig::default();
        policy.apply_to_docker_config(&mut host_config);
        assert!(host_config.dns.is_none());
        Ok(())
    }
}
