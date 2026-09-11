//! Deferred-operation wire layer — the generic shapes shared by every server
//! that hands a slow upstream call off the dispatch path, by every client
//! that collects the result later, and by the TypeScript bindings generated
//! from this module.
//!
//! # Why this exists
//!
//! A JSON-RPC dispatch is bounded by the serving framework's dispatch stall
//! limit (a **liveness guard measured in seconds**, see
//! `plana-rpc-server`'s crate docs). Any handler whose upstream can outlive
//! that guard — an LLM call, a payment/settlement request, a provider
//! callback — must answer **immediately** with a deferred reference and
//! settle the real result later. Blocking the dispatch instead has two
//! failure modes the fleet has actually hit:
//!
//! 1. the client is answered a structured stall error (`-32603` with
//!    `data.stalled=true` today), and
//! 2. worse, a cancelled *state-changing* dispatch can complete upstream
//!    after cancellation — money or credits spent with nothing returned.
//!
//! # The contract
//!
//! - A deferred-capable handler answers
//!   `{"op_id": "<opaque random id>", "expires_in": <seconds>}` — the
//!   [`DeferredOpCreated`] shape — where `expires_in` is the **remaining
//!   validity measured from creation**, so a client that reconnects inside
//!   the window can still collect the outcome.
//! - Client-facing ids are valid for **10–30 minutes**
//!   ([`MIN_CLIENT_TTL_SECS`] / [`MAX_CLIENT_TTL_SECS`]); the default is 30
//!   minutes because the slowest canonical case (a diagnostic LLM call) is
//!   bounded by minutes, not seconds.
//! - Collection is **non-destructive**: a settled outcome stays collectable
//!   until its TTL elapses — repeatedly — because a lost answer frame must
//!   never cost the caller the work that was already paid for. The one
//!   exception is a server under retention pressure: when its entry cap is
//!   reached it evicts the oldest settled entries, so an uncollected outcome
//!   can disappear *before* its window elapses and the id then answers
//!   `-32052` (bounded memory is the trade-off, and the cap is configurable).
//! - Three methods carry the whole surface, served by the framework so no
//!   service hand-rolls them: [`OPS_RESULT_METHOD`] (`ops.result`, collect),
//!   [`OPS_CANCEL_METHOD`] (`ops.cancel`, best-effort cancellation request)
//!   and [`OPS_SETTLED_METHOD`] (`ops.settled`, an **advisory** settlement
//!   notification on the data lane — never the authoritative path).
//!
//! The id is opaque and unguessable (a random UUID v4 rendered as a string),
//! not a counter: anyone holding an id can collect the outcome, so ids must
//! not be enumerable.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::types::JsonRpcError;

/// Collection method: `ops.result {op_id}` → the current
/// [`DeferredOpOutcome`].
pub const OPS_RESULT_METHOD: &str = "ops.result";

/// Cancellation method: `ops.cancel {op_id}` → [`OpsCancelResult`].
/// Best-effort only: it records a flag the worker may observe.
pub const OPS_CANCEL_METHOD: &str = "ops.cancel";

/// Advisory settlement notification: `ops.settled {op_id, status}`.
///
/// Best-effort — it is dropped when the originating connection is gone, so
/// clients must always be able to fall back to polling `ops.result`. It is
/// also **unordered** with respect to the response that carries the `op_id`:
/// a worker that settles immediately can have its announcement overtake that
/// response on the data lane, so a client must ignore announcements for ids it
/// does not know rather than treating one as the arrival of an id.
pub const OPS_SETTLED_METHOD: &str = "ops.settled";

/// Lower bound of the intended client-facing validity window (10 minutes).
pub const MIN_CLIENT_TTL_SECS: u64 = 10 * 60;

/// Upper bound of the intended client-facing validity window (30 minutes).
pub const MAX_CLIENT_TTL_SECS: u64 = 30 * 60;

/// Default client-facing validity window (30 minutes).
pub const DEFAULT_CLIENT_TTL_SECS: u64 = MAX_CLIENT_TTL_SECS;

/// Client-facing reference to a deferred operation.
///
/// Opaque, unguessable and stable for the whole validity window: the id is
/// the same string before and after a reconnect, so a client (or a different
/// client, e.g. a browser tab resuming a rescue session) can collect with it
/// while the entry is retained — the registry's retention cap can evict a
/// settled outcome early, the id string itself never changes. Treat it as a
/// bearer value — whoever holds it can read the outcome.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS, JsonSchema,
)]
#[ts(export, export_to = "ops.ts")]
pub struct DeferredOpRef(String);

