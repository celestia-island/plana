use anyhow::{Result, anyhow};
use serde::{
    Deserialize,
    de::{self, MapAccess, Visitor},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use tracing::warn;

use _core::execution_mode::ExecutionMode;

fn deserialize_description<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct DescVisitor;

    impl<'de> Visitor<'de> for DescVisitor {
        type Value = HashMap<String, String>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a string or a table of language keys")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            let mut map = HashMap::new();
            map.insert("en".to_string(), v.to_string());
            Ok(map)
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = HashMap::new();
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                result.insert(key, value);
            }
            Ok(result)
        }
    }

    deserializer.deserialize_any(DescVisitor)
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepAction {
    pub agent: String,
    pub name: String,
}

impl StepAction {
    pub fn target(&self) -> String {
        format!("{}::{}", self.agent, self.name)
    }
}

impl<'de> Deserialize<'de> for StepAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StepActionVisitor;

        impl<'de> Visitor<'de> for StepActionVisitor {
            type Value = StepAction;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a table with 'agent' and 'name' fields")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct StepActionFields {
                    agent: String,
                    name: String,
                }
                let fields =
                    StepActionFields::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(StepAction {
                    agent: fields.agent,
                    name: fields.name,
                })
            }
        }

        deserializer.deserialize_any(StepActionVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelatedTool {
    pub agent_name: String,
    pub tool_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerConfig {
    pub topic_pattern: String,
    #[serde(default)]
    pub http_path: Option<String>,
    #[serde(default)]
    pub http_method: Option<String>,
    #[serde(default)]
    pub secret_env: Option<String>,
}

/// Prompt template metadata (TOML front matter)
#[derive(Debug, Clone, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    #[serde(deserialize_with = "deserialize_description")]
    pub description: HashMap<String, String>,
    pub agent: String,
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub next_action: Vec<StepAction>,
    #[serde(default)]
    pub related_tools: Vec<RelatedTool>,
    #[serde(default)]
    pub related_skills: Vec<RelatedTool>,
    #[serde(default)]
    pub triggers: Vec<TriggerConfig>,
    #[serde(default)]
    pub features: PromptFeatures,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PromptFeatures {
    #[serde(default)]
    pub location: Option<_state_sync::mcp::SkillLocation>,
    #[serde(default)]
    pub config: Vec<String>,
    #[serde(default)]
    pub will_fork_container: bool,
    #[serde(default)]
    pub report_only: bool,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub requires_edge_node: Option<String>,
    #[serde(default = "default_true")]
    pub must_touch_next_action: bool,
    #[serde(default)]
    pub must_use_at_least_once: Vec<String>,
    /// Skill role: `"coordinator"` (dispatches sub-skills, must not perform
    /// writes/exec/container ops itself) or `"worker"` (default — may use the
    /// full tool whitelist). Enforced in the skill-chain pipeline (IB-02).
    #[serde(default)]
    pub role: SkillRole,
}

/// Classifies a skill for tool-whitelist enforcement.
///
/// Coordinators orchestrate other skills and must never execute dangerous
/// side-effecting tools directly; the pipeline strips `file_write`,
/// `host_command_exec`, industrial writes, and container ops from their
/// `allowed_tools`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillRole {
    /// Default — full whitelist permitted (gated by other policies).
    #[default]
    Worker,
    /// Orchestrator — dangerous write/exec/container tools are stripped.
    Coordinator,
}

impl SkillRole {
    pub fn is_coordinator(self) -> bool {
        matches!(self, SkillRole::Coordinator)
    }
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_true() -> bool {
    true
}

/// Prompt template
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub metadata: PromptMetadata,
    pub template: String,
}

/// Prompt loader
pub struct PromptLoader {
    base_path: PathBuf,
}

impl PromptLoader {
    /// Create new Prompt loader
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Create from default location (res/prompts/)
    pub fn from_agent(_agent: &str) -> Self {
        let base_path = PathBuf::from("res/prompts");
        Self::new(base_path)
    }

    /// Parse from string (for compile-time included strings)
    pub fn parse_from_string(content: &str) -> Result<PromptTemplate> {
        Self::parse_content(content)
    }

    /// Parse Prompt template content
    pub fn parse_content(content: &str) -> Result<PromptTemplate> {
        let parts = crate::front_matter::extract_front_matter(content)
            .ok_or_else(|| anyhow!("TOML front matter not found"))?;

        let metadata: PromptMetadata = parts
            .parse_toml()
            .map_err(|e| anyhow!("TOML parse failed: {}", e))?;

        Ok(PromptTemplate {
            metadata,
            template: parts.body.to_string(),
        })
    }

    /// Verify required tools are available
    pub fn validate_tools(required_tools: &[String], available_tools: &[String]) -> Result<()> {
        for tool in required_tools {
            if !available_tools.contains(tool) {
                return Err(anyhow!(
                    "Required tool '{}' is not available. Available tools: {:?}",
                    tool,
                    available_tools
                ));
            }
        }
        Ok(())
    }

    /// Load Prompt template
    pub async fn load(&self, skill_name: &str) -> Result<PromptTemplate> {
        let file_path = self.base_path.join(format!("{}.md", skill_name));

        if tokio::fs::metadata(&file_path).await.is_err() {
            return Err(anyhow!("file not found: {}", file_path.to_string_lossy()));
        }

        let content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| anyhow!("failed to read file {}: {}", file_path.display(), e))?;

        Self::parse_content(&content)
    }

    /// Load all Prompts
    pub async fn load_all(&self) -> Result<HashMap<String, PromptTemplate>> {
        let mut prompts = HashMap::new();

        if tokio::fs::metadata(&self.base_path).await.is_err() {
            return Ok(prompts);
        }

        let mut entries = tokio::fs::read_dir(&self.base_path)
            .await
            .map_err(|e| anyhow!("failed to read directory: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| anyhow!("failed to read directory entry: {}", e))?
        {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("invalid filename"))?;

                match self.load(stem).await {
                    Ok(prompt) => {
                        prompts.insert(stem.to_string(), prompt);
                    }
                    Err(e) => {
                        warn!("failed to load prompt: {} - {}", stem, e);
                    }
                }
            }
        }

        Ok(prompts)
    }

    /// Replace Prompt template variables
    pub fn render(&self, template: &str, variables: &HashMap<&str, String>) -> Result<String> {
        let mut result = template.to_string();

        for (key, value) in variables {
            let pattern_spaced = format!("{{{{ {} }}}}", key);
            let pattern_tight = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern_spaced, value);
            result = result.replace(&pattern_tight, value);
        }

        Ok(result)
    }

    /// Build complete system prompt (includes role definition and template)
    ///
    /// # Parameters
    /// - `metadata`: prompt metadata
    /// - `template`: prompt template content
    /// - `soul_content`: optional soul file content (included in system prompt if provided)
    /// - `system_content`: optional top-level system prompt content (highest priority, placed first)
    pub fn build_system_prompt(
        &self,
        metadata: &PromptMetadata,
        template: &str,
        soul_content: Option<&str>,
        system_content: Option<&str>,
    ) -> String {
        let system_section = if let Some(sys) = system_content {
            format!("{}\n\n", sys.trim())
        } else {
            String::new()
        };

        let soul_section = if let Some(soul) = soul_content {
            format!("{}\n\n", soul.trim())
        } else {
            String::new()
        };

        format!(
            r#"{}{}You are {}, {}.

Version: {}

## Core Duties

{}
"#,
            system_section,
            soul_section,
            metadata.agent,
            metadata
                .description
                .get("en")
                .map(|s| s.as_str())
                .unwrap_or_else(|| metadata
                    .description
                    .values()
                    .next()
                    .map(|s| s.as_str())
                    .unwrap_or("")),
            metadata.version,
            template.trim()
        )
    }
}

