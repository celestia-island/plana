//! Deferred-operation registry: the framework primitive for handlers whose
//! upstream call must not sit on the JSON-RPC dispatch path.
//!
//! # The rule this implements
//!
//! [`crate::RpcServerConfig::dispatch_stall_limit`] is a **liveness guard
//! measured in seconds, never an upstream budget**. A handler whose upstream
//! can outlive it (an LLM call with a 90s budget, a payment/settlement
//! request, a provider callback) answers **immediately** with a deferred
//! reference and settles the real result later through this registry:
//!
//! ```no_run
//! use plana_rpc_server::{RpcRequestCtx, RpcServer};
//! use serde_json::json;
//!
//! # async fn example() {
//! let server = RpcServer::builder()
//!     .method_ctx("payment.submit", |ctx: RpcRequestCtx| async move {
//!         // The worker owns the upstream call; the dispatch returns now.
//!         ctx.defer(move |handle| async move {
//!             let receipt = pay_upstream().await?;      // may take minutes
//!             let _ = handle;                           // handle.complete is implicit
//!             Ok(receipt)
//!         })
//!     })
//!     .build();
//! # let _ = server;
//! # }
//! # async fn pay_upstream() -> Result<serde_json::Value, plana::jsonrpc::JsonRpcError> {
//! #     Ok(json!({"paid": true}))
//! # }
//! ```
//!
//! # Semantics (deliberate choices, see the module docs of
//! `plana::jsonrpc::deferred` for the wire shapes)
//!
//! - **Validity is measured from creation**, not from settlement: the window
//!   a client was promised (`expires_in` in the immediate answer) is the
//!   window a reconnecting client gets. The intended band for client-facing
//!   ids is 10–30 minutes; the default is 30.
//! - **Collection is non-destructive.** A settled outcome stays collectable —
//!   repeatedly — until the TTL elapses, *or until the retention cap below
//!   evicts it* (the id then answers `-32052` even though its window is still
//!   open). A destructive read would make a single lost response frame lose an
//!   outcome the caller already paid for, and would make retrying a collection
//!   unsafe. The price is retention bounded by the caps below.
//! - **Settlement notification is advisory.** When the originating
//!   connection is still open the registry pushes `ops.settled {op_id,
//!   status}` on the data lane, best-effort and payload-free. It is never the
//!   authoritative path: a client that misses it polls `ops.result`.
//! - **Cancellation is a flag, not a scheduler.** `ops.cancel` records a
//!   request the worker may observe ([`OpHandle::cancel_requested`]); the
//!   registry never kills anything.
//! - **Bounded and self-pruning.** Expired entries are dropped on every
//!   [`DeferredOps::begin`] and on every lookup of an expired id, so memory
//!   cannot grow without limit; there is no background task to own or shut
//!   down. Once the pending cap is reached a `begin` is refused
//!   structurally (`-32054`) instead of silently dropping older ops.
//! - **Constant work per dispatch.** The pending count, the expiry sweep and
//!   the retention eviction are all answered from side indexes that are
//!   maintained under the same lock, so `begin` on the dispatch path never
//!   scans the table: it reads one counter and touches two ordered indexes.
//!   Only the entries that actually expired are walked.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use plana::jsonrpc::deferred::{
    DeferredOpCreated, DeferredOpOutcome, DeferredOpRef, DeferredOpSettledParams, DeferredOpStatus,
    OpsCancelResult, DEFAULT_CLIENT_TTL_SECS, OPS_SETTLED_METHOD,
};
use plana::jsonrpc::{error_codes, JsonRpcError};
use serde_json::Value;

use crate::connection::OutboundHandle;

/// Tuning knobs of a [`DeferredOps`] registry.
#[derive(Debug, Clone)]
pub struct DeferredOpsConfig {
    /// How long a client-facing id stays valid, measured from creation
    /// (default 30 minutes — the top of the intended 10–30 minute band).
    /// Shortening this below the band is a client-facing contract change:
    /// a client that reconnects near the end of the window loses the outcome.
    /// The registry accepts 1ms–24h (see [`DeferredOpsConfig::MIN_TTL`] and
    /// [`DeferredOpsConfig::MAX_TTL`]) and advertises the configured window
    /// verbatim, so a deployment outside the band is publishing a different
    /// contract than the service profile describes.
    pub ttl: Duration,

    /// Maximum number of concurrently **pending** (unsettled) operations
    /// (default 1024). A `begin` beyond this is refused with `-32054` so
    /// callers see backpressure instead of an op that can never be
    /// collected.
    ///
    /// The cap is server-global, with no per-connection accounting: one caller
    /// can fill it for the length of a window. Per-caller fairness belongs in
    /// the per-request guard (`RpcServerBuilder::guard`), which sees the
    /// method being dispatched.
    pub max_pending: usize,

    /// Maximum number of **retained** entries, pending plus settled-but-
    /// uncollected (default 4096). When the cap is reached the oldest
    /// settled entries are evicted first — pending work is never evicted —
    /// so a client that has not collected yet cannot starve the service.
    /// Clamped up to `max_pending`.
    ///
    /// This bounds the **entry count**, not the bytes: a settled entry keeps
    /// whatever `Value` the worker produced until it is collected or the
    /// window elapses, so the payload size is the service's responsibility.
    /// Eviction is the one case where a settled outcome can disappear
    /// **before** its window elapses (the id then answers `-32052`), which is
    /// the price of a hard bound; raise `max_retained` for services whose
    /// results are large or whose clients collect slowly.
    pub max_retained: usize,
}

impl Default for DeferredOpsConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(DEFAULT_CLIENT_TTL_SECS),
            max_pending: 1024,
            max_retained: 4096,
        }
    }
}

impl DeferredOpsConfig {
    /// Lower bound of a usable window: short enough for tests and local
    /// tuning, long enough that an id is never born dead. A zero window would
    /// advertise `expires_in = 0` and drop every worker result on arrival.
    pub const MIN_TTL: Duration = Duration::from_millis(1);

