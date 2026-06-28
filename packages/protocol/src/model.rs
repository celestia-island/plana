//! Unified model management — shared types for the entelecheia + shittim-chest
//! model lifecycle.
//!
//! ## Architecture
//!
//! All AI models (LLM / embedding / speech / vision) are managed by the
//! upstream engine (scepter) via evernight. The web UI (shittim-chest) never
//! loads a model directly — it requests inference through the WS JSON-RPC
//! channel and the upstream engine routes to the appropriate backend:
//!
//! ```text
//!  WebUI (chest)  ──WS──▶  Scepter  ──▶  evernight
//!                                   ├─ CPU models: local Docker (ollama, whisper.cpp…)
//!                                   └─ GPU models: forward to a GPU-equipped remote host
//! ```
//!
//! Arona provides the shared vocabulary so both sides describe models in the
//! same terms. Each side manages its own instances ("两边各管各的"), but the
//! **types** are unified here.
//!
//! ## Model categories
//!
//! | Category | Example | Primary consumer |
//! |---|---|---|
//! | LLM | gpt-5.5, claude-opus-4.8 | scepter (agent skill execution) |
//! | Embedding | bge-m3, nomic-embed-text | scepter (RAG / vector store) |
//! | Speech → Text | whisper tiny/base/small | chest (voice input → text) |
//! | Text → Speech | tts-1, elevenlabs | scepter (generation) |
//! | Vision | mediapipe pose/gesture | chest (holographic AR mode) — **stub** |
//!
//! CPU-first: all model types prefer CPU-only small models that fit in a
//! Docker container. GPU backends are optional and routed via evernight to
//! remote hosts with PCI-passthrough GPUs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Model category — what KIND of model is this?
// ═══════════════════════════════════════════════════════════════

/// Top-level model category. Determines which subsystem consumes the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
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
    /// Vision / pose / gesture recognition (MediaPipe etc.).
    /// Stub — implementation deferred until AR/holographic hardware is available.
    Vision,
}

// ═══════════════════════════════════════════════════════════════
// Execution backend — WHERE does the model run?
// ═══════════════════════════════════════════════════════════════

/// Where a model physically executes. Determines deployment, latency, and
/// whether a GPU is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelBackend {
    /// Remote API (OpenAI, Anthropic, ZhiPu …). No local resources.
    RemoteApi,
    /// Local CPU model in a Docker container (ollama, whisper.cpp …).
    LocalCpu,
    /// Local GPU model — requires PCI passthrough or device mount.
    LocalGpu,
    /// Forwarded to a GPU-equipped remote host via evernight.
    RemoteGpu,
}

// ═══════════════════════════════════════════════════════════════
// Model descriptor — unified description of any model
// ═══════════════════════════════════════════════════════════════

/// A unified description of an AI model, shared between scepter and chest.
///
/// Both sides can enumerate available models, check their status, and request
/// inference using this common vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
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
}

/// Re-export so consumers don't need a separate import.
pub use super::ModelTier;

// ═══════════════════════════════════════════════════════════════
// Model server lifecycle — managed local model process
// ═══════════════════════════════════════════════════════════════

/// Status of a local model server (ollama, whisper.cpp, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
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

/// A managed local model server instance.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelServerInfo {
    /// Server kind (determines the Docker image / launch command).
    pub kind: ModelServerKind,
    /// HTTP endpoint (e.g. `http://localhost:8178`).
    pub endpoint: String,
    /// Current lifecycle status.
    pub status: ModelServerStatus,
    /// Docker container id (if managed by a container runtime).
    #[serde(default)]
    #[ts(optional)]
    pub container_id: Option<String>,
    /// Which models are loaded in this server.
    #[serde(default)]
    pub loaded_models: Vec<String>,
}

/// The type of local model server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelServerKind {
    /// Ollama — LLM + embedding models (candle/GGUF, CPU-first).
    Ollama,
    /// whisper.cpp — speech-to-text.
    WhisperCpp,
    /// vLLM — high-throughput LLM serving (GPU).
    Vllm,
    /// MediaPipe / pose estimation (vision — stub).
    MediaPipe,
}

// ═══════════════════════════════════════════════════════════════
// Model request — how the web UI asks for inference
// ═══════════════════════════════════════════════════════════════

/// A model inference request, sent from the web UI to the upstream engine
/// over the WS JSON-RPC channel. The engine routes it to the appropriate
/// backend (local CPU / local GPU / remote GPU / remote API).
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelInferenceRequest {
    /// Which model to use (by id).
    pub model_id: String,
    /// Input data (format depends on category: text for LLM/embedding,
    /// base64 audio for STT, base64 image for vision).
    pub input: String,
    /// Optional parameters (temperature, max_tokens, language hint …).
    #[serde(default)]
    #[ts(optional)]
    pub parameters: Option<serde_json::Value>,
}

/// Inference result returned to the web UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
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
    #[ts(optional)]
    pub usage: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════
// WS protocol — TuiMessage variants for model management
// ═══════════════════════════════════════════════════════════════

/// `Tui.RequestModelList` — enumerate available models.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestModelListParams {
    /// Filter by category (omit for all).
    #[serde(default)]
    #[ts(optional)]
    pub category: Option<ModelCategory>,
}

/// `Tui.ModelList` — model catalogue response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelListParams {
    pub models: Vec<ModelDescriptor>,
    pub servers: Vec<ModelServerInfo>,
}

/// `Tui.RequestModelInference` — ask the engine to run a model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestModelInferenceParams {
    pub request: ModelInferenceRequest,
}

/// `Tui.ModelInferenceResult` — inference result push.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelInferenceResultParams {
    pub result: ModelInferenceResult,
}

/// `Tui.RequestModelServerAction` — start / stop / restart a local model server.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestModelServerActionParams {
    pub kind: ModelServerKind,
    pub action: ModelServerAction,
}

/// `Tui.ModelServerActionResult` — action result.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ModelServerActionResultParams {
    pub kind: ModelServerKind,
    pub status: ModelServerStatus,
}

/// What to do with a model server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "WsTypes.ts")]
#[serde(rename_all = "snake_case")]
pub enum ModelServerAction {
    Start,
    Stop,
    Restart,
}
