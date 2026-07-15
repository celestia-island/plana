use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tracing::{info, warn};

// ─── Embedding model index ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelEntry {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub dimension: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_repo: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingModelIndex {
    pub models: Vec<EmbeddingModelEntry>,
}

impl EmbeddingModelIndex {
    pub fn load() -> Self {
        let path = Self::index_path();
        if !path.exists() {
            return Self::builtin();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                warn!(error = %e, path = %path.display(), "failed to parse embedding model index");
                Self::builtin()
            }),
            Err(_) => Self::builtin(),
        }
    }

    pub fn find(&self, model_id: &str) -> Option<&EmbeddingModelEntry> {
        self.models.iter().find(|m| m.id == model_id)
    }

    pub fn find_by_provider(&self, provider_id: &str) -> Vec<&EmbeddingModelEntry> {
        self.models
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .collect()
    }

    pub fn is_api_model(&self, model_id: &str) -> bool {
        self.find(model_id).is_some_and(|m| m.api_path.is_some())
    }

    fn index_path() -> PathBuf {
        super::UserConfig::config_dir().join("embedding_models.toml")
    }

    fn builtin() -> Self {
        Self { models: vec![] }
    }
}

// ─── Mirror table ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorEntry {
    pub namespace: String,
    pub source: String,
    #[serde(default)]
    pub mirrors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MirrorTable {
    pub mirrors: Vec<MirrorEntry>,
}

impl MirrorTable {
    pub fn load() -> Self {
        let path = Self::table_path();
        if !path.exists() {
            return Self::builtin();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                warn!(error = %e, path = %path.display(), "failed to parse mirror table");
                Self::builtin()
            }),
            Err(_) => Self::builtin(),
        }
    }

    pub fn resolve(&self, namespace: &str, prefer_mirror: bool) -> String {
        let entry = self.mirrors.iter().find(|m| m.namespace == namespace);
        match entry {
            Some(e) if prefer_mirror && !e.mirrors.is_empty() => e.mirrors[0].clone(),
            Some(e) => e.source.clone(),
            None => namespace.to_string(),
        }
    }

    pub fn try_mirrors(&self, namespace: &str) -> Vec<String> {
        let entry = self.mirrors.iter().find(|m| m.namespace == namespace);
        match entry {
            Some(e) => {
                let mut all = vec![e.source.clone()];
                all.extend(e.mirrors.iter().cloned());
                all
            }
            None => vec![namespace.to_string()],
        }
    }

    fn table_path() -> PathBuf {
        super::UserConfig::config_dir().join("mirrors.toml")
    }

    fn builtin() -> Self {
        Self {
            mirrors: vec![
                MirrorEntry {
                    namespace: "huggingface.co".into(),
                    source: "huggingface.co".into(),
                    mirrors: vec!["hf-mirror.com".into()],
                },
                MirrorEntry {
                    namespace: "github.com".into(),
                    source: "github.com".into(),
                    mirrors: vec!["gh-proxy.com/https://github.com".into()],
                },
                MirrorEntry {
                    namespace: "docker.io".into(),
                    source: "docker.io".into(),
                    mirrors: vec![],
                },
            ],
        }
    }
}

// ─── Hardware detection result ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareProfile {
    pub has_nvidia_gpu: bool,
    pub has_amd_gpu: bool,
    pub has_intel_gpu: bool,
    pub gpu_name: Option<String>,
    pub gpu_memory_mb: Option<u64>,
    pub onnx_ep: OnnxEp,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum OnnxEp {
    #[default]
    Cpu,
    Cuda,
    Rocm,
    DirectMl,
    OpenVINO,
    CoreMl,
}

impl std::fmt::Display for OnnxEp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnnxEp::Cpu => write!(f, "cpu"),
            OnnxEp::Cuda => write!(f, "cuda"),
            OnnxEp::Rocm => write!(f, "rocm"),
            OnnxEp::DirectMl => write!(f, "directml"),
            OnnxEp::OpenVINO => write!(f, "openvino"),
            OnnxEp::CoreMl => write!(f, "coreml"),
        }
    }
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let has_nvidia = std::process::Command::new("nvidia-smi")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let has_amd = std::process::Command::new("rocm-smi")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let gpu_name = if has_nvidia {
            std::process::Command::new("nvidia-smi")
                .args(["--query-gpu=name", "--format=csv,noheader,nounits"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        } else if has_amd {
            std::process::Command::new("rocm-smi")
                .args(["--showproductname", "--json"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| {
                    serde_json::from_str::<serde_json::Value>(&s)
                        .ok()
                        .and_then(|v| {
                            v.get("card0")
                                .and_then(|c| c.get("Card series"))
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                })
        } else {
            None
        };

        let onnx_ep = if has_nvidia {
            OnnxEp::Cuda
        } else if has_amd {
            OnnxEp::Rocm
        } else {
            OnnxEp::Cpu
        };

        let profile = Self {
            has_nvidia_gpu: has_nvidia,
            has_amd_gpu: has_amd,
            has_intel_gpu: false,
            gpu_name,
            gpu_memory_mb: None,
            onnx_ep,
        };

        info!(
            gpu = ?profile.gpu_name,
            onnx_ep = %profile.onnx_ep,
            "hardware detection complete"
        );

        profile
    }
}
