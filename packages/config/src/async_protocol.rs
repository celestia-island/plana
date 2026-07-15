//! Async protocol handler — the submit → poll → result pattern shared by
//! all asynchronous generation protocols (Seedance 2.0, Kling, Sora, Jimeng,
//! Hyper3D, Hunyuan 3D, Google Veo).
//!
//! ## Protocol flow
//!
//! ```text
//! Client                         Server
//!   |── POST submit ──────────────→|  creates task, returns task_id
//!   |←── { id, status: "queued" ──|
//!   |                              |
//!   |── GET poll(task_id) ───────→|  (repeat until terminal)
//!   |←── { status: "generating" ──|
//!   |── GET poll(task_id) ───────→|
//!   |←── { status: "completed", ──|
//!   |     result: { url, ... } }  |
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::gen_protocol::{Capability, GenProtocol};

// ─── AsyncGenRequest ───────────────────────────────────────────────

/// A generation request for an async protocol.
///
/// The `protocol` field determines which submit/poll endpoints to use.
/// `params` carries protocol-specific fields (aspect_ratio, duration,
/// resolution, seed, etc.) as a flexible JSON blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncGenRequest {
    /// The protocol to use — must be one of the async variants.
    pub protocol: GenProtocol,

    /// Model identifier as the provider expects it
    /// (e.g. `"bytedance/seedance-2-0"`, `"kling-v3"`).
    pub model: String,

    /// Text prompt describing what to generate.
    pub prompt: String,

    /// Optional reference inputs (image URLs, audio URLs, video URLs).
    #[serde(default)]
    pub reference_inputs: Vec<ReferenceInput>,

    /// Protocol-specific parameters (aspect_ratio, duration, resolution, seed,
    /// watermark, etc.).
    #[serde(default)]
    pub params: serde_json::Value,
}

/// A reference input for guided generation (image-to-video, audio lip-sync, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceInput {
    /// What kind of reference this is.
    pub kind: ReferenceKind,
    /// URL or base64-encoded data.
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    Image,
    Audio,
    Video,
    #[serde(rename = "model3d")]
    Model3D,
}

// ─── AsyncTaskStatus ───────────────────────────────────────────────

/// Status of an async generation task, returned by `poll()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AsyncTaskStatus {
    /// Task is queued, waiting for a worker.
    Queued { task_id: String },

    /// Task is actively generating. `progress` is 0.0–1.0 if available.
    Generating {
        task_id: String,
        #[serde(default)]
        progress: Option<f32>,
    },

    /// Task completed successfully. Contains the result URL.
    Completed {
        task_id: String,
        result: AsyncGenResult,
    },

    /// Task failed. Contains the error reason.
    Failed { task_id: String, reason: String },
}

impl AsyncTaskStatus {
    pub fn task_id(&self) -> &str {
        match self {
            Self::Queued { task_id }
            | Self::Generating { task_id, .. }
            | Self::Completed { task_id, .. }
            | Self::Failed { task_id, .. } => task_id,
        }
    }

    /// Whether this status is terminal (no more polling needed).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }

    /// Whether the task succeeded.
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Extract the result if completed.
    pub fn result(&self) -> Option<&AsyncGenResult> {
        match self {
            Self::Completed { result, .. } => Some(result),
            _ => None,
        }
    }
}

/// The result of a completed async generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncGenResult {
    /// Download URL for the generated asset.
    #[serde(default)]
    pub url: Option<String>,

    /// Base64-encoded data, if the provider returns inline data.
    #[serde(default)]
    pub base64: Option<String>,

    /// What was generated, based on the protocol's capabilities.
    pub output_capability: Capability,

    /// Additional metadata (duration, width, height, format, etc.).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

// ─── AsyncProtocolHandler trait ────────────────────────────────────

/// Handler for asynchronous generation protocols.
///
/// All async protocols share the same submit → poll → cancel flow. Concrete
/// implementations translate between this unified interface and the
/// provider-specific wire format.
///
/// ## Implementation pattern
///
/// Each provider (Seedance, Kling, Sora, Hyper3D, etc.) implements this trait.
/// The `protocol()` method returns the [`GenProtocol`] variant it handles.
/// A registry maps `GenProtocol` → `Arc<dyn AsyncProtocolHandler>`.
#[async_trait]
pub trait AsyncProtocolHandler: Send + Sync {
    /// Which protocol this handler implements.
    fn protocol(&self) -> GenProtocol;

    /// Submit a generation task. Returns the task ID on success.
    ///
    /// The task is created on the provider's server. The caller should then
    /// call [`poll`](Self::poll) until the status is terminal.
    async fn submit(&self, request: AsyncGenRequest) -> Result<String, AsyncProtocolError>;

    /// Poll the status of a previously submitted task.
    ///
    /// Returns the current status. Callers should implement exponential
    /// backoff or fixed-interval polling (typically 10–15 seconds).
    async fn poll(&self, task_id: &str) -> Result<AsyncTaskStatus, AsyncProtocolError>;

