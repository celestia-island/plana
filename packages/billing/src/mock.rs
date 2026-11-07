//! In-memory [`Wallet`] reference implementation for tests.
//!
//! This is the shared test double for every consuming repo (ERP / scepter /
//! arona / new-api): its semantics ARE the contract, exercised by the test
//! suite in this crate. It uses a single `Mutex<HashMap>` and has no I/O,
//! so tests run anywhere `tokio` does.
//!
//! Semantics (mirroring chest's ledger facts and arona's settle):
//!
//! - every subject starts with the constructor-given initial balance
//!   (lazily seeded on first touch; `ext_key` never participates in
//!   identity — only `(owner, id)` does);
//! - `reserve` deducts the hold from the balance immediately and is
//!   idempotent per `(subject, ref_id)` (duplicate →
//!   [`WalletError::AlreadyReserved`]);
//! - `refund` releases at most the ref's reserved net
//!   (`reserved − refunded`; over-cap requests are capped, not rejected);
//! - `settle` charges the ref's hold first, then the available balance,
//!   and clamps the total charge at what the wallet can cover
//!   ([`SettlementOutcome::Clamped`] — never negative);
//! - every mutation appends a [`LedgerEntry`] using the chest vocabulary
//!   (`reserve` / `refund` / `consume` / `topup` / `grant` / `adjust` /
//!   `expire`), newest-first pagination over those entries.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::error::WalletError;
use crate::types::{
    CreditReason, EntryPage, EntryQuery, LedgerEntry, OwnerKind, ReserveOp, Settlement,
    SettlementOutcome, TaskBudget, Unit, WalletBalance, WalletSubject,
};
use crate::wallet::Wallet;

/// Identity key of a subject: `ext_key` deliberately excluded.
type SubjectKey = (OwnerKind, Uuid);

/// Full identity key of a reservation: subject + caller-chosen ref id.
type ReservationKey = (OwnerKind, Uuid, String);

/// One reservation against a `(subject, ref_id)` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reservation {
    reserved: i64,
    refunded: i64,
}

impl Reservation {
    fn net(&self) -> i64 {
        self.reserved - self.refunded
    }
}

#[derive(Debug)]
struct MockState {
    initial: i64,
    unit: Unit,
    balances: HashMap<SubjectKey, i64>,
    reservations: HashMap<ReservationKey, Reservation>,
    entries: HashMap<SubjectKey, Vec<LedgerEntry>>,
    /// First outcome per (subject, ref_id): settle retries return it
    /// verbatim instead of re-applying (the contract crate pins
    /// settle/credit as idempotent — a lost response retried must not
    /// double-charge or double-mint).
    settlements: HashMap<(SubjectKey, String), Settlement>,
    /// Balance as of first application per (subject, ref_type, ref_id):
    /// credit retries return it without appending.
    credit_replays: HashMap<(SubjectKey, String, String), i64>,
}

/// In-memory wallet; wrap in `Arc` and hand out as `Arc<dyn Wallet>`.
#[derive(Debug)]
pub struct MockWallet {
    state: Mutex<MockState>,
}

impl MockWallet {
    /// A wallet whose subjects all start with `initial` (unit
    /// [`Unit::Degrees`]).
    pub fn new(initial: i64) -> Self {
        Self::with_unit(initial, Unit::Degrees)
    }