    /// Upper bound of a usable window. The published client-facing band is
    /// 10–30 minutes; the ceiling exists so a misconfigured "effectively
    /// infinite" window can neither overflow the deadline computation (which
    /// runs on the dispatch path) nor lie to clients about validity.
    pub const MAX_TTL: Duration = Duration::from_secs(24 * 60 * 60);

    /// Keep the knobs in a state the registry can honour: a usable window, a
    /// non-zero pending cap, and room for every pending op inside the
    /// retention cap.
    ///
    /// A window below the published 10-minute client-facing minimum is a
    /// deliberate local choice (tests, local tuning), never a client-facing
    /// contract.
    fn normalized(&self) -> Self {
        let max_pending = self.max_pending.max(1);
        Self {
            ttl: self.ttl.clamp(Self::MIN_TTL, Self::MAX_TTL),
            max_pending,
            max_retained: self.max_retained.max(max_pending),
        }
    }
}

/// Everything that can go wrong in the registry.
///
/// `Unknown`/`Expired`/`Capacity` map onto the published service-profile
/// error codes through `From<DeferredOpsError> for JsonRpcError`, so a
/// handler can simply `?` a registry call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeferredOpsError {
    /// The registry is at its pending cap; the caller should retry later.
    Capacity {
        /// Currently pending operations.
        pending: usize,
        /// The configured cap.
        max_pending: usize,
    },
    /// The id was never issued by this server (or was evicted before its
    /// window elapsed).
    Unknown,
    /// The id was issued by this server and its validity window elapsed.
    Expired,
    /// The operation already settled; there is nothing left to settle.
    Settled,
}

impl std::fmt::Display for DeferredOpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capacity {
                pending,
                max_pending,
            } => write!(
                f,
                "deferred-op registry at capacity ({pending}/{max_pending} pending)"
            ),
            Self::Unknown => f.write_str("unknown deferred operation"),
            Self::Expired => f.write_str("deferred operation expired"),
            Self::Settled => f.write_str("deferred operation already settled"),
        }
    }
}

impl std::error::Error for DeferredOpsError {}

impl From<DeferredOpsError> for JsonRpcError {
    fn from(err: DeferredOpsError) -> Self {
        match err {
            DeferredOpsError::Capacity {
                pending,
                max_pending,
            } => JsonRpcError::new(
                error_codes::OPS_CAPACITY,
                "deferred-op registry at capacity",
            )
            .with_data(serde_json::json!({
                "pending": pending,
                "max_pending": max_pending,
            })),
            DeferredOpsError::Unknown => {
                JsonRpcError::new(error_codes::OPS_UNKNOWN, "unknown deferred operation")
                    .with_data(serde_json::json!({ "reason": "unknown_op_id" }))
            }
            DeferredOpsError::Expired => {
                JsonRpcError::new(error_codes::OPS_EXPIRED, "deferred operation expired")
                    .with_data(serde_json::json!({ "reason": "op_id_expired" }))
            }
            DeferredOpsError::Settled => {
                JsonRpcError::internal_error("deferred operation already settled")
            }
        }
    }
}

struct Settlement {
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

struct Entry {
    method: String,
    /// Creation + TTL: the validity window is measured from **creation**, so
    /// a client that reconnects mid-window still sees the remainder.
    deadline: Instant,
    settlement: Option<Settlement>,
    cancel_requested: bool,
    /// The connection the op was begun on, kept for the best-effort
    /// settlement notification only; released as soon as the op settles.
    origin: Option<OutboundHandle>,
    /// Monotonic order of settlement, and the key this entry holds in
    /// `State::settled` once it settles (0 while pending, which is a valid
    /// NOTE: 0 doubles as the "pending" sentinel while the first real settled
    /// key is ALSO 0 — nothing relies on the sentinel (gating is on
    /// `settlement.is_some()`), do not key anything off the raw value.
    settled_seq: u64,
}

impl Entry {
    fn status(&self) -> DeferredOpStatus {
        match &self.settlement {
            None => DeferredOpStatus::Pending,
            Some(s) if s.error.is_some() => DeferredOpStatus::Failed,
            Some(_) => DeferredOpStatus::Completed,
        }
    }

    fn expires_in_secs(&self, now: Instant) -> u64 {
        secs_ceil(self.deadline.saturating_duration_since(now))
    }

    fn outcome(&self, op_id: &DeferredOpRef, now: Instant) -> DeferredOpOutcome {
        DeferredOpOutcome {
            op_id: op_id.clone(),
            status: self.status(),
            method: self.method.clone(),
            expires_in: self.expires_in_secs(now),
            cancel_requested: self.cancel_requested,
            result: self.settlement.as_ref().and_then(|s| s.result.clone()),
            error: self.settlement.as_ref().and_then(|s| s.error.clone()),
        }
    }
}

/// The locked half of a registry: the entries plus the two side indexes that
/// make `begin` cheap.
///
/// - `Inner::pending` (an atomic outside this struct) is the exact pending
///   count, so the capacity check reads one `usize` instead of walking every
///   entry.
/// - `settled` orders settled entries by settlement, so eviction picks the
///   oldest victim in `O(log n)`.
/// - `by_deadline` orders live entries by deadline, so the expiry sweep pops
///   only what expired instead of retaining the whole table.
///
/// The three only stay in agreement because **every** mutation of `entries`
/// goes through `State::insert` and `State::remove`; no other code may
/// touch the map.
#[derive(Default)]
struct State {
    entries: HashMap<DeferredOpRef, Entry>,
    /// Settled entries keyed by their unique `Entry::settled_seq`.
    settled: BTreeMap<u64, DeferredOpRef>,
    /// Live entries grouped by deadline. A `Vec` per deadline because two
    /// `begin`s can land on the same `Instant`; `remove` drops the bucket when
    /// it empties, so there is no tombstone growth.
    by_deadline: BTreeMap<Instant, Vec<DeferredOpRef>>,
}

impl State {
    /// Register a freshly begun (still pending) entry in both indexes.
    fn insert(&mut self, pending: &AtomicUsize, op: DeferredOpRef, entry: Entry) {
        debug_assert!(
            entry.settlement.is_none(),
            "an entry is pending until it is settled"
        );
        self.by_deadline
            .entry(entry.deadline)
            .or_default()
            .push(op.clone());
        self.entries.insert(op, entry);
        pending.fetch_add(1, Ordering::Relaxed);
    }

