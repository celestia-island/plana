use anyhow::{Result, anyhow};
use std::collections::HashMap;

use include_dir::{Dir, include_dir};

const SECTION_SKILLS: &str = "skills";
const FILE_EXT_MD_DOT: &str = ".md";

/// Compile-time embedded agents docs directory
///
/// Path is relative to packages/res CARGO_MANIFEST_DIR
pub static AGENTS_DOCS_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../res/prompts/agents");

/// Compile-time embedded domain_agents docs directory
///
/// Path is relative to packages/res CARGO_MANIFEST_DIR
pub static DOMAIN_AGENTS_DOCS_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../res/prompts/domain_agents");

/// Get agents docs directory
pub fn get_agents_docs_dir() -> &'static Dir<'static> {
    &AGENTS_DOCS_DIR
}

/// Get domain_agents docs directory
pub fn get_domain_agents_docs_dir() -> &'static Dir<'static> {
    &DOMAIN_AGENTS_DOCS_DIR
}

/// Compile-time embedded system docs directory (for @system/ reference injection)
pub static SYSTEM_DOCS_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../res/prompts/system");

/// Compile-time embedded soul prompts directory
pub static SOUL_DOCS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../res/prompts/soul");

/// Compile-time embedded prompts directory
pub static PROMPTS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../res/prompts");

/// Get system docs directory
pub fn get_system_docs_dir() -> &'static Dir<'static> {
    &SYSTEM_DOCS_DIR
}

/// Parse Markdown front matter (+++ wrapped TOML)
pub fn extract_front_matter(content: &str) -> Option<String> {
    let start = content.find("+++")?;
    let after_first = start + 3;
    let rest = content.get(after_first..)?;
    let end_in_rest = rest.find("+++")?;
    let toml_text = rest[..end_in_rest].trim();
    Some(toml_text.to_string())
}

/// Validate that a Markdown document's front matter can be parsed as TOML
pub fn validate_markdown_front_matter(content: &str) -> Result<()> {
    let front_matter = extract_front_matter(content)
        .ok_or_else(|| anyhow!("No front matter found (expected +++ delimiters)"))?;

    toml::from_str::<toml::Value>(&front_matter)
        .map_err(|e| anyhow!("Failed to parse front matter TOML: {}", e))?;

    Ok(())
}

