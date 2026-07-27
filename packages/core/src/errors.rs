use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorCode {
    ModelNoProviders,
    ModelNoModels,
    ModelTierMismatch,
    ModelAllExcluded,
    ModelEnvIncomplete,
    ModelSelectionRetryExhausted,
    LlmCallFailed,
    LlmEmptyResponse,
    LlmRateLimited,
    LlmAuthFailed,
    LlmTimeout,
    CosmosNoConnection,
    CosmosToolFailed,
    CosmosLocalUnavailable,
    ChainMaxDepth,
    ChainCycle,
    ChainFailed,
    SkillFailed,
    SkillEmptyOutput,
    SkillMissingReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredAgentError {
    pub code: AgentErrorCode,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub context: HashMap<String, String>,
}

impl StructuredAgentError {
    pub fn new(code: AgentErrorCode) -> Self {
        Self {
            code,
            detail: None,
            context: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn debug_message(&self) -> String {
        let detail = self.detail.as_deref().unwrap_or("");
        if self.context.is_empty() {
            format!("{:?}: {}", self.code, detail)
        } else {
            let ctx: Vec<String> = self
                .context
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("{:?}: {} [{}]", self.code, detail, ctx.join(", "))
        }
    }
}

impl std::fmt::Display for StructuredAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_message())
    }
}

impl std::error::Error for StructuredAgentError {}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("invalid credential type: {0}")]
    InvalidType(String),
}