    /// Drop an entry together with every index reference to it, keeping the
    /// pending count exact: a pending entry that goes away (expiry) frees its
    /// slot, a settled one leaves the eviction order.
    fn remove(&mut self, pending: &AtomicUsize, op: &DeferredOpRef) -> Option<Entry> {
        let entry = self.entries.remove(op)?;
        match entry.settlement {
            Some(_) => {
                self.settled.remove(&entry.settled_seq);
            }
            None => {
                pending.fetch_sub(1, Ordering::Relaxed);
            }
        }
        let emptied = match self.by_deadline.get_mut(&entry.deadline) {
            Some(bucket) => {
                bucket.retain(|queued| queued != op);
                bucket.is_empty()
            }
            None => false,
        };
        if emptied {
            self.by_deadline.remove(&entry.deadline);
        }
        Some(entry)
    }
}

struct Inner {
    config: DeferredOpsConfig,
    state: Mutex<State>,
    /// Exact number of pending (unsettled) entries. Every mutation happens
    /// under the `state` lock — `begin` adds one, settlement and the removal
    /// of a pending entry each subtract one — so a read taken under that lock
    /// is exact, which is what the capacity check needs.
    ///
    /// Kept as its own atomic (rather than a field of [`State`]) so
    /// [`DeferredOps::pending_len`] can report without taking the lock;
    /// `Relaxed` suffices because the lock's release/acquire pair already
    /// orders the writes for the locked readers.
    pending: AtomicUsize,
    /// Bumped on every settlement before the state lock is taken. The total
    /// order is therefore the fetch order, which two racing settlers can
    /// invert relative to the order they actually take the lock in — an
    /// arbitrary but harmless tie-break, since concurrent settlements have no
    /// meaningful "older" one. The values are unique, which is what lets them
    /// key the eviction order.
    seq: AtomicU64,
}

/// Server-global registry of deferred operations.
///
/// Cloneable and cheap (one `Arc`); the same instance is shared by every
/// connection of an [`crate::RpcServer`], which is what lets a client collect
/// an outcome from a **different** connection than the one that began it.
#[derive(Clone)]
pub struct DeferredOps {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for DeferredOps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeferredOps")
            .field("ttl", &self.inner.config.ttl)
            .field("pending", &self.pending_len())
            .field("retained", &self.retained_len())
            .finish()
    }
}

impl Default for DeferredOps {
    fn default() -> Self {
        Self::new(DeferredOpsConfig::default())
    }
}

