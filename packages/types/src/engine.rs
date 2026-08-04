//! Celestia Engine Protocol (CEP) — unified model-runtime access protocol.
//!
//! Any model-serving engine — llama.cpp, vLLM, a speech/sensor model, or a
//! custom engine written in any language — speaks this protocol over a
//! WebSocket transport using JSON-RPC 2.0 envelopes (see
//! [`crate::protocol::jsonrpc`]) to join the arona gateway's Router cluster.
//! The gateway acts as the intermediary: clients never talk to engines
//! directly, only to the gateway, and the gateway multiplexes requests
//! across engines via least-count routing, session affinity and
//! capacity-aware placement.
//!
//! ## The protocol is capability-driven, not input/output-locked
//!
//! CEP does NOT assume a text LLM. Engines declare what they actually
//! consume and produce at handshake time (`modalities`, `content_types`),
//! and request payloads are free-form JSON ([`EngineInvokeParams`]) or
//! content-part messages ([`EngineMessage`]) that can carry text, base64
//! binary (audio/image/video), structured sensor readings or arbitrary
//! data. The convenience methods `Engine.Chat` / `Engine.Embeddings` are
//! merely the two most common shapes; engines with specialised I/O
//! (audio generation, sensor-signal processing, tensor streaming…) use
//! `Engine.Invoke` with engine-defined method names and payloads. The
//! gateway routes by declared capability and passes unknown payloads
//! through untouched.
//!
//! ## Wire methods (JSON-RPC method names)
//!
//! | Method | Direction | Purpose |
//! |--------|-----------|---------|
//! | `Engine.Handshake` | engine → gateway (first) | Identity + capability declaration |
//! | `Engine.Chat` | gateway → engine | Convenience: non-streaming text chat |
//! | `Engine.ChatStart` | gateway → engine | Convenience: streaming text chat |
//! | `Engine.ChatChunk` | engine → gateway (notification) | Streamed text token delta |
//! | `Engine.Embeddings` | gateway → engine | Convenience: batch text embeddings |
//! | `Engine.Invoke` | gateway → engine | Generic method: any engine-defined operation |
//! | `Engine.InvokeStart` | gateway → engine | Generic streaming invocation |
//! | `Engine.StreamChunk` | engine → gateway (notification) | Generic streamed data block (any mime) |
//! | `Engine.Models` | gateway → engine | List serving models |
//! | `Engine.Stats` | gateway → engine | Telemetry (GPU utilisation etc.) |
//! | `Engine.Shutdown` | gateway → engine | Graceful stop (deploy stop path) |
//!
//! ## Streaming
//!
//! Text streams use the convenience `Engine.ChatStart` / `Engine.ChatChunk`
//! pair (gateway-generated `stream_id` correlation). Generic streams —
//! audio frames, sensor samples, tensors — use `Engine.InvokeStart`
//! (same `stream_id` accept/notify shape) with [`EngineStreamChunk`]
//! carrying a mime + encoding description so the consumer can decode
//! without prior agreement. Large binary blocks may additionally travel
//! as WebSocket binary frames: a JSON notification announces the chunk
//! (`encoding: "binary-frame"`), and the immediately following WS binary
//! frame carries the bytes.
//!
//! ## Handshake
//!
//! The engine MUST send `Engine.Handshake` as its first message. The
//! gateway replies with [`EngineHandshakeResult`]; a rejected handshake
//! closes the connection with the given error.
//!
//! ## Binary transfer (JSON-RPC cannot carry binary — this is the escape
//! hatch, and every byte is labelled with a MIME type)
//!
//! JSON-RPC 2.0 defines messages as JSON objects; there is no binary frame
//! in the spec. WebSocket, however, has first-class binary frames that may
//! interleave with text frames on one connection. CEP uses that:
//!
//! 1. **Announce** — the sender emits a JSON-RPC *notification*
//!    `Engine.BinaryStart` declaring `transfer_id`, the MIME type of the
//!    payload, `total_bytes`, the expected binary-frame count and an
//!    optional SHA-256 hex checksum. The receiver switches to
//!    "binary-receive" state for this transfer id.
//! 2. **Payload** — the bytes travel as a sequence of raw WebSocket
//!    **binary frames** (no JSON envelope). Senders MUST split large
//!    payloads into frames of at most [`ENGINE_BINARY_MAX_FRAME_BYTES`]
//!    so interop holds across implementations (browsers, embedded
//!    engines). The receiver concatenates frames in arrival order.
//! 3. **Finish** — after the last frame the sender emits the JSON-RPC
//!    *notification* `Engine.BinaryEnd` with the received byte count and
//!    checksum status. The receiver validates and returns to normal RPC
//!    operation. A single connection carries at most one active binary
//!    transfer at a time (frames are ordered per-connection; concurrent
//!    transfers would be ambiguous — use separate connections or the
//!    stream channel for concurrency).
//! 4. **Abort** — either side may emit `Engine.BinaryAbort` to cancel;
//!    the receiver discards buffered bytes and returns to normal RPC.
//!
//! ### Resynchronisation & failure handling (how the receiver always finds
//! the next JSON message)
//!
//! WebSocket frames carry a type (Text vs Binary) and TCP underneath makes
//! frames ordered and lossless within a connection — the receiver's state
//! machine therefore never scans bytes for JSON boundaries:
//!
//! - **Normal state** accepts only Text frames; a Binary frame without an
//!   active transfer is a protocol violation → the connection is closed.
//! - **Binary-receive state** accepts only Binary frames; a Text frame that
//!   is not `Engine.BinaryEnd`/`Engine.BinaryAbort` means the sender
//!   abandoned the transfer → buffered bytes are discarded and the Text
//!   frame is processed normally (the state machine simply resets).
//! - **Timeouts** ([`ENGINE_BINARY_RECEIVE_TIMEOUT_SECS`]) guard against a
//!   sender that dies mid-transfer: if the first frame after
//!   `Engine.BinaryStart`, or any inter-frame gap, exceeds the timeout the
//!   receiver aborts the transfer (locally and/or via `Engine.BinaryAbort`)
//!   and returns to normal RPC.
//! - **Corruption** (byte-count or checksum mismatch at
//!   `Engine.BinaryEnd`) invalidates the whole transfer — binary payloads
//!   have no internal boundaries, so partial data can never be trusted.
//! - **Hard failure** (connection close mid-transfer) resets everything:
//!   reconnect + re-handshake, then re-run the transfer.
//!
//! No sliding window is needed: frames are ordered and lossless, so there
//! is nothing to reorder or retransmit inside a connection.
//!
//! Every transfer is labelled with a MIME type ([`EngineBinaryStartParams`])
//! — the same MIME vocabulary used by content parts and stream chunks — so
//! consumers never guess what the bytes are.
//!
//! ### Why not base64-only?
//!
//! Base64 works everywhere but costs +33% bandwidth and CPU. The
//! announce/payload/finish triple keeps the JSON-RPC control plane intact
//! (handshake, correlation, errors all stay JSON) while the bulk bytes
//! avoid the encoding tax. Engines that prefer simplicity may still send
//! base64 content parts — both paths are first-class.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// CEP wire-protocol version. Bumped on incompatible payload changes.
pub const ENGINE_PROTOCOL_VERSION: u32 = 3;