impl DeferredOpRef {
    /// Mint a fresh random reference (UUID v4 — unguessable, not a counter).
    pub fn new_random() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Borrow the opaque id as it appears on the wire.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Adopt an id received from the wire.
    pub fn from_wire(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for DeferredOpRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<DeferredOpRef> for String {
    fn from(op: DeferredOpRef) -> Self {
        op.0
    }
}

/// Lifecycle of a deferred operation.
///
/// A cancellation request does **not** add a state: `ops.cancel` only sets a
/// flag the worker may observe, and a cancelled operation settles as
/// [`DeferredOpStatus::Failed`] with `-32055` (`error_codes::OPS_CANCELLED`)
/// when the worker honours it. That keeps the state machine to the three
/// states a client actually has to handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ops.ts")]
pub enum DeferredOpStatus {
    /// Accepted, the worker has not settled it yet.
    Pending,
    /// Settled with a result.
    Completed,
    /// Settled with an error (including a worker-honoured cancellation).
    Failed,
}

impl DeferredOpStatus {
    /// True once the operation can no longer change state.
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending)
    }

    /// The snake_case wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Immediate answer of a handler that switched to deferred mode.
///
/// `expires_in` is the **remaining** validity in seconds measured from
/// creation, not from the moment of serialization — a client that reconnects
/// half-way through the window sees the shrunk remainder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct DeferredOpCreated {
    /// The opaque client-facing reference.
    pub op_id: DeferredOpRef,
    /// Remaining validity of `op_id`, in seconds.
    pub expires_in: u64,
}

/// Current state of a deferred operation, as answered by `ops.result`.
///
/// Exactly one of `result` / `error` is present once the status is terminal;
/// both are absent while the status is `pending`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct DeferredOpOutcome {
    /// The collected reference (echoed so a multiplexed client can route).
    pub op_id: DeferredOpRef,
    /// Current lifecycle status.
    pub status: DeferredOpStatus,
    /// The method whose handler deferred the work — audit and debugging
    /// surface; the outcome is meaningless without knowing what it is a
    /// result of.
    pub method: String,
    /// Remaining validity of this entry, in seconds. Once it reaches zero the
    /// entry is gone and the id answers `-32053`.
    pub expires_in: u64,
    /// Whether a cancellation was requested through `ops.cancel` while the
    /// operation was still pending (advisory — the worker may not honour it).
    pub cancel_requested: bool,
    /// Present iff `status == completed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub result: Option<Value>,
    /// Present iff `status == failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<JsonRpcError>,
}

impl DeferredOpOutcome {
    /// True once the operation can no longer change state.
    pub fn is_settled(&self) -> bool {
        self.status.is_terminal()
    }
}

/// `ops.result` params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct OpsResultParams {
    /// The reference to collect.
    pub op_id: DeferredOpRef,
}

/// `ops.cancel` params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct OpsCancelParams {
    /// The reference to ask cancellation for.
    pub op_id: DeferredOpRef,
}

/// `ops.cancel` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct OpsCancelResult {
    /// The reference the request was recorded for.
    pub op_id: DeferredOpRef,
    /// The status observed at request time.
    pub status: DeferredOpStatus,
    /// True when the request was recorded for a still-pending operation (so
    /// the worker can observe it); false when the operation had already
    /// settled and there was nothing to cancel.
    pub cancel_requested: bool,
}

/// `ops.settled` notification params.
///
/// Advisory only: it says *that* an operation settled, never *what* it
/// produced — the client still calls `ops.result` to collect the payload.
/// That keeps a single authoritative collection path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ops.ts")]
pub struct DeferredOpSettledParams {
    /// The reference that settled.
    pub op_id: DeferredOpRef,
    /// Its new status.
    pub status: DeferredOpStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn created_answer_uses_the_documented_wire_shape() {
        let created = DeferredOpCreated {
            op_id: DeferredOpRef::from_wire("op-1"),
            expires_in: DEFAULT_CLIENT_TTL_SECS,
        };
        let wire = serde_json::to_value(&created).unwrap();
        assert_eq!(wire, json!({"op_id": "op-1", "expires_in": 1800}));
        let back: DeferredOpCreated = serde_json::from_value(wire).unwrap();
        assert_eq!(back, created);
    }

    #[test]
    fn op_ref_is_transparent_on_the_wire_and_unguessable() {
        let op = DeferredOpRef::new_random();
        // Transparent newtype: the id is a bare JSON string.
        assert_eq!(serde_json::to_value(&op).unwrap(), json!(op.as_str()));
        let back: DeferredOpRef = serde_json::from_value(json!(op.as_str())).unwrap();
        assert_eq!(back, op);

        // Random, not sequential: 32 ids are all distinct and every one is a
        // hyphenated UUID v4 (a counter, a constant or a v7 timestamp would
        // fail one of these three assertions).
        let ids: std::collections::HashSet<String> = (0..32)
            .map(|_| DeferredOpRef::new_random().to_string())
            .collect();
        assert_eq!(ids.len(), 32);
        for id in &ids {
            assert_eq!(id.len(), 36, "uuid v4 hyphenated form: {id}");
            assert_eq!(
                id.chars().nth(14),
                Some('4'),
                "uuid v4 sets the version nibble: {id}"
            );
        }
    }

