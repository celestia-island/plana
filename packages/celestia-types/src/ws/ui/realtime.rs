//! Realtime omni-session protocol — full-duplex audio/video conversation.
//!
//! Wire vocabulary aligned with the Qwen-Omni-Realtime / OpenAI-Realtime
//! event family (`session.update`, `input_audio_buffer.*`,
//! `response.audio.delta`, `speech_started`, `response.done`). The gateway
//! proxies these events between clients and upstream realtime engines
//! (cloud Qwen-Omni-Realtime / OpenAI Realtime, or a local CEP engine).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Server-side VAD (voice activity detection) configuration for a realtime
/// session. Mirrors the upstream `turn_detection` payload.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/realtime.ts")]
pub struct RealtimeTurnDetection {
    /// `"server_vad"` | `"semantic_vad"` | `null` (manual commit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub r#type: Option<String>,
    /// VAD activation threshold in [-1, 1] (server_vad).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub threshold: Option<f32>,
    /// Padding to keep before speech onset, ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub prefix_padding_ms: Option<u32>,
    /// Silence duration that ends a user turn, ms (default 800).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub silence_duration_ms: Option<u32>,
    /// Idle timeout that force-commits the buffer, ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub idle_timeout_ms: Option<u32>,
    /// Semantic VAD: auto-create a response at end of user turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub create_response: Option<bool>,
    /// Semantic VAD: interrupt the current response when user speech begins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub interrupt_response: Option<bool>,
}

/// `session.update` payload — the full realtime session configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/realtime.ts")]
pub struct RealtimeSessionConfig {
    pub model: String,
    /// Output modalities: `["text", "audio"]` or `["text"]`.
    #[serde(default)]
    pub modalities: Vec<String>,
    /// Voice id for speech output (vendor-specific).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub voice: Option<String>,
    /// `"pcm16"` — input audio encoding (16 kHz).
    #[serde(default)]
    pub input_audio_format: String,
    /// `"pcm16"` — output audio encoding (24 kHz).
    #[serde(default)]
    pub output_audio_format: String,
    /// System prompt for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub turn_detection: Option<RealtimeTurnDetection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub temperature: Option<f32>,
}

/// One PCM audio block (base64-encoded in JSON payloads).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/realtime.ts")]
pub struct RealtimeAudioChunk {
    /// MIME type of the payload (e.g. `"audio/pcm"`).
    pub mime: String,
    /// Sample rate in Hz (16 kHz client→model, 24 kHz model→client).
    pub sample_rate: u32,
    /// Base64-encoded PCM16 LE audio bytes.
    pub data_base64: String,
}

/// One video frame (image) in a realtime session — e.g. a JPEG frame
/// streamed to the model at ~1 fps, or a generated frame streamed back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/realtime.ts")]
pub struct RealtimeVideoFrame {
    /// MIME type (e.g. `"image/jpeg"`).
    pub mime: String,
    /// Monotonic frame sequence number for ordering / dedup.
    pub frame_seq: u32,
    /// Base64-encoded image bytes.
    pub data_base64: String,
}

/// Token usage reported in `response.done` — surface for billing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/realtime.ts")]
pub struct RealtimeUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
}

/// Client → gateway realtime events (client-sent).
///
/// Wired as JSON-RPC notifications on the session channel; binary audio
/// (16 kHz PCM16) travels base64-encoded in `data_base64`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "ws/realtime.ts")]
pub enum RealtimeClientEvent {
    SessionUpdate { session: RealtimeSessionConfig },
    InputAudioBufferAppend { audio: RealtimeAudioChunk },
    InputAudioBufferCommit,
    InputAudioBufferClear,
    InputImageBufferAppend { frame: RealtimeVideoFrame },
    ResponseCreate,
    ResponseCancel,
    SessionStop,
}

