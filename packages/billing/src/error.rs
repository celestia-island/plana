//! Error vocabulary of the wallet contract.
//!
//! Codes returned by [`WalletError::rpc_code`] align with the chest billing
//! plane's reserved JSON-RPC band `-32060..=-32079` (chest assigns
//! `-32060` = quota-exceeded / insufficient and `-32002` = forbidden today;
//! the `-32062`/`-32063` assignments below were made by this crate, which is
//! the single home of the wire protocol until the ERP side implements it).

use serde::{Deserialize, Serialize};

/// Everything that can go wrong across the [`crate::Wallet`] surface.
///
/// Serialized with adjacent tagging (`{"kind": ..., "data": ...}`) so the
/// string-carrying `Internal` variant stays a plain newtype.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WalletError {
    /// The wallet cannot cover the request. `have` is the current available
    /// balance, `need` the requested amount; both are in the wallet's unit.
    /// JSON-RPC `-32060`, aligned with chest's `BILLING_QUOTA_EXCEEDED`.
    Insufficient {
        /// Available balance at request time.
        have: i64,
        /// Requested amount.
        need: i64,
    },
    /// A reservation with the same `(subject, ref_id)` already exists —
    /// reserve is idempotent per ref id, so a duplicate is a caller bug.
    /// JSON-RPC `-32062` (assigned by this contract).
    AlreadyReserved {
        /// The conflicting reservation ref id.
        ref_id: String,
    },
    /// The subject, reservation ref or resource addressed by the call does
    /// not exist. JSON-RPC `-32063` (assigned by this contract).
    NotFound,
    /// The caller is not allowed to operate this wallet.
    /// JSON-RPC `-32002`, aligned with chest's `JSONRPC_FORBIDDEN`.
    Forbidden,
    /// Chest's wallet authority has migrated to ERP (切刀专用): the caller
    /// must stop using chest `billing.task.*` and route through the ERP
    /// bridge. JSON-RPC `-32061` (wallet-authority-moved lane).
    AuthorityMoved,
    /// The bridge/service is unreachable (timeout, connect failure, 5xx,
    /// unparsable answer). Callers degrade per their existing policy.
    /// JSON-RPC `-32000` (server-defined range base).
    Unavailable,
    /// Implementation-side fault (storage failure, malformed response
    /// payload, contract-violating argument). JSON-RPC `-32603`, the
    /// JSON-RPC standard internal-error code.
    Internal(String),
}

impl WalletError {
    /// JSON-RPC error code carried on the wire for this error, per the
    /// mapping documented on each variant.
    pub fn rpc_code(&self) -> i64 {
        match self {
            WalletError::Insufficient { .. } => -32060,
            WalletError::AlreadyReserved { .. } => -32062,
            WalletError::NotFound => -32063,
            WalletError::Forbidden => -32002,
            WalletError::AuthorityMoved => -32061,
            WalletError::Unavailable => -32000,
            WalletError::Internal(_) => -32603,
        }
    }
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::Insufficient { have, need } => {
                write!(f, "insufficient balance: have {have}, need {need}")
            }
            WalletError::AlreadyReserved { ref_id } => {
                write!(f, "already reserved for ref id `{ref_id}`")
            }
            WalletError::NotFound => write!(f, "wallet subject or resource not found"),
            WalletError::Forbidden => write!(f, "operation on this wallet is forbidden"),
            WalletError::AuthorityMoved => {
                write!(
                    f,
                    "wallet authority has moved to ERP; chest billing.task.* is retired"
                )
            }
            WalletError::Unavailable => write!(f, "wallet service unavailable"),
            WalletError::Internal(msg) => write!(f, "internal wallet error: {msg}"),
        }
    }
}

impl std::error::Error for WalletError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_codes_match_the_contract() {
        assert_eq!(
            WalletError::Insufficient { have: 5, need: 10 }.rpc_code(),
            -32060
        );
        assert_eq!(
            WalletError::AlreadyReserved { ref_id: "t".into() }.rpc_code(),
            -32062
        );
        assert_eq!(WalletError::NotFound.rpc_code(), -32063);
        assert_eq!(WalletError::Forbidden.rpc_code(), -32002);
        assert_eq!(WalletError::AuthorityMoved.rpc_code(), -32061);
        assert_eq!(WalletError::Unavailable.rpc_code(), -32000);
        assert_eq!(WalletError::Internal("x".into()).rpc_code(), -32603);
    }

    /// The serde form is the bridge's error wire shape
    /// (`bridge::RawError` re-encodes from it); keep it tagged and stable.
    #[test]
    fn serde_form_is_tagged_and_stable() {
        let json = serde_json::to_value(WalletError::Insufficient { have: 5, need: 10 }).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"kind": "insufficient", "data": {"have": 5, "need": 10}})
        );
        let back: WalletError = serde_json::from_value(json).unwrap();
        assert_eq!(back, WalletError::Insufficient { have: 5, need: 10 });
        // The newtype-with-string variant must serialize too (this is why
        // the enum uses adjacent, not internal, tagging).
        let internal = serde_json::to_value(WalletError::Internal("boom".into())).unwrap();
        assert_eq!(
            internal,
            serde_json::json!({"kind": "internal", "data": "boom"})
        );
        let unit = serde_json::to_value(WalletError::AuthorityMoved).unwrap();
        assert_eq!(unit, serde_json::json!({"kind": "authority_moved"}));
    }

    #[test]
    fn display_mentions_the_numbers_for_insufficient() {
        let e = WalletError::Insufficient { have: 3, need: 7 };
        assert_eq!(e.to_string(), "insufficient balance: have 3, need 7");
    }
}
