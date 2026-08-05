use base64::Engine;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

const MEDIA_TYPE_PNG: &str = "image/png";
const MEDIA_TYPE_JPEG: &str = "image/jpeg";

/// Positional role of an image in a multi-image modality (e.g. video
/// generation models that take 1-2 reference images).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmImagePosition {
    /// First frame / leading reference (video head).
    Head,
    /// Last frame / trailing reference (video tail).
    Tail,
    /// No positional constraint (any reference).
    #[default]
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmImageContent {
    pub media_type: String,
    #[serde(with = "super::utils::bytes_base64")]
    pub data: Bytes,
    /// Positional marker for multi-image modalities (head/tail frames).
    #[serde(default)]
    pub position: LlmImagePosition,
}

impl LlmImageContent {
    pub fn png(data: impl Into<Bytes>) -> Self {
        Self {
            media_type: MEDIA_TYPE_PNG.into(),
            data: data.into(),
            position: LlmImagePosition::Any,
        }
    }

    pub fn jpeg(data: impl Into<Bytes>) -> Self {
        Self {
            media_type: MEDIA_TYPE_JPEG.into(),
            data: data.into(),
            position: LlmImagePosition::Any,
        }
    }

    pub fn new(media_type: &str, data: impl Into<Bytes>) -> Self {
        Self {
            media_type: media_type.to_string(),
            data: data.into(),
            position: LlmImagePosition::Any,
        }
    }

    pub fn with_position(mut self, position: LlmImagePosition) -> Self {
        self.position = position;
        self
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.data)
    }
}

/// Audio content attached to a message — raw audio bytes that go straight
/// to an omni/realtime model without an intermediate ASR step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAudioContent {
    /// MIME type of the payload (e.g. "audio/wav", "audio/pcm").
    pub media_type: String,
    /// Sample rate in Hz when the payload is raw PCM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    #[serde(with = "super::utils::bytes_base64")]
    pub data: Bytes,
}

impl LlmAudioContent {
    pub fn new(media_type: &str, data: impl Into<Bytes>) -> Self {
        Self {
            media_type: media_type.to_string(),
            sample_rate: None,
            data: data.into(),
        }
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_audio_content_round_trip() {
        let audio = LlmAudioContent::new("audio/pcm", bytes::Bytes::from_static(b"\x00\x01\x02"))
            .with_sample_rate(16_000);
        let json = serde_json::to_string(&audio).unwrap();
        let back: LlmAudioContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.media_type, "audio/pcm");
        assert_eq!(back.sample_rate, Some(16_000));
        assert_eq!(back.to_base64(), audio.to_base64());
    }

    #[test]
    fn llm_audio_content_sample_rate_optional() {
        let audio = LlmAudioContent::new("audio/wav", b"RIFF".to_vec());
        let json = serde_json::to_string(&audio).unwrap();
        assert!(
            !json.contains("sample_rate"),
            "optional field omitted: {json}"
        );
        let back: LlmAudioContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sample_rate, None);
    }
}