/// Check whether the agent docs have all language versions
///
/// Returns list of errors, empty list if all pass
pub fn validate_agent_doc_completeness() -> Vec<String> {
    let mut errors = Vec::new();

    for docs_dir in [&AGENTS_DOCS_DIR, &DOMAIN_AGENTS_DOCS_DIR] {
        for agent_entry in docs_dir.entries() {
            if let Some(agent_dir) = agent_entry.as_dir() {
                let agent_name = agent_entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                for section_name in ["skills", "tools"] {
                    let section_entry = agent_dir.entries().iter().find(|e| {
                        e.path()
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == section_name)
                            .unwrap_or(false)
                    });

                    let Some(section_dir) = section_entry.and_then(|e| e.as_dir()) else {
                        continue;
                    };

                    for item_entry in section_dir.entries() {
                        let item_path = item_entry.path();
                        let Some(item_name) = item_path.file_name().and_then(|n| n.to_str()) else {
                            continue;
                        };

                        if !item_name.ends_with(FILE_EXT_MD_DOT) {
                            continue;
                        }

                        if item_entry.as_dir().is_some() {
                            continue;
                        }

                        let skill_name = item_name.strip_suffix(".md").unwrap_or(item_name);

                        if let Some(file) = item_entry.as_file()
                            && let Some(content) = file.contents_utf8()
                            && let Some(front_matter) = extract_front_matter(content)
                        {
                            match toml::from_str::<toml::Value>(&front_matter) {
                                Ok(parsed) => {
                                    if section_name == SECTION_SKILLS {
                                        for field in ["name", "agent"] {
                                            if parsed.get(field).and_then(|v| v.as_str()).is_none()
                                            {
                                                errors.push(format!(
                                                    "Agent {}/{}/{}: missing '{}' in front matter",
                                                    agent_name, section_name, skill_name, field
                                                ));
                                            }
                                        }
                                        if parsed.get("description").is_none() {
                                            errors.push(format!(
                                                        "Agent {}/{}/{}: missing 'description' in front matter",
                                                        agent_name, section_name, skill_name
                                                    ));
                                        }

                                        if let Some(declared_agent) =
                                            parsed.get("agent").and_then(|v| v.as_str())
                                            && normalize_agent_name(declared_agent)
                                                != Some(agent_name.to_string())
                                        {
                                            errors.push(format!(
                                                            "Agent {}/{}/{}: front matter agent '{}' does not match directory '{}'",
                                                            agent_name, section_name, skill_name,
                                                            declared_agent, agent_name
                                                        ));
                                        }
                                    }
                                }
                                Err(_) => {
                                    errors.push(format!(
                                        "Agent {}/{}/{}: invalid front matter TOML",
                                        agent_name, section_name, skill_name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    errors
}

/// Load the body content of a system doc by its stem name (without `.md`).
///
/// Looks up `res/prompts/system/{name}.md` in the compile-time embedded directory,
/// strips any front matter (`+++...+++`), and returns the trimmed body text.
/// Returns `None` if the file does not exist.
///
/// This is the canonical way for Rust code to reference prompt fragments
/// stored in `res/prompts/system/` instead of hardcoding prompt strings.
pub fn load_system_doc(name: &str) -> Option<&'static str> {
    let filename = format!("{name}.md");
    for entry in SYSTEM_DOCS_DIR.entries() {
        let path = entry.path();
        let Some(entry_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if entry_name != filename {
            continue;
        }
        let Some(file) = entry.as_file() else {
            continue;
        };
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        let body = extract_body_after_front_matter(content);
        return Some(body.trim());
    }
    None
}

/// Expand @system/xxx references in skill body content.
///
/// Each `@system/name` on its own line is replaced by the content of
/// `res/prompts/system/name.md` (without its front matter, if any).
pub fn expand_system_refs(content: &str) -> String {
    let mut result = content.to_string();
    for entry in SYSTEM_DOCS_DIR.entries() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        let Some(file) = entry.as_file() else {
            continue;
        };
        let Some(file_content) = file.contents_utf8() else {
            continue;
        };
        let pattern = format!("@system/{stem}");
        if result.contains(&pattern) {
            let body = extract_body_after_front_matter(file_content);
            result = result.replace(&pattern, body.trim());
        }
    }
    result
}

fn extract_body_after_front_matter(content: &str) -> &str {
    let start = content.find("+++").map(|s| s + 3);
    let Some(start) = start else { return content };
    let rest = match content.get(start..) {
        Some(r) => r,
        None => return content,
    };
    let Some(end) = rest.find("+++") else {
        return content;
    };
    let body_start = start + end + 3;
    match content.get(body_start..) {
        Some(b) => b.trim_start_matches(['\r', '\n']),
        None => content,
    }
}

/// Expand `{{variable}}` placeholders in content using the provided variable map.
///
/// Placeholders use the `{{var_name}}` syntax. If a variable is not found in the map,
/// the placeholder is left as-is (not removed), allowing partial expansion.
pub fn expand_dynamic_vars(content: &str, vars: &HashMap<&str, String>) -> String {
    let mut result = content.to_string();
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        if result.contains(&pattern) {
            result = result.replace(&pattern, value);
        }
    }
    result
}

/// Expand @system/xxx references, then expand {{variable}} placeholders.
///
/// This is the combined expansion pipeline:
/// 1. Static: replace `@system/name` with the body of `res/prompts/system/name.md`
/// 2. Dynamic: replace `{{var_name}}` with runtime values
///
/// The two-phase design allows system docs to contain dynamic placeholders
/// (e.g., `{{container_badge}}`, `{{current_datetime}}`) that are resolved
/// at prompt construction time.
pub fn expand_system_refs_with_vars(content: &str, vars: &HashMap<&str, String>) -> String {
    let expanded = expand_system_refs(content);
    expand_dynamic_vars(&expanded, vars)
}

fn normalize_agent_name(value: &str) -> Option<String> {
    let candidate = value.trim().to_lowercase().replace([' ', '-', '_'], "");

    crate::agent_names::KNOWN_AGENTS
        .iter()
        .find(|&&agent| agent == candidate)
        .map(|agent| agent.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_extract_front_matter() -> Result<()> {
        let content = r#"
+++
name = "test"
description = "A test"
+++

# Body content
"#;
        let front_matter = extract_front_matter(content);
        assert!(front_matter.is_some());
        let fm = front_matter.context("expected front matter")?;
        assert!(fm.contains("name"));
        Ok(())
    }

    #[test]
    fn test_validate_front_matter() -> Result<()> {
        let content = r#"
+++
name = "test"
description = "A test"
+++

# Body content
"#;
        assert!(validate_markdown_front_matter(content).is_ok());
        Ok(())
    }

    #[test]
    fn test_validate_invalid_front_matter() -> Result<()> {
        let content = r#"
+++
name = "test
invalid toml
+++

# Body content
"#;
        assert!(validate_markdown_front_matter(content).is_err());
        Ok(())
    }

    #[test]
    fn test_agents_docs_dir_exists() -> Result<()> {
        let agents_dir = get_agents_docs_dir();
        assert!(
            agents_dir
                .find("*.md")
                .is_ok_and(|mut i| i.next().is_some()),
            "agents docs dir should contain at least one .md file"
        );
        let domain_dir = get_domain_agents_docs_dir();
        assert!(
            domain_dir
                .find("*.md")
                .is_ok_and(|mut i| i.next().is_some()),
            "domain agents docs dir should contain at least one .md file"
        );
        Ok(())
    }

    #[test]
    fn test_system_docs_dir_exists() -> Result<()> {
        let system_dir = get_system_docs_dir();
        assert!(
            system_dir
                .find("*.md")
                .is_ok_and(|mut i| i.next().is_some()),
            "system docs dir should contain at least one .md file"
        );
        Ok(())
    }

    #[test]
    fn test_expand_system_refs_known_reference() -> Result<()> {
        let body = "Some skill content\n\n@system/return-type-convention";
        let expanded = expand_system_refs(body);
        assert!(
            expanded.contains("IEPL Type Enforcement"),
            "should contain injected system doc content, got:\n{expanded}"
        );
        assert!(
            !expanded.contains("@system/return-type-convention"),
            "reference token should be replaced"
        );
        Ok(())
    }

    #[test]
    fn test_expand_system_refs_no_references() -> Result<()> {
        let body = "Plain skill content with no references.";
        let expanded = expand_system_refs(body);
        assert_eq!(expanded, body);
        Ok(())
    }

    #[test]
    fn test_expand_system_refs_multiple_refs() -> Result<()> {
        let body = "@system/return-type-convention\n\n@system/decision-philosophy";
        let expanded = expand_system_refs(body);
        assert!(expanded.contains("IEPL Type Enforcement"));
        assert!(expanded.contains("Decision Philosophy"));
        Ok(())
    }

    #[test]
    fn test_expand_dynamic_vars_basic() -> Result<()> {
        let content = "Badge: {{container_badge}}, Mode: {{execution_mode}}";
        let mut vars = HashMap::new();
        vars.insert("container_badge", "#123".to_string());
        vars.insert("execution_mode", "write".to_string());
        let result = expand_dynamic_vars(content, &vars);
        assert_eq!(result, "Badge: #123, Mode: write");
        Ok(())
    }

    #[test]
    fn test_expand_dynamic_vars_missing_var_left_as_is() -> Result<()> {
        let content = "Badge: {{container_badge}}, Unknown: {{unknown_var}}";
        let mut vars = HashMap::new();
        vars.insert("container_badge", "#123".to_string());
        let result = expand_dynamic_vars(content, &vars);
        assert_eq!(result, "Badge: #123, Unknown: {{unknown_var}}");
        Ok(())
    }

    #[test]
    fn test_expand_system_refs_with_vars_combined() -> Result<()> {
        let body = "Skill content\n\n@system/return-type-convention\n\nBadge: {{container_badge}}";
        let mut vars = HashMap::new();
        vars.insert("container_badge", "#demiurge".to_string());
        let result = expand_system_refs_with_vars(body, &vars);
        assert!(
            result.contains("IEPL Type Enforcement"),
            "static @system/ should expand"
        );
        assert!(
            result.contains("#demiurge"),
            "dynamic {{var}} should expand"
        );
        assert!(!result.contains("@system/return-type-convention"));
        assert!(!result.contains("{{container_badge}}"));
        Ok(())
    }
}