/// Upper bound for a single binary-transfer frame, in bytes. Senders MUST
/// split larger payloads; receivers SHOULD refuse oversized frames.
pub const ENGINE_BINARY_MAX_FRAME_BYTES: usize = 256 * 1024;

/// Timeout guard for an in-flight binary transfer, in seconds. The
/// receiver aborts the transfer if the first frame after `Engine.BinaryStart`
/// or any inter-frame gap exceeds this bound (sender died mid-transfer).
/// A transfer this size or smaller should normally complete in well under
/// this budget.
pub const ENGINE_BINARY_RECEIVE_TIMEOUT_SECS: u64 = 60;

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

/// Input/output modalities an engine can handle. The gateway does NOT
/// assume text — it routes and passes payloads through based on this
/// declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub enum EngineModality {
    Text,
    Audio,
    Image,
    Video,
    Sensor,
    Tensor,
    Generic,
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
    /// Modalities the engine can consume as input (empty = text only).
    #[serde(default)]
    pub input_modalities: Vec<EngineModality>,
    /// Modalities the engine can produce as output.
    #[serde(default)]
    pub output_modalities: Vec<EngineModality>,
    /// MIME content types accepted as input (e.g. "audio/wav",
    /// "application/octet-stream", "application/json").
    #[serde(default)]
    pub content_types: Vec<String>,
    /// Engine-defined `Engine.Invoke` method names beyond the standard
    /// convenience methods (e.g. "audio.generate", "signal.filter").
    /// Any engine-specific operation is reachable via `Engine.Invoke`
    /// even when absent from this list.
    #[serde(default)]
    pub methods: Vec<String>,
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
///
/// Handshake direction: when the gateway connects to an engine (engine is
/// the server), the gateway sends `Engine.Handshake` and the engine answers
/// with this result carrying its **own** declared capabilities, so the
/// gateway learns modalities/content types before any request. When the
/// engine connects to the gateway (engine is the client), the engine sends
/// `Engine.Handshake` with its capabilities in the params and the gateway
/// answers with `ok` only — the `capabilities` field is then ignored.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineHandshakeResult {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    pub protocol_version: u32,
    /// The engine's own capability declaration (server-mode handshake).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub capabilities: Option<EngineCapabilities>,
}

