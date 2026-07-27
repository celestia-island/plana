use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub ecosystem: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub mirrors: Vec<String>,
    #[serde(default)]
    pub detect_files: Vec<String>,
}

impl RegistryEntry {
    pub fn is_always_active(&self) -> bool {
        self.detect_files.is_empty()
    }

    pub fn all_domains(&self) -> Vec<&str> {
        let mut result: Vec<&str> = self.domains.iter().map(|s| s.as_str()).collect();
        for m in &self.mirrors {
            result.push(m.as_str());
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryWhitelistFile {
    pub registries: toml::Value,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryWhitelist {
    pub registries: Vec<RegistryEntry>,
}

impl RegistryWhitelist {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path).with_context(|| {
            format!("failed to read registry whitelist from {}", path.display())
        })?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self> {
        let value: toml::Value =
            toml::from_str(raw).context("failed to parse registry whitelist")?;

        let registries_table = value
            .get("registries")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();

        let mut registries = Vec::new();
        for (_key, entry_value) in &registries_table {
            let entry: RegistryEntry = entry_value
                .clone()
                .try_into()
                .context("failed to parse registry whitelist")?;
            registries.push(entry);
        }

        Ok(Self { registries })
    }

    pub fn resolve_from_workspace(&self, workspace: &Path) -> Vec<String> {
        let mut enabled = HashSet::new();

        for entry in &self.registries {
            let should_enable = if entry.is_always_active() {
                true
            } else {
                entry
                    .detect_files
                    .iter()
                    .any(|pattern| Self::detect_file_exists(workspace, pattern))
            };

            if should_enable {
                for domain in &entry.domains {
                    enabled.insert(domain.clone());
                }
                for mirror in &entry.mirrors {
                    enabled.insert(mirror.clone());
                }
            }
        }

        let mut result: Vec<String> = enabled.into_iter().collect();
        result.sort();
        result
    }

    fn detect_file_exists(workspace: &Path, pattern: &str) -> bool {
        if pattern.contains('*') {
            Self::glob_match(workspace, pattern)
        } else {
            workspace.join(pattern).exists()
        }
    }

    fn glob_match(workspace: &Path, pattern: &str) -> bool {
        if let Ok(entries) = std::fs::read_dir(workspace) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if Self::simple_glob_match(pattern, &name_str) {
                    return true;
                }
            }
        }
        false
    }

    fn simple_glob_match(pattern: &str, name: &str) -> bool {
        let prefix = pattern.strip_suffix('*').unwrap_or(pattern);
        if let Some(suffix) = pattern.strip_prefix('*') {
            name.ends_with(suffix)
        } else if pattern.ends_with('*') {
            name.starts_with(prefix)
        } else {
            name == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use std::fs;

    #[test]
    fn parse_minimal_toml() -> Result<()> {
        let toml = r#"
[registries]
"#;
        let wl = RegistryWhitelist::parse(toml)?;
        assert!(wl.registries.is_empty());
        Ok(())
    }

    #[test]
    fn parse_single_registry() -> Result<()> {
        let toml = r#"
[registries]

[registries.crates_io]
ecosystem = "Rust"
description = "Cargo package registry"
domains = ["crates.io", "static.crates.io"]
mirrors = ["mirrors.tuna.tsinghua.edu.cn"]
detect_files = ["Cargo.toml", "Cargo.lock"]
"#;
        let wl = RegistryWhitelist::parse(toml)?;
        assert_eq!(wl.registries.len(), 1);
        let entry = &wl.registries[0];
        assert_eq!(entry.ecosystem, "Rust");
        assert_eq!(entry.domains, vec!["crates.io", "static.crates.io"]);
        assert_eq!(entry.mirrors, vec!["mirrors.tuna.tsinghua.edu.cn"]);
        assert_eq!(entry.detect_files, vec!["Cargo.toml", "Cargo.lock"]);
        assert!(!entry.is_always_active());
        Ok(())
    }

    #[test]
    fn always_active_registry() -> Result<()> {
        let toml = r#"
[registries]

[registries.github]
ecosystem = "Universal"
description = "GitHub"
domains = ["github.com", "raw.githubusercontent.com"]
detect_files = []
"#;
        let wl = RegistryWhitelist::parse(toml)?;
        assert!(wl.registries[0].is_always_active());
        Ok(())
    }

    #[test]
    fn resolve_from_workspace_rust() -> Result<()> {
        let dir = tempfile::tempdir().context("create temp dir")?;
        fs::write(dir.path().join("Cargo.toml"), "").context("write Cargo.toml")?;

        let toml = r#"
[registries]

[registries.crates_io]
ecosystem = "Rust"
domains = ["crates.io", "static.crates.io"]
mirrors = ["mirrors.tuna.tsinghua.edu.cn"]
detect_files = ["Cargo.toml"]

[registries.github]
ecosystem = "Universal"
domains = ["github.com"]
detect_files = []
"#;
        let wl = RegistryWhitelist::parse(toml)?;
        let domains = wl.resolve_from_workspace(dir.path());
        assert!(domains.contains(&"crates.io".to_string()));
        assert!(domains.contains(&"static.crates.io".to_string()));
        assert!(domains.contains(&"mirrors.tuna.tsinghua.edu.cn".to_string()));
        assert!(domains.contains(&"github.com".to_string()));
        Ok(())
    }

    #[test]
    fn resolve_empty_workspace_only_universal() -> Result<()> {
        let dir = tempfile::tempdir().context("create temp dir")?;

        let toml = r#"
[registries]

[registries.crates_io]
ecosystem = "Rust"
domains = ["crates.io"]
detect_files = ["Cargo.toml"]

[registries.github]
ecosystem = "Universal"
domains = ["github.com"]
detect_files = []
"#;
        let wl = RegistryWhitelist::parse(toml)?;
        let domains = wl.resolve_from_workspace(dir.path());
        assert!(!domains.contains(&"crates.io".to_string()));
        assert!(domains.contains(&"github.com".to_string()));
        Ok(())
    }

    #[test]
    fn glob_detect_pattern() -> Result<()> {
        let dir = tempfile::tempdir().context("create temp dir")?;
        fs::write(dir.path().join("main.csproj"), "").context("write main.csproj")?;

        assert!(RegistryWhitelist::detect_file_exists(
            dir.path(),
            "*.csproj"
        ));
        assert!(!RegistryWhitelist::detect_file_exists(
            dir.path(),
            "*.fsproj"
        ));
        Ok(())
    }

    #[test]
    fn all_domains_combines_primary_and_mirrors() -> Result<()> {
        let entry = RegistryEntry {
            ecosystem: "test".to_string(),
            description: "test".to_string(),
            domains: vec!["a.com".to_string(), "b.com".to_string()],
            mirrors: vec!["m1.com".to_string()],
            detect_files: vec![],
        };
        let all = entry.all_domains();
        assert_eq!(all, vec!["a.com", "b.com", "m1.com"]);
        Ok(())
    }
}
