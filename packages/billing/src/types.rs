//! Wire types of the wallet contract.
//!
//! Every type derives `Serialize`/`Deserialize`: the same structs are the
//! in-process contract (through [`crate::Wallet`]) and the line format of the
//! optional ERP bridge ([`crate::bridge`], feature `client`). Enum variants
//! serialize as `snake_case` (`User` → `"user"`), matching the JSON-RPC
//! conventions of the chest billing plane these semantics were carved from.
//!
//! [`LedgerEntry::entry_type`] is deliberately an open `String`, not an
//! enum: the chest ledger's factual vocabulary is
//! `grant / topup / consume / expire / adjust / reserve / refund`, and a
//! closed Rust enum here would freeze the vocabulary against later
//! extensions on the ERP side. Consumers match on known words and must
//! tolerate unknown ones.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Who owns a wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerKind {
    /// A personal account.
    User,
    /// A shared group wallet (members escrow against it, serialized by the
    /// owner-side lock).
    Group,
    /// An organization wallet.
    Org,
}

/// The subject a wallet operation is performed for.
///
/// Identity is `(owner, id)`; `ext_key` is the external bridging key
/// (conventionally the e-mail address) that lets the ERP side correlate a
/// subject it does not store a UUID for. Two subjects with the same
/// `(owner, id)` but different `ext_key` denote the same wallet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletSubject {
    /// Which kind of owner holds the wallet.
    pub owner: OwnerKind,
    /// Stable UUID of the owner (chest account id, ERP-side correlation key).
    pub id: Uuid,
    /// Optional e-mail bridging key (ERP ↔ chest convention).
    pub ext_key: Option<String>,
}

/// Accounting unit of a wallet balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// Degrees (chest 计费单位，度数).
    Degrees,
    /// Points (arona points-first 语义的点数).
    Points,
    /// Chinese yuan, integer minor-unit-free whole yuan.
    Cny,
}

/// A point-in-time wallet balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletBalance {
    /// Available (unreserved, unsettled) amount in `unit`.
    pub available: i64,
    /// Accounting unit the amount is denominated in.
    pub unit: Unit,
}

/// A reservation request: hold `amount` against the caller-chosen `ref_id`
/// (conventionally the task id) until refunded or settled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveOp {
    /// Caller-chosen correlation id; idempotency is per `(subject, ref_id)`.
    pub ref_id: String,
    /// Strictly positive amount to hold.
    pub amount: i64,
}

/// Read-only budget snapshot of one reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBudget {
    /// Total amount successfully reserved under the ref id.
    pub reserved: i64,
    /// Total amount released back (refunds + settled consumption).
    pub refunded: i64,
    /// `reserved - refunded`: what is still held for the task.
    pub net: i64,
    /// `true` while `net` is still positive (the task holds funds).
    pub open: bool,
}

/// How a settlement landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementOutcome {
    /// The full requested amount was deducted.
    Covered,
    /// The balance ran out mid-settlement: deducted only up to zero
    /// (arona points-first clamp semantics — a wallet never goes negative).
    Clamped,
}

/// Result of [`crate::Wallet::settle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    /// Whether the full amount was covered or clamped to the balance.
    pub outcome: SettlementOutcome,
    /// Amount actually deducted (`min(amount, available)`).
    pub deducted: i64,
}

/// Why a credit entered the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditReason {
    /// Recharge (paid top-up).
    Topup,
    /// Administrative or promotional grant.
    Grant,
    /// Manual correction.
    Adjust,
    /// Expiry write-off.
    Expire,
}

/// Ledger pagination request. `page` is 1-based; a `page` of `0` is treated
/// as `1`, and a `page_size` of `0` yields an empty page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryQuery {
    /// 1-based page number.
    pub page: u32,
    /// Page size (number of entries per page).
    pub page_size: u32,
}

impl Default for EntryQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 50,
        }
    }
}

/// One page of ledger entries, newest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryPage {
    /// The requested slice of entries (newest first).
    pub entries: Vec<LedgerEntry>,
    /// Total number of entries across all pages.
    pub total: u64,
}

/// A single ledger movement.
///
/// `entry_type` uses the chest vocabulary (`grant / topup / consume /
/// expire / adjust / reserve / refund`) as an open `String` — see the
/// module docs for why it is not an enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Open vocabulary word describing the movement kind.
    pub entry_type: String,
    /// Signed amount: credits positive, debits negative.
    pub amount: i64,
    /// Optional kind of the correlation object (e.g. `"task"`).
    pub ref_type: Option<String>,
    /// Optional correlation id (e.g. the task id behind a reserve/refund).
    pub ref_id: Option<String>,
    /// Available balance immediately after this entry was appended.
    pub balance_after: i64,
    /// When the entry was appended.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wire shape of the core types is frozen: consumers hash / log /
    /// bridge these JSON forms, so any accidental field or casing change
    /// must break a test here first.
    #[test]
    fn wire_shape_is_stable() {
        let subject = WalletSubject {
            owner: OwnerKind::User,
            id: Uuid::parse_str("01933e52-7c7a-7113-bf13-9a01a3d1f6a7").unwrap(),
            ext_key: Some("user@example.com".to_string()),
        };
        let json = serde_json::to_value(&subject).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "owner": "user",
                "id": "01933e52-7c7a-7113-bf13-9a01a3d1f6a7",
                "ext_key": "user@example.com",
            })
        );
        assert_eq!(serde_json::to_string(&OwnerKind::Org).unwrap(), r#""org""#);
        assert_eq!(
            serde_json::to_string(&Unit::Degrees).unwrap(),
            r#""degrees""#
        );
        assert_eq!(
            serde_json::to_string(&SettlementOutcome::Clamped).unwrap(),
            r#""clamped""#
        );
        assert_eq!(
            serde_json::to_string(&CreditReason::Topup).unwrap(),
            r#""topup""#
        );
    }

    #[test]
    fn ledger_entry_round_trips() {
        let entry = LedgerEntry {
            entry_type: "reserve".to_string(),
            amount: -100,
            ref_type: Some("task".to_string()),
            ref_id: Some("task-1".to_string()),
            balance_after: 900,
            created_at: DateTime::parse_from_rfc3339("2026-11-07T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: LedgerEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
        // Open vocabulary: an unknown entry_type from a future backend must
        // survive the round trip untouched.
        let unknown: LedgerEntry =
            serde_json::from_str(&json.replace("reserve", "liquidate")).unwrap();
        assert_eq!(unknown.entry_type, "liquidate");
    }

    #[test]
    fn entry_query_defaults_to_first_page() {
        let q = EntryQuery::default();
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 50);
    }
}
