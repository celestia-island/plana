//! Unified model management — shared types for the entelecheia + shittim-chest
//! model lifecycle.
//!
//! ## Architecture
//!
//! **evernight owns all model deployment.** Neither the upstream engine
//! (scepter) nor the web UI (shittim-chest) loads, starts, or stops models
//! directly. They send requests; evernight handles the lifecycle:
//!
//! ```text
//!  WebUI (chest)   ──WS──▶  Scepter  ──▶  evernight
//!  Scepter (RAG)   ──────────────────▶  evernight
//!                                        │
//!                   ┌────────────────────┴────────────────────┐
//!                   │ GPU-first: detect GPU nodes → deploy     │
//!                   │   vLLM / faster GPU backends             │
//!                   │ GPU unavailable? → degrade to CPU:       │
//!                   │   ollama / whisper.cpp / onnxruntime     │
//!                   └─────────────────────────────────────────┘
//! ```
//!
//! **GPU-first, CPU-fallback.** evernight always attempts GPU first. Only
//! when no GPU is detected (or no GPU node is reachable) does it fall back
//! to CPU-only small models. CPU is a degraded mode, never the default.
//!
//! Arona provides the shared vocabulary so both sides describe models in the
//! same terms.
//!
//! ## Model categories
//!
//! | Category | Example | Primary consumer |
//! |---|---|---|
//! | LLM | gpt-5.5, claude-opus-4.8 | scepter (agent skill execution) |
//! | Embedding | bge-m3, nomic-embed-text | scepter (RAG / vector store) |
//! | Speech → Text | whisper tiny/base/small | chest (voice input → text) |
//! | Text → Speech | tts-1, elevenlabs | scepter (generation) |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Model category — what KIND of model is this?
// ═══════════════════════════════════════════════════════════════

/// Top-level model category. Determines which subsystem consumes the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelCategory {
    /// Large language model — chat, reasoning, tool use.
    Llm,
    /// Text embedding model — vector representation for RAG / similarity.
    Embedding,
    /// Speech-to-text — transcribe audio input to text.
    SpeechToText,
    /// Text-to-speech — synthesise audio from text.
    TextToSpeech,
    /// Image generation — DALL-E, Stable Diffusion, ComfyUI pipelines.
    /// Consumed by the MediaFlow node graph (image_to_image, text_to_image nodes).
    ImageGeneration,
    /// 3D model generation — TRELLIS, Meshy, Rodin. Produces GLB / mesh output.
    /// Consumed by the MediaFlow node graph (image_to_3d, text_to_3d nodes).
    ThreeDGeneration,
    /// Multimodal — models that accept text + image input in the same message
    /// (glm-4.6v, glm-5v-turbo, GPT-4V). Used for vision critique nodes.
    MultiModal,
}

/// Fine-grained model capability flags.
///
/// Whereas [`ModelCategory`] answers "what kind of model is this?",
/// `ModelCapability` answers "what specific things can this model do?".
/// Skills declare their `required_capabilities`; the scepter router filters
/// available models by capability intersection.
///
/// This replaces the scattered boolean flags (`supports_vision`,
/// `supports_function_calling` …) with a single extensible enum. The booleans
/// remain for backward compatibility but are superseded by this enum when
/// present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    // ── Text / LLM ──
    /// Standard chat / completion.
    TextChat,
    /// SSE streaming output.
    TextStreaming,
    /// Tool / function calling.
    FunctionCalling,
    /// Chain-of-thought / deep reasoning.
    Reasoning,
    /// Code generation (JavaScript, Python, OpenSCAD, CSG …).
    CodeGeneration,

    // ── Embedding ──
    /// Text → vector embedding.
    TextEmbedding,

    // ── Audio ──
    /// Audio → text (speech-to-text).
    SpeechToText,
    /// Text → audio (text-to-speech).
    TextToSpeech,

    // ── Vision / multimodal input ──
    /// Accept image in chat messages (multimodal input).
    ImageInput,
    /// Accept video in chat messages.
    VideoInput,
    /// Analyse an image and produce a structured critique + improvement plan.
    /// This is the key capability for the MediaFlow vision_critique node.
    VisualCritique,

    // ── Image generation ──
    /// Text prompt → image.
    TextToImage,
    /// Image + text → modified image.
    ImageToImage,
    /// Inpainting / outpainting / selective editing.
    ImageEdit,
    /// Increase resolution with detail preservation.
    ImageUpscale,

    // ── 3D generation ──
    /// Text prompt → 3D mesh (GLB).
    #[serde(rename = "text_to_3d")]
    TextTo3D,
    /// Single / multi image → 3D mesh.
    #[serde(rename = "image_to_3d")]
    ImageTo3D,
    /// Modify an existing 3D model programmatically (CSG, parametric).
    #[serde(rename = "three_d_edit")]
    ThreeDEdit,
    /// Export to GLB / FBX / OBJ.
    #[serde(rename = "three_d_export")]
    ThreeDExport,
    /// Generate PBR texture sets (albedo / normal / roughness / metalness).
    #[serde(rename = "pbr_texturing")]
    PBRTexturing,
    /// Decimation, vertex merging, remeshing.
    #[serde(rename = "mesh_optimization")]
    MeshOptimization,
}

