//! Generation Protocol system — flat enum for all generation protocols.
//!
//! Every protocol variant (LLM chat, image, video, audio, 3D) is a peer in
//! [`GenProtocol`]. What a protocol can *read* (input understanding) and
//! *generate* (output creation) is expressed through the [`ProtocolCapability`]
//! trait, which returns a compile-time static [`Capability`] slice.
//!
//! ## Capability hierarchy
//!
//! Capabilities are hierarchical: a base media type (e.g. `Audio`) can have
//! sub-kinds (`Speech`, `Music`). This is expressed via [`AudioKind`] and
//! [`ThreeDKind`] enums, not through flat variant explosion.
//!
//! ## Version numbers
//!
//! Protocol variant names carry version numbers. Underscores in code that
//! represent decimal points are restored to dots in all textual output
//! (`as_str`, `Display`, serde). For example `StabilityImageGenV3_5` serializes
//! as `"stability_image_gen_v3.5"`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ─── Audio sub-kind ────────────────────────────────────────────────

/// Sub-kind for audio capabilities: general audio, speech, or music.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioKind {
    General,
    Speech,
    Music,
}

impl AudioKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "audio",
            Self::Speech => "speech",
            Self::Music => "music",
        }
    }

    pub fn parse_opt(s: &str) -> Option<Self> {
        match s {
            "audio" => Some(Self::General),
            "speech" => Some(Self::Speech),
            "music" => Some(Self::Music),
            _ => None,
        }
    }
}

impl Serialize for AudioKind {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AudioKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse_opt(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown audio kind: {s}")))
    }
}

impl fmt::Display for AudioKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─── 3D sub-kind ───────────────────────────────────────────────────

/// Sub-kind for 3D generation capabilities: general mesh, Gaussian splat, or
/// topology map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreeDKind {
    General,
    GaussianSplat,
    TopologyMap,
}

impl ThreeDKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "3d",
            Self::GaussianSplat => "3d_gaussian_splat",
            Self::TopologyMap => "3d_topology_map",
        }
    }

    pub fn parse_opt(s: &str) -> Option<Self> {
        match s {
            "3d" => Some(Self::General),
            "3d_gaussian_splat" => Some(Self::GaussianSplat),
            "3d_topology_map" => Some(Self::TopologyMap),
            _ => None,
        }
    }
}

impl Serialize for ThreeDKind {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ThreeDKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse_opt(&s).ok_or_else(|| serde::de::Error::custom(format!("unknown 3d kind: {s}")))
    }
}

impl fmt::Display for ThreeDKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─── Capability ────────────────────────────────────────────────────

/// Protocol capability — what a protocol can read (input) and generate
/// (output), plus special abilities.
///
/// Hierarchical: media types like audio and 3D carry a sub-kind to distinguish
/// speech vs music, or Gaussian splat vs topology map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    // ── Read / input understanding ──
    ReadText,
    ReadImage,
    ReadVideo,
    ReadAudio(AudioKind),
    Read3D(ThreeDKind),

    // ── Generate / output creation ──
    GenerateText,
    GenerateImage,
    GenerateVideo,
    GenerateAudio(AudioKind),
    Generate3D(ThreeDKind),

    // ── Special abilities ──
    DeepReasoning,
    ToolCalling,
    Streaming,
    Embedding,
}

