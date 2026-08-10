use anyhow::{Result, anyhow};
use std::collections::HashMap;

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

/// Strip the front matter (`+++...+++`) from a Markdown prompt and return the
/// body text. The prompt tree lives in the entelecheia repo; consumers embed
/// their own copy and drive this shared helper with its content.
pub fn extract_body_after_front_matter(content: &str) -> &str {
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
}
