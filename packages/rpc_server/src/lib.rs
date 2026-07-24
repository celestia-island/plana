//! JSON-RPC server module — session management and SSE transport for plana.
//!
//! This crate provides the server-side counterpart to `plana-rpc-client`:
//! session management, SSE event streaming, and RPC handler dispatch.
//!
//! ## Architecture
//! - [`SessionManager`] — manages per-client JSON-RPC sessions
//! - [`sse`] — Server-Sent Events handler with keep-alive heartbeat
//! - [`events`] — typed RPC notifications pushed to clients
//!
//! Pair with `plana-rpc-client` (npm: `@celestia-island/plana-rpc-client`)
//! for the browser-side 4-tier fallback transport.

pub mod sse;
pub mod session;

pub use session::SessionManager;
