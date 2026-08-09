//! Skill definition and registration
//!
//! This module provides a mechanism to define and register skills, including:
//! - Compile-time prompt file inclusion (using `include_str!`)
//! - Front matter metadata parsing
//! - Required tool availability validation
//! - Complete system prompt construction

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use _prompt::{
    prompt_loader::{PromptLoader, PromptMetadata},
    soul_loader::{SoulContent, SoulLoader},
};
use _state_sync::Agent;

/// Skill result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub tool_calls: Vec<serde_json::Value>,
}

impl SkillResult {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            tool_calls: vec![],
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error),
            tool_calls: vec![],
        }
    }
}

/// Skill invoker trait
#[async_trait::async_trait]
pub trait SkillInvoker: Send + Sync {
    async fn invoke(&self, skill_name: &str, parameters: serde_json::Value) -> SkillResult;

    fn get_skills(&self) -> Vec<_state_sync::SkillInfo>;
}

/// Skill definition (contains compile-time included prompt)
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name
    pub name: String,
    /// Skill description (multi-language)
    pub description: std::collections::HashMap<String, String>,
    /// Agent type
    pub agent_type: Agent,
    /// List of required tools
    pub required_tools: Vec<String>,
    /// Skill version
    pub version: String,
    /// Prompt template (compile-time included)
    pub prompt_template: String,
    /// Prompt metadata
    pub metadata: PromptMetadata,
    pub next_action: Vec<_prompt::prompt_loader::StepAction>,
    pub must_touch_next_action: bool,
}

