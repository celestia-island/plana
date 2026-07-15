//! Newline-delimited JSON-RPC transport over a Unix-domain socket.
//!
//! On Unix this re-exports the real [`unix_impl`] implementation built on
//! `tokio::net::UnixStream`. On non-Unix targets (Windows) the socket transport
//! is unused — the scriptum TUI talks to the backend over a WebSocket instead —
//! so we provide a [`stub_impl`] with the same public surface that returns a
//! runtime error if anything is ever called. This keeps the crate compilable on
//! `x86_64-pc-windows-msvc` without dragging in a Unix-only code path.

#[cfg(unix)]
pub mod unix_impl;
#[cfg(unix)]
pub use unix_impl::{
    IncomingMessage, JsonRpcReceiver, JsonRpcSender, JsonRpcServer, JsonRpcTransport, TimeoutPolicy,
};

#[cfg(not(unix))]
pub mod stub_impl;
#[cfg(not(unix))]
pub use stub_impl::{
    IncomingMessage, JsonRpcReceiver, JsonRpcSender, JsonRpcServer, JsonRpcTransport, TimeoutPolicy,
};
