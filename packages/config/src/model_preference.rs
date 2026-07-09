//! Agent and Skill model preference system.
//!
//! Extends the existing 3-tier (`Deep / Normal / Basic`) model selection with
//! per-agent and per-skill overrides. The priority chain is:
//!
//! 1. **Skill-level, capability-keyed**: when a required capability is given,
//!    try `by_capability[cap]` from the skill preference first
//! 2. **Skill-level preferred**: skill-specific `preferred_models`
//! 3. **Agent-level, capability-keyed**: `by_capability[cap]` from the agent
//! 4. **Agent-level preferred**: agent-specific `preferred_models`
//! 5. **Tier fallback**: `ModelTier::Deep / Normal / Basic` system
//!
//! ## Config format
//!
//! Preferences live in `agent_model_prefs.toml`, separate from
//! `provider_config.toml`:
//!
//! ```toml
//! [[agent_prefs]]
//! agent = "kalos"            # AgentKind folder_name
//! preferred_models = [
//!   { provider_id = "zhipu_glm", model_name = "glm-5.2" },
//! ]
//!
//! [agent_prefs.by_capability]
//! "generate_image" = [
//!   { provider_id = "openai", model_name = "gpt-image-2" },
//! ]
//!
//! [[skill_prefs]]
//! skill = "smart_write_file"
//! preferred_models = [
//!   { provider_id = "anthropic", model_name = "claude-sonnet-4-20250514" },
//! ]
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use _core::ModelTier;

use crate::gen_protocol::Capability;

// ─── QualifiedModelId ──────────────────────────────────────────────

/// A specific model identified by provider + model name.
///
/// Unlike tier-based selection which picks from a pool, this points to one
/// exact model. Multiple entries form a priority-ordered fallback list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QualifiedModelId {
    /// Provider identifier (e.g. "openai", "anthropic", "zhipu_glm").
    pub provider_id: String,
    /// Model name as the provider expects it (e.g. "gpt-image-2").
    pub model_name: String,
}

impl QualifiedModelId {
    pub fn new(provider_id: impl Into<String>, model_name: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            model_name: model_name.into(),
        }
    }

    /// Compact string form: `provider_id/model_name`.
    pub fn qualified(&self) -> String {
        format!("{}/{}", self.provider_id, self.model_name)
    }
}

impl std::fmt::Display for QualifiedModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider_id, self.model_name)
    }
}

impl FromStr for QualifiedModelId {
    type Err = QualifiedModelIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (provider_id, model_name) = s
            .split_once('/')
            .ok_or_else(|| QualifiedModelIdParseError(s.to_string()))?;
        if provider_id.is_empty() || model_name.is_empty() {
            return Err(QualifiedModelIdParseError(s.to_string()));
        }
        Ok(Self {
            provider_id: provider_id.to_string(),
            model_name: model_name.to_string(),
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid qualified model id (expected 'provider/model'): {0}")]
pub struct QualifiedModelIdParseError(pub String);

// ─── ModelPreference ───────────────────────────────────────────────

/// Model preferences for a single agent or skill.
///
/// Fields are checked in priority order:
/// 1. `by_capability` — capability-keyed models (most specific first)
/// 2. `preferred_models` — direct model overrides (checked second)
/// 3. `fallback_tier` — if nothing matches, fall through to tier
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelPreference {
    /// Priority-ordered list of specific model IDs. In `resolve()`, these are
    /// checked after capability-keyed models when a capability is specified.
    #[serde(default)]
    pub preferred_models: Vec<QualifiedModelId>,

    /// Capability-keyed model lists. Keys are capability string IDs
    /// (e.g. `"generate_image"`, `"generate_audio_speech"`).
    #[serde(default)]
    pub by_capability: HashMap<String, Vec<QualifiedModelId>>,

    /// Fallback tier when no preference matches. `None` means use the
    /// caller-supplied default tier.
    #[serde(default)]
    pub fallback_tier: Option<ModelTier>,
}

impl ModelPreference {
    /// Create an empty preference that always falls through to tier selection.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a preference with only preferred models.
    pub fn with_models(models: Vec<QualifiedModelId>) -> Self {
        Self {
            preferred_models: models,
            ..Default::default()
        }
    }

    /// Resolve models for a given capability. Returns only capability-specific
    /// models; does NOT fall back to `preferred_models`. Callers that need
    /// fallback should also check `preferred_models` separately.
    pub fn models_for_capability(&self, cap: Capability) -> &[QualifiedModelId] {
        let key = cap.as_str();
        self.by_capability.get(key).map_or(&[], |v| v.as_slice())
    }

    /// Whether this preference has any non-default configuration.
    pub fn is_configured(&self) -> bool {
        !self.preferred_models.is_empty()
            || !self.by_capability.is_empty()
            || self.fallback_tier.is_some()
    }

    /// Add a capability-keyed model preference.
    pub fn with_capability(mut self, cap: Capability, models: Vec<QualifiedModelId>) -> Self {
        self.by_capability.insert(cap.as_str().to_string(), models);
        self
    }
}

// ─── AgentModelPreferenceEntry / SkillModelPreferenceEntry ─────────

/// A single agent's preference entry in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelPreferenceEntry {
    /// AgentKind folder name (e.g. "kalos", "hubris", "aporia").
    pub agent: String,
    #[serde(flatten)]
    pub preference: ModelPreference,
}

/// A single skill's preference entry in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillModelPreferenceEntry {
    /// Skill name (e.g. "smart_write_file", "remote_deploy_amphoreus").
    pub skill: String,
    #[serde(flatten)]
    pub preference: ModelPreference,
}

// ─── AgentModelPreferences ─────────────────────────────────────────

/// Complete model preference configuration: per-agent + per-skill overrides.
///
/// Loaded from `agent_model_prefs.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentModelPreferences {
    /// Per-agent preferences.
    #[serde(default)]
    pub agent_prefs: Vec<AgentModelPreferenceEntry>,

