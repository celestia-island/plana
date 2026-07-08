use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use dashmap::DashMap;

#[derive(Debug, Clone, Serialize, Deserialize, arona_macros::Getters)]
pub struct AgentDescriptor {
    pub friendly_name: &'static str,
    pub folder_name: &'static str,
    pub description: &'static str,
    pub layer: u8,
    pub containerized: bool,
}

pub static AGENT_REGISTRY: LazyLock<AgentMetadataRegistry> = LazyLock::new(|| {
    let registry = AgentMetadataRegistry::new();
    register_builtins(&registry);
    registry
});

pub struct AgentMetadataRegistry {
    descriptors: DashMap<String, AgentDescriptor>,
}

impl Default for AgentMetadataRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentMetadataRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: DashMap::new(),
        }
    }

    pub fn register(&self, descriptor: AgentDescriptor) {
        self.descriptors
            .insert(descriptor.folder_name.to_string(), descriptor);
    }

    pub fn get_by_folder(&self, name: &str) -> Option<AgentDescriptor> {
        self.descriptors
            .get(&name.to_lowercase())
            .map(|r| r.value().clone())
    }

    pub fn layer1_agents(&self) -> Vec<AgentDescriptor> {
        self.descriptors
            .iter()
            .filter(|r| r.layer == 1)
            .map(|r| r.value().clone())
            .collect()
    }

    pub fn layer2_agents(&self) -> Vec<AgentDescriptor> {
        self.descriptors
            .iter()
            .filter(|r| r.layer == 2)
            .map(|r| r.value().clone())
            .collect()
    }

    pub fn all_agents(&self) -> Vec<AgentDescriptor> {
        self.descriptors.iter().map(|r| r.value().clone()).collect()
    }
}

macro_rules! builtin_agents {
    ($registry:expr, { $($friendly:literal / $folder:literal / $desc:literal / $layer:literal / $containerized:literal),* $(,)? }) => {
        $(
            $registry.register(AgentDescriptor {
                friendly_name: $friendly,
                folder_name: $folder,
                description: $desc,
                layer: $layer,
                containerized: $containerized,
            });
        )*
    };
}

fn register_builtins(registry: &AgentMetadataRegistry) {
    builtin_agents!(registry, {
        "HapLotes"   / "haplotes"   / "Communication gateway"                    / 1 / false,
        "SkoPeo"     / "skopeo"     / "Central coordinator"                      / 1 / true,
        "HubRis"     / "hubris"      / "Work planning engine"                     / 1 / false,
        "KaLos"      / "kalos"       / "File operations"                          / 1 / true,
        "NeiKos"     / "neikos"      / "Container management"                     / 1 / true,
        "SkeMma"     / "skemma"      / "Script execution"                         / 1 / true,
        "ApoRia"     / "aporia"      / "Knowledge management"                     / 1 / false,
        "EleOs"      / "eleos"       / "Web search"                               / 1 / true,
        "EpieiKeia"  / "epieikeia"   / "Diagnostics"                              / 1 / true,
        "OreXis"     / "orexis"      / "Security"                                 / 1 / true,
        "PhiLia"     / "philia"      / "Data storage"                             / 1 / true,
        "PoleMos"    / "polemos"     / "Hardware & vision"                        / 1 / true,
        "Web Automation" / "web_automation" / "Web automation and browser testing" / 2 / false,
        "Classic Software Engineering" / "classic_software_engineering" / "Code review, LSP integration, and refactoring" / 2 / false,
        "WebUI Panel" / "web_ui_panel" / "Pluggable dashboard views — SCADA, kanban, media flow, data tables" / 2 / false,
        "Industrial IoT" / "industrial_iot" / "Industrial IoT — PLC communication, sensor polling, alarm management" / 2 / false,
        "Remote Operations" / "remote_operations" / "Remote operations — SSH, remote terminal, file transfer" / 2 / false
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn test_registry_has_all_17_agents() -> Result<()> {
        let registry = &*AGENT_REGISTRY;
        let all = registry.all_agents();
        assert_eq!(all.len(), 17);
        Ok(())
    }

    #[test]
    fn test_layer1_has_12_agents() -> Result<()> {
        let registry = &*AGENT_REGISTRY;
        let layer1 = registry.layer1_agents();
        assert_eq!(layer1.len(), 12);
        Ok(())
    }

    #[test]
    fn test_layer2_has_5_agents() -> Result<()> {
        let registry = &*AGENT_REGISTRY;
        let layer2 = registry.layer2_agents();
        assert_eq!(layer2.len(), 5);
        Ok(())
    }

    #[test]
    fn test_lookup_by_folder() -> Result<()> {
        let registry = &*AGENT_REGISTRY;
        let hubris = registry
            .get_by_folder("hubris")
            .context("missing hubris descriptor")?;
        assert_eq!(hubris.friendly_name, "HubRis");
        assert_eq!(hubris.layer, 1);
        Ok(())
    }

    #[test]
    fn test_containerized_flags() -> Result<()> {
        let registry = &*AGENT_REGISTRY;
        let neikos = registry
            .get_by_folder("neikos")
            .context("missing neikos descriptor")?;
        assert!(neikos.containerized);
        let hubris = registry
            .get_by_folder("hubris")
            .context("missing hubris descriptor")?;
        assert!(!hubris.containerized);
        Ok(())
    }
}
