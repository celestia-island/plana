//! Production-grade strict WebSocket JSON-RPC 2.0 server for the PLANA
//! service profile.
//!
//! This crate turns the minimal [`plana_jsonrpc::rpc_router`] skeleton into
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

pub mod auth;
pub mod close_codes;
pub mod config;
pub mod connection;
pub mod server;

pub use auth::{AuthContext, ConnectionAuthFn, RequestGuardFn};
pub use close_codes::{
    HEARTBEAT_ACK_METHOD, HEARTBEAT_METHOD, IDLE_TIMEOUT, INTERNAL_ERROR, NORMAL, POLICY_VIOLATION,
    UNSUPPORTED_DATA,
};
pub use config::RpcServerConfig;
pub use connection::{LaneClosed, OutboundHandle};
pub use server::{ext_error_codes, RpcRequestCtx, RpcServer, RpcServerBuilder, ServerHandlerFn};

/// Protocol constants shared by servers built on this crate.
pub mod protocol {
    pub use crate::close_codes::{
        HEARTBEAT_ACK_METHOD, HEARTBEAT_METHOD, IDLE_TIMEOUT, INTERNAL_ERROR, NORMAL,
        POLICY_VIOLATION, UNSUPPORTED_DATA,
    };
    pub use crate::server::ext_error_codes;
}