/// Gateway → client realtime events (server-sent).
///
/// Wired as JSON-RPC notifications on the session channel; binary audio
/// (24 kHz PCM16) travels base64-encoded in `delta.data_base64`. Clients
/// stop playback on `speech_started` (barge-in) and treat `response_done`
/// as the terminal billing event for one response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "ws/realtime.ts")]
pub enum RealtimeServerEvent {
    SessionCreated {
        session: RealtimeSessionConfig,
    },
    SessionUpdated {
        session: RealtimeSessionConfig,
    },
    /// VAD detected speech onset — clients should stop playback (barge-in).
    SpeechStarted {
        audio_start_ms: u64,
    },
    SpeechStopped {
        audio_end_ms: u64,
    },
    ResponseCreated {
        response_id: String,
    },
    /// Streaming audio output block (base64 PCM16 24 kHz).
    ResponseAudioDelta {
        response_id: String,
        delta: RealtimeAudioChunk,
    },
    ResponseAudioDone {
        response_id: String,
    },
    /// Streaming transcript of the audio output.
    ResponseAudioTranscriptDelta {
        response_id: String,
        delta: String,
    },
    /// Text-only output delta (modalities `["text"]`).
    ResponseTextDelta {
        response_id: String,
        delta: String,
    },
    /// Terminal event for one response — carries usage for billing.
    ResponseDone {
        response_id: String,
        usage: RealtimeUsage,
    },
    /// Streaming video frame output (LPM-style character video / generated
    /// world-model frames delivered as a live frame stream).
    ResponseVideoFrame {
        response_id: String,
        frame: RealtimeVideoFrame,
    },
    Error {
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> RealtimeSessionConfig {
        RealtimeSessionConfig {
            model: "qwen3.5-omni-plus-realtime".to_string(),
            modalities: vec!["text".to_string(), "audio".to_string()],
            voice: Some("Cherry".to_string()),
            input_audio_format: "pcm16".to_string(),
            output_audio_format: "pcm16".to_string(),
            instructions: Some("You are a helpful assistant.".to_string()),
            turn_detection: Some(RealtimeTurnDetection {
                r#type: Some("server_vad".to_string()),
                threshold: Some(0.5),
                prefix_padding_ms: Some(300),
                silence_duration_ms: Some(800),
                idle_timeout_ms: None,
                create_response: None,
                interrupt_response: None,
            }),
            temperature: None,
        }
    }

    #[test]
    fn client_event_round_trip_tagged() {
        let ev = RealtimeClientEvent::SessionUpdate {
            session: sample_session(),
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeClientEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"type\":\"session_update\""));
    }

    #[test]
    fn client_event_audio_append_round_trip() {
        let ev = RealtimeClientEvent::InputAudioBufferAppend {
            audio: RealtimeAudioChunk {
                mime: "audio/pcm".to_string(),
                sample_rate: 16_000,
                data_base64: "AAAAIAAg".to_string(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeClientEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"input_audio_buffer_append\""));
    }

    #[test]
    fn server_event_round_trip_tagged() {
        let ev = RealtimeServerEvent::ResponseAudioDelta {
            response_id: "resp_1".to_string(),
            delta: RealtimeAudioChunk {
                mime: "audio/pcm".to_string(),
                sample_rate: 24_000,
                data_base64: "QUJD".to_string(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeServerEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"response_audio_delta\""));
    }

    #[test]
    fn server_speech_started_round_trip() {
        let ev = RealtimeServerEvent::SpeechStarted {
            audio_start_ms: 120,
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeServerEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"speech_started\""));
    }

    #[test]
    fn response_done_carries_usage() {
        let ev = RealtimeServerEvent::ResponseDone {
            response_id: "resp_1".to_string(),
            usage: RealtimeUsage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeServerEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"input_tokens\":10"));
    }

    #[test]
    fn video_frame_round_trip() {
        let frame = RealtimeVideoFrame {
            mime: "image/jpeg".to_string(),
            frame_seq: 3,
            data_base64: "Zm9v".to_string(),
        };
        let s = serde_json::to_string(&frame).unwrap();
        let back: RealtimeVideoFrame = serde_json::from_str(&s).unwrap();
        assert_eq!(back, frame);
    }

    #[test]
    fn response_video_frame_round_trip() {
        let ev = RealtimeServerEvent::ResponseVideoFrame {
            response_id: "resp_v".to_string(),
            frame: RealtimeVideoFrame {
                mime: "image/jpeg".to_string(),
                frame_seq: 7,
                data_base64: "aW1n".to_string(),
            },
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: RealtimeServerEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ev);
        assert!(s.contains("\"response_video_frame\""));
    }

    #[test]
    fn turn_detection_defaults_are_optional() {
        let cfg = RealtimeTurnDetection::default();
        let s = serde_json::to_string(&cfg).unwrap();
        assert_eq!(s, "{}");
        let back: RealtimeTurnDetection = serde_json::from_str("{}").unwrap();
        assert_eq!(back, cfg);
    }
}