    /// Cancel a pending or in-progress task.
    ///
    /// Not all providers support cancellation. The default implementation
    /// returns [`AsyncProtocolError::Unsupported`].
    async fn cancel(&self, _task_id: &str) -> Result<(), AsyncProtocolError> {
        Err(AsyncProtocolError::Unsupported)
    }
}

// ─── Errors ────────────────────────────────────────────────────────

#[derive(Debug, Clone, thiserror::Error)]
pub enum AsyncProtocolError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("server error: status={status}, body={body}")]
    Server { status: u16, body: String },

    #[error("operation not supported by this protocol")]
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_task_id_always_available() {
        let statuses = vec![
            AsyncTaskStatus::Queued {
                task_id: "q1".into(),
            },
            AsyncTaskStatus::Generating {
                task_id: "g1".into(),
                progress: Some(0.5),
            },
            AsyncTaskStatus::Completed {
                task_id: "c1".into(),
                result: AsyncGenResult {
                    url: Some("https://example.com/video.mp4".into()),
                    base64: None,
                    output_capability: Capability::GenerateVideo,
                    metadata: serde_json::json!({}),
                },
            },
            AsyncTaskStatus::Failed {
                task_id: "f1".into(),
                reason: "timeout".into(),
            },
        ];
        for status in &statuses {
            assert!(!status.task_id().is_empty());
        }
    }

    #[test]
    fn task_status_terminal() {
        assert!(
            AsyncTaskStatus::Failed {
                task_id: "x".into(),
                reason: "err".into()
            }
            .is_terminal()
        );

        assert!(
            AsyncTaskStatus::Completed {
                task_id: "x".into(),
                result: AsyncGenResult {
                    url: None,
                    base64: None,
                    output_capability: Capability::GenerateImage,
                    metadata: serde_json::json!({}),
                }
            }
            .is_terminal()
        );

        assert!(
            !AsyncTaskStatus::Queued {
                task_id: "x".into()
            }
            .is_terminal()
        );
        assert!(
            !AsyncTaskStatus::Generating {
                task_id: "x".into(),
                progress: None
            }
            .is_terminal()
        );
    }

    #[test]
    fn async_request_serialization() {
        let req = AsyncGenRequest {
            protocol: GenProtocol::Seedance2AsyncV1,
            model: "bytedance/seedance-2-0".into(),
            prompt: "A dragon flying over mountains".into(),
            reference_inputs: vec![ReferenceInput {
                kind: ReferenceKind::Image,
                url: "https://example.com/ref.jpg".into(),
            }],
            params: serde_json::json!({
                "aspect_ratio": "16:9",
                "duration": 5,
                "resolution": "720p"
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: AsyncGenRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.protocol, GenProtocol::Seedance2AsyncV1);
        assert_eq!(back.reference_inputs.len(), 1);
    }

    #[test]
    fn reference_kind_model3d_serializes_correctly() {
        // Regression: serde rename_all=snake_case turns Model3D into "model3_d"
        let json = serde_json::to_string(&ReferenceKind::Model3D).unwrap();
        assert_eq!(
            json, "\"model3d\"",
            "Model3D must serialize as model3d not model3_d"
        );
        let back: ReferenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ReferenceKind::Model3D);
    }

    #[test]
    fn reference_kind_all_variants_snake_case() {
        // Verify all variants serialize as snake_case (except Model3D override)
        assert_eq!(
            serde_json::to_string(&ReferenceKind::Image).unwrap(),
            "\"image\""
        );
        assert_eq!(
            serde_json::to_string(&ReferenceKind::Audio).unwrap(),
            "\"audio\""
        );
        assert_eq!(
            serde_json::to_string(&ReferenceKind::Video).unwrap(),
            "\"video\""
        );
        assert_eq!(
            serde_json::to_string(&ReferenceKind::Model3D).unwrap(),
            "\"model3d\""
        );
    }

    #[test]
    fn async_task_status_serde_roundtrip() {
        // Internally-tagged enum — must survive JSON round-trip
        let cases = vec![
            serde_json::json!({"status": "queued", "task_id": "q1"}),
            serde_json::json!({"status": "generating", "task_id": "g1", "progress": 0.5}),
            serde_json::json!({
                "status": "completed",
                "task_id": "c1",
                "result": {
                    "url": "https://example.com/v.mp4",
                    "output_capability": "generate_video",
                    "metadata": {}
                }
            }),
            serde_json::json!({"status": "failed", "task_id": "f1", "reason": "timeout"}),
        ];
        for json in cases {
            let status: AsyncTaskStatus = serde_json::from_value(json.clone()).unwrap();
            let reser = serde_json::to_value(&status).unwrap();
            assert_eq!(
                reser["status"], json["status"],
                "status tag mismatch: {:?} vs {:?}",
                reser, json
            );
        }
    }

    #[test]
    fn async_gen_result_with_capability_serde() {
        let result = AsyncGenResult {
            url: Some("https://example.com/img.png".into()),
            base64: None,
            output_capability: Capability::GenerateImage,
            metadata: serde_json::json!({"width": 1024}),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(
            json.contains("\"generate_image\""),
            "capability must serialize as snake_case: {}",
            json
        );
        let back: AsyncGenResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.output_capability, Capability::GenerateImage);
    }
}
