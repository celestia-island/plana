use serde::{Deserialize, Serialize};

pub use _core::execution_mode::{ExecutionMode, UnknownExecutionModeError};
use _core::shell_safety::contains_shell_metacharacters;

const SYSTEMCTL_STATUS_PREFIX: &str = "systemctl status";
const SYSTEMCTL_LIST_PREFIX: &str = "systemctl list";

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown access mode: {0}")]
pub struct UnknownAccessModeError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AccessMode {
    #[default]
    Read,
    Write,
    Execute,
}

impl std::str::FromStr for AccessMode {
    type Err = UnknownAccessModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(AccessMode::Read),
            "write" => Ok(AccessMode::Write),
            "execute" => Ok(AccessMode::Execute),
            _ => Err(UnknownAccessModeError(s.to_string())),
        }
    }
}

impl std::fmt::Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessMode::Read => write!(f, "read"),
            AccessMode::Write => write!(f, "write"),
            AccessMode::Execute => write!(f, "execute"),
        }
    }
}

/// Map a kirino `Permission` name string to an `AccessMode`.
///
/// The mapping follows the convention established in the hierarchical
/// permission model:
///   - `.read` / `.list`         → `AccessMode::Read`
///   - `.write` / `.create` / `.update` / `.delete` → `AccessMode::Write`
///   - `.execute` / `.use` / `.manage` / `.connect` → `AccessMode::Execute`
#[must_use]
pub fn permission_name_to_access_mode(name: &str) -> Option<AccessMode> {
    if name.ends_with(".read") || name.ends_with(".list") {
        Some(AccessMode::Read)
    } else if name.ends_with(".write")
        || name.ends_with(".create")
        || name.ends_with(".update")
        || name.ends_with(".delete")
    {
        Some(AccessMode::Write)
    } else if name.ends_with(".execute")
        || name.ends_with(".use")
        || name.ends_with(".manage")
        || name.ends_with(".connect")
    {
        Some(AccessMode::Execute)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Info,
    Safe,
    Unsafe,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolScope {
    InternalOnly,
    Any,
    ExternalApproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Internal,
    Trusted,
    Untrusted,
    Restricted,
}

impl TrustLevel {
    pub fn is_internal(&self) -> bool {
        matches!(self, TrustLevel::Internal)
    }

    pub fn allows_external_ops(&self) -> bool {
        matches!(self, TrustLevel::Trusted | TrustLevel::Internal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandSafety {
    ReadOnly,
    Idempotent,
    Destructive,
    Arbitrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCapability {
    pub access_mode: AccessMode,
    pub risk_level: RiskLevel,
    pub scope: ToolScope,
}

impl Default for ToolCapability {
    fn default() -> Self {
        Self {
            access_mode: AccessMode::Read,
            risk_level: RiskLevel::Info,
            scope: ToolScope::Any,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolRequest {
    pub agent_name: String,
    pub tool_name: String,
    #[serde(default)]
    pub access_mode: AccessMode,
}

impl Default for SkillToolRequest {
    fn default() -> Self {
        Self {
            agent_name: String::new(),
            tool_name: String::new(),
            access_mode: AccessMode::Read,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialKind {
    InsufficientAccessMode,
    ScopeRestricted,
    RiskRestricted,
    ApprovalRequired,
    ExecutionLocationRequired,
}

#[derive(Debug, Clone)]
pub struct PermissionDecision {
    pub allowed: bool,
    pub reason: String,
    pub denial_kind: Option<DenialKind>,
}

impl PermissionDecision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            reason: String::new(),
            denial_kind: None,
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            denial_kind: None,
        }
    }

    pub fn deny_with_kind(reason: impl Into<String>, kind: DenialKind) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
            denial_kind: Some(kind),
        }
    }
}

pub fn check_dual_authorization(
    tool_cap: &ToolCapability,
    skill_request: AccessMode,
    target_trust: TrustLevel,
) -> PermissionDecision {
    if tool_cap.access_mode < skill_request {
        return PermissionDecision::deny_with_kind(
            format!(
                "tool access_mode {:?} insufficient for skill request {:?}",
                tool_cap.access_mode, skill_request
            ),
            DenialKind::InsufficientAccessMode,
        );
    }

    if target_trust.is_internal() {
        return PermissionDecision::allow();
    }

    match tool_cap.scope {
        ToolScope::InternalOnly => PermissionDecision::deny_with_kind(
            "tool restricted to internal targets only",
            DenialKind::ScopeRestricted,
        ),
        ToolScope::ExternalApproved => PermissionDecision::deny_with_kind(
            "tool requires explicit approval for external targets",
            DenialKind::ApprovalRequired,
        ),
        ToolScope::Any => match tool_cap.risk_level {
            RiskLevel::Critical => PermissionDecision::deny_with_kind(
                "critical operations require approval",
                DenialKind::RiskRestricted,
            ),
            RiskLevel::Unsafe => {
                if skill_request >= AccessMode::Execute {
                    PermissionDecision::deny_with_kind(
                        "execute-level access on external targets requires elevated scope",
                        DenialKind::RiskRestricted,
                    )
                } else {
                    PermissionDecision::allow()
                }
            }
            RiskLevel::Safe | RiskLevel::Info => PermissionDecision::allow(),
        },
    }
}

pub fn check_execution_location_gate(tool_name: &str, can_run_locally: bool) -> PermissionDecision {
    if can_run_locally {
        PermissionDecision::allow()
    } else {
        PermissionDecision::deny_with_kind(
            format!(
                "tool '{}' requires cosmos container (not available in current execution context)",
                tool_name
            ),
            DenialKind::ExecutionLocationRequired,
        )
    }
}

pub fn classify_command(command: &str) -> CommandSafety {
    static DESTRUCTIVE_PATTERNS: &[&str] = &[
        "rm ",
        "rm\t",
        "rmdir ",
        "dd ",
        "mkfs.",
        "format ",
        "shutdown",
        "reboot",
        "halt",
        "poweroff",
        "kill ",
        "killall ",
        "systemctl stop",
        "systemctl disable",
        "systemctl restart",
        "docker rm",
        "podman rm",
        "python3 -c",
        "python -c",
        "perl -e",
        "ruby -e",
        "node -e",
        "bash -c",
        "sh -c",
        "zsh -c",
        "lua -e",
        "php -r",
        "awk '{",
        "awk 'BEGIN",
        "sed 'e",
    ];

    static DESTRUCTIVE_BASES: &[&str] = &[
        "rm", "rmdir", "dd", "mkfs", "format", "shutdown", "reboot", "halt", "poweroff", "kill",
        "killall", "eval", "source",
    ];

    static IDEMPOTENT_PATTERNS: &[&str] = &[
        "mkdir -p", "touch ", "chmod ", "chown ", "cp ", "ln -s", "scp ", "rsync ",
    ];

    static READ_ONLY_PATTERNS: &[&str] = &[
        "curl --head",
        "curl -I",
        "docker ps",
        "docker logs",
        "docker images",
        "podman ps",
        "podman logs",
        "git status",
        "git log",
        "git diff",
        "git branch",
        "git show",
        "git remote",
        "python3 --version",
        "node --version",
        "rustc --version",
        "cargo --version",
        "systemctl list",
        "cat /proc",
        "cat /etc",
    ];

    static READ_ONLY_COMMANDS: &[&str] = &[
        "ls",
        "cat",
        "head",
        "tail",
        "stat",
        "file",
        "find",
        "grep",
        "ps",
        "top",
        "df",
        "du",
        "free",
        "uname",
        "hostname",
        "id",
        "whoami",
        "env",
        "printenv",
        "pwd",
        "echo",
        "date",
        "uptime",
        "netstat",
        "ss",
        "ip",
        "ifconfig",
        "ping",
        "traceroute",
        "nslookup",
        "dig",
        "jq",
        "wc",
        "sort",
        "uniq",
        "less",
        "more",
        "which",
        "whereis",
        "type",
    ];

    static IDEMPOTENT_BASES: &[&str] = &["awk", "sed"];

    let trimmed = command.trim();

    if contains_shell_metacharacters(trimmed) {
        return CommandSafety::Destructive;
    }

    let mut parts = trimmed.split_whitespace();
    let first_token = parts.next().unwrap_or("");
    let base = if first_token.contains('/') {
        first_token.rsplit('/').next().unwrap_or(first_token)
    } else {
        first_token
    };
    let base = base.strip_prefix('\\').unwrap_or(base);

    let normalized = {
        let rest: String = parts.collect::<Vec<_>>().join(" ");
        if rest.is_empty() {
            base.to_string()
        } else {
            format!("{} {}", base, rest)
        }
    };

    for pat in DESTRUCTIVE_PATTERNS {
        if normalized.starts_with(pat) {
            return CommandSafety::Destructive;
        }
    }

    for pat in IDEMPOTENT_PATTERNS {
        if normalized.starts_with(pat) {
            return CommandSafety::Idempotent;
        }
    }

    for pat in READ_ONLY_PATTERNS {
        if normalized.starts_with(pat) {
            return CommandSafety::ReadOnly;
        }
    }

    if READ_ONLY_COMMANDS.contains(&base) {
        return CommandSafety::ReadOnly;
    }

    if DESTRUCTIVE_BASES.contains(&base) {
        return CommandSafety::Destructive;
    }

    if is_scripting_inline_code(base, &normalized) {
        return CommandSafety::Destructive;
    }

    if IDEMPOTENT_BASES.contains(&base) {
        return CommandSafety::Idempotent;
    }

    if normalized.starts_with(SYSTEMCTL_STATUS_PREFIX)
        || normalized.starts_with(SYSTEMCTL_LIST_PREFIX)
    {
        return CommandSafety::ReadOnly;
    }

    CommandSafety::Arbitrary
}

fn is_scripting_inline_code(base: &str, normalized: &str) -> bool {
    static SCRIPTING_TOOLS: &[&str] = &[
        "awk", "sed", "perl", "python", "python3", "ruby", "node", "bash", "sh", "zsh", "lua",
        "php",
    ];
    static INLINE_PATTERNS: &[&str] = &["'{", "'BEGIN", "'e", "-c", "-e", "-r"];
    if !SCRIPTING_TOOLS.contains(&base) {
        return false;
    }
    let after_base = &normalized[base.len()..];
    for pat in INLINE_PATTERNS {
        if after_base.contains(pat) {
            return true;
        }
    }
    if base == "sed"
        && let Some(q) = after_base.find('\'')
        && after_base[q + 1..].contains("e ")
    {
        return true;
    }
    false
}

pub fn check_command_safety(safety: CommandSafety, trust: TrustLevel) -> PermissionDecision {
    match trust {
        TrustLevel::Internal => PermissionDecision::allow(),
        TrustLevel::Trusted => match safety {
            CommandSafety::Destructive => PermissionDecision::deny(
                "destructive commands on trusted external nodes require approval",
            ),
            _ => PermissionDecision::allow(),
        },
        TrustLevel::Untrusted => match safety {
            CommandSafety::ReadOnly | CommandSafety::Idempotent => PermissionDecision::allow(),
            CommandSafety::Destructive | CommandSafety::Arbitrary => {
                PermissionDecision::deny("arbitrary/destructive commands denied on untrusted nodes")
            }
        },
        TrustLevel::Restricted => match safety {
            CommandSafety::ReadOnly => PermissionDecision::allow(),
            _ => PermissionDecision::deny("only read-only commands allowed on restricted nodes"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_mode_ordering() -> anyhow::Result<()> {
        assert!(AccessMode::Read < AccessMode::Write);
        assert!(AccessMode::Write < AccessMode::Execute);
        assert!(AccessMode::Read < AccessMode::Execute);
        Ok(())
    }

    #[test]
    fn test_risk_level_ordering() -> anyhow::Result<()> {
        assert!(RiskLevel::Info < RiskLevel::Safe);
        assert!(RiskLevel::Safe < RiskLevel::Unsafe);
        assert!(RiskLevel::Unsafe < RiskLevel::Critical);
        Ok(())
    }

    #[test]
    fn test_internal_bypasses_scope_and_risk_checks() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Execute,
            risk_level: RiskLevel::Critical,
            scope: ToolScope::InternalOnly,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Execute, TrustLevel::Internal);
        assert!(decision.allowed);
        Ok(())
    }

    #[test]
    fn test_internal_still_requires_sufficient_access_mode() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Read,
            risk_level: RiskLevel::Info,
            scope: ToolScope::Any,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Write, TrustLevel::Internal);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_insufficient_access_mode() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Read,
            risk_level: RiskLevel::Info,
            scope: ToolScope::Any,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Write, TrustLevel::Internal);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_external_internal_only_scope() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Execute,
            risk_level: RiskLevel::Unsafe,
            scope: ToolScope::InternalOnly,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Read, TrustLevel::Trusted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_external_unsafe_execute_denied() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Execute,
            risk_level: RiskLevel::Unsafe,
            scope: ToolScope::Any,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Execute, TrustLevel::Trusted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_external_safe_read_allowed() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Read,
            risk_level: RiskLevel::Safe,
            scope: ToolScope::Any,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Read, TrustLevel::Trusted);
        assert!(decision.allowed);
        Ok(())
    }

    #[test]
    fn test_external_critical_denied() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Write,
            risk_level: RiskLevel::Critical,
            scope: ToolScope::Any,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Write, TrustLevel::Trusted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_classify_command_read_only() -> anyhow::Result<()> {
        assert_eq!(classify_command("ls -la"), CommandSafety::ReadOnly);
        assert_eq!(classify_command("cat /etc/hosts"), CommandSafety::ReadOnly);
        assert_eq!(classify_command("ps aux"), CommandSafety::ReadOnly);
        assert_eq!(classify_command("df -h"), CommandSafety::ReadOnly);
        assert_eq!(
            classify_command("/usr/bin/hostname"),
            CommandSafety::ReadOnly
        );
        Ok(())
    }

    #[test]
    fn test_classify_command_destructive() -> anyhow::Result<()> {
        assert_eq!(classify_command("rm -rf /"), CommandSafety::Destructive);
        assert_eq!(classify_command("reboot"), CommandSafety::Destructive);
        assert_eq!(classify_command("kill -9 1234"), CommandSafety::Destructive);
        assert_eq!(
            classify_command("dd if=/dev/zero of=/dev/sda"),
            CommandSafety::Destructive
        );
        Ok(())
    }

    #[test]
    fn test_classify_command_idempotent() -> anyhow::Result<()> {
        assert_eq!(
            classify_command("mkdir -p /tmp/test"),
            CommandSafety::Idempotent
        );
        assert_eq!(
            classify_command("chmod 755 /tmp/test"),
            CommandSafety::Idempotent
        );
        assert_eq!(
            classify_command("touch /tmp/file"),
            CommandSafety::Idempotent
        );
        Ok(())
    }

    #[test]
    fn test_classify_command_arbitrary() -> anyhow::Result<()> {
        assert_eq!(
            classify_command("curl http://example.com"),
            CommandSafety::Arbitrary
        );
        assert_eq!(
            classify_command("python3 script.py"),
            CommandSafety::Arbitrary
        );
        assert_eq!(
            classify_command("bash -c 'echo hello'"),
            CommandSafety::Destructive
        );
        Ok(())
    }

    #[test]
    fn test_command_safety_trusted_destructive_denied() -> anyhow::Result<()> {
        let decision = check_command_safety(CommandSafety::Destructive, TrustLevel::Trusted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_command_safety_untrusted_arbitrary_denied() -> anyhow::Result<()> {
        let decision = check_command_safety(CommandSafety::Arbitrary, TrustLevel::Untrusted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_command_safety_restricted_readonly_allowed() -> anyhow::Result<()> {
        let decision = check_command_safety(CommandSafety::ReadOnly, TrustLevel::Restricted);
        assert!(decision.allowed);
        Ok(())
    }

    #[test]
    fn test_command_safety_restricted_write_denied() -> anyhow::Result<()> {
        let decision = check_command_safety(CommandSafety::Idempotent, TrustLevel::Restricted);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn test_execution_location_gate_allow() -> anyhow::Result<()> {
        let decision = check_execution_location_gate("file_read", true);
        assert!(decision.allowed);
        assert!(decision.denial_kind.is_none());
        Ok(())
    }

    #[test]
    fn test_execution_location_gate_requires_cosmos() -> anyhow::Result<()> {
        let decision = check_execution_location_gate("container_fork", false);
        assert!(!decision.allowed);
        assert_eq!(
            decision.denial_kind,
            Some(DenialKind::ExecutionLocationRequired)
        );
        assert!(decision.reason.contains("cosmos container"));
        Ok(())
    }

    #[test]
    fn test_denial_kind_in_dual_auth() -> anyhow::Result<()> {
        let cap = ToolCapability {
            access_mode: AccessMode::Read,
            risk_level: RiskLevel::Info,
            scope: ToolScope::InternalOnly,
        };
        let decision = check_dual_authorization(&cap, AccessMode::Read, TrustLevel::Untrusted);
        assert!(!decision.allowed);
        assert_eq!(decision.denial_kind, Some(DenialKind::ScopeRestricted));
        Ok(())
    }
}
