//! LLM Provider abstraction layer
//!
//! Defines the [`LlmProvider`] trait, provider implementations (OpenAI, Anthropic, Gemini),
//! message types, tool calls, streaming, and the [`ProviderConfig`] for authentication.
//!
//! ## Design documentation
//!
//! See [`docs/design/en/llm-provider-config.md`](https://github.com/celestia-island/entelecheia/blob/main/docs/design/en/llm-provider-config.md)
//! for detailed design docs covering: Provider TOML configuration, Metadata Management,
//! and ProviderScratch Layer3 agent architecture.
#![allow(clippy::type_complexity)]

pub mod anthropic;
pub mod errors;
pub mod gemini;
pub mod generation;
pub mod metering;
pub mod model_router;
pub mod openai;
pub mod openai_responses;
pub mod quota_meter;
pub mod registry;
pub mod rpc;
pub mod sse_util;
pub mod verification;

use futures::Stream;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, pin::Pin, time::Duration};

pub use anthropic::AnthropicProvider;
use async_trait::async_trait;
pub use errors::{LlmError, ProviderError};
pub use gemini::GeminiProvider;
pub use generation::{
    GenerationError, GenerationOutput, GenerationOutputData, GenerationProvider,
    GenerationRegistry, GenerationRequest, GenerationUsage, register_all_generation_providers,
};
pub use openai::{
    OpenAiCompatibleProvider, OpenAiFunction, OpenAiMessage, OpenAiRequest, OpenAiResponse,
    OpenAiToolCall, OpenAiUsageResponse,
};
pub use openai_responses::OpenAiResponsesProvider;
pub use registry::ProviderRegistry;
pub use rpc::RpcProvider;
use tracing::warn;
pub use verification::{ContentIntegrity, ContentVerification, VerificationStatus};

pub(crate) use _config::GenProtocol;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlContent },
    #[serde(rename = "image_base64")]
    ImageBase64 { media_type: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrlContent {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    #[default]
    Stop,
    Length,
    MaxTokens,
    ToolUse,
    ContentFilter,
    #[serde(other)]
    Unknown,
}

impl FinishReason {
    pub fn is_truncated(self) -> bool {
        matches!(self, Self::Length | Self::MaxTokens)
    }
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::Length => write!(f, "length"),
            Self::MaxTokens => write!(f, "max_tokens"),
            Self::ToolUse => write!(f, "tool_use"),
            Self::ContentFilter => write!(f, "content_filter"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for FinishReason {
    fn from(s: &str) -> Self {
        match s {
            "stop" | "end_turn" | "STOP" => Self::Stop,
            "length" => Self::Length,
            "max_tokens" => Self::MaxTokens,
            "tool_use" | "tool_calls" => Self::ToolUse,
            "content_filter" => Self::ContentFilter,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for MessageRole {
    fn from(s: &str) -> Self {
        match s {
            "system" => Self::System,
            "user" => Self::User,
            "assistant" => Self::Assistant,
            "tool" => Self::Tool,
            _ => Self::User,
        }
    }
}

pub use _core::LlmImageContent;

pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HEADER_AUTHORIZATION: &str = "Authorization";
pub const APPLICATION_JSON: &str = "application/json";

pub(crate) const CONFIG_KEY_HTTP_CLIENT: &str = "http_client";
pub(crate) const EXPECTED_VALID_JSON: &str = "valid JSON response";

pub(crate) const ROLE_USER: &str = "user";
pub(crate) const ROLE_ASSISTANT: &str = "assistant";
pub(crate) const ROLE_MODEL: &str = "model";
pub(crate) const TYPE_FUNCTION: &str = "function";
pub(crate) const MODE_AUTO: &str = "AUTO";
pub(crate) const CHOICE_AUTO: &str = "auto";
pub(crate) const SOURCE_TYPE_BASE64: &str = "base64";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Named(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<LlmImageContent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_blocks: Option<Vec<ContentBlock>>,
}

impl LlmMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.to_string()),
            tool_call_id: None,
            tool_calls: None,
            images: None,
            content_blocks: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.to_string()),
            tool_call_id: None,
            tool_calls: None,
            images: None,
            content_blocks: None,
        }
    }

    pub fn user_with_images(content: &str, images: Vec<LlmImageContent>) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.to_string()),
            tool_call_id: None,
            tool_calls: None,
            images: Some(images),
            content_blocks: None,
        }
    }

    pub fn user_multimodal(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: MessageRole::User,
            content: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
            content_blocks: Some(blocks),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.to_string()),
            tool_call_id: None,
            tool_calls: None,
            images: None,
            content_blocks: None,
        }
    }

    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
            images: None,
            content_blocks: None,
        }
    }

    pub fn tool(tool_call_id: &str, content: &str) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content.to_string()),
            tool_call_id: Some(tool_call_id.to_string()),
            tool_calls: None,
            images: None,
            content_blocks: None,
        }
    }

    pub fn tool_ack(tool_call_id: &str, ack: ToolResultAck) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(serde_json::to_string(&ack).unwrap_or_else(|e| {
                warn!(error=%e, "tool ack serialization failed");
                "{}".to_string()
            })),
            tool_call_id: Some(tool_call_id.to_string()),
            tool_calls: None,
            images: None,
            content_blocks: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<ContentIntegrity>,
}