    /// Per-skill preferences.
    #[serde(default)]
    pub skill_prefs: Vec<SkillModelPreferenceEntry>,
}

impl AgentModelPreferences {
    /// Look up preferences for a specific agent by folder name.
    pub fn for_agent(&self, agent_folder: &str) -> Option<&ModelPreference> {
        self.agent_prefs
            .iter()
            .find(|e| e.agent.eq_ignore_ascii_case(agent_folder))
            .map(|e| &e.preference)
    }

    /// Look up preferences for a specific skill.
    pub fn for_skill(&self, skill_name: &str) -> Option<&ModelPreference> {
        self.skill_prefs
            .iter()
            .find(|e| e.skill.eq_ignore_ascii_case(skill_name))
            .map(|e| &e.preference)
    }

    /// Resolve the full model selection chain for a (agent, skill, capability)
    /// tuple.
    ///
    /// Returns the ordered list of qualified model IDs to try, and whether a
    /// tier fallback should be used if all fail.
    pub fn resolve(
        &self,
        agent_folder: Option<&str>,
        skill_name: Option<&str>,
        required_capability: Option<&Capability>,
    ) -> ResolvedPreference {
        // Track fallback_tier from the most-specific matched preference
        // (skill takes priority over agent).
        let mut matched_fallback_tier: Option<ModelTier> = None;

        // 1. Skill-level preferred models (skill prefs are most specific)
        if let Some(skill) = skill_name
            && let Some(pref) = self.for_skill(skill)
        {
            matched_fallback_tier = pref.fallback_tier;
            if let Some(cap) = required_capability {
                let models = pref.models_for_capability(*cap);
                if !models.is_empty() {
                    return ResolvedPreference::from_models(models, pref.fallback_tier);
                }
            }
            if !pref.preferred_models.is_empty() {
                return ResolvedPreference::from_models(&pref.preferred_models, pref.fallback_tier);
            }
        }

        // 2. Agent-level preferred models
        if let Some(agent) = agent_folder
            && let Some(pref) = self.for_agent(agent)
        {
            // Only override fallback_tier if skill didn't set it
            if matched_fallback_tier.is_none() {
                matched_fallback_tier = pref.fallback_tier;
            }
            let effective_tier = matched_fallback_tier.or(pref.fallback_tier);
            if let Some(cap) = required_capability {
                let models = pref.models_for_capability(*cap);
                if !models.is_empty() {
                    return ResolvedPreference::from_models(models, effective_tier);
                }
            }
            if !pref.preferred_models.is_empty() {
                return ResolvedPreference::from_models(&pref.preferred_models, effective_tier);
            }
        }

        // 3. No specific models matched — propagate matched fallback_tier
        ResolvedPreference::tier_fallback(matched_fallback_tier)
    }
}