    /// Like [`MockWallet::new`] but with an explicit accounting unit.
    pub fn with_unit(initial: i64, unit: Unit) -> Self {
        Self {
            state: Mutex::new(MockState {
                initial,
                unit,
                balances: HashMap::new(),
                reservations: HashMap::new(),
                entries: HashMap::new(),
                settlements: HashMap::new(),
                credit_replays: HashMap::new(),
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, MockState> {
        // A panic in another test thread must not poison the shared double
        // into permanent lock failures; recover the state instead.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn subject_key(s: &WalletSubject) -> SubjectKey {
        (s.owner, s.id)
    }

    fn reservation_key(s: &WalletSubject, ref_id: &str) -> ReservationKey {
        (s.owner, s.id, ref_id.to_string())
    }
}

/// Amounts must be strictly positive (`Expire` credits may be negative to
/// write value off). Anything else is a caller-side contract violation and
/// answers [`WalletError::Internal`], changing nothing.
fn check_amount(amount: i64, what: &str) -> Result<(), WalletError> {
    if amount <= 0 {
        Err(WalletError::Internal(format!(
            "{what} amount must be positive, got {amount}"
        )))
    } else {
        Ok(())
    }
}

const ERR_CREDIT_ZERO: &str = "credit amount must be non-zero";

fn entry_type_for(reason: CreditReason) -> &'static str {
    match reason {
        CreditReason::Topup => "topup",
        CreditReason::Grant => "grant",
        CreditReason::Adjust => "adjust",
        CreditReason::Expire => "expire",
    }
}

#[async_trait]
impl Wallet for MockWallet {
    async fn balance(&self, s: &WalletSubject) -> Result<WalletBalance, WalletError> {
        let mut state = self.lock();
        let key = Self::subject_key(s);
        let available = available_of(&mut state, key);
        Ok(WalletBalance {
            available,
            unit: state.unit,
        })
    }

    async fn reserve(&self, s: &WalletSubject, op: &ReserveOp) -> Result<i64, WalletError> {
        check_amount(op.amount, "reserve")?;
        let mut state = self.lock();
        let rkey = Self::reservation_key(s, &op.ref_id);
        if state.reservations.contains_key(&rkey) {
            return Err(WalletError::AlreadyReserved {
                ref_id: op.ref_id.clone(),
            });
        }
        let key = Self::subject_key(s);
        let have = available_of(&mut state, key);
        if have < op.amount {
            return Err(WalletError::Insufficient {
                have,
                need: op.amount,
            });
        }
        let balance = have - op.amount;
        state.balances.insert(key, balance);
        state.reservations.insert(
            rkey,
            Reservation {
                reserved: op.amount,
                refunded: 0,
            },
        );
        append_entry(
            &mut state,
            key,
            "reserve",
            -op.amount,
            Some("task"),
            Some(&op.ref_id),
            balance,
        );
        Ok(balance)
    }

    async fn refund(
        &self,
        s: &WalletSubject,
        ref_id: &str,
        amount: i64,
    ) -> Result<i64, WalletError> {
        check_amount(amount, "refund")?;
        let mut state = self.lock();
        let rkey = Self::reservation_key(s, ref_id);
        let Some(reservation) = state.reservations.get_mut(&rkey) else {
            return Err(WalletError::NotFound);
        };
        // Hard cap: total refunds for a ref can never exceed its reserved
        // net. Over-cap requests are capped (the contract's clamp
        // philosophy); the released amount is what actually returns.
        let released = amount.min(reservation.net());
        reservation.refunded += released;
        let key = Self::subject_key(s);
        let balance = available_of(&mut state, key) + released;
        state.balances.insert(key, balance);
        append_entry(
            &mut state,
            key,
            "refund",
            released,
            Some("task"),
            Some(ref_id),
            balance,
        );
        Ok(balance)
    }

    async fn task_card(&self, s: &WalletSubject, ref_id: &str) -> Result<TaskBudget, WalletError> {
        let state = self.lock();
        let rkey = Self::reservation_key(s, ref_id);
        let Some(reservation) = state.reservations.get(&rkey) else {
            return Err(WalletError::NotFound);
        };
        Ok(TaskBudget {
            reserved: reservation.reserved,
            refunded: reservation.refunded,
            net: reservation.net(),
            open: reservation.net() > 0,
        })
    }

    async fn settle(
        &self,
        s: &WalletSubject,
        amount: i64,
        ref_id: &str,
    ) -> Result<Settlement, WalletError> {
        check_amount(amount, "settle")?;
        let mut state = self.lock();
        let key = Self::subject_key(s);
        let rkey = Self::reservation_key(s, ref_id);
        // Idempotent replay (contract: a settle retry after a lost
        // response must not double-charge). The first outcome for this
        // (subject, ref_id) is stored and returned verbatim.
        let settle_key = (key, ref_id.to_string());
        if let Some(prev) = state.settlements.get(&settle_key) {
            return Ok(*prev);
        }
        // The ref's hold absorbs the charge first (escrow semantics); only
        // the part beyond the hold touches the available balance. The total
        // charge clamps at hold + available: a wallet never goes negative
        // (arona points-first).
        let hold_net = state.reservations.get(&rkey).map_or(0, Reservation::net);
        let available = available_of(&mut state, key);
        let charged = amount.min(hold_net.saturating_add(available));
        let from_hold = hold_net.min(charged);
        let from_available = charged - from_hold;
        let balance = available - from_available;
        state.balances.insert(key, balance);
        if from_hold > 0 {
            let reservation = state
                .reservations
                .get_mut(&rkey)
                .expect("hold_net > 0 implies the reservation exists");
            reservation.refunded += from_hold;
        }
        append_entry(
            &mut state,
            key,
            "consume",
            -charged,
            Some("task"),
            Some(ref_id),
            balance,
        );
        let outcome = if charged == amount {
            SettlementOutcome::Covered
        } else {
            SettlementOutcome::Clamped
        };
        let settlement = Settlement {
            outcome,
            deducted: charged,
        };
        state.settlements.insert(settle_key, settlement);
        Ok(settlement)
    }

    async fn credit(
        &self,
        s: &WalletSubject,
        amount: i64,
        reason: CreditReason,
        ref_type: &str,
        ref_id: &str,
    ) -> Result<i64, WalletError> {
        let valid = match reason {
            CreditReason::Expire => amount != 0,
            // Only `Expire` legitimately writes value off; the other
            // reasons must add funds.
            _ => amount > 0,
        };
        if !valid {
            return Err(WalletError::Internal(ERR_CREDIT_ZERO.to_string()));
        }
        // Idempotent replay (contract): a credit retry with the same
        // (subject, ref_type, ref_id) returns the balance as of the
        // first application without appending anything (no double-mint).
        let credit_key = (
            Self::subject_key(s),
            ref_type.to_string(),
            ref_id.to_string(),
        );
        let mut state = self.lock();
        if let Some(balance) = state.credit_replays.get(&credit_key) {
            return Ok(*balance);
        }
        let key = Self::subject_key(s);
        let balance = available_of(&mut state, key) + amount;
        state.balances.insert(key, balance);
        state.credit_replays.insert(credit_key, balance);
        append_entry(
            &mut state,
            key,
            entry_type_for(reason),
            amount,
            Some(ref_type),
            Some(ref_id),
            balance,
        );
        Ok(balance)
    }

    async fn entries(&self, s: &WalletSubject, q: &EntryQuery) -> Result<EntryPage, WalletError> {
        let state = self.lock();
        let key = Self::subject_key(s);
        let all = state.entries.get(&key).map_or(&[][..], Vec::as_slice);
        let total = all.len() as u64;
        let page = q.page.max(1) as u64;
        let entries = if q.page_size == 0 {
            Vec::new()
        } else {
            // Newest first: entries are appended chronologically (oldest at
            // index 0), so the newest-first page `p` is the slice ending at
            // `len - (p-1)*size`, walked backwards.
            let size = q.page_size as u64;
            let upper = all
                .len()
                .saturating_sub(page.saturating_sub(1).saturating_mul(size) as usize);
            let lower = upper.saturating_sub(size as usize);
            all[lower..upper].iter().rev().cloned().collect()
        };
        Ok(EntryPage { entries, total })
    }
}

/// Current available balance, lazily seeding the subject with the initial
/// amount on first touch.
fn available_of(state: &mut MockState, key: SubjectKey) -> i64 {
    *state.balances.entry(key).or_insert(state.initial)
}

/// Append a ledger entry with the current timestamp.
fn append_entry(
    state: &mut MockState,
    key: SubjectKey,
    entry_type: &str,
    amount: i64,
    ref_type: Option<&str>,
    ref_id: Option<&str>,
    balance_after: i64,
) {
    state.entries.entry(key).or_default().push(LedgerEntry {
        entry_type: entry_type.to_string(),
        amount,
        ref_type: ref_type.map(str::to_string),
        ref_id: ref_id.map(str::to_string),
        balance_after,
        created_at: Utc::now(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject(id: u32) -> WalletSubject {
        WalletSubject {
            owner: OwnerKind::User,
            id: Uuid::from_u128(id as u128),
            ext_key: Some(format!("user{id}@example.com")),
        }
    }

    fn reserve_of(ref_id: &str, amount: i64) -> ReserveOp {
        ReserveOp {
            ref_id: ref_id.to_string(),
            amount,
        }
    }

    #[tokio::test]
    async fn balance_reports_the_initial_available() {
        let wallet = MockWallet::new(1_000);
        let bal = wallet.balance(&subject(1)).await.unwrap();
        assert_eq!(bal.available, 1_000);
        assert_eq!(bal.unit, Unit::Degrees);
    }

    #[tokio::test]
    async fn subjects_are_identified_by_owner_and_id_only() {
        let wallet = MockWallet::new(500);
        let a = WalletSubject {
            owner: OwnerKind::User,
            id: Uuid::from_u128(7),
            ext_key: Some("a@example.com".into()),
        };
        let same_wallet_other_ext = WalletSubject {
            owner: OwnerKind::User,
            id: Uuid::from_u128(7),
            ext_key: None,
        };
        // Same (owner, id), different ext_key → same wallet: the credit is
        // visible through both handles.
        wallet
            .credit(&a, 25, CreditReason::Grant, "admin", "g-1")
            .await
            .unwrap();
        assert_eq!(
            wallet
                .balance(&same_wallet_other_ext)
                .await
                .unwrap()
                .available,
            525
        );
        // A different owner kind is a different wallet despite the same id.
        let group = WalletSubject {
            owner: OwnerKind::Group,
            id: Uuid::from_u128(7),
            ext_key: None,
        };
        assert_eq!(wallet.balance(&group).await.unwrap().available, 500);
    }

    #[tokio::test]
    async fn reserve_deducts_and_opens_the_task_card() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        let balance = wallet
            .reserve(&s, &reserve_of("task-1", 100))
            .await
            .unwrap();
        assert_eq!(balance, 900);
        assert_eq!(wallet.balance(&s).await.unwrap().available, 900);
        let card = wallet.task_card(&s, "task-1").await.unwrap();
        assert_eq!(
            card,
            TaskBudget {
                reserved: 100,
                refunded: 0,
                net: 100,
                open: true,
            }
        );
    }

    #[tokio::test]
    async fn reserve_is_idempotent_per_subject_and_ref() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        wallet
            .reserve(&s, &reserve_of("task-1", 100))
            .await
            .unwrap();
        // Same subject + same ref: rejected, state untouched.
        let err = wallet
            .reserve(&s, &reserve_of("task-1", 50))
            .await
            .unwrap_err();
        assert_eq!(
            err,
            WalletError::AlreadyReserved {
                ref_id: "task-1".into()
            }
        );
        assert_eq!(wallet.balance(&s).await.unwrap().available, 900);
        assert_eq!(
            wallet.task_card(&s, "task-1").await.unwrap(),
            TaskBudget {
                reserved: 100,
                refunded: 0,
                net: 100,
                open: true
            }
        );
        // A different subject may reuse the ref id independently.
        let other = subject(2);
        wallet
            .reserve(&other, &reserve_of("task-1", 100))
            .await
            .unwrap();
        assert_eq!(wallet.balance(&other).await.unwrap().available, 900);
    }

    #[tokio::test]
    async fn reserve_rejects_insufficient_with_have_and_need() {
        let wallet = MockWallet::new(40);
        let err = wallet
            .reserve(&subject(1), &reserve_of("task-1", 100))
            .await
            .unwrap_err();
        assert_eq!(
            err,
            WalletError::Insufficient {
                have: 40,
                need: 100
            }
        );
        // Nothing changed.
        assert_eq!(wallet.balance(&subject(1)).await.unwrap().available, 40);
        assert!(matches!(
            wallet.task_card(&subject(1), "task-1").await,
            Err(WalletError::NotFound)
        ));
    }

    #[tokio::test]
    async fn refund_is_capped_by_the_reserved_net() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        wallet
            .reserve(&s, &reserve_of("task-1", 100))
            .await
            .unwrap();
        // Ask for 150: only the 100 net can come back.
        let balance = wallet.refund(&s, "task-1", 150).await.unwrap();
        assert_eq!(balance, 1_000);
        let card = wallet.task_card(&s, "task-1").await.unwrap();
        assert_eq!(card.net, 0);
        assert!(!card.open);
        // The closed ref can no longer release anything.
        let again = wallet.refund(&s, "task-1", 10).await.unwrap();
        assert_eq!(again, 1_000);
    }

    #[tokio::test]
    async fn refund_of_unknown_ref_is_not_found() {
        let wallet = MockWallet::new(1_000);
        let err = wallet.refund(&subject(1), "nope", 10).await.unwrap_err();
        assert_eq!(err, WalletError::NotFound);
    }

    #[tokio::test]
    async fn unknown_task_card_is_not_found() {
        let wallet = MockWallet::new(1_000);
        let err = wallet.task_card(&subject(1), "nope").await.unwrap_err();
        assert_eq!(err, WalletError::NotFound);
    }

    #[tokio::test]
    async fn settle_covers_when_the_wallet_can_pay() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        let out = wallet.settle(&s, 300, "task-1").await.unwrap();
        assert_eq!(
            out,
            Settlement {
                outcome: SettlementOutcome::Covered,
                deducted: 300
            }
        );
        assert_eq!(wallet.balance(&s).await.unwrap().available, 700);
    }

    /// The contract crate's job is to pin retry semantics BEFORE four
    /// repos invent four different ones: settle/credit are idempotent per
    /// reference, because the lost-response-then-retry case is exactly
    /// when double-charging (settle) and double-minting (credit) happen.
    #[tokio::test]
    async fn settle_and_credit_replays_return_the_first_outcome() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        let first = wallet.settle(&s, 300, "task-retry").await.unwrap();
        let balance_after_first = wallet.balance(&s).await.unwrap().available;
        // the lost-acknowledgement retry: same ref, verbatim outcome
        let replay = wallet.settle(&s, 300, "task-retry").await.unwrap();
        assert_eq!(
            replay, first,
            "settle replay must return the first settlement"
        );
        assert_eq!(
            wallet.balance(&s).await.unwrap().available,
            balance_after_first,
            "a settle replay must not charge twice"
        );

        wallet
            .credit(&s, 500, CreditReason::Topup, "order", "ord-9")
            .await
            .unwrap();
        let balance_after_credit = wallet.balance(&s).await.unwrap().available;
        let replayed = wallet
            .credit(&s, 500, CreditReason::Topup, "order", "ord-9")
            .await
            .unwrap();
        assert_eq!(
            replayed, balance_after_credit,
            "credit replay returns the credited balance"
        );
        assert_eq!(
            wallet.balance(&s).await.unwrap().available,
            balance_after_credit,
            "a credit replay must not mint twice"
        );
        // a DIFFERENT ref is a new charge/mint, not a replay
        let second = wallet.settle(&s, 100, "task-retry-2").await.unwrap();
        assert_eq!(second.deducted, 100, "a new ref charges normally");
    }

    #[tokio::test]
    async fn settle_clamps_at_zero_points_first() {
        let wallet = MockWallet::new(50);
        let s = subject(1);
        let out = wallet.settle(&s, 80, "task-1").await.unwrap();
        assert_eq!(
            out,
            Settlement {
                outcome: SettlementOutcome::Clamped,
                deducted: 50
            }
        );
        assert_eq!(wallet.balance(&s).await.unwrap().available, 0);
    }

    /// Canonical task flow: reserve → settle → refund the remainder, with
    /// strict conservation (only the settled amount is actually spent).
    #[tokio::test]
    async fn task_flow_settles_from_the_hold_then_refunds_the_rest() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        wallet
            .reserve(&s, &reserve_of("task-1", 100))
            .await
            .unwrap();
        let out = wallet.settle(&s, 30, "task-1").await.unwrap();
        assert_eq!(out.deducted, 30);
        assert_eq!(out.outcome, SettlementOutcome::Covered);
        // The hold absorbed the charge: available untouched, net shrunk.
        assert_eq!(wallet.balance(&s).await.unwrap().available, 900);
        assert_eq!(wallet.task_card(&s, "task-1").await.unwrap().net, 70);
        // The caller closes the task by refunding the remainder.
        wallet.refund(&s, "task-1", 70).await.unwrap();
        assert_eq!(wallet.balance(&s).await.unwrap().available, 970);
        let card = wallet.task_card(&s, "task-1").await.unwrap();
        assert!(!card.open);
    }