impl ToolCall {
    pub fn arguments_are_valid_json(&self) -> bool {
        if self.arguments.is_empty() {
            return false;
        }
        serde_json::from_str::<serde_json::Value>(&self.arguments).is_ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ToolResultAck {
    Delivered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u64>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_memory_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_memory_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

pub use _core::ToolDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<LlmUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<LlmImageContent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStreamChunk {
    pub content: Option<String>,
    pub tool_call: Option<ToolCallDelta>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<LlmUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<ContentIntegrity>,
}

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;
const DEFAULT_ANTHROPIC_TIMEOUT_SECS: u64 = 180;

/// Provider configuration for LLM API authentication.
///
/// The `api_key` field is intentionally `pub String` for serde compatibility.
/// Use [`ProviderConfig::api_key_str`] for read access. The [`Debug`] impl
/// redacts this field. Future versions will migrate to `SecretString`.
#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub extra_headers: HashMap<String, String>,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub protocol: Option<_config::GenProtocol>,
    #[serde(default)]
    pub request_timeout_secs: Option<u64>,
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("api_key", &"<REDACTED>")
            .field("base_url", &self.base_url)
            .field("default_model", &self.default_model)
            .field("auth_type", &self.auth_type)
            .field("protocol", &self.protocol)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .finish()
    }
}

impl ProviderConfig {
    /// Create a new provider config with the given API key.
    ///
    /// The key is stored as a plain `String`. For production deployments,
    /// ensure the key is scoped to the minimum necessary permissions.
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: None,
            default_model: None,
            extra_headers: HashMap::new(),
            auth_type: None,
            auth_header: None,
            protocol: None,
            request_timeout_secs: None,
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }

    pub fn with_default_model(mut self, model: &str) -> Self {
        self.default_model = Some(model.to_string());
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.extra_headers
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_auth(mut self, auth_type: &str, auth_header: &str) -> Self {
        self.auth_type = Some(auth_type.to_string());
        self.auth_header = Some(auth_header.to_string());
        self
    }

    pub fn with_protocol(mut self, protocol: _config::GenProtocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn with_request_timeout(mut self, secs: u64) -> Self {
        self.request_timeout_secs = Some(secs);
        self
    }

    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(
            self.request_timeout_secs
                .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS),
        )
    }

    pub fn timeout_secs(&self) -> u64 {
        self.request_timeout_secs
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS)
    }

    /// Read the API key string.
    ///
    /// Prefer this over direct field access to make future migrations
    /// to a secret-managed type easier.
    pub fn api_key_str(&self) -> &str {
        &self.api_key
    }

    fn resolve_auth(&self) -> (&str, String) {
        let api_key = &self.api_key;
        if let (Some(at), Some(ah)) = (&self.auth_type, &self.auth_header) {
            match at.as_str() {
                "bearer" => (HEADER_AUTHORIZATION, format!("Bearer {api_key}")),
                _ => (ah.as_str(), api_key.clone()),
            }
        } else if let Some(proto) = &self.protocol {
            (proto.auth_header_name(), proto.auth_header_value(api_key))
        } else {
            (HEADER_AUTHORIZATION, format!("Bearer {api_key}"))
        }
    }

    pub fn build_authenticated_post(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> reqwest::RequestBuilder {
        let mut req = client
            .post(url)
            .header(HEADER_CONTENT_TYPE, APPLICATION_JSON);

        let (header_name, header_value) = self.resolve_auth();
        req = req.header(header_name, header_value);

        for (key, value) in &self.extra_headers {
            req = req.header(key, value);
        }
        req
    }
}

