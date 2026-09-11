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
    ///
    /// **This is a liveness guard measured in seconds, never an upstream
    /// budget.** It exists to kill a wedged or runaway dispatch, not to bound
    /// how long an upstream may take: a handler whose upstream can outlive
    /// this window MUST answer immediately with a deferred op ref plus
    /// `expires_in` (`RpcRequestCtx::defer` / [`crate::DeferredOps`]) and
    /// settle the outcome later through `ops.result`. Raising this value is
    /// not the fix — a stall during a state-changing call can complete
    /// upstream and then be cancelled on the wire, spending the money or the
    /// quota with nothing returned. See the crate docs for the full policy.
    pub dispatch_stall_limit: Duration,

    /// Deferred-operation registry tuning: validity window, pending cap and
    /// retention cap for handlers that answer with an op ref instead of
    /// blocking the dispatch path (see [`crate::DeferredOpsConfig`]).
    pub deferred_ops: crate::deferred::DeferredOpsConfig,

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
            deferred_ops: crate::deferred::DeferredOpsConfig::default(),
            data_lane_capacity: 64,
            heartbeat: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fleet baseline these values encode is a contract, not a taste:
    /// the stall limit is a liveness guard (raising it to fit a slow upstream
    /// is exactly the mistake the deferred-op primitive exists to remove),
    /// and the client-facing deferred window must stay in the published
    /// 10–30 minute band.
    #[test]
    fn defaults_pin_the_fleet_baseline() {
        let config = RpcServerConfig::default();
        assert_eq!(
            config.dispatch_stall_limit,
            Duration::from_secs(8),
            "the 8s stall limit is a liveness guard and must not be raised"
        );
        assert!(config.dispatch_stall_limit < Duration::from_secs(60));
        assert_eq!(config.deferred_ops.ttl, Duration::from_secs(30 * 60));
        assert_eq!(
            config.deferred_ops.ttl.as_secs(),
            plana::jsonrpc::deferred::DEFAULT_CLIENT_TTL_SECS
        );
        assert!(
            config.deferred_ops.ttl.as_secs() >= plana::jsonrpc::deferred::MIN_CLIENT_TTL_SECS
                && config.deferred_ops.ttl.as_secs()
                    <= plana::jsonrpc::deferred::MAX_CLIENT_TTL_SECS,
            "the client-facing window must stay inside the published 10-30 minute band"
        );
        assert!(config.deferred_ops.max_pending > 0);
        assert!(config.deferred_ops.max_retained >= config.deferred_ops.max_pending);
    }
}
