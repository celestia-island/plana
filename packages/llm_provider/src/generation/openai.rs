use anyhow::{Context, Result};
use base64::Engine;
use serde::Serialize;

use async_trait::async_trait;
use reqwest::Client;

use super::{
    GenerationError, GenerationOutput, GenerationOutputData, GenerationProvider, GenerationRequest,
};
use arona_config::model_category::GenerationModality;

#[derive(Serialize)]
struct ImageGenBody<'a> {
    model: &'a str,
    prompt: &'a str,
    size: &'a str,
    n: u32,
    response_format: &'a str,
}

#[derive(Serialize)]
struct AudioGenBody<'a> {
    model: &'a str,
    input: &'a str,
    voice: &'a str,
    response_format: &'a str,
}

#[derive(Serialize)]
struct VideoGenBody<'a> {
    model: &'a str,
    prompt: &'a str,
    size: &'a str,
}

pub struct OpenAiGenerationProvider {
    client: Client,
}

impl Default for OpenAiGenerationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAiGenerationProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn build_headers(&self, api_key: &str) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key)
                .parse()
                .context("invalid Bearer header value")?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json"
                .parse()
                .context("invalid Content-Type header value")?,
        );
        Ok(headers)
    }

    fn resolve_base_url(params: &serde_json::Value) -> String {
        params
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.openai.com/v1")
            .to_string()
    }

    fn resolve_api_key(params: &serde_json::Value) -> String {
        params
            .get("api_key")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    async fn generate_image(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationOutput, GenerationError> {
        let base_url = Self::resolve_base_url(&request.params);
        let api_key = Self::resolve_api_key(&request.params);
        let url = format!("{}/images/generations", base_url);
        let size = request
            .params
            .get("size")
            .and_then(|v| v.as_str())
            .unwrap_or("1024x1024");
        let n = request
            .params
            .get("num_images")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        let format = request
            .params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("png");

        let body = serde_json::to_value(ImageGenBody {
            model: &request.model,
            prompt: &request.prompt,
            size,
            n,
            response_format: "url",
        })
        .unwrap_or_default();

        let response = self
            .client
            .post(&url)
            .headers(
                self.build_headers(&api_key)
                    .map_err(|e| GenerationError::RequestFailed(e.to_string()))?,
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(GenerationError::ApiError {
                status,
                message: text,
            });
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;

        let images = result
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        let first_image = images.first().cloned().unwrap_or_default();
        let image_url = first_image
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from);
        let b64 = first_image
            .get("b64_json")
            .and_then(|v| v.as_str())
            .map(String::from);
        let revised_prompt = first_image
            .get("revised_prompt")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(GenerationOutput {
            modality: GenerationModality::Image,
            model: request.model.clone(),
            data: GenerationOutputData::Image {
                url: image_url,
                base64: b64,
                revised_prompt,
                width: None,
                height: None,
                format: format.to_string(),
            },
            usage: None,
        })
    }

    async fn generate_audio(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationOutput, GenerationError> {
        let base_url = Self::resolve_base_url(&request.params);
        let api_key = Self::resolve_api_key(&request.params);
        let url = format!("{}/audio/speech", base_url);
        let voice = request
            .params
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("alloy");
        let format = request
            .params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("mp3");

        let body = serde_json::to_value(AudioGenBody {
            model: &request.model,
            input: &request.prompt,
            voice,
            response_format: format,
        })
        .unwrap_or_default();

        let response = self
            .client
            .post(&url)
            .headers(
                self.build_headers(&api_key)
                    .map_err(|e| GenerationError::RequestFailed(e.to_string()))?,
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(GenerationError::ApiError {
                status,
                message: text,
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes.as_ref());

        Ok(GenerationOutput {
            modality: GenerationModality::Audio,
            model: request.model.clone(),
            data: GenerationOutputData::Audio {
                url: None,
                base64: Some(b64),
                duration_seconds: None,
                format: format.to_string(),
                sample_rate_hz: None,
                voice: Some(voice.to_string()),
            },
            usage: None,
        })
    }

    async fn generate_video(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationOutput, GenerationError> {
        let base_url = Self::resolve_base_url(&request.params);
        let api_key = Self::resolve_api_key(&request.params);
        let url = format!("{}/video/generations", base_url);
        let resolution = request
            .params
            .get("resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("1920x1080");
        let format = request
            .params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("mp4");

        let body = serde_json::to_value(VideoGenBody {
            model: &request.model,
            prompt: &request.prompt,
            size: resolution,
        })
        .unwrap_or_default();

        let response = self
            .client
            .post(&url)
            .headers(
                self.build_headers(&api_key)
                    .map_err(|e| GenerationError::RequestFailed(e.to_string()))?,
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(GenerationError::ApiError {
                status,
                message: text,
            });
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GenerationError::RequestFailed(e.to_string()))?;

        let data = result.get("data").cloned().unwrap_or_default();
        let video_url = data.get("url").and_then(|v| v.as_str()).map(String::from);

        Ok(GenerationOutput {
            modality: GenerationModality::Video,
            model: request.model.clone(),
            data: GenerationOutputData::Video {
                url: video_url,
                duration_seconds: None,
                width: None,
                height: None,
                format: format.to_string(),
            },
            usage: None,
        })
    }
}

#[async_trait]
impl GenerationProvider for OpenAiGenerationProvider {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn supported_modalities(&self) -> Vec<GenerationModality> {
        vec![
            GenerationModality::Image,
            GenerationModality::Audio,
            GenerationModality::Video,
        ]
    }

    fn list_models(&self) -> Vec<String> {
        vec![
            "dall-e-3".to_string(),
            "gpt-image-1".to_string(),
            "tts-1".to_string(),
            "tts-1-hd".to_string(),
            "sora".to_string(),
        ]
    }

    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationOutput, GenerationError> {
        match request.modality {
            GenerationModality::Image => self.generate_image(&request).await,
            GenerationModality::Audio => self.generate_audio(&request).await,
            GenerationModality::Video => self.generate_video(&request).await,
            GenerationModality::Model3D => Err(GenerationError::ModalityNotSupported(
                GenerationModality::Model3D,
            )),
        }
    }
}
