//! Read-only mirrors of the config filesystem — provider entry points, model
//! catalogs, user preferences and API-key metadata — as served to the TUI by
//! the `GetProvidersFromFs` / `GetModelsFromFs` / `GetUserConfig` / `ListKeys`
//! sync requests.
use serde::{Deserialize, Serialize};

/// HTTP surface of one provider entry point: which protocol it speaks, where it
/// lives and how a call is authenticated. Extends the registry TOML's
/// `[entrypoint.api]` table with the concrete chat/model routes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointApiConfigInfo {
    /// Wire dialect the endpoint speaks; the family's known generation protocols
    /// are enumerated by `GenProtocol` in plana-config (`OpenAIChatV1`,
    /// `AnthropicMessagesV1`, ...).
    pub protocol: String,
    /// Root URL requests to this entry point are built from; the registry TOML
    /// field is `[entrypoint.api] base_url`.
    pub base_url: String,
    /// Chat-completion route the TUI should call on this entry point.
    pub chat_endpoint: String,
    /// Optional model-listing route; `None` (absent key or explicit `null`)
    /// means the entry point advertises no model list.
    #[serde(default)]
    pub models_endpoint: Option<String>,
    /// Authentication scheme applied to requests; the registry TOML field is
    /// `[entrypoint.api] auth_type`.
    pub auth_type: String,
    /// Optional custom header carrying the credential; `None` when the entry
    /// point names no header of its own.
    #[serde(default)]
    pub auth_header: Option<String>,
    /// Environment variable that holds this entry point's API key.
    pub env_var: String,
}

/// One access route of a provider — a specific upstream endpoint with its own
/// billing plan — as loaded from that provider's entry-point TOML files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointConfigInfo {
    /// Entry-point id from the registry TOML (`entrypoint.id`), unique within
    /// its provider.
    pub id: String,
    /// Localized display names keyed by BCP 47 language tag, from the TOML's
    /// `[entrypoint.name]` table (legacy `zhs` / `zht` keys also appear in the
    /// registry).
    pub name: std::collections::HashMap<String, String>,
    /// Entry-point kind, serialized as the JSON key `type`; an absent key
    /// deserializes to an empty string via `#[serde(default)]`.
    #[serde(default)]
    pub r#type: String,
    /// How this entry point is billed (one-time vs. periodic plan); empty
    /// string when the payload omits it.
    #[serde(default)]
    pub billing_type: String,
    /// Subscription plan tier this entry point belongs to; empty when unset.
    #[serde(default)]
    pub plan_tier: String,
    /// Protocol, URL and credential details for reaching this entry point.
    pub api: EntrypointApiConfigInfo,
    /// Model picks used when the user has no explicit choice for a depth tier;
    /// all lists empty when the payload omits the block.
    #[serde(default)]
    pub defaults: EntrypointDefaultsInfo,
    /// Usage quotas advertised for this entry point; empty when omitted.
    #[serde(default)]
    pub quotas: Vec<QuotaInfo>,
    /// Model ids served through this entry point, referencing `ModelFsInfo.id`
    /// values.
    pub models: Vec<String>,
}

/// Default model ids per agent depth tier, plus the per-tier concurrency caps
/// for one entry point.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntrypointDefaultsInfo {
    /// Model ids used by default for `deep`-tier agents; empty when omitted.
    #[serde(default)]
    pub deep: Vec<String>,
    /// Model ids used by default for `normal`-tier agents; empty when omitted.
    #[serde(default)]
    pub normal: Vec<String>,
    /// Model ids used by default for `basic`-tier agents; empty when omitted.
    #[serde(default)]
    pub basic: Vec<String>,
    /// Per-tier concurrent-request caps; all three counters are `0` when the
    /// payload omits the block.
    #[serde(default)]
    pub max_concurrent: MaxConcurrentInfo,
}