impl DeferredOps {
    /// Build a registry with the given knobs.
    pub fn new(config: DeferredOpsConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                config: config.normalized(),
                state: Mutex::new(State::default()),
                pending: AtomicUsize::new(0),
                seq: AtomicU64::new(0),
            }),
        }
    }

    /// The effective configuration (after normalization).
    pub fn config(&self) -> &DeferredOpsConfig {
        &self.inner.config
    }

    /// Adopt an operation: mint a client-facing reference and hand the worker
    /// a settle handle.
    ///
    /// Returns the immediate answer to send the client (id + remaining
    /// validity) **together with** the handle, so a handler cannot advertise
    /// an `op_id` it has no way to settle, nor settle one it never told the
    /// client about.
    ///
    /// `origin` is the connection to notify on settlement; pass
    /// [`OutboundHandle::closed`] on transports with no server-initiated
    /// direction (the HTTP POST fallback).
    pub fn begin(
        &self,
        method: &str,
        origin: OutboundHandle,
    ) -> Result<(DeferredOpCreated, OpHandle), DeferredOpsError> {
        let now = Instant::now();
        let op_id = DeferredOpRef::new_random();
        // `checked_add`, not `+`: `begin` runs on the dispatch path, outside
        // the worker's panic guard, so a misconfigured window must never panic
        // the dispatch. The fallback is the clamped ceiling rather than "now",
        // because "now" would hand the caller an id that is already expired.
        // The fallback must itself be checked: `ttl` is already clamped to
        // MAX_TTL, so the only way to get here is `now` being within a window
        // of the platform's instant bound — where `now + MAX_TTL` would panic
        // with the very overflow this guards against.
        let deadline = now
            .checked_add(self.inner.config.ttl)
            .or_else(|| now.checked_add(DeferredOpsConfig::MIN_TTL))
            .unwrap_or(now);

        {
            let mut state = self.inner.state.lock().unwrap();
            // The sweep runs under the same lock as the reservation: it can
            // only free pending slots, and the capacity check has to see the
            // count *after* it.
            sweep_expired(&mut state, &self.inner.pending, now);
            let pending = self.inner.pending.load(Ordering::Relaxed);
            if pending >= self.inner.config.max_pending {
                return Err(DeferredOpsError::Capacity {
                    pending,
                    max_pending: self.inner.config.max_pending,
                });
            }
            evict_for_retention(
                &mut state,
                &self.inner.pending,
                self.inner.config.max_retained,
            );
            state.insert(
                &self.inner.pending,
                op_id.clone(),
                Entry {
                    method: method.to_string(),
                    deadline,
                    settlement: None,
                    cancel_requested: false,
                    origin: Some(origin),
                    settled_seq: 0,
                },
            );
        }

        // Derived from the stored deadline rather than the configured window:
        // the two can differ when the deadline fell back to MIN_TTL, and the
        // client must be told the window it actually has.
        let created = DeferredOpCreated {
            op_id: op_id.clone(),
            expires_in: secs_ceil(deadline.saturating_duration_since(now)),
        };
        Ok((
            created,
            OpHandle {
                ops: self.clone(),
                op: op_id,
            },
        ))
    }

    /// Look an operation up: pending and settled-but-uncollected operations
    /// both answer `Ok(outcome)`.
    ///
    /// An id past its window answers [`DeferredOpsError::Expired`] and is
    /// pruned on the spot, so a second lookup of the same id answers
    /// [`DeferredOpsError::Unknown`]. An id this server never issued (or one
    /// evicted before its window elapsed) answers `Unknown` directly.
    pub fn status(&self, op: &DeferredOpRef) -> Result<DeferredOpOutcome, DeferredOpsError> {
        let now = Instant::now();
        let mut state = self.inner.state.lock().unwrap();
        {
            let Some(entry) = state.entries.get(op) else {
                return Err(DeferredOpsError::Unknown);
            };
            if now < entry.deadline {
                return Ok(entry.outcome(op, now));
            }
        }
        state.remove(&self.inner.pending, op);
        Err(DeferredOpsError::Expired)
    }

    /// Record a best-effort cancellation request.
    ///
    /// A still-pending operation gets `cancel_requested = true` (the worker
    /// observes it through [`OpHandle::cancel_requested`]); an already
    /// settled operation answers its final status with
    /// `cancel_requested = false` — there is nothing left to cancel, which is
    /// a valid answer, not an error.
    pub fn cancel(&self, op: &DeferredOpRef) -> Result<OpsCancelResult, DeferredOpsError> {
        let now = Instant::now();
        let mut state = self.inner.state.lock().unwrap();
        {
            let Some(entry) = state.entries.get_mut(op) else {
                return Err(DeferredOpsError::Unknown);
            };
            if now < entry.deadline {
                if entry.settlement.is_none() {
                    entry.cancel_requested = true;
                }
                return Ok(OpsCancelResult {
                    op_id: op.clone(),
                    status: entry.status(),
                    cancel_requested: entry.cancel_requested && entry.settlement.is_none(),
                });
            }
        }
        state.remove(&self.inner.pending, op);
        Err(DeferredOpsError::Expired)
    }

    /// Drop every entry whose window has elapsed; returns how many were
    /// dropped. Called automatically from [`DeferredOps::begin`], and exposed
    /// for supervisors that want to force the sweep.
    pub fn prune_expired(&self) -> usize {
        let now = Instant::now();
        let mut state = self.inner.state.lock().unwrap();
        sweep_expired(&mut state, &self.inner.pending, now)
    }

    /// Number of retained entries (pending + settled but not yet collected or
    /// expired).
    pub fn retained_len(&self) -> usize {
        self.inner.state.lock().unwrap().entries.len()
    }

    /// Number of retained entries still pending.
    ///
    /// Answered from the incrementally maintained counter, so it takes no
    /// lock; under concurrent traffic it is a gauge that was true at some
    /// instant rather than a snapshot of one.
    pub fn pending_len(&self) -> usize {
        self.inner.pending.load(Ordering::Relaxed)
    }

    /// Settle an operation: returns the status to announce and the origin to
    /// notify, or the reason the settlement was refused.
    fn settle(
        &self,
        op: &DeferredOpRef,
        settlement: Settlement,
    ) -> Result<(DeferredOpStatus, Option<OutboundHandle>), DeferredOpsError> {
        let now = Instant::now();
        let seq = self.inner.seq.fetch_add(1, Ordering::Relaxed);
        let mut state = self.inner.state.lock().unwrap();
        // The deadline check is a separate borrow from the mutation below: an
        // expired entry has to be *removed* (dropping it from both indexes and
        // freeing its pending slot), which needs the map borrowed mutably.
        let expired = match state.entries.get(op) {
            None => return Err(DeferredOpsError::Unknown),
            Some(entry) => now >= entry.deadline,
        };
        if expired {
            state.remove(&self.inner.pending, op);
            return Err(DeferredOpsError::Expired);
        }
        let entry = state
            .entries
            .get_mut(op)
            .expect("the entry exists until it is removed");
        if entry.settlement.is_some() {
            return Err(DeferredOpsError::Settled);
        }
        entry.settlement = Some(settlement);
        entry.settled_seq = seq;
        let status = entry.status();
        // The settlement notification is best-effort: release the origin as
        // soon as it has been handed back to the caller.
        let origin = entry.origin.take();
        state.settled.insert(seq, op.clone());
        self.inner.pending.fetch_sub(1, Ordering::Relaxed);
        Ok((status, origin))
    }

    /// Read the cancellation flag without a full lookup.
    fn cancel_requested(&self, op: &DeferredOpRef) -> bool {
        self.inner
            .state
            .lock()
            .unwrap()
            .entries
            .get(op)
            .map(|e| e.cancel_requested && e.settlement.is_none())
            .unwrap_or(false)
    }
}

/// Drop every entry whose window has elapsed; returns how many went away.
///
/// The deadline index makes this proportional to the entries that actually
/// expired (`O(k log n)`) rather than to the size of the table, which is what
/// keeps [`DeferredOps::begin`]'s lock hold time independent of retention.
fn sweep_expired(state: &mut State, pending: &AtomicUsize, now: Instant) -> usize {
    let mut dropped = 0;
    loop {
        let oldest = state.by_deadline.keys().next().copied();
        let Some(deadline) = oldest else { break };
        if now < deadline {
            break;
        }
        // `State::remove` drops a bucket as soon as it empties, so a key still
        // in the index always has at least one live entry behind it.
        let bucket = state
            .by_deadline
            .remove(&deadline)
            .expect("the key was just read from the index");
        for op in bucket {
            if state.remove(pending, &op).is_some() {
                dropped += 1;
            }
        }
    }
    dropped
}

