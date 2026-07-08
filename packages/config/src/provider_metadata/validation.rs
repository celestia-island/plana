use super::{
    PricingModel, ProviderMetadata, UsageType, ValidationError, ValidationErrorType,
    ValidationResult, ValidationWarning,
};

const HTTP_SCHEME: &str = "http://";
const HTTPS_SCHEME: &str = "https://";

/// Provider metadata validator
pub struct ProviderMetadataValidator;

impl ProviderMetadataValidator {
    /// Validate provider metadata
    pub fn validate(metadata: &ProviderMetadata) -> ValidationResult {
        let mut result = ValidationResult::new();

        result = Self::validate_format(metadata, result);
        result = Self::validate_logic(metadata, result);
        result = Self::validate_completeness(metadata, result);
        result = Self::validate_consistency(metadata, result);

        result
    }

    fn validate_format(
        metadata: &ProviderMetadata,
        mut result: ValidationResult,
    ) -> ValidationResult {
        if !metadata
            .provider_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            result = result.with_error(ValidationError {
                error_type: ValidationErrorType::Format,
                field: "provider_id".to_string(),
                message: "provider_id must contain only letters, digits, underscores and hyphens"
                    .to_string(),
            });
        }

        if !metadata.api_endpoint.starts_with(HTTP_SCHEME)
            && !metadata.api_endpoint.starts_with(HTTPS_SCHEME)
        {
            result = result.with_error(ValidationError {
                error_type: ValidationErrorType::Format,
                field: "api_endpoint".to_string(),
                message: "api_endpoint must start with http:// or https://".to_string(),
            });
        }

