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
                    "url source requires url=\"https://...\" or git URL"
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

pub async fn clone_repository(repo_url: &str, dest: &Path) -> Result<()> {
    let status = tokio::process::Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(repo_url)
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
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