/// Generation quality tier — applies to image / 3D generation models.
///
/// Distinct from [`super::ModelTier`] (which ranks LLM reasoning depth),
/// `GenerationTier` ranks output fidelity vs. speed for generative models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum GenerationTier {
    /// Low resolution, fast (≤30s). For iteration previews inside MediaFlow loops.
    FastPreview,
    /// Medium resolution (1–2 min). Default for most generation nodes.
    Standard,
    /// High resolution (5 min+). Final export quality.
    Production,
}

/// Minimum hardware requirements for a local generative model.
///
/// Populated for GPU-deployed models (TRELLIS, SDXL …). Remote-API models
/// leave this as `None`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
pub struct HardwareRequirements {
    /// Minimum VRAM in GB.
    #[serde(default)]
    #[ts(optional)]
    pub min_vram_gb: Option<u32>,
    /// Minimum system RAM in GB.
    #[serde(default)]
    #[ts(optional)]
    pub min_ram_gb: Option<u32>,
    /// Recommended GPU (e.g. `"NVIDIA RTX 4090"`).
    #[serde(default)]
    #[ts(optional)]
    pub recommended_gpu: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Execution backend — WHERE does the model run?
// ═══════════════════════════════════════════════════════════════

/// Where a model physically executes. evernight chooses the backend based on
/// GPU availability — GPU-first, CPU-fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelBackend {
    /// Remote API (OpenAI, Anthropic, ZhiPu …). No local resources.
    RemoteApi,
    /// GPU node — either local (PCI passthrough) or remote (forwarded by
    /// evernight to a GPU-equipped host). This is the **preferred** backend;
    /// evernight always tries this first.
    Gpu,
    /// CPU fallback — used only when no GPU is detected or reachable.
    /// Degraded mode: slower, smaller models.
    Cpu,
}

// ═══════════════════════════════════════════════════════════════
// Model-management catalog — ModelType × DeploymentMode
// ═══════════════════════════════════════════════════════════════

/// Coarse catalog-level type of a model-management directory entry
/// (shittim-chest PR #1430's P1 wave, adjudicated 2026-10-06).
///
/// This is the **catalog axis** — "how does the directory describe this
/// entry to a human picking something to add?" — and is deliberately
/// coarser than [`ModelCategory`], the consumer axis that routes a model
/// to the subsystem that runs it (`speech_to_text` vs `text_to_speech`
/// split there; one `realtime_voice` here, because a voice-engine
/// registration carries STT · TTS · duplex as one installable unit).
/// `embedding` exists on both axes under the same name. Map between the
/// two at the consumer site; do not merge them.
///
/// Grounded in the values chest stores today, not invented:
///
/// | Variant | Existing anchor |
/// |---|---|
/// | `llm` | every stock `llm_providers.category` row (`"chat"` migrates to `llm`) |
/// | `realtime_voice` | `speech_engines.kind` = `whisper_docker` / `cloud_endpoint` / `cep_engine` |
/// | `vision_caption` / `action_recognition` | the P1 proposal's edge-input set (fast perception feeding the world-model layer) |
/// | `embedding` | a `llm_providers.category` value the seed-JSON channel can carry (chest's seed test sample) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    /// Large language model — chat, reasoning, tool use (cloud provider or
    /// local engine).
    Llm,
    /// Realtime voice engine — speech-to-text · text-to-speech · duplex
    /// sessions as one installable unit (whisper / cloud STT / CEP).
    RealtimeVoice,
    /// Fast image captioning — edge vision input for the world model.
    VisionCaption,
    /// Action / gesture recognition — edge video input for the world model.
    ActionRecognition,
    /// Text embedding model — vectors for RAG / similarity.
    Embedding,
}

