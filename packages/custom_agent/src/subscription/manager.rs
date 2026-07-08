use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use tracing::warn;

use super::{
    super::{
        Layer3Workspace, LocalLayer3Agent, PreflightDecision, SkillInfo,
        parser::{
            load_local_agents, load_manifest_from_dir, parse_front_matter, run_preflight_audit,
        },
    },
    SubscriptionEntry, SubscriptionSource,
    events::{clone_repository, copy_dir_recursive, find_agent_root},
};
use _state_sync::gateway::tui_types::layer2::CustomAgentInfo;

pub struct CustomAgentManager;

impl CustomAgentManager {
    pub fn custom_agents_root() -> PathBuf {
        _config::UserConfig::custom_agents_dir()
    }

    pub fn git_dir() -> PathBuf {
        Self::custom_agents_root().join("git")
    }

    pub fn subscriptions_path() -> PathBuf {
        Self::custom_agents_root().join("subscriptions.toml")
    }

    pub fn subscriptions_state_path() -> PathBuf {
        Self::custom_agents_root().join("subscriptions.state.toml")
    }

    pub fn ensure_dirs() -> Result<()> {
        let root = Self::custom_agents_root();
        fs::create_dir_all(&root).context("failed to create custom_agents root directory")?;
        fs::create_dir_all(Self::git_dir())
            .context("failed to create custom_agents/git directory")?;
        Ok(())
    }