/// Maximum number of concurrent requests allowed per agent depth tier; every
/// counter deserializes to `0` when the payload omits it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaxConcurrentInfo {
    /// Concurrency cap for `deep`-tier agents; `0` when omitted.
    #[serde(default)]
    pub deep: usize,
    /// Concurrency cap for `normal`-tier agents; `0` when omitted.
    #[serde(default)]
    pub normal: usize,
    /// Concurrency cap for `basic`-tier agents; `0` when omitted.
    #[serde(default)]
    pub basic: usize,
}

/// One usage quota attached to an entry point: how much may be consumed and
/// over which window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    /// Quota ceiling for the window, counted in the unit named by
    /// `billing_metric`.
    pub data_limit: u64,
    /// Window length in hours; `None` when the payload omits it.
    #[serde(default)]
    pub period_hours: Option<u32>,
    /// Window length in days; `None` when the payload omits it.
    #[serde(default)]
    pub period_days: Option<u32>,
    /// Name of the metric `data_limit` counts (defined by the producing
    /// registry, not by this crate); empty string when omitted.
    #[serde(default)]
    pub billing_metric: String,
}

/// One provider as exposed by the config filesystem: localized naming, the
/// transport protocol, its entry points and the model catalog it offers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFsInfo {
    /// Provider id used to join entry points, models and API keys.
    pub id: String,
    /// Localized provider display names keyed by BCP 47 language tag.
    pub name: std::collections::HashMap<String, String>,
    /// Protocol the provider speaks, as recorded in the registry.
    pub protocol: String,
    /// Registry category label used for grouping in the TUI; empty string when
    /// unspecified.
    pub category: String,
    /// Provider-wide base URL; `None` when the payload carries no
    /// provider-level URL.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Entry points this provider exposes; empty when the payload omits them.
    #[serde(default)]
    pub entry_points: Vec<EntrypointConfigInfo>,
    /// Model catalog of this provider, as `ModelFsInfo` rows.
    #[serde(default)]
    pub models: Vec<ModelFsInfo>,
    /// Capability defaults for the provider as a whole; individual models carry
    /// their own flags in `ModelFsInfo`.
    #[serde(default)]
    pub capabilities: ProviderCapabilitiesInfo,
    /// Provider-level concurrency, rate and timeout limits.
    #[serde(default)]
    pub limits: ProviderLimitsInfo,
    /// Pricing-model label for the provider; the family's `PricingModel` enum in
    /// plana-config defines `one_time` / `periodic` / `pay_as_you_go`. Empty when
    /// unset.
    #[serde(default)]
    pub pricing_model: String,
}

/// Capability flags advertised at provider level; each deserializes to `false`
/// when the payload omits it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderCapabilitiesInfo {
    /// Whether the provider supports token-streaming responses.
    #[serde(default)]
    pub streaming: bool,
    /// Whether the provider supports tool/function calling.
    #[serde(default)]
    pub function_calling: bool,
    /// Whether the provider accepts image input.
    #[serde(default)]
    pub vision: bool,
    /// Whether the provider exposes reasoning/thinking output.
    #[serde(default)]
    pub reasoning: bool,
}

/// Provider-level request limits; every counter is `0` or `None` when the
/// payload omits it.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderLimitsInfo {
    /// Maximum concurrent requests for the provider; `0` when omitted.
    #[serde(default)]
    pub max_concurrent: u32,
    /// Requests-per-minute ceiling; `None` when the provider advertises no such
    /// cap.
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
    /// Request timeout budget in seconds; `0` when omitted.
    #[serde(default)]
    pub timeout_seconds: u64,
}

