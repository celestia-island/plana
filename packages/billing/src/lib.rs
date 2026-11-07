//! `plana-billing` — the wallet interface contract shared by the
//! celestia-island services (ERP / scepter / arona / new-api).
//!
//! This crate is upstream of every wallet consumer: it declares the types,
//! the error vocabulary and the async [`Wallet`] trait, and ships two
//! implementations:
//!
//! - [`mock::MockWallet`] — an in-memory reference implementation whose
//!   semantics are the executable form of the contract (reserve idempotency,
//!   refund cap, settle clamp, insufficient rejection). Use it in tests in
//!   any repo; it has no I/O and no runtime dependencies beyond `tokio` in
//!   tests.
//! - [`bridge::ErpBridgeWallet`] (feature `client`, off by default) — the
//!   signed HTTP bridge to the ERP wallet service
//!   (`/api/wallet/service/*`): HMAC-SHA256 over
//!   `method|subject.owner|subject.id|ts|body`, keyed by the service
//!   secret. The ERP side does not implement this line protocol yet — this
//!   crate is its single home (PR-A), so the bridge's request/response
//!   shapes below are the protocol definition of record.
//!
//! # Semantics of record
//!
//! Carved from chest's `billing.task.*` RPC behavior plus arona's
//! `LedgerManager` settle; the load-bearing rules are documented on the
//! [`Wallet`] trait. In one line: reserve is idempotent per
//! `(subject, ref_id)`, refunds cap at the ref's reserved net, settle
//! clamps at zero, and the ledger speaks the open chest vocabulary
//! `grant / topup / consume / expire / adjust / reserve / refund`.
//!
//! # Errors on the wire
//!
//! [`error::WalletError::rpc_code`] maps each variant to a JSON-RPC code in
//! (or adjacent to) chest's reserved billing band `-32060..=-32079`:
//! `-32060` insufficient (chest `BILLING_QUOTA_EXCEEDED`), `-32066`
//! authority-moved (chest's wallet authority migrated to ERP), `-32062`
//! already-reserved, `-32064` not-found, plus the shared `-32002`
//! forbidden. `-32061`/`-32063` are chest's own (channel quota /
//! payment-channel-not-configured) and stay untouched.
//!
//! # Credential discipline
//!
//! Bridge secrets arrive per instance (`ErpBridgeWallet::new`), live in
//! memory only, and never appear in this crate's sources or tests — the
//! tests use the `test-secret` placeholder constant exclusively.
//!
//! The crate deliberately does not depend on kirino or plana-config: the
//! contract stays zero-burden for consumers that only need the trait.

#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "client")]
pub mod bridge;
pub mod error;
pub mod mock;
pub mod types;
pub mod wallet;

#[cfg(feature = "client")]
pub use bridge::{ErpBridgeWallet, WALLET_SERVICE_PATH_PREFIX};
pub use error::WalletError;
pub use mock::MockWallet;
pub use types::{
    CreditReason, EntryPage, EntryQuery, LedgerEntry, OwnerKind, ReserveOp, Settlement,
    SettlementOutcome, TaskBudget, Unit, WalletBalance, WalletSubject,
};
pub use wallet::Wallet;
