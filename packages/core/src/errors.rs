use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use thiserror::Error;

yuuka::derive_enum!(
    #[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
    /// Wire taxonomy for structured agent errors. This enum is a serialization
    /// twin of `plana_celestia_types::AgentErrorCode` (same variants, same
    /// order, same snake_case wire form) — keep both copies in lockstep when
    /// adding variants (the state-sync census is the guard). Declared through
    /// the yuuka `derive_enum!` house form, mirroring the celestia-types twin;
    /// doc comments sit inside the invocation because rustdoc does not attach
    /// docs through a macro call from the outside.
    #[serde(rename_all = "snake_case")]
    pub enum AgentErrorCode {
        ModelNoProviders,
        ModelNoModels,
        ModelTierMismatch,
        ModelAllExcluded,
        ModelEnvIncomplete,
        ModelSelectionRetryExhausted,
        ModelNoCapableModel,
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
        /// An infrastructure dependency (IPC peer, upstream service, database) is
        /// unreachable or unhealthy. See [`StructuredAgentError`]'s `context`
        /// field for the canonical keys carried alongside this code.
        DependencyDown,
        /// The request was refused because the engine is already processing
        /// another user message (single-flight guard), not because of a fault.
        AgentBusy,
        /// The request was refused because the task's budget (steps, tokens or
        /// wall-clock allowance) is exhausted, not because of a fault.
        BudgetExhausted,
    }
);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredAgentError {
    pub code: AgentErrorCode,
    #[serde(default)]
    pub detail: Option<String>,
    /// Free-form context bag; absent on the wire deserializes as an empty
    /// map (serde default), so payloads written by newer producers stay
    /// readable by older consumers and vice versa.
    ///
    /// Identity-bearing values are closed vocabularies declared in
    /// `plana_celestia_types` (re-exported for the engine through
    /// state-sync): `reason` carries the wire form of `AgentErrorReason`
    /// and `interrupted` the wire form of `AgentInterruptReason`. The
    /// string-map keys:
    /// - `dependency`: stable identifier of the failed dependency
    ///   (e.g. `evernight-ipc`, `arona`);
    /// - `reason`: machine-readable reason label (e.g. `peer_unreachable`);
    /// - `interrupted`: why an in-flight chain was interrupted (`restart`);
    /// - `hint`: optional operator-facing remediation hint (free prose).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The core copy of `AgentErrorCode` must accept the same wire values as
    /// the celestia-types twin, including the new capability-gated code.
    #[test]
    fn agent_error_code_no_capable_model_round_trips_snake_case() {
        let wire = serde_json::to_string(&AgentErrorCode::ModelNoCapableModel).unwrap();
        assert_eq!(wire, r#""model_no_capable_model""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::ModelNoCapableModel);
    }

    /// The wire string of the dependency-failure code is a cross-service
    /// contract (chest-side error routing reads it verbatim).
    #[test]
    fn dependency_down_round_trips_snake_case() {
        let wire = serde_json::to_string(&AgentErrorCode::DependencyDown).unwrap();
        assert_eq!(wire, r#""dependency_down""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::DependencyDown);
    }

    /// The wire strings of the two engine-flow codes are cross-service
    /// contracts (chest-side error routing reads them verbatim).
    #[test]
    fn engine_flow_codes_round_trip_snake_case() {
        let wire = serde_json::to_string(&AgentErrorCode::AgentBusy).unwrap();
        assert_eq!(wire, r#""agent_busy""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::AgentBusy);

        let wire = serde_json::to_string(&AgentErrorCode::BudgetExhausted).unwrap();
        assert_eq!(wire, r#""budget_exhausted""#);
        let back: AgentErrorCode = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, AgentErrorCode::BudgetExhausted);
    }

    /// Backward-compatibility tolerance for the `context` field, in both
    /// directions: a payload that carries `context` must stay readable by a
    /// consumer whose struct predates the field (unknown fields are ignored),
    /// and the serde-default field must materialize as empty for payloads
    /// that omit it.
    #[test]
    fn structured_agent_error_tolerates_context_field_in_both_directions() {
        // Old-consumer struct shape: `code` + optional `detail`, no `context`.
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct LegacyStructuredAgentError {
            code: AgentErrorCode,
            #[serde(default)]
            detail: Option<String>,
        }

        let payload = serde_json::json!({
            "code": "dependency_down",
            "detail": "evernight ipc bridge is unreachable",
            "context": {
                "dependency": "evernight-ipc",
                "reason": "peer_unreachable",
                "hint": "check the evernight-ipc service on the host",
            },
        });

        let legacy: LegacyStructuredAgentError = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(legacy.code, AgentErrorCode::DependencyDown);
        assert_eq!(
            legacy.detail.as_deref(),
            Some("evernight ipc bridge is unreachable")
        );

        let full: StructuredAgentError = serde_json::from_value(payload).unwrap();
        assert_eq!(full.code, AgentErrorCode::DependencyDown);
        assert_eq!(
            full.context.get("dependency").map(String::as_str),
            Some("evernight-ipc")
        );
        assert_eq!(
            full.context.get("reason").map(String::as_str),
            Some("peer_unreachable")
        );
        assert_eq!(
            full.context.get("hint").map(String::as_str),
            Some("check the evernight-ipc service on the host")
        );

        // A payload without `context` deserializes with the default empty bag.
        let bare: StructuredAgentError =
            serde_json::from_value(serde_json::json!({"code": "chain_failed"})).unwrap();
        assert_eq!(bare.code, AgentErrorCode::ChainFailed);
        assert_eq!(bare.detail, None);
        assert!(bare.context.is_empty());
    }

    #[test]
    fn dependency_down_full_round_trip_preserves_context() {
        let err = StructuredAgentError::new(AgentErrorCode::DependencyDown)
            .with_detail("infrastructure dependency is unreachable")
            .with_context("dependency", "arona")
            .with_context("reason", "http_probe_failed")
            .with_context("hint", "restart the arona container");
        let wire = serde_json::to_string(&err).unwrap();
        let back: StructuredAgentError = serde_json::from_str(&wire).unwrap();
        assert_eq!(back, err);
        // The wire form carries the canonical keys verbatim.
        let v = serde_json::to_value(&err).unwrap();
        assert_eq!(v["code"], "dependency_down");
        assert_eq!(v["context"]["dependency"], "arona");
        assert_eq!(v["context"]["reason"], "http_probe_failed");
        assert_eq!(v["context"]["hint"], "restart the arona container");
    }
}
