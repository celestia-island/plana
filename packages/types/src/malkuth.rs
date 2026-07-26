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
//! ## Phase 1: worker lifecycle (TBD)
//! - DrainRequest / WorkerRegistration / WorkerStatus / HealthResponse

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