impl Capability {
    /// Whether this is a "read" (input understanding) capability.
    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Self::ReadText
                | Self::ReadImage
                | Self::ReadVideo
                | Self::ReadAudio(_)
                | Self::Read3D(_)
        )
    }

    /// Whether this is a "generate" (output creation) capability.
    pub fn is_generate(&self) -> bool {
        matches!(
            self,
            Self::GenerateText
                | Self::GenerateImage
                | Self::GenerateVideo
                | Self::GenerateAudio(_)
                | Self::Generate3D(_)
        )
    }

    /// Whether this is a special (non-media) capability.
    pub fn is_special(&self) -> bool {
        matches!(
            self,
            Self::DeepReasoning | Self::ToolCalling | Self::Streaming | Self::Embedding
        )
    }

    /// Textual identifier used in config files and logs.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadText => "read_text",
            Self::ReadImage => "read_image",
            Self::ReadVideo => "read_video",
            Self::ReadAudio(AudioKind::General) => "read_audio",
            Self::ReadAudio(AudioKind::Speech) => "read_audio_speech",
            Self::ReadAudio(AudioKind::Music) => "read_audio_music",
            Self::Read3D(ThreeDKind::General) => "read_3d",
            Self::Read3D(ThreeDKind::GaussianSplat) => "read_3d_gaussian_splat",
            Self::Read3D(ThreeDKind::TopologyMap) => "read_3d_topology_map",
            Self::GenerateText => "generate_text",
            Self::GenerateImage => "generate_image",
            Self::GenerateVideo => "generate_video",
            Self::GenerateAudio(AudioKind::General) => "generate_audio",
            Self::GenerateAudio(AudioKind::Speech) => "generate_audio_speech",
            Self::GenerateAudio(AudioKind::Music) => "generate_audio_music",
            Self::Generate3D(ThreeDKind::General) => "generate_3d",
            Self::Generate3D(ThreeDKind::GaussianSplat) => "generate_3d_gaussian_splat",
            Self::Generate3D(ThreeDKind::TopologyMap) => "generate_3d_topology_map",
            Self::DeepReasoning => "deep_reasoning",
            Self::ToolCalling => "tool_calling",
            Self::Streaming => "streaming",
            Self::Embedding => "embedding",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Capability {
    type Err = UnknownCapabilityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read_text" => Ok(Self::ReadText),
            "read_image" => Ok(Self::ReadImage),
            "read_video" => Ok(Self::ReadVideo),
            "read_audio" => Ok(Self::ReadAudio(AudioKind::General)),
            "read_audio_speech" => Ok(Self::ReadAudio(AudioKind::Speech)),
            "read_audio_music" => Ok(Self::ReadAudio(AudioKind::Music)),
            "read_3d" => Ok(Self::Read3D(ThreeDKind::General)),
            "read_3d_gaussian_splat" => Ok(Self::Read3D(ThreeDKind::GaussianSplat)),
            "read_3d_topology_map" => Ok(Self::Read3D(ThreeDKind::TopologyMap)),
            "generate_text" => Ok(Self::GenerateText),
            "generate_image" => Ok(Self::GenerateImage),
            "generate_video" => Ok(Self::GenerateVideo),
            "generate_audio" => Ok(Self::GenerateAudio(AudioKind::General)),
            "generate_audio_speech" => Ok(Self::GenerateAudio(AudioKind::Speech)),
            "generate_audio_music" => Ok(Self::GenerateAudio(AudioKind::Music)),
            "generate_3d" => Ok(Self::Generate3D(ThreeDKind::General)),
            "generate_3d_gaussian_splat" => Ok(Self::Generate3D(ThreeDKind::GaussianSplat)),
            "generate_3d_topology_map" => Ok(Self::Generate3D(ThreeDKind::TopologyMap)),
            "deep_reasoning" => Ok(Self::DeepReasoning),
            "tool_calling" => Ok(Self::ToolCalling),
            "streaming" => Ok(Self::Streaming),
            "embedding" => Ok(Self::Embedding),
            _ => Err(UnknownCapabilityError(s.to_string())),
        }
    }
}

// Custom serde for Capability — serializes as the flat as_str() form
impl Serialize for Capability {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown capability: {0}")]
pub struct UnknownCapabilityError(pub String);

// ─── ProtocolCapability trait ──────────────────────────────────────

/// Capability manifest for a protocol. Every [`GenProtocol`] variant must
/// return its full capability list as a compile-time static slice.
pub trait ProtocolCapability: Send + Sync {
    /// All capabilities of this protocol — both input (read) and output
    /// (generate). This is a `&'static` slice for zero-cost, zero-allocation
    /// introspection.
    fn capabilities(&self) -> &'static [Capability];

    /// Whether this protocol uses the submit → poll → result async pattern.
    fn is_async(&self) -> bool;

