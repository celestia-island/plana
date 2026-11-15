use anyhow::{Context, Result, anyhow};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use tracing::warn;

use super::{super::parser::load_manifest_from_dir, SubscriptionEntry, SubscriptionSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InstalledSubscriptionMetadata {
    pub name: String,
    pub source: SubscriptionSource,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    Installed,
    Updated,
    Removed,
    Skipped,
    Verified,
}

impl SyncAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Updated => "updated",
            Self::Removed => "removed",
            Self::Skipped => "skipped",
            Self::Verified => "verified",
        }
    }
}

impl std::fmt::Display for SyncAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionSyncReport {
    pub name: String,
    pub action: SyncAction,
    pub detail: String,
}

pub fn parse_check_interval(value: &str) -> Result<Duration> {
    match value.trim().to_ascii_lowercase().as_str() {
        "hourly" => Ok(Duration::hours(1)),
        "daily" => Ok(Duration::days(1)),
        "weekly" => Ok(Duration::weeks(1)),
        "monthly" => Ok(Duration::days(30)),
        other => Err(anyhow!(
            "unsupported subscription check interval '{}', only hourly/daily/weekly/monthly are supported",
            other
        )),
    }
}

pub fn validate_subscription_entry(entry: &SubscriptionEntry) -> Result<()> {
    if entry.name.trim().is_empty() {
        return Err(anyhow!("subscription name must not be empty"));
    }

    match entry.source {
        SubscriptionSource::Official => Ok(()),
        SubscriptionSource::Github => {
            if entry
                .repository
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(anyhow!("github source requires repository=\"owner/repo\""));
            }
            Ok(())
        }
        SubscriptionSource::Url => {
            if entry.url.as_deref().unwrap_or_default().trim().is_empty() {
                return Err(anyhow!(
                    "url source requires an https:// git URL in url=\"...\""
                ));
            }
            Ok(())
        }
    }
}

pub fn subscription_repo_url(entry: &SubscriptionEntry) -> Result<String> {
    match entry.source {
        SubscriptionSource::Official => Err(anyhow!(
            "official source does not require remote repository URL"
        )),
        SubscriptionSource::Github => {
            let repo = entry
                .repository
                .as_deref()
                .ok_or_else(|| anyhow!("github source missing repository"))?;
            Ok(format!("https://github.com/{}.git", repo))
        }
        SubscriptionSource::Url => entry
            .url
            .clone()
            .ok_or_else(|| anyhow!("url source missing url field")),
    }
}

/// Transport allow-list for subscription clones.
///
/// Only `https` is accepted. `file://`, `ssh://`, `git://` and the
/// `<transport>::<address>` helper syntax (`ext::sh -c …`) are rejected: a
/// subscription URL is attacker-controlled data, and `git` treats a
/// URL-shaped argv element that starts with `-` as an *option* — which is how
/// `--upload-pack=<cmd>` turns a clone into command execution. No local
/// `file://` development flow exists for subscriptions (nothing in the repo,
/// docs or tests references one), so the whitelist stays narrow.
const ALLOWED_CLONE_SCHEMES: &[&str] = &["https"];

/// Reject subscription repository URLs that cannot be safely handed to `git`.
///
/// This is the URL-level half of the clone hardening; the `--` separator in
/// [`clone_command`] is the argv-level half, so an option-shaped URL is
/// stopped twice.
fn validate_clone_url(repo_url: &str) -> Result<()> {
    let url = repo_url.trim();
    if url.is_empty() {
        return Err(anyhow!("subscription repository URL must not be empty"));
    }
    // `git` parses any argv element starting with `-` as an option, so a URL
    // such as `--upload-pack=sh -c …` would be executed as a flag.
    if url.starts_with('-') {
        return Err(anyhow!(
            "subscription repository URL must not start with '-': {}",
            url
        ));
    }
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(anyhow!(
            "subscription repository URL must not contain whitespace or control characters: {}",
            url
        ));
    }
    let Some((scheme, rest)) = url.split_once("://") else {
        return Err(anyhow!(
            "unsupported subscription repository URL '{}': only {}:// is allowed",
            url,
            ALLOWED_CLONE_SCHEMES.join(" / ")
        ));
    };
    let scheme = scheme.to_ascii_lowercase();
    if !ALLOWED_CLONE_SCHEMES.contains(&scheme.as_str()) {
        return Err(anyhow!(
            "unsupported subscription repository URL scheme '{}': only {}:// is allowed",
            scheme,
            ALLOWED_CLONE_SCHEMES.join(" / ")
        ));
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if host.is_empty() || host == "." || host == ".." {
        return Err(anyhow!(
            "unsupported subscription repository URL '{}': missing host",
            url
        ));
    }
    Ok(())
}

