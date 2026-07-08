use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use tracing::{debug, info, trace, warn};

use super::model_category::{GenerationParams, ModelCategory};
use arona_core::{ModelTier, is_invalid_api_key};

const TIER_NORMAL: &str = "normal";

#[derive(Debug, Clone, Deserialize)]
struct TomlEntrypoint {
    entrypoint: TomlEntrypointData,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlEntrypointData {
    id: String,
    provider_id: String,
    #[serde(default)]
    website_domain: String,
    api: TomlApi,
    #[serde(default)]
    defaults: TomlDefaults,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlApi {
    base_url: String,
    protocol: String,
    #[serde(default)]
    auth_type: Option<String>,
    #[serde(default)]
    auth_header: Option<String>,
    env_var: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct TomlDefaults {
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    deep: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    normal: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    basic: Vec<String>,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct StringOrVec;
    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string or a list of strings")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(vec![v.to_string()])
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut v = Vec::new();
            while let Some(item) = seq.next_element()? {
                v.push(item);
            }
            Ok(v)
        }
    }
    deserializer.deserialize_any(StringOrVec)
}

pub struct ResolvedEntrypoint {
    pub entrypoint_id: String,
    pub provider_id: String,
    pub website_domain: String,
    pub env_var: String,
    pub protocol: String,
    pub base_url: String,
    pub auth_type: Option<String>,
    pub auth_header: Option<String>,
    pub deep_models: Vec<String>,
    pub normal_models: Vec<String>,
    pub basic_models: Vec<String>,
}

pub fn load_all_entrypoints_from_toml() -> Vec<ResolvedEntrypoint> {
    let mut results = Vec::new();
    for provider_dir in arona_res::entrypoint::ENTRYPOINT_DIR.dirs() {
        for file in provider_dir.files() {
            if file.path().extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let Some(content) = file.contents_utf8() else {
                continue;
            };
            let Ok(parsed) = toml::from_str::<TomlEntrypoint>(content) else {
                continue;
            };
            let ep = parsed.entrypoint;
            debug!(
                entrypoint_id = %ep.id,
                provider_id = %ep.provider_id,
                "loaded entrypoint"
            );
            results.push(ResolvedEntrypoint {
                entrypoint_id: ep.id,
                provider_id: ep.provider_id,
                website_domain: ep.website_domain,
                env_var: ep.api.env_var,
                protocol: ep.api.protocol,
                base_url: ep.api.base_url,
                auth_type: ep.api.auth_type,
                auth_header: ep.api.auth_header,
                deep_models: ep.defaults.deep,
                normal_models: ep.defaults.normal,
                basic_models: ep.defaults.basic,
            });
        }
    }
    results
}

fn default_rate_multiplier() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_max_concurrent() -> usize {
    1
}

/// Periodic billing configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AvailablePeriodConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub billing_type: BillingType,
    #[serde(default)]
    pub period_unit: PeriodUnit,
    #[serde(default)]
    pub period_hours: u32,
    #[serde(default)]
    pub period_start: Option<String>,
    #[serde(default)]
    pub stats_type: StatsType,
    #[serde(default)]
    pub quota_limit: Option<u64>,
    #[serde(default)]
    pub quota_used: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillingType {
    #[default]
    OneTime,
    Periodic,
}

impl BillingType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneTime => "one_time",
            Self::Periodic => "periodic",
        }
    }
}

impl std::fmt::Display for BillingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PeriodUnit {
    #[default]
    Hours,
    Days,
}

impl PeriodUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hours => "hours",
            Self::Days => "days",
        }
    }

    pub fn to_hours(self, value: u32) -> u32 {
        match self {
            Self::Hours => value,
            Self::Days => value * 24,
        }
    }
}

impl std::fmt::Display for PeriodUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StatsType {
    #[default]
    Tokens,
    Requests,
}

impl StatsType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tokens => "tokens",
            Self::Requests => "requests",
        }
    }
}

impl std::fmt::Display for StatsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Mirrors TUI AvailableProvider struct (read-only, server-side)
#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub uuid: Uuid,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub is_validated: bool,
    #[serde(default)]
    pub is_custom: bool,
    #[serde(default)]
    pub entry_point_id: Option<String>,
    #[serde(default)]
    pub period_billing_configs: Vec<AvailablePeriodConfig>,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub auth_header: Option<String>,
}

impl std::fmt::Debug for ProviderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderEntry")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("uuid", &self.uuid)
            .field("api_key", &"<REDACTED>")
            .field("base_url", &self.base_url)
            .field("is_validated", &self.is_validated)
            .field("is_custom", &self.is_custom)
            .field("entry_point_id", &self.entry_point_id)
            .field("auth_type", &self.auth_type)
            .finish()
    }
}

/// Priority order entry: a tier with an ordered list of provider UUIDs.
/// Order in the list = priority (first = highest).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriorityOrderEntry {
    pub tier: String,
    #[serde(default)]
    pub provider_uuids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousMemory {
    #[serde(default)]
    pub covers_user: bool,
    #[serde(default)]
    pub covers_workspace: bool,
    #[serde(default)]
    pub user_model_prefix: Option<String>,
    #[serde(default)]
    pub workspace_model_prefix: Option<String>,
}

impl AutonomousMemory {
    pub fn skip_philia(&self) -> bool {
        self.covers_user
    }

    pub fn skip_aporia(&self) -> bool {
        self.covers_workspace
    }
}