    /// Formal protocol name with version number. Decimal points are written as
    /// dots, not underscores (e.g. `"stability_image_gen_v3.5"`).
    fn name(&self) -> &'static str;
}

// ─── GenProtocol enum ──────────────────────────────────────────────

/// All generation protocols — flat enum.
///
/// LLM chat protocols, image/video/audio/3D generation protocols are all
/// peers. What they can do is expressed by [`ProtocolCapability`].
///
/// ## Naming convention
///
/// Variant names end with a version marker (`V1`, `V2`, `V3_5`). Underscores
/// in the version segment represent decimal points and are restored to dots in
/// textual output (`as_str` / `Display` / serde).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GenProtocol {
    // ═══ Text generation (LLM Chat) ═══
    #[default]
    OpenAIChatV1,
    OpenAIResponsesV1,
    AnthropicMessagesV1,
    AnthropicMessagesV2,
    GeminiGenerateV1,
    RpcV1,

    // ═══ Image generation ═══
    OpenAIImageGenV1,
    StabilityImageGenV3_5,
    JimengAsyncV1,
    GoogleImagenV1,

    // ═══ Video generation ═══
    Seedance2AsyncV1,
    KlingAsyncV3_0,
    Sora2AsyncV1,
    GoogleVeo3_1AsyncV1,

    // ═══ Audio generation ═══
    ElevenLabsTTSV1,
    ElevenLabsMusicV1,

    // ═══ 3D generation ═══
    Hyper3DAsyncV1,
    Hunyuan3DAsyncV1,
}