/// One model of a provider's catalog: context/output limits, capability flags
/// and the pricing data the TUI needs to offer and cost it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFsInfo {
    /// Model id this row describes; entry-point and user config reference it.
    pub id: String,
    /// Human-readable model name shown in the UI.
    pub name: String,
    /// Id of the owning provider (`ProviderFsInfo.id`).
    pub provider_id: String,
    /// Maximum prompt context, in tokens.
    pub context_window: u64,
    /// Maximum tokens the model may emit in a single response.
    pub max_output_tokens: u64,
    /// Whether the model accepts image input; `false` when omitted.
    #[serde(default)]
    pub supports_vision: bool,
    /// Whether the model supports tool/function calling; an absent key defaults
    /// to `true`.
    #[serde(default = "default_true")]
    pub supports_function_calling: bool,
    /// Whether the model supports streaming responses; an absent key defaults to
    /// `true`.
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    /// Whether the model exposes reasoning/thinking output; `false` when
    /// omitted.
    #[serde(default)]
    pub supports_reasoning: bool,
    /// Free-form tags used for filtering and grouping in the TUI; empty when
    /// omitted.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Per-token pricing, or `None` when the payload ships none (absent key or
    /// explicit `null`).
    pub pricing: Option<ModelFsPricing>,
    /// Multiplier applied on top of the base price (1.0 = list price); `None`
    /// when omitted.
    #[serde(default)]
    pub rate_multiplier: Option<f64>,
    /// Time-of-day pricing windows; empty means flat pricing at
    /// `rate_multiplier`.
    #[serde(default)]
    pub rate_rules: Vec<RateRuleInfo>,
}

/// Time-of-day pricing window for a model: inside it the peak multiplier
/// applies, outside it the off-peak multiplier does.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateRuleInfo {
    /// Semantics TBC — see packages/celestia-types/src/ws/services/llm_provider.rs:248.
    pub timezone_offset: i32,
    /// Start bound of the peak window; the wire carries no unit, so consumers
    /// must not assume hours.
    pub peak_start: u32,
    /// End bound of the peak window; same unitless convention as `peak_start`.
    pub peak_end: u32,
    /// Price multiplier applied to the model's base rate inside the peak
    /// window.
    pub peak_multiplier: f64,
    /// Price multiplier applied outside the peak window.
    pub off_peak_multiplier: f64,
}

fn default_true() -> bool {
    true
}

/// Per-million-token prices for a model, in USD — the same convention as the
/// pricing table in plana-llm-provider.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelFsPricing {
    /// USD per 1M input (prompt) tokens.
    #[serde(default)]
    pub input_per_million: Option<f64>,
    /// USD per 1M output (completion) tokens.
    #[serde(default)]
    pub output_per_million: Option<f64>,
    /// USD per 1M prompt-cache HIT tokens, i.e. the discounted rate for reused
    /// prompt prefixes.
    #[serde(default)]
    pub cached_per_million: Option<f64>,
}

/// User-level configuration mirror behind the TUI's settings screen
/// (`GetUserConfig` / `UpdateUserConfig`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// BCP 47 language tag selecting the UI language (e.g. `en`, `zh-Hans`).
    pub preferred_language: String,
    /// Default model id per key of the family's model-tier vocabulary
    /// (`basic` / `normal` / `deep`).
    pub default_models: std::collections::HashMap<String, String>,
    /// Provider ids the user has enabled.
    pub enabled_providers: Vec<String>,
    /// Model ids the user has enabled.
    pub enabled_models: Vec<String>,
    /// Priority weight per model id, used to order the enabled models.
    pub model_priorities: std::collections::HashMap<String, u32>,
    /// Environment variable names whose API keys are auto-imported into this
    /// user's config.
    pub auto_import_from_env: Vec<String>,
}

/// Metadata about one stored API key, never the key itself: the sync socket
/// carries presence and labels only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Provider the key belongs to (`ProviderFsInfo.id`).
    pub provider: String,
    /// User-visible label of the key.
    pub display_name: String,
    /// Whether a key is currently stored for this provider — the key material
    /// itself is never sent.
    pub has_key: bool,
    /// Creation time as a producer-formatted string (no format is fixed on the
    /// wire).
    pub created_at: String,
    /// Time of the last write as a producer-formatted string.
    pub updated_at: String,
    /// Where the key came from (user input vs. environment import); a
    /// producer-defined label.
    pub source: String,
}

/// Optional metadata a caller may supply alongside `SaveApiKey` when storing a
/// key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Optional user-chosen label for the key; `None` when the caller supplies
    /// none.
    pub display_name: Option<String>,
    /// Producer-defined label for where the key comes from; same vocabulary as
    /// `KeyInfo.source`.
    pub source: String,
}
