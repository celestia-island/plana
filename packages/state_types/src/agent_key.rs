//! The runtime-extensible agent identity: `AgentKey`.
//!
//! The 2026-10-21 L2 plugin-platform ruling (PLAN §2.5): a plugin-declared
//! L2 agent is **never** a hardcoded enum variant — the static [`Agent`]
//! enum keeps only the built-ins, and everything a plugin carries is
//! addressed at runtime as `AgentKey::Plugin { plugin_id, folder }`.
//!
//! ## Wire form (one string, both ways)
//!
//! - Builtin: the agent's folder name (`haplotes`, `industrial_iot`) —
//!   the same string [`Agent`]'s `Display` already yields, so existing
//!   string-typed persistence (`agent_type` columns, session headers)
//!   accepts builtin keys unchanged.
//! - Plugin: `plugin:<plugin_id>:<folder>` (e.g.
//!   `plugin:acme-office:office`). The `plugin:` prefix makes the two
//!   forms unambiguous — plugin ids and folders cannot contain `:`.
//!
//! `Serialize` emits the wire string; `Deserialize` parses via
//! [`FromStr`], which accepts both spellings for builtins (variant id or
//! folder name — the same looseness [`Agent`] already allows).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::Agent;

/// The prefix marking a plugin-carried agent on the wire.
pub const PLUGIN_PREFIX: &str = "plugin:";

/// Upper bound for the plugin id segment (matches the akivili manifest
/// plugin-id rule).
const PLUGIN_ID_MAX: usize = 64;
/// Upper bound for the folder segment (matches the akivili
/// `[[agents]]` folder rule).
const FOLDER_MAX: usize = 48;

/// A runtime agent identity: one of the built-in agents, or an agent a
/// plugin declares.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AgentKey {
    /// One of the compiled-in agents (Layer 1, the widget L2s, the
    /// engine-internal identities).
    Builtin(Agent),
    /// An agent a plugin declares: which plugin carries it, and the
    /// roster folder it owns.
    Plugin { plugin_id: String, folder: String },
}

/// The shape errors [`AgentKey::new_plugin`] rejects loudly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidAgentKey(pub String);

impl fmt::Display for InvalidAgentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid agent key: {}", self.0)
    }
}

impl std::error::Error for InvalidAgentKey {}

/// `^[a-z0-9][a-z0-9-]*$` — the akivili plugin-id shape.
fn is_valid_plugin_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= PLUGIN_ID_MAX
        && id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `^[a-z][a-z0-9_]*$` — the agent-folder shape (`AgentKind::
/// folder_name` vocabulary; underscores, never hyphens).
fn is_valid_agent_folder(folder: &str) -> bool {
    !folder.is_empty()
        && folder.len() <= FOLDER_MAX
        && folder
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
        && folder
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl AgentKey {
    /// Build a plugin-carried key, validating both segments' shapes
    /// (they become wire strings and path-adjacent identifiers, so a
    /// bad shape is rejected here instead of being laundered later).
    pub fn new_plugin(
        plugin_id: impl Into<String>,
        folder: impl Into<String>,
    ) -> Result<Self, InvalidAgentKey> {
        let plugin_id = plugin_id.into();
        let folder = folder.into();
        if !is_valid_plugin_id(&plugin_id) {
            return Err(InvalidAgentKey(format!(
                "plugin id '{plugin_id}' must match ^[a-z0-9][a-z0-9-]*$ (<={PLUGIN_ID_MAX} chars)"
            )));
        }
        if !is_valid_agent_folder(&folder) {
            return Err(InvalidAgentKey(format!(
                "folder '{folder}' must match ^[a-z][a-z0-9_]*$ (<={FOLDER_MAX} chars)"
            )));
        }
        Ok(Self::Plugin { plugin_id, folder })
    }

    /// The roster folder this key owns (the prompt/tool path segment).
    pub fn folder_name(&self) -> &str {
        match self {
            Self::Builtin(agent) => agent.folder_name(),
            Self::Plugin { folder, .. } => folder,
        }
    }

    /// The human-readable label (the roster's fallback display name).
    pub fn display_name(&self) -> &str {
        match self {
            Self::Builtin(agent) => agent.display_name(),
            Self::Plugin { folder, .. } => folder,
        }
    }

    /// The carrying plugin's id, or `None` for a built-in agent.
    pub fn plugin_id(&self) -> Option<&str> {
        match self {
            Self::Builtin(_) => None,
            Self::Plugin { plugin_id, .. } => Some(plugin_id),
        }
    }

    /// Whether a plugin carries this agent.
    pub fn is_plugin(&self) -> bool {
        matches!(self, Self::Plugin { .. })
    }

    /// The built-in agent, when this key is one.
    pub fn as_builtin(&self) -> Option<&Agent> {
        match self {
            Self::Builtin(agent) => Some(agent),
            Self::Plugin { .. } => None,
        }
    }

    /// Consume into the built-in agent, when this key is one.
    pub fn into_builtin(self) -> Option<Agent> {
        match self {
            Self::Builtin(agent) => Some(agent),
            Self::Plugin { .. } => None,
        }
    }

    /// The wire string (what `Display`, `Serialize` and the string
    /// persistence paths all use).
    pub fn as_wire_str(&self) -> String {
        self.to_string()
    }
}

impl From<Agent> for AgentKey {
    fn from(agent: Agent) -> Self {
        Self::Builtin(agent)
    }
}

impl From<&Agent> for AgentKey {
    fn from(agent: &Agent) -> Self {
        Self::Builtin(agent.clone())
    }
}

impl fmt::Display for AgentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin(agent) => write!(f, "{agent}"),
            Self::Plugin { plugin_id, folder } => {
                write!(f, "{PLUGIN_PREFIX}{plugin_id}:{folder}")
            }
        }
    }
}