/// Build the `git clone` invocation for one subscription.
///
/// `--` terminates option parsing, so both the repository URL and the
/// destination are treated as positional arguments. The caller validates the
/// URL first ([`validate_clone_url`]); keeping the argv construction in one
/// place is what makes the argument order testable without running `git`.
fn clone_command(repo_url: &str, dest: &Path) -> tokio::process::Command {
    let mut command = tokio::process::Command::new("git");
    command
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--")
        .arg(repo_url)
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit());
    command
}

/// Clone a subscription repository into a temporary directory.
///
/// Note: callers verify the package signature *after* this call (the trusted
/// source allow-list and this URL whitelist run before it), so a hostile
/// source can still cause a clone attempt. Moving signature verification
/// ahead of the clone is a structural change and is deliberately out of scope
/// for this hardening; the URL whitelist plus `--` close the `--upload-pack`
/// command-execution path in the meantime.
pub async fn clone_repository(repo_url: &str, dest: &Path) -> Result<()> {
    validate_clone_url(repo_url)?;

    let status = clone_command(repo_url, dest)
        .status()
        .await
        .with_context(|| format!("git clone execution failed: {}", repo_url))?;

    if !status.success() {
        return Err(anyhow!("git clone failed: {}", repo_url));
    }

    if let Err(e) = tokio::task::spawn_blocking({
        let dest = dest.to_path_buf();
        move || libnoa::repo::Repository::init(&dest)
    })
    .await
    {
        warn!(error = %e, "noa init after agent subscription clone failed (non-fatal)");
    }

    Ok(())
}

pub fn find_agent_root(root: &Path, subscription_name: &str) -> Result<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    let mut candidates = Vec::new();

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
                if path.join("agent.toml").exists() {
                    candidates.push(path.clone());
                }
                stack.push(path);
            }
        }
    }

    if root.join("agent.toml").exists() {
        candidates.push(root.to_path_buf());
    }

    candidates.sort();
    candidates.dedup();

    if let Some(candidate) = candidates.iter().find(|candidate| {
        candidate.file_name().and_then(|value| value.to_str()) == Some(subscription_name)
    }) {
        return Ok(candidate.clone());
    }

    if let Some(candidate) = candidates.iter().find(|candidate| {
        load_manifest_from_dir(candidate)
            .map(|manifest| manifest.agent.id == subscription_name)
            .unwrap_or(false)
    }) {
        return Ok(candidate.clone());
    }

    match candidates.as_slice() {
        [only] => Ok(only.clone()),
        [] => Err(anyhow!(
            "no Layer3 Agent with agent.toml found in {}",
            root.display()
        )),
        _ => Err(anyhow!(
            "multiple candidate agents found in {}, please organize repo as single-agent structure or use subscription name matching directory/agent.id",
            root.display()
        )),
    }
}

pub fn install_subscription_agent(
    source_dir: &Path,
    target_dir: &Path,
    entry: &SubscriptionEntry,
) -> Result<SyncAction> {
    let action = if target_dir.exists() {
        if target_dir.join(".subscription.toml").exists() {
            fs::remove_dir_all(target_dir).with_context(|| {
                format!(
                    "failed to clean old subscription directory: {}",
                    target_dir.display()
                )
            })?;
            SyncAction::Updated
        } else {
            return Err(anyhow!(
                "target directory {} already exists and is not a subscription-managed directory, refused to overwrite",
                target_dir.display()
            ));
        }
    } else {
        SyncAction::Installed
    };

    copy_dir_recursive(source_dir, target_dir)?;
    let metadata = InstalledSubscriptionMetadata {
        name: entry.name.clone(),
        source: entry.source.clone(),
        repository: entry.repository.clone(),
        url: entry.url.clone(),
        version: entry.version.clone(),
    };
    let rendered =
        toml::to_string_pretty(&metadata).context("failed to serialize subscription metadata")?;
    fs::write(target_dir.join(".subscription.toml"), rendered).with_context(|| {
        format!(
            "failed to write subscription metadata: {}",
            target_dir.display()
        )
    })?;

    Ok(action)
}