impl Skill {
    /// Create Skill from compile-time included string
    ///
    /// # Example
    /// ```rust,ignore
    /// const PROMPT: &str = include_str!("../../../res/prompts/agents/skopeo/prompts/human_requirement_parse.md");
    /// let skill = Skill::from_include_str(PROMPT, &available_tools)?;
    /// ```
    pub fn parse_steps(content: &str) -> Vec<_prompt::prompt_loader::StepAction> {
        let mut next_action = Vec::new();

        if let Some(parts) = _prompt::front_matter::extract_front_matter(content)
            && let Ok(parsed) = parts.parse_toml_value()
            && let Some(ns) = parsed.get("next_action").and_then(|v| v.as_array())
        {
            for item in ns {
                if let Some(table) = item.as_table() {
                    let agent = table.get("agent").and_then(|v| v.as_str()).unwrap_or("");
                    let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if !agent.is_empty() && !name.is_empty() {
                        next_action.push(_prompt::prompt_loader::StepAction {
                            agent: agent.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
            }
        }

        next_action
    }

    pub fn parse_must_touch_next_action(content: &str) -> bool {
        _prompt::front_matter::extract_front_matter(content)
            .and_then(|parts| parts.parse_toml_value().ok())
            .and_then(|parsed| {
                parsed
                    .get("must_touch_next_action")
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true)
    }

    pub fn from_include_str(content: &str, available_tools: &[String]) -> Result<Self> {
        let template = PromptLoader::parse_from_string(content)
            .map_err(|e| anyhow!("failed to parse prompt: {}", e))?;

        let mut required = template.metadata.required_tools.clone();
        for rt in &template.metadata.related_tools {
            if !required.contains(&rt.tool_name) {
                required.push(rt.tool_name.clone());
            }
        }
        for rs in &template.metadata.related_skills {
            if !required.contains(&rs.tool_name) {
                required.push(rs.tool_name.clone());
            }
        }

        PromptLoader::validate_tools(&required, available_tools)?;

        let agent_type = template
            .metadata
            .agent
            .parse::<Agent>()
            .map_err(|_| anyhow!("Unknown agent type: {}", template.metadata.agent))?;

        let next_action = Self::parse_steps(content);
        let must_touch_next_action = template.metadata.features.must_touch_next_action;

        Ok(Self {
            name: template.metadata.name.clone(),
            description: template.metadata.description.clone(),
            agent_type,
            required_tools: required,
            version: template.metadata.version.clone(),
            prompt_template: template.template,
            metadata: template.metadata,
            next_action,
            must_touch_next_action,
        })
    }

    /// Build complete system prompt
    pub fn build_system_prompt(&self) -> String {
        self.build_system_prompt_with_lang(None)
    }

    /// Build complete system prompt (specified language)
    ///
    /// # Parameters
    /// - `lang`: language code (e.g. "en", "zh-Hans", "ja"), uses default language if None
    ///
    /// This method attempts to load the system prompt and soul file for the given language and include them in the system prompt
    pub fn build_system_prompt_with_lang(&self, lang: Option<&str>) -> String {
        let loader = PromptLoader::new(std::path::PathBuf::new());

        // Try to load soul file
        let default_lang = SoulLoader::get_default_lang();
        let lang_code = lang.unwrap_or(&default_lang);
        let normalized_lang = SoulLoader::normalize_lang(lang_code);

        let agent_name = self.agent_type.to_string();
        let soul_content = SoulLoader::load_sync(&agent_name, &normalized_lang)
            .ok()
            .map(|soul| soul.content);

        // Load top-level system prompt (per-agent override → global shared)
        let system_content = SoulLoader::load_system_sync(&agent_name, &normalized_lang);

        loader.build_system_prompt(
            &self.metadata,
            &self.prompt_template,
            soul_content.as_deref(),
            system_content.as_deref(),
        )
    }

    /// Get soul content (not included in system prompt)
    ///
    /// # Parameters
    /// - `lang`: language code (e.g. "en", "zh-Hans", "ja"), uses default language if None
    pub fn get_soul_content(&self, lang: Option<&str>) -> Option<SoulContent> {
        let default_lang = SoulLoader::get_default_lang();
        let lang_code = lang.unwrap_or(&default_lang);
        let normalized_lang = SoulLoader::normalize_lang(lang_code);
        let agent_name = self.agent_type.to_string();

        SoulLoader::load_sync(&agent_name, &normalized_lang).ok()
    }

    /// Render prompt template (replace variables)
    pub fn render_prompt(&self, variables: &HashMap<&str, String>) -> Result<String> {
        let loader = PromptLoader::new(std::path::PathBuf::new());
        loader
            .render(&self.prompt_template, variables)
            .map_err(|e| anyhow!("{}", e))
    }

    /// Convert to SkillInfo (for compatibility with core library)
    pub fn to_info(&self) -> _state_sync::SkillInfo {
        let location = self
            .metadata
            .features
            .location
            .unwrap_or(_state_sync::tools::SkillLocation::Scepter);
        _state_sync::SkillInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            agent_type: self.agent_type.clone(),
            required_tools: self.required_tools.clone(),
            location,
            ..Default::default()
        }
    }
}

/// Skill registry (thread-safe)
#[derive(Clone)]
pub struct SkillRegistry {
    skills: Arc<RwLock<HashMap<String, Skill>>>,
}

impl SkillRegistry {
    /// Create new Skill registry
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register Skill
    pub async fn register(&self, skill: Skill) {
        let mut skills = self.skills.write().await;
        skills.insert(skill.name.clone(), skill);
    }

    /// Batch register Skills
    pub async fn register_all(&self, skills: Vec<Skill>) {
        let mut skills_map = self.skills.write().await;
        for skill in skills {
            skills_map.insert(skill.name.clone(), skill);
        }
    }

    /// Get Skill
    pub async fn get(&self, name: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.get(name).cloned()
    }

    /// Get Skill's system prompt
    pub async fn get_system_prompt(&self, name: &str) -> Option<String> {
        self.get(name)
            .await
            .map(|skill| skill.build_system_prompt())
    }

    /// Get Skill's system prompt (specified language)
    ///
    /// # Parameters
    /// - `name`: skill name
    /// - `lang`: language code (e.g. "en", "zh-Hans", "ja"), uses default language if None
    pub async fn get_system_prompt_with_lang(
        &self,
        name: &str,
        lang: Option<&str>,
    ) -> Option<String> {
        self.get(name)
            .await
            .map(|skill| skill.build_system_prompt_with_lang(lang))
    }

    /// Get Skill's soul content
    ///
    /// # Parameters
    /// - `name`: skill name
    /// - `lang`: language code (e.g. "en", "zh-Hans", "ja"), uses default language if None
    pub async fn get_soul_content(&self, name: &str, lang: Option<&str>) -> Option<SoulContent> {
        self.get(name)
            .await
            .and_then(|skill| skill.get_soul_content(lang))
    }

    /// Get all Skills
    pub async fn list_all(&self) -> Vec<_state_sync::SkillInfo> {
        let skills = self.skills.read().await;
        skills.values().map(|skill| skill.to_info()).collect()
    }

    /// Get Skills for specified Agent
    pub async fn list_by_agent(&self, agent_type: Agent) -> Vec<_state_sync::SkillInfo> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|skill| skill.agent_type == agent_type)
            .map(|skill| skill.to_info())
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_skill_from_include_str() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_skill"
description = "Test Skill"
agent = "SkoPeo"
version = "1.0.0"
required_tools = ["tool1"]
+++

This is test skill template

## Usage

{{input}}
"#;

        let available_tools = vec!["tool1".to_string()];
        let skill = Skill::from_include_str(TEST_PROMPT, &available_tools)?;

        assert_eq!(skill.name, "test_skill");
        assert_eq!(
            skill
                .description
                .get("en")
                .context("missing 'en' description")?,
            "Test Skill"
        );
        assert_eq!(skill.required_tools, vec!["tool1"]);
        Ok(())
    }

    #[test]
    fn test_skill_tool_validation() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_skill"
description = "Test Skill"
agent = "SkoPeo"
version = "1.0.0"
required_tools = ["missing_tool"]
+++

Test content"#;

        let available_tools = vec!["tool1".to_string()];
        let result = Skill::from_include_str(TEST_PROMPT, &available_tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Required tool"));
        Ok(())
    }

    #[test]
    fn test_build_system_prompt() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_skill"
description = "Test skill description"
agent = "SkoPeo"
version = "1.0.0"
+++

Skill template content"#;

        let available_tools = vec![];
        let skill = Skill::from_include_str(TEST_PROMPT, &available_tools)?;

        let system_prompt = skill.build_system_prompt();
        assert!(system_prompt.contains("You are SkoPeo"));
        assert!(system_prompt.contains("Test skill description"));
        assert!(system_prompt.contains("Skill template content"));
        assert!(system_prompt.contains("## Core Duties"));
        Ok(())
    }

    #[test]
    fn test_skill_steps_parsing() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_skill"
description = "Test Skill"
agent = "HubRis"
version = "1.0.0"

[[next_action]]
agent = "hubris"
name = "workplan_generate"

[[next_action]]
agent = "hubris"
name = "operator"
+++

Test content"#;

        let available_tools = vec![];
        let skill = Skill::from_include_str(TEST_PROMPT, &available_tools)?;
        assert_eq!(skill.next_action.len(), 2);
        assert_eq!(skill.next_action[0].agent, "hubris");
        assert_eq!(skill.next_action[0].name, "workplan_generate");
        assert_eq!(skill.next_action[1].agent, "hubris");
        assert_eq!(skill.next_action[1].name, "operator");
        Ok(())
    }