/// Mirrors TUI AvailableModel struct (read-only, server-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub api_model: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default)]
    pub is_custom: bool,
    #[serde(default)]
    pub priority: u32,
    /// "basic" | "normal" | "deep"
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub context_window: Option<u64>,
    #[serde(default)]
    pub compression_threshold: Option<u64>,
    /// Model supports per-usage (token) billing
    #[serde(default = "default_true")]
    pub has_per_usage: bool,
    /// Model participates in provider periodic billing plan(s)
    #[serde(default)]
    pub has_periodic: bool,
    #[serde(default)]
    pub price_input: Option<f64>,
    #[serde(default)]
    pub price_cache_input: Option<f64>,
    #[serde(default)]
    pub price_output: Option<f64>,
    #[serde(default = "default_rate_multiplier")]
    pub rate_multiplier: f64,
    #[serde(default)]
    pub supports_image: bool,
    #[serde(default)]
    pub supports_audio: bool,
    #[serde(default)]
    pub supports_video: bool,
    #[serde(default)]
    pub can_reason: bool,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default)]
    pub category: ModelCategory,
    #[serde(default)]
    pub generation: Option<GenerationParams>,
    #[serde(default)]
    pub autonomous_memory: Option<AutonomousMemory>,
}

/// Mirrors TUI ProviderConfigStoreData struct
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfigData {
    #[serde(default)]
    pub providers: Vec<ProviderEntry>,
    #[serde(default)]
    pub models: Vec<ModelEntry>,
    #[serde(default)]
    pub priority_orders: Vec<PriorityOrderEntry>,
}

impl ProviderConfigData {
    /// In-memory migration for backward compatibility:
    /// 1. Generate UUIDs for providers that have nil UUIDs
    /// 2. Build priority_orders from legacy priority values if none exist
    fn migrate_in_memory(&mut self) {
        // 1. Generate UUIDs
        for provider in self.providers.iter_mut() {
            if provider.uuid.is_nil() {
                provider.uuid = Uuid::now_v7();
                trace!(
                    "[ProviderConfig] Generated UUID {} for provider {}",
                    provider.uuid, provider.id
                );
            }
        }

        // 2. Build priority_orders from legacy priority if none exist
        if !self.priority_orders.is_empty() {
            return;
        }
        for tier_variant in ModelTier::all() {
            let tier = tier_variant.as_tier_str();
            let mut tier_providers: Vec<(Uuid, u32)> = self
                .models
                .iter()
                .filter(|m| m.tier.eq_ignore_ascii_case(tier) && m.is_enabled)
                .filter_map(|m| {
                    self.providers
                        .iter()
                        .find(|p| p.id == m.provider_id && !p.uuid.is_nil())
                        .map(|p| (p.uuid, m.priority))
                })
                .collect();
            if tier_providers.is_empty() {
                continue;
            }
            tier_providers.sort_by_key(|a| a.1);
            tier_providers.dedup_by_key(|(uuid, _)| *uuid);
            let uuids: Vec<Uuid> = tier_providers.into_iter().map(|(uuid, _)| uuid).collect();
            self.priority_orders.push(PriorityOrderEntry {
                tier: tier.to_string(),
                provider_uuids: uuids,
            });
        }
    }
}

/// Return a priority-ordered chain of candidate `provider_config.toml`
/// paths (descending priority — first entry wins).
///
/// Priority layers:
///   [1] ENV layer — `ENTELECHEIA_CONFIG_PATH` (explicit file) then
///       `ENTELECHEIA_CONFIG_DIR/provider_config.toml` (explicit directory)
///   [2] GLOBAL layer — `UserConfig::config_dir()/provider_config.toml`
///       (resolved via XDG_CONFIG_HOME / HOME, never CWD)
fn config_chain() -> Vec<PathBuf> {
    const FILENAME: &str = "provider_config.toml";
    let mut chain: Vec<PathBuf> = Vec::new();

    // ── [1] ENV layer ─────────────────────────────────────────
    if let Ok(p) = std::env::var("ENTELECHEIA_CONFIG_PATH") {
        chain.push(PathBuf::from(p));
    }
    if let Ok(dir) = std::env::var("ENTELECHEIA_CONFIG_DIR") {
        chain.push(PathBuf::from(&dir).join(FILENAME));
    }

    // ── [2] GLOBAL layer — standard XDG config directory only ──
    // No CWD walk-up: the config location is determined by environment
    // (XDG_CONFIG_HOME / HOME / ENTELECHEIA_CONFIG_DIR), never guessed
    // from the working directory.
    chain.push(super::app_config::UserConfig::config_dir().join(FILENAME));

    chain
}

/// Backwards-compatible shim: return the first *existing* path from the
/// chain, or `None` when no candidate is on disk.
fn find_config_path() -> Option<PathBuf> {
    config_chain().into_iter().find(|p| p.exists())
}

/// Public accessor for the full config chain (used by the watcher to
/// watch all candidate directories at once).
pub fn config_chain_public() -> Vec<PathBuf> {
    config_chain()
}

/// Return the path to provider_config.toml (if it exists), for external watching
pub fn find_provider_config_path() -> Option<PathBuf> {
    find_config_path()
}

