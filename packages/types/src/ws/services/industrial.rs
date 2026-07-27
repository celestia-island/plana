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
// `Industrial*` / `topology.*` SyncMessage variants. They are arona-specific
// (entelecheia dispatches them inline on the SyncMessage enum) but are vendored
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── IndustrialAlarmLevel (PascalCase serde) ───────────────────

    #[test]
    fn alarm_level_pascal_case_round_trip() {
        for level in [
            IndustrialAlarmLevel::Log,
            IndustrialAlarmLevel::LowLow,
            IndustrialAlarmLevel::Low,
            IndustrialAlarmLevel::High,
            IndustrialAlarmLevel::HighHigh,
            IndustrialAlarmLevel::RateOfChange,
            IndustrialAlarmLevel::Emergency,
        ] {
            let s = serde_json::to_string(&level).unwrap();
            let back: IndustrialAlarmLevel = serde_json::from_str(&s).unwrap();
            assert_eq!(back, level);
        }
    }

    #[test]
    fn alarm_level_serializes_pascal_case() {
        assert_eq!(
            serde_json::to_string(&IndustrialAlarmLevel::HighHigh).unwrap(),
            r#""HighHigh""#
        );
        assert_eq!(
            serde_json::to_string(&IndustrialAlarmLevel::RateOfChange).unwrap(),
            r#""RateOfChange""#
        );
    }

    // ── IndustrialSensorReading ───────────────────────────────────

    #[test]
    fn sensor_reading_round_trip() {
        let r = IndustrialSensorReading {
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            address: "40001".into(),
            name: "reactor_pressure".into(),
            raw_value: 16384.0,
            scaled_value: 6.4,
            unit: "bar".into(),
            quality: "good".into(),
            timestamp: "2026-07-07T12:00:00Z".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["station_id"], "PLC-01");
        assert_eq!(v["scaled_value"], 6.4);
        assert_eq!(v["unit"], "bar");
        let back: IndustrialSensorReading = serde_json::from_value(v).unwrap();
        assert_eq!(back.raw_value, 16384.0);
    }

    // ── IndustrialAlarmEvent ──────────────────────────────────────

    #[test]
    fn alarm_event_breach_round_trip() {
        let e = IndustrialAlarmEvent {
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            address: "40001".into(),
            field_name: "reactor_pressure".into(),
            level: IndustrialAlarmLevel::HighHigh,
            value: 9.8,
            threshold: 8.0,
            unit: "bar".into(),
            breached: true,
            timestamp: "2026-07-07T12:00:00Z".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["level"], "HighHigh");
        assert_eq!(v["breached"], true);
        assert_eq!(v["value"], 9.8);
        let back: IndustrialAlarmEvent = serde_json::from_value(v).unwrap();
        assert_eq!(back.level, IndustrialAlarmLevel::HighHigh);
        assert!(back.breached);
    }

    #[test]
    fn alarm_event_clear_round_trip() {
        let e = IndustrialAlarmEvent {
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            address: "40001".into(),
            field_name: "pressure".into(),
            level: IndustrialAlarmLevel::High,
            value: 7.0,
            threshold: 8.0,
            unit: "bar".into(),
            breached: false,
            timestamp: "2026-07-07T12:01:00Z".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["breached"], false);
    }

    // ── IndustrialDiscoveryPhase ──────────────────────────────────

    #[test]
    fn discovery_phase_round_trip_all_variants() {
        for phase in [
            IndustrialDiscoveryPhase::TransportScan,
            IndustrialDiscoveryPhase::ProtocolIdentify,
            IndustrialDiscoveryPhase::DataModelScan,
            IndustrialDiscoveryPhase::SemanticInference,
            IndustrialDiscoveryPhase::ManifestGeneration,
            IndustrialDiscoveryPhase::ManifestValidation,
            IndustrialDiscoveryPhase::Complete,
        ] {
            let s = serde_json::to_string(&phase).unwrap();
            let back: IndustrialDiscoveryPhase = serde_json::from_str(&s).unwrap();
            assert_eq!(back, phase);
        }
    }

    #[test]
    fn discovery_progress_round_trip() {
        let p = IndustrialDiscoveryProgress {
            session_id: "disc-001".into(),
            phase: IndustrialDiscoveryPhase::ProtocolIdentify,
            message: "Identifying Modbus devices".into(),
            found_devices: 3,
            progress_percent: 45,
            raw_findings: Some(json!({"candidates": ["PLC-01", "PLC-02"]})),
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["phase"], "ProtocolIdentify");
        assert_eq!(v["progress_percent"], 45);
        assert_eq!(v["raw_findings"]["candidates"][0], "PLC-01");
        let back: IndustrialDiscoveryProgress = serde_json::from_value(v).unwrap();
        assert_eq!(back.found_devices, 3);
    }

    #[test]
    fn discovery_progress_without_raw_findings() {
        let p = IndustrialDiscoveryProgress {
            session_id: "s".into(),
            phase: IndustrialDiscoveryPhase::Complete,
            message: "done".into(),
            found_devices: 0,
            progress_percent: 100,
            raw_findings: None,
        };
        let v = serde_json::to_value(&p).unwrap();
        // #[ts(optional)] without skip → null on wire.
        assert_eq!(v["raw_findings"], serde_json::Value::Null);
    }

    // ── WriteApprovalRisk (lowercase serde) ───────────────────────

    #[test]
    fn write_approval_risk_lowercase_round_trip() {
        for risk in [
            WriteApprovalRisk::Safe,
            WriteApprovalRisk::Caution,
            WriteApprovalRisk::Critical,
        ] {
            let s = serde_json::to_string(&risk).unwrap();
            let back: WriteApprovalRisk = serde_json::from_str(&s).unwrap();
            assert_eq!(back, risk);
        }
    }

    #[test]
    fn write_approval_risk_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&WriteApprovalRisk::Safe).unwrap(),
            r#""safe""#
        );
        assert_eq!(
            serde_json::to_string(&WriteApprovalRisk::Critical).unwrap(),
            r#""critical""#
        );
    }

    // ── WriteApprovalRequest (safety-critical) ────────────────────

    #[test]
    fn write_approval_request_round_trip() {
        let r = WriteApprovalRequest {
            request_id: "req-001".into(),
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            address: "40001".into(),
            field_name: "setpoint".into(),
            current_value: 50.0,
            proposed_value: 75.0,
            unit: "%".into(),
            reason: "Agent requested setpoint increase".into(),
            agent: "polemos".into(),
            risk_level: WriteApprovalRisk::Caution,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["risk_level"], "caution");
        assert_eq!(v["proposed_value"], 75.0);
        assert_eq!(v["request_id"], "req-001");
        let back: WriteApprovalRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back.risk_level, WriteApprovalRisk::Caution);
    }

    #[test]
    fn write_approval_request_id_defaults_empty() {
        // request_id has #[serde(default)] — old messages without it
        // deserialize to empty string.
        let raw = json!({
            "station_id": "PLC-01",
            "protocol": "modbus-tcp",
            "address": "40001",
            "field_name": "setpoint",
            "current_value": 50.0,
            "proposed_value": 75.0,
            "unit": "%",
            "reason": "test",
            "agent": "agent",
            "risk_level": "safe"
        });
        let r: WriteApprovalRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(r.request_id, "");
    }

    #[test]
    fn write_approval_response_with_modified_value() {
        let r = WriteApprovalResponseParams {
            request_id: "req-001".into(),
            approved: true,
            approved_by: "operator-1".into(),
            modified_value: Some(70.0),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["approved"], true);
        assert_eq!(v["modified_value"], 70.0);
    }

    #[test]
    fn write_approval_response_without_modified_value() {
        let r = WriteApprovalResponseParams {
            request_id: "req-001".into(),
            approved: false,
            approved_by: "operator-1".into(),
            modified_value: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["modified_value"], serde_json::Value::Null);
    }

    // ── IndustrialStationInfo with nested fields ─────────────────

    #[test]
    fn station_info_with_alarm_thresholds() {
        let s = IndustrialStationInfo {
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            connection: "192.168.1.5:502".into(),
            device_class: "PLC".into(),
            vendor: Some("Siemens".into()),
            model: Some("S7-1200".into()),
            firmware: None,
            status: "online".into(),
            fields: vec![IndustrialStationField {
                address: "40001".into(),
                name: "pressure".into(),
                data_type: "float32".into(),
                unit: Some("bar".into()),
                alarm: Some(IndustrialAlarmThresholds {
                    ll: None,
                    l: Some(2.0),
                    h: Some(8.0),
                    hh: Some(10.0),
                }),
                current_value: Some(5.5),
            }],
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["vendor"], "Siemens");
        assert_eq!(v["fields"][0]["alarm"]["hh"], 10.0);
        assert_eq!(v["fields"][0]["current_value"], 5.5);
        let back: IndustrialStationInfo = serde_json::from_value(v).unwrap();
        assert_eq!(back.fields.len(), 1);
        assert_eq!(back.fields[0].alarm.as_ref().unwrap().hh, Some(10.0));
    }

    #[test]
    fn station_info_minimal() {
        let s = IndustrialStationInfo {
            station_id: "X".into(),
            protocol: "p".into(),
            connection: "c".into(),
            device_class: "d".into(),
            vendor: None,
            model: None,
            firmware: None,
            status: "offline".into(),
            fields: vec![],
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["vendor"], serde_json::Value::Null);
        assert_eq!(v["fields"], json!([]));
    }

    // ── Telemetry batch ───────────────────────────────────────────

    #[test]
    fn telemetry_batch_round_trip() {
        let batch = IndustrialTelemetryBatch {
            readings: vec![IndustrialSensorReading {
                station_id: "PLC-01".into(),
                protocol: "modbus-tcp".into(),
                address: "40001".into(),
                name: "temp".into(),
                raw_value: 1.0,
                scaled_value: 25.5,
                unit: "°C".into(),
                quality: "good".into(),
                timestamp: "2026-07-07T00:00:00Z".into(),
            }],
            station_id: "PLC-01".into(),
            timestamp: "2026-07-07T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&batch).unwrap();
        assert_eq!(v["readings"].as_array().unwrap().len(), 1);
        assert_eq!(v["readings"][0]["unit"], "°C");
        let back: IndustrialTelemetryBatch = serde_json::from_value(v).unwrap();
        assert_eq!(back.readings.len(), 1);
    }

    // ── Alarm history ─────────────────────────────────────────────

    #[test]
    fn alarm_history_entry_acknowledged() {
        let e = IndustrialAlarmHistoryEntry {
            station_id: "PLC-01".into(),
            protocol: "modbus-tcp".into(),
            address: "40001".into(),
            field_name: "pressure".into(),
            level: IndustrialAlarmLevel::High,
            value: 9.0,
            threshold: 8.0,
            unit: "bar".into(),
            breached: true,
            timestamp: "2026-07-07T00:00:00Z".into(),
            acknowledged: true,
            acknowledged_at: Some("2026-07-07T00:01:00Z".into()),
            acknowledged_by: Some("operator-1".into()),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["acknowledged"], true);
        assert_eq!(v["acknowledged_by"], "operator-1");
        let back: IndustrialAlarmHistoryEntry = serde_json::from_value(v).unwrap();
        assert!(back.acknowledged);
    }

    #[test]
    fn alarm_history_entry_unacknowledged() {
        let e = IndustrialAlarmHistoryEntry {
            station_id: "X".into(),
            protocol: "p".into(),
            address: "a".into(),
            field_name: "f".into(),
            level: IndustrialAlarmLevel::Low,
            value: 1.0,
            threshold: 2.0,
            unit: "u".into(),
            breached: true,
            timestamp: "t".into(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["acknowledged"], false);
        assert_eq!(v["acknowledged_at"], serde_json::Value::Null);
    }
}