/// Test Prompt loading
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_parse_from_string() -> Result<()> {
        let content = r#"+++
name = "test_skill"
description = "Test Skill"
agent = "TestAgent"
version = "1.0.0"
+++

This is the skill template content

## Example

{{example}}
"#;

        let template = PromptLoader::parse_from_string(content)?;
        assert_eq!(template.metadata.name, "test_skill");
        assert!(
            template
                .template
                .contains("This is the skill template content")
        );
        assert!(template.template.contains("{{example}}"));
        Ok(())
    }

    #[test]
    fn test_validate_tools() -> Result<()> {
        let required_tools = vec!["tool1".to_string(), "tool2".to_string()];
        let available_tools = vec![
            "tool1".to_string(),
            "tool2".to_string(),
            "tool3".to_string(),
        ];

        assert!(PromptLoader::validate_tools(&required_tools, &available_tools).is_ok());

        let missing_tool = vec!["tool4".to_string()];
        assert!(PromptLoader::validate_tools(&missing_tool, &available_tools).is_err());
        Ok(())
    }

    #[test]
    fn test_skill_role_defaults_to_worker_and_parses_coordinator() -> Result<()> {
        // Missing role → default Worker.
        let worker = r#"+++
name = "w"
description = "worker"
agent = "TestAgent"
+++
body"#;
        let t = PromptLoader::parse_from_string(worker)?;
        assert_eq!(t.metadata.features.role, SkillRole::Worker);

        // Explicit coordinator.
        let coord = r#"+++
name = "c"
description = "coordinator"
agent = "TestAgent"

[features]
role = "coordinator"
+++
body"#;
        let t = PromptLoader::parse_from_string(coord)?;
        assert_eq!(t.metadata.features.role, SkillRole::Coordinator);
        assert!(t.metadata.features.role.is_coordinator());
        Ok(())
    }

    #[tokio::test]
    async fn test_load_prompt() -> Result<()> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let base_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .context("manifest has no parent")?
            .parent()
            .context("parent has no parent")?
            .parent()
            .context("grandparent has no parent")?
            .join("res/prompts");
        let loader = PromptLoader::new(base_path);
        let prompt = loader.load("human_requirement_parse").await;
        if let Err(ref e) = prompt {
            eprintln!("Load error: {:?}", e);
        }
        assert!(prompt.is_ok(), "Should load prompt successfully");

        let template = prompt?;
        assert_eq!(template.metadata.name, "human_requirement_parse");
        assert!(template.template.contains("SkoPeo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_render_template() -> Result<()> {
        let loader = PromptLoader::new(PathBuf::new());
        let template = "Hello {{name}}, your age is {{age}}";
        let variables = HashMap::from([("name", "Alice".to_string()), ("age", "30".to_string())]);

        let result = loader.render(template, &variables)?;
        assert_eq!(result, "Hello Alice, your age is 30");
        Ok(())
    }
}
