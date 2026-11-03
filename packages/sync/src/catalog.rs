//! CatalogSync — the multi-writer catalog replication protocol (B2).
//!
//! Design: `_reports/catalog-sync-protocol-20261103/DESIGN.md`
//! (workspace-side). This module is the **wire layer**: the op shapes,
//! the method names and the session handshake types shared by the hub
//! (arona) and every spoke (scepter, chest, future replicas). The
//! convergence mathematics live in `yuuka::crdt` (Dvv / LwwRegister /
//! OrSet — merge is a semilattice join), so this layer only carries
//! *intent + causal dots*, never conflict resolution.
//!
//! Topology: **hub-authoritative with write-back** (user ruling
//! 2026-11-03). arona is the sole durable authority (its DB rows are
//! the materialized projection of the CRDT state); spokes hold
//! full replicas, may write while offline, and converge through
//! anti-entropy (`Delta`) in both directions — any participant can
//! join at any time (`Bootstrap`) and heal any gap by exchanging
//! "what you're missing" vectors.
//!
//! Method family (JSON-RPC; rides the same WS/SSE transport as
//! `Sync.*`):
//!
//! | method | direction | payload |
//! |---|---|---|
//! | `catalog.bootstrap` | spoke → hub | `→ {state, hub_dvv, gc_watermark}` |
//! | `catalog.delta` | both | `{since_dvv} → {ops, need}` |
//! | `catalog.op` | hub → spoke | `{op}` (hot push after hub merge) |
//! | `catalog.commit` | spoke → hub | `{ops: [op]}` (offline write-back) |
//! | `catalog.gc` | hub → spoke | `{watermark}` (tombstone compaction) |
//!
//! Delivery is at-least-once and MAY reorder freely: every op carries
//! a `Dot` (unique per writer session), and the receiving CRDT dedups
//! by dot — exactly-once transport is not required.

use serde::{Deserialize, Serialize};
use yuuka::crdt::{Dot, Dvv};

// ───────────────────────── wire methods ─────────────────────────

/// JSON-RPC method names (snake_case request side, `CatalogSync.*`
/// notification side, matching the `Sync.*` family convention).
pub const RPC_CATALOG_BOOTSTRAP: &str = "catalog.bootstrap";
pub const RPC_CATALOG_DELTA: &str = "catalog.delta";
pub const RPC_CATALOG_COMMIT: &str = "catalog.commit";
pub const METHOD_CATALOG_OP: &str = "CatalogSync.Op";
pub const METHOD_CATALOG_GC: &str = "CatalogSync.Gc";

// ───────────────────────── op shapes ─────────────────────────

/// One replicated catalog intent, stamped with its causal dot.
/// The body names WHAT changed; the CRDT layer decides how concurrent
/// bodies merge (field-level LWW for upserts, add-wins OR-Set for
/// existence).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogOp {
    pub dot: Dot,
    pub body: CatalogOpBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CatalogOpBody {
    /// Insert or update a provider row. `fields` carries the LWW
    /// register writes (endpoint, priority, enabled, protocol, …);
    /// absent fields are "no write this op", not "clear".
    UpsertProvider {
        /// The provider's stable string id (its catalog name).
        id: String,
        fields: ProviderFields,
    },
    /// Tombstone the provider's currently-visible add dots (remove
    /// intents stay traceable in the OR-Set — add-wins on concurrency).
    RemoveProvider {
        id: String,
    },
    UpsertModel {
        /// The model's stable catalog id.
        id: String,
        /// Owning provider string id.
        provider: String,
        fields: ModelFields,
    },
    RemoveModel {
        id: String,
    },
}

/// The LWW-writable provider fields. Every value carries its stamp
/// (ts + session) so the register merge is self-contained on the wire.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProviderFields {
    pub endpoint: Option<Stamped<String>>,
    pub priority: Option<Stamped<i64>>,
    pub enabled: Option<Stamped<bool>>,
    pub protocol: Option<Stamped<String>>,
    /// Sealed key material — hub-only at rest; spokes receive it
    /// re-sealed per receiver (design §3). Absent = "no key write".
    pub api_key_sealed: Option<Stamped<String>>,
}

