//! Celestia Engine Protocol (CEP) — unified model-runtime access protocol.
//!
//! Any model-serving engine — llama.cpp, vLLM, or a custom engine written in
//! any language — speaks this protocol over a WebSocket transport using
//! JSON-RPC 2.0 envelopes (see [`crate::protocol::jsonrpc`]) to join the
//! arona gateway's Router cluster. The gateway acts as the intermediary:
//! clients never talk to engines directly, only to the gateway, and the
//! gateway multiplexes requests across engines via least-count routing,
//! session affinity and capacity-aware placement.
//!
//! ## Wire methods (JSON-RPC method names)
//!
//! | Method | Direction | Purpose |
//! |--------|-----------|---------|
//! | `Engine.Handshake` | engine → gateway (first) | Identity + capability declaration |
//! | `Engine.Chat` | gateway → engine | Non-streaming chat completion |
//! | `Engine.ChatStart` | gateway → engine | Begin a streaming chat completion |
//! | `Engine.ChatChunk` | engine → gateway (notification) | Streamed token delta |
//! | `Engine.Embeddings` | gateway → engine | Batch text embeddings |
//! | `Engine.Models` | gateway → engine | List serving models |
//! | `Engine.Stats` | gateway → engine | Telemetry (GPU utilisation etc.) |
//! | `Engine.Shutdown` | gateway → engine | Graceful stop (deploy stop path) |
//!
//! Streaming follows the gateway's existing `chat.stream` pattern: the
//! gateway sends `Engine.ChatStart` (carrying a gateway-generated
//! `stream_id`), the engine answers one `Engine.ChatStartResult` acceptance
//! response, then pushes zero or more `Engine.ChatChunk` JSON-RPC
//! notifications and finishes with a chunk whose `is_complete` is true.
//!
//! Handshake: the engine MUST send `Engine.Handshake` as its first message.
//! The gateway replies with [`EngineHandshakeResult`]; a rejected handshake
//! closes the connection with the given error.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// CEP wire-protocol version. Bumped on incompatible payload changes.
pub const ENGINE_PROTOCOL_VERSION: u32 = 1;

// ═══════════════════════════════════════════════════════════
// Handshake / identity
// ═══════════════════════════════════════════════════════════

/// `Engine.Handshake` params — the engine's first message on connect.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineHandshakeParams {
    /// Optional shared token; the gateway rejects mismatches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub token: Option<String>,
    pub engine: EngineIdentity,
    pub capabilities: EngineCapabilities,
}

/// Engine implementation identity (any language is fine — this is the
/// interchange contract).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineIdentity {
    pub name: String,
    pub version: String,
    /// Implementation language, e.g. "rust", "cpp".
    #[serde(default)]
    #[ts(optional)]
    pub language: Option<String>,
    /// Optional vendor URL.
    #[serde(default)]
    #[ts(optional)]
    pub vendor: Option<String>,
}

/// Static capability declaration supplied at handshake time.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineCapabilities {
    #[serde(default = "default_true")]
    pub streaming: bool,
    #[serde(default)]
    pub embeddings: bool,
    #[serde(default = "default_context")]
    pub max_context_length: usize,
    #[serde(default)]
    pub hardware: Vec<EngineGpuInfo>,
}

fn default_true() -> bool {
    true
}

fn default_context() -> usize {
    128_000
}

/// One GPU the engine can drive — used by capacity-aware placement.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineGpuInfo {
    pub name: String,
    #[serde(default)]
    pub vram_gb: u64,
}

/// `Engine.Handshake` result. `ok: false` closes the connection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineHandshakeResult {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    pub protocol_version: u32,
}

// ═══════════════════════════════════════════════════════════
// Chat
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatMessage {
    pub role: String,
    pub content: String,
}

/// `Engine.Chat` (non-streaming) / `Engine.ChatStart` (streaming) params.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatParams {
    pub model: String,
    pub messages: Vec<EngineChatMessage>,
    #[serde(default)]
    #[ts(optional)]
    pub temperature: Option<f32>,
    #[serde(default)]
    #[ts(optional)]
    pub max_tokens: Option<u32>,
    /// Present for streaming requests — chunks are tagged with this id.
    #[serde(default)]
    #[ts(optional)]
    pub stream_id: Option<String>,
    /// Free-form passthrough merged into the upstream payload (same
    /// semantics as the gateway's `extra` field).
    #[serde(default)]
    #[ts(optional)]
    pub extra: Option<serde_json::Value>,
}

/// `Engine.Chat` result for a non-streaming completion.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatResult {
    pub model: String,
    pub content: String,
    #[serde(default)]
    #[ts(optional)]
    pub usage: Option<EngineUsage>,
}

/// `Engine.ChatStart` acceptance result. `ok: false` rejects the stream
/// before any chunk is sent.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatStartResult {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    pub stream_id: String,
}

/// `Engine.ChatChunk` notification — streamed token delta.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatChunk {
    pub stream_id: String,
    #[serde(default)]
    pub token: String,
    /// Set on the final chunk.
    #[serde(default)]
    pub is_complete: bool,
    #[serde(default)]
    #[ts(optional)]
    pub usage: Option<EngineUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineUsage {
    #[serde(default)]
    pub prompt_tokens: usize,
    #[serde(default)]
    pub completion_tokens: usize,
}

// ═══════════════════════════════════════════════════════════
// Embeddings
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineEmbeddingsParams {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineEmbeddingsResult {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
}

// ═══════════════════════════════════════════════════════════
// Models / stats / shutdown
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineModelInfo {
    pub id: String,
    #[serde(default)]
    #[ts(optional)]
    pub context_length: Option<usize>,
    #[serde(default)]
    pub embedding: bool,
}

/// `Engine.Models` result.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineModelsResult {
    pub models: Vec<EngineModelInfo>,
}

/// `Engine.Stats` result — live telemetry for capacity-aware placement.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineStatsResult {
    /// Per-GPU utilisation percentages (0-100), same shape as the agent
    /// control-plane heartbeats.
    #[serde(default)]
    pub gpu_utilization: Vec<u32>,
    #[serde(default)]
    pub uptime_secs: u64,
    /// Model id currently loaded, when the engine pins a single model.
    #[serde(default)]
    #[ts(optional)]
    pub model_loaded: Option<String>,
}

/// `Engine.Shutdown` params — graceful stop requested by the gateway.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineShutdownParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}
