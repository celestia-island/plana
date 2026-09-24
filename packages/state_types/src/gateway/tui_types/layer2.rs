//! Layer-2 marketplace descriptors: extension agents that can be listed and
//! subscribed to, plus the tools and skills they expose with their registry
//! cross-references.
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// One Layer-2 extension agent offered by the marketplace, as listed to the
/// TUI by `Sync.Layer2AgentListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2AgentInfo {
    /// Registry/folder name of the agent, also its id in later layer-2
    /// requests.
    pub name: String,
    /// Human-readable summary shown in the marketplace list.
    pub description: String,
    /// Number of tools the agent exposes.
    pub tool_count: usize,
    /// Number of skills the agent exposes.
    pub skills_count: usize,
    /// Language codes the agent's prompts and tool/skill descriptions exist
    /// in.
    pub languages: Vec<String>,
    /// Whether the agent is currently enabled; an omitted key defaults to
    /// `true` (unlike `YoloTierConfig.enabled`).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// A custom (subscribed) agent installed from an external source: the rows
/// behind `Sync.CustomAgentListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentInfo {
    /// Registry name of the custom agent (its folder name and id).
    pub name: String,
    /// User-facing title taken from the agent manifest.
    pub display_name: String,
    /// Summary from the agent manifest; empty string when the manifest omits
    /// it.
    pub description: String,
    /// Number of skills the agent contributes.
    pub skills_count: usize,
    /// Repository URL the agent was subscribed from.
    pub source: String,
    /// Version declared by the agent manifest; `None` when the source reports
    /// none.
    pub version: Option<String>,
    /// RFC 3339 timestamp of the last install/update; `None` when the agent was
    /// never updated through this path.
    pub last_updated: Option<String>,
}

/// One tool exposed by a Layer-2 agent, together with the registry
/// cross-references the TUI renders next to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2ToolInfo {
    /// Tool name used in agent/tool requests.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// Language codes this tool's prompts are translated into.
    pub languages: Vec<String>,
    /// Names of Layer-1 (built-in domain) items this tool builds on; empty when
    /// the payload omits them.
    #[serde(default)]
    pub references_layer1: Vec<String>,
    /// Names of other Layer-2 items this tool builds on; empty when omitted.
    #[serde(default)]
    pub references_layer2: Vec<String>,
    /// Sibling registry items the TUI surfaces as related; empty when
    /// omitted.
    #[serde(default)]
    pub related_items: Vec<String>,
    /// Names of items that reference this tool — the reverse index of the two
    /// `references_*` lists; empty when omitted.
    #[serde(default)]
    pub referenced_by_items: Vec<String>,
}

/// One skill exposed by a Layer-2 agent; same cross-reference set as
/// `Layer2ToolInfo`, resolved at skill granularity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2SkillInfo {
    /// Skill name used in agent/skill requests.
    pub name: String,
    /// Human-readable description of what the skill does.
    pub description: String,
    /// Language codes this skill's prompts are translated into.
    pub languages: Vec<String>,
    /// Names of Layer-1 (built-in domain) items this skill builds on; empty when
    /// the payload omits them.
    #[serde(default)]
    pub references_layer1: Vec<String>,
    /// Names of other Layer-2 items this skill builds on; empty when omitted.
    #[serde(default)]
    pub references_layer2: Vec<String>,
    /// Sibling registry items the TUI surfaces as related; empty when
    /// omitted.
    #[serde(default)]
    pub related_items: Vec<String>,
    /// Names of items that reference this skill — the reverse index of the two
    /// `references_*` lists; empty when omitted.
    #[serde(default)]
    pub referenced_by_items: Vec<String>,
}
