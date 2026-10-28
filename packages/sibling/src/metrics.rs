//! Metrics seam reserved for the P2-1 observability baseline.
//!
//! Nothing is wired yet: plana-sibling's only telemetry today is the
//! `sibling_call` tracing span (sibling / endpoint / method / path /
//! result / elapsed_ms). This module reserves the interface shape so the
//! future rollout does not become a semver-visible dependency addition:
//! enable the `metrics` feature, implement [`MetricsSink`] on your
//! collector, and wait for the wiring PR — the trait signature below is the
//! contract the P2-1 work is expected to fulfill (per-attempt, not
//! per-call, so failover stays observable).

use std::time::Duration;

/// Outcome classification for one attempt (mirrors the span `result` field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallOutcome {
    /// The endpoint answered with a success status.
    Ok,
    /// Transport-level failure (unreachable, reset, DNS, TLS).
    Unreachable,
    /// A timeout layer cut the attempt short.
    Timeout,
    /// The endpoint answered 4xx (business error).
    ClientError,
    /// The endpoint answered 5xx.
    ServerError,
    /// No endpoint was available at all (empty resolved list).
    NoEndpoints,
}

/// A metrics sink; implement this on your collector once the observability
/// baseline lands.
pub trait MetricsSink: Send + Sync {
    /// Record one attempt of one sibling call.
    fn record_call(&self, sibling: &str, endpoint: &str, outcome: &CallOutcome, elapsed: Duration);
}