impl GenProtocol {
    /// All variants, for iteration and validation.
    pub const ALL: &'static [GenProtocol] = &[
        Self::OpenAIChatV1,
        Self::OpenAIResponsesV1,
        Self::AnthropicMessagesV1,
        Self::AnthropicMessagesV2,
        Self::GeminiGenerateV1,
        Self::RpcV1,
        Self::OpenAIImageGenV1,
        Self::StabilityImageGenV3_5,
        Self::JimengAsyncV1,
        Self::GoogleImagenV1,
        Self::Seedance2AsyncV1,
        Self::KlingAsyncV3_0,
        Self::Sora2AsyncV1,
        Self::GoogleVeo3_1AsyncV1,
        Self::ElevenLabsTTSV1,
        Self::ElevenLabsMusicV1,
        Self::Hyper3DAsyncV1,
        Self::Hunyuan3DAsyncV1,
    ];

    /// Formal name — underscores in version segments are restored to dots.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAIChatV1 => "openai_chat_v1",
            Self::OpenAIResponsesV1 => "openai_responses_v1",
            Self::AnthropicMessagesV1 => "anthropic_messages_v1",
            Self::AnthropicMessagesV2 => "anthropic_messages_v2",
            Self::GeminiGenerateV1 => "gemini_generate_v1",
            Self::RpcV1 => "rpc_v1",
            Self::OpenAIImageGenV1 => "openai_image_gen_v1",
            Self::StabilityImageGenV3_5 => "stability_image_gen_v3.5",
            Self::JimengAsyncV1 => "jimeng_async_v1",
            Self::GoogleImagenV1 => "google_imagen_v1",
            Self::Seedance2AsyncV1 => "seedance2_async_v1",
            Self::KlingAsyncV3_0 => "kling_async_v3.0",
            Self::Sora2AsyncV1 => "sora2_async_v1",
            Self::GoogleVeo3_1AsyncV1 => "google_veo_v3.1_async_v1",
            Self::ElevenLabsTTSV1 => "eleven_labs_tts_v1",
            Self::ElevenLabsMusicV1 => "eleven_labs_music_v1",
            Self::Hyper3DAsyncV1 => "hyper3d_async_v1",
            Self::Hunyuan3DAsyncV1 => "hunyuan_3d_async_v1",
        }
    }

    /// Human-friendly display name for UI.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::OpenAIChatV1 => "OpenAI Chat v1",
            Self::OpenAIResponsesV1 => "OpenAI Responses v1",
            Self::AnthropicMessagesV1 => "Anthropic Messages v1",
            Self::AnthropicMessagesV2 => "Anthropic Messages v2",
            Self::GeminiGenerateV1 => "Gemini Generate v1",
            Self::RpcV1 => "Plana RPC v1",
            Self::OpenAIImageGenV1 => "OpenAI Image Gen v1",
            Self::StabilityImageGenV3_5 => "Stability Image Gen v3.5",
            Self::JimengAsyncV1 => "Jimeng Async v1",
            Self::GoogleImagenV1 => "Google Imagen v1",
            Self::Seedance2AsyncV1 => "Seedance 2 Async v1",
            Self::KlingAsyncV3_0 => "Kling Async v3.0",
            Self::Sora2AsyncV1 => "Sora 2 Async v1",
            Self::GoogleVeo3_1AsyncV1 => "Google Veo v3.1 Async v1",
            Self::ElevenLabsTTSV1 => "ElevenLabs TTS v1",
            Self::ElevenLabsMusicV1 => "ElevenLabs Music v1",
            Self::Hyper3DAsyncV1 => "Hyper3D Async v1",
            Self::Hunyuan3DAsyncV1 => "Hunyuan 3D Async v1",
        }
    }

    /// Whether this protocol uses the submit → poll → result async pattern.
    pub fn is_async(self) -> bool {
        matches!(
            self,
            Self::JimengAsyncV1
                | Self::Seedance2AsyncV1
                | Self::KlingAsyncV3_0
                | Self::Sora2AsyncV1
                | Self::GoogleVeo3_1AsyncV1
                | Self::Hyper3DAsyncV1
                | Self::Hunyuan3DAsyncV1
        )
    }

    // ── LLM-specific auth/connection helpers ─────────────────────────
    //
    // These methods are meaningful only for LLM chat protocols. For
    // non-LLM protocols they return sensible defaults (empty string /
    // false) and callers should check `is_llm_chat()` before relying on
    // them.

    /// Whether this is an LLM chat protocol (text generation with auth
    /// headers).
    pub fn is_llm_chat(self) -> bool {
        matches!(
            self,
            Self::OpenAIChatV1
                | Self::OpenAIResponsesV1
                | Self::AnthropicMessagesV1
                | Self::AnthropicMessagesV2
                | Self::GeminiGenerateV1
                | Self::RpcV1
        )
    }

    /// Build a validation URL from the given base URL. Only meaningful for
    /// LLM chat protocols.
    pub fn validation_url(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        match self {
            Self::OpenAIChatV1 | Self::OpenAIResponsesV1 | Self::RpcV1 => {
                format!("{}/models", base)
            }
            Self::AnthropicMessagesV1 | Self::AnthropicMessagesV2 => {
                let base = base.trim_end_matches("/v1");
                format!("{}/v1/models", base)
            }
            Self::GeminiGenerateV1 => {
                let base = base
                    .trim_end_matches("/v1beta/models")
                    .trim_end_matches("/v1/models");
                format!("{}/v1beta/models", base)
            }
            _ => format!("{}/models", base),
        }
    }

    /// Whether this protocol passes the API key as a query parameter
    /// instead of a header (Gemini uses `?key=…`).
    pub fn uses_query_param_auth(&self) -> bool {
        matches!(self, Self::GeminiGenerateV1)
    }

    /// HTTP header name for the auth token.
    pub fn auth_header_name(&self) -> &'static str {
        match self {
            Self::OpenAIChatV1
            | Self::OpenAIResponsesV1
            | Self::GeminiGenerateV1
            | Self::OpenAIImageGenV1
            | Self::StabilityImageGenV3_5
            | Self::GoogleImagenV1 => "Authorization",
            Self::AnthropicMessagesV1 | Self::AnthropicMessagesV2 => "x-api-key",
            Self::ElevenLabsTTSV1 | Self::ElevenLabsMusicV1 => "xi-api-key",
            _ => "Authorization",
        }
    }

    /// HTTP header value for the auth token.
    pub fn auth_header_value(&self, api_key: &str) -> String {
        match self {
            Self::AnthropicMessagesV1
            | Self::AnthropicMessagesV2
            | Self::ElevenLabsTTSV1
            | Self::ElevenLabsMusicV1 => api_key.to_string(),
            _ => format!("Bearer {}", api_key),
        }
    }

    /// Resolve the protocol from a TOML `protocol` field or provider_id.
    ///
    /// When called with a `protocol` value already parsed from TOML, it returns
    /// the matching variant. When called with a raw `provider_id`, it falls
    /// back to a provider-id mapping.
    pub fn resolve(id: &str) -> Self {
        match Self::from_str(id) {
            Ok(p) => p,
            Err(_) => Self::from_provider_id(id),
        }
    }

    /// Map a raw `provider_id` to its protocol when no TOML data is available.
    fn from_provider_id(provider_id: &str) -> Self {
        match provider_id {
            "anthropic" | "anthropic_compatible" => Self::AnthropicMessagesV2,
            "google" | "gemini" | "gemini_compatible" => Self::GeminiGenerateV1,
            "rpc" => Self::RpcV1,
            _ => Self::OpenAIChatV1,
        }
    }
}

