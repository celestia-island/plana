//! # arona-supervisor
//!
//! Unified supervision / rolling-update / replication runtime shared by
//! entelecheia (scepter), shittim-chest (chest) and evernight.
//!
//! This crate is the *runtime* counterpart to the protocol types in
//! [`arona::lifecycle`]. See
//! `docs/<lang>/design/platform/supervision-and-rolling-update.md` for the
//! full design.
//!
//! ## Layers
//!
//! - **Layer 1 — lifecycle** ([`lifecycle`], [`probes`]): uniform signal
//!   semantics (`SIGTERM`=drain / `SIGHUP`=reload / `SIGQUIT`=immediate),
//!   a drain controller, and split `/healthz` + `/readyz` probes with a
//!   drain bit.
//! - **Worker supervision** ([`worker`]): [`WorkerSpec`] + [`Supervisor`]
//!   — restart a crashed child-process resource per OTP-style policy
//!   (permanent/transient/temporary) with sliding-window rate limiting.
//! - **Layer 3 — listener handoff** ([`listener`]): [`acquire_listener`]
//!   prefers systemd socket activation (pure-Rust) and falls back to a
//!   plain bind.
//! - **Layer 2 — coordination** ([`lock`]): [`CoordinationLock`] trait
//!   with file / pg / lease backends.
//! - **Subsystem A / B** ([`replica`] / [`leader`], feature-gated): peer
//!   replicas vs leader/follower, built on the primitives above.

pub mod lifecycle;
pub mod listener;
pub mod lock;
pub mod probes;
pub mod worker;

#[cfg(feature = "replica")]
pub mod replica;
#[cfg(feature = "leader-follower")]
pub mod leader;

pub use lifecycle::{DrainController, ShutdownKind};
pub use listener::acquire_listener;
pub use lock::{CoordinationLock, LockError, LockGuard};
pub use probes::{probe_router, ProbeState};
pub use worker::{Supervisor, WorkerSpec};