impl ModelType {
    /// Canonical wire string (matches the serde representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::RealtimeVoice => "realtime_voice",
            Self::VisionCaption => "vision_caption",
            Self::ActionRecognition => "action_recognition",
            Self::Embedding => "embedding",
        }
    }

    /// Parse a legacy free-text `llm_providers.category` value (chest stores
    /// the column as an unchecked `String`).
    ///
    /// `"chat"` — the column default every stock row carries (the env
    /// override's documented domain is `chat`/`image`) — maps to
    /// [`ModelType::Llm`] (PR #1430: the ever-`"chat"` stock rows migrate
    /// smoothly to `llm`); `"embedding"` — a value chest's provider-seed
    /// JSON can carry (its seed test uses exactly this sample) — maps to
    /// [`ModelType::Embedding`]; the canonical [`ModelType::as_str`] values
    /// pass through unchanged. This is a **whitelist, not an exhaustive
    /// domain**: the column accepts any string (seed JSON can write
    /// arbitrary values, and the env-documented domain also names `image`),
    /// so anything unmapped returns `None` for the caller to surface —
    /// never a silent fallback.
    pub fn from_legacy_category(raw: &str) -> Option<Self> {
        match raw {
            "chat" => Some(Self::Llm),
            s => Self::all().into_iter().find(|t| t.as_str() == s),
        }
    }

    /// Every variant, in declaration order (drives round-trip tests).
    pub fn all() -> [Self; 5] {
        [
            Self::Llm,
            Self::RealtimeVoice,
            Self::VisionCaption,
            Self::ActionRecognition,
            Self::Embedding,
        ]
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<ModelType> for String {
    fn from(v: ModelType) -> String {
        v.as_str().to_string()
    }
}

/// How a model-management catalog entry is deployed and managed
/// (shittim-chest PR #1430's P1 wave, adjudicated 2026-10-06).
///
/// The **management axis** of the P2 "add model" flow: `cloud` entries are
/// provider registrations (an endpoint + credentials chest stores and
/// probes, today's `llm_providers` rows); `local` entries are engine-side
/// installations (P3: a `speech_engines` row publishes into the catalog as
/// `mode = local`, `engine = whisper/cep/…`). Distinct from
/// [`ModelBackend`], the evernight execution axis (remote API vs GPU vs
/// CPU) — a `local` whisper container may still run on either backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
    /// Cloud provider registration — external endpoint + key, nothing
    /// deployed locally.
    Cloud,
    /// Local engine installation — deployed/registered on the engine side
    /// (chest-hosted container or a locally-registered external engine).
    Local,
}

impl DeploymentMode {
    /// Canonical wire string (matches the serde representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cloud => "cloud",
            Self::Local => "local",
        }
    }

    /// Every variant, in declaration order (drives round-trip tests).
    pub fn all() -> [Self; 2] {
        [Self::Cloud, Self::Local]
    }
}

impl std::fmt::Display for DeploymentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<DeploymentMode> for String {
    fn from(v: DeploymentMode) -> String {
        v.as_str().to_string()
    }
}

// ═══════════════════════════════════════════════════════════════
// Model descriptor — unified description of any model
// ═══════════════════════════════════════════════════════════════

/// A unified description of an AI model, shared between scepter and chest.
///
/// Both sides can enumerate available models, check their status, and request
/// inference using this common vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
pub struct ModelDescriptor {
    /// Unique id, e.g. `"bge-m3"`, `"whisper-tiny"`, `"claude-opus-4.8"`.
    pub id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// What kind of model this is.
    pub category: ModelCategory,
    /// Where it runs.
    pub backend: ModelBackend,
    /// Size tier (LLM coding-plan concept; `None` for non-LLM models).
    #[serde(default)]
    #[ts(optional)]
    pub tier: Option<ModelTier>,
    /// Output dimension (embedding models only).
    #[serde(default)]
    #[ts(optional)]
    pub dimension: Option<u32>,
    /// Approximate model size in bytes (local models).
    #[serde(default)]
    #[ts(optional)]
    pub size_bytes: Option<u64>,
    /// Provider index (`#N` convention; LLM models only).
    #[serde(default)]
    #[ts(optional)]
    pub provider_index: Option<u8>,
    /// Fine-grained capability flags. When non-empty, the scepter router uses
    /// these to match skills that declare `required_capabilities`. When empty,
    /// falls back to the legacy boolean flags on `ModelFsInfo`.
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
    /// Generation quality tier (image / 3D generation models only).
    #[serde(default)]
    #[ts(optional)]
    pub generation_tier: Option<GenerationTier>,
    /// Hardware requirements (local generative models only).
    #[serde(default)]
    #[ts(optional)]
    pub hardware_requirements: Option<HardwareRequirements>,
}