impl ProtocolCapability for GenProtocol {
    fn capabilities(&self) -> &'static [Capability] {
        match self {
            Self::OpenAIChatV1 => &CAPS_OPENAI_CHAT_V1,
            Self::OpenAIResponsesV1 => &CAPS_OPENAI_RESPONSES_V1,
            Self::AnthropicMessagesV1 => &CAPS_ANTHROPIC_MESSAGES_V1,
            Self::AnthropicMessagesV2 => &CAPS_ANTHROPIC_MESSAGES_V2,
            Self::GeminiGenerateV1 => &CAPS_GEMINI_GENERATE_V1,
            Self::RpcV1 => &CAPS_RPC_V1,
            Self::OpenAIImageGenV1 => &CAPS_OPENAI_IMAGE_GEN_V1,
            Self::StabilityImageGenV3_5 => &CAPS_STABILITY_IMAGE_GEN_V3_5,
            Self::JimengAsyncV1 => &CAPS_JIMENG_ASYNC_V1,
            Self::GoogleImagenV1 => &CAPS_GOOGLE_IMAGEN_V1,
            Self::Seedance2AsyncV1 => &CAPS_SEEDANCE2_ASYNC_V1,
            Self::KlingAsyncV3_0 => &CAPS_KLING_ASYNC_V3_0,
            Self::Sora2AsyncV1 => &CAPS_SORA2_ASYNC_V1,
            Self::GoogleVeo3_1AsyncV1 => &CAPS_GOOGLE_VEO_V3_1_ASYNC_V1,
            Self::ElevenLabsTTSV1 => &CAPS_ELEVEN_LABS_TTS_V1,
            Self::ElevenLabsMusicV1 => &CAPS_ELEVEN_LABS_MUSIC_V1,
            Self::Hyper3DAsyncV1 => &CAPS_HYPER3D_ASYNC_V1,
            Self::Hunyuan3DAsyncV1 => &CAPS_HUNYUAN_3D_ASYNC_V1,
        }
    }

    fn is_async(&self) -> bool {
        GenProtocol::is_async(*self)
    }

    fn name(&self) -> &'static str {
        GenProtocol::as_str(*self)
    }
}

impl fmt::Display for GenProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for GenProtocol {
    type Err = UnknownProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for proto in Self::ALL {
            if proto.as_str() == s {
                return Ok(*proto);
            }
        }
        Err(UnknownProtocolError(s.to_string()))
    }
}

impl AsRef<str> for GenProtocol {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// Custom serde for GenProtocol — serializes as the flat as_str() form
// (e.g. "stability_image_gen_v3.5", not "StabilityImageGenV3_5")
impl Serialize for GenProtocol {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GenProtocol {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown protocol: {0}")]
pub struct UnknownProtocolError(pub String);

// ─── Static capability slices ──────────────────────────────────────
//
// Compile-time capability manifests for each protocol. These are the single
// source of truth for what a protocol can read and generate.

static CAPS_OPENAI_CHAT_V1: [Capability; 5] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
];

static CAPS_OPENAI_RESPONSES_V1: [Capability; 6] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
    Capability::DeepReasoning,
];

static CAPS_ANTHROPIC_MESSAGES_V1: [Capability; 6] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
    Capability::DeepReasoning,
];

static CAPS_ANTHROPIC_MESSAGES_V2: [Capability; 6] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
    Capability::DeepReasoning,
];

