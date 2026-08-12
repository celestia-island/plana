//! MDD — Model Deployment Descriptor schema v1.
//!
//! A single, engine-agnostic description of how a generative model is
//! decomposed, wired, and deployed: which components (encoder / DiT / VAE
//! …) exist, how they depend on each other, which runtimes serve them,
//! what the inference API looks like, and how the deployment scales.
//!
//! Consumers read an [`MddDescriptor`] and derive a concrete plan from it:
//!
//! ```text
//!   MDD file ──▶ planner ──▶ component DAG ──▶ runtime selection ──▶ tiers
//!                                  │
//!                                  ▼
//!                   evernight deploys the matching runtimes
//! ```
//!
//! Schema versioning is explicit: `schema_version` identifies the descriptor
//! layout (currently `1`). Older readers must reject newer versions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Top-level descriptor
// ═══════════════════════════════════════════════════════════════

/// Top-level model deployment descriptor — the root of an MDD v1 document.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddDescriptor {
    /// Schema version of this descriptor (currently `1`).
    pub schema_version: u32,
    /// Model identity and architecture summary.
    pub model: MddModel,
    /// Components that make up the model, as a DAG (see `dependencies`).
    pub components: Vec<MddComponent>,
    /// Inference pipeline and API wiring.
    pub deploy: MddDeploy,
    /// Scale / sizing estimation for capacity planning.
    pub scale: MddScale,
}

/// Model identity block of an MDD descriptor.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddModel {
    /// Unique model id, e.g. `"demo-h3-mini"`.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Model family / lineage, e.g. `"demo-dit"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub family: Option<String>,
    /// Architecture identifier, e.g. `"dit"`, `"llama"`.
    pub architecture: String,
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Components — the model decomposed into DAG nodes
// ═══════════════════════════════════════════════════════════════

/// One node of the model component DAG.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddComponent {
    /// Unique component id within the descriptor.
    pub id: String,
    /// What role the component plays.
    pub kind: MddComponentKind,
    /// Architecture hint, e.g. `"transformer-encoder"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub arch: Option<String>,
    /// DAG edges: ids of components this one depends on (upstream).
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Input contracts accepted by this component.
    #[serde(default)]
    pub inputs: Vec<MddIoContract>,
    /// Output contracts produced by this component.
    #[serde(default)]
    pub outputs: Vec<MddIoContract>,
    /// Runtimes available for this component (best-effort ordering).
    pub runtimes: Vec<MddRuntime>,
}

/// The role a component plays inside a generative pipeline.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddComponentKind {
    /// Text / conditioning encoder.
    Encoder,
    /// Diffusion transformer (core denoising backbone).
    Dit,
    /// Variational auto-encoder (latent <-> pixel space).
    Vae,
    /// Token / pixel decoder head.
    Decoder,
    /// Tokenizer.
    Tokenizer,
    /// Anything not covered above.
    Other,
}

/// An input or output contract of a component.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddIoContract {
    /// Contract name, e.g. `"prompt"`, `"latents"`.
    pub name: String,
    /// Data type of the tensor / value.
    pub dtype: MddDtype,
    /// Optional shape description, e.g. `"[1, 4096]"`, `"token_ids"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub shape: Option<String>,
}

/// Data type of an IO contract.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddDtype {
    /// 32-bit float.
    F32,
    /// 16-bit float.
    F16,
    /// bfloat16.
    Bf16,
    /// 8-bit signed integer.
    I8,
    /// 8-bit unsigned integer.
    U8,
    /// Token id sequence.
    TokenIds,
    /// Plain text.
    Text,
    /// Anything not covered above.
    Other,
}

// ═══════════════════════════════════════════════════════════════
// Runtimes — which engine serves a component
// ═══════════════════════════════════════════════════════════════

/// One concrete way to serve a component.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddRuntime {
    /// Serving engine.
    pub engine: MddEngine,
    /// Where the weights / entrypoint live.
    pub entry: MddRuntimeEntry,
    /// Engine features this runtime relies on (e.g. `["cuda", "flash-attn"]`).
    #[serde(default)]
    pub features: Vec<String>,
    /// Quantization variants this runtime supports.
    #[serde(default)]
    pub quantizations: Vec<MddQuantization>,
    /// Hardware requirements (local runtimes only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hardware: Option<MddHardwareRequirement>,
    /// Lifecycle status of this runtime.
    pub status: MddRuntimeStatus,
}

