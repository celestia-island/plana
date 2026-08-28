//! Target-scope guard and endpoint configuration (spec §3.1 / §4.2).
//!
//! The guard enforces the single-node principle: a dispatch may only target
//! the one designated node whose logical alias was resolved from deployment
//! configuration. Anything else — a foreign alias or (once the enum grows)
//! any non-single-node scope — is rejected locally with the auth error code
//! and the structured `target_out_of_scope` reason, before any transport
//! call and without emitting broker traffic (spec §4.2).

use plana_jsonrpc::error_codes;
use serde::{Deserialize, Serialize};

use crate::classify::{ClassifiedError, ErrorKind};
use crate::envelope::TargetScope;

/// Broker endpoint used by the client configuration.
///
/// The contract's baseline form is the same-host Unix socket IPC; the remote
/// TCP form is reserved for the deferred wss transport (spec §5.1). Doc
/// examples use RFC 5737 documentation addresses only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endpoint {
    /// Filesystem path of the broker's Unix domain socket.
    Ipc(String),
    /// Remote host:port pair (unused until the deferred ws transport lands).
    Tcp {
        /// Broker host (documentation examples: `192.0.2.x` only).
        host: String,
        /// Broker port.
        port: u16,
    },
}

/// Validates [`TargetScope`]s against the one designated node alias.
#[derive(Debug, Clone)]
pub struct TargetScopeGuard {
    designated_alias: String,
}

impl TargetScopeGuard {
    /// Creates a guard for the designated node alias resolved from the
    /// deployment configuration (config key
    /// `terminal_route.target_node_alias`; never a free-text address).
    pub fn new(designated_alias: impl Into<String>) -> Self {
        Self {
            designated_alias: designated_alias.into(),
        }
    }

    /// The configured designated alias.
    pub fn designated_alias(&self) -> &str {
        &self.designated_alias
    }

    /// Validates a dispatch target scope.
    ///
    /// Returns `Ok(())` only for the single-node scope whose alias equals
    /// the designated alias. Any other scope is rejected with
    /// [`ErrorKind::TargetOutOfScope`], code `-32005` (`AUTH_ERROR`) and
    /// `data.reason = "target_out_of_scope"` (spec §4.2).
    pub fn validate(&self, scope: &TargetScope) -> Result<(), ClassifiedError> {
        let matches = match scope {
            TargetScope::DesignatedSingleNode { alias } => *alias == self.designated_alias,
        };
        if matches {
            return Ok(());
        }
        Err(ClassifiedError::new(
            ErrorKind::TargetOutOfScope,
            error_codes::AUTH_ERROR,
            format!(
                "target scope rejected: requested node is not the designated single node {:?}",
                self.designated_alias
            ),
        )
        .with_reason("target_out_of_scope"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn designated_scope() -> TargetScope {
        TargetScope::DesignatedSingleNode {
            alias: "hydro-lab".to_string(),
        }
    }

    #[test]
    fn designated_alias_is_accepted() {
        let guard = TargetScopeGuard::new("hydro-lab");
        assert_eq!(guard.designated_alias(), "hydro-lab");
        assert!(guard.validate(&designated_scope()).is_ok());
    }

    #[test]
    fn foreign_alias_is_rejected_with_target_out_of_scope() {
        let guard = TargetScopeGuard::new("hydro-lab");
        let foreign = TargetScope::DesignatedSingleNode {
            alias: "other-node".to_string(),
        };
        let error = guard.validate(&foreign).expect_err("must be rejected");
        assert_eq!(error.kind, ErrorKind::TargetOutOfScope);
        assert_eq!(error.code, error_codes::AUTH_ERROR);
        assert_eq!(error.code, -32005);
        assert_eq!(error.data, Some(json!({"reason": "target_out_of_scope"})));
        assert!(error.message.contains("hydro-lab"));
    }

    #[test]
    fn alias_comparison_is_exact_equality() {
        // The guard compares the scope alias to the configured alias with
        // exact string equality — no prefix, no case folding, no emptiness
        // shortcuts in either direction.
        let guard = TargetScopeGuard::new("hydro-lab");
        let scope = TargetScope::DesignatedSingleNode {
            alias: "hydro-lab-ext".to_string(),
        };
        assert!(guard.validate(&scope).is_err());
        let case_variant = TargetScope::DesignatedSingleNode {
            alias: "Hydro-Lab".to_string(),
        };
        assert!(guard.validate(&case_variant).is_err());
        let empty_config = TargetScopeGuard::new("");
        let scope = TargetScope::DesignatedSingleNode {
            alias: String::new(),
        };
        assert!(empty_config.validate(&scope).is_ok());
        let scope = TargetScope::DesignatedSingleNode {
            alias: "x".to_string(),
        };
        assert!(empty_config.validate(&scope).is_err());
    }

    #[test]
    fn endpoint_serializes_ipc_and_tcp() {
        // Externally tagged enum representation (serde default).
        let ipc = Endpoint::Ipc("/var/run/evernight/broker.sock".to_string());
        assert_eq!(
            serde_json::to_value(&ipc).unwrap(),
            json!({"Ipc": "/var/run/evernight/broker.sock"})
        );
        let tcp = Endpoint::Tcp {
            host: "192.0.2.10".to_string(),
            port: 7777,
        };
        let value = serde_json::to_value(&tcp).unwrap();
        assert_eq!(value["Tcp"]["host"], "192.0.2.10");
        assert_eq!(value["Tcp"]["port"], 7777);
        let back: Endpoint = serde_json::from_value(value).unwrap();
        assert_eq!(back, tcp);
    }
}
