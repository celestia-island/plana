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
