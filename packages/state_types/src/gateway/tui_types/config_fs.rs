use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointApiConfigInfo {
    pub protocol: String,
    pub base_url: String,
    pub chat_endpoint: String,
    #[serde(default)]
    pub models_endpoint: Option<String>,
    pub auth_type: String,
    #[serde(default)]
    pub auth_header: Option<String>,
    pub env_var: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointConfigInfo {
    pub id: String,
    pub name: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub billing_type: String,
    #[serde(default)]
    pub plan_tier: String,
    pub api: EntrypointApiConfigInfo,
    #[serde(default)]
    pub defaults: EntrypointDefaultsInfo,
    #[serde(default)]
    pub quotas: Vec<QuotaInfo>,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaxConcurrentInfo {
    #[serde(default)]
    pub deep: usize,
    #[serde(default)]
    pub normal: usize,
    #[serde(default)]
    pub basic: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub data_limit: u64,
    #[serde(default)]
    pub period_hours: Option<u32>,
    #[serde(default)]
    pub period_days: Option<u32>,
    #[serde(default)]
    pub billing_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFsInfo {
    pub id: String,
    pub name: std::collections::HashMap<String, String>,
    pub protocol: String,
    pub category: String,
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderLimitsInfo {
    #[serde(default)]
    pub max_concurrent: u32,
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default)]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFsInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u64,
    pub max_output_tokens: u64,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default = "default_true")]
    pub supports_function_calling: bool,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub pricing: Option<ModelFsPricing>,
    #[serde(default)]
    pub rate_multiplier: Option<f64>,
    #[serde(default)]
    pub rate_rules: Vec<RateRuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateRuleInfo {
    pub timezone_offset: i32,
    pub peak_start: u32,
    pub peak_end: u32,
    pub peak_multiplier: f64,
    pub off_peak_multiplier: f64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelFsPricing {
    #[serde(default)]
    pub input_per_million: Option<f64>,
    #[serde(default)]
    pub output_per_million: Option<f64>,
    #[serde(default)]
    pub cached_per_million: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub preferred_language: String,
    pub default_models: std::collections::HashMap<String, String>,
    pub enabled_providers: Vec<String>,
    pub enabled_models: Vec<String>,
    pub model_priorities: std::collections::HashMap<String, u32>,
    pub auto_import_from_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub provider: String,
    pub display_name: String,
    pub has_key: bool,
    pub created_at: String,
    pub updated_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub display_name: Option<String>,
    pub source: String,
}
