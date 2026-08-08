use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use tracing::warn;

use super::{
    LocalLayer3Agent, PreflightAuditReport, PreflightDecision,
    parser::{
        collect_preflight_findings, collect_text_like_files, decide_preflight, highest_risk_level,
        load_local_agents, load_toml_value, merge_toml,
    },
    subscription::{
        SubscribeConfig, SubscriptionEntry, SubscriptionSource, SubscriptionSyncReport, SyncAction,
        clone_repository, find_agent_root, install_subscription_agent, parse_check_interval,
        pubkey_for, subscription_owner, subscription_repo_url, validate_subscription_entry,
        verify_agent_package,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct SubscribeState {
    #[serde(default)]
    entries: Vec<SubscriptionStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionStateEntry {
    name: String,
    last_synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct PreflightExecutionState {
    #[serde(default)]
    audited_agents: Vec<String>,
}

impl SubscribeState {
    pub(super) fn last_synced_at(&self, name: &str) -> Option<DateTime<Utc>> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .and_then(|entry| DateTime::parse_from_rfc3339(&entry.last_synced_at).ok())
            .map(|value| value.with_timezone(&Utc))
    }

    pub(super) fn touch(&mut self, name: &str, timestamp: DateTime<Utc>) {
        let rendered = timestamp.to_rfc3339();
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.name == name) {
            entry.last_synced_at = rendered;
            return;
        }

        self.entries.push(SubscriptionStateEntry {
            name: name.to_string(),
            last_synced_at: rendered,
        });
    }
}

pub struct Layer3Workspace {
    pub root_dir: PathBuf,
    pub amphoreus_dir: PathBuf,
    pub project_config: toml::Value,
    pub personal_config: Option<toml::Value>,
    pub merged_config: toml::Value,
    pub subscriptions: SubscribeConfig,
    pub local_agents: Vec<LocalLayer3Agent>,
}

impl Layer3Workspace {
    pub fn discover_root(start: &Path) -> Option<PathBuf> {
        for dir in start.ancestors() {
            if dir.join(".amphoreus").is_dir() {
                return Some(dir.to_path_buf());
            }
        }
        None
    }

    pub fn load_from(start: impl AsRef<Path>) -> Result<Self> {
        let start = start.as_ref();
        let root_dir = Self::discover_root(start).ok_or_else(|| {
            anyhow!(
                "cannot find project root with .amphoreus upward from {}",
                start.display()
            )
        })?;

        let amphoreus_dir = root_dir.join(".amphoreus");
        let project_config_path = amphoreus_dir.join("config.toml");
        let personal_config_path = amphoreus_dir.join("config.self.toml");
        let subscribe_path = amphoreus_dir.join("subscribe.toml");

        let project_config = load_toml_value(&project_config_path).with_context(|| {
            format!(
                "failed to load project config: {}",
                project_config_path.display()
            )
        })?;

        let personal_config = if personal_config_path.exists() {
            Some(load_toml_value(&personal_config_path).with_context(|| {
                format!(
                    "failed to load personal config: {}",
                    personal_config_path.display()
                )
            })?)
        } else {
            None
        };

        let merged_config = merge_toml(
            project_config.clone(),
            personal_config
                .clone()
                .unwrap_or_else(|| toml::Value::Table(toml::map::Map::new())),
        );

        let subscriptions = if subscribe_path.exists() {
            let raw = std::fs::read_to_string(&subscribe_path).with_context(|| {
                format!(
                    "failed to read subscription file: {}",
                    subscribe_path.display()
                )
            })?;
            toml::from_str::<SubscribeConfig>(&raw).with_context(|| {
                format!(
                    "failed to parse subscription file: {}",
                    subscribe_path.display()
                )
            })?
        } else {
            SubscribeConfig::default()
        };

        let local_agents = load_local_agents(&amphoreus_dir)?;

        Ok(Self {
            root_dir,
            amphoreus_dir,
            project_config,
            personal_config,
            merged_config,
            subscriptions,
            local_agents,
        })
    }

    pub fn agent_script_path(&self, agent: &str) -> Option<PathBuf> {
        self.local_agents.iter().find_map(|entry| {
            let id_match = entry.manifest.agent.id == agent;
            let dir_match = entry.directory_name == agent;
            if id_match || dir_match {
                let script = entry.directory_path.join("run.py");
                if script.exists() {
                    return Some(script);
                }
            }
            None
        })
    }

    fn find_local_agent(&self, agent: &str) -> Option<&LocalLayer3Agent> {
        self.local_agents
            .iter()
            .find(|entry| entry.directory_name == agent || entry.manifest.agent.id == agent)
    }

    pub fn preflight_audit_local_agent(&self, agent: &str) -> Result<PreflightAuditReport> {
        let entry = self
            .find_local_agent(agent)
            .ok_or_else(|| anyhow!("Layer3 Agent not found for audit: {}", agent))?;
        self.audit_layer3_agent_path(&entry.directory_name, &entry.directory_path)
    }

    pub fn should_run_first_execution_audit(&self, agent: &str) -> Result<bool> {
        let entry = self
            .find_local_agent(agent)
            .ok_or_else(|| anyhow!("Layer3 Agent not found: {}", agent))?;
        let state = self.load_preflight_execution_state()?;
        Ok(!state
            .audited_agents
            .iter()
            .any(|value| value == &entry.directory_name))
    }

    pub fn mark_first_execution_audited(&self, agent: &str) -> Result<()> {
        let entry = self
            .find_local_agent(agent)
            .ok_or_else(|| anyhow!("Layer3 Agent not found: {}", agent))?;
        let mut state = self.load_preflight_execution_state()?;
        if !state
            .audited_agents
            .iter()
            .any(|value| value == &entry.directory_name)
        {
            state.audited_agents.push(entry.directory_name.clone());
        }
        self.save_preflight_execution_state(&state)
    }

    pub fn subscribe_path(&self) -> PathBuf {
        self.amphoreus_dir.join("subscribe.toml")
    }

    pub fn save_subscriptions(&self) -> Result<()> {
        let rendered = toml::to_string_pretty(&self.subscriptions)
            .context("failed to serialize subscribe.toml")?;
        fs::write(self.subscribe_path(), rendered).context("failed to write subscribe.toml")?;
        Ok(())
    }

    pub fn upsert_subscription(&mut self, entry: SubscriptionEntry) -> Result<()> {
        validate_subscription_entry(&entry)?;
        if let Some(existing) = self
            .subscriptions
            .subscriptions
            .iter_mut()
            .find(|existing| existing.name == entry.name)
        {
            *existing = entry;
        } else {
            self.subscriptions.subscriptions.push(entry);
        }
        self.save_subscriptions()
    }

    pub fn remove_subscription(&mut self, name: &str) -> Result<bool> {
        let original_len = self.subscriptions.subscriptions.len();
        self.subscriptions
            .subscriptions
            .retain(|entry| entry.name != name);
        let changed = self.subscriptions.subscriptions.len() != original_len;
        if changed {
            self.save_subscriptions()?;
        }
        Ok(changed)
    }

    pub async fn sync_subscriptions(
        &mut self,
        filter_name: Option<&str>,
    ) -> Result<Vec<SubscriptionSyncReport>> {
        let mut state = self.load_subscribe_state()?;
        let selected = self
            .subscriptions
            .subscriptions
            .iter()
            .filter(|entry| match filter_name {
                Some(name) => entry.name == name,
                None => true,
            })
            .cloned()
            .collect::<Vec<_>>();

        if selected.is_empty()
            && let Some(name) = filter_name
        {
            return Err(anyhow!("subscription not found: {}", name));
        }

        let mut reports = Vec::new();
        for entry in selected {
            if !entry.enabled {
                reports.push(SubscriptionSyncReport {
                    name: entry.name.clone(),
                    action: SyncAction::Skipped,
                    detail: "subscription disabled".to_string(),
                });
                continue;
            }
            let report = self.sync_subscription(&entry).await?;
            state.touch(&entry.name, Utc::now());
            reports.push(report);
        }

        self.save_subscribe_state(&state)?;
        self.local_agents = load_local_agents(&self.amphoreus_dir)?;
        Ok(reports)
    }

    pub async fn auto_update_subscriptions(&mut self) -> Result<Vec<SubscriptionSyncReport>> {
        let now = Utc::now();
        let interval = parse_check_interval(&self.subscriptions.settings.check_interval)?;
        let mut state = self.load_subscribe_state()?;
        let mut reports = Vec::new();

        for entry in self.subscriptions.subscriptions.clone() {
            if !entry.enabled {
                reports.push(SubscriptionSyncReport {
                    name: entry.name,
                    action: SyncAction::Skipped,
                    detail: "subscription disabled".to_string(),
                });
                continue;
            }

            if !(self.subscriptions.settings.auto_update || entry.auto_update) {
                reports.push(SubscriptionSyncReport {
                    name: entry.name,
                    action: SyncAction::Skipped,
                    detail: "auto-update not enabled".to_string(),
                });
                continue;
            }

            if let Some(last_synced_at) = state.last_synced_at(&entry.name)
                && now - last_synced_at < interval
            {
                reports.push(SubscriptionSyncReport {
                    name: entry.name,
                    action: SyncAction::Skipped,
                    detail: format!(
                        "not yet time to update, last synced at: {}",
                        last_synced_at.to_rfc3339()
                    ),
                });
                continue;
            }

            let report = self.sync_subscription(&entry).await?;
            state.touch(&entry.name, now);
            reports.push(report);
        }

        self.save_subscribe_state(&state)?;
        self.local_agents = load_local_agents(&self.amphoreus_dir)?;
        Ok(reports)
    }

    async fn sync_subscription(&self, entry: &SubscriptionEntry) -> Result<SubscriptionSyncReport> {
        match entry.source {
            SubscriptionSource::Official => self.sync_official_subscription(entry),
            SubscriptionSource::Github | SubscriptionSource::Url => {
                self.sync_git_subscription(entry).await
            }
        }
    }

    fn sync_official_subscription(
        &self,
        entry: &SubscriptionEntry,
    ) -> Result<SubscriptionSyncReport> {
        let installed = self.local_agents.iter().any(|agent| {
            agent.directory_name == entry.name || agent.manifest.agent.id == entry.name
        });
        if installed {
            return Ok(SubscriptionSyncReport {
                name: entry.name.clone(),
                action: SyncAction::Verified,
                detail: "official agent already exists in local .amphoreus directory".to_string(),
            });
        }

        Err(anyhow!(
            "official subscription '{}' not found locally, please place the agent in .amphoreus/{}",
            entry.name,
            entry.name
        ))
    }

    async fn sync_git_subscription(
        &self,
        entry: &SubscriptionEntry,
    ) -> Result<SubscriptionSyncReport> {
        // Same tiered policy as subscribe_agent (PLAN §11.3): trusted-source
        // allow-list plus package signature verification on every sync —
        // including auto-updates, which must not bypass the checks.
        let settings = &self.subscriptions.settings;
        let owner = subscription_owner(
            entry
                .repository
                .as_deref()
                .or(entry.url.as_deref())
                .unwrap_or(""),
        );
        if !settings.trusted_sources.iter().any(|s| s == &owner) {
            return Err(anyhow!(
                "subscription '{}': source `{}` is not in trusted_sources: {:?}",
                entry.name,
                owner,
                settings.trusted_sources
            ));
        }

        let repo_url = subscription_repo_url(entry)?;
        let tmp_dir = std::env::temp_dir().join(format!(
            "entelecheia-layer3-subscription-{}",
            Uuid::now_v7()
        ));

        clone_repository(&repo_url, &tmp_dir).await?;
        let agent_root = find_agent_root(&tmp_dir, &entry.name)?;

        if settings.verify_signature {
            let pubkey = pubkey_for(&owner).ok_or_else(|| {
                anyhow!(
                    "subscription '{}': no registered signing key for trusted source `{}`; cannot verify agent",
                    entry.name,
                    owner
                )
            })?;
            verify_agent_package(&agent_root, &pubkey).with_context(|| {
                format!(
                    "subscription '{}': package signature verification failed for source `{}`",
                    entry.name, owner
                )
            })?;
        }

        let preflight = self.audit_layer3_agent_path(&entry.name, &agent_root)?;
        if preflight.decision != PreflightDecision::Allow {
            let _ = fs::remove_dir_all(&tmp_dir);
            return Err(anyhow!(
                "subscription '{}' preflight audit failed: {} (decision={})",
                entry.name,
                preflight.summary,
                preflight.decision.as_str()
            ));
        }

        let target_dir = self.amphoreus_dir.join(&entry.name);
        let action = install_subscription_agent(&agent_root, &target_dir, entry)?;
        let _ = fs::remove_dir_all(&tmp_dir);

        Ok(SubscriptionSyncReport {
            name: entry.name.clone(),
            action,
            detail: format!("synced from {} to {}", repo_url, target_dir.display()),
        })
    }

    pub(super) fn audit_layer3_agent_path(
        &self,
        agent: &str,
        agent_root: &Path,
    ) -> Result<PreflightAuditReport> {
        let mut findings = Vec::new();
        let files = collect_text_like_files(agent_root)?;

        for file in files {
            let raw = fs::read_to_string(&file).unwrap_or_else(|e| {
                warn!(path = %file.display(), error = %e, "failed to read workspace file during audit");
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
            "no risk patterns detected".to_string()
        } else {
            format!(
                "found {} risk signals, highest risk level: {}",
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

    fn subscribe_state_path(&self) -> PathBuf {
        self.amphoreus_dir.join("subscribe.state.toml")
    }

    fn load_subscribe_state(&self) -> Result<SubscribeState> {
        let path = self.subscribe_state_path();
        if !path.exists() {
            return Ok(SubscribeState::default());
        }

        let raw = fs::read_to_string(&path).with_context(|| {
            format!("failed to read subscription state file: {}", path.display())
        })?;
        toml::from_str(&raw).with_context(|| {
            format!(
                "failed to parse subscription state file: {}",
                path.display()
            )
        })
    }

    fn save_subscribe_state(&self, state: &SubscribeState) -> Result<()> {
        if state.entries.is_empty() {
            let path = self.subscribe_state_path();
            if path.exists() {
                fs::remove_file(&path).with_context(|| {
                    format!(
                        "failed to delete empty subscription state file: {}",
                        path.display()
                    )
                })?;
            }
            return Ok(());
        }

        let rendered =
            toml::to_string_pretty(state).context("failed to serialize subscription state")?;
        fs::write(self.subscribe_state_path(), rendered)
            .context("failed to write subscription state")?;
        Ok(())
    }

    fn preflight_execution_state_path(&self) -> PathBuf {
        self.amphoreus_dir.join("preflight.state.toml")
    }

    fn load_preflight_execution_state(&self) -> Result<PreflightExecutionState> {
        let path = self.preflight_execution_state_path();
        if !path.exists() {
            return Ok(PreflightExecutionState::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read preflight state: {}", path.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("failed to parse preflight state: {}", path.display()))
    }

    fn save_preflight_execution_state(&self, state: &PreflightExecutionState) -> Result<()> {
        let path = self.preflight_execution_state_path();
        if state.audited_agents.is_empty() {
            if path.exists() {
                fs::remove_file(&path).with_context(|| {
                    format!("failed to delete empty preflight state: {}", path.display())
                })?;
            }
            return Ok(());
        }

        let rendered =
            toml::to_string_pretty(state).context("failed to serialize preflight state")?;
        fs::write(&path, rendered)
            .with_context(|| format!("failed to write preflight state: {}", path.display()))?;
        Ok(())
    }
}
