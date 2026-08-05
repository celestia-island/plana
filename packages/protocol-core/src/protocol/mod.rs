//! Transport core — JSON-RPC 2.0 envelope, base protocol messages, and the
//! connection handshake primitives.
//!
//! `jsonrpc` stays reachable at the crate root (`plana_protocol_core::jsonrpc`);
//! the `plana` umbrella crate shadows that path with its own
//! `plana::jsonrpc` module re-exporting `plana-jsonrpc`. The base messages
//! and handshake types are flat type re-exports at the crate root.

pub mod base_messages;
pub mod handshake;
pub mod jsonrpc;