impl FromStr for AgentKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix(PLUGIN_PREFIX) {
            let (plugin_id, folder) = rest.split_once(':').ok_or_else(|| {
                format!("malformed plugin agent key '{s}' (want plugin:<plugin-id>:<folder>)")
            })?;
            return Self::new_plugin(plugin_id, folder)
                .map_err(|e| format!("malformed plugin agent key '{s}': {e}"));
        }
        s.parse::<Agent>()
            .map(Self::Builtin)
            .map_err(|_| format!("unknown agent key: {s}"))
    }
}

impl Serialize for AgentKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for AgentKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::from_str(&raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_keys_round_trip_through_the_folder_spelling() {
        let key: AgentKey = "haplotes".parse().unwrap();
        assert_eq!(key, AgentKey::Builtin(Agent::HapLotes));
        assert_eq!(key.folder_name(), "haplotes");
        assert_eq!(key.to_string(), "haplotes");
        assert!(key.plugin_id().is_none());
        assert!(!key.is_plugin());
        // The variant spelling parses to the same key.
        assert_eq!("HapLotes".parse::<AgentKey>().unwrap(), key);
    }

    #[test]
    fn plugin_keys_round_trip_through_the_wire_form() {
        let key = AgentKey::new_plugin("acme-office", "office").unwrap();
        assert_eq!(key.to_string(), "plugin:acme-office:office");
        assert_eq!(key.folder_name(), "office");
        assert_eq!(key.plugin_id(), Some("acme-office"));
        assert!(key.is_plugin());
        assert_eq!(
            "plugin:acme-office:office".parse::<AgentKey>().unwrap(),
            key
        );
    }

    #[test]
    fn serde_uses_the_wire_string_both_ways() {
        let key = AgentKey::new_plugin("acme-office", "office").unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, "\"plugin:acme-office:office\"");
        let back: AgentKey = serde_json::from_str(&json).unwrap();
        assert_eq!(back, key);

        let builtin = AgentKey::Builtin(Agent::SkoPeo);
        assert_eq!(serde_json::to_string(&builtin).unwrap(), "\"skopeo\"");
        let back: AgentKey =
            serde_json::from_str("\"SkoPeo\"").expect("variant spelling deserializes");
        assert_eq!(back, builtin);
    }

    #[test]
    fn new_plugin_rejects_bad_shapes_loudly() {
        for bad_id in ["", "-x", "Acme", "a b", &"a".repeat(65)] {
            assert!(
                AgentKey::new_plugin(bad_id, "office").is_err(),
                "id {bad_id:?}"
            );
        }
        for bad_folder in ["", "Office", "dev-ops", "1x", &"x".repeat(49)] {
            assert!(
                AgentKey::new_plugin("acme", bad_folder).is_err(),
                "folder {bad_folder:?}"
            );
        }
    }

    #[test]
    fn parsing_rejects_malformed_plugin_forms_and_unknown_agents() {
        // Missing folder segment.
        assert!("plugin:acme".parse::<AgentKey>().is_err());
        // Empty segments.
        assert!("plugin::office".parse::<AgentKey>().is_err());
        assert!("plugin:acme:".parse::<AgentKey>().is_err());
        // Folder with a hyphen (underscore vocabulary only).
        assert!("plugin:acme:dev-ops".parse::<AgentKey>().is_err());
        // Unknown builtin.
        assert!("no_such_agent".parse::<AgentKey>().is_err());
    }

    #[test]
    fn plugin_prefix_is_case_sensitive_and_extra_colons_do_not_shift() {
        // Mixed-case prefixes must not take the plugin path silently (the
        // prefix match is exact; the builtin fallback rejects the colons).
        assert!("Plugin:acme:office".parse::<AgentKey>().is_err());
        assert!("PLUGIN:acme:office".parse::<AgentKey>().is_err());
        // A colon leaking into the folder segment fails shape validation
        // instead of splitting into a wrong (id, folder) pair.
        assert!("plugin:acme:office:extra".parse::<AgentKey>().is_err());
    }

    #[test]
    fn every_builtin_wire_string_parses_back_to_the_same_key() {
        // Every folder spelling — including the underscore ones
        // (`data_table`, `industrial_iot`) — survives a wire round trip.
        for agent in Agent::all() {
            let key = AgentKey::Builtin(agent.clone());
            let wire = key.as_wire_str();
            let parsed: AgentKey = wire.parse().unwrap_or_else(|e| panic!("{e}: {wire}"));
            assert_eq!(parsed, key);
        }
    }

    #[test]
    fn builtin_wire_form_stays_compatible_with_agent_display() {
        // The invariant existing string persistence relies on: a
        // builtin key's wire string equals the agent's own Display.
        for agent in Agent::all() {
            let key = AgentKey::Builtin(agent.clone());
            assert_eq!(key.as_wire_str(), agent.to_string());
        }
    }

    #[test]
    fn conversions_cover_the_builtin_side() {
        let key: AgentKey = Agent::OreXis.into();
        assert_eq!(key.as_builtin(), Some(&Agent::OreXis));
        assert_eq!(key.into_builtin(), Some(Agent::OreXis));
        let plugin = AgentKey::new_plugin("acme", "office").unwrap();
        assert!(plugin.as_builtin().is_none());
        assert!(plugin.into_builtin().is_none());
    }
}