/// Serving engine for a runtime.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddEngine {
    /// llama.cpp (GGUF local inference).
    LlamaCpp,
    /// vLLM (GPU serving).
    Vllm,
    /// SGLang.
    Sglang,
    /// Candle (embedded Rust inference).
    Candle,
    /// Ollama.
    Ollama,
    /// Hosted cloud inference.
    Cloud,
    /// Generic external HTTP API.
    ExternalApi,
    /// Native / built-in runtime.
    Native,
}

/// Where a runtime's entrypoint (weights or endpoint) lives.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddRuntimeEntry {
    /// Entry kind.
    pub kind: MddEntryKind,
    /// Path / URL / registry reference, depending on `kind`.
    pub path: String,
    /// SHA-256 checksum of the artifact (file downloads only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sha256: Option<String>,
    /// Artifact size in bytes (file downloads only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size_bytes: Option<u64>,
}

/// Entry kind for a runtime.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddEntryKind {
    /// Local file path.
    File,
    /// Downloadable URL.
    Url,
    /// Model registry reference (Hugging Face style `owner/repo`).
    Registry,
    /// GGUF blob.
    Gguf,
}

/// A quantization variant of a runtime.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddQuantization {
    /// Quantization id, e.g. `"q4_k_m"`.
    pub id: String,
    /// Bit width per weight.
    pub bits: u32,
    /// Approximate size relative to the FP16 baseline (e.g. `0.25`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size_multiplier: Option<f64>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub notes: Option<String>,
}

/// Minimum hardware requirement for a local runtime.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddHardwareRequirement {
    /// Minimum VRAM in MB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_vram_mb: Option<u64>,
    /// Minimum system RAM in MB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_ram_mb: Option<u64>,
    /// Minimum GPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_gpus: Option<u32>,
    /// CUDA compute capability, e.g. `"8.0"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub compute_capability: Option<String>,
}

/// Lifecycle status of a runtime.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddRuntimeStatus {
    /// Deployed and serving.
    Ready,
    /// Known and planned, not yet deployed.
    Planned,
    /// Cannot be deployed in the current environment.
    Unavailable,
}

// ═══════════════════════════════════════════════════════════════
// Deploy — pipeline and API wiring
// ═══════════════════════════════════════════════════════════════

/// Deployment plan: pipeline stages plus the exposed inference API.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddDeploy {
    /// Ordered pipeline stages.
    pub pipeline: Vec<MddPipelineStage>,
    /// Inference API wiring.
    pub api: MddDeployApi,
}

/// One stage of the inference pipeline.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddPipelineStage {
    /// Unique stage id within the pipeline.
    pub id: String,
    /// When the stage runs.
    pub phase: MddPipelinePhase,
    /// Cache key template (deterministic stages only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cache_key: Option<String>,
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

/// When a pipeline stage runs.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddPipelinePhase {
    /// Before the core generation loop (prompt encoding, upscaling …).
    Pre,
    /// Inside the iterative denoising / generation loop.
    Iterative,
    /// After the loop (VAE decode, token detokenization …).
    Post,
}

/// Inference API exposed by the deployment.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddDeployApi {
    /// Task the API implements.
    pub task: MddTaskKind,
    /// Submit request schema.
    pub submit: MddApiSchema,
    /// Result response schema.
    pub result: MddApiSchema,
}

/// Task kind implemented by the inference API.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddTaskKind {
    /// Text generation / chat.
    TextGeneration,
    /// Embedding.
    Embedding,
    /// Image generation.
    ImageGeneration,
    /// Video generation.
    VideoGeneration,
    /// Audio generation.
    AudioGeneration,
    /// Anything not covered above.
    Other,
}

/// Named parameter schema for one API endpoint.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddApiSchema {
    /// Endpoint / schema name, e.g. `"submit"`.
    pub name: String,
    /// Accepted parameters.
    #[serde(default)]
    pub params: Vec<MddParam>,
}

/// A single parameter of an API schema.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddParam {
    /// Parameter name.
    pub name: String,
    /// Type descriptor, e.g. `"string"`, `"integer"`, `"float"`.
    pub dtype: String,
    /// Whether the parameter must be supplied.
    pub required: bool,
    /// Default value when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub default: Option<serde_json::Value>,
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Scale — sizing estimation and deploy tiers
// ═══════════════════════════════════════════════════════════════