/// Evict the oldest settled entries until there is room for one more, if the
/// retention cap is reached. Pending entries are never evicted.
///
/// The head of the settlement order is `O(log n)` to find, instead of a scan
/// per eviction.
fn evict_for_retention(state: &mut State, pending: &AtomicUsize, max_retained: usize) {
    while state.entries.len() >= max_retained {
        let victim = state.settled.iter().next().map(|(_, op)| op.clone());
        match victim {
            Some(op) => {
                state.remove(pending, &op);
            }
            // Everything is pending; the pending cap is what stops growth.
            None => break,
        }
    }
}

/// Round a duration up to whole seconds (so a freshly created window reads as
/// its full length rather than one second short).
fn secs_ceil(d: Duration) -> u64 {
    let millis = d.as_millis();
    u64::try_from(millis.div_ceil(1000)).unwrap_or(u64::MAX)
}

/// Worker-side handle of one deferred operation.
///
/// Cheap to clone; every clone refers to the same operation, and settlement
/// happens at most once (a second attempt answers
/// [`DeferredOpsError::Settled`]).
pub struct OpHandle {
    ops: DeferredOps,
    op: DeferredOpRef,
}

impl Clone for OpHandle {
    fn clone(&self) -> Self {
        Self {
            ops: self.ops.clone(),
            op: self.op.clone(),
        }
    }
}

impl std::fmt::Debug for OpHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpHandle")
            .field("op_id", &self.op.as_str())
            .finish_non_exhaustive()
    }
}

impl OpHandle {
    /// The client-facing reference this handle settles.
    pub fn op_ref(&self) -> &DeferredOpRef {
        &self.op
    }

    /// The client-facing reference as it appears on the wire.
    pub fn op_id(&self) -> &str {
        self.op.as_str()
    }

    /// Whether a client asked for cancellation and the operation is still
    /// pending. Advisory: the worker decides whether and when to honour it,
    /// and a worker that honours it settles with
    /// [`plana::jsonrpc::error_codes::OPS_CANCELLED`].
    pub fn cancel_requested(&self) -> bool {
        self.ops.cancel_requested(&self.op)
    }

    /// Settle with a result: the outcome becomes collectable (non-
    /// destructively, until the window elapses or the retention cap evicts it)
    /// and the originating connection — if it is still open and draining — is
    /// notified best-effort.
    ///
    /// Retention is synchronous: the announcement is fire-and-forget, so a
    /// slow or vanished client cannot make a worker wait (the client falls
    /// back to `ops.result`). The async signature leaves room for a future
    /// implementation to await delivery without breaking callers.
    pub async fn complete(&self, result: Value) -> Result<(), DeferredOpsError> {
        self.settle(Settlement {
            result: Some(result),
            error: None,
        })
        .await
    }

    /// Settle with an error.
    pub async fn fail(&self, error: JsonRpcError) -> Result<(), DeferredOpsError> {
        self.settle(Settlement {
            result: None,
            error: Some(error),
        })
        .await
    }