/// Re-export so consumers don't need a separate import.
pub use super::ModelTier;

// ═══════════════════════════════════════════════════════════════
// Model server lifecycle — managed local model process
// ═══════════════════════════════════════════════════════════════

/// Status of a local model server (ollama, whisper.cpp, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelServerStatus {
    /// Server is running and accepting requests.
    Running,
    /// Server is starting up (model loading).
    Starting,
    /// Server is stopped.
    Stopped,
    /// Server failed to start.
    Failed,
}

/// A managed model server instance, deployed and owned by **evernight**.
/// Neither scepter nor chest starts/stops these directly — they issue
/// `RequestModelServerAction` and evernight performs the lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
pub struct ModelServerInfo {
    /// Server kind (determines the Docker image / launch command).
    pub kind: ModelServerKind,
    /// HTTP endpoint (e.g. `http://localhost:8178`).
    pub endpoint: String,
    /// Current lifecycle status.
    pub status: ModelServerStatus,
    /// Which backend this server is running on (GPU or CPU).
    pub backend: ModelBackend,
    /// Docker container id (managed by evernight's container runtime).
    #[serde(default)]
    #[ts(optional)]
    pub container_id: Option<String>,
    /// Which models are loaded in this server.
    #[serde(default)]
    pub loaded_models: Vec<String>,
}

/// The type of model server. evernight deploys and manages these; the choice
/// of GPU vs CPU variant is made by evernight at deploy time (GPU-first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelServerKind {
    /// Ollama — LLM + embedding models (GPU via candle/CUDA; CPU via GGUF fallback).
    Ollama,
    /// whisper.cpp — speech-to-text (GPU build when available; CPU otherwise).
    WhisperCpp,
    /// vLLM — high-throughput LLM serving. GPU-only (no CPU build).
    Vllm,
}

// ═══════════════════════════════════════════════════════════════
// Model request — how the web UI asks for inference
// ═══════════════════════════════════════════════════════════════

/// A model inference request, sent from the web UI to the upstream engine
/// over the WS JSON-RPC channel. The engine routes it to the appropriate
/// backend (local CPU / local GPU / remote GPU / remote API).
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
pub struct ModelInferenceRequest {
    /// Which model to use (by id).
    pub model_id: String,
    /// Input data (format depends on category: text for LLM/embedding,
    /// base64 audio for STT, base64 image for vision).
    pub input: String,
    /// Optional parameters (temperature, max_tokens, language hint …).
    #[serde(default)]
    #[ts(optional, type = "Record<string, unknown>")]
    pub parameters: Option<serde_json::Value>,
}

/// Inference result returned to the web UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
pub struct ModelInferenceResult {
    /// Model that produced the output.
    pub model_id: String,
    /// Output data (text for LLM/embedding/TTS, text for STT, JSON for vision).
    pub output: String,
    /// Time spent (milliseconds).
    #[serde(default)]
    #[ts(optional)]
    pub elapsed_ms: Option<u64>,
    /// Token/processing usage (if applicable).
    #[serde(default)]
    #[ts(optional, type = "Record<string, unknown>")]
    pub usage: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════
// WS protocol — SyncMessage variants for model management
// ═══════════════════════════════════════════════════════════════

/// `Sync.RequestModelList` — enumerate available models.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct RequestModelListParams {
    /// Filter by category (omit for all).
    #[serde(default)]
    #[ts(optional)]
    pub category: Option<ModelCategory>,
}

/// `Sync.ModelList` — model catalogue response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct ModelListParams {
    pub models: Vec<ModelDescriptor>,
    pub servers: Vec<ModelServerInfo>,
}

/// `Sync.RequestModelInference` — ask the engine to run a model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct RequestModelInferenceParams {
    pub request: ModelInferenceRequest,
}

/// `Sync.ModelInferenceResult` — inference result push.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct ModelInferenceResultParams {
    pub result: ModelInferenceResult,
}

/// `Sync.RequestModelServerAction` — ask evernight (via scepter) to start /
/// stop / restart a model server. Neither chest nor scepter performs the
/// deployment directly; the action is forwarded to evernight's model lifecycle
/// manager.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct RequestModelServerActionParams {
    pub kind: ModelServerKind,
    pub action: ModelServerAction,
    /// Preferred backend. evernight will honour this if possible; falls back
    /// to CPU if the requested GPU is unavailable.
    #[serde(default)]
    #[ts(optional)]
    pub preferred_backend: Option<ModelBackend>,
}

