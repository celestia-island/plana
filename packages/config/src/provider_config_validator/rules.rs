use tracing::warn;

use super::{
    super::provider_config::AvailablePeriodConfig, AvailableModel, AvailableProvider,
    DefaultProviderConfigValidator, ProviderConfigValidator, ProviderValidationError,
    ValidationErrorDetail, ValidationWarningDetail,
};

const HTTP_SCHEME: &str = "http://";
const HTTPS_SCHEME: &str = "https://";

impl DefaultProviderConfigValidator {
    pub fn validate_url(url: &str) -> Result<(), ProviderValidationError> {
        if url.is_empty() {
            return Ok(());
        }

        if !url.starts_with(HTTP_SCHEME) && !url.starts_with(HTTPS_SCHEME) {
            return Err(ProviderValidationError::InvalidUrlFormat(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        if let Err(e) = url::Url::parse(url) {
            return Err(ProviderValidationError::InvalidUrlFormat(format!(
                "Invalid URL format: {}",
                e
            )));
        }

        Ok(())
    }

    pub fn validate_period_config(
        provider_id: &str,
        config: &AvailablePeriodConfig,
    ) -> Result<(), ProviderValidationError> {
        if config.id.is_empty() {
            return Err(ProviderValidationError::InvalidPeriodConfig {
                provider: provider_id.to_string(),
                reason: "Period config ID cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

impl ProviderConfigValidator for DefaultProviderConfigValidator {
    fn validate_providers(
        &self,
        providers: &[AvailableProvider],
        models: &[AvailableModel],
    ) -> super::ProviderValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut seen_provider_ids = std::collections::HashSet::new();
        let mut seen_model_ids = std::collections::HashSet::new();

        for provider in providers {
            if provider.id.is_empty() {
                errors.push(ValidationErrorDetail {
                    provider_id: None,
                    field: "provider.id".to_string(),
                    message: "Provider ID cannot be empty".to_string(),
                });
                continue;
            }

            if !seen_provider_ids.insert(&provider.id) {
                errors.push(ValidationErrorDetail {
                    provider_id: Some(provider.id.clone()),
                    field: "provider.id".to_string(),
                    message: format!("Provider ID '{}' is duplicated", provider.id),
                });
            }

            if let Some(ref base_url) = provider.base_url
                && let Err(e) = Self::validate_url(base_url)
            {
                errors.push(ValidationErrorDetail {
                    provider_id: Some(provider.id.clone()),
                    field: "api.base_url".to_string(),
                    message: format!("Invalid base URL: {}", e),
                });
            }

            for period_config in &provider.period_billing_configs {
                if let Err(e) = Self::validate_period_config(&provider.id, period_config) {
                    errors.push(ValidationErrorDetail {
                        provider_id: Some(provider.id.clone()),
                        field: "period_billing_configs".to_string(),
                        message: e.to_string(),
                    });
                }
            }

            if !provider.is_validated && !provider.api_key.is_empty() {
                warnings.push(ValidationWarningDetail {
                    provider_id: Some(provider.id.clone()),
                    field: "is_validated".to_string(),
                    message: "Provider has API key but is not validated".to_string(),
                });
            }
        }

        for model in models {
            if model.id.is_empty() {
                errors.push(ValidationErrorDetail {
                    provider_id: Some(model.provider_id.clone()),
                    field: "model.id".to_string(),
                    message: "Model ID cannot be empty".to_string(),
                });
                continue;
            }

            let model_key = format!("{}:{}", model.provider_id, model.id);
            if !seen_model_ids.insert(model_key.clone()) {
                errors.push(ValidationErrorDetail {
                    provider_id: Some(model.provider_id.clone()),
                    field: "model.id".to_string(),
                    message: format!(
                        "Model ID '{}' is duplicated for provider '{}'",
                        model.id, model.provider_id
                    ),
                });
            }

            if !providers.iter().any(|p| p.id == model.provider_id) {
                warnings.push(ValidationWarningDetail {
                    provider_id: Some(model.provider_id.clone()),
                    field: "model.provider_id".to_string(),
                    message: format!(
                        "Model '{}' references non-existent provider '{}'",
                        model.id, model.provider_id
                    ),
                });
            }

            if model.is_enabled && model.api_model.is_empty() {
                errors.push(ValidationErrorDetail {
                    provider_id: Some(model.provider_id.clone()),
                    field: "model.api_model".to_string(),
                    message: format!("Enabled model '{}' must have api_model set", model.id),
                });
            }
        }

        if errors.is_empty() {
            super::ProviderValidationResult {
                is_valid: true,
                errors,
                warnings,
            }
        } else {
            super::ProviderValidationResult {
                is_valid: false,
                errors,
                warnings,
            }
        }
    }

    fn validate_provider(
        &self,
        provider: &AvailableProvider,
    ) -> Result<(), ProviderValidationError> {
        if provider.id.is_empty() {
            return Err(ProviderValidationError::EmptyProviderId);
        }

        if let Some(ref base_url) = provider.base_url
            && let Err(e) = Self::validate_url(base_url)
        {
            return Err(ProviderValidationError::InvalidBaseUrl {
                provider: provider.id.clone(),
                url: base_url.clone(),
                reason: e.to_string(),
            });
        }

        for period_config in &provider.period_billing_configs {
            Self::validate_period_config(&provider.id, period_config)?;
        }

        Ok(())
    }

    fn validate_model(
        &self,
        model: &AvailableModel,
        provider_id: &str,
    ) -> Result<(), ProviderValidationError> {
        if model.id.is_empty() {
            return Err(ProviderValidationError::EmptyModelId(
                provider_id.to_string(),
            ));
        }

        if model.is_enabled && model.api_model.is_empty() {
            warn!(
                "[ProviderConfigValidator] Enabled model '{}' for provider '{}' has empty api_model",
                model.id, provider_id
            );
        }

        Ok(())
    }
}
