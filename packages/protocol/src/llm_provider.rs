//! LLM provider configuration & config-filesystem types.
//!
//! Covers provider lifecycle events, configured-provider catalog, usage
//! periods, the on-disk provider/model filesystem schema, user config, and
//! API-key registry info.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::PeriodType;

// ═══════════════════════════════════════════════════════════════
// LLM Provider configuration
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct LlmProviderConfiguredParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderRenamedParams {
    pub provider_name: String,
    pub new_display_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderEditedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderDeletedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ConfiguredProviderInfo {
    pub provider_name: String,
    pub display_name: String,
    #[serde(default)]
    #[ts(optional)]
    pub api_endpoint: Option<String>,
    pub default_model: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ConfiguredProvidersListParams {
    pub providers: Vec<ConfiguredProviderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ModelProviderConfigUpdatedParams {
    pub provider_name: String,
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct EndpointValidatedParams {
    pub provider_name: String,
    pub is_reachable: bool,
    #[serde(default)]
    #[ts(optional)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct UsagePeriodData {
    pub user_id: String,
    pub period_type: PeriodType,
    pub used_tokens: u64,
    pub cost: f64,
    pub start_time: String,
    #[serde(default)]
    #[ts(optional)]
    pub remaining_tokens: Option<u64>,
    #[serde(default)]
    #[ts(optional)]
    pub remaining_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct UsagePeriodResponseParams {
    pub data: Vec<UsagePeriodData>,
}

// ═══════════════════════════════════════════════════════════════
// Config Filesystem
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct EntrypointApiConfigInfo {
    pub protocol: String,
    pub base_url: String,
    pub chat_endpoint: String,
    #[serde(default)]
    #[ts(optional)]
    pub models_endpoint: Option<String>,
    pub auth_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub auth_header: Option<String>,
    pub env_var: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct MaxConcurrentInfo {
    #[serde(default)]
    pub deep: usize,
    #[serde(default)]
    pub normal: usize,
    #[serde(default)]
    pub basic: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct EntrypointDefaultsInfo {
    #[serde(default)]
    pub deep: Vec<String>,
    #[serde(default)]
    pub normal: Vec<String>,
    #[serde(default)]
    pub basic: Vec<String>,
    #[serde(default)]
    pub max_concurrent: MaxConcurrentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct QuotaInfo {
    pub data_limit: u64,
    #[serde(default)]
    #[ts(optional)]
    pub period_hours: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub period_days: Option<u32>,
    #[serde(default)]
    pub billing_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct EntrypointConfigInfo {
    pub id: String,
    #[serde(default)]
    pub name: std::collections::HashMap<String, String>,
    #[serde(default, rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub billing_type: String,
    #[serde(default)]
    pub plan_tier: String,
    pub api: EntrypointApiConfigInfo,
    #[serde(default)]
    pub defaults: EntrypointDefaultsInfo,
    #[serde(default)]
    pub quotas: Vec<QuotaInfo>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderCapabilitiesInfo {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub function_calling: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub reasoning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderLimitsInfo {
    #[serde(default)]
    pub max_concurrent: u32,
    #[serde(default)]
    #[ts(optional)]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default)]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderFsInfoParams {
    pub providers: Vec<ProviderFsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ModelFsPricing {
    #[serde(default)]
    #[ts(optional)]
    pub input_per_million: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub output_per_million: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub cached_per_million: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct RateRuleInfo {
    pub timezone_offset: i32,
    pub peak_start: u32,
    pub peak_end: u32,
    pub peak_multiplier: f64,
    pub off_peak_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ModelFsInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u64,
    pub max_output_tokens: u64,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default = "crate::default_true")]
    pub supports_function_calling: bool,
    #[serde(default = "crate::default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub pricing: Option<ModelFsPricing>,
    #[serde(default)]
    #[ts(optional)]
    pub rate_multiplier: Option<f64>,
    #[serde(default)]
    pub rate_rules: Vec<RateRuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ModelFsInfoParams {
    pub models: Vec<ModelFsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ProviderFsInfo {
    pub id: String,
    #[serde(default)]
    pub name: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    #[ts(optional)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub entry_points: Vec<EntrypointConfigInfo>,
    #[serde(default)]
    pub models: Vec<ModelFsInfo>,
    #[serde(default)]
    pub capabilities: ProviderCapabilitiesInfo,
    #[serde(default)]
    pub limits: ProviderLimitsInfo,
    #[serde(default)]
    pub pricing_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct UserConfigResponseParams {
    pub config: UserInfoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct UserInfoConfig {
    pub preferred_language: String,
    #[serde(default)]
    pub default_models: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub enabled_providers: Vec<String>,
    #[serde(default)]
    pub enabled_models: Vec<String>,
    #[serde(default)]
    pub model_priorities: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub auto_import_from_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct KeysListResponseParams {
    pub keys: Vec<KeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct KeyInfo {
    pub provider: String,
    pub display_name: String,
    pub has_key: bool,
    pub created_at: String,
    pub updated_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/llm_provider.ts")]
pub struct ApiKeyInfoResponseParams {
    pub info: KeyInfo,
}
