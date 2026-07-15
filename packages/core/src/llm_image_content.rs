use base64::Engine;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

const MEDIA_TYPE_PNG: &str = "image/png";
const MEDIA_TYPE_JPEG: &str = "image/jpeg";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmImageContent {
    pub media_type: String,
    #[serde(with = "super::utils::bytes_base64")]
    pub data: Bytes,
}

impl LlmImageContent {
    pub fn png(data: impl Into<Bytes>) -> Self {
        Self {
            media_type: MEDIA_TYPE_PNG.into(),
            data: data.into(),
        }
    }

    pub fn jpeg(data: impl Into<Bytes>) -> Self {
        Self {
            media_type: MEDIA_TYPE_JPEG.into(),
            data: data.into(),
        }
    }

    pub fn new(media_type: &str, data: impl Into<Bytes>) -> Self {
        Self {
            media_type: media_type.to_string(),
            data: data.into(),
        }
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.data)
    }
}