    #[test]
    fn test_skill_steps_default_empty() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_skill"
description = "Test Skill"
agent = "HubRis"
version = "1.0.0"
+++

Test content"#;

        let available_tools = vec![];
        let skill = Skill::from_include_str(TEST_PROMPT, &available_tools)?;
        assert!(skill.next_action.is_empty());
        Ok(())
    }

    #[test]
    fn test_related_tools_merged_into_required() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_gateway"
description = "Test Gateway"
agent = "KaLos"
version = "1.0.0"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"
+++

Test content"#;

        let available_tools = vec![
            "file_read".to_string(),
            "file_write".to_string(),
            "report".to_string(),
        ];
        let skill = Skill::from_include_str(TEST_PROMPT, &available_tools)?;
        assert_eq!(skill.required_tools.len(), 3);
        assert!(skill.required_tools.contains(&"file_read".to_string()));
        assert!(skill.required_tools.contains(&"file_write".to_string()));
        assert!(skill.required_tools.contains(&"report".to_string()));
        Ok(())
    }

    #[test]
    fn test_related_tools_validation_fails_for_missing() -> Result<()> {
        const TEST_PROMPT: &str = r#"+++
name = "test_gateway"
description = "Test Gateway"
agent = "KaLos"
version = "1.0.0"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_read"

[[related_tools]]
agent_name = "kalos"
tool_name = "nonexistent_tool"
+++

Test content"#;

        let available_tools = vec!["file_read".to_string()];
        let result = Skill::from_include_str(TEST_PROMPT, &available_tools);
        assert!(result.is_err());
        Ok(())
    }
}
