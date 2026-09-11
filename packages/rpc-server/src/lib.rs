//! Production-grade strict WebSocket JSON-RPC 2.0 server for the PLANA
//! service profile.
//!
//! This crate turns the minimal [`plana::jsonrpc::rpc_router`] skeleton into
//! the server shape the celestia fleet has converged on (evernight host
//! agent `/ws`, shittim-chest WS bridge hardening):
//!
//! - **Strict request/response semantics** — one JSON-RPC object per text
//!   frame, `id` correlated by the caller. Notifications are honored for the
//!   heartbeat service only; client-initiated notification handlers and
//!   batch inputs are rejected by the profile (see `docs/en/rpc/service-profile.md`).
//! - **Control-lane writer** — responses ride a bounded data lane while
//!   heartbeat acks ride an unbounded control lane drained with a biased
//!   `select!`, so a saturated data lane can never stall keepalive traffic.
//! - **Dispatch stall watchdog** — a handler that exceeds
//!   [`RpcServerConfig::dispatch_stall_limit`] is cancelled and answered
//!   with a structured stall error instead of holding the connection
//!   hostage. Handlers MUST therefore be cancellation-safe.
//! - **Deferred operations** — a first-class primitive for handlers whose
//!   upstream cannot be bounded in seconds: answer immediately with an
//!   opaque op ref plus its validity window, settle later, collect through
//!   the built-in `ops.result` / `ops.cancel` methods ([`DeferredOps`]).
//! - **Connection admission** — a semaphore caps concurrent connections;
//!   upgrades beyond the cap are refused before the WebSocket handshake.
//! - **Per-connection auth + per-request guard** — the auth hook runs once
//!   at upgrade time (HTTP 401 on failure), the guard runs before every
//!   dispatch (JSON-RPC `-32005`-style error on failure).
//! - **HTTP POST fallback** — the same method map is mounted for plain
//!   HTTP POST on the same path, matching `@celestia-island/plana-rpc-client`'s
//!   degraded transport. Handler-initiated notifications are dropped on
//!   this transport.
//!
//! The profile this crate implements is the open protocol surface: any
//! party can build a compatible server on top of it, and hosted celestia
//! services (e.g. gateway.celestia.world) are reference deployments.
//!
//! # Latency policy: the dispatch stall limit is a liveness guard, never an
//! # upstream budget
//!
//! [`RpcServerConfig::dispatch_stall_limit`] (default 8s) exists to kill a
//! wedged or runaway dispatch and keep a connection responsive. It is **not**
//! a statement about how long an upstream may take, and it is not the knob to
//! turn when an upstream is slower than it:
//!
//! - **Any handler whose upstream can exceed the stall limit must answer with
//!   a deferred operation** — an op ref plus `expires_in`, returned
//!   immediately — and settle the real outcome later through [`DeferredOps`]
//!   (`RpcRequestCtx::defer` for the whole pattern in one call). Answering
//!   `-32051 dispatch stalled` to a caller whose request was actually in
//!   flight is a protocol failure, not backpressure.
//! - **Canonical cases**: payment / settlement requests, provider callbacks,
//!   and LLM calls. Anything metered, billed or otherwise state-changing on
//!   the upstream is a canonical case *even if it usually returns fast* —
//!   because a stall during a state-changing call means the upstream can
//!   complete **after** the dispatch was cancelled: money or quota spent with
//!   nothing returned to the caller. Deferring removes that class of loss.
//! - **Client-facing ids are valid for 10–30 minutes** (default 30, the band
//!   published in `plana::jsonrpc::deferred`), measured from creation so a
//!   client that reconnects inside the window still collects.
//! - **Do not raise the stall limit to accommodate a slow upstream.** It is
//!   per-dispatch liveness; a larger value just makes a wedged handler hold a
//!   connection (and the client's ack window) hostage for longer.
//!
//! Anything that fits comfortably inside the stall limit — a database read, a
//! cache lookup, an in-process computation — stays on the normal dispatch
//! path; deferring is for latency you do not own.

pub mod auth;
pub mod close_codes;
pub mod config;
pub mod connection;
pub mod deferred;
pub mod server;

pub use auth::{AuthContext, ConnectionAuthFn, RequestGuardFn};
pub use close_codes::{
    HEARTBEAT_ACK_METHOD, HEARTBEAT_METHOD, IDLE_TIMEOUT, INTERNAL_ERROR, NORMAL, POLICY_VIOLATION,
    UNSUPPORTED_DATA,
};
pub use config::RpcServerConfig;
pub use connection::{LaneClosed, OutboundHandle};
pub use deferred::{DeferredOps, DeferredOpsConfig, DeferredOpsError, OpHandle};
pub use server::{ext_error_codes, RpcRequestCtx, RpcServer, RpcServerBuilder, ServerHandlerFn};

/// Protocol constants shared by servers built on this crate.
pub mod protocol {
    pub use crate::close_codes::{
        HEARTBEAT_ACK_METHOD, HEARTBEAT_METHOD, IDLE_TIMEOUT, INTERNAL_ERROR, NORMAL,
        POLICY_VIOLATION, UNSUPPORTED_DATA,
    };
    pub use crate::server::ext_error_codes;

    /// Deferred-operation methods served by this crate, re-exported so a
    /// client-side consumer can name them without depending on the wire
    /// crate directly.
    pub use plana::jsonrpc::deferred::{
        DEFAULT_CLIENT_TTL_SECS, MAX_CLIENT_TTL_SECS, MIN_CLIENT_TTL_SECS, OPS_CANCEL_METHOD,
        OPS_RESULT_METHOD, OPS_SETTLED_METHOD,
    };
}