// ─── ResolvedPreference ────────────────────────────────────────────

/// The result of resolving the preference chain — either a list of specific
/// models to try, or a directive to use tier-based selection.
#[derive(Debug, Clone)]
pub struct ResolvedPreference {
    /// Ordered model IDs to try. Empty means "use tier fallback".
    pub models: Vec<QualifiedModelId>,
    /// Fallback tier to use if all models in `models` fail.
    pub fallback_tier: Option<ModelTier>,
}

impl ResolvedPreference {
    fn from_models(models: &[QualifiedModelId], fallback_tier: Option<ModelTier>) -> Self {
        Self {
            models: models.to_vec(),
            fallback_tier,
        }
    }

    fn tier_fallback(tier: Option<ModelTier>) -> Self {
        Self {
            models: Vec::new(),
            fallback_tier: tier,
        }
    }

    /// Whether the caller should skip tier-based selection entirely.
    pub fn use_specific_models(&self) -> bool {
        !self.models.is_empty()
    }
}

// ─── Config loading with mtime cache ───────────────────────────────

const PREFS_FILENAME: &str = "agent_model_prefs.toml";

/// Return the priority-ordered candidate paths for `agent_model_prefs.toml`.
///
/// Layer priority (first wins):
///   [1] ENV: `ENTELECHEIA_PREFS_PATH` (explicit file)
///   [2] ENV: `ENTELECHEIA_CONFIG_DIR/agent_model_prefs.toml`
///   [3] GLOBAL: `UserConfig::config_dir()/agent_model_prefs.toml`
fn prefs_config_chain() -> Vec<std::path::PathBuf> {
    let mut chain: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(p) = std::env::var("ENTELECHEIA_PREFS_PATH") {
        chain.push(std::path::PathBuf::from(p));
    }
    if let Ok(dir) = std::env::var("ENTELECHEIA_CONFIG_DIR") {
        chain.push(std::path::PathBuf::from(&dir).join(PREFS_FILENAME));
    }
    chain.push(crate::app_config::UserConfig::config_dir().join(PREFS_FILENAME));
    chain
}

/// Find the first existing `agent_model_prefs.toml` on the config chain.
pub fn find_prefs_config_path() -> Option<std::path::PathBuf> {
    prefs_config_chain().into_iter().find(|p| p.exists())
}

// ── mtime-based cache to avoid re-reading on every chat message ──

use std::sync::OnceLock;

struct PrefsCache {
    data: AgentModelPreferences,
    mtime: Option<std::time::SystemTime>,
    path: Option<std::path::PathBuf>,
}

static PREFS_CACHE: OnceLock<parking_lot::RwLock<PrefsCache>> = OnceLock::new();

fn prefs_cache() -> &'static parking_lot::RwLock<PrefsCache> {
    PREFS_CACHE.get_or_init(|| {
        parking_lot::RwLock::new(PrefsCache {
            data: AgentModelPreferences::default(),
            mtime: None,
            path: None,
        })
    })
}

/// Load `agent_model_prefs.toml` from disk, with mtime-based caching.
///
/// On the first call, reads and parses the file. On subsequent calls, checks
/// the file's mtime — if unchanged, returns the cached parsed data without
/// re-reading. If the file is missing or invalid, returns empty preferences.
///
/// This is safe to call from async context — the blocking I/O only happens
/// on cache miss (file changed or first load), and `stat()` is very fast.
pub fn load_agent_model_preferences() -> AgentModelPreferences {
    let path = match find_prefs_config_path() {
        Some(p) => p,
        None => return AgentModelPreferences::default(),
    };

    let current_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

    // Fast path: check cache under read lock
    {
        let cache = prefs_cache().read();
        if cache.path.as_ref() == Some(&path) && cache.mtime == current_mtime {
            return cache.data.clone();
        }
    }
    // Read lock released here — I/O happens without any lock held

    // Cache miss — read and parse (I/O done before acquiring write lock)
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to read agent_model_prefs.toml: {}", e);
            // Don't cache the error — retry on next call
            return AgentModelPreferences::default();
        }
    };

    let prefs = match toml::from_str::<AgentModelPreferences>(&content) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("failed to parse agent_model_prefs.toml: {}", e);
            // Don't cache the parse error — retry on next call
            return AgentModelPreferences::default();
        }
    };

    tracing::info!(
        "loaded agent_model_prefs.toml: {} agent prefs, {} skill prefs (from {:?})",
        prefs.agent_prefs.len(),
        prefs.skill_prefs.len(),
        path
    );

    // Acquire write lock. Re-check mtime to avoid TOCTOU: another thread may
    // have loaded the same (or newer) data since we released the read lock.
    let mut cache = prefs_cache().write();
    let fresh_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
    // If another thread already loaded this mtime, skip the update
    if cache.path.as_ref() == Some(&path) && cache.mtime == fresh_mtime {
        // Already cached by another thread — return that thread's result
        return cache.data.clone();
    }

    cache.data = prefs.clone();
    cache.mtime = fresh_mtime;
    cache.path = Some(path);

    prefs
}