/// Auto-derive a minimal ProviderConfigData from environment variables.
///
/// When provider_config.toml does not exist, this allows the server to work
/// directly from env vars without relying on TUI-generated config.
///
/// Variables read (priority high to low):
/// - endpoint: LLM_ENDPOINT > LLM_API_ENDPOINT > LLM_BASE_URL
/// - api_key:  LLM_API_KEY   > API_KEY
/// - model:    LLM_MODEL
/// - Optional tier models: LLM_MODEL_DEEP / LLM_MODEL_NORMAL / LLM_MODEL_BASIC
fn derive_config_from_env() -> Option<ProviderConfigData> {
    let mut providers: Vec<ProviderEntry> = Vec::new();
    let mut models: Vec<ModelEntry> = Vec::new();
    let mut priority_orders: Vec<PriorityOrderEntry> = Vec::new();

    let endpoint = std::env::var("LLM_ENDPOINT")
        .or_else(|_| std::env::var("LLM_API_ENDPOINT"))
        .or_else(|_| std::env::var("LLM_BASE_URL"))
        .unwrap_or_default();

    let api_key = std::env::var("LLM_API_KEY")
        .or_else(|_| std::env::var("API_KEY"))
        .unwrap_or_default();

    let model = std::env::var("LLM_MODEL")
        .or_else(|_| std::env::var("MODEL"))
        .unwrap_or_default();

    if !endpoint.is_empty() && !api_key.is_empty() && !model.is_empty() {
        let provider_id = "env_derived".to_string();
        let provider_uuid = Uuid::now_v7();

        let protocol = std::env::var("LLM_PROTOCOL").unwrap_or_default();
        let (auth_type, auth_header) = derive_auth(&protocol);

        providers.push(ProviderEntry {
            id: provider_id.clone(),
            name: "Env-derived Provider".to_string(),
            uuid: provider_uuid,
            api_key,
            base_url: Some(endpoint),
            is_validated: true,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type,
            auth_header,
        });

        push_tier_models(
            &provider_id,
            &model,
            provider_uuid,
            &mut models,
            &mut priority_orders,
        );
    }

    let all_entrypoints = load_all_entrypoints_from_toml();

    for ep in &all_entrypoints {
        debug!(
            entrypoint_id = %ep.entrypoint_id,
            provider_id = %ep.provider_id,
            "checking entrypoint for env key"
        );
        let Ok(key_val) = std::env::var(&ep.env_var) else {
            continue;
        };
        if is_invalid_api_key(&key_val) {
            continue;
        }
        if providers.iter().any(|p| p.id == ep.entrypoint_id) {
            continue;
        }

        let provider_uuid = Uuid::now_v7();
        let (auth_type, auth_header) = resolve_auth(ep);

        providers.push(ProviderEntry {
            id: ep.entrypoint_id.clone(),
            name: ep.entrypoint_id.clone(),
            uuid: provider_uuid,
            api_key: key_val,
            base_url: Some(ep.base_url.clone()),
            is_validated: true,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type,
            auth_header,
        });

        push_tier_models_from_toml(ep, provider_uuid, &mut models, &mut priority_orders);
    }

    if providers.is_empty() {
        return None;
    }

    // Merge per-provider priority orders into a single entry per tier,
    // ordered by cached capability score, so cross-provider preference is
    // preserved (instead of every provider collapsing onto position 0).
    let entrypoint_provider_map: std::collections::HashMap<String, String> = all_entrypoints
        .iter()
        .map(|ep| (ep.entrypoint_id.clone(), ep.provider_id.clone()))
        .collect();
    merge_priority_orders_by_tier(
        &mut priority_orders,
        &providers,
        &models,
        &entrypoint_provider_map,
    );

    info!(
        "derived config from env vars + TOML entrypoints: {} provider(s), {} model(s)",
        providers.len(),
        models.len()
    );

    Some(ProviderConfigData {
        providers,
        models,
        priority_orders,
    })
}

/// Ensure `provider_config.toml` exists on disk.
///
/// If the file already exists, this is a no-op (the user or a previous init
/// has already configured providers). If the file is missing, the function
/// derives a complete `ProviderConfigData` from currently-set environment
/// variables (generic `LLM_*` vars + entrypoint-specific API keys) and writes
/// it to `{config_dir}/entelecheia/provider_config.toml`.
///
/// This should be called by CLI/TUI **before** starting the scepter container,
/// so that scepter never needs to fall back to `derive_config_from_env()` at
/// runtime. The scepter server can always read the file from the mounted
/// config directory.
///
/// Returns `Ok(true)` if a new file was written, `Ok(false)` if it already
/// existed, or an error if writing failed.
pub fn ensure_provider_config_from_env() -> std::io::Result<bool> {
    let config_dir = super::app_config::UserConfig::config_dir();
    let config_path = config_dir.join("provider_config.toml");

    if config_path.exists() {
        debug!(
            "provider_config.toml already exists at {:?}, skipping generation",
            config_path
        );
        return Ok(false);
    }

    let Some(data) = derive_config_from_env() else {
        info!("no LLM env vars found, skipping provider config generation");
        return Ok(false);
    };

    if data.providers.is_empty() {
        info!("derived config has no providers, skipping file generation");
        return Ok(false);
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let toml_str = toml::to_string_pretty(&data).map_err(|e| {
        std::io::Error::other(format!("failed to serialize provider config: {}", e))
    })?;

    std::fs::write(&config_path, &toml_str)?;

    info!(
        "generated provider_config.toml at {:?}: {} provider(s), {} model(s)",
        config_path,
        data.providers.len(),
        data.models.len()
    );

    Ok(true)
}

fn resolve_auth(ep: &ResolvedEntrypoint) -> (Option<String>, Option<String>) {
    let auth_type = ep.auth_type.clone().or_else(|| Some("bearer".to_string()));
    let auth_header = ep
        .auth_header
        .clone()
        .or_else(|| Some("Authorization".to_string()));
    (auth_type, auth_header)
}

fn push_tier_models_from_toml(
    ep: &ResolvedEntrypoint,
    provider_uuid: Uuid,
    models: &mut Vec<ModelEntry>,
    priority_orders: &mut Vec<PriorityOrderEntry>,
) {
    let tier_models = [
        ("deep", &ep.deep_models),
        (TIER_NORMAL, &ep.normal_models),
        ("basic", &ep.basic_models),
    ];

    for (tier, model_list) in tier_models {
        let model_name = model_list.first().map(|s| s.as_str()).unwrap_or("");
        if model_name.is_empty() {
            continue;
        }
        models.push(ModelEntry {
            id: format!("{}_{}", ep.entrypoint_id, tier),
            name: model_name.to_string(),
            provider_id: ep.entrypoint_id.clone(),
            api_model: model_name.to_string(),
            is_enabled: true,
            is_custom: false,
            priority: 0,
            tier: tier.to_string(),
            context_window: None,
            compression_threshold: None,
            has_per_usage: true,
            has_periodic: false,
            price_input: None,
            price_cache_input: None,
            price_output: None,
            rate_multiplier: 1.0,
            supports_image: false,
            supports_audio: false,
            supports_video: false,
            can_reason: false,
            max_concurrent: 1,
            category: ModelCategory::default(),
            generation: None,
            autonomous_memory: None,
        });
        priority_orders.push(PriorityOrderEntry {
            tier: tier.to_string(),
            provider_uuids: vec![provider_uuid],
        });
    }
}

/// Read the cached capability score (`[model.score] value`) for a model from
/// the embedded registry model card. Returns `None` when the provider dir,
/// model file, or score section is absent.
fn read_model_card_score(registry_provider_id: &str, model_name: &str) -> Option<f64> {
    let dir = arona_res::entrypoint::get_provider_models_dir(registry_provider_id)?;
    let target = format!("{}.toml", model_name.replace('/', "_").replace(':', "-"));
    // Iterate rather than get_file(): include_dir's get_file is unreliable for
    // filenames containing dots, but files() reliably yields every entry.
    let file = dir.files().find(|f| {
        f.path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == target)
            .unwrap_or(false)
    })?;
    let content = file.contents_utf8()?;
    let parsed: toml::Value = toml::from_str(content).ok()?;
    parsed
        .get("model")
        .and_then(|m| m.get("score"))
        .and_then(|s| s.get("value"))
        .and_then(|v| v.as_float())
}