/// Scale / sizing estimation for capacity planning.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddScale {
    /// Total parameter count in billions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub total_parameters_b: Option<f64>,
    /// Weight storage estimate.
    pub weights: MddScaleWeights,
    /// Supported quantization variants.
    #[serde(default)]
    pub quantization: Vec<MddQuantization>,
    /// Activation memory estimate.
    pub activation: MddScaleActivation,
    /// Context / KV-cache estimate.
    pub context: MddScaleContext,
    /// Throughput estimate.
    pub throughput: MddScaleThroughput,
    /// Deployment tiers (cheapest-first ordering is conventional).
    pub tiers: Vec<MddDeployTier>,
}

/// Weight storage estimate.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddScaleWeights {
    /// Total weight size in GB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size_gb: Option<f64>,
    /// Storage format, e.g. `"safetensors"`, `"gguf"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub format: Option<String>,
}

/// Activation memory estimate.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddScaleActivation {
    /// Peak activation memory in GB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub peak_gb: Option<f64>,
    /// Activation bytes per token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bytes_per_token: Option<f64>,
}

/// Context / KV-cache estimate.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddScaleContext {
    /// Maximum context length in tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_tokens: Option<u32>,
    /// KV-cache bytes per token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kv_cache_bytes_per_token: Option<f64>,
}

/// Throughput estimate.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddScaleThroughput {
    /// Tokens per second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tokens_per_second: Option<f64>,
    /// Requests per second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub requests_per_second: Option<f64>,
    /// End-to-end latency in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub latency_ms: Option<u32>,
}

/// One deployment tier — a hardware / placement strategy for the whole model.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
pub struct MddDeployTier {
    /// Tier id, e.g. `"cloud-api"`, `"48gb-native"`.
    pub id: String,
    /// Engines used by this tier.
    pub engines: Vec<MddEngine>,
    /// Quantization ids this tier is known to fit.
    #[serde(default)]
    pub quantizations: Vec<String>,
    /// Where the tier runs.
    pub placement: MddPlacement,
    /// Minimum total VRAM in GB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_vram_gb: Option<f64>,
    /// Minimum GPU count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_gpus: Option<u32>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub notes: Option<String>,
}

/// Where a deploy tier runs.
#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "mdd.ts")]
#[serde(rename_all = "snake_case")]
pub enum MddPlacement {
    /// Single GPU card.
    SingleCard,
    /// Multiple GPU cards on one or more hosts.
    MultiCard,
    /// Hosted cloud.
    Cloud,
    /// External service outside the deployment.
    External,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use serde_json::json;

    fn round_trip<T: Serialize + DeserializeOwned>(value: &T) -> T {
        let json = serde_json::to_string(value).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    fn assert_json_round_trip<T: Serialize + DeserializeOwned>(value: &T) {
        let original = serde_json::to_value(value).expect("serialize");
        let re: T = serde_json::from_value(original.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&re).expect("re-serialize"), original);
    }

