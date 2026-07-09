use anyhow::{Context, Result, anyhow};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use tracing::warn;

const FILE_AGENT_TOML: &str = "agent.toml";
const PATH_TRAVERSAL_PREFIX: &str = "../";
const DOCKERFILE_PREFIX: &str = "dockerfile";
const CONTAINER_PREFIX: &str = "container";
const DOCKER_USER_DIRECTIVE: &str = "user:";
const DOCKER_USER_ROOT: &str = "user: root";
const DOCKER_USER_ROOT_QUOTED: &str = "user: \"root\"";

use super::{
    FindingSeverity, Layer3AgentManifest, LocalLayer3Agent, PreflightAuditReport,
    PreflightDecision, PreflightFinding,
};

pub(super) fn load_toml_value(path: &Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read TOML file: {}", path.display()))?;
    toml::from_str::<toml::Value>(&raw)
        .with_context(|| format!("failed to parse TOML file: {}", path.display()))
}

pub(super) fn merge_toml(base: toml::Value, overlay: toml::Value) -> toml::Value {
    match (base, overlay) {
        (toml::Value::Table(mut left), toml::Value::Table(right)) => {
            for (k, v) in right {
                if let Some(existing) = left.remove(&k) {
                    left.insert(k, merge_toml(existing, v));
                } else {
                    left.insert(k, v);
                }
            }
            toml::Value::Table(left)
        }
        (_, right) => right,
    }
}

pub fn load_manifest_from_dir(dir: &Path) -> Result<Layer3AgentManifest> {
    let manifest_path = dir.join(FILE_AGENT_TOML);
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read agent.toml: {}", manifest_path.display()))?;
    toml::from_str::<Layer3AgentManifest>(&raw)
        .with_context(|| format!("failed to parse agent.toml: {}", manifest_path.display()))
}

pub(super) fn load_local_agents(amphoreus_dir: &Path) -> Result<Vec<LocalLayer3Agent>> {
    let mut agents = Vec::new();
    for entry in std::fs::read_dir(amphoreus_dir)
        .with_context(|| format!("failed to read directory: {}", amphoreus_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        let manifest_path = path.join(FILE_AGENT_TOML);
        if !manifest_path.exists() {
            continue;
        }

        let raw = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read agent.toml: {}", manifest_path.display()))?;
        let manifest = toml::from_str::<Layer3AgentManifest>(&raw)
            .with_context(|| format!("failed to parse agent.toml: {}", manifest_path.display()))?;

        if manifest.agent.layer != 3 {
            return Err(anyhow!(
                "layer={} is not Layer3 in {}",
                manifest.agent.layer,
                manifest_path.display()
            ));
        }
        if manifest.agent.id.trim().is_empty() {
            return Err(anyhow!(
                "agent.id must not be empty in {}",
                manifest_path.display()
            ));
        }

        agents.push(LocalLayer3Agent {
            directory_name: name.to_string(),
            directory_path: path,
            manifest,
        });
    }

    Ok(agents)
}

pub(super) fn parse_front_matter(content: &str) -> Result<(toml::Value, String)> {
    let parts: Vec<&str> = content.splitn(3, "+++").collect();
    if parts.len() < 3 {
        return Err(anyhow!(
            "invalid front matter format: missing +++ separator"
        ));
    }
    let front_matter = parts[1].trim();
    let body = if parts.len() > 2 {
        parts[2].to_string()
    } else {
        String::new()
    };
    let metadata: toml::Value =
        toml::from_str(front_matter).with_context(|| "failed to parse TOML front matter")?;
    Ok((metadata, body))
}

pub(super) fn collect_text_like_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if path.file_name() == Some(OsStr::new(".git")) {
                    continue;
                }
                stack.push(path);
                continue;
            }

            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_default();
            if matches!(
                ext.as_str(),
                "md" | "txt" | "toml" | "py" | "yaml" | "yml" | "json"
            ) {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub(super) fn collect_preflight_findings(
    content: &str,
    file: &Path,
    findings: &mut Vec<PreflightFinding>,
) {
    let check_patterns: [(&str, super::FindingSeverity, &[&str]); 3] = [
        (
            "poisoning",
            super::FindingSeverity::High,
            &[
                "ignore previous instructions",
                "override system prompt",
                "hidden instruction",
                "prompt injection",
                "backdoor",
            ],
        ),
        (
            "bias",
            super::FindingSeverity::Medium,
            &[
                "always prefer",
                "must use our platform",
                "sponsored",
                "affiliate",
                "always prefer some platform",
            ],
        ),
        (
            "out_of_scope",
            super::FindingSeverity::Critical,
            &[
                "xmrig",
                "cryptominer",
                "mining pool",
                "sqlmap",
                "metasploit",
                "nmap -s",
                "collect personal information",
                "scrape personal information",
                "penetration exploit",
                "crypto mining",
            ],
        ),
    ];
    let repl_injection_patterns: &[&str] = &[
        "exec(pasted_",
        "eval(pasted_",
        "exec(terminal_",
        "eval(terminal_",
        "compile(pasted_",
        "os.system(pasted_",
        "subprocess.run(pasted_",
        "__import__(pasted_",
        "open(pasted_",
        "open(terminal_",
    ];

    for (category, severity, patterns) in check_patterns {
        for pattern in patterns {
            if content.contains(pattern) {
                findings.push(PreflightFinding {
                    category: category.to_string(),
                    severity,
                    evidence: format!("{} matched pattern: {}", file.display(), pattern),
                });
            }
        }
    }

    for pattern in repl_injection_patterns {
        if content.contains(pattern) {
            findings.push(PreflightFinding {
                category: "repl_variable_injection".to_string(),
                severity: super::FindingSeverity::Critical,
                evidence: format!(
                    "{} matched REPL injection pattern: {}",
                    file.display(),
                    pattern
                ),
            });
        }
    }
}

pub(super) fn highest_risk_level(findings: &[PreflightFinding]) -> &'static str {
    super::FindingSeverity::risk_level(findings)
}

