//! malkuth supervision protocol types.
//!
//! Wire types for the restart-authorization gate and worker lifecycle
//! management across the celestia-island supervision tree.
//!
//! ## Phase 0: authorization gate
//! - RestartProposal: generated after a repo rebuild completes
//! - RestartGateDecision: OreXis audit result (allow/review/block)
//! - GateDecision: enum for the three-state gate
//!
//! ## Phase 1: worker lifecycle
//! - DrainRequest / WorkerRegistration / WorkerStatus / WorkerHealthResponse

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A rebuild has completed and restart is proposed for a supervised worker.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct RestartProposal {
    pub proposal_id: String,
    pub worker_id: String,
    pub repo_path: String,
    pub git_diff_summary: String,
    pub affected_services: Vec<String>,
    pub risk_estimate: RestartRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
#[serde(rename_all = "snake_case")]
pub enum RestartRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// OreXis LAYER3_PREFLIGHT_GUARD gate decision for a restart proposal.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct RestartGateDecision {
    pub proposal_id: String,
    pub decision: GateDecision,
    pub findings: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    /// Proceed with restart.
    Allow,
    /// Escalate to human for review.
    Review,
    /// Block restart — security risk detected.
    Block,
}

// ═══════════════════════════════════════════════════════════════
// Phase 1 — worker lifecycle wire types
// ═══════════════════════════════════════════════════════════════

/// External request to drain (gracefully shut down) a supervised worker.
/// Sent to malkuth daemon after a restart proposal has been approved.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct DrainRequest {
    /// Which worker to drain.
    pub worker_id: String,
    /// Authorization: must match a previously approved GateDecision.proposal_id.
    pub proposal_id: String,
    /// Optional drain budget in seconds (default derived from worker config).
    pub drain_budget_secs: Option<u64>,
}

/// Registers a new worker with the malkuth supervisor.
/// Sent by a worker at startup to announce its presence and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct WorkerRegistration {
    pub worker_id: String,
    pub worker_kind: String,
    pub repo_path: String,
    pub connections: Vec<ConnectionEndpoint>,
    pub health_check_path: Option<String>,
    pub drain_budget_secs: u64,
}

/// Describes how clients connect to a supervised worker.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct ConnectionEndpoint {
    pub protocol: ConnectionProtocol,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
#[serde(rename_all = "snake_case")]
pub enum ConnectionProtocol {
    Http,
    Ws,
    Ipc,
    Tcp,
}

/// Current status of a supervised worker returned by malkuth when queried.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct WorkerStatus {
    pub worker_id: String,
    pub state: WorkerState,
    pub pid: Option<u32>,
    pub restart_count: u32,
    pub last_restart_at: Option<String>,
    pub connections: Vec<ConnectionEndpoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
#[serde(rename_all = "snake_case")]
pub enum WorkerState {
    Running,
    /// Worker is accepting /readyz but not new work (drain in progress).
    Draining,
    Stopped,
    Crashed,
}

/// Health probe response returned by each malkuth supervision worker at
/// `/healthz` or `/readyz` (worker-lifecycle plane).
///
/// Distinct from the generic backend descriptor
/// `plana::protocol_core::http::HealthResponse` (the `/api/health` payload of
/// plana backends): the two share only the historical concept, not a wire
/// shape — this struct carries the worker probe fields, the other the backend
/// service/build fields — so they are separate types with distinct shapes
/// (both carry the generic `version` field).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "ws/malkuth.ts")]
pub struct WorkerHealthResponse {
    pub worker_id: String,
    pub healthy: bool,
    pub ready: bool,
    /// If not ready, the reason (e.g. "draining", "starting").
    pub not_ready_reason: Option<String>,
    pub uptime_secs: u64,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::WorkerHealthResponse;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use serde_json::{json, Value};

    /// Serialize → assert the exact JSON → deserialize → re-serialize, the
    /// same contract the evernight wire-shape tests pin.
    fn round_trip<T>(value: &T, want: Value)
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let wire = serde_json::to_value(value).unwrap();
        assert_eq!(wire, want, "serialization must pin the wire shape");
        let back: T = serde_json::from_value(want.clone()).unwrap();
        assert_eq!(&back, value, "deserialization must reconstruct the value");
        assert_eq!(
            serde_json::to_value(&back).unwrap(),
            want,
            "the round-tripped value must serialize identically"
        );
    }

    fn sample() -> WorkerHealthResponse {
        WorkerHealthResponse {
            worker_id: "w-1".into(),
            healthy: true,
            ready: false,
            not_ready_reason: Some("draining".into()),
            uptime_secs: 42,
            version: "1.0.0".into(),
        }
    }

    #[test]
    fn worker_health_response_wire_shape_is_pinned() {
        round_trip(
            &sample(),
            json!({
                "worker_id": "w-1",
                "healthy": true,
                "ready": false,
                "not_ready_reason": "draining",
                "uptime_secs": 42,
                "version": "1.0.0",
            }),
        );
    }
}
