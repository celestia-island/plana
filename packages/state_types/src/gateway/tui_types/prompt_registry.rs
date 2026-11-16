//! Prompt-registry wire types (P5): one enumerable surface over EVERY
//! prompt the engine can serve — builtin souls, system fragments, and the
//! tools/skills of every registered plugin agent (the 2026-11-16 ruling:
//! a plugin may read every registered plugin's prompts plus the twelve
//! builtin souls, and create agents from that material — zero copies).
//!
//! The list carries metadata only (`body_chars` + `sha256` of the raw
//! document, front matter included) so consumers can cache by
//! `(agent_key, name, lang, sha256)` and only `Get` what changed. `Get`
//! requires an explicit `lang` — a locale outside the item's `langs` is a
//! named refusal server-side, never a silent fallback (the house
//! "never guessed" discipline).

use serde::{Deserialize, Serialize};

/// Which shelf of the prompt registry an item sits on. Mirrors the
/// five-level inventory the operator console already serves, opened to
/// plugin-carried agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptLevel {
    System,
    SystemFragment,
    Soul,
    Skill,
    Tool,
}

impl PromptLevel {
    /// Lowercase wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            PromptLevel::System => "system",
            PromptLevel::SystemFragment => "system_fragment",
            PromptLevel::Soul => "soul",
            PromptLevel::Skill => "skill",
            PromptLevel::Tool => "tool",
        }
    }
}

/// One addressable prompt document in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptRegistryItem {
    /// Which shelf: soul / skill / tool / system / system_fragment.
    pub level: PromptLevel,
    /// Owning agent — a bare builtin folder (`"hubris"`), a plugin wire
    /// key (`"plugin:<id>:<folder>"`), or the engine itself for the
    /// system levels.
    pub agent_key: String,
    /// Convenience spelling of the folder segment (== `agent_key` for
    /// builtin folders and system items).
    pub folder: String,
    /// Document name (the doc stem for skills/tools, the soul name for
    /// souls).
    pub name: String,
    /// Locales the document actually exists in (today the trees are
    /// flat single-locale; the wire carries the axis anyway).
    pub langs: Vec<String>,
    /// SHA-256 of the RAW document — front matter included — so a
    /// front-matter-only edit still busts caches.
    pub sha256: String,
    /// Body length in characters (metadata for list views; the body
    /// itself only travels on `PromptRegistryGet`).
    pub body_chars: usize,
}

/// The document a `PromptRegistryGet` asked for, with the locale that
/// was actually served.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptRegistryDoc {
    /// The addressed item's metadata (hash/length as served).
    pub item: PromptRegistryItem,
    /// The locale served — always echoes a member of `item.langs`; the
    /// server refuses a requested locale outside that list by name.
    pub lang: String,
    /// The full raw document (front matter + body).
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_wire_spellings() {
        for (level, wire) in [
            (PromptLevel::System, "system"),
            (PromptLevel::SystemFragment, "system_fragment"),
            (PromptLevel::Soul, "soul"),
            (PromptLevel::Skill, "skill"),
            (PromptLevel::Tool, "tool"),
        ] {
            assert_eq!(
                serde_json::to_string(&level).unwrap(),
                format!("\"{wire}\"")
            );
            assert_eq!(level.as_str(), wire);
        }
        assert!(serde_json::from_str::<PromptLevel>("\"nope\"").is_err());
    }

    #[test]
    fn item_round_trips_with_plugin_agent_key() {
        let item = PromptRegistryItem {
            level: PromptLevel::Soul,
            agent_key: "plugin:celestia-wishwell:wishwell".to_string(),
            folder: "wishwell".to_string(),
            name: "mayor_charter".to_string(),
            langs: vec!["en".to_string()],
            sha256: "a".repeat(64),
            body_chars: 1337,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: PromptRegistryItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back, item);
        assert!(json.contains("\"plugin:celestia-wishwell:wishwell\""));
        assert!(json.contains("\"soul\""));
    }

    #[test]
    fn doc_round_trips() {
        let doc = PromptRegistryDoc {
            item: PromptRegistryItem {
                level: PromptLevel::Tool,
                agent_key: "hubris".to_string(),
                folder: "hubris".to_string(),
                name: "report_human".to_string(),
                langs: vec!["en".to_string()],
                sha256: "b".repeat(64),
                body_chars: 10,
            },
            lang: "en".to_string(),
            body: "+++\nname = \"report_human\"\n+++\n\nbody".to_string(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(
            serde_json::from_str::<PromptRegistryDoc>(&json).unwrap(),
            doc
        );
    }
}