pub(super) fn decide_preflight(findings: &[PreflightFinding]) -> PreflightDecision {
    if findings
        .iter()
        .any(|item| item.severity >= super::FindingSeverity::High)
    {
        return PreflightDecision::Block;
    }
    if findings
        .iter()
        .any(|item| item.severity == super::FindingSeverity::Medium)
    {
        return PreflightDecision::Review;
    }
    PreflightDecision::Allow
}

pub(super) fn run_preflight_audit(agent: &str, agent_root: &Path) -> Result<PreflightAuditReport> {
    let mut findings = Vec::new();
    let files = collect_text_like_files(agent_root)?;

    for file in files {
        let raw = fs::read_to_string(&file).unwrap_or_else(|e| {
            warn!(path = %file.display(), error = %e, "failed to read file during agent audit");
            String::new()
        });
        if raw.is_empty() {
            continue;
        }
        let lowered = raw.to_ascii_lowercase();
        collect_preflight_findings(&lowered, &file, &mut findings);
    }

    let decision = decide_preflight(&findings);
    let risk_level = highest_risk_level(&findings).to_string();
    let summary = if findings.is_empty() {
        "no risk signals detected".to_string()
    } else {
        format!(
            "found {} risk signals, highest risk: {}",
            findings.len(),
            risk_level
        )
    };

    Ok(PreflightAuditReport {
        agent: agent.to_string(),
        decision,
        risk_level,
        summary,
        findings,
    })
}

const ALLOWED_PERMISSIONS: &[&str] = &[
    "file_read",
    "file_write",
    "file_list",
    "http_get",
    "http_post",
    "memory_store",
    "memory_query",
    "llm_chat",
    "script_exec",
    "container_list",
    "container_exec",
    "web_search",
    "web_fetch",
    "report",
    "todo_read",
    "todo_write",
    "task_schedule",
];

pub fn check_permissions(permissions: &[String], findings: &mut Vec<PreflightFinding>) {
    for perm in permissions {
        if !ALLOWED_PERMISSIONS.contains(&perm.as_str()) {
            findings.push(PreflightFinding {
                category: "permission_violation".to_string(),
                severity: FindingSeverity::High,
                evidence: format!("unknown permission requested: {}", perm),
            });
        }
    }

    let has = |p: &str| permissions.iter().any(|x| x == p);

    if has("file_write") && has("container_exec") {
        findings.push(PreflightFinding {
            category: "permission_violation".to_string(),
            severity: FindingSeverity::Medium,
            evidence:
                "dangerous combination: file_write + container_exec (could modify and execute)"
                    .to_string(),
        });
    }
    if has("script_exec") && has("http_post") && has("file_read") {
        findings.push(PreflightFinding {
            category: "permission_violation".to_string(),
            severity: FindingSeverity::Medium,
            evidence:
                "dangerous combination: script_exec + http_post + file_read (could exfiltrate data)"
                    .to_string(),
        });
    }
    if has("container_exec") && has("file_write") && has("memory_store") {
        findings.push(PreflightFinding {
            category: "permission_violation".to_string(),
            severity: FindingSeverity::High,
            evidence: "dangerous combination: container_exec + file_write + memory_store (full system access)"
                .to_string(),
        });
    }
}

const ALLOWED_MOUNT_PREFIXES: &[&str] = &["./workspace/", "./data/", "./output/", "./tmp/"];
const FORBIDDEN_PATH_PREFIXES: &[&str] = &["/etc/", "/home/", "/root/", "/var/"];

