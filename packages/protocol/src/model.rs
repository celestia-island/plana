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
//! | Vision | mediapipe pose/gesture | chest (holographic AR mode) — **stub** |

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
    /// Vision / pose / gesture recognition (MediaPipe etc.).
    /// Stub — implementation deferred until AR/holographic hardware is available.
    Vision,
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
#[ts(export, export_to = "model.ts")]
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
    #[ts(optional)]
    pub usage: Option<serde_json::Value>,
}

// ═══════════════════════════════════════════════════════════════
// WS protocol — TuiMessage variants for model management
// ═══════════════════════════════════════════════════════════════

/// `Tui.RequestModelList` — enumerate available models.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct RequestModelListParams {
    /// Filter by category (omit for all).
    #[serde(default)]
    #[ts(optional)]
    pub category: Option<ModelCategory>,
}

/// `Tui.ModelList` — model catalogue response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct ModelListParams {
    pub models: Vec<ModelDescriptor>,
    pub servers: Vec<ModelServerInfo>,
}

/// `Tui.RequestModelInference` — ask the engine to run a model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct RequestModelInferenceParams {
    pub request: ModelInferenceRequest,
}

/// `Tui.ModelInferenceResult` — inference result push.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "model.ts")]
pub struct ModelInferenceResultParams {
    pub result: ModelInferenceResult,
}

/// `Tui.RequestModelServerAction` — ask evernight (via scepter) to start /
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

/// `Tui.ModelServerActionResult` — action result.
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
