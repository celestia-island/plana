//! Industrial Control — telemetry, alarms, discovery, write approval, topology.
//!
//! Mirrors `entelecheia/packages/shared/state_types/src/gateway/tui_types/
//! message/types/mod.rs` (the canonical source of truth) 1:1. Field naming,
//! serde rename rules, and the `Industrial`-prefixed names all match entelecheia
//! so both sides of the WebSocket stay in sync without remapping.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ── Value types (canonical, from entelecheia) ──────────────

/// Severity ordering matches ISA-18.2 alarm severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialAlarmLevel {
    Log,
    LowLow,
    Low,
    High,
    HighHigh,
    RateOfChange,
    Emergency,
}

/// A single live reading from an industrial field (e.g. pressure cell on a
/// Modbus register, S7 DBX bit). Pushed by scepter at the scan cycle of the
/// underlying transport.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialSensorReading {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub name: String,
    pub raw_value: f64,
    pub scaled_value: f64,
    pub unit: String,
    pub quality: String,
    pub timestamp: String,
}

/// Fired on threshold breach (breached=true) or clear (breached=false).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmEvent {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
}

/// Phases of an evernight discovery scan. Ordered by typical progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
#[serde(rename_all = "PascalCase")]
pub enum IndustrialDiscoveryPhase {
    TransportScan,
    ProtocolIdentify,
    DataModelScan,
    SemanticInference,
    ManifestGeneration,
    ManifestValidation,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialDiscoveryProgress {
    pub session_id: String,
    pub phase: IndustrialDiscoveryPhase,
    pub message: String,
    pub found_devices: u64,
    pub progress_percent: u32,
    #[serde(default)]
    #[ts(optional, type = "Record<string, unknown> | null")]
    pub raw_findings: Option<serde_json::Value>,
}

/// Operator confirmation gate for safety-critical PLC writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
#[serde(rename_all = "lowercase")]
pub enum WriteApprovalRisk {
    Safe,
    Caution,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct WriteApprovalRequest {
    /// Unique id assigned by the producer (orexis). The operator UI echoes it
    /// back in `industrial.approveWrite` so scepter's resolver can match the
    /// response to the pending oneshot. `#[serde(default)]` keeps the wire
    /// format backward-compatible with older push events that predate it.
    #[serde(default)]
    pub request_id: String,
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub current_value: f64,
    pub proposed_value: f64,
    pub unit: String,
    pub reason: String,
    pub agent: String,
    pub risk_level: WriteApprovalRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialStationField {
    pub address: String,
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub alarm: Option<IndustrialAlarmThresholds>,
    #[serde(default)]
    #[ts(optional)]
    pub current_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmThresholds {
    #[serde(default)]
    #[ts(optional)]
    pub ll: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub l: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub h: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub hh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialStationInfo {
    pub station_id: String,
    pub protocol: String,
    pub connection: String,
    pub device_class: String,
    #[serde(default)]
    #[ts(optional)]
    pub vendor: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub model: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub firmware: Option<String>,
    pub status: String,
    #[serde(default)]
    pub fields: Vec<IndustrialStationField>,
}

/// One entry in the historical alarm log (last N days).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmHistoryEntry {
    pub station_id: String,
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
    /// Whether an operator acknowledged the alarm, and when.
    #[serde(default)]
    pub acknowledged: bool,
    #[serde(default)]
    #[ts(optional)]
    pub acknowledged_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmHistory {
    pub entries: Vec<IndustrialAlarmHistoryEntry>,
    pub total: u64,
}

// ── WS push / RPC param wrappers ───────────────────────────
//
// These wrap the canonical value types above as the `params` payload of the
// `Industrial*` / `topology.*` TuiMessage variants. They are arona-specific
// (entelecheia dispatches them inline on the TuiMessage enum) but are vendored
// by shittim-chest's webui, so they are retained here.

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialTelemetryBatch {
    pub readings: Vec<IndustrialSensorReading>,
    pub station_id: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialTelemetryPushParams {
    pub batch: IndustrialTelemetryBatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmPushParams {
    pub alarm: IndustrialAlarmEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmAckParams {
    pub station_id: String,
    pub address: String,
    pub acknowledged_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialDiscoveryProgressPushParams {
    pub event: IndustrialDiscoveryProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct WriteApprovalRequestParams {
    pub request: WriteApprovalRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct WriteApprovalResponseParams {
    pub request_id: String,
    pub approved: bool,
    pub approved_by: String,
    #[serde(default)]
    #[ts(optional)]
    pub modified_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialTopologyParams {
    pub stations: Vec<IndustrialStationInfo>,
}