static CAPS_GEMINI_GENERATE_V1: [Capability; 8] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::ReadAudio(AudioKind::General),
    Capability::ReadVideo,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
    Capability::DeepReasoning,
];

static CAPS_RPC_V1: [Capability; 7] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::ReadAudio(AudioKind::General),
    Capability::ReadVideo,
    Capability::GenerateText,
    Capability::ToolCalling,
    Capability::Streaming,
];

static CAPS_OPENAI_IMAGE_GEN_V1: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateImage,
];

static CAPS_STABILITY_IMAGE_GEN_V3_5: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateImage,
];

static CAPS_JIMENG_ASYNC_V1: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateImage,
];

static CAPS_GOOGLE_IMAGEN_V1: [Capability; 2] = [Capability::ReadText, Capability::GenerateImage];

static CAPS_SEEDANCE2_ASYNC_V1: [Capability; 5] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::ReadAudio(AudioKind::General),
    Capability::GenerateVideo,
    Capability::GenerateAudio(AudioKind::General),
];

static CAPS_KLING_ASYNC_V3_0: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateVideo,
];

static CAPS_SORA2_ASYNC_V1: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateVideo,
];

static CAPS_GOOGLE_VEO_V3_1_ASYNC_V1: [Capability; 4] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::GenerateVideo,
    Capability::GenerateAudio(AudioKind::General),
];

static CAPS_ELEVEN_LABS_TTS_V1: [Capability; 3] = [
    Capability::ReadText,
    Capability::GenerateAudio(AudioKind::Speech),
    Capability::Streaming,
];

static CAPS_ELEVEN_LABS_MUSIC_V1: [Capability; 2] = [
    Capability::ReadText,
    Capability::GenerateAudio(AudioKind::Music),
];

static CAPS_HYPER3D_ASYNC_V1: [Capability; 3] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::Generate3D(ThreeDKind::General),
];

static CAPS_HUNYUAN_3D_ASYNC_V1: [Capability; 5] = [
    Capability::ReadText,
    Capability::ReadImage,
    Capability::Generate3D(ThreeDKind::General),
    Capability::Generate3D(ThreeDKind::GaussianSplat),
    Capability::Generate3D(ThreeDKind::TopologyMap),
];

#[cfg(test)]
mod tests {
    use super::*;

    // ── Capability hierarchy tests ─────────────────────────────────

    #[test]
    fn capability_read_write_classification() {
        assert!(Capability::ReadText.is_read());
        assert!(!Capability::ReadText.is_generate());
        assert!(Capability::GenerateText.is_generate());
        assert!(!Capability::GenerateText.is_read());
        assert!(Capability::DeepReasoning.is_special());
    }

    #[test]
    fn audio_kind_hierarchy() {
        assert_eq!(
            Capability::GenerateAudio(AudioKind::Speech).as_str(),
            "generate_audio_speech"
        );
        assert_eq!(
            Capability::GenerateAudio(AudioKind::Music).as_str(),
            "generate_audio_music"
        );
        assert_eq!(
            Capability::GenerateAudio(AudioKind::General).as_str(),
            "generate_audio"
        );
    }

    #[test]
    fn three_d_kind_hierarchy() {
        assert_eq!(
            Capability::Generate3D(ThreeDKind::GaussianSplat).as_str(),
            "generate_3d_gaussian_splat"
        );
        assert_eq!(
            Capability::Generate3D(ThreeDKind::TopologyMap).as_str(),
            "generate_3d_topology_map"
        );
    }

    #[test]
    fn capability_roundtrip() {
        for proto in GenProtocol::ALL {
            for cap in proto.capabilities() {
                let s = cap.as_str();
                let back = Capability::from_str(s)
                    .unwrap_or_else(|_| panic!("capability roundtrip failed for: {s}"));
                assert_eq!(cap, &back, "roundtrip mismatch for {s}");
            }
        }
    }

    // ── Protocol naming tests ──────────────────────────────────────

    #[test]
    fn version_decimal_in_output() {
        // Underscore in code → dot in text
        assert!(GenProtocol::StabilityImageGenV3_5.as_str().contains("v3.5"));
        assert!(!GenProtocol::StabilityImageGenV3_5.as_str().contains("v3_5"));
        assert!(GenProtocol::KlingAsyncV3_0.as_str().contains("v3.0"));
        assert!(GenProtocol::GoogleVeo3_1AsyncV1.as_str().contains("v3.1"));
    }