/// `Sync.ModelServerActionResult` — action result.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct ModelServerActionResultParams {
    pub kind: ModelServerKind,
    pub status: ModelServerStatus,
}

/// What to do with a model server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "model.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelServerAction {
    Start,
    Stop,
    Restart,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ModelType / DeploymentMode wire contract ───────────────────
    //
    // The exact snake_case strings are the cross-repo contract (the TS
    // union members chest/arona will switch on, and the values a consumer
    // writes back into chest's free-text `llm_providers.category` column).
    // They are pinned literally — a renamed variant must be a deliberate
    // wire break, not a silent reshuffle.

    #[test]
    fn model_type_wire_strings_are_pinned() {
        let expected = [
            "llm",
            "realtime_voice",
            "vision_caption",
            "action_recognition",
            "embedding",
        ];
        for (variant, want) in ModelType::all().into_iter().zip(expected) {
            assert_eq!(variant.to_string(), want);
            assert_eq!(variant.as_str(), want);
            assert_eq!(String::from(variant), want);
            // serde and as_str must agree — unlike the str_enum! vocabulary
            // in `enums.rs` (documented PascalCase divergence there), these
            // catalog types are new with no legacy wire to honor.
            assert_eq!(
                serde_json::to_string(&variant).unwrap(),
                format!("{want:?}")
            );
        }
    }

    #[test]
    fn deployment_mode_wire_strings_are_pinned() {
        for (variant, want) in DeploymentMode::all().into_iter().zip(["cloud", "local"]) {
            assert_eq!(variant.to_string(), want);
            assert_eq!(variant.as_str(), want);
            assert_eq!(String::from(variant), want);
            assert_eq!(
                serde_json::to_string(&variant).unwrap(),
                format!("{want:?}")
            );
        }
    }

    #[test]
    fn catalog_enums_round_trip_through_serde() {
        for variant in ModelType::all() {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ModelType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
        for variant in DeploymentMode::all() {
            let json = serde_json::to_string(&variant).unwrap();
            let back: DeploymentMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn unknown_wire_values_are_rejected_not_guessed() {
        // No silent fallback: a value outside the pinned set must error,
        // so a typo surfaces at the boundary instead of masquerading as a
        // default (configuration-concentration rule ③).
        for raw in ["chat", "generation_image", "Chat", "LLM", ""] {
            assert!(
                serde_json::from_str::<ModelType>(&format!("{raw:?}")).is_err(),
                "raw {raw:?} must not deserialize as a ModelType"
            );
            assert!(
                serde_json::from_str::<DeploymentMode>(&format!("{raw:?}")).is_err(),
                "raw {raw:?} must not deserialize as a DeploymentMode"
            );
        }
    }

    #[test]
    fn legacy_category_maps_chat_and_passthrough() {
        // Legacy anchors, not an exhaustive domain (the column is an
        // unchecked String): the default "chat" every stock row carries,
        // and "embedding" — the seed-JSON test sample. Anything unmapped
        // must surface as None below.
        assert_eq!(
            ModelType::from_legacy_category("chat"),
            Some(ModelType::Llm)
        );
        assert_eq!(
            ModelType::from_legacy_category("embedding"),
            Some(ModelType::Embedding)
        );
        // Canonical names pass through — one entry point for the column.
        for variant in ModelType::all() {
            assert_eq!(
                ModelType::from_legacy_category(variant.as_str()),
                Some(variant)
            );
        }
        // Anything else is unknown, not guessed. "image" is deliberate:
        // chest's env-documented domain names it, and P1 has no variant
        // for it yet — the caller must surface it, not get a guess.
        for raw in ["gpt", "Chat", "stt", "generation_image", "image", ""] {
            assert_eq!(ModelType::from_legacy_category(raw), None, "raw {raw:?}");
        }
    }

    #[test]
    fn catalog_surface_resolves_at_expected_paths() {
        // Compile-time pin of the public paths consumers will import from —
        // the crate-root named list in lib.rs (same footgun the TS flat
        // surface hit: a type missing from the list stays deep-import-only).
        fn _path_pins() {
            let _ = crate::ModelType::Llm;
            let _ = crate::model::ModelType::Llm;
            let _ = crate::DeploymentMode::Cloud;
            let _ = crate::model::DeploymentMode::Cloud;
        }
    }
}