    pub fn validate_agent_name(name: &str) -> Result<()> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
            || name.contains('\0')
        {
            return Err(anyhow!("invalid custom agent name: '{}'", name));
        }
        Ok(())
    }

    pub fn detect_workspace_layer3_agents() -> Vec<LocalLayer3Agent> {
        let amphoreus_root = Layer3Workspace::discover_root(
            std::env::current_dir().as_deref().unwrap_or(Path::new(".")),
        );
        match amphoreus_root {
            Some(root) => {
                let amphoreus_dir = root.join(".amphoreus");
                if amphoreus_dir.is_dir() {
                    load_local_agents(&amphoreus_dir).unwrap_or_default()
                } else {
                    Vec::new()
                }
            },
            None => Vec::new(),
        }
    }

    pub fn workspace_layer3_amphoreus_dir() -> Option<PathBuf> {
        Layer3Workspace::discover_root(std::env::current_dir().as_deref().unwrap_or(Path::new(".")))
            .map(|root| root.join(".amphoreus"))
    }

    pub fn load_subscriptions() -> Result<super::SubscribeConfig> {
        let path = Self::subscriptions_path();
        if !path.exists() {
            return Ok(super::SubscribeConfig::default());
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read subscription file: {}", path.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("failed to parse subscription file: {}", path.display()))
    }

    pub fn save_subscriptions(config: &super::SubscribeConfig) -> Result<()> {
        Self::ensure_dirs()?;
        let rendered =
            toml::to_string_pretty(config).context("failed to serialize subscription file")?;
        fs::write(Self::subscriptions_path(), rendered).context("failed to write subscription file")
    }

    pub fn load_local_agents() -> Result<Vec<LocalLayer3Agent>> {
        Self::ensure_dirs()?;
        let git_dir = Self::git_dir();
        let mut agents = load_local_agents(&git_dir)?;

        let amphoreus_dir = Layer3Workspace::discover_root(
            std::env::current_dir().as_deref().unwrap_or(Path::new(".")),
        );
        if let Some(amphoreus_dir) = amphoreus_dir {
            let local_amphoreus = amphoreus_dir.join(".amphoreus");
            if local_amphoreus.is_dir()
                && let Ok(local_agents) = load_local_agents(&local_amphoreus)
            {
                let mut existing_names: HashSet<String> =
                    agents.iter().map(|a| a.directory_name.clone()).collect();
                for agent in local_agents {
                    if !existing_names.contains(&agent.directory_name) {
                        existing_names.insert(agent.directory_name.clone());
                        agents.push(agent);
                    }
                }
            }
        }

        Ok(agents)
    }

    pub fn load_subscription_agents_only() -> Result<Vec<LocalLayer3Agent>> {
        Self::ensure_dirs()?;
        let git_dir = Self::git_dir();
        load_local_agents(&git_dir)
    }

    pub fn load_custom_agent_skills(agent_dir: &Path) -> Result<Vec<SkillInfo>> {
        let skills_dir = agent_dir.join("skills");
        if !skills_dir.exists() {
            return Ok(vec![]);
        }

        let mut infos = Vec::new();
        for entry in fs::read_dir(&skills_dir)
            .with_context(|| format!("failed to read skills directory: {}", skills_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read skill file: {}", path.display()))?;
            let (metadata, _) = parse_front_matter(&raw)?;

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            let description: std::collections::HashMap<String, String> = metadata
                .get("description")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        let mut m = std::collections::HashMap::new();
                        m.insert("en".to_string(), s.to_string());
                        Some(m)
                    } else if let Some(table) = v.as_table() {
                        let m: std::collections::HashMap<String, String> = table
                            .iter()
                            .filter_map(|(k, val)| Some((k.clone(), val.as_str()?.to_string())))
                            .collect();
                        if m.is_empty() { None } else { Some(m) }
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let required_tools: Vec<String> = metadata
                .get("required_tools")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            infos.push(SkillInfo {
                name,
                description,
                required_tools,
            });
        }

        infos.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(infos)
    }

    pub fn add_subscription(entry: SubscriptionEntry) -> Result<()> {
        let mut config = Self::load_subscriptions()?;
        if config.subscriptions.iter().any(|e| e.name == entry.name) {
            return Ok(());
        }
        config.subscriptions.push(entry);
        Self::save_subscriptions(&config)
    }

    pub fn remove_subscription(name: &str) -> Result<bool> {
        let mut config = Self::load_subscriptions()?;
        let original_len = config.subscriptions.len();
        config.subscriptions.retain(|e| e.name != name);
        if config.subscriptions.len() == original_len {
            return Ok(false);
        }
        Self::save_subscriptions(&config)?;
        Ok(true)
    }

    pub async fn subscribe_agent(
        source: &str,
        repository: Option<&str>,
        url: Option<&str>,
    ) -> Result<CustomAgentInfo> {
        let subscription_source = match source {
            "github" => SubscriptionSource::Github,
            "url" => SubscriptionSource::Url,
            _ => SubscriptionSource::Github,
        };

        let repo = match subscription_source {
            SubscriptionSource::Github => repository
                .ok_or_else(|| anyhow!("github source requires repository field"))?
                .trim()
                .to_string(),
            SubscriptionSource::Url => url
                .ok_or_else(|| anyhow!("url source requires url field"))?
                .trim()
                .to_string(),
            _ => return Err(anyhow!("unsupported source: {}", source)),
        };

        if repo.is_empty() {
            return Err(anyhow!("repository/url must not be empty"));
        }

        let agent_name = repo.split('/').next_back().unwrap_or("unknown").to_string();
        Self::validate_agent_name(&agent_name)?;
        let repo_url = match subscription_source {
            SubscriptionSource::Github => format!("https://github.com/{}.git", repo),
            SubscriptionSource::Url => repo.clone(),
            SubscriptionSource::Official => {
                return Err(anyhow!("official source not supported for subscription"));
            },
        };

        let tmp_dir =
            std::env::temp_dir().join(format!("entelecheia-custom-agent-sub-{}", Uuid::now_v7()));

        clone_repository(&repo_url, &tmp_dir).await?;

        let agent_root = find_agent_root(&tmp_dir, &agent_name)?;
        let manifest = load_manifest_from_dir(&agent_root)?;

        if manifest.agent.layer != 3 {
            if let Err(e) = fs::remove_dir_all(&tmp_dir) {
                warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
            }
            return Err(anyhow!(
                "agent.toml layer={} is not Layer 3",
                manifest.agent.layer
            ));
        }

        let preflight = run_preflight_audit(&agent_name, &agent_root)?;
        if preflight.decision != PreflightDecision::Allow {
            if let Err(e) = fs::remove_dir_all(&tmp_dir) {
                warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
            }
            return Err(anyhow!(
                "preflight audit blocked: {} (decision={})",
                preflight.summary,
                preflight.decision.as_str()
            ));
        }

        let target_dir = Self::git_dir().join(&agent_name);
        if target_dir.exists()
            && let Err(e) = fs::remove_dir_all(&target_dir)
        {
            warn!(path = %target_dir.display(), error = %e, "failed to clean up target directory");
        }
        copy_dir_recursive(&agent_root, &target_dir)?;
        if let Err(e) = fs::remove_dir_all(&tmp_dir) {
            warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
        }

        let entry = SubscriptionEntry {
            name: agent_name.clone(),
            source: subscription_source,
            repository: repository.map(|s| s.to_string()),
            url: url.map(|s| s.to_string()),
            version: Some(manifest.agent.version.clone()),
            enabled: true,
            auto_update: false,
            enabled_tools: None,
            enabled_skills: None,
            granted_permissions: None,
        };
        Self::add_subscription(entry)?;

        let skills_count = Self::load_custom_agent_skills(&target_dir)
            .map(|s| s.len())
            .unwrap_or(0);

        Ok(CustomAgentInfo {
            name: agent_name.clone(),
            display_name: manifest.agent.name.clone(),
            description: manifest.agent.description.clone().unwrap_or_default(),
            skills_count,
            source: repo_url.clone(),
            version: Some(manifest.agent.version.clone()),
            last_updated: Some(Utc::now().to_rfc3339()),
        })
    }

    pub async fn unsubscribe_agent(name: &str) -> Result<()> {
        Self::validate_agent_name(name)?;
        let agent_dir = Self::git_dir().join(name);
        if !agent_dir.exists() {
            return Err(anyhow!("custom agent '{}' does not exist", name));
        }

        Self::remove_subscription(name)?;
        fs::remove_dir_all(&agent_dir).with_context(|| {
            format!(
                "failed to delete custom agent directory: {}",
                agent_dir.display()
            )
        })?;
        Ok(())
    }

    pub async fn subscribe_agent_by_url(git_url: &str) -> Result<CustomAgentInfo> {
        let url = git_url.trim().to_string();
        if url.is_empty() {
            return Err(anyhow!("Git URL must not be empty"));
        }

        let agent_name = url
            .trim_end_matches(".git")
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .unwrap_or("unknown")
            .to_string();
        Self::validate_agent_name(&agent_name)?;

        let tmp_dir =
            std::env::temp_dir().join(format!("entelecheia-custom-agent-sub-{}", Uuid::now_v7()));

        clone_repository(&url, &tmp_dir).await?;

        let agent_root = find_agent_root(&tmp_dir, &agent_name)?;
        let manifest = load_manifest_from_dir(&agent_root)?;

        if manifest.agent.layer != 3 {
            if let Err(e) = fs::remove_dir_all(&tmp_dir) {
                warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
            }
            return Err(anyhow!(
                "agent.toml layer={} is not Layer 3",
                manifest.agent.layer
            ));
        }

        let preflight = run_preflight_audit(&agent_name, &agent_root)?;
        if preflight.decision != PreflightDecision::Allow {
            if let Err(e) = fs::remove_dir_all(&tmp_dir) {
                warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
            }
            return Err(anyhow!(
                "preflight audit blocked: {} (decision={})",
                preflight.summary,
                preflight.decision.as_str()
            ));
        }

        Self::ensure_dirs()?;
        let target_dir = Self::git_dir().join(&agent_name);
        if target_dir.exists()
            && let Err(e) = fs::remove_dir_all(&target_dir)
        {
            warn!(path = %target_dir.display(), error = %e, "failed to clean up target directory");
        }
        copy_dir_recursive(&agent_root, &target_dir)?;
        if let Err(e) = fs::remove_dir_all(&tmp_dir) {
            warn!(path = %tmp_dir.display(), error = %e, "failed to clean up temp directory");
        }

        let entry = SubscriptionEntry {
            name: agent_name.clone(),
            source: SubscriptionSource::Url,
            repository: None,
            url: Some(url.clone()),
            version: Some(manifest.agent.version.clone()),
            enabled: true,
            auto_update: false,
            enabled_tools: None,
            enabled_skills: None,
            granted_permissions: None,
        };
        Self::add_subscription(entry)?;

        let skills_count = Self::load_custom_agent_skills(&target_dir)
            .map(|s| s.len())
            .unwrap_or(0);

        Ok(CustomAgentInfo {
            name: agent_name.clone(),
            display_name: manifest.agent.name.clone(),
            description: manifest.agent.description.clone().unwrap_or_default(),
            skills_count,
            source: url.clone(),
            version: Some(manifest.agent.version.clone()),
            last_updated: Some(Utc::now().to_rfc3339()),
        })
    }
}
