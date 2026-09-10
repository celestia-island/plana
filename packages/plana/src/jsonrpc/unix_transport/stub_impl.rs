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

use anyhow::{bail, Result};

use crate::jsonrpc::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

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
    Request(crate::jsonrpc::types::JsonRpcRequest),
    Notification(crate::jsonrpc::types::JsonRpcNotification),
    Response(crate::jsonrpc::types::JsonRpcResponse),
}

fn platform_error() -> anyhow::Error {
    anyhow::anyhow!(
        "Unix-domain-socket JSON-RPC transport is not available on this platform \
         (use the WebSocket transport instead)"
    )
}

impl JsonRpcTransport {
    pub async fn connect(_socket_path: &Path) -> Result<Self> {
        bail!(platform_error());
    }

    pub fn split(self) -> (JsonRpcSender, JsonRpcReceiver) {
        (
            JsonRpcSender { _private: () },
            JsonRpcReceiver { _private: () },
        )
    }

    pub async fn send(
        &mut self,
        _request: &JsonRpcRequest,
        _policy: TimeoutPolicy,
    ) -> Result<crate::jsonrpc::types::JsonRpcResponse> {
        bail!(platform_error());
    }

    pub async fn send_notification(&mut self, _notification: &JsonRpcNotification) -> Result<()> {
        bail!(platform_error());
    }

    pub async fn send_response(&mut self, _response: &JsonRpcResponse) -> Result<()> {
        bail!(platform_error());
    }

    pub async fn send_raw(&mut self, _text: &str) -> Result<()> {
        bail!(platform_error());
    }

    pub async fn receive(&mut self) -> Result<Option<IncomingMessage>> {
        Ok(None)
    }
}

impl JsonRpcSender {
    pub async fn send_response(&mut self, _response: &JsonRpcResponse) -> Result<()> {
        bail!(platform_error());
    }

    pub async fn send_notification(&mut self, _notification: &JsonRpcNotification) -> Result<()> {
        bail!(platform_error());
    }

    pub async fn send_request(&mut self, _request: &JsonRpcRequest) -> Result<()> {
        bail!(platform_error());
    }
}

impl JsonRpcServer {
    pub async fn bind(_socket_path: &Path) -> Result<Self> {
        bail!(platform_error());
    }

    pub async fn accept(&self) -> Result<JsonRpcTransport> {
        bail!(platform_error());
    }
}