#[cfg(test)]
mod tests {
    use super::*;
    use _core::ModelTier;

    #[test]
    fn qualified_model_id_parse() {
        let q: QualifiedModelId = "openai/gpt-image-2".parse().unwrap();
        assert_eq!(q.provider_id, "openai");
        assert_eq!(q.model_name, "gpt-image-2");
        assert_eq!(q.to_string(), "openai/gpt-image-2");
    }

    #[test]
    fn qualified_model_id_rejects_missing_slash() {
        assert!("no-slash".parse::<QualifiedModelId>().is_err());
        assert!("/no-provider".parse::<QualifiedModelId>().is_err());
        assert!("no-model/".parse::<QualifiedModelId>().is_err());
    }

    #[test]
    fn model_preference_empty_falls_through() {
        let pref = ModelPreference::empty();
        assert!(!pref.is_configured());
        let resolved =
            AgentModelPreferences::default().resolve(Some("kalos"), Some("smart_write_file"), None);
        assert!(!resolved.use_specific_models());
    }

    #[test]
    fn agent_preferred_models_direct() {
        let mut prefs = AgentModelPreferences::default();
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "kalos".into(),
            preference: ModelPreference::with_models(vec![QualifiedModelId::new(
                "anthropic",
                "claude-sonnet-4-20250514",
            )]),
        });

        let resolved = prefs.resolve(Some("kalos"), None, None);
        assert!(resolved.use_specific_models());
        assert_eq!(resolved.models.len(), 1);
        assert_eq!(resolved.models[0].model_name, "claude-sonnet-4-20250514");
    }

    #[test]
    fn skill_prefs_override_agent_prefs() {
        let mut prefs = AgentModelPreferences::default();
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "hubris".into(),
            preference: ModelPreference::with_models(vec![QualifiedModelId::new(
                "openai", "gpt-4o",
            )]),
        });
        prefs.skill_prefs.push(SkillModelPreferenceEntry {
            skill: "smart_write_file".into(),
            preference: ModelPreference::with_models(vec![QualifiedModelId::new(
                "anthropic",
                "claude-opus-4-20250514",
            )]),
        });

        let resolved = prefs.resolve(Some("hubris"), Some("smart_write_file"), None);
        assert!(resolved.use_specific_models());
        assert_eq!(resolved.models[0].model_name, "claude-opus-4-20250514");
    }

    #[test]
    fn capability_match_finds_image_model() {
        let mut prefs = AgentModelPreferences::default();
        let mut pref = ModelPreference::empty();
        pref.by_capability.insert(
            "generate_image".into(),
            vec![QualifiedModelId::new("openai", "gpt-image-2")],
        );
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "kalos".into(),
            preference: pref,
        });

        let resolved = prefs.resolve(Some("kalos"), None, Some(&Capability::GenerateImage));
        assert!(resolved.use_specific_models());
        assert_eq!(resolved.models[0].model_name, "gpt-image-2");
    }

    #[test]
    fn fallback_tier_propagated() {
        let mut prefs = AgentModelPreferences::default();
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "aporia".into(),
            preference: ModelPreference {
                preferred_models: vec![QualifiedModelId::new("test", "model")],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Deep),
            },
        });

        let resolved = prefs.resolve(Some("aporia"), None, None);
        assert_eq!(resolved.fallback_tier, Some(ModelTier::Deep));
    }

    #[test]
    fn no_match_returns_tier_fallback() {
        let prefs = AgentModelPreferences::default();
        let resolved = prefs.resolve(Some("nonexistent_agent"), None, None);
        assert!(!resolved.use_specific_models());
        assert!(resolved.models.is_empty());
    }

    #[test]
    fn toml_roundtrip() {
        let toml_str = r#"
[[agent_prefs]]
agent = "kalos"
preferred_models = [
  { provider_id = "anthropic", model_name = "claude-sonnet-4-20250514" },
]

[agent_prefs.by_capability]
"generate_image" = [
  { provider_id = "openai", model_name = "gpt-image-2" },
]

[[skill_prefs]]
skill = "smart_write_file"
preferred_models = [
  { provider_id = "anthropic", model_name = "claude-opus-4-20250514" },
]
fallback_tier = "Deep"
"#;
        let prefs: AgentModelPreferences =
            toml::from_str(toml_str).expect("failed to parse agent_model_prefs.toml");

        assert_eq!(prefs.agent_prefs.len(), 1);
        assert_eq!(prefs.skill_prefs.len(), 1);

        let kalos = prefs.for_agent("kalos").expect("kalos prefs missing");
        assert_eq!(kalos.preferred_models.len(), 1);
        assert_eq!(
            kalos.preferred_models[0].model_name,
            "claude-sonnet-4-20250514"
        );

        let img_models = kalos
            .by_capability
            .get("generate_image")
            .expect("generate_image capability missing");
        assert_eq!(img_models[0].model_name, "gpt-image-2");

        let skill = prefs
            .for_skill("smart_write_file")
            .expect("skill prefs missing");
        assert_eq!(skill.fallback_tier, Some(ModelTier::Deep));
    }

    #[test]
    fn fallback_tier_propagated_when_no_models_match() {
        // Skill has fallback_tier = "Basic" but no preferred_models
        let mut prefs = AgentModelPreferences::default();
        prefs.skill_prefs.push(SkillModelPreferenceEntry {
            skill: "my_skill".into(),
            preference: ModelPreference {
                preferred_models: vec![],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Basic),
            },
        });

        let resolved = prefs.resolve(None, Some("my_skill"), None);
        // No specific models, but fallback_tier should propagate
        assert!(!resolved.use_specific_models());
        assert_eq!(
            resolved.fallback_tier,
            Some(ModelTier::Basic),
            "fallback_tier from matched skill preference must propagate"
        );
    }

    #[test]
    fn skill_fallback_tier_takes_priority_over_agent() {
        let mut prefs = AgentModelPreferences::default();
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "kalos".into(),
            preference: ModelPreference {
                preferred_models: vec![],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Normal),
            },
        });
        prefs.skill_prefs.push(SkillModelPreferenceEntry {
            skill: "my_skill".into(),
            preference: ModelPreference {
                preferred_models: vec![],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Deep),
            },
        });

        let resolved = prefs.resolve(Some("kalos"), Some("my_skill"), None);
        assert_eq!(
            resolved.fallback_tier,
            Some(ModelTier::Deep),
            "skill fallback_tier should take priority over agent"
        );
    }

    #[test]
    fn skill_fallback_tier_wins_when_agent_has_matching_models() {
        let mut prefs = AgentModelPreferences::default();
        prefs.agent_prefs.push(AgentModelPreferenceEntry {
            agent: "kalos".into(),
            preference: ModelPreference {
                preferred_models: vec![QualifiedModelId::new("openai", "gpt-4o")],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Basic),
            },
        });
        prefs.skill_prefs.push(SkillModelPreferenceEntry {
            skill: "my_skill".into(),
            preference: ModelPreference {
                preferred_models: vec![],
                by_capability: HashMap::new(),
                fallback_tier: Some(ModelTier::Deep),
            },
        });

        let resolved = prefs.resolve(Some("kalos"), Some("my_skill"), None);
        // Agent has preferred_models, skill has no models but sets fallback_tier=Deep
        assert!(
            resolved.use_specific_models(),
            "agent models should be returned"
        );
        assert_eq!(
            resolved.fallback_tier,
            Some(ModelTier::Deep),
            "skill's fallback_tier should win over agent's even when agent has models"
        );
    }
}
