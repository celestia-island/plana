//! Windows / non-Unix stub for the Unix-socket JSON-RPC transport.
//!
//! The Unix-domain-socket transport is fundamentally Unix-only: it is built on
//! `tokio::net::UnixStream` / `tokio::net::UnixListener`, neither of which exists
//! on Windows. On Windows the scriptum TUI reaches the backend over a WebSocket
//! instead, so this transport is never actually used. To keep the crate
//! compilable on `x86_64-pc-windows-msvc` (so that the shared `types`/`bridge`/
//! `json_keys` modules remain usable) we expose the *same public type names* as
//! [`super::unix_impl`] does on Unix, but every constructor returns an error and
//! every method is a no-op/error. Nothing on Windows should ever reach this
//! code; if it does, the error makes the cause obvious.

use std::path::Path;

use anyhow::{Result, bail};

/// Placeholder transport type. Cannot be constructed on non-Unix targets.
pub struct JsonRpcTransport {
    _private: (),
}

/// Placeholder sender type. Cannot be constructed on non-Unix targets.
pub struct JsonRpcSender {
    _private: (),
}

/// Placeholder receiver type. Cannot be constructed on non-Unix targets.
pub struct JsonRpcReceiver {
    _private: (),
}

/// Placeholder server type. Cannot be constructed on non-Unix targets.
pub struct JsonRpcServer {
    _private: (),
}

/// Timeout policy — a pure configuration enum, so it is fully usable on Windows
/// (e.g. when constructing request values shared with the Unix side).
#[derive(Debug, Clone, Copy)]
pub enum TimeoutPolicy {
    Default,
    Persistent,
    Indefinite,
    Deadline(std::time::Instant),
}

/// Incoming message envelope. Mirrors the Unix variant so that code sharing
/// message-handling logic can pattern-match on it.
#[derive(Debug)]
pub enum IncomingMessage {
    Request(crate::types::JsonRpcRequest),
    Notification(crate::types::JsonRpcNotification),
    Response(crate::types::JsonRpcResponse),
}

impl JsonRpcTransport {
    pub async fn connect(_socket_path: &Path) -> Result<Self> {
        bail!(
            "Unix-domain-socket JSON-RPC transport is not available on this platform \
             (use the WebSocket transport instead)"
        );
    }
}

impl JsonRpcServer {
    pub async fn bind(_socket_path: &Path) -> Result<Self> {
        bail!(
            "Unix-domain-socket JSON-RPC server is not available on this platform \
             (use the WebSocket transport instead)"
        );
    }
}