    #[test]
    fn protocol_from_str_roundtrip() {
        for proto in GenProtocol::ALL {
            let s = proto.as_str();
            let back =
                GenProtocol::from_str(s).unwrap_or_else(|e| panic!("from_str failed for {s}: {e}"));
            assert_eq!(*proto, back, "roundtrip mismatch for {s}");
        }
    }

    #[test]
    fn all_protocols_have_capabilities() {
        for proto in GenProtocol::ALL {
            let caps = proto.capabilities();
            assert!(!caps.is_empty(), "{:?} has zero capabilities", proto);
        }
    }

    #[test]
    fn async_protocols_identified() {
        let async_protocols = GenProtocol::ALL
            .iter()
            .filter(|p| p.is_async())
            .collect::<Vec<_>>();
        assert!(
            async_protocols.len() >= 6,
            "expected at least 6 async protocols, got {}",
            async_protocols.len()
        );
        // Sync protocols
        assert!(!GenProtocol::OpenAIChatV1.is_async());
        assert!(!GenProtocol::OpenAIImageGenV1.is_async());
    }

    #[test]
    fn llm_protocols_generate_text() {
        let llm_protocols = [
            GenProtocol::OpenAIChatV1,
            GenProtocol::OpenAIResponsesV1,
            GenProtocol::AnthropicMessagesV1,
            GenProtocol::AnthropicMessagesV2,
            GenProtocol::GeminiGenerateV1,
        ];
        for proto in llm_protocols {
            assert!(
                proto.capabilities().contains(&Capability::GenerateText),
                "{:?} should have GenerateText",
                proto
            );
        }
    }

    #[test]
    fn display_name_is_human_friendly() {
        assert_eq!(
            GenProtocol::StabilityImageGenV3_5.display_name(),
            "Stability Image Gen v3.5"
        );
        assert_eq!(
            GenProtocol::GoogleVeo3_1AsyncV1.display_name(),
            "Google Veo v3.1 Async v1"
        );
    }

    #[test]
    fn embedding_is_special_not_media() {
        assert!(Capability::Embedding.is_special());
        assert!(!Capability::Embedding.is_read());
        assert!(!Capability::Embedding.is_generate());
    }

    #[test]
    fn is_llm_chat_identifies_text_protocols() {
        assert!(GenProtocol::OpenAIChatV1.is_llm_chat());
        assert!(GenProtocol::AnthropicMessagesV1.is_llm_chat());
        assert!(GenProtocol::GeminiGenerateV1.is_llm_chat());
        assert!(!GenProtocol::OpenAIImageGenV1.is_llm_chat());
        assert!(!GenProtocol::Seedance2AsyncV1.is_llm_chat());
    }

    #[test]
    fn auth_header_name_llm_vs_non_llm() {
        assert_eq!(
            GenProtocol::AnthropicMessagesV1.auth_header_name(),
            "x-api-key"
        );
        assert_eq!(
            GenProtocol::OpenAIChatV1.auth_header_name(),
            "Authorization"
        );
    }

    #[test]
    fn auth_header_value_formats() {
        assert_eq!(
            GenProtocol::OpenAIChatV1.auth_header_value("sk-test"),
            "Bearer sk-test"
        );
        assert_eq!(
            GenProtocol::AnthropicMessagesV1.auth_header_value("ant-key"),
            "ant-key"
        );
    }

    #[test]
    fn validation_url_for_llm_protocols() {
        assert_eq!(
            GenProtocol::OpenAIChatV1.validation_url("https://api.openai.com"),
            "https://api.openai.com/models"
        );
        assert_eq!(
            GenProtocol::AnthropicMessagesV1.validation_url("https://api.anthropic.com"),
            "https://api.anthropic.com/v1/models"
        );
        // Gemini has complex trim logic — must be tested
        assert_eq!(
            GenProtocol::GeminiGenerateV1
                .validation_url("https://generativelanguage.googleapis.com"),
            "https://generativelanguage.googleapis.com/v1beta/models"
        );
        // Gemini with already-suffixed URL
        assert_eq!(
            GenProtocol::GeminiGenerateV1
                .validation_url("https://generativelanguage.googleapis.com/v1beta/models"),
            "https://generativelanguage.googleapis.com/v1beta/models"
        );
    }