    async fn settle(&self, settlement: Settlement) -> Result<(), DeferredOpsError> {
        let (status, origin) = self.ops.settle(&self.op, settlement)?;
        if let Some(origin) = origin {
            let params = DeferredOpSettledParams {
                op_id: self.op.clone(),
                status,
            };
            if let Ok(params) = serde_json::to_value(params) {
                // Best-effort and never blocking: a client that is not
                // draining its lane (or has gone away) must not be able to
                // stall the worker that just settled the operation — the
                // outcome is already retained and collectable.
                if origin.try_notify(OPS_SETTLED_METHOD, params).is_err() {
                    tracing::debug!(
                        op_id = self.op.as_str(),
                        "settle notification dropped; the client can still collect"
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutboundHandle;

    fn registry(ttl: Duration, max_pending: usize, max_retained: usize) -> DeferredOps {
        DeferredOps::new(DeferredOpsConfig {
            ttl,
            max_pending,
            max_retained,
        })
    }

    fn begin(ops: &DeferredOps, method: &str) -> (DeferredOpCreated, OpHandle) {
        ops.begin(method, OutboundHandle::closed())
            .expect("registry has room")
    }

    #[tokio::test]
    async fn begin_advertises_the_full_window_and_tracks_the_remainder() {
        let ops = registry(Duration::from_secs(1800), 8, 8);
        let (created, handle) = begin(&ops, "payment.submit");
        assert_eq!(created.expires_in, 1800);
        assert_eq!(created.op_id.as_str().len(), 36);

        let outcome = ops.status(&created.op_id).unwrap();
        assert_eq!(outcome.status, DeferredOpStatus::Pending);
        assert_eq!(outcome.method, "payment.submit");
        assert_eq!(
            outcome.expires_in, 1800,
            "a freshly created id still has its whole window"
        );
        assert!(!outcome.is_settled());

        handle
            .complete(serde_json::json!({"paid": true}))
            .await
            .unwrap();
        let settled = ops.status(&created.op_id).unwrap();
        assert_eq!(settled.status, DeferredOpStatus::Completed);
        assert_eq!(settled.result, Some(serde_json::json!({"paid": true})));
        assert!(settled.error.is_none());
    }

    #[test]
    fn config_is_normalized_to_what_the_registry_can_honour() {
        // A zero window would mint ids that are already expired, and an
        // "infinite" one cannot be turned into a deadline at all.
        let degenerate = registry(Duration::ZERO, 4, 4);
        assert_eq!(degenerate.config().ttl, DeferredOpsConfig::MIN_TTL);
        let absurd = registry(Duration::MAX, 4, 4);
        assert_eq!(absurd.config().ttl, DeferredOpsConfig::MAX_TTL);
        let (created, _handle) = begin(&absurd, "payment.submit");
        assert_eq!(
            created.expires_in,
            DeferredOpsConfig::MAX_TTL.as_secs(),
            "an absurd TTL is clamped, not turned into an already-expired id"
        );
        assert_eq!(
            absurd.status(&created.op_id).unwrap().status,
            DeferredOpStatus::Pending
        );

        let ops = registry(Duration::from_secs(60), 0, 0);
        assert_eq!(ops.config().max_pending, 1, "a zero pending cap is useless");
        assert_eq!(
            ops.config().max_retained,
            1,
            "retention must have room for every pending op"
        );

        // Retention below the pending cap is raised, so a full registry can
        // always hold its pending ops.
        let ops = registry(Duration::from_secs(60), 10, 2);
        assert_eq!(ops.config().max_retained, 10);
    }

    #[tokio::test]
    async fn settled_outcomes_stay_collectable_and_settle_only_once() {
        let ops = registry(Duration::from_secs(60), 8, 8);
        let (created, handle) = begin(&ops, "llm.diagnose");
        handle
            .complete(serde_json::json!({"diagnosis": "ok"}))
            .await
            .unwrap();

        // Non-destructive collection: repeated reads keep answering.
        for _ in 0..3 {
            let outcome = ops.status(&created.op_id).unwrap();
            assert_eq!(outcome.status, DeferredOpStatus::Completed);
            assert_eq!(outcome.result, Some(serde_json::json!({"diagnosis": "ok"})));
        }

        // A second settlement is rejected, and does not overwrite the first.
        assert_eq!(
            handle.complete(serde_json::json!("late")).await,
            Err(DeferredOpsError::Settled)
        );
        assert_eq!(
            handle.fail(JsonRpcError::internal_error("late")).await,
            Err(DeferredOpsError::Settled)
        );
        assert_eq!(
            ops.status(&created.op_id).unwrap().result,
            Some(serde_json::json!({"diagnosis": "ok"}))
        );
    }

    #[tokio::test]
    async fn failed_settlement_carries_the_worker_error() {
        let ops = registry(Duration::from_secs(60), 8, 8);
        let (created, handle) = begin(&ops, "payment.submit");
        handle
            .fail(JsonRpcError::new(
                error_codes::OPS_CANCELLED,
                "operation cancelled",
            ))
            .await
            .unwrap();

        let outcome = ops.status(&created.op_id).unwrap();
        assert_eq!(outcome.status, DeferredOpStatus::Failed);
        assert!(outcome.result.is_none());
        assert_eq!(
            outcome.error.as_ref().map(|e| e.code),
            Some(error_codes::OPS_CANCELLED)
        );
    }

    #[test]
    fn expiry_answers_expired_then_unknown() {
        let ops = registry(Duration::from_millis(80), 8, 8);
        let (created, _handle) = begin(&ops, "payment.submit");
        assert_eq!(created.expires_in, 1, "80ms rounds up to one second");

        std::thread::sleep(Duration::from_millis(120));
        assert_eq!(
            ops.status(&created.op_id).unwrap_err(),
            DeferredOpsError::Expired
        );
        // The expired entry was pruned by the failed lookup.
        assert_eq!(
            ops.status(&created.op_id).unwrap_err(),
            DeferredOpsError::Unknown
        );
        assert_eq!(ops.retained_len(), 0);
        assert_eq!(
            ops.pending_len(),
            0,
            "a lookup that prunes an expired pending op releases its slot"
        );
    }

    #[test]
    fn expiry_prunes_on_begin_so_memory_cannot_grow_without_limit() {
        let ops = registry(Duration::from_millis(80), 8, 8);
        for _ in 0..5 {
            begin(&ops, "payment.submit");
        }
        assert_eq!(ops.retained_len(), 5);
        std::thread::sleep(Duration::from_millis(120));

        begin(&ops, "payment.submit");
        assert_eq!(
            ops.retained_len(),
            1,
            "every expired entry is dropped by the next begin"
        );
    }

    #[test]
    fn pending_cap_refuses_structurally_and_maps_to_the_published_code() {
        let ops = registry(Duration::from_secs(60), 1, 8);
        begin(&ops, "payment.submit");

        let err = ops
            .begin("payment.submit", OutboundHandle::closed())
            .expect_err("the second pending op is over the cap");
        assert_eq!(
            err,
            DeferredOpsError::Capacity {
                pending: 1,
                max_pending: 1
            }
        );
        let wire: JsonRpcError = err.into();
        assert_eq!(wire.code, error_codes::OPS_CAPACITY);
        assert_eq!(wire.data.unwrap()["max_pending"], 1);
    }

    #[tokio::test]
    async fn settlement_frees_pending_capacity() {
        let ops = registry(Duration::from_secs(60), 1, 8);
        let (_, handle) = begin(&ops, "payment.submit");
        assert!(ops
            .begin("payment.submit", OutboundHandle::closed())
            .is_err());

        handle.complete(serde_json::json!(1)).await.unwrap();
        assert_eq!(ops.pending_len(), 0);
        assert!(
            ops.begin("payment.submit", OutboundHandle::closed())
                .is_ok(),
            "settled entries are retained but no longer count as pending"
        );
    }

    #[tokio::test]
    async fn retention_evicts_the_oldest_settled_entry_never_a_pending_one() {
        // A registry at its retention cap that also holds a **pending** op:
        // the eviction the next `begin` triggers must take the settled entry,
        // because a pending op that is dropped can never be settled at all.
        let guarded = registry(Duration::from_secs(60), 3, 3);
        let (settled_a, handle_a) = begin(&guarded, "payment.submit");
        handle_a.complete(serde_json::json!("a")).await.unwrap();
        let (pending, _pending_handle) = begin(&guarded, "payment.submit");
        let (settled_b, handle_b) = begin(&guarded, "payment.submit");
        handle_b.complete(serde_json::json!("b")).await.unwrap();
        assert_eq!(guarded.retained_len(), 3);
        assert_eq!(guarded.pending_len(), 1);

        let (newcomer, _newcomer_handle) = begin(&guarded, "payment.submit");
        assert_eq!(
            guarded.status(&settled_a.op_id).unwrap_err(),
            DeferredOpsError::Unknown,
            "the oldest settled entry is the eviction victim"
        );
        assert_eq!(
            guarded.status(&pending.op_id).unwrap().status,
            DeferredOpStatus::Pending,
            "a pending op must never be evicted"
        );
        assert_eq!(
            guarded.status(&settled_b.op_id).unwrap().result,
            Some(serde_json::json!("b")),
            "the newer settled entry is retained"
        );
        assert_eq!(
            guarded.status(&newcomer.op_id).unwrap().status,
            DeferredOpStatus::Pending
        );

        let ops = registry(Duration::from_secs(60), 1, 2);

        let (first, handle) = begin(&ops, "payment.submit");
        handle.complete(serde_json::json!("first")).await.unwrap();
        let (second, second_handle) = begin(&ops, "payment.submit");
        second_handle
            .complete(serde_json::json!("second"))
            .await
            .unwrap();
        assert_eq!(ops.retained_len(), 2);

        // Over the retention cap: the oldest settled entry goes, the newer
        // settled entry and the new pending one stay.
        let (third, _third_handle) = begin(&ops, "payment.submit");
        assert_eq!(ops.retained_len(), 2);
        assert_eq!(
            ops.status(&first.op_id).unwrap_err(),
            DeferredOpsError::Unknown,
            "the oldest settled entry was evicted"
        );
        assert_eq!(
            ops.status(&second.op_id).unwrap().result,
            Some(serde_json::json!("second"))
        );
        assert_eq!(
            ops.status(&third.op_id).unwrap().status,
            DeferredOpStatus::Pending
        );
    }

    /// Eviction follows the **settlement** order, not the creation order — the
    /// registry evicts the outcome that has been waiting longest, and the
    /// ordered index must reproduce exactly that choice.
    #[tokio::test]
    async fn eviction_follows_settlement_order_not_creation_order() {
        // `max_retained` is clamped up to `max_pending`, so a three-entry
        // retention cap is only reachable with a three-op pending cap.
        let ops = registry(Duration::from_secs(60), 3, 3);
        let (created_a, handle_a) = begin(&ops, "payment.submit");
        let (created_b, handle_b) = begin(&ops, "payment.submit");
        let (created_c, handle_c) = begin(&ops, "payment.submit");
        assert_eq!(ops.retained_len(), 3);

        // Settle in reverse creation order: the newest op is the oldest
        // settlement.
        handle_c.complete(serde_json::json!("c")).await.unwrap();
        handle_a.complete(serde_json::json!("a")).await.unwrap();
        handle_b.complete(serde_json::json!("b")).await.unwrap();
        assert_eq!(ops.pending_len(), 0);

        // The next begin has no retention room and must take `created_c`.
        let (newcomer, _newcomer_handle) = begin(&ops, "payment.submit");
        assert_eq!(
            ops.status(&created_c.op_id).unwrap_err(),
            DeferredOpsError::Unknown,
            "the earliest settlement is the eviction victim, whatever the creation order"
        );
        assert_eq!(
            ops.status(&created_a.op_id).unwrap().result,
            Some(serde_json::json!("a"))
        );
        assert_eq!(
            ops.status(&created_b.op_id).unwrap().result,
            Some(serde_json::json!("b"))
        );
        assert_eq!(
            ops.status(&newcomer.op_id).unwrap().status,
            DeferredOpStatus::Pending
        );
        assert_eq!(ops.retained_len(), 3);
        assert_eq!(ops.pending_len(), 1, "eviction never touches the counter");
    }

    /// The incrementally maintained counter must agree with the live set
    /// across the whole lifecycle: the capacity answer is derived from it, so
    /// drift would either refuse a lawful `begin` or admit an uncollectable op.
    #[tokio::test]
    async fn pending_counter_matches_the_live_set_through_the_lifecycle() {
        let ops = registry(Duration::from_secs(60), 4, 4);
        let (created_a, handle_a) = begin(&ops, "m");
        let (created_b, handle_b) = begin(&ops, "m");
        let (_created_c, handle_c) = begin(&ops, "m");
        assert_eq!(ops.pending_len(), 3);

        // Settlement of any kind frees exactly one slot ...
        handle_a.complete(serde_json::json!(1)).await.unwrap();
        assert_eq!(ops.pending_len(), 2);
        // ... while a cancellation request is only a flag: still pending.
        assert!(ops.cancel(&created_b.op_id).unwrap().cancel_requested);
        assert_eq!(ops.pending_len(), 2);
        handle_b
            .fail(JsonRpcError::internal_error("boom"))
            .await
            .unwrap();
        assert_eq!(ops.pending_len(), 1);
        handle_c.complete(serde_json::json!(3)).await.unwrap();
        assert_eq!(ops.pending_len(), 0);
        assert_eq!(ops.retained_len(), 3, "settled entries stay retained");
        assert_eq!(created_a.expires_in, 60);

        // Fill the cap back up: the retained settled entries are evicted, but
        // the counter only tracks the pending ones.
        for _ in 0..4 {
            let (_created, _handle) = begin(&ops, "m");
        }
        assert_eq!(ops.pending_len(), 4);
        assert_eq!(ops.retained_len(), 4, "eviction kept the cap");
        let refused = ops
            .begin("m", OutboundHandle::closed())
            .expect_err("the cap is full");
        assert_eq!(
            refused,
            DeferredOpsError::Capacity {
                pending: 4,
                max_pending: 4
            },
            "the refusal quotes the counter, so it must equal the live count"
        );
        assert_eq!(ops.pending_len(), 4);
    }

    /// An expired **pending** entry releases its slot on the next sweep: the
    /// counter has to fall with the entry, or the registry would refuse every
    /// `begin` from then on.
    #[tokio::test]
    async fn an_expired_settled_entry_drops_its_eviction_key_too() {
        // M6 (R2): a settled entry removed by the sweep must also leave the
        // `settled` index, or retention eviction spins forever on a stale
        // key (a HANG, not a red assert — this test pins the interaction).
        let ops = registry(Duration::from_millis(30), 4, 4);
        let (_a, ha) = begin(&ops, "sweep-a");
        ha.complete(serde_json::json!("a")).await.unwrap();
        let (_b, hb) = begin(&ops, "sweep-b");
        hb.complete(serde_json::json!("b")).await.unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        // Sweep the expired pair implicitly via a fresh begin, then drive
        // retention to capacity: with a stale eviction key this loops
        // forever; correctly, it evicts the remaining settled entries.
        let (_c, _hc) = begin(&ops, "long-lived");
        for i in 0..8 {
            let (_op, handle) = begin(&ops, &format!("x{i}"));
            handle.complete(serde_json::json!(i)).await.unwrap();
        }
        assert!(
            ops.retained_len() <= 4,
            "retention stays bounded after sweep+eviction interleave"
        );
    }

    #[test]
    fn an_expired_pending_op_releases_its_slot() {
        let ops = registry(Duration::from_millis(60), 1, 8);
        let (_created, _handle) = begin(&ops, "payment.submit");
        assert_eq!(ops.pending_len(), 1);

        let refused = ops
            .begin("payment.submit", OutboundHandle::closed())
            .expect_err("one pending op already fills the cap");
        assert_eq!(
            refused,
            DeferredOpsError::Capacity {
                pending: 1,
                max_pending: 1
            }
        );

        std::thread::sleep(Duration::from_millis(100));
        let (_kept, _kept_handle) = begin(&ops, "payment.submit");
        assert_eq!(ops.retained_len(), 1, "the expired pending op was swept");
        assert_eq!(ops.pending_len(), 1, "and its slot was released");
    }

    /// The sweep drops pending and settled entries alike, and leaves no stale
    /// reference behind in either index.
    #[tokio::test]
    async fn prune_drops_expired_pending_and_settled_entries_alike() {
        let ops = registry(Duration::from_millis(60), 8, 8);
        let (_pending, _pending_handle) = begin(&ops, "m");
        let (_settled, settled_handle) = begin(&ops, "m");
        settled_handle
            .complete(serde_json::json!("done"))
            .await
            .unwrap();
        assert_eq!(ops.pending_len(), 1);
        assert_eq!(ops.retained_len(), 2);

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(ops.prune_expired(), 2, "both windows elapsed");
        assert_eq!(ops.retained_len(), 0);
        assert_eq!(ops.pending_len(), 0, "the counter followed the sweep");

        // The eviction order no longer holds the pruned settled entry: a fresh
        // begin neither evicts it again nor trips over a stale reference.
        let (_fresh, _fresh_handle) = begin(&ops, "m");
        assert_eq!(ops.retained_len(), 1);
        assert_eq!(ops.pending_len(), 1);
    }

    /// Churn past the retention cap: the cap holds, the counter returns to
    /// zero, and exactly the newest `max_retained` outcomes survive.
    #[tokio::test]
    async fn churn_keeps_retention_bounded_and_the_indexes_exact() {
        const CAP: usize = 16;
        const ROUNDS: u32 = 200;
        let ops = registry(Duration::from_secs(60), CAP, CAP);
        let mut created = Vec::new();
        for value in 0..ROUNDS {
            let (settled, handle) = begin(&ops, "churn");
            handle.complete(serde_json::json!(value)).await.unwrap();
            assert_eq!(ops.pending_len(), 0);
            assert!(
                ops.retained_len() <= CAP,
                "round {value} grew past the retention cap"
            );
            created.push(settled);
        }

        let evicted = ROUNDS - CAP as u32;
        for (value, settled) in created.iter().enumerate() {
            let value = u32::try_from(value).unwrap();
            match ops.status(&settled.op_id) {
                Ok(outcome) => {
                    assert!(
                        value >= evicted,
                        "op {value} should have been evicted by the cap"
                    );
                    assert_eq!(outcome.status, DeferredOpStatus::Completed);
                    assert_eq!(outcome.result, Some(serde_json::json!(value)));
                }
                Err(err) => {
                    assert!(value < evicted, "op {value} should still be retained");
                    assert_eq!(err, DeferredOpsError::Unknown);
                }
            }
        }
        assert_eq!(ops.retained_len(), CAP);
    }

    #[test]
    fn unknown_references_are_unknown_not_expired() {
        let ops = registry(Duration::from_secs(60), 8, 8);
        let stranger = DeferredOpRef::new_random();
        assert_eq!(
            ops.status(&stranger).unwrap_err(),
            DeferredOpsError::Unknown
        );
        assert_eq!(
            ops.cancel(&stranger).unwrap_err(),
            DeferredOpsError::Unknown
        );
    }

    #[tokio::test]
    async fn cancel_flags_pending_operations_only() {
        let ops = registry(Duration::from_secs(60), 8, 8);
        let (created, handle) = begin(&ops, "payment.submit");
        assert!(!handle.cancel_requested());

        let recorded = ops.cancel(&created.op_id).unwrap();
        assert_eq!(recorded.status, DeferredOpStatus::Pending);
        assert!(recorded.cancel_requested);
        assert!(
            handle.cancel_requested(),
            "the worker observes the client's request"
        );

        handle.complete(serde_json::json!("done")).await.unwrap();
        let late = ops.cancel(&created.op_id).unwrap();
        assert_eq!(late.status, DeferredOpStatus::Completed);
        assert!(
            !late.cancel_requested,
            "there is nothing left to cancel once settled"
        );
        assert!(!handle.cancel_requested());
    }

    #[tokio::test]
    async fn settling_after_expiry_is_refused_not_panicking() {
        let ops = registry(Duration::from_millis(60), 8, 8);
        let (_, handle) = begin(&ops, "payment.submit");
        std::thread::sleep(Duration::from_millis(100));

        assert_eq!(
            handle.complete(serde_json::json!("late")).await,
            Err(DeferredOpsError::Expired)
        );
        assert_eq!(ops.retained_len(), 0);
        assert_eq!(
            ops.pending_len(),
            0,
            "a refused settlement still releases the expired op's slot"
        );
    }
}