// ═══════════════════════════════════════════════════════════
// Chat
// ═══════════════════════════════════════════════════════════

/// One content unit inside a message. Text is a plain string; everything
/// else is a data block described by mime + encoding so consumers can
/// decode without prior agreement:
/// - `data`: base64 bytes (standard `encoding: "base64"`)
/// - `encoding: "binary-frame"`: bytes arrive in the immediately following
///   WebSocket binary frame (JSON notification is the announcer/trailer)
/// - `encoding: "json"`: `data` is inline JSON (structured sensor readings,
///   tensors, feature vectors…)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineContentPart {
    /// MIME type — "text/plain" for plain text parts.
    pub mime: String,
    /// Encoding of `data` ("base64" | "binary-frame" | "json" | "utf-8").
    pub encoding: String,
    /// Payload: base64 text, inline JSON, or raw text depending on
    /// `encoding`. Empty for binary-frame parts (bytes follow as a WS
    /// binary frame).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub data: Option<serde_json::Value>,
    /// Optional shape hint for tensor/sensor parts, e.g. [1, 16000]
    /// (channels × samples) or the sensor schema id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub shape: Option<Vec<usize>>,
}

impl EngineContentPart {
    pub fn text(content: &str) -> Self {
        Self {
            mime: "text/plain".into(),
            encoding: "utf-8".into(),
            data: Some(serde_json::Value::String(content.to_string())),
            shape: None,
        }
    }

    pub fn base64(mime: &str, bytes: &str) -> Self {
        Self {
            mime: mime.into(),
            encoding: "base64".into(),
            data: Some(serde_json::Value::String(bytes.to_string())),
            shape: None,
        }
    }

    pub fn json(mime: &str, value: serde_json::Value) -> Self {
        Self {
            mime: mime.into(),
            encoding: "json".into(),
            data: Some(value),
            shape: None,
        }
    }

    pub fn binary_frame(mime: &str) -> Self {
        Self {
            mime: mime.into(),
            encoding: "binary-frame".into(),
            data: None,
            shape: None,
        }
    }
}