pub fn check_mount_paths(manifest: &Layer3AgentManifest, findings: &mut Vec<PreflightFinding>) {
    let mounts = match manifest.skills.get("mounts") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>(),
        Some(toml::Value::Table(tbl)) => tbl
            .values()
            .filter_map(|v| v.get("path").and_then(|p| p.as_str()).map(String::from))
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    for path in &mounts {
        if path.starts_with('/') {
            let is_critical = FORBIDDEN_PATH_PREFIXES.iter().any(|p| path.starts_with(p));
            findings.push(PreflightFinding {
                category: "mount_path_violation".to_string(),
                severity: if is_critical {
                    FindingSeverity::Critical
                } else {
                    FindingSeverity::High
                },
                evidence: format!("absolute mount path: {}", path),
            });
            continue;
        }
        if path.starts_with(PATH_TRAVERSAL_PREFIX) {
            findings.push(PreflightFinding {
                category: "mount_path_violation".to_string(),
                severity: FindingSeverity::Critical,
                evidence: format!("path traversal mount: {}", path),
            });
            continue;
        }
        let matches_allowed = ALLOWED_MOUNT_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix));
        if !matches_allowed {
            findings.push(PreflightFinding {
                category: "mount_path_violation".to_string(),
                severity: FindingSeverity::High,
                evidence: format!("mount path not in allowed prefixes: {}", path),
            });
        }
    }
}

const CONTAINER_SECURITY_PATTERNS: &[(&str, &str, FindingSeverity)] = &[
    (
        "privileged: true",
        "privileged mode enabled",
        FindingSeverity::Critical,
    ),
    (
        "hostnetwork: true",
        "hostNetwork enabled",
        FindingSeverity::Critical,
    ),
    (
        "hostpid: true",
        "hostPID enabled",
        FindingSeverity::Critical,
    ),
    (
        "hostipc: true",
        "hostIPC enabled",
        FindingSeverity::Critical,
    ),
    (
        "capabilities: add: [\"all\"]",
        "ALL capabilities requested",
        FindingSeverity::Critical,
    ),
    ("sys_admin", "SYS_ADMIN capability", FindingSeverity::High),
    ("net_admin", "NET_ADMIN capability", FindingSeverity::High),
    ("sys_ptrace", "SYS_PTRACE capability", FindingSeverity::High),
    (
        "mountpropagation: bidirectional",
        "Bidirectional mount propagation",
        FindingSeverity::High,
    ),
];

pub fn check_container_security(agent_root: &Path, findings: &mut Vec<PreflightFinding>) {
    let files = match collect_config_like_files(agent_root) {
        Ok(f) => f,
        Err(_) => return,
    };

    for file in files {
        let raw = match fs::read_to_string(&file) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let lowered = raw.to_ascii_lowercase();

        for (pattern, description, severity) in CONTAINER_SECURITY_PATTERNS {
            if lowered.contains(pattern) {
                findings.push(PreflightFinding {
                    category: "container_security".to_string(),
                    severity: *severity,
                    evidence: format!("{} in {}", description, file.display()),
                });
            }
        }

        if !lowered.contains(DOCKER_USER_DIRECTIVE) {
            findings.push(PreflightFinding {
                category: "container_security".to_string(),
                severity: FindingSeverity::Medium,
                evidence: format!("no user directive (runs as root): {}", file.display()),
            });
        } else if lowered.contains(DOCKER_USER_ROOT) || lowered.contains(DOCKER_USER_ROOT_QUOTED) {
            findings.push(PreflightFinding {
                category: "container_security".to_string(),
                severity: FindingSeverity::High,
                evidence: format!("container runs as root: {}", file.display()),
            });
        }
    }
}

fn collect_config_like_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if path.file_name() == Some(OsStr::new(".git")) {
                    continue;
                }
                stack.push(path);
                continue;
            }

            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_ascii_lowercase())
                .unwrap_or_default();

            if fname == FILE_AGENT_TOML {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| v.to_ascii_lowercase())
                .unwrap_or_default();

            let is_dockerfile =
                fname.starts_with(DOCKERFILE_PREFIX) || fname.starts_with(CONTAINER_PREFIX);
            let is_config_ext = matches!(ext.as_str(), "yaml" | "yml" | "toml" | "json");

            if is_dockerfile || is_config_ext {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub fn run_preflight_audit_with_permissions(
    agent: &str,
    agent_root: &Path,
    permissions: &[String],
) -> Result<PreflightAuditReport> {
    let mut report = run_preflight_audit(agent, agent_root)?;

    check_permissions(permissions, &mut report.findings);

    if let Ok(manifest) = load_manifest_from_dir(agent_root) {
        check_mount_paths(&manifest, &mut report.findings);
    }

    check_container_security(agent_root, &mut report.findings);

    report.decision = decide_preflight(&report.findings);
    report.risk_level = highest_risk_level(&report.findings).to_string();
    report.summary = if report.findings.is_empty() {
        "no risk signals detected".to_string()
    } else {
        format!(
            "found {} risk signals, highest risk: {}",
            report.findings.len(),
            report.risk_level
        )
    };

    Ok(report)
}