    /// Minimal H3-style descriptor: encoder → dit → vae diffusion pipeline
    /// with a cloud tier (remote API) and a 48 GB-native multi-card tier.
    /// All ids, checksums, and endpoints are fake.
    fn h3_fixture() -> MddDescriptor {
        MddDescriptor {
            schema_version: 1,
            model: MddModel {
                id: "demo-h3-mini".into(),
                name: "Demo H3 Mini".into(),
                family: Some("demo-dit".into()),
                architecture: "dit".into(),
                description: Some("Fake H3-style diffusion transformer for tests.".into()),
            },
            components: vec![
                MddComponent {
                    id: "encoder".into(),
                    kind: MddComponentKind::Encoder,
                    arch: Some("transformer-encoder".into()),
                    dependencies: vec![],
                    inputs: vec![MddIoContract {
                        name: "prompt".into(),
                        dtype: MddDtype::Text,
                        shape: None,
                    }],
                    outputs: vec![MddIoContract {
                        name: "prompt_embeds".into(),
                        dtype: MddDtype::F32,
                        shape: Some("[1, 4096, 512]".into()),
                    }],
                    runtimes: vec![MddRuntime {
                        engine: MddEngine::LlamaCpp,
                        entry: MddRuntimeEntry {
                            kind: MddEntryKind::Gguf,
                            path: "fake-repo/demo-encoder-q4_k_m.gguf".into(),
                            sha256: Some(
                                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                                    .into(),
                            ),
                            size_bytes: Some(2_000_000_000),
                        },
                        features: vec!["cpu".into()],
                        quantizations: vec![MddQuantization {
                            id: "q4_k_m".into(),
                            bits: 4,
                            size_multiplier: Some(0.25),
                            notes: None,
                        }],
                        hardware: None,
                        status: MddRuntimeStatus::Ready,
                    }],
                },
                MddComponent {
                    id: "dit".into(),
                    kind: MddComponentKind::Dit,
                    arch: Some("dit-b2".into()),
                    dependencies: vec!["encoder".into()],
                    inputs: vec![
                        MddIoContract {
                            name: "prompt_embeds".into(),
                            dtype: MddDtype::F32,
                            shape: Some("[1, 4096, 512]".into()),
                        },
                        MddIoContract {
                            name: "noise_latents".into(),
                            dtype: MddDtype::F16,
                            shape: Some("[1, 16, 32, 32]".into()),
                        },
                    ],
                    outputs: vec![MddIoContract {
                        name: "denoised_latents".into(),
                        dtype: MddDtype::F16,
                        shape: Some("[1, 16, 32, 32]".into()),
                    }],
                    runtimes: vec![
                        MddRuntime {
                            engine: MddEngine::Vllm,
                            entry: MddRuntimeEntry {
                                kind: MddEntryKind::Registry,
                                path: "fake-org/demo-h3-mini-dit".into(),
                                sha256: None,
                                size_bytes: None,
                            },
                            features: vec!["cuda".into(), "flash-attn".into()],
                            quantizations: vec![
                                MddQuantization {
                                    id: "fp8".into(),
                                    bits: 8,
                                    size_multiplier: Some(0.5),
                                    notes: Some("fp8_e4m3".into()),
                                },
                                MddQuantization {
                                    id: "bf16".into(),
                                    bits: 16,
                                    size_multiplier: Some(1.0),
                                    notes: None,
                                },
                            ],
                            hardware: Some(MddHardwareRequirement {
                                min_vram_mb: Some(40_000),
                                min_ram_mb: Some(64_000),
                                min_gpus: Some(2),
                                compute_capability: Some("8.0".into()),
                            }),
                            status: MddRuntimeStatus::Planned,
                        },
                        MddRuntime {
                            engine: MddEngine::Cloud,
                            entry: MddRuntimeEntry {
                                kind: MddEntryKind::Url,
                                path: "https://api.example.invalid/v1/demo".into(),
                                sha256: None,
                                size_bytes: None,
                            },
                            features: vec![],
                            quantizations: vec![],
                            hardware: None,
                            status: MddRuntimeStatus::Ready,
                        },
                    ],
                },
                MddComponent {
                    id: "vae".into(),
                    kind: MddComponentKind::Vae,
                    arch: None,
                    dependencies: vec!["dit".into()],
                    inputs: vec![MddIoContract {
                        name: "denoised_latents".into(),
                        dtype: MddDtype::F16,
                        shape: Some("[1, 16, 32, 32]".into()),
                    }],
                    outputs: vec![MddIoContract {
                        name: "image".into(),
                        dtype: MddDtype::U8,
                        shape: Some("[1, 128, 128, 3]".into()),
                    }],
                    runtimes: vec![MddRuntime {
                        engine: MddEngine::Native,
                        entry: MddRuntimeEntry {
                            kind: MddEntryKind::Registry,
                            path: "fake-org/demo-h3-mini-vae".into(),
                            sha256: None,
                            size_bytes: None,
                        },
                        features: vec![],
                        quantizations: vec![],
                        hardware: Some(MddHardwareRequirement {
                            min_vram_mb: Some(4_000),
                            min_ram_mb: None,
                            min_gpus: Some(1),
                            compute_capability: None,
                        }),
                        status: MddRuntimeStatus::Ready,
                    }],
                },
            ],
            deploy: MddDeploy {
                pipeline: vec![
                    MddPipelineStage {
                        id: "encode".into(),
                        phase: MddPipelinePhase::Pre,
                        cache_key: Some("encoder-v1".into()),
                        description: Some("Prompt encoding".into()),
                    },
                    MddPipelineStage {
                        id: "denoise".into(),
                        phase: MddPipelinePhase::Iterative,
                        cache_key: None,
                        description: Some("Denoising loop".into()),
                    },
                    MddPipelineStage {
                        id: "decode".into(),
                        phase: MddPipelinePhase::Post,
                        cache_key: Some("vae-v1".into()),
                        description: None,
                    },
                ],
                api: MddDeployApi {
                    task: MddTaskKind::ImageGeneration,
                    submit: MddApiSchema {
                        name: "submit".into(),
                        params: vec![
                            MddParam {
                                name: "prompt".into(),
                                dtype: "string".into(),
                                required: true,
                                default: None,
                                description: Some("Text prompt".into()),
                            },
                            MddParam {
                                name: "steps".into(),
                                dtype: "integer".into(),
                                required: false,
                                default: Some(json!(28)),
                                description: None,
                            },
                            MddParam {
                                name: "guidance".into(),
                                dtype: "float".into(),
                                required: false,
                                default: Some(json!(3.5)),
                                description: Some("Guidance scale".into()),
                            },
                        ],
                    },
                    result: MddApiSchema {
                        name: "result".into(),
                        params: vec![
                            MddParam {
                                name: "image_b64".into(),
                                dtype: "string".into(),
                                required: true,
                                default: None,
                                description: Some("Base64 PNG output".into()),
                            },
                            MddParam {
                                name: "seed".into(),
                                dtype: "integer".into(),
                                required: false,
                                default: Some(json!(-1)),
                                description: Some("Random seed".into()),
                            },
                        ],
                    },
                },
            },
            scale: MddScale {
                total_parameters_b: Some(13.5),
                weights: MddScaleWeights {
                    size_gb: Some(26.0),
                    format: Some("safetensors".into()),
                },
                quantization: vec![
                    MddQuantization {
                        id: "bf16".into(),
                        bits: 16,
                        size_multiplier: Some(1.0),
                        notes: None,
                    },
                    MddQuantization {
                        id: "fp8".into(),
                        bits: 8,
                        size_multiplier: Some(0.5),
                        notes: Some("fp8_e4m3".into()),
                    },
                ],
                activation: MddScaleActivation {
                    peak_gb: Some(4.0),
                    bytes_per_token: Some(0.5),
                },
                context: MddScaleContext {
                    max_tokens: Some(64_000),
                    kv_cache_bytes_per_token: Some(1.25),
                },
                throughput: MddScaleThroughput {
                    tokens_per_second: Some(96.0),
                    requests_per_second: Some(2.0),
                    latency_ms: Some(450),
                },
                tiers: vec![
                    MddDeployTier {
                        id: "cloud-api".into(),
                        engines: vec![MddEngine::Cloud, MddEngine::ExternalApi],
                        quantizations: vec!["bf16".into()],
                        placement: MddPlacement::Cloud,
                        min_vram_gb: None,
                        min_gpus: None,
                        notes: Some("Fake hosted endpoint; no local hardware.".into()),
                    },
                    MddDeployTier {
                        id: "48gb-native".into(),
                        engines: vec![MddEngine::Vllm],
                        quantizations: vec!["bf16".into(), "fp8".into()],
                        placement: MddPlacement::MultiCard,
                        min_vram_gb: Some(48.0),
                        min_gpus: Some(2),
                        notes: Some("2x24 GB cards; fp8 native.".into()),
                    },
                ],
            },
        }
    }