/// The LWW-writable model fields.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelFields {
    pub api_model: Option<Stamped<String>>,
    pub tier: Option<Stamped<String>>,
    pub priority: Option<Stamped<i64>>,
    pub enabled: Option<Stamped<bool>>,
}

/// A value plus its LWW stamp. The tie-break total order is
/// `(ts, session)` — see `yuuka::crdt::LwwRegister`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stamped<T> {
    pub value: T,
    pub ts: u64,
    pub session: String,
}

// ───────────────────────── handshake shapes ─────────────────────────

/// `catalog.bootstrap` response: the full CRDT state a newcomer folds
/// locally, the hub's causal context, and the tombstone-compaction
/// point (dots covered by the watermark may already be dropped).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogBootstrap {
    /// Serialized CRDT state (the `CatalogCrdt` snapshot — providers
    /// map, models map, tombstones). Kept as `serde_json::Value` so
    /// this layer does not pin the CRDT's internal layout.
    pub state: serde_json::Value,
    pub hub_dvv: Dvv,
    pub gc_watermark: Dvv,
}

/// `catalog.delta` request/response — the anti-entropy exchange. The
/// caller sends its causal context; the peer answers with every op the
/// caller's context does not cover, plus (hub side) the context gaps
/// it wants back. Both directions of the same method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogDeltaRequest {
    pub since: Dvv,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogDeltaResponse {
    pub ops: Vec<CatalogOp>,
    /// The peer's own context, so the caller can answer the reverse
    /// question next round (`None` on the hub — it never requests
    /// spoke-held ops it already merged).
    pub need: Option<Dvv>,
}

/// `catalog.commit` request: a spoke's offline op buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogCommit {
    pub ops: Vec<CatalogOp>,
    /// The spoke's context AFTER applying its buffered ops — the hub
    /// folds this into its own view to keep `Delta` answers exact.
    pub context: Dvv,
}

/// `CatalogSync.Gc` notification: tombstones covered by the watermark
/// are stable; spokes may drop them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogGc {
    pub watermark: Dvv,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(n: u64) -> Dot {
        Dot::new("s-test", n)
    }

    /// The wire claim: every shape round-trips through
    /// `serde_json::Value` (the transport is JSON-RPC).
    #[test]
    fn wire_shapes_round_trip() {
        let op = CatalogOp {
            dot: dot(1),
            body: CatalogOpBody::UpsertProvider {
                id: "llama-local".into(),
                fields: ProviderFields {
                    enabled: Some(Stamped {
                        value: true,
                        ts: 7,
                        session: "s-test".into(),
                    }),
                    priority: Some(Stamped {
                        value: -1,
                        ts: 7,
                        session: "s-test".into(),
                    }),
                    ..Default::default()
                },
            },
        };
        let json = serde_json::to_value(&op).unwrap();
        let back: CatalogOp = serde_json::from_value(json).unwrap();
        assert_eq!(back, op);

        let commit = CatalogCommit {
            ops: vec![op],
            context: Dvv::new(),
        };
        let back: CatalogCommit =
            serde_json::from_value(serde_json::to_value(&commit).unwrap()).unwrap();
        assert_eq!(back, commit);

        let delta = CatalogDeltaResponse {
            ops: vec![],
            need: Some(Dvv::new()),
        };
        let back: CatalogDeltaResponse =
            serde_json::from_value(serde_json::to_value(&delta).unwrap()).unwrap();
        assert_eq!(back, delta);

        let gc = CatalogGc {
            watermark: Dvv::new(),
        };
        let back: CatalogGc = serde_json::from_value(serde_json::to_value(&gc).unwrap()).unwrap();
        assert_eq!(back, gc);
    }

    /// Absent fields mean "no write": a default `ProviderFields` is a
    /// pure existence op (the OR-Set add) with no register writes.
    #[test]
    fn default_fields_are_no_writes() {
        let f = ProviderFields::default();
        assert!(
            f.endpoint.is_none()
                && f.priority.is_none()
                && f.enabled.is_none()
                && f.protocol.is_none()
                && f.api_key_sealed.is_none()
        );
    }
}
