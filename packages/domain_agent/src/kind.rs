//! Unified Agent kind enum.
//!
//! [`AgentKind`] covers the 13 compiled-in agents (Layer1 × 12 + the execution-only industrial gateway); the 15 widget L2 folders retired with W11-6 slice C — they are plugin-carried (`AgentKey::Plugin`) now,
//! The single source for agent names, doc paths, UI display, and other strings.
//! The `AgentType` in TUI is a re-export (type alias) of this type.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown agent: {0}")]
pub struct UnknownAgentError(pub String);

/// Unified Agent kind (Layer1 × 12 + industrial_iot; the widget L2 variants retired with W11-6 slice C)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AgentKind {
    // ── Layer 1 ──────────────────────────────────────────────────────────────
    /// Communication gateway
    HapLotes,
    /// Central coordinator
    SkoPeo,
    /// Work planning engine
    HubRis,
    /// Workflow management
    KaLos,
    /// Container management
    NeiKos,
    /// Script execution and microservice runtime
    SkeMma,
    /// Storage and LLM hub
    ApoRia,
    /// Security and information gathering
    EleOs,
    /// Backup and scheduling
    ///
    /// Historical payloads may carry the former variant spelling; the alias
    /// keeps old serialized inputs deserializable while new output writes
    /// the canonical form.
    #[serde(alias = "EpieiKeia")]
    Epieikeia,
    /// Security audit and external integration
    OreXis,
    /// System integration
    PhiLia,
    /// Edge computing and device management
    PoleMos,

    // ── Layer 2 ──────────────────────────────────────────────────────────────
    // The 2026-10-13 user ruling: the builtin layer-2 lineup is ONE
    // AGENT PER OFFICIAL WIDGET PLUGIN (twelve) — each bundles the
    // plugin's operations as its tools and the usage recipes as its
    // skills, so no builtin ships tools-only or skills-only. The eight
    // pre-ruling folders (web_automation & co.) are retired wholesale.
    /// EXECUTION-ONLY industrial protocol gateway (modbus & co.) — the
    /// hydro axis's engine-internal dispatch identity. It never appears
    /// in either console roster (its descriptor carries layer 0) and no
    /// builtin-agent count includes it; entelecheia's industrial tools
    /// initialize against it. Not a persona, a wire.
    IndustrialIoT,
}

// ─── Display (Pascal-case human-readable name) ─────────────────────────────

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.friendly_name())
    }
}

// ─── FromStr (accepts folder_name or friendly_name) ────────────────────────

impl FromStr for AgentKind {
    type Err = UnknownAgentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let candidate = s.trim().to_lowercase().replace([' ', '-', '_'], "");
        AgentKind::from_folder_name(&candidate).ok_or_else(|| UnknownAgentError(s.to_string()))
    }
}

// ─── Main methods ─────────────────────────────────────────────────────────────