pub type StreamResult =
    Pin<Box<dyn Stream<Item = Result<LlmStreamChunk, crate::errors::ProviderError>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn default_model(&self) -> &str;

    fn supports_tools(&self) -> bool {
        false
    }
    fn supports_vision(&self) -> bool {
        false
    }
    fn supports_multimodal(&self) -> bool {
        self.supports_vision()
    }
    fn supports_streaming(&self) -> bool {
        true
    }

    fn resolve_model(&self, request: &LlmChatRequest, config: &ProviderConfig) -> String {
        if request.model.is_empty() {
            config
                .default_model
                .clone()
                .unwrap_or_else(|| self.default_model().to_string())
        } else {
            request.model.clone()
        }
    }

    fn create_http_client(
        &self,
        timeout: Duration,
    ) -> Result<reqwest::Client, crate::errors::ProviderError> {
        reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| crate::errors::ProviderError::ConfigError {
                key: CONFIG_KEY_HTTP_CLIENT.into(),
                reason: format!("Failed to create HTTP client: {}", e),
            })
    }

    fn handle_http_error(
        &self,
        status: u16,
        body: String,
        retry_after: Option<u64>,
    ) -> crate::errors::ProviderError {
        if status == 401 || status == 403 {
            return crate::errors::ProviderError::AuthFailed;
        }
        if status == 429 || status == 529 {
            return crate::errors::ProviderError::RateLimited {
                retry_after_secs: retry_after.unwrap_or(60),
                body: Some(body),
            };
        }
        crate::errors::ProviderError::ApiError {
            status,
            message: body,
        }
    }

    async fn chat(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<LlmChatResponse, crate::errors::ProviderError>;

    async fn chat_stream(
        &self,
        request: LlmChatRequest,
        config: &ProviderConfig,
    ) -> Result<StreamResult, crate::errors::ProviderError>;

    fn list_models(&self) -> Vec<String>;
}

