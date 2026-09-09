//! Outbound proxy resolution — the `tauri.net.set_proxy` payload and the
//! layered decision (bridge override → environment → direct).
//!
//! The bridge never honors ambient `HTTP(S)_PROXY`: a factory/desktop
//! tool's traffic policy is explicit configuration, not ambient
//! environment. Direct is the default.

use serde::{Deserialize, Serialize};

/// Proxy wire type as installed by `tauri.net.set_proxy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// `http` | `https` | `socks5`; anything else is rejected upstream.
    pub scheme: String,
    /// `host:port` — the proxy gateway.
    pub host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl ProxyConfig {
    /// Full proxy URL (`scheme://host`), the form reqwest/tungstenite
    /// dialers take.
    pub fn url(&self) -> String {
        format!("{}://{}", self.scheme, self.host)
    }
}

/// The resolved outbound decision the bridge hands to endpoint dialers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyDecision {
    /// Explicitly direct (the default — and what an empty `host` in
    /// `tauri.net.set_proxy` installs).
    Direct,
    /// Route through the configured proxy.
    Through(ProxyConfig),
}

/// Resolve the decision: `override` (from the bridge's persisted
/// settings) wins; otherwise the `PLANA_PROXY` / `PLANA_PROXY_SCHEME`
/// (etc.) environment pair; otherwise direct.
pub fn resolve(override_config: Option<&ProxyConfig>) -> ProxyDecision {
    if let Some(config) = override_config {
        if config.host.trim().is_empty() {
            return ProxyDecision::Direct;
        }
        return ProxyDecision::Through(config.clone());
    }
    match std::env::var("PLANA_PROXY") {
        Ok(host) if !host.trim().is_empty() => ProxyDecision::Through(ProxyConfig {
            scheme: std::env::var("PLANA_PROXY_SCHEME").unwrap_or_else(|_| "http".into()),
            host,
            username: std::env::var("PLANA_PROXY_USERNAME").ok(),
            password: std::env::var("PLANA_PROXY_PASSWORD").ok(),
        }),
        _ => ProxyDecision::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_with_empty_host_is_direct() {
        let empty = ProxyConfig {
            scheme: "http".into(),
            host: "  ".into(),
            username: None,
            password: None,
        };
        assert_eq!(resolve(Some(&empty)), ProxyDecision::Direct);
    }

    #[test]
    fn override_carries_credentials() {
        let cfg = ProxyConfig {
            scheme: "socks5".into(),
            host: "127.0.0.1:7890".into(),
            username: Some("factory".into()),
            password: None,
        };
        assert_eq!(resolve(Some(&cfg)), ProxyDecision::Through(cfg.clone()));
        assert_eq!(cfg.url(), "socks5://127.0.0.1:7890");
    }
}
