//! PLANA — Protocol for Live Agent Network Automation: a typed
//! application-layer protocol for real-time state synchronization and
//! control between a client shell and a backend service runtime, built
//! on JSON-RPC 2.0 (the way HTTP is built on TCP). Not a general-purpose
//! RPC framework.
//!
//! This crate is the **protocol foundation**: the JSON-RPC 2.0 wire layer
//! ([`jsonrpc`]) and the generic protocol core ([`protocol_core`]) live
//! here directly. The former standalone crates `plana-jsonrpc` and
//! `plana-protocol-core` remain in the workspace as one-line re-export
//! shims so existing git-pin consumers keep compiling while they migrate
//! to the merged paths.
//!
//! Layering (no cycles): everything below may be depended on by the
//! upper crates, never the reverse:
//!
//! - `plana` (this crate) — envelope + protocol semantics.
//! - `plana-rpc-server` / `plana-rpc-client` — the service-profile
//!   server framework and client (+ the relay extension).
//! - `plana-tauri` — the Tauri adapter and standard `relay.*` battery.
//! - `plana-celestia-types` — the celestia platform domain profile
//!   (depends on this crate; NOT re-exported here — the former
//!   `celestia` facade feature is gone, depend on it directly).
//! - `plana-evernight-client` — the terminal-route dispatch client
//!   (depends on this crate; likewise depend on it directly).
//!
//! The `rpc-server` feature adds this crate's own axum mounting module
//! ([`rpc_server`]) — geo-aware network descriptors and SSE lanes.

pub mod jsonrpc;
pub mod protocol_core;

// Root convenience: the generic protocol core's surface stays reachable
// at the crate root exactly as the former umbrella re-exported it.
#[cfg(feature = "tracing-helpers")]
pub use protocol_core::tracing_helpers;
pub use protocol_core::*;

#[cfg(feature = "rpc-server")]
pub mod rpc_server;
