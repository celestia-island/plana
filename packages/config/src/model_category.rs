use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum GenerationModality {
    Image,
    Audio,
    Video,
    Model3D,
}

impl std::fmt::Display for GenerationModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "image"),
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::Model3D => write!(f, "model3d"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ModelCategory {
    #[default]
    Chat,
    Generation(GenerationModality),
}

impl std::fmt::Display for ModelCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat => write!(f, "chat"),
            Self::Generation(m) => write!(f, "generation_{}", m),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct GenerationParams {
    pub modalities: Vec<GenerationModality>,
    #[serde(default)]
    pub default_format: Option<String>,
    #[serde(default)]
    pub maxarona_resolution: Option<String>,
    #[serde(default)]
    pub supported_formats: Vec<String>,
    #[serde(default)]
    pub supports_streaming: bool,
    #[serde(default)]
    pub input_modalities: Vec<String>,
}
