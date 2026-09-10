//! Transport core — base protocol messages and the connection handshake
//! primitives.
//!
//! The generic JSON-RPC 2.0 envelope has a single canonical definition in
//! `plana-jsonrpc` (`plana_jsonrpc::types`); the former copy here was removed
//! after the two drifted apart. The base messages and handshake types are
//! flat type re-exports at the crate root.

pub mod base_messages;
pub mod handshake;
