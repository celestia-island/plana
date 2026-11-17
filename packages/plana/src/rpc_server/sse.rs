//! Server-Sent Events handler for JSON-RPC event streaming.
//!
//! Pairs with the browser-side SSE transport in the plana-rpc-client npm
//! package. Sends an immediate `:connected` comment and periodic keep-alive
//! frames to prevent proxy/nginx timeout. The periodic frame is ONE SSE
//! event carrying the `:heartbeat` comment AND the `Base.Heartbeat`
//! JSON-RPC notification as data: comments alone keep proxies fed but do
//! not reset the data-frame idle timers of agent-style clients (the
//! 2026-11-17 newapi incident). The wiring lives in
//! [`crate::jsonrpc::session::sse_event_stream`].

/// SSE keep-alive interval in seconds. Defined on the ungated
/// `jsonrpc::session` side (the heartbeat wiring lives there) and
/// re-exported here for the client-facing surface.
pub use crate::jsonrpc::session::SSE_HEARTBEAT_INTERVAL_SECS;

/// Initial SSE comment sent on connection.
pub const SSE_CONNECTED_COMMENT: &str = ":connected\n\n";

/// Comment half of the periodic keep-alive beat. The same SSE event also
/// carries the `Base.Heartbeat` notification as its data payload (see
/// `crate::jsonrpc::session::sse_event_stream`).
pub const SSE_HEARTBEAT_COMMENT: &str = ":heartbeat\n\n";
