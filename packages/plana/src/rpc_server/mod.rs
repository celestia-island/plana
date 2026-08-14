//! JSON-RPC server module — SSE transport and network detection.
//!
//! This module is the server-side counterpart to a JSON-RPC client layer:
//! SSE event streaming and RPC handler dispatch.
//!
//! ## Architecture
//! - [`sse`] — Server-Sent Events handler with keep-alive heartbeat
//! - [`network`] — request transport/geolocation detection
//!   ([`detect_network`]) for populating connection metadata

pub mod network;
pub mod sse;

pub use network::detect_network;
