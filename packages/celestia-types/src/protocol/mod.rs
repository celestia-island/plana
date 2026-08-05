//! WebSocket transport domain — client capability payloads and the
//! scepter-flavored connection handshake.
//!
//! The generic handshake primitives (handshake version, ack, ping) live in
//! `plana-protocol-core`; this module carries the client-capability
//! vocabulary and the connection payload that references it. The base
//! protocol messages (`base_messages`) are re-exported from
//! `plana-protocol-core` so the domain profile never duplicates them.

pub mod base_messages;
pub mod handshake;
pub mod jsonrpc;