pub fn copy_dir_recursive(source_dir: &Path, target_dir: &Path) -> Result<()> {
    fs::create_dir_all(target_dir)
        .with_context(|| format!("failed to create directory: {}", target_dir.display()))?;

    for entry in fs::read_dir(source_dir)
        .with_context(|| format!("failed to read directory: {}", source_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == OsStr::new(".git") {
            continue;
        }

        let target = target_dir.join(&name);
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target).with_context(|| {
                format!(
                    "failed to copy file: {} -> {}",
                    path.display(),
                    target.display()
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(repo_url: &str, dest: &Path) -> Vec<String> {
        clone_command(repo_url, dest)
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn clone_command_terminates_options_before_url() -> Result<()> {
        let dest = Path::new("/tmp/entelecheia-sub-test");
        let args = argv("https://github.com/owner/repo.git", dest);
        assert_eq!(
            args,
            vec![
                "clone",
                "--depth",
                "1",
                "--",
                "https://github.com/owner/repo.git",
                "/tmp/entelecheia-sub-test",
            ]
        );
        Ok(())
    }

    #[test]
    fn clone_url_rejects_option_shaped_urls() {
        // Without `--`, git parses the URL as an option; `--upload-pack=<cmd>`
        // is the command-execution shape.
        assert!(validate_clone_url("--upload-pack=touch /tmp/pwn").is_err());
        assert!(validate_clone_url("--upload-pack=touch").is_err());
        assert!(validate_clone_url("-u/tmp/pwn").is_err());
    }

    #[test]
    fn clone_url_rejects_transport_helpers_and_other_schemes() {
        for url in [
            "ext::sh -c touch% /tmp/pwn",
            "ext::sh",
            "file:///tmp/repo",
            "http://github.com/owner/repo.git",
            "git://github.com/owner/repo.git",
            "ssh://git@github.com/owner/repo.git",
            "git@github.com:owner/repo.git",
            "github.com/owner/repo.git",
            "/tmp/repo",
            "",
            "   ",
            "https://",
            "https:///owner/repo.git",
        ] {
            assert!(
                validate_clone_url(url).is_err(),
                "expected rejection for {url:?}"
            );
        }
    }

    #[test]
    fn clone_url_accepts_https() -> Result<()> {
        validate_clone_url("https://github.com/celestia-island/plana-agents.git")?;
        validate_clone_url("HTTPS://github.com/owner/repo.git")?;
        validate_clone_url("https://git.example.dev:8443/owner/repo.git")?;
        Ok(())
    }

    #[test]
    fn missing_verify_signature_key_defaults_to_true() -> Result<()> {
        // A settings table that predates the field: the key is absent, so a
        // plain `#[serde(default)]` would have yielded `false`.
        let config: crate::SubscribeConfig = toml::from_str(
            r#"
            [settings]
            auto_update = true
            "#,
        )?;
        assert!(config.settings.verify_signature);

        // No settings table at all keeps the `Default` value too.
        let config: crate::SubscribeConfig = toml::from_str("")?;
        assert!(config.settings.verify_signature);
        Ok(())
    }

    #[test]
    fn explicit_verify_signature_false_is_honoured() -> Result<()> {
        let config: crate::SubscribeConfig = toml::from_str(
            r#"
            [settings]
            verify_signature = false
            "#,
        )?;
        assert!(!config.settings.verify_signature);
        Ok(())
    }

    #[tokio::test]
    async fn clone_repository_rejects_unsafe_url_before_spawning_git() {
        let dest = std::env::temp_dir().join("entelecheia-clone-url-test");
        let err = clone_repository("--upload-pack=touch /tmp/pwn", &dest)
            .await
            .expect_err("unsafe clone URL must be rejected");
        assert!(err.to_string().contains("must not start with '-'"));
    }
}
