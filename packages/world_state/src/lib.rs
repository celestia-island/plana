//! Typed world-state store for physical-consistency entity tracking.
//!
//! Part of the embodied-AI roadmap (root PLAN §8.4/§8.9): while
//! `plana_sync` synchronises *UI* state (untyped `serde_json` trees), this
//! crate tracks the *physical* world — typed entities with attributes, data
//! quality, and dual timestamps — so agents and future world-model services
//! share one queryable "what is, right now".
//!
//! # Design
//!
//! - [`WorldStateStore`] follows the same single-writer + broadcast pattern
//!   as `plana_sync`'s `StateTree`, but stores typed [`WorldEntity`] values
//!   instead of untyped JSON.
//! - Entities carry a wall-clock timestamp (`updated_wall`, for display and
//!   audit) and an optional monotonic capture timestamp (`updated_mono_ns`,
//!   for ordering and sensor fusion).
//! - The first producer is industrial telemetry: [`ingest::apply_telemetry_batch`]
//!   maps `plana` industrial readings onto station/point entities.
//! - [`WorldEventLog`] is the append-only persistence seam (PLAN §8.9); the
//!   store itself is in-memory only.
//!
//! # Semantics notes
//!
//! - **Steady-state liveness**: an upsert whose *attributes* are unchanged
//!   but whose `updated_wall` advanced still counts as a change — the wall
//!   timestamp is observable liveness ("alive, value unchanged" vs "no
//!   data"). Subscribers that only care about value changes must diff
//!   `attributes` themselves.
//! - **Broadcast**: the change channel is a `tokio::broadcast`; a subscriber
//!   that falls more than `capacity` changes behind receives `Lagged` and
//!   should re-read [`WorldStateStore::snapshot`]. Broadcast version
//!   ordering is only guaranteed for a single writer thread.
//! - **NaN**: `AttributeValue::Number(f64)` does not survive NaN (JSON has
//!   no NaN — it serialises to `null` and reads back as `Json(null)`);
//!   producers should map non-finite values to [`Quality::Error`] instead.

pub mod ingest;
pub mod store;
pub mod types;

pub use ingest::apply_telemetry_batch;
pub use store::{WorldChange, WorldEventLog, WorldStateStore};
pub use types::{AttributeValue, EntityId, EntityKind, Quality, WorldEntity, WorldRelation};
