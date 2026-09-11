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
//!   any number of subscribers;
//! - **deferred operations** — [`RpcClient::await_op`] collects the outcome of
//!   a deferred operation (notification-first, `ops.result` polling as the
//!   fallback) towards a caller-supplied deadline. This half is Rust-only for
//!   now: the published TypeScript client has no equivalent helper yet.
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
pub use plana::jsonrpc::deferred::{
    DeferredOpCreated, DeferredOpOutcome, DeferredOpRef, DeferredOpStatus, OPS_CANCEL_METHOD,
    OPS_RESULT_METHOD, OPS_SETTLED_METHOD,
};
pub use plana::jsonrpc::{Id, JsonRpcError};

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

/// Everything that can go wrong while awaiting a deferred operation.
///
/// The three terminal conditions are spelled out separately because they
/// call for different reactions: `Unknown`/`Expired` mean the outcome is
/// gone for good (re-issue the request, or ask the user to retry), while
/// `Deadline` means the operation is still running on the server and the
/// **same** `op_id` can be collected again later.
#[derive(Debug, thiserror::Error)]
pub enum DeferredOpError {
    /// The server does not know this id: never issued, or evicted **before**
    /// its window elapsed by the server's retention cap (`-32052`). Eviction
    /// is the one case where a settled outcome can vanish early, so a caller
    /// that receives this after an `ops.settled` announcement should treat the
    /// payload as lost rather than retry forever.
    #[error("unknown deferred operation")]
    Unknown,
    /// The id was issued and its validity window elapsed before collection
    /// (`-32053`).
    #[error("deferred operation expired")]
    Expired,
    /// The caller's deadline elapsed first; the operation may still settle —
    /// collect again with the same id while it is valid.
    #[error("deferred operation not settled within {0:?}")]
    Deadline(std::time::Duration),
    /// The server answered `ops.result` with a payload this client cannot
    /// read (version skew, or a service that overrode the method). Permanent:
    /// retrying the same call cannot help.
    #[error("ops.result answered an unreadable outcome: {0}")]
    Protocol(String),
    /// Transport or protocol failure while collecting.
    #[error(transparent)]
    Rpc(#[from] RpcError),
}

/// Heartbeat method names, shared with `plana-rpc-server`.
pub mod close_codes {
    pub const HEARTBEAT_METHOD: &str = "Base.Heartbeat";
    pub const HEARTBEAT_ACK_METHOD: &str = "Base.HeartbeatAck";
}
