use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LlmProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub website_domain: String,
}

impl std::fmt::Debug for LlmProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmProviderConfig")
            .field("name", &self.name)
            .field("provider_type", &self.provider_type)
            .field("api_key", &"<REDACTED>")
            .field("model", &self.model)
            .field("endpoint", &self.endpoint)
            .finish()
    }
}