impl AgentKind {
    /// Return the Pascal-case friendly name (matches the enum variant name).
    ///
    /// This is the **single source** for the hardcoded string list;
    /// [`fmt::Display`] and `display_name()` both delegate here.
    pub fn friendly_name(&self) -> &'static str {
        let folder = self.folder_name();
        if let Some(desc) = super::descriptor::AGENT_REGISTRY.get_by_folder(folder) {
            desc.friendly_name
        } else {
            self.friendly_name_fallback()
        }
    }

    fn friendly_name_fallback(&self) -> &'static str {
        match self {
            AgentKind::HapLotes => "HapLotes",
            AgentKind::SkoPeo => "SkoPeo",
            AgentKind::HubRis => "HubRis",
            AgentKind::KaLos => "KaLos",
            AgentKind::NeiKos => "NeiKos",
            AgentKind::SkeMma => "SkeMma",
            AgentKind::ApoRia => "ApoRia",
            AgentKind::EleOs => "EleOs",
            AgentKind::Epieikeia => "Epieikeia",
            AgentKind::OreXis => "OreXis",
            AgentKind::PhiLia => "PhiLia",
            AgentKind::PoleMos => "PoleMos",
            AgentKind::IndustrialIoT => "Industrial IoT",
        }
    }

    /// Return the Display string (same as `to_string()`), for direct use in code.
    pub fn display_name(&self) -> String {
        self.to_string()
    }

    /// Return the lowercase folder/binary name, used for doc paths, process launching, etc.
    ///
    /// This is the **single source** for the AgentKind → lowercase string mapping.
    pub fn folder_name(&self) -> &'static str {
        match self {
            AgentKind::HapLotes => "haplotes",
            AgentKind::SkoPeo => "skopeo",
            AgentKind::HubRis => "hubris",
            AgentKind::KaLos => "kalos",
            AgentKind::NeiKos => "neikos",
            AgentKind::SkeMma => "skemma",
            AgentKind::ApoRia => "aporia",
            AgentKind::EleOs => "eleos",
            AgentKind::Epieikeia => "epieikeia",
            AgentKind::OreXis => "orexis",
            AgentKind::PhiLia => "philia",
            AgentKind::PoleMos => "polemos",
            AgentKind::IndustrialIoT => "industrial_iot",
        }
    }

    /// Parse from folder name (lowercase string after normalizing hyphens/underscores/spaces) to AgentKind.
    pub fn from_folder_name(name: &str) -> Option<Self> {
        let normalized = name.replace(['-', ' '], "_");
        match normalized.as_str() {
            "haplotes" => Some(AgentKind::HapLotes),
            "skopeo" => Some(AgentKind::SkoPeo),
            "hubris" => Some(AgentKind::HubRis),
            "kalos" => Some(AgentKind::KaLos),
            "neikos" => Some(AgentKind::NeiKos),
            "skemma" => Some(AgentKind::SkeMma),
            "aporia" => Some(AgentKind::ApoRia),
            "eleos" => Some(AgentKind::EleOs),
            "epieikeia" => Some(AgentKind::Epieikeia),
            "orexis" => Some(AgentKind::OreXis),
            "philia" => Some(AgentKind::PhiLia),
            "polemos" => Some(AgentKind::PoleMos),
            "industrial_iot" | "industrialiot" => Some(AgentKind::IndustrialIoT),
            _ => None,
        }
    }

    /// Parse from Layer2 agent name (does not recognize Layer1, returns None).
    pub fn from_layer2_name(name: &str) -> Option<Self> {
        Self::from_folder_name(name).filter(|a| a.is_layer2())
    }

    /// If it is a Layer2 agent, return its folder name; otherwise return None.
    pub fn to_layer2_name(&self) -> Option<&'static str> {
        if self.is_layer2() {
            Some(self.folder_name())
        } else {
            None
        }
    }

    /// Whether this is a Layer2 extension agent.
    pub fn is_layer2(&self) -> bool {
        let folder = self.folder_name();
        if let Some(desc) = super::descriptor::AGENT_REGISTRY.get_by_folder(folder) {
            desc.layer == 2
        } else {
            false
        }
    }

    /// Layer name, for UI display.
    pub fn layer_name(&self) -> &'static str {
        if self.is_layer2() { "Layer2" } else { "Layer1" }
    }

    /// English description (short phrase).
    pub fn description(&self) -> &'static str {
        let folder = self.folder_name();
        if let Some(desc) = super::descriptor::AGENT_REGISTRY.get_by_folder(folder) {
            desc.description
        } else {
            self.description_fallback()
        }
    }

    fn description_fallback(&self) -> &'static str {
        match self {
            AgentKind::HapLotes => "Network communication gateway",
            AgentKind::SkoPeo => "Central coordinator",
            AgentKind::HubRis => "Work planning engine",
            AgentKind::KaLos => "Workflow management",
            AgentKind::NeiKos => "Container management",
            AgentKind::SkeMma => "Script execution and microservice runtime",
            AgentKind::ApoRia => "Storage and LLM hub",
            AgentKind::EleOs => "Security and information acquisition",
            AgentKind::Epieikeia => "Backup and scheduling",
            AgentKind::OreXis => "Security audit and external integration",
            AgentKind::PhiLia => "System integration",
            AgentKind::PoleMos => "Edge computing and device management",
            AgentKind::IndustrialIoT => {
                "Industrial protocol gateway — engine-internal execution identity"
            }
        }
    }

    /// All Layer1 agents (12 total).
    pub fn all() -> Vec<Self> {
        let registry = &*super::descriptor::AGENT_REGISTRY;
        let layer1_folders: Vec<&str> = registry
            .layer1_agents()
            .iter()
            .map(|d| d.folder_name)
            .collect();
        let mut result = Vec::new();
        for folder in layer1_folders {
            if let Some(kind) = Self::from_folder_name(folder) {
                result.push(kind);
            }
        }
        if result.is_empty() {
            return Self::all_fallback();
        }
        result
    }

    fn all_fallback() -> Vec<Self> {
        vec![
            AgentKind::HapLotes,
            AgentKind::SkoPeo,
            AgentKind::HubRis,
            AgentKind::KaLos,
            AgentKind::NeiKos,
            AgentKind::SkeMma,
            AgentKind::ApoRia,
            AgentKind::EleOs,
            AgentKind::Epieikeia,
            AgentKind::OreXis,
            AgentKind::PhiLia,
            AgentKind::PoleMos,
        ]
    }

    /// All Layer2 extension agents — empty since W11-6 slice C retired the 15 widget variants (plugin-carried now); the method stays for the seam.
    pub fn domain_agents() -> Vec<Self> {
        let registry = &*super::descriptor::AGENT_REGISTRY;
        let layer2_folders: Vec<&str> = registry
            .layer2_agents()
            .iter()
            .map(|d| d.folder_name)
            .collect();
        let mut result = Vec::new();
        for folder in layer2_folders {
            if let Some(kind) = Self::from_folder_name(folder) {
                result.push(kind);
            }
        }
        if result.is_empty() {
            return Self::domain_agents_fallback();
        }
        result
    }

    fn domain_agents_fallback() -> Vec<Self> {
        Vec::new()
    }

    /// All 27 agents.
    pub fn all_agents() -> Vec<Self> {
        let mut v = Self::all();
        v.extend(Self::domain_agents());
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn all_agents_has_13_variants() -> anyhow::Result<()> {
        assert_eq!(AgentKind::all_agents().len(), 13);
        Ok(())
    }

    #[test]
    fn layer1_has_12_variants() -> anyhow::Result<()> {
        assert_eq!(AgentKind::all().len(), 12);
        Ok(())
    }

    #[test]
    fn layer2_has_zero_variants_after_retirement() -> anyhow::Result<()> {
        assert_eq!(AgentKind::domain_agents().len(), 0);
        Ok(())
    }

    #[test]
    fn folder_name_roundtrip() -> anyhow::Result<()> {
        for ak in AgentKind::all_agents() {
            let folder = ak.folder_name();
            let parsed = AgentKind::from_folder_name(folder);
            assert_eq!(
                parsed,
                Some(ak),
                "folder_name roundtrip failed for {:?}",
                ak
            );
        }
        Ok(())
    }

    #[test]
    fn from_str_roundtrip() -> Result<()> {
        for ak in AgentKind::all_agents() {
            let name = ak.to_string();
            let parsed: AgentKind = name.parse()?;
            assert_eq!(parsed, ak, "FromStr roundtrip failed for {:?}", ak);
        }
        Ok(())
    }

    #[test]
    fn no_duplicate_folder_names() -> anyhow::Result<()> {
        let agents = AgentKind::all_agents();
        let folders: Vec<&str> = agents.iter().map(|a| a.folder_name()).collect();
        let unique: std::collections::HashSet<&str> = folders.iter().copied().collect();
        assert_eq!(
            folders.len(),
            unique.len(),
            "Duplicate folder names detected"
        );
        Ok(())
    }

    #[test]
    fn unknown_folder_name_returns_none() -> anyhow::Result<()> {
        assert!(AgentKind::from_folder_name("nonexistent").is_none());
        Ok(())
    }
}