    #[test]
    fn uses_query_param_auth_gemini_only() {
        assert!(GenProtocol::GeminiGenerateV1.uses_query_param_auth());
        assert!(!GenProtocol::OpenAIChatV1.uses_query_param_auth());
        assert!(!GenProtocol::AnthropicMessagesV1.uses_query_param_auth());
        assert!(!GenProtocol::OpenAIImageGenV1.uses_query_param_auth());
    }

    #[test]
    fn resolve_from_provider_id() {
        assert_eq!(
            GenProtocol::resolve("anthropic"),
            GenProtocol::AnthropicMessagesV2
        );
        assert_eq!(
            GenProtocol::resolve("google"),
            GenProtocol::GeminiGenerateV1
        );
        assert_eq!(GenProtocol::resolve("deepseek"), GenProtocol::OpenAIChatV1);
    }

    #[test]
    fn default_is_openai_chat_v1() {
        assert_eq!(GenProtocol::default(), GenProtocol::OpenAIChatV1);
    }

    #[test]
    fn serde_gen_protocol_uses_as_str_form() {
        // Critical: serde output must match as_str() (snake_case + dots)
        for proto in GenProtocol::ALL {
            let json = serde_json::to_string(proto).unwrap();
            let expected = format!("\"{}\"", proto.as_str());
            assert_eq!(
                json, expected,
                "serde output {:?} != as_str() {:?} for {:?}",
                json, expected, proto
            );
            let back: GenProtocol = serde_json::from_str(&json).unwrap();
            assert_eq!(proto, &back, "serde roundtrip failed for {:?}", proto);
        }
    }

    #[test]
    fn serde_gen_protocol_dotted_version_roundtrip() {
        // Specifically test protocols with dotted version numbers
        let protocols = [
            GenProtocol::StabilityImageGenV3_5,
            GenProtocol::KlingAsyncV3_0,
            GenProtocol::GoogleVeo3_1AsyncV1,
        ];
        for proto in protocols {
            let json = serde_json::to_string(&proto).unwrap();
            let back: GenProtocol = serde_json::from_str(&json).unwrap();
            assert_eq!(proto, back, "serde roundtrip failed for {:?}", proto);
        }
    }

    #[test]
    fn serde_capability_uses_as_str_form() {
        let caps = [
            Capability::ReadText,
            Capability::GenerateImage,
            Capability::ReadAudio(AudioKind::Speech),
            Capability::Generate3D(ThreeDKind::GaussianSplat),
            Capability::Embedding,
        ];
        for cap in caps {
            let json = serde_json::to_string(&cap).unwrap();
            let expected = format!("\"{}\"", cap.as_str());
            assert_eq!(json, expected, "serde mismatch for {:?}", cap);
            let back: Capability = serde_json::from_str(&json).unwrap();
            assert_eq!(cap, back, "roundtrip failed for {:?}", cap);
        }
    }

    #[test]
    fn serde_audio_kind_matches_as_str() {
        assert_eq!(
            serde_json::to_string(&AudioKind::General).unwrap(),
            "\"audio\""
        );
        assert_eq!(
            serde_json::to_string(&AudioKind::Speech).unwrap(),
            "\"speech\""
        );
    }

    #[test]
    fn serde_three_d_kind_matches_as_str() {
        assert_eq!(
            serde_json::to_string(&ThreeDKind::General).unwrap(),
            "\"3d\""
        );
        assert_eq!(
            serde_json::to_string(&ThreeDKind::GaussianSplat).unwrap(),
            "\"3d_gaussian_splat\""
        );
    }

    #[test]
    fn gemini_has_tool_calling() {
        assert!(
            GenProtocol::GeminiGenerateV1
                .capabilities()
                .contains(&Capability::ToolCalling),
            "Gemini must advertise ToolCalling"
        );
    }
}