/// A message in an `Engine.Chat` / `Engine.ChatStart` / `Engine.Invoke`
/// payload. Content is a list of parts so mixed-modality inputs (text +
/// audio + sensor…) are representable. `role` is advisory; specialised
/// engines may ignore it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineMessage {
    pub role: String,
    pub content: Vec<EngineContentPart>,
}

impl EngineMessage {
    pub fn text(role: &str, content: &str) -> Self {
        Self {
            role: role.into(),
            content: vec![EngineContentPart::text(content)],
        }
    }
}

/// `Engine.Chat` (non-streaming) / `Engine.ChatStart` (streaming) params.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineChatParams {
    pub model: String,
    pub messages: Vec<EngineMessage>,
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
// Generic invocation (capability-driven extension channel)
// ═══════════════════════════════════════════════════════════

/// `Engine.Invoke` / `Engine.InvokeStart` params — the generic extension
/// channel. `method` is engine-defined (e.g. "audio.generate",
/// "signal.filter", "train.step"); `params` is any JSON the engine
/// understands. `messages` is optional and reuses the multimodal content
/// model for engines that mix free-form payloads with content parts.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineInvokeParams {
    pub method: String,
    pub params: serde_json::Value,
    #[serde(default)]
    #[ts(optional)]
    pub messages: Option<Vec<EngineMessage>>,
    /// Present for streaming invocations — chunks are tagged with this id.
    #[serde(default)]
    #[ts(optional)]
    pub stream_id: Option<String>,
}

/// `Engine.Invoke` result — any JSON the engine returns.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineInvokeResult {
    pub method: String,
    pub result: serde_json::Value,
}

/// `Engine.InvokeStart` acceptance result (same shape as ChatStart).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineInvokeStartResult {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    pub stream_id: String,
}

/// `Engine.StreamChunk` notification — a generic streamed data block for
/// any output modality (audio frame, sensor sample batch, tensor slice…).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineStreamChunk {
    pub stream_id: String,
    /// MIME type of this block (e.g. "audio/wav", "application/json").
    pub mime: String,
    /// "base64" | "binary-frame" | "json" | "utf-8".
    pub encoding: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub data: Option<serde_json::Value>,
    /// Optional shape hint for tensor/sensor blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub shape: Option<Vec<usize>>,
    /// Set on the final block.
    #[serde(default)]
    pub is_complete: bool,
    #[serde(default)]
    #[ts(optional)]
    pub usage: Option<EngineUsage>,
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

// ═══════════════════════════════════════════════════════════
// Binary transfer (JSON-RPC control plane + WS binary frames)
// ═══════════════════════════════════════════════════════════

/// `Engine.BinaryStart` notification params — the announce packet sent
/// BEFORE any binary frame. Every payload is labelled with a MIME type.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineBinaryStartParams {
    /// Correlation id shared by announce / frames / finish.
    pub transfer_id: String,
    /// MIME type of the whole payload (e.g. "audio/wav",
    /// "application/octet-stream"). Empty means unspecified binary.
    pub mime: String,
    pub total_bytes: u64,
    /// Expected number of binary frames (for early truncation checks).
    pub chunk_count: u32,
    /// Optional SHA-256 hex digest of the payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub checksum: Option<String>,
    /// Optional association with a stream (generic streaming).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub stream_id: Option<String>,
}

/// `Engine.BinaryEnd` notification params — sent after the final binary
/// frame; the receiver validates and returns to normal RPC operation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineBinaryEndParams {
    pub transfer_id: String,
    /// Bytes actually received across all frames.
    pub bytes_received: u64,
    /// Whether the checksum matched (None when no checksum was announced).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub checksum_ok: Option<bool>,
}

/// `Engine.BinaryAbort` notification params — cancels an in-flight
/// transfer; the receiver discards buffered bytes and returns to normal
/// RPC operation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "engine.ts")]
pub struct EngineBinaryAbortParams {
    pub transfer_id: String,
    pub reason: String,
}
