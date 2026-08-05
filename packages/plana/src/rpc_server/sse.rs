//! Server-Sent Events handler for JSON-RPC event streaming.
//!
//! Pairs with the browser-side SSE transport in the plana-rpc-client npm
//! package. Sends an immediate `:connected` comment and periodic `:heartbeat`
//! keep-alive frames to prevent proxy/nginx timeout.

/// SSE keep-alive interval in seconds.
pub const SSE_HEARTBEAT_INTERVAL_SECS: u64 = 25;

/// Initial SSE comment sent on connection.
pub const SSE_CONNECTED_COMMENT: &str = ":connected\n\n";

/// SSE heartbeat comment sent periodically.
pub const SSE_HEARTBEAT_COMMENT: &str = ":heartbeat\n\n";