    // ── Serde round-trip ─────────────────────────────────────────

    #[test]
    fn descriptor_serde_round_trip() {
        let descriptor = h3_fixture();
        assert_json_round_trip(&descriptor);
    }

    #[test]
    fn enums_serde_round_trip() {
        // Typed round-trips through the enum types themselves.
        assert_enum_round_trip(MddComponentKind::Encoder);
        assert_enum_round_trip(MddComponentKind::Dit);
        assert_enum_round_trip(MddDtype::Bf16);
        assert_enum_round_trip(MddEngine::LlamaCpp);
        assert_enum_round_trip(MddEngine::ExternalApi);
        assert_enum_round_trip(MddEntryKind::Gguf);
        assert_enum_round_trip(MddRuntimeStatus::Planned);
        assert_enum_round_trip(MddPipelinePhase::Iterative);
        assert_enum_round_trip(MddTaskKind::ImageGeneration);
        assert_enum_round_trip(MddPlacement::MultiCard);
        // snake_case wire form.
        assert_eq!(
            serde_json::to_value(MddEngine::LlamaCpp).unwrap(),
            json!("llama_cpp")
        );
        assert_eq!(
            serde_json::to_value(MddDtype::TokenIds).unwrap(),
            json!("token_ids")
        );
        assert_eq!(
            serde_json::to_value(MddPlacement::SingleCard).unwrap(),
            json!("single_card")
        );
    }