        for model in &metadata.available_models {
            if model.model_id.is_empty() {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Format,
                    field: format!("models.{}.model_id", model.model_id),
                    message: "model_id must not be empty".to_string(),
                });
            }
        }

        result
    }

    fn validate_logic(
        metadata: &ProviderMetadata,
        mut result: ValidationResult,
    ) -> ValidationResult {
        for model in &metadata.available_models {
            if let Some(cw) = model.context_window {
                if cw < 1 {
                    result = result.with_error(ValidationError {
                        error_type: ValidationErrorType::Logic,
                        field: format!("models.{}.context_window", model.model_id),
                        message: "context_window must be greater than 0".to_string(),
                    });
                }
                if cw > 2_000_000 {
                    result = result.with_warning(ValidationWarning {
                        field: format!("models.{}.context_window", model.model_id),
                        message: "context_window is unusually large, please verify".to_string(),
                    });
                }
            }

            if let Some(mo) = model.max_output_tokens
                && mo < 1
            {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Logic,
                    field: format!("models.{}.max_output_tokens", model.model_id),
                    message: "max_output_tokens must be greater than 0".to_string(),
                });
            }

            if let Some(price) = model.price_input {
                if price < 0.0 {
                    result = result.with_error(ValidationError {
                        error_type: ValidationErrorType::Logic,
                        field: format!("models.{}.price_input", model.model_id),
                        message: "price_input must not be negative".to_string(),
                    });
                }
                if price > 1000.0 {
                    result = result.with_warning(ValidationWarning {
                        field: format!("models.{}.price_input", model.model_id),
                        message: "price_input is unusually high, verify unit is per-million tokens"
                            .to_string(),
                    });
                }
            }

            if let Some(price) = model.price_output
                && price < 0.0
            {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Logic,
                    field: format!("models.{}.price_output", model.model_id),
                    message: "price_output must not be negative".to_string(),
                });
            }
        }

        if let Some(ref rate_limit) = metadata.rate_limit
            && let Some(rpm) = rate_limit.requests_per_minute
            && rpm == 0
        {
            result = result.with_warning(ValidationWarning {
                field: "rate_limit.requests_per_minute".to_string(),
                message: "requests_per_minute of 0 means unlimited".to_string(),
            });
        }

        result
    }

    fn validate_completeness(
        metadata: &ProviderMetadata,
        mut result: ValidationResult,
    ) -> ValidationResult {
        if metadata.display_name.is_empty() {
            result = result.with_error(ValidationError {
                error_type: ValidationErrorType::Completeness,
                field: "display_name".to_string(),
                message: "display_name is required".to_string(),
            });
        }

        if metadata.api_endpoint.is_empty() {
            result = result.with_error(ValidationError {
                error_type: ValidationErrorType::Completeness,
                field: "api_endpoint".to_string(),
                message: "api_endpoint is required".to_string(),
            });
        }

        if metadata.available_models.is_empty() {
            result = result.with_warning(ValidationWarning {
                field: "available_models".to_string(),
                message: "available_models is empty, consider adding at least one model"
                    .to_string(),
            });
        }

        for model in &metadata.available_models {
            if model.model_name.is_empty() {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Completeness,
                    field: format!("models.{}.model_name", model.model_id),
                    message: "model_name is required".to_string(),
                });
            }
        }

        result
    }

    fn validate_consistency(
        metadata: &ProviderMetadata,
        mut result: ValidationResult,
    ) -> ValidationResult {
        if let Some(ref default_model) = metadata.default_model {
            let exists = metadata
                .available_models
                .iter()
                .any(|m| &m.model_id == default_model);
            if !exists {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Consistency,
                    field: "default_model".to_string(),
                    message: format!(
                        "default_model '{}' not found in available_models",
                        default_model
                    ),
                });
            }
        }

        match (metadata.pricing_model, metadata.usage_type) {
            (PricingModel::OneTime, UsageType::Metered) => {
                result = result.with_warning(ValidationWarning {
                    field: "pricing_model".to_string(),
                    message: "One-time pricing is typically not paired with metered usage"
                        .to_string(),
                });
            },
            (PricingModel::PayAsYouGo, UsageType::Unlimited) => {
                result = result.with_warning(ValidationWarning {
                    field: "pricing_model".to_string(),
                    message: "Pay-as-you-go is typically not paired with unlimited usage"
                        .to_string(),
                });
            },
            _ => {},
        }

        for model in &metadata.available_models {
            if let (Some(cw), Some(mo)) = (model.context_window, model.max_output_tokens)
                && mo > cw
            {
                result = result.with_error(ValidationError {
                    error_type: ValidationErrorType::Consistency,
                    field: format!("models.{}.max_output_tokens", model.model_id),
                    message: format!(
                        "max_output_tokens ({}) must not exceed context_window ({})",
                        mo, cw
                    ),
                });
            }
        }

        if metadata.usage_type == UsageType::Unlimited && metadata.quota.is_some() {
            result = result.with_warning(ValidationWarning {
                field: "quota".to_string(),
                message: "Usage type is Unlimited but quota is set".to_string(),
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_format_validation() -> Result<()> {
        let metadata = ProviderMetadata {
            provider_id: "invalid@id".to_string(),
            display_name: "Test Provider".to_string(),
            api_endpoint: "not-a-url".to_string(),
            pricing_model: PricingModel::PayAsYouGo,
            usage_type: UsageType::Metered,
            available_models: vec![],
            default_model: None,
            rate_limit: None,
            quota: None,
            source: super::super::MetadataSource::Official,
        };

        let result = ProviderMetadataValidator::validate(&metadata);
        assert!(!result.passed);
        assert!(result.errors.iter().any(|e| e.field == "provider_id"));
        assert!(result.errors.iter().any(|e| e.field == "api_endpoint"));
        Ok(())
    }

    #[test]
    fn test_logic_validation() -> Result<()> {
        let metadata = ProviderMetadata {
            provider_id: "test-provider".to_string(),
            display_name: "Test Provider".to_string(),
            api_endpoint: "https://api.example.com".to_string(),
            pricing_model: PricingModel::PayAsYouGo,
            usage_type: UsageType::Metered,
            available_models: vec![super::super::ModelMetadata {
                model_id: "test-model".to_string(),
                model_name: "Test Model".to_string(),
                context_window: Some(0),
                max_output_tokens: Some(100),
                supports_vision: false,
                supports_function_calling: false,
                supports_streaming: true,
                price_input: Some(-1.0),
                price_output: None,
            }],
            default_model: None,
            rate_limit: None,
            quota: None,
            source: super::super::MetadataSource::Official,
        };

        let result = ProviderMetadataValidator::validate(&metadata);
        assert!(!result.passed);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field.contains("context_window"))
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field.contains("price_input"))
        );
        Ok(())
    }

    #[test]
    fn test_consistency_validation() -> Result<()> {
        let metadata = ProviderMetadata {
            provider_id: "test-provider".to_string(),
            display_name: "Test Provider".to_string(),
            api_endpoint: "https://api.example.com".to_string(),
            pricing_model: PricingModel::PayAsYouGo,
            usage_type: UsageType::Metered,
            available_models: vec![super::super::ModelMetadata {
                model_id: "test-model".to_string(),
                model_name: "Test Model".to_string(),
                context_window: Some(1000),
                max_output_tokens: Some(2000),
                supports_vision: false,
                supports_function_calling: false,
                supports_streaming: true,
                price_input: None,
                price_output: None,
            }],
            default_model: Some("non-existent".to_string()),
            rate_limit: None,
            quota: None,
            source: super::super::MetadataSource::Official,
        };

        let result = ProviderMetadataValidator::validate(&metadata);
        assert!(!result.passed);
        assert!(result.errors.iter().any(|e| e.field == "default_model"));
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field.contains("max_output_tokens"))
        );
        Ok(())
    }
}
