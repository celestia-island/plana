pub mod openai;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;

pub use crate::errors::GenerationError;
use _config::model_category::GenerationModality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub modality: GenerationModality,
    pub prompt: String,
    pub model: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOutput {
    pub modality: GenerationModality,
    pub model: String,
    pub data: GenerationOutputData,
    pub usage: Option<GenerationUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerationOutputData {
    Image {
        url: Option<String>,
        base64: Option<String>,
        revised_prompt: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        format: String,
    },
    Audio {
        url: Option<String>,
        base64: Option<String>,
        duration_seconds: Option<f64>,
        format: String,
        sample_rate_hz: Option<u32>,
        voice: Option<String>,
    },
    Video {
        url: Option<String>,
        duration_seconds: Option<f64>,
        width: Option<u32>,
        height: Option<u32>,
        format: String,
    },
    Model3D {
        url: Option<String>,
        format: String,
        vertex_count: Option<u32>,
        face_count: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost: Option<f64>,
}

#[async_trait]
pub trait GenerationProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_modalities(&self) -> Vec<GenerationModality>;
    fn list_models(&self) -> Vec<String>;
    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationOutput, GenerationError>;
}

pub struct GenerationRegistry {
    providers: RwLock<HashMap<String, Arc<Box<dyn GenerationProvider>>>>,
}

impl GenerationRegistry {
    fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    pub fn global() -> Arc<Self> {
        static INSTANCE: std::sync::OnceLock<Arc<GenerationRegistry>> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Self::new())).clone()
    }

    pub fn register(&self, provider: Box<dyn GenerationProvider>) {
        let name = provider.provider_name().to_string();
        let mut providers = self.providers.write().unwrap_or_else(|e| e.into_inner());
        providers.insert(name, Arc::new(provider));
    }

    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn GenerationProvider>>> {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers.get(name).cloned()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers.keys().cloned().collect()
    }

    pub fn find_provider_for_model(&self, model: &str) -> Option<Arc<Box<dyn GenerationProvider>>> {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers
            .values()
            .find(|p| p.list_models().iter().any(|m| m == model))
            .cloned()
    }
}

pub fn register_all_generation_providers() {
    let registry = GenerationRegistry::global();
    registry.register(Box::new(openai::OpenAiGenerationProvider::new()));
}
