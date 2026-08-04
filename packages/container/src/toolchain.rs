//! Toolchain image profile configuration for agent containers.
//!
//! [`ToolchainImageProfile`] describes a container image's capabilities: supported
//! languages, available tools, LSP servers ([`LspServerEntry`]), cache directories,
//! system packages, resource limits, and egress domain allowlist. Profiles are
//! loaded from `.yaml` files in `amphoreus_dir/image/` and discovered by language
//! or LSP need. This crate is the configuration layer that wires toolchain
//! containers into the agent execution sandbox.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerEntry {
    pub language: String,
    pub cmd: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_min_memory_mb() -> u64 {
    2048
}

fn default_min_cpu_cores() -> u64 {
    2
}

fn default_max_pids() -> u64 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainImageProfile {
    pub id: String,
    pub display_name: String,
    pub source_image: String,
    #[serde(default)]
    pub source_image_is_alpine: bool,
    #[serde(default)]
    pub binary_paths: Vec<String>,
    #[serde(default)]
    pub available_tools: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub supported_languages: Vec<String>,
    #[serde(default)]
    pub verify: Vec<String>,
    #[serde(default)]
    pub requires_network: bool,
    #[serde(default)]
    pub lsp_servers: Vec<LspServerEntry>,
    #[serde(default)]
    pub cache_dirs: Vec<String>,
    #[serde(default)]
    pub sysroot_packages: Vec<String>,
    #[serde(default)]
    pub egress_domains: Vec<String>,
    #[serde(default)]
    pub writable_workspace: bool,
    #[serde(default = "default_min_memory_mb")]
    pub min_memory_mb: u64,
    #[serde(default = "default_min_cpu_cores")]
    pub min_cpu_cores: u64,
    #[serde(default = "default_max_pids")]
    pub max_pids: u64,
}

impl ToolchainImageProfile {
    pub fn find_lsp_server(&self, language: &str) -> Option<&LspServerEntry> {
        self.lsp_servers.iter().find(|s| s.language == language)
    }

    pub fn needs_compile_support(&self) -> bool {
        self.writable_workspace || !self.cache_dirs.is_empty() || !self.sysroot_packages.is_empty()
    }
}

fn image_dir(amphoreus_dir: &Path) -> PathBuf {
    amphoreus_dir.join("image")
}

pub fn list_profiles(amphoreus_dir: &Path) -> Vec<String> {
    let dir = image_dir(amphoreus_dir);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                profiles.push(stem.to_string());
            }
        }
    }
    profiles.sort();
    profiles
}

pub fn load_profile(amphoreus_dir: &Path, profile_id: &str) -> Result<ToolchainImageProfile> {
    let dir = image_dir(amphoreus_dir);
    for ext in &["yaml", "yml"] {
        let path = dir.join(format!("{}.{}", profile_id, ext));
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            return parse_profile(&raw, &path.display().to_string());
        }
    }
    Err(anyhow!(
        "profile '{}' not found in {}",
        profile_id,
        dir.display()
    ))
}

pub fn parse_profile(raw: &str, source: &str) -> Result<ToolchainImageProfile> {
    serde_yaml::from_str(raw).with_context(|| format!("failed to parse {}", source))
}

pub fn find_for_language(amphoreus_dir: &Path, lang: &str) -> Option<String> {
    for id in list_profiles(amphoreus_dir) {
        if let Ok(profile) = load_profile(amphoreus_dir, &id)
            && profile.supported_languages.iter().any(|l| l == lang)
        {
            return Some(id);
        }
    }
    None
}

pub fn find_lsp_profile(amphoreus_dir: &Path, language: &str) -> Option<(String, LspServerEntry)> {
    for id in list_profiles(amphoreus_dir) {
        if let Ok(profile) = load_profile(amphoreus_dir, &id)
            && let Some(entry) = profile.find_lsp_server(language)
        {
            return Some((id, entry.clone()));
        }
    }
    None
}