    #[tokio::test]
    async fn settle_overflow_charges_available_after_the_hold() {
        let wallet = MockWallet::new(100);
        let s = subject(1);
        wallet.reserve(&s, &reserve_of("task-1", 80)).await.unwrap();
        // 90 total: 80 from the hold, 10 beyond it from available.
        let out = wallet.settle(&s, 90, "task-1").await.unwrap();
        assert_eq!(
            out,
            Settlement {
                outcome: SettlementOutcome::Covered,
                deducted: 90
            }
        );
        assert_eq!(wallet.balance(&s).await.unwrap().available, 10);
        assert_eq!(wallet.task_card(&s, "task-1").await.unwrap().net, 0);
    }

    #[tokio::test]
    async fn settle_clamps_at_hold_plus_available() {
        let wallet = MockWallet::new(100);
        let s = subject(1);
        wallet.reserve(&s, &reserve_of("task-1", 80)).await.unwrap();
        // Ask 200 with only 80 (hold) + 20 (available) coverable.
        let out = wallet.settle(&s, 200, "task-1").await.unwrap();
        assert_eq!(
            out,
            Settlement {
                outcome: SettlementOutcome::Clamped,
                deducted: 100
            }
        );
        assert_eq!(wallet.balance(&s).await.unwrap().available, 0);
        assert_eq!(wallet.task_card(&s, "task-1").await.unwrap().net, 0);
    }