    #[test]
    fn status_spellings_are_snake_case_and_terminality_is_exact() {
        assert_eq!(
            serde_json::to_value(DeferredOpStatus::Pending).unwrap(),
            json!("pending")
        );
        assert_eq!(
            serde_json::to_value(DeferredOpStatus::Completed).unwrap(),
            json!("completed")
        );
        assert_eq!(
            serde_json::to_value(DeferredOpStatus::Failed).unwrap(),
            json!("failed")
        );
        assert!(!DeferredOpStatus::Pending.is_terminal());
        assert!(DeferredOpStatus::Completed.is_terminal());
        assert!(DeferredOpStatus::Failed.is_terminal());
        for s in [
            DeferredOpStatus::Pending,
            DeferredOpStatus::Completed,
            DeferredOpStatus::Failed,
        ] {
            assert_eq!(serde_json::to_value(s).unwrap(), json!(s.as_str()));
        }
    }

    #[test]
    fn outcome_omits_the_absent_half_of_the_settlement() {
        let pending = DeferredOpOutcome {
            op_id: DeferredOpRef::from_wire("op-2"),
            status: DeferredOpStatus::Pending,
            method: "rescue.diagnose".into(),
            expires_in: 1799,
            cancel_requested: false,
            result: None,
            error: None,
        };
        let wire = serde_json::to_value(&pending).unwrap();
        assert!(wire.get("result").is_none());
        assert!(wire.get("error").is_none());
        assert!(!pending.is_settled());

        let completed = DeferredOpOutcome {
            status: DeferredOpStatus::Completed,
            result: Some(json!({"diagnosis": "ok"})),
            ..pending.clone()
        };
        let wire = serde_json::to_value(&completed).unwrap();
        assert_eq!(wire["status"], "completed");
        assert_eq!(wire["result"]["diagnosis"], "ok");
        assert!(wire.get("error").is_none());
        assert!(completed.is_settled());

        let failed = DeferredOpOutcome {
            status: DeferredOpStatus::Failed,
            result: None,
            error: Some(JsonRpcError::new(
                super::super::error_codes::OPS_CANCELLED,
                "operation cancelled",
            )),
            ..pending
        };
        let wire = serde_json::to_value(&failed).unwrap();
        assert_eq!(wire["status"], "failed");
        assert_eq!(
            wire["error"]["code"],
            super::super::error_codes::OPS_CANCELLED
        );
        assert!(wire.get("result").is_none());
        assert!(failed.is_settled());
    }

    #[test]
    fn ops_params_round_trip_and_reject_a_missing_id() {
        let params: OpsResultParams = serde_json::from_value(json!({"op_id": "op-3"})).unwrap();
        assert_eq!(params.op_id.as_str(), "op-3");

        // A missing op_id is a params error, never a silent default.
        assert!(serde_json::from_value::<OpsResultParams>(json!({})).is_err());

        let cancel = OpsCancelResult {
            op_id: DeferredOpRef::from_wire("op-3"),
            status: DeferredOpStatus::Pending,
            cancel_requested: true,
        };
        assert_eq!(
            serde_json::to_value(&cancel).unwrap(),
            json!({"op_id": "op-3", "status": "pending", "cancel_requested": true})
        );
    }

    #[test]
    fn settled_notification_carries_status_but_never_the_payload() {
        let settled = DeferredOpSettledParams {
            op_id: DeferredOpRef::from_wire("op-4"),
            status: DeferredOpStatus::Completed,
        };
        let wire = serde_json::to_value(&settled).unwrap();
        assert_eq!(
            wire,
            json!({"op_id": "op-4", "status": "completed"}),
            "the notification must stay payload-free: collection is authoritative"
        );
    }

    #[test]
    fn method_names_and_ttl_band_are_the_published_contract() {
        assert_eq!(OPS_RESULT_METHOD, "ops.result");
        assert_eq!(OPS_CANCEL_METHOD, "ops.cancel");
        assert_eq!(OPS_SETTLED_METHOD, "ops.settled");
        assert_eq!(MIN_CLIENT_TTL_SECS, 600);
        assert_eq!(MAX_CLIENT_TTL_SECS, 1800);
        assert_eq!(DEFAULT_CLIENT_TTL_SECS, MAX_CLIENT_TTL_SECS);
    }
}
