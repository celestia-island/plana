use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DangerousFlagSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for DangerousFlagSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DangerousFlagSeverity::Info => write!(f, "info"),
            DangerousFlagSeverity::Warning => write!(f, "warning"),
            DangerousFlagSeverity::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousFlag {
    pub key: String,
    pub current_value: String,
    pub severity: DangerousFlagSeverity,
    pub recommendation: String,
}

pub fn collect_dangerous_flags(config: &serde_json::Value) -> Vec<DangerousFlag> {
    let mut flags = Vec::new();

    check_bind_address(config, &mut flags);
    check_tool_wildcard(config, &mut flags);
    check_sandbox_mode(config, &mut flags);
    check_sensitive_mount(config, &mut flags);
    check_insecure_auth(config, &mut flags);

    flags.sort_by(|a, b| {
        let a_order = match a.severity {
            DangerousFlagSeverity::Critical => 0,
            DangerousFlagSeverity::Warning => 1,
            DangerousFlagSeverity::Info => 2,
        };
        let b_order = match b.severity {
            DangerousFlagSeverity::Critical => 0,
            DangerousFlagSeverity::Warning => 1,
            DangerousFlagSeverity::Info => 2,
        };
        a_order.cmp(&b_order)
    });

    flags
}

fn check_bind_address(config: &serde_json::Value, flags: &mut Vec<DangerousFlag>) {
    let bind = config
        .get("gateway")
        .and_then(|g| g.get("bind"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if bind == "0.0.0.0" || bind == "::" {
        flags.push(DangerousFlag {
            key: "gateway.bind".to_string(),
            current_value: bind.to_string(),
            severity: DangerousFlagSeverity::Warning,
            recommendation: "Binding to all interfaces exposes the API to the network. \
                Consider binding to 127.0.0.1 or a specific interface."
                .to_string(),
        });
    }
}

fn check_tool_wildcard(config: &serde_json::Value, flags: &mut Vec<DangerousFlag>) {
    let tools = config
        .get("tools")
        .and_then(|t| t.get("allow"))
        .and_then(|v| v.as_array());

    if let Some(arr) = tools
        && arr.iter().any(|v| v.as_str() == Some("*"))
    {
        flags.push(DangerousFlag {
            key: "tools.allow".to_string(),
            current_value: "*".to_string(),
            severity: DangerousFlagSeverity::Warning,
            recommendation: "Wildcard tool allowlist grants access to all tools. \
                    Consider restricting to specific tool names."
                .to_string(),
        });
    }
}

fn check_sandbox_mode(config: &serde_json::Value, flags: &mut Vec<DangerousFlag>) {
    let mode = config
        .get("sandbox")
        .and_then(|s| s.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if mode == "off" || mode == "host" {
        flags.push(DangerousFlag {
            key: "sandbox.mode".to_string(),
            current_value: mode.to_string(),
            severity: DangerousFlagSeverity::Warning,
            recommendation: "Sandbox is disabled — tool execution runs on the host without \
                container isolation. This is acceptable for development but not for production."
                .to_string(),
        });
    }
}

fn check_sensitive_mount(config: &serde_json::Value, flags: &mut Vec<DangerousFlag>) {
    let workspace = config
        .get("workspace")
        .and_then(|w| w.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let sensitive_paths = [
        "/", "/etc", "/root", "/home", "/var", "/usr", "/sys", "/proc",
    ];

    for &sp in &sensitive_paths {
        if workspace == sp {
            flags.push(DangerousFlag {
                key: "workspace.path".to_string(),
                current_value: workspace.to_string(),
                severity: DangerousFlagSeverity::Critical,
                recommendation: format!(
                    "Workspace is set to a sensitive system directory '{}'. \
                     This grants the agent access to critical system files.",
                    sp
                ),
            });
            break;
        }
    }
}

fn check_insecure_auth(config: &serde_json::Value, flags: &mut Vec<DangerousFlag>) {
    let auth = config
        .get("auth")
        .and_then(|a| a.get("enabled"))
        .and_then(|v| v.as_bool());

    if Some(false) == auth {
        flags.push(DangerousFlag {
            key: "auth.enabled".to_string(),
            current_value: "false".to_string(),
            severity: DangerousFlagSeverity::Critical,
            recommendation: "Authentication is disabled. All API endpoints are accessible \
                without credentials. Enable authentication for any non-local deployment."
                .to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_wildcard_bind() {
        let config = json!({"gateway": {"bind": "0.0.0.0"}});
        let flags = collect_dangerous_flags(&config);
        assert!(flags.iter().any(|f| f.key == "gateway.bind"));
    }

    #[test]
    fn no_warning_for_localhost_bind() {
        let config = json!({"gateway": {"bind": "127.0.0.1"}});
        let flags = collect_dangerous_flags(&config);
        assert!(!flags.iter().any(|f| f.key == "gateway.bind"));
    }

    #[test]
    fn detects_tool_wildcard() {
        let config = json!({"tools": {"allow": ["*"]}});
        let flags = collect_dangerous_flags(&config);
        assert!(flags.iter().any(|f| f.key == "tools.allow"));
    }

    #[test]
    fn detects_sandbox_off() {
        let config = json!({"sandbox": {"mode": "off"}});
        let flags = collect_dangerous_flags(&config);
        assert!(flags.iter().any(|f| f.key == "sandbox.mode"));
    }

    #[test]
    fn detects_sensitive_mount() {
        let config = json!({"workspace": {"path": "/etc"}});
        let flags = collect_dangerous_flags(&config);
        assert!(flags.iter().any(|f| f.key == "workspace.path" && f.severity == DangerousFlagSeverity::Critical));
    }

    #[test]
    fn detects_auth_disabled() {
        let config = json!({"auth": {"enabled": false}});
        let flags = collect_dangerous_flags(&config);
        assert!(
            flags
                .iter()
                .any(|f| f.key == "auth.enabled" && f.severity == DangerousFlagSeverity::Critical)
        );
    }

    #[test]
    fn no_flags_for_safe_config() {
        let config = json!({"gateway": {"bind": "127.0.0.1"}});
        let flags = collect_dangerous_flags(&config);
        assert!(flags.is_empty());
    }

    #[test]
    fn flags_sorted_by_severity() {
        let config = json!({
            "gateway": {"bind": "0.0.0.0"},
            "workspace": {"path": "/etc"},
            "sandbox": {"mode": "off"}
        });
        let flags = collect_dangerous_flags(&config);
        if flags.len() >= 2 {
            assert!(matches!(flags[0].severity, DangerousFlagSeverity::Critical));
        }
    }
}
