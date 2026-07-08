use serde::{Deserialize, Serialize};

use arona_state_sync::ModelTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSubcallResult {
    pub success: bool,
    pub content: String,
    pub model_name: Option<String>,
    pub token_usage: Option<(u32, u32)>,
}

#[async_trait::async_trait]
pub trait LlmSubcallService: Send + Sync {
    async fn llm_chat(
        &self,
        tier: ModelTier,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> LlmSubcallResult;

    fn record_token_usage(
        &self,
        agent_id: &str,
        agent_type: arona_state_sync::Agent,
        model_name: Option<&str>,
        input_tokens: u32,
        output_tokens: u32,
    );
}
