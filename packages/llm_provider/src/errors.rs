use thiserror::Error;

use _config::model_category::GenerationModality;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },
    #[error("Configuration error [{key}]: {reason}")]
    ConfigError { key: String, reason: String },
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Transient network error: {0}")]
    TransientNetworkError(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited {
        retry_after_secs: u64,
        body: Option<String>,
    },
    #[error("Invalid response: expected {expected}, got {got}")]
    InvalidResponse { expected: String, got: String },
}

#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("Generation request failed: {0}")]
    RequestFailed(String),
    #[error("Modality not supported: {0:?}")]
    ModalityNotSupported(GenerationModality),
    #[error("API error: {status} {message}")]
    ApiError { status: u16, message: String },
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LLM API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("LLM config error [{key}]: {reason}")]
    ConfigError { key: String, reason: String },

    #[error("LLM timeout")]
    Timeout,

    #[error("invalid response: expected {expected}, got {got}")]
    InvalidResponse { expected: String, got: String },

    #[error("HTTP error (status {status}): {endpoint}")]
    HttpError { status: u16, endpoint: String },

    #[error("provider not found: {provider_id}")]
    ProviderNotFound { provider_id: String },
}

impl From<ProviderError> for LlmError {
    fn from(e: ProviderError) -> Self {
        match e {
            ProviderError::ApiError { status, message } => LlmError::ApiError { status, message },
            ProviderError::ConfigError { key, reason } => LlmError::ConfigError { key, reason },
            ProviderError::NetworkError(msg) => LlmError::HttpError {
                status: 0,
                endpoint: msg,
            },
            ProviderError::TransientNetworkError(msg) => LlmError::HttpError {
                status: 0,
                endpoint: msg,
            },
            ProviderError::AuthFailed => LlmError::ApiError {
                status: 401,
                message: "authentication failed".to_string(),
            },
            ProviderError::RateLimited {
                retry_after_secs, ..
            } => LlmError::ApiError {
                status: 429,
                message: format!("rate limited, retry after {}s", retry_after_secs),
            },
            ProviderError::InvalidResponse { expected, got } => {
                LlmError::InvalidResponse { expected, got }
            }
        }
    }
}
