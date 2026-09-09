//! Rust client for the PLANA service profile (strict WebSocket JSON-RPC
//! 2.0), mirroring `@celestia-island/plana-rpc-client` on the TypeScript
//! side.
//!
//! [`RpcClient`] maintains one persistent connection with:
//!
//! - **id correlation** — request ids are UUID v7 strings; responses are
//!   matched against in-flight calls and delivered to exactly one caller;
//! - **call timeouts** — every [`RpcClient::call`] is bounded independently
//!   of transport health;
//! - **heartbeat keepalive** — `Base.Heartbeat` notifications at a fixed
//!   cadence; if no inbound frame (ack or otherwise) arrives within the
//!   watchdog window, the connection is considered dead and cycles;
//! - **reconnect with exponential backoff** — dropped connections re-dial
//!   with 1.5x growth capped at 30s; pending calls are rejected (not
//!   silently retried) at every disconnect, matching the TS client;
//! - **connection-state watch** — [`RpcClient::state`] /
//!   [`RpcClient::subscribe_state`] for UIs and supervisors;
//! - **notification fan-out** — server-initiated notifications broadcast to
//!   any number of subscribers.
//!
//! Transport scope of this first release: plaintext `ws://`. TLS (`wss://`)
//! and the degraded-transport racing of the TS client are deliberately
//! deferred until the first production consumer needs them; the HTTP POST
//! fallback exists as a standalone one-shot transport in [`http`]
//! (`http://` only) for degraded environments.

pub mod client;
pub mod http;
pub mod relay;

pub use client::{RpcClient, RpcClientBuilder, RpcClientConfig};
pub use http::post_rpc;
pub use plana_jsonrpc::{Id, JsonRpcError};

/// Connection lifecycle states surfaced by [`RpcClient::state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Dialing the first (or a replacement) connection.
    Connecting,
    /// A connection is open and frames flow.
    Connected,
    /// The connection was lost; backing off before the next attempt.
    Reconnecting,
    /// The reconnect budget is exhausted; the client will not dial again.
    /// Calls fail fast until a new client is built.
    Failed,
}

/// Everything that can go wrong on a client call.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Socket-level failure (connect, read, write).
    #[error("transport failure: {0}")]
    Transport(String),
    /// The call did not complete within its timeout.
    #[error("call timed out after {0:?}")]
    Timeout(std::time::Duration),
    /// The server answered with a JSON-RPC error object.
    #[error("jsonrpc error {code}: {message}")]
    Rpc {
        code: i64,
        message: String,
        data: Option<serde_json::Value>,
    },
    /// No usable connection (offline, reconnecting, or reconnect budget
    /// exhausted). Retry at the call site.
    #[error("connection closed")]
    Closed,
}

impl From<JsonRpcError> for RpcError {
    fn from(err: JsonRpcError) -> Self {
        RpcError::Rpc {
            code: err.code,
            message: err.message,
            data: err.data,
        }
    }
}

/// Heartbeat method names, shared with `plana-rpc-server`.
pub mod close_codes {
    pub const HEARTBEAT_METHOD: &str = "Base.Heartbeat";
    pub const HEARTBEAT_ACK_METHOD: &str = "Base.HeartbeatAck";
}
