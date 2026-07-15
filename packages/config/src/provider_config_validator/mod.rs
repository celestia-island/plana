//! Provider configuration validator
//!
//! Implements completeness validation for Provider TOML configuration, including:
//! - provider.id: non-empty and unique
//! - api.base_url: valid URL format
//! - models[].id: non-empty
//! - pricing.model: enum value check

pub mod fixes;
pub mod rules;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use fixes::validate_and_log_providers;

use super::{
    model_category::{GenerationParams, ModelCategory},
    provider_config::AvailablePeriodConfig,
};

fn default_true() -> bool {
    true
}

fn default_rate_multiplier() -> f64 {
    1.0
}

fn default_max_concurrent() -> usize {
    1
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailableProvider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub uuid: Uuid,
    #[serde(default)]
    pub api_key: String,
    pub base_url: Option<String>,
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

impl std::fmt::Debug for AvailableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvailableProvider")
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailableModel {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub api_model: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default)]
    pub is_custom: bool,
    pub priority: u32,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub context_window: Option<u64>,
    #[serde(default)]
    pub compression_threshold: Option<u64>,
    #[serde(default = "default_true")]
    pub has_per_usage: bool,
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
}

pub use crate::errors::ProviderValidationError;

/// Provider configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationErrorDetail>,
    pub warnings: Vec<ValidationWarningDetail>,
}

/// Validation error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    pub provider_id: Option<String>,
    pub field: String,
    pub message: String,
}

/// Validation warning detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarningDetail {
    pub provider_id: Option<String>,
    pub field: String,
    pub message: String,
}

impl ProviderValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<ValidationErrorDetail>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, field: String, message: String) -> Self {
        self.warnings.push(ValidationWarningDetail {
            provider_id: None,
            field,
            message,
        });
        self
    }
}

impl Default for ProviderValidationResult {
    fn default() -> Self {
        Self::success()
    }
}

/// Provider configuration validator trait
pub trait ProviderConfigValidator {
    fn validate_providers(
        &self,
        providers: &[AvailableProvider],
        models: &[AvailableModel],
    ) -> ProviderValidationResult;

    fn validate_provider(
        &self,
        provider: &AvailableProvider,
    ) -> Result<(), ProviderValidationError>;

    fn validate_model(
        &self,
        model: &AvailableModel,
        provider_id: &str,
    ) -> Result<(), ProviderValidationError>;
}

/// Default provider configuration validator implementation
#[derive(Debug, Default)]
pub struct DefaultProviderConfigValidator;

impl DefaultProviderConfigValidator {
    pub fn new() -> Self {
        Self
    }
}
