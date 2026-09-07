//! Server tuning knobs.

use std::time::Duration;

/// Limits and timings for an [`crate::RpcServer`].
///
/// Defaults follow fleet parity: scepter's connection cap and frame/message
/// budgets, chest's WS-bridge data-lane capacity and 8s dispatch stall
/// limit, and an idle window sized as ~3x the 15s heartbeat cadence of
/// `@celestia-island/plana-rpc-client`.
#[derive(Debug, Clone)]
pub struct RpcServerConfig {
    /// Maximum number of concurrently open WebSocket connections. Additional
    /// upgrades are refused with HTTP 429 before the handshake.
    pub max_connections: usize,

    /// Maximum encoded WebSocket message size in bytes (default 4 MiB).
    pub max_message_bytes: usize,

    /// Maximum single WebSocket frame size in bytes (default 1 MiB).
    pub max_frame_bytes: usize,

    /// Close the connection with code 4000 when no inbound frame — including
    /// heartbeats — arrives within this window (default 45s).
    pub idle_timeout: Duration,

    /// Cancel and structurally error a dispatch that runs longer than this
    /// (default 8s, below the client's 10s ack window so a stalled handler
    /// does not read as a dead connection). Handlers must be
    /// cancellation-safe.
    pub dispatch_stall_limit: Duration,

    /// Bounded capacity of the data lane carrying responses and
    /// handler-initiated notifications (default 64). Backpressure propagates
    /// to the read loop when full; the control lane is unbounded.
    pub data_lane_capacity: usize,

    /// Serve the built-in `Base.Heartbeat` → `Base.HeartbeatAck` service on
    /// the control lane (default on).
    pub heartbeat: bool,
}

impl Default for RpcServerConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            max_message_bytes: 4 * 1024 * 1024,
            max_frame_bytes: 1024 * 1024,
            idle_timeout: Duration::from_secs(45),
            dispatch_stall_limit: Duration::from_secs(8),
            data_lane_capacity: 64,
            heartbeat: true,
        }
    }
}
