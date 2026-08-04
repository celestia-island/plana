use anyhow::{Result, anyhow};
use std::{path::PathBuf, sync::Arc};

use tracing::warn;

#[derive(Debug, Clone)]
pub struct SoulContent {
    pub name: String,
    pub description: String,
    pub content: String,
}

pub trait ContentProvider: Send + Sync {
    fn load_soul(&self, agent: &str, lang: &str) -> Option<String>;
    fn load_system(&self, agent: &str, lang: &str) -> Option<String>;
}

static CONTENT_PROVIDER: std::sync::OnceLock<Arc<dyn ContentProvider>> = std::sync::OnceLock::new();

pub fn set_content_provider(provider: Arc<dyn ContentProvider>) {
    if CONTENT_PROVIDER.set(provider).is_err() {
        warn!("ContentProvider already set — ignoring duplicate registration");
    }
}

fn get_content_provider() -> Option<&'static Arc<dyn ContentProvider>> {
    CONTENT_PROVIDER.get()
}

pub struct SoulLoader;

impl SoulLoader {
    fn get_soul_path(agent: &str, _lang: &str) -> PathBuf {
        PathBuf::from("res/prompts/soul").join(format!("{}.md", agent))
    }

    pub fn is_extra_agent(agent: &str) -> bool {
        _domain_agent::AgentKind::from_folder_name(agent)
            .map(|k: _domain_agent::AgentKind| k.is_layer2())
            .unwrap_or(false)
    }

    fn parse_front_matter(content: &str) -> Result<(String, String)> {
        let parts = crate::front_matter::extract_front_matter(content)
            .ok_or_else(|| anyhow!("front matter not found"))?;

        let value: toml::Value = parts
            .parse_toml_value()
            .map_err(|e| anyhow!("TOML parse failed: {}", e))?;

        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing 'name' field"))?
            .to_string();

        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("missing 'description' field"))?
            .to_string();

