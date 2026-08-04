//! JSON-RPC server module — session management and SSE transport.
//!
//! This module is the server-side counterpart to a JSON-RPC client layer:
//! session management, SSE event streaming, and RPC handler dispatch.
//!
//! ## Architecture
//! - [`SessionManager`] — manages per-client JSON-RPC sessions
//! - [`sse`] — Server-Sent Events handler with keep-alive heartbeat
//! - [`network`] — request transport/geolocation detection
//!   ([`detect_network`]) for populating connection metadata

pub mod network;
pub mod session;
pub mod sse;

pub use network::detect_network;
pub use session::SessionManager;
