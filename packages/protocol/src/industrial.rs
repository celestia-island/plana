//! Industrial Control — telemetry, alarms, discovery, write approval, topology.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ── Telemetry ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialSensorReading {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub name: String,
    pub raw_value: f64,
    pub scaled_value: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub quality: String,
    pub timestamp: String,
}

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

// ── Alarms ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub enum IndustrialAlarmLevel {
    Log,
    LowLow,
    Low,
    High,
    HighHigh,
    RateOfChange,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialAlarmEvent {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub level: IndustrialAlarmLevel,
    pub value: f64,
    pub threshold: f64,
    #[serde(default)]
    pub unit: String,
    pub breached: bool,
    pub timestamp: String,
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

// ── Discovery ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub enum DiscoveryPhase {
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
pub struct DiscoveryProgressEvent {
    pub session_id: String,
    pub phase: DiscoveryPhase,
    pub message: String,
    #[serde(default)]
    pub found_devices: u32,
    #[serde(default)]
    pub progress_percent: u8,
    #[serde(default)]
    #[ts(optional)]
    pub raw_findings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct DiscoveryProgressPushParams {
    pub event: DiscoveryProgressEvent,
}

// ── Write Approval (human-in-the-loop) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct WriteApprovalRequest {
    /// Mirrors `_shared_state_sync::WriteApprovalRequest::request_id`. The
    /// operator UI echoes this back in `industrial.approveWrite` so the
    /// resolver can match the response to the pending producer oneshot.
    #[serde(default)]
    pub request_id: String,
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    pub address: String,
    pub field_name: String,
    pub current_value: f64,
    pub proposed_value: f64,
    #[serde(default)]
    pub unit: String,
    pub reason: String,
    pub agent: String,
    #[serde(default)]
    pub risk_level: String,
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

// ── Topology (station metadata for UI) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct AlarmThresholdInfo {
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
pub struct StationFieldInfo {
    pub address: String,
    pub name: String,
    #[serde(default)]
    pub data_type: String,
    #[serde(default)]
    #[ts(optional)]
    pub unit: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub alarm: Option<AlarmThresholdInfo>,
    #[serde(default)]
    #[ts(optional)]
    pub current_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialStationInfo {
    pub station_id: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub connection: String,
    #[serde(default)]
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
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub fields: Vec<StationFieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/industrial.ts")]
pub struct IndustrialTopologyParams {
    pub stations: Vec<IndustrialStationInfo>,
}