pub fn register_all_providers() {
    let registry = ProviderRegistry::global();
    use GenProtocol::*;
    registry.register(
        OpenAIChatV1.as_str(),
        Box::new(OpenAiCompatibleProvider::new()),
    );
    registry.register(
        OpenAIResponsesV1.as_str(),
        Box::new(OpenAiResponsesProvider::new()),
    );
    registry.register(
        AnthropicMessagesV1.as_str(),
        Box::new(AnthropicProvider::new()),
    );
    registry.register(
        AnthropicMessagesV2.as_str(),
        Box::new(AnthropicProvider::new()),
    );
    registry.register(GeminiGenerateV1.as_str(), Box::new(GeminiProvider::new()));
    registry.register(RpcV1.as_str(), Box::new(RpcProvider::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result, anyhow};

    #[test]
    fn test_finish_reason_is_truncated() -> Result<()> {
        assert!(FinishReason::Length.is_truncated());
        assert!(FinishReason::MaxTokens.is_truncated());
        assert!(!FinishReason::Stop.is_truncated());
        assert!(!FinishReason::ToolUse.is_truncated());
        assert!(!FinishReason::ContentFilter.is_truncated());
        Ok(())
    }

    #[test]
    fn test_finish_reason_default() -> Result<()> {
        assert_eq!(FinishReason::default(), FinishReason::Stop);
        Ok(())
    }

    #[test]
    fn test_finish_reason_display() -> Result<()> {
        assert_eq!(format!("{}", FinishReason::Stop), "stop");
        assert_eq!(format!("{}", FinishReason::ToolUse), "tool_use");
        Ok(())
    }

    #[test]
    fn test_finish_reason_from_str() -> Result<()> {
        assert_eq!(FinishReason::from("stop"), FinishReason::Stop);
        assert_eq!(FinishReason::from("end_turn"), FinishReason::Stop);
        assert_eq!(FinishReason::from("STOP"), FinishReason::Stop);
        assert_eq!(FinishReason::from("length"), FinishReason::Length);
        assert_eq!(FinishReason::from("tool_calls"), FinishReason::ToolUse);
        assert_eq!(
            FinishReason::from("content_filter"),
            FinishReason::ContentFilter
        );
        assert_eq!(FinishReason::from("something_else"), FinishReason::Unknown);
        Ok(())
    }

    #[test]
    fn test_finish_reason_serde() -> Result<()> {
        let json = serde_json::to_string(&FinishReason::ToolUse)?;
        assert_eq!(json, "\"tool_use\"");
        let back: FinishReason = serde_json::from_str(&json)?;
        assert_eq!(back, FinishReason::ToolUse);
        Ok(())
    }

    #[test]
    fn test_finish_reason_serde_unknown_fallback() -> Result<()> {
        let back: FinishReason = serde_json::from_str("\"nonexistent_reason\"")?;
        assert_eq!(back, FinishReason::Unknown);
        Ok(())
    }

    #[test]
    fn test_message_role_as_str() -> Result<()> {
        assert_eq!(MessageRole::System.as_str(), "system");
        assert_eq!(MessageRole::User.as_str(), "user");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(MessageRole::Tool.as_str(), "tool");
        Ok(())
    }

    #[test]
    fn test_message_role_display() -> Result<()> {
        assert_eq!(format!("{}", MessageRole::User), "user");
        Ok(())
    }

    #[test]
    fn test_provider_config_new_redacts_debug() -> Result<()> {
        let config = ProviderConfig::new("sk-super-secret-key-12345");
        let debug_str = format!("{:?}", config);
        assert!(
            !debug_str.contains("sk-super-secret-key-12345"),
            "Debug must not leak api_key"
        );
        assert!(
            debug_str.contains("<REDACTED>"),
            "Debug should show REDACTED"
        );
        Ok(())
    }

    #[test]
    fn test_provider_config_api_key_str() -> Result<()> {
        let config = ProviderConfig::new("test-key-456");
        assert_eq!(config.api_key_str(), "test-key-456");
        Ok(())
    }

    #[test]
    fn test_provider_config_serde_roundtrip() -> Result<()> {
        let config = ProviderConfig::new("serde-key-789")
            .with_base_url("https://api.example.com")
            .with_default_model("gpt-4")
            .with_request_timeout(30);
        let json = serde_json::to_string(&config)?;
        assert!(
            json.contains("serde-key-789"),
            "Serialization should include api_key"
        );
        let deserialized: ProviderConfig = serde_json::from_str(&json)?;
        assert_eq!(deserialized.api_key_str(), "serde-key-789");
        assert_eq!(
            deserialized.base_url.as_deref(),
            Some("https://api.example.com")
        );
        Ok(())
    }

    #[test]
    fn test_provider_config_build_authenticated_post() -> Result<()> {
        let config = ProviderConfig::new("bearer-token").with_base_url("https://api.example.com");
        let client = reqwest::Client::new();
        let req = config.build_authenticated_post(&client, "https://api.example.com/chat");
        let built_req = req.build()?;
        let auth_value = built_req
            .headers()
            .get("Authorization")
            .ok_or_else(|| anyhow!("missing Authorization header"))?
            .to_str()
            .context("invalid Authorization header value")?;
        assert_eq!(auth_value, "Bearer bearer-token");
        Ok(())
    }

    #[test]
    fn test_provider_config_timeout_defaults() -> Result<()> {
        let config = ProviderConfig::new("test");
        assert_eq!(config.timeout_secs(), DEFAULT_REQUEST_TIMEOUT_SECS);
        assert_eq!(
            config.timeout_duration(),
            Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS)
        );

        let config = ProviderConfig::new("test").with_request_timeout(60);
        assert_eq!(config.timeout_secs(), 60);
        assert_eq!(config.timeout_duration(), Duration::from_secs(60));
        Ok(())
    }
}
