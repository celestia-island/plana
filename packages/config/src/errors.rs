use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderValidationError {
    #[error("Provider ID cannot be empty")]
    EmptyProviderId,
    #[error("Provider ID '{0}' is duplicated")]
    DuplicateProviderId(String),
    #[error("Invalid base URL for provider '{provider}': {url} - {reason}")]
    InvalidBaseUrl {
        provider: String,
        url: String,
        reason: String,
    },
    #[error("Model ID cannot be empty for provider '{0}'")]
    EmptyModelId(String),
    #[error("Model ID '{model_id}' is duplicated for provider '{provider}'")]
    DuplicateModelId { provider: String, model_id: String },
    #[error(
        "Invalid pricing model '{0}' for provider '{1}', must be one of: pay_as_you_go, one_time, periodic, free"
    )]
    InvalidPricingModel(String, String),
    #[error("Invalid period config for provider '{provider}': {reason}")]
    InvalidPeriodConfig { provider: String, reason: String },
    #[error("Invalid URL format: {0}")]
    InvalidUrlFormat(String),
}
