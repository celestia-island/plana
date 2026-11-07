//! The [`Wallet`] trait — the multi-repo wallet interface contract.
//!
//! Design basis: chest's `billing.task.*` RPC semantics
//! (`billing.task.reserve` / `billing.task.refund` / `billing.task.budget`)
//! plus arona's `LedgerManager` settle. The three load-bearing rules those
//! semantics fix, and which every implementation must honor:
//!
//! 1. **Reserve is idempotent per `(subject, ref_id)`** — enforced by the
//!    implementation's uniqueness constraint, not by the caller. A second
//!    reserve with the same ref id fails with
//!    [`WalletError::AlreadyReserved`]; different subjects may reuse the
//!    same ref id independently.
//! 2. **Refund is capped by the ref's reserved net** — total refunds for a
//!    ref can never exceed what is still held for it
//!    (`reserved − already refunded`).
//! 3. **Settle clamps at zero** — the wallet never goes negative
//!    (arona points-first semantics): when the balance cannot cover the
//!    amount, the shortfall is not an error but a
//!    [`crate::types::SettlementOutcome::Clamped`] settlement.
//!
//! Amounts are integers in the wallet's [`crate::types::Unit`]; all
//! mutating methods require strictly positive amounts and reject
//! non-positive ones with [`WalletError::Internal`] (a caller-side contract
//! violation, not a wallet state).

use async_trait::async_trait;

use crate::error::WalletError;
use crate::types::{
    CreditReason, EntryPage, EntryQuery, ReserveOp, Settlement, TaskBudget, WalletBalance,
    WalletSubject,
};

/// Shared wallet interface for ERP / scepter / arona / new-api.
///
/// `#[async_trait]` keeps the trait object-safe: business code holds
/// `Arc<dyn Wallet>` / `Box<dyn Wallet>` and must not downcast to a
/// concrete wallet. The [`crate::mock::MockWallet`] is the reference
/// in-memory implementation for tests in every consuming repo.
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Current available balance of the subject's wallet.
    ///
    /// Available = credited − reserved − settled + refunded: funds held by
    /// open reservations are *not* part of `available`.
    async fn balance(&self, s: &WalletSubject) -> Result<WalletBalance, WalletError>;

    /// Hold `op.amount` for the caller-chosen `op.ref_id`.
    ///
    /// Idempotent per `(subject, ref_id)` via the implementation's unique
    /// constraint: a duplicate ref id answers [`WalletError::AlreadyReserved`]
    /// and changes nothing. On success the amount is deducted from
    /// `available` immediately (the hold) and the post-operation available
    /// balance is returned (chest's `billing.task.reserve` answers
    /// `{balance, reserved}`; this contract keeps the `balance` half).
    /// [`WalletError::Insufficient`] carries `have`/`need` and changes
    /// nothing.
    async fn reserve(&self, s: &WalletSubject, op: &ReserveOp) -> Result<i64, WalletError>;

    /// Release previously reserved funds.
    ///
    /// `amount` is capped by the ref's reserved net (`reserved − refunded`
    /// so far): a refund larger than the net releases only the net — the
    /// cap is a hard invariant, never a negative balance. Refunding a ref
    /// that has no reservation answers [`WalletError::NotFound`]. Returns
    /// the post-operation available balance.
    async fn refund(
        &self,
        s: &WalletSubject,
        ref_id: &str,
        amount: i64,
    ) -> Result<i64, WalletError>;

    /// Read-only budget snapshot of one reservation: `reserved`,
    /// `refunded`, `net = reserved − refunded`, and `open` (still holding
    /// funds). Unknown ref ids answer [`WalletError::NotFound`].
    async fn task_card(&self, s: &WalletSubject, ref_id: &str) -> Result<TaskBudget, WalletError>;

    /// Deduct actual consumption, clamping at zero.
    ///
    /// Deducts `min(amount, available)`; a balance that runs out
    /// mid-settlement yields
    /// [`crate::types::SettlementOutcome::Clamped`] instead of an
    /// error (arona points-first: a wallet never goes negative), while a
    /// full deduction yields
    /// [`crate::types::SettlementOutcome::Covered`]. When a
    /// reservation exists under `ref_id`, the deducted amount consumes that
    /// reservation's net (the consumed part is no longer refundable) — the
    /// caller then refunds the remainder to close the task.
    async fn settle(
        &self,
        s: &WalletSubject,
        amount: i64,
        ref_id: &str,
    ) -> Result<Settlement, WalletError>;

    /// Add funds (`Topup` / `Grant` / `Adjust` / `Expire` reasons;
    /// `Expire` is expected to carry a negative `amount`) and append a
    /// ledger entry tagged with `ref_type`/`ref_id`. Returns the
    /// post-operation available balance.
    async fn credit(
        &self,
        s: &WalletSubject,
        amount: i64,
        reason: CreditReason,
        ref_type: &str,
        ref_id: &str,
    ) -> Result<i64, WalletError>;

    /// Paged ledger for the subject, newest entries first. `q.page` is
    /// 1-based (0 is treated as 1); `q.page_size == 0` yields an empty
    /// page; out-of-range pages yield empty `entries` with the true
    /// `total`.
    async fn entries(&self, s: &WalletSubject, q: &EntryQuery) -> Result<EntryPage, WalletError>;
}
