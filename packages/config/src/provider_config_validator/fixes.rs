use tracing::{error, info, warn};

use super::{
    AvailableModel, AvailableProvider, DefaultProviderConfigValidator, ProviderConfigValidator,
    ProviderValidationResult,
};

/// Validate provider configuration and log detailed errors
pub fn validate_and_log_providers(
    providers: &[AvailableProvider],
    models: &[AvailableModel],
) -> ProviderValidationResult {
    let validator = DefaultProviderConfigValidator::new();
    let result = validator.validate_providers(providers, models);

    if !result.is_valid {
        error!("[ProviderConfigValidator] Provider configuration validation failed:");
        for error in &result.errors {
            error!(
                "  - Provider: {:?}, Field: {}, Error: {}",
                error.provider_id, error.field, error.message
            );
        }
    } else {
        info!(
            "[ProviderConfigValidator] Provider configuration validation passed ({} providers, {} models)",
            providers.len(),
            models.len()
        );
    }

    if !result.warnings.is_empty() {
        for warning in &result.warnings {
            warn!(
                "[ProviderConfigValidator] Warning - Provider: {:?}, Field: {}, Warning: {}",
                warning.provider_id, warning.field, warning.message
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AvailablePeriodConfig;
    use crate::{BillingType, PeriodUnit, StatsType};
    use anyhow::Result;

    #[test]
    fn test_validate_url_valid() -> Result<()> {
        assert!(DefaultProviderConfigValidator::validate_url("https://api.example.com").is_ok());
        assert!(DefaultProviderConfigValidator::validate_url("http://localhost:8080").is_ok());
        assert!(DefaultProviderConfigValidator::validate_url("").is_ok());
        Ok(())
    }

    #[test]
    fn test_validate_url_invalid() -> Result<()> {
        assert!(DefaultProviderConfigValidator::validate_url("ftp://example.com").is_err());
        assert!(DefaultProviderConfigValidator::validate_url("not-a-url").is_err());
        assert!(DefaultProviderConfigValidator::validate_url("://example.com").is_err());
        Ok(())
    }

    #[test]
    fn test_validate_providers_empty_id() -> Result<()> {
        let validator = DefaultProviderConfigValidator::new();
        let providers = vec![AvailableProvider {
            id: "".to_string(),
            name: "Test".to_string(),
            uuid: Default::default(),
            api_key: "test_key".to_string(),
            base_url: Some("https://api.example.com".to_string()),
            is_validated: false,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type: None,
            auth_header: None,
        }];
        let models = vec![];

        let result = validator.validate_providers(&providers, &models);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "provider.id");
        Ok(())
    }

    #[test]
    fn test_validate_providers_duplicate_id() -> Result<()> {
        let validator = DefaultProviderConfigValidator::new();
        let provider = AvailableProvider {
            id: "test".to_string(),
            name: "Test".to_string(),
            uuid: Default::default(),
            api_key: "test_key".to_string(),
            base_url: Some("https://api.example.com".to_string()),
            is_validated: false,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type: None,
            auth_header: None,
        };
        let providers = vec![provider.clone(), provider];
        let models = vec![];

        let result = validator.validate_providers(&providers, &models);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("duplicated"));
        Ok(())
    }

    #[test]
    fn test_validate_providers_invalid_url() -> Result<()> {
        let validator = DefaultProviderConfigValidator::new();
        let providers = vec![AvailableProvider {
            id: "test".to_string(),
            name: "Test".to_string(),
            uuid: Default::default(),
            api_key: "test_key".to_string(),
            base_url: Some("not-a-url".to_string()),
            is_validated: false,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type: None,
            auth_header: None,
        }];
        let models = vec![];

        let result = validator.validate_providers(&providers, &models);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "api.base_url");
        Ok(())
    }

    #[test]
    fn test_validate_models_empty_id() -> Result<()> {
        let validator = DefaultProviderConfigValidator::new();
        let providers = vec![AvailableProvider {
            id: "test_provider".to_string(),
            name: "Test".to_string(),
            uuid: Default::default(),
            api_key: "test_key".to_string(),
            base_url: Some("https://api.example.com".to_string()),
            is_validated: false,
            is_custom: false,
            entry_point_id: None,
            period_billing_configs: vec![],
            auth_type: None,
            auth_header: None,
        }];
        let models = vec![AvailableModel {
            id: "".to_string(),
            name: "Test Model".to_string(),
            provider_id: "test_provider".to_string(),
            api_model: "gpt-4".to_string(),
            is_enabled: true,
            is_custom: false,
            priority: 0,
            tier: "".to_string(),
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
            category: Default::default(),
            generation: None,
        }];

        let result = validator.validate_providers(&providers, &models);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "model.id");
        Ok(())
    }

    #[test]
    fn test_validate_period_config_valid() -> Result<()> {
        let config = AvailablePeriodConfig {
            id: "monthly".to_string(),
            name: "Monthly Plan".to_string(),
            billing_type: BillingType::Periodic,
            period_unit: PeriodUnit::Hours,
            period_hours: 720,
            period_start: None,
            stats_type: StatsType::Tokens,
            quota_limit: Some(1000000),
            quota_used: 0,
        };

        assert!(
            DefaultProviderConfigValidator::validate_period_config("test_provider", &config)
                .is_ok()
        );
        Ok(())
    }

    #[test]
    fn test_validate_period_config_defaults() -> Result<()> {
        let config = AvailablePeriodConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            billing_type: Default::default(),
            period_unit: Default::default(),
            period_hours: 0,
            period_start: None,
            stats_type: Default::default(),
            quota_limit: None,
            quota_used: 0,
        };

        let result =
            DefaultProviderConfigValidator::validate_period_config("test_provider", &config);
        assert!(result.is_ok());
        Ok(())
    }
}