    #[tokio::test]
    async fn credit_appends_reason_entries() {
        let wallet = MockWallet::new(0);
        let s = subject(1);
        assert_eq!(
            wallet
                .credit(&s, 500, CreditReason::Topup, "order", "o-1")
                .await
                .unwrap(),
            500
        );
        assert_eq!(
            wallet
                .credit(&s, 25, CreditReason::Grant, "admin", "g-1")
                .await
                .unwrap(),
            525
        );
        // Expire writes value off with a negative amount.
        assert_eq!(
            wallet
                .credit(&s, -25, CreditReason::Expire, "policy", "e-1")
                .await
                .unwrap(),
            500
        );
        let page = wallet.entries(&s, &EntryQuery::default()).await.unwrap();
        assert_eq!(page.total, 3);
        let kinds: Vec<&str> = page.entries.iter().map(|e| e.entry_type.as_str()).collect();
        assert_eq!(kinds, ["expire", "grant", "topup"]);
    }

    #[tokio::test]
    async fn entries_paginate_newest_first_with_total() {
        let wallet = MockWallet::new(0);
        let s = subject(1);
        for n in 0..5 {
            wallet
                .credit(&s, 10, CreditReason::Topup, "order", &format!("o-{n}"))
                .await
                .unwrap();
        }
        // Page 1, size 2 → the two newest.
        let page = wallet
            .entries(
                &s,
                &EntryQuery {
                    page: 1,
                    page_size: 2,
                },
            )
            .await
            .unwrap();
        assert_eq!(page.total, 5);
        let refs: Vec<&str> = page
            .entries
            .iter()
            .map(|e| e.ref_id.as_deref().unwrap())
            .collect();
        assert_eq!(refs, ["o-4", "o-3"]);
        // Page 3 holds only the oldest entry.
        let last = wallet
            .entries(
                &s,
                &EntryQuery {
                    page: 3,
                    page_size: 2,
                },
            )
            .await
            .unwrap();
        assert_eq!(last.total, 5);
        assert_eq!(last.entries.len(), 1);
        assert_eq!(last.entries[0].ref_id.as_deref(), Some("o-0"));
        // Out-of-range page: empty entries, true total.
        let beyond = wallet
            .entries(
                &s,
                &EntryQuery {
                    page: 9,
                    page_size: 2,
                },
            )
            .await
            .unwrap();
        assert!(beyond.entries.is_empty());
        assert_eq!(beyond.total, 5);
        // page 0 is treated as page 1; page_size 0 yields an empty page.
        let zero_page = wallet
            .entries(
                &s,
                &EntryQuery {
                    page: 0,
                    page_size: 2,
                },
            )
            .await
            .unwrap();
        assert_eq!(zero_page.entries.len(), 2);
        let zero_size = wallet
            .entries(
                &s,
                &EntryQuery {
                    page: 1,
                    page_size: 0,
                },
            )
            .await
            .unwrap();
        assert!(zero_size.entries.is_empty());
        assert_eq!(zero_size.total, 5);
    }