    fn assert_enum_round_trip<T: Serialize + DeserializeOwned>(value: T) {
        let json = serde_json::to_value(&value).expect("serialize enum");
        let back: T = serde_json::from_value(json.clone()).expect("deserialize enum");
        assert_eq!(
            serde_json::to_value(&back).expect("re-serialize enum"),
            json
        );
    }

    // ── Fixture semantics ─────────────────────────────────────────

    #[test]
    fn h3_fixture_deserializes_and_keeps_key_fields() {
        let descriptor = h3_fixture();
        let json = serde_json::to_string(&descriptor).unwrap();
        let parsed: MddDescriptor = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.model.id, "demo-h3-mini");
        assert_eq!(parsed.model.architecture, "dit");

        // Three components forming the encoder → dit → vae DAG.
        assert_eq!(parsed.components.len(), 3);
        let kinds: Vec<_> = parsed
            .components
            .iter()
            .map(|c| serde_json::to_value(&c.kind).unwrap())
            .collect();
        assert_eq!(kinds, vec![json!("encoder"), json!("dit"), json!("vae")]);
        assert!(parsed.components[1]
            .dependencies
            .contains(&"encoder".to_string()));
        assert!(parsed.components[2]
            .dependencies
            .contains(&"dit".to_string()));

        // Two tiers: cloud (no local hardware) + 48 GB-native multi-card.
        assert_eq!(parsed.scale.tiers.len(), 2);
        assert_eq!(parsed.scale.tiers[0].id, "cloud-api");
        assert_eq!(
            serde_json::to_value(&parsed.scale.tiers[0].placement).unwrap(),
            json!("cloud")
        );
        assert_eq!(parsed.scale.tiers[1].id, "48gb-native");
        assert_eq!(
            serde_json::to_value(&parsed.scale.tiers[1].placement).unwrap(),
            json!("multi_card")
        );
        assert_eq!(parsed.scale.tiers[1].min_vram_gb, Some(48.0));
        assert_eq!(parsed.scale.tiers[1].min_gpus, Some(2));

        // 48 GB-native semantics: local vLLM runtime is Planned, cloud is Ready.
        let dit_runtimes = &parsed.components[1].runtimes;
        assert_eq!(dit_runtimes.len(), 2);
        assert_eq!(
            serde_json::to_value(&dit_runtimes[0].status).unwrap(),
            json!("planned")
        );
        assert_eq!(
            dit_runtimes[0].hardware.as_ref().unwrap().min_vram_mb,
            Some(40_000)
        );
        assert_eq!(
            serde_json::to_value(&dit_runtimes[1].status).unwrap(),
            json!("ready")
        );

        // API wiring kept intact.
        assert_eq!(
            serde_json::to_value(&parsed.deploy.api.task).unwrap(),
            json!("image_generation")
        );
        assert_eq!(parsed.deploy.api.submit.params.len(), 3);
        assert_eq!(parsed.deploy.api.submit.params[1].default, Some(json!(28)));
    }

    #[test]
    fn unavailability_semantics_survive_round_trip() {
        let mut descriptor = h3_fixture();
        descriptor.components[1].runtimes[0].status = MddRuntimeStatus::Unavailable;
        let re = round_trip(&descriptor);
        assert_eq!(
            serde_json::to_value(&re.components[1].runtimes[0].status).unwrap(),
            json!("unavailable")
        );
    }

    #[test]
    fn optional_fields_are_omitted_from_json() {
        let descriptor = h3_fixture();
        let json = serde_json::to_value(&descriptor).unwrap();
        let encoder = &json["components"][0];
        assert!(encoder.get("arch").is_some(), "Some fields serialize");
        assert!(
            encoder["runtimes"][0].get("hardware").is_none(),
            "None hardware must be skipped"
        );
        assert!(
            json["scale"]["tiers"][0].get("min_vram_gb").is_none(),
            "None min_vram_gb must be skipped"
        );
    }
}