        Ok((name, description))
    }

    pub fn load_sync(agent: &str, lang: &str) -> Result<SoulContent> {
        if let Some(provider) = get_content_provider()
            && let Some(content) = provider.load_soul(agent, lang)
        {
            let (name, description) = Self::parse_front_matter(&content)?;
            return Ok(SoulContent {
                name,
                description,
                content,
            });
        }

        let path = Self::get_soul_path(agent, lang);

        if !path.exists() {
            return Err(anyhow!(
                "Soul file not found for agent '{}' with language '{}': {}",
                agent,
                lang,
                path.display()
            ));
        }

        let content =
            std::fs::read_to_string(&path).map_err(|e| anyhow!("{}: {}", path.display(), e))?;

        let (name, description) = Self::parse_front_matter(&content)?;

        Ok(SoulContent {
            name,
            description,
            content,
        })
    }

    pub async fn load(agent: &str, lang: &str) -> Result<SoulContent> {
        let agent = agent.to_string();
        let lang = lang.to_string();
        tokio::task::spawn_blocking(move || Self::load_sync(&agent, &lang))
            .await
            .map_err(|e| anyhow!("task join error: {}", e))?
    }

    fn get_system_path(_agent: &str, _lang: &str) -> PathBuf {
        PathBuf::from("res/prompts/system/system.md")
    }

    pub fn load_named_prompt(name: &str) -> Option<String> {
        let path = PathBuf::from("res/prompts/system").join(format!("{}.md", name));
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .map(|content| Self::strip_front_matter(&content))
        } else {
            None
        }
    }

    fn strip_front_matter(content: &str) -> String {
        crate::front_matter::strip_front_matter(content).to_string()
    }

    pub fn load_system_sync(agent: &str, lang: &str) -> Option<String> {
        if let Some(provider) = get_content_provider()
            && let Some(content) = provider.load_system(agent, lang)
        {
            return Some(Self::strip_front_matter(&content));
        }

        let path = Self::get_system_path(agent, lang);

        if !path.exists() {
            return None;
        }

        std::fs::read_to_string(&path)
            .ok()
            .map(|content| Self::strip_front_matter(&content))
    }

    pub async fn load_system(agent: &str, lang: &str) -> Result<String> {
        let agent_owned = agent.to_string();
        let lang_owned = lang.to_string();
        let result =
            tokio::task::spawn_blocking(move || Self::load_system_sync(&agent_owned, &lang_owned))
                .await
                .map_err(|e| anyhow!("task join error: {}", e))?;
        result.ok_or_else(|| {
            anyhow!(
                "System prompt file not found for agent '{}' with language '{}'",
                agent,
                lang
            )
        })
    }

    pub fn get_default_lang() -> String {
        std::env::var("APP_LANG").unwrap_or_else(|_| "zh-Hans".to_string())
    }

    pub fn is_valid_lang(lang: &str) -> bool {
        matches!(
            lang,
            "en" | "zh-Hans" | "zh-Hant" | "ja" | "ko" | "fr" | "es" | "ru"
        )
    }

    pub fn normalize_lang(lang: &str) -> String {
        match lang.to_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_hans" => "zh-Hans".to_string(),
            "zh-tw" | "zh_hk" | "zh_hant" => "zh-Hant".to_string(),
            "jp" => "ja".to_string(),
            "kr" => "ko".to_string(),
            other => other.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_is_valid_lang() -> Result<()> {
        assert!(SoulLoader::is_valid_lang("zh-Hans"));
        assert!(SoulLoader::is_valid_lang("en"));
        assert!(SoulLoader::is_valid_lang("ja"));
        assert!(SoulLoader::is_valid_lang("ko"));
        assert!(!SoulLoader::is_valid_lang("de"));
        assert!(!SoulLoader::is_valid_lang("it"));
        Ok(())
    }

    #[test]
    fn test_normalize_lang() -> Result<()> {
        assert_eq!(SoulLoader::normalize_lang("zh"), "zh-Hans");
        assert_eq!(SoulLoader::normalize_lang("zh-cn"), "zh-Hans");
        assert_eq!(SoulLoader::normalize_lang("zh-tw"), "zh-Hant");
        assert_eq!(SoulLoader::normalize_lang("jp"), "ja");
        assert_eq!(SoulLoader::normalize_lang("en"), "en");
        assert_eq!(SoulLoader::normalize_lang("EN"), "en");
        Ok(())
    }

    #[test]
    fn test_parse_front_matter() -> Result<()> {
        let content = r#"+++
name = "Test Agent"
description = "This is a test agent"
+++

# Agent Content

Some content here."#;

        let (name, description) = SoulLoader::parse_front_matter(content)?;
        assert_eq!(name, "Test Agent");
        assert_eq!(description, "This is a test agent");
        Ok(())
    }

    #[test]
    fn test_layer2_soul_path() -> Result<()> {
        let path = SoulLoader::get_soul_path("web_automation", "zh-Hans");
        assert_eq!(path, PathBuf::from("res/prompts/soul/web_automation.md"));
        Ok(())
    }

    #[tokio::test]
    async fn test_load_soul() -> Result<()> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let manifest_path = std::path::PathBuf::from(manifest_dir);
        let repo_root = manifest_path
            .parent()
            .and_then(|p| p.parent())
            .context("no repo root")?;

        std::env::set_current_dir(repo_root)?;

        let result = SoulLoader::load("skopeo", "zh-Hans").await;
        assert!(
            result.is_ok(),
            "Failed to load skopeo soul: {:?}",
            result.err()
        );

        let soul = result?;
        assert_eq!(soul.name, "SkoPeo - Central Coordinator");
        assert!(soul.content.contains("SkoPeo"));
        Ok(())
    }

    #[test]
    fn test_load_soul_sync() -> Result<()> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let manifest_path = std::path::PathBuf::from(manifest_dir);
        let repo_root = manifest_path
            .parent()
            .and_then(|p| p.parent())
            .context("no repo root")?;

        std::env::set_current_dir(repo_root)?;

        let result = SoulLoader::load_sync("skopeo", "zh-Hans");
        assert!(
            result.is_ok(),
            "Failed to load skopeo soul sync: {:?}",
            result.err()
        );

        let soul = result?;
        assert_eq!(soul.name, "SkoPeo - Central Coordinator");
        assert!(soul.content.contains("SkoPeo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_load_extra_agent_soul() -> Result<()> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let manifest_path = std::path::PathBuf::from(manifest_dir);
        let repo_root = manifest_path
            .parent()
            .and_then(|p| p.parent())
            .context("no repo root")?;

        std::env::set_current_dir(repo_root)?;

        let result = SoulLoader::load("web_automation", "zh-Hans").await;
        assert!(
            result.is_ok(),
            "Failed to load web_automation soul: {:?}",
            result.err()
        );

        let soul = result?;
        assert!(soul.content.contains("Web Automation"));
        Ok(())
    }
}