    #[tokio::test]
    async fn non_positive_amounts_are_contract_violations() {
        let wallet = MockWallet::new(1_000);
        let s = subject(1);
        for amount in [0, -5] {
            assert!(matches!(
                wallet.reserve(&s, &reserve_of("t", amount)).await,
                Err(WalletError::Internal(_))
            ));
            assert!(matches!(
                wallet.refund(&s, "t", amount).await,
                Err(WalletError::Internal(_))
            ));
            assert!(matches!(
                wallet.settle(&s, amount, "t").await,
                Err(WalletError::Internal(_))
            ));
            if amount == 0 {
                assert!(matches!(
                    wallet
                        .credit(&s, amount, CreditReason::Grant, "a", "b")
                        .await,
                    Err(WalletError::Internal(_))
                ));
            }
        }
        // Nothing happened.
        assert_eq!(wallet.balance(&s).await.unwrap().available, 1_000);
        assert_eq!(
            wallet
                .entries(&s, &EntryQuery::default())
                .await
                .unwrap()
                .total,
            0
        );
    }

    #[tokio::test]
    async fn credit_rejects_non_positive_non_expire_reasons() {
        let wallet = MockWallet::new(100);
        let s = subject(1);
        assert!(matches!(
            wallet.credit(&s, 0, CreditReason::Topup, "a", "b").await,
            Err(WalletError::Internal(_))
        ));
        assert!(matches!(
            wallet.credit(&s, -10, CreditReason::Grant, "a", "b").await,
            Err(WalletError::Internal(_))
        ));
        assert_eq!(wallet.balance(&s).await.unwrap().available, 100);
    }

    /// The trait must stay object-safe and shareable: consumers hold
    /// `Arc<dyn Wallet>` / `Box<dyn Wallet>` across threads.
    #[tokio::test]
    async fn wallet_is_object_safe_and_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<MockWallet>();
        assert_send_sync::<dyn Wallet>();

        let wallet: std::sync::Arc<dyn Wallet> = std::sync::Arc::new(MockWallet::new(1_000));
        let s = subject(1);
        wallet
            .reserve(&s, &reserve_of("task-1", 100))
            .await
            .unwrap();
        let handle = tokio::spawn(async move { wallet.balance(&s).await.unwrap().available });
        assert_eq!(handle.await.unwrap(), 900);
    }
}
