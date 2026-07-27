//! WebSocket transport core — JSON-RPC 2.0 envelope, base protocol messages,
//! and the connection handshake.
//!
//! `jsonrpc` stays reachable at `arona::jsonrpc` (re-exported as a module from
//! the crate root) so existing deep-path consumers keep working; the base
//! messages and handshake types are flat type re-exports at the crate root.

pub mod base_messages;
pub mod handshake;
pub mod jsonrpc;