/// Merge per-provider `PriorityOrderEntry`s into a single entry per tier,
/// ordering `provider_uuids` by the cached capability score of each
/// provider's tier model (descending). Providers whose tier model has no
/// score sort last, preserving their original relative order (stable).
///
/// This fixes cross-provider ordering: previously each provider pushed its
/// own one-uuid entry, so `build_provider_order_map` collapsed every
/// provider onto position 0 and the runtime fell back to array order.
/// After merging, position 0..N within a tier genuinely reflects capability
/// preference across providers.
fn merge_priority_orders_by_tier(
    priority_orders: &mut Vec<PriorityOrderEntry>,
    providers: &[ProviderEntry],
    models: &[ModelEntry],
    entrypoint_provider_map: &std::collections::HashMap<String, String>,
) {
    use std::collections::HashMap;

    if priority_orders.is_empty() {
        return;
    }

    // uuid -> entrypoint_id
    let uuid_to_ep: HashMap<Uuid, &str> =
        providers.iter().map(|p| (p.uuid, p.id.as_str())).collect();
    // (entrypoint_id, tier_lower) -> first tier model's api_model name
    let mut ep_tier_model: HashMap<(&str, String), &str> = HashMap::new();
    for m in models {
        ep_tier_model
            .entry((m.provider_id.as_str(), m.tier.to_ascii_lowercase()))
            .or_insert(m.api_model.as_str());
    }

    // Score lookup for a (uuid, tier_lower) pair.
    let score_for = |uuid: &Uuid, tier_lower: &str| -> Option<f64> {
        let ep = uuid_to_ep.get(uuid).copied();
        let provider_id = ep
            .and_then(|e| entrypoint_provider_map.get(e))
            .map(|s| s.as_str());
        let model_name = ep
            .and_then(|e| ep_tier_model.get(&(e, tier_lower.to_string())))
            .copied();
        match (provider_id, model_name) {
            (Some(p), Some(m)) => read_model_card_score(p, m),
            _ => None,
        }
    };

    // Group (uuid, original_global_pos) by tier, tracking tier first-seen order.
    let mut tiers_order: Vec<String> = Vec::new();
    let mut by_tier: HashMap<String, Vec<(Uuid, usize)>> = HashMap::new();
    let mut global_pos = 0usize;
    for entry in priority_orders.iter() {
        let tier = entry.tier.to_ascii_lowercase();
        if !by_tier.contains_key(&tier) {
            tiers_order.push(tier.clone());
        }
        for uuid in &entry.provider_uuids {
            by_tier
                .entry(tier.clone())
                .or_default()
                .push((*uuid, global_pos));
            global_pos += 1;
        }
    }

    // Build one merged entry per tier.
    let merged: Vec<PriorityOrderEntry> = tiers_order
        .iter()
        .map(|tier| {
            let mut items = by_tier.remove(tier).unwrap_or_default();
            // Sort: score descending; None last; stable on original position.
            items.sort_by(|a, b| {
                let sa = score_for(&a.0, tier);
                let sb = score_for(&b.0, tier);
                match (sa, sb) {
                    (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
                .then(a.1.cmp(&b.1))
            });
            PriorityOrderEntry {
                tier: tier.clone(),
                provider_uuids: items.into_iter().map(|(u, _)| u).collect(),
            }
        })
        .collect();

    *priority_orders = merged;
}

fn derive_auth(_protocol: &str) -> (Option<String>, Option<String>) {
    (
        Some("bearer".to_string()),
        Some("Authorization".to_string()),
    )
}

fn push_tier_models(
    provider_id: &str,
    model: &str,
    provider_uuid: Uuid,
    models: &mut Vec<ModelEntry>,
    priority_orders: &mut Vec<PriorityOrderEntry>,
) {
    let base = ModelEntry {
        id: format!("{}_normal", provider_id),
        name: model.to_string(),
        provider_id: provider_id.to_string(),
        api_model: model.to_string(),
        is_enabled: true,
        is_custom: false,
        priority: 0,
        tier: "normal".to_string(),
        context_window: None,
        compression_threshold: None,
        has_per_usage: true,
        has_periodic: false,
        price_input: None,
        price_cache_input: None,
        price_output: None,
        rate_multiplier: 1.0,
        supports_image: false,
        supports_audio: false,
        supports_video: false,
        can_reason: false,
        max_concurrent: 1,
        category: ModelCategory::default(),
        generation: None,
        autonomous_memory: None,
    };

    for tier in ["deep", TIER_NORMAL, "basic"] {
        models.push(ModelEntry {
            tier: tier.to_string(),
            id: format!("{}_{}", provider_id, tier),
            ..base.clone()
        });
        priority_orders.push(PriorityOrderEntry {
            tier: tier.to_string(),
            provider_uuids: vec![provider_uuid],
        });
    }
}

/// Load provider_config.toml from disk (falls back to env var derivation if not found)
pub fn load_provider_config() -> ProviderConfigData {
    let Some(path) = find_config_path() else {
        if let Some(derived) = derive_config_from_env() {
            info!(
                "provider_config.toml not found, derived config from env vars: {} provider(s), {} model(s)",
                derived.providers.len(),
                derived.models.len()
            );
            return derived;
        }
        info!("provider_config.toml not found, using empty config");
        return ProviderConfigData::default();
    };

    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<ProviderConfigData>(&content) {
            Ok(mut data) => {
                data.migrate_in_memory();
                info!(
                    "loaded {} providers, {} models (from {:?})",
                    data.providers.len(),
                    data.models.len(),
                    path
                );
                data
            },
            Err(e) => {
                warn!("failed to parse provider_config.toml: {}", e);
                ProviderConfigData::default()
            },
        },
        Err(e) => {
            warn!("failed to read provider_config.toml: {}", e);
            ProviderConfigData::default()
        },
    }
}

/// A fully-resolved model card: the [`ModelEntry`] paired with its owning
/// [`ProviderEntry`], so every downstream consumer has immediate access to
/// billing configuration, autonomous-memory settings, pricing, and connection
/// details without re-reading the config file.
///
/// Resolve once via [`select_model_card_for_tier`], then pass the card by
/// reference wherever model/provider info is needed.
#[derive(Debug, Clone)]
pub struct ResolvedModelCard {
    /// The selected model entry (tier, pricing, capabilities, etc.).
    pub model: ModelEntry,
    /// The provider that owns this model (api key, base url, billing configs).
    pub provider: ProviderEntry,
}

impl ResolvedModelCard {
    /// Effective billing metric for this model's provider.
    ///
    /// Returns [`StatsType::Requests`] when the provider has any periodic
    /// billing config billed per-request (the cost-sensitive case), otherwise
    /// [`StatsType::Tokens`].
    pub fn effective_stats_type(&self) -> StatsType {
        let has_requests = self
            .provider
            .period_billing_configs
            .iter()
            .any(|c| c.stats_type == StatsType::Requests);
        if has_requests {
            StatsType::Requests
        } else {
            StatsType::Tokens
        }
    }

    /// Autonomous-memory configuration for this model, if any.
    pub fn autonomous_memory(&self) -> Option<&AutonomousMemory> {
        self.model.autonomous_memory.as_ref()
    }
}

/// Resolve the complete [`ResolvedModelCard`] for the best available model in
/// the given tier.
///
/// This is the single entry point that calls [`load_provider_config`]; all
/// downstream consumers should reuse the returned card instead of re-resolving.
pub fn select_model_card_for_tier(tier: &str) -> Option<ResolvedModelCard> {
    let config = load_provider_config();
    let tier_lower = tier.to_ascii_lowercase();

    let order_map = build_provider_order_map(&config.priority_orders);

    let tier_key = tier_lower.clone();
    let model = config
        .models
        .iter()
        .filter(|m| {
            let provider = config.providers.iter().find(|p| p.id == m.provider_id);
            m.is_enabled
                && !m.api_model.is_empty()
                && m.tier == tier_lower
                && provider.is_some_and(|p| !is_invalid_api_key(&p.api_key) && p.base_url.is_some())
        })
        .min_by_key(|m| {
            let p = config.providers.iter().find(|p| p.id == m.provider_id);
            let uuid = p.map(|p| p.uuid).unwrap_or_default();
            order_map
                .get(&(tier_key.clone(), uuid))
                .copied()
                .unwrap_or(u32::MAX)
        })
        .cloned()?;

    let provider = config
        .providers
        .iter()
        .find(|p| p.id == model.provider_id)?
        .clone();

    Some(ResolvedModelCard { model, provider })
}

/// Select the best available [`ModelEntry`] for a given tier.
///
/// Convenience wrapper around [`select_model_card_for_tier`] that discards the
/// provider. Prefer [`select_model_card_for_tier`] when you also need provider
/// or billing info, to avoid a redundant config round-trip.
pub fn select_model_entry_for_tier(tier: &str) -> Option<ModelEntry> {
    select_model_card_for_tier(tier).map(|card| card.model)
}

/// Build a lookup map: (tier, provider_uuid) → position in priority order.
/// Position 0 = highest priority. Missing entries get u32::MAX.
pub fn build_provider_order_map(
    orders: &[PriorityOrderEntry],
) -> std::collections::HashMap<(String, Uuid), u32> {
    let mut map = std::collections::HashMap::new();
    for entry in orders {
        let tier = entry.tier.to_ascii_lowercase();
        for (pos, uuid) in entry.provider_uuids.iter().enumerate() {
            map.entry((tier.clone(), *uuid)).or_insert(pos as u32);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn test_build_provider_order_map_empty() -> Result<()> {
        let map = build_provider_order_map(&[]);
        assert!(map.is_empty());
        Ok(())
    }

    #[test]
    fn test_build_provider_order_map_single_entry() -> Result<()> {
        let uuid1 = Uuid::now_v7();
        let uuid2 = Uuid::now_v7();
        let orders = vec![PriorityOrderEntry {
            tier: "basic".to_string(),
            provider_uuids: vec![uuid1, uuid2],
        }];
        let map = build_provider_order_map(&orders);
        assert_eq!(map.get(&("basic".to_string(), uuid1)), Some(&0));
        assert_eq!(map.get(&("basic".to_string(), uuid2)), Some(&1));
        assert_eq!(map.len(), 2);
        Ok(())
    }

    #[test]
    fn test_build_provider_order_map_multi_tier() -> Result<()> {
        let uuid1 = Uuid::now_v7();
        let uuid2 = Uuid::now_v7();
        let orders = vec![
            PriorityOrderEntry {
                tier: "BASIC".to_string(),
                provider_uuids: vec![uuid1],
            },
            PriorityOrderEntry {
                tier: "deep".to_string(),
                provider_uuids: vec![uuid2, uuid1],
            },
        ];
        let map = build_provider_order_map(&orders);
        assert_eq!(map.get(&("basic".to_string(), uuid1)), Some(&0));
        assert_eq!(map.get(&("deep".to_string(), uuid2)), Some(&0));
        assert_eq!(map.get(&("deep".to_string(), uuid1)), Some(&1));
        Ok(())
    }

    #[test]
    fn test_build_provider_order_map_first_wins() -> Result<()> {
        let uuid1 = Uuid::now_v7();
        let orders = vec![
            PriorityOrderEntry {
                tier: "basic".to_string(),
                provider_uuids: vec![uuid1],
            },
            PriorityOrderEntry {
                tier: "basic".to_string(),
                provider_uuids: vec![uuid1],
            },
        ];
        let map = build_provider_order_map(&orders);
        assert_eq!(map.get(&("basic".to_string(), uuid1)), Some(&0));
        assert_eq!(map.len(), 1);
        Ok(())
    }

    /// Build a minimal ProviderEntry for tests.
    fn mk_provider(id: &str, uuid: Uuid) -> ProviderEntry {
        ProviderEntry {
            id: id.to_string(),
            name: id.to_string(),
            uuid,
            api_key: String::new(),
            base_url: None,
            is_validated: true,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type: None,
            auth_header: None,
        }
    }

    /// Build a minimal ModelEntry tying an entrypoint to a tier model name.
    fn mk_model(provider_id: &str, tier: &str, api_model: &str) -> ModelEntry {
        ModelEntry {
            id: format!("{}_{}", provider_id, tier),
            name: api_model.to_string(),
            provider_id: provider_id.to_string(),
            api_model: api_model.to_string(),
            is_enabled: true,
            is_custom: false,
            priority: 0,
            tier: tier.to_string(),
            context_window: None,
            compression_threshold: None,
            has_per_usage: true,
            has_periodic: false,
            price_input: None,
            price_cache_input: None,
            price_output: None,
            rate_multiplier: 1.0,
            supports_image: false,
            supports_audio: false,
            supports_video: false,
            can_reason: false,
            max_concurrent: 1,
            category: ModelCategory::default(),
            generation: None,
            autonomous_memory: None,
        }
    }

    /// Real registry scores (design_arena-mapped) at time of writing:
    ///   glm-5.2=67.89  glm-5.2=67.89  glm-5-turbo=65.73
    ///   deepseek-v4-pro=50.82  deepseek-v4-flash=55.63  deepseek-reasoner=none
    #[test]
    fn test_merge_priority_orders_score_driven() -> Result<()> {
        let glm_uuid = Uuid::now_v7();
        let ds_uuid = Uuid::now_v7();
        // deepseek is listed FIRST on purpose: the merge must promote glm to
        // the front wherever its tier model scores higher.
        let providers = vec![
            mk_provider("deepseek_default", ds_uuid),
            mk_provider("zhipu_glm_coding_max_domestic", glm_uuid),
        ];
        let models = vec![
            mk_model("deepseek_default", "deep", "deepseek-reasoner"),
            mk_model("deepseek_default", "normal", "deepseek-v4-pro"),
            mk_model("deepseek_default", "basic", "deepseek-v4-flash"),
            mk_model("zhipu_glm_coding_max_domestic", "deep", "glm-5.2"),
            mk_model("zhipu_glm_coding_max_domestic", "normal", "glm-5.2"),
            mk_model("zhipu_glm_coding_max_domestic", "basic", "glm-5-turbo"),
        ];
        let mut orders = vec![
            PriorityOrderEntry {
                tier: "deep".to_string(),
                provider_uuids: vec![ds_uuid],
            },
            PriorityOrderEntry {
                tier: "deep".to_string(),
                provider_uuids: vec![glm_uuid],
            },
            PriorityOrderEntry {
                tier: "normal".to_string(),
                provider_uuids: vec![ds_uuid],
            },
            PriorityOrderEntry {
                tier: "normal".to_string(),
                provider_uuids: vec![glm_uuid],
            },
            PriorityOrderEntry {
                tier: "basic".to_string(),
                provider_uuids: vec![ds_uuid],
            },
            PriorityOrderEntry {
                tier: "basic".to_string(),
                provider_uuids: vec![glm_uuid],
            },
        ];
        let map = std::collections::HashMap::from([
            ("deepseek_default".to_string(), "deepseek".to_string()),
            (
                "zhipu_glm_coding_max_domestic".to_string(),
                "zhipu_glm".to_string(),
            ),
        ]);
        merge_priority_orders_by_tier(&mut orders, &providers, &models, &map);

        // One merged entry per tier.
        assert_eq!(orders.len(), 3, "expected one entry per tier");

        let by_tier: std::collections::HashMap<String, Vec<Uuid>> = orders
            .iter()
            .map(|e| (e.tier.clone(), e.provider_uuids.clone()))
            .collect();

        // deep: glm-5.2 (67.89) > deepseek-reasoner (none) -> glm first
        assert_eq!(
            by_tier["deep"][0], glm_uuid,
            "deep tier should prefer glm-5.2"
        );
        assert_eq!(by_tier["deep"][1], ds_uuid);
        // normal: glm-5 (59.11) > deepseek-v4-pro (50.82) -> glm first
        assert_eq!(
            by_tier["normal"][0], glm_uuid,
            "normal tier should prefer glm-5.2"
        );
        assert_eq!(by_tier["normal"][1], ds_uuid);
        // basic: glm-5-turbo (65.73) > deepseek-v4-flash (55.63) -> glm first
        assert_eq!(
            by_tier["basic"][0], glm_uuid,
            "basic tier should prefer glm-5-turbo"
        );
        assert_eq!(by_tier["basic"][1], ds_uuid);
        Ok(())
    }

    /// When two providers' tier models both lack a score, the merge must keep
    /// their original relative order (stable).
    #[test]
    fn test_merge_priority_orders_no_score_stable() -> Result<()> {
        let a_uuid = Uuid::now_v7();
        let b_uuid = Uuid::now_v7();
        let providers = vec![
            mk_provider("env_derived", a_uuid),
            mk_provider("deepseek_default", b_uuid),
        ];
        // env_derived has no registry provider -> no score; deepseek-reasoner has no score either.
        let models = vec![
            mk_model("env_derived", "deep", "some-model"),
            mk_model("deepseek_default", "deep", "deepseek-reasoner"),
        ];
        let mut orders = vec![
            PriorityOrderEntry {
                tier: "deep".to_string(),
                provider_uuids: vec![a_uuid],
            },
            PriorityOrderEntry {
                tier: "deep".to_string(),
                provider_uuids: vec![b_uuid],
            },
        ];
        let map = std::collections::HashMap::from([(
            "deepseek_default".to_string(),
            "deepseek".to_string(),
        )]);
        merge_priority_orders_by_tier(&mut orders, &providers, &models, &map);
        assert_eq!(orders.len(), 1);
        assert_eq!(
            orders[0].provider_uuids,
            vec![a_uuid, b_uuid],
            "original order preserved when no scores"
        );
        Ok(())
    }

    /// The merged order must be reflected by build_provider_order_map: the
    /// first provider in a merged entry gets position 0.
    #[test]
    fn test_merge_then_order_map_consistent() -> Result<()> {
        let glm_uuid = Uuid::now_v7();
        let ds_uuid = Uuid::now_v7();
        let providers = vec![
            mk_provider("deepseek_default", ds_uuid),
            mk_provider("zhipu_glm_coding_max_domestic", glm_uuid),
        ];
        let models = vec![
            mk_model("deepseek_default", "normal", "deepseek-v4-pro"),
            mk_model("zhipu_glm_coding_max_domestic", "normal", "glm-5.2"),
        ];
        let mut orders = vec![
            PriorityOrderEntry {
                tier: "normal".to_string(),
                provider_uuids: vec![ds_uuid],
            },
            PriorityOrderEntry {
                tier: "normal".to_string(),
                provider_uuids: vec![glm_uuid],
            },
        ];
        let map = std::collections::HashMap::from([
            ("deepseek_default".to_string(), "deepseek".to_string()),
            (
                "zhipu_glm_coding_max_domestic".to_string(),
                "zhipu_glm".to_string(),
            ),
        ]);
        merge_priority_orders_by_tier(&mut orders, &providers, &models, &map);
        let order_map = build_provider_order_map(&orders);
        assert_eq!(
            order_map.get(&("normal".to_string(), glm_uuid)),
            Some(&0),
            "glm-5.2 should be position 0"
        );
        assert_eq!(order_map.get(&("normal".to_string(), ds_uuid)), Some(&1));
        Ok(())
    }

    #[test]
    fn test_read_model_card_score_lookup() -> Result<()> {
        // Sanity: real registry cards must expose scores for these tier models.
        let glm52 = read_model_card_score("zhipu_glm", "glm-5.2")
            .ok_or_else(|| anyhow::anyhow!("glm-5.2 score missing"))?;
        assert!(glm52 > 60.0, "glm-5.2 score unexpectedly low");
        let dspro = read_model_card_score("deepseek", "deepseek-v4-pro")
            .ok_or_else(|| anyhow::anyhow!("deepseek-v4-pro score missing"))?;
        // glm-5.2 must beat deepseek-v4-pro so the normal tier prefers glm.
        assert!(glm52 > dspro, "glm-5.2 score should exceed deepseek-v4-pro");
        // deepseek-reasoner is absent from OpenRouter -> no card score.
        assert!(read_model_card_score("deepseek", "deepseek-reasoner").is_none());
        Ok(())
    }

    #[test]
    fn test_autonomous_memory_defaults() -> Result<()> {
        let am: AutonomousMemory = toml::from_str("")?;
        assert!(!am.covers_user);
        assert!(!am.covers_workspace);
        assert!(am.user_model_prefix.is_none());
        assert!(am.workspace_model_prefix.is_none());
        assert!(!am.skip_philia());
        assert!(!am.skip_aporia());
        Ok(())
    }

    #[test]
    fn test_autonomous_memory_skip_methods() -> Result<()> {
        let am = AutonomousMemory {
            covers_user: true,
            covers_workspace: false,
            user_model_prefix: Some("ft-user".into()),
            workspace_model_prefix: None,
        };
        assert!(am.skip_philia());
        assert!(!am.skip_aporia());
        Ok(())
    }

    #[test]
    fn test_autonomous_memory_both_layers() -> Result<()> {
        let am = AutonomousMemory {
            covers_user: true,
            covers_workspace: true,
            user_model_prefix: Some("ft-user".into()),
            workspace_model_prefix: Some("ft-ws".into()),
        };
        assert!(am.skip_philia());
        assert!(am.skip_aporia());
        Ok(())
    }

    #[test]
    fn test_model_entry_deserialize_with_autonomous_memory() -> Result<()> {
        let toml_str = r#"
id = "test-model"
name = "Test Model"
provider_id = "test-provider"
api_model = "test-api"
tier = "normal"

[autonomous_memory]
covers_user = true
user_model_prefix = "ft-user"
"#;
        let entry: ModelEntry = toml::from_str(toml_str)?;
        assert_eq!(entry.id, "test-model");
        let am = entry.autonomous_memory.context("test precondition")?;
        assert!(am.covers_user);
        assert!(!am.covers_workspace);
        assert_eq!(am.user_model_prefix.as_deref(), Some("ft-user"));
        assert!(am.workspace_model_prefix.is_none());
        Ok(())
    }

    #[test]
    fn test_model_entry_deserialize_without_autonomous_memory() -> Result<()> {
        let toml_str = r#"
id = "plain-model"
name = "Plain"
provider_id = "p1"
api_model = "plain"
tier = "basic"
"#;
        let entry: ModelEntry = toml::from_str(toml_str)?;
        assert_eq!(entry.id, "plain-model");
        assert!(entry.autonomous_memory.is_none());
        Ok(())
    }

    #[test]
    fn test_model_entry_deserialize_full_autonomous_memory() -> Result<()> {
        let toml_str = r#"
id = "full-mem"
name = "Full Memory"
provider_id = "p1"
api_model = "full-api"
tier = "deep"

[autonomous_memory]
covers_user = true
covers_workspace = true
user_model_prefix = "ft-user"
workspace_model_prefix = "ft-ws"
"#;
        let entry: ModelEntry = toml::from_str(toml_str)?;
        let am = entry.autonomous_memory.context("test precondition")?;
        assert!(am.skip_philia());
        assert!(am.skip_aporia());
        assert_eq!(am.user_model_prefix.as_deref(), Some("ft-user"));
        assert_eq!(am.workspace_model_prefix.as_deref(), Some("ft-ws"));
        Ok(())
    }

    #[test]
    fn test_find_config_path_env_var_takes_priority() -> Result<()> {
        let tmp = std::env::temp_dir();
        let env_path = tmp.join("entelecheia_test_env_config.toml");
        std::fs::write(&env_path, "[test]\nkey = \"value\"").context("test precondition")?;

        let xdg_path = dirs::config_dir().map(|d| d.join("entelecheia/provider_config.toml"));

        // SAFETY: test-only mutation of process-local env var
        unsafe {
            std::env::set_var(
                "ENTELECHEIA_CONFIG_PATH",
                env_path.to_str().context("test precondition")?,
            );
        }
        let result = find_config_path();
        unsafe {
            std::env::remove_var("ENTELECHEIA_CONFIG_PATH");
        }

        let _ = std::fs::remove_file(&env_path);

        assert!(
            result.is_some(),
            "Should find config at ENTELECHEIA_CONFIG_PATH"
        );
        let found = result.context("test precondition")?;
        assert_eq!(
            found, env_path,
            "ENTELECHEIA_CONFIG_PATH should take priority over other paths"
        );

        if let Some(xdg) = xdg_path
            && xdg.exists()
        {
            assert_ne!(
                found, xdg,
                "Should NOT match XDG path when ENTELECHEIA_CONFIG_PATH is set"
            );
        }
        Ok(())
    }

    #[test]
    fn test_find_config_path_returns_none_when_nothing_exists() -> Result<()> {
        // SAFETY: test-only mutation of process-local env var
        unsafe {
            std::env::remove_var("ENTELECHEIA_CONFIG_PATH");
        }
        let result = find_config_path();
        if let Some(path) = result {
            assert!(
                !path
                    .to_str()
                    .context("test precondition")?
                    .contains("ENTELECHEIA_CONFIG_PATH"),
                "Should not use env var when not set"
            );
        }
        Ok(())
    }

    #[test]
    fn test_entrypoint_defaults_parse_string_or_vec() -> Result<()> {
        let toml_str = r#"
[entrypoint]
id = "test_ep"
provider_id = "test_provider"
type = "one_time"
billing_type = "one_time"

[entrypoint.api]
base_url = "https://api.test.com/v1"
protocol = "openai_chat_v1"
env_var = "TEST_API_KEY"

[entrypoint.defaults]
deep = "model-deep"
normal = ["model-normal-1", "model-normal-2"]
basic = "model-basic"

[entrypoint.defaults.max_concurrent]
deep = 1
normal = 2
basic = 3
"#;
        let entry: TomlEntrypoint = toml::from_str(toml_str)?;
        assert_eq!(entry.entrypoint.defaults.deep, vec!["model-deep"]);
        assert_eq!(
            entry.entrypoint.defaults.normal,
            vec!["model-normal-1", "model-normal-2"]
        );
        assert_eq!(entry.entrypoint.defaults.basic, vec!["model-basic"]);
        Ok(())
    }
}
