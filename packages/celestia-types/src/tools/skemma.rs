use uuid::Uuid;

use crate::enums::{ScriptExecutionOutcome, ScriptLanguage, SideEffectClass};

/// Result of one `script_exec` run. `outcome` replaces the former
/// `exit_code` with orthogonal failure classes; `value` carries the
/// lossless JSON completion value of the program (distinct from the
/// textual `stdout` console capture, which remains subject to the output
/// limit — see `stdout_truncated`).
///
/// Invariants (producer contract; the DTO itself does not enforce):
/// - `outcome == OutputLimit` implies `stdout_truncated == true`. The
///   truncation marker covers `stdout` only — `stderr` has no separate
///   flag, so engines that limit the streams jointly must mark the cut
///   here and say so in `stderr`.
/// - `value == None` means the completion was absent, undefined, or not
///   losslessly expressible. A completion value of JSON `null` is
///   indistinguishable from absence on the wire (`"value": null`
///   collapses to `None` on deserialize) — the one known loss of the
///   otherwise lossless boundary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ScriptExecResult {
    pub language: ScriptLanguage,
    pub execution_id: Uuid,
    pub outcome: ScriptExecutionOutcome,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub stdout_truncated: bool,
    #[serde(default)]
    pub dispatches: Vec<ScriptDispatchRecord>,
}

/// One tool dispatch performed from inside a script execution, recorded as
/// an implicit start/settle pair with parent linkage: `call_id` follows the
/// `<execution_id>:js:<n>` scheme assigned in submission order.
///
/// Invariants (producer contract): `settled_at_ms >= started_at_ms`, both
/// relative to the start of the enclosing execution. Consumers branch on
/// `side_effect.effective()`, never on raw variant equality (fail-closed
/// `Undeclared` handling).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ScriptDispatchRecord {
    pub call_id: String,
    pub tool: String,
    /// Milliseconds relative to the start of the enclosing execution.
    pub started_at_ms: u64,
    /// Milliseconds relative to the start of the enclosing execution; the
    /// producer contract guarantees `settled_at_ms >= started_at_ms`.
    pub settled_at_ms: u64,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub side_effect: SideEffectClass,
    /// Whether the dispatch's platform-side effects were committed by the
    /// enclosing execution's settlement. `false` on a settled record means
    /// the effects were discarded with the program; irreversible dispatches
    /// that already fired keep `committed = false` here as their audit
    /// trail — the real-world side effect is not undoable.
    pub committed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct Layer2ScriptExecResult {
    pub language: ScriptLanguage,
    pub agent: String,
    pub tool: String,
    pub execution_id: Uuid,
    pub outcome: ScriptExecutionOutcome,
    pub duration_ms: u64,
    pub output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct RemoteConnectionInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub connected: bool,
    pub connected_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ListRemotesResult {
    pub remotes: Vec<RemoteConnectionInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ConnectRemoteResult {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct DisconnectRemoteResult {
    pub disconnected: bool,
    pub remote_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ExecOnRemoteResult {
    pub remote_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ScreenshotResult {
    pub remote_id: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct MouseOperateResult {
    pub remote_id: String,
    pub action: String,
    pub x: i32,
    pub y: i32,
    pub button: String,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct KeyboardOperateResult {
    pub remote_id: String,
    pub action: String,
    pub keys: Vec<String>,
    pub success: bool,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusScanConfig {
    pub address: u16,
    pub count: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_code: Option<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusWriteConfig {
    pub address: u16,
    pub value: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_code: Option<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ScriptExecParams {
    pub code: String,
    pub language: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusReadParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub scan: Option<Vec<ModbusScanConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub count: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusWriteParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub writes: Option<Vec<ModbusWriteConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub values: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct SignalNormalizeParams {
    pub values: Vec<f64>,
    pub method: Option<String>,
    pub signed: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ConnectRemoteViaSshParams {
    pub host: String,
    pub port: Option<u64>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ExecOnRemoteParams {
    pub remote_id: String,
    pub command: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ScreenshotParams {
    pub remote_id: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct MouseOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub button: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct KeyboardOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct DisconnectRemoteParams {
    pub remote_id: String,
}

// ── Tool result structs (signal/modbus/opcua) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct SignalStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct SignalNormalizeResult {
    pub method: String,
    pub input_count: usize,
    pub output: Vec<f64>,
    pub stats: SignalStats,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct RegisterRangeResult {
    pub register_type: String,
    pub start_address: u16,
    pub count: u16,
    pub values: Vec<u16>,
    pub raw_bytes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusReadResult {
    pub station: u16,
    pub transport: String,
    pub endpoint: String,
    pub results: Vec<RegisterRangeResult>,
    pub total_registers: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct WriteRangeResult {
    pub register_type: String,
    pub start_address: u16,
    pub count: u16,
    pub confirmed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skemma.ts")]
pub struct ModbusWriteResult {
    pub station: u16,
    pub transport: String,
    pub endpoint: String,
    pub writes: Vec<WriteRangeResult>,
    pub total_written: usize,
    pub all_confirmed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ScriptLanguage;

    #[test]
    fn script_exec_result_round_trip() {
        let r = ScriptExecResult {
            language: ScriptLanguage::Typescript,
            execution_id: Uuid::new_v4(),
            outcome: ScriptExecutionOutcome::Completed,
            duration_ms: 1500,
            stdout: "hello\n".into(),
            stderr: String::new(),
            value: Some(serde_json::json!({"rows": 3})),
            stdout_truncated: false,
            dispatches: vec![ScriptDispatchRecord {
                call_id: "018f3a2e-9c1d-7b4a-8e2f-1a2b3c4d5e6f:js:0".into(),
                tool: "skemma.modbus_read".into(),
                started_at_ms: 12,
                settled_at_ms: 48,
                ok: true,
                error: None,
                side_effect: SideEffectClass::Pure,
                committed: true,
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        // ScriptLanguage serializes as PascalCase variant name (serde default).
        assert_eq!(v["language"], "Typescript");
        assert_eq!(v["outcome"], "Completed");
        assert_eq!(v["duration_ms"], 1500);
        assert_eq!(v["value"]["rows"], 3);
        assert_eq!(
            v["dispatches"][0]["call_id"],
            "018f3a2e-9c1d-7b4a-8e2f-1a2b3c4d5e6f:js:0"
        );
        assert_eq!(v["dispatches"][0]["side_effect"], "Pure");
        assert_eq!(v["dispatches"][0]["committed"], true);
        let back: ScriptExecResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.language, ScriptLanguage::Typescript);
        assert_eq!(back.outcome, ScriptExecutionOutcome::Completed);
        assert_eq!(back.dispatches.len(), 1);
        assert_eq!(back.dispatches[0].settled_at_ms, 48);
    }

    #[test]
    fn script_exec_result_omits_absent_value() {
        let r = ScriptExecResult {
            language: ScriptLanguage::Javascript,
            execution_id: Uuid::new_v4(),
            outcome: ScriptExecutionOutcome::Timeout,
            duration_ms: 30_000,
            stdout: String::new(),
            stderr: "script exceeded timeout".into(),
            value: None,
            stdout_truncated: false,
            dispatches: Vec::new(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert!(v.get("value").is_none());
        assert_eq!(v["outcome"], "Timeout");
        // defaults materialize on deserialize
        let back: ScriptExecResult = serde_json::from_value(v).unwrap();
        assert!(back.dispatches.is_empty());
    }

    #[test]
    fn dispatch_record_irreversible_undeclared_round_trip() {
        let rec = ScriptDispatchRecord {
            call_id: "x:js:1".into(),
            tool: "neikos.git_push".into(),
            started_at_ms: 100,
            settled_at_ms: 250,
            ok: true,
            error: None,
            side_effect: SideEffectClass::Irreversible,
            committed: false,
        };
        let v = serde_json::to_value(&rec).unwrap();
        assert_eq!(v["side_effect"], "Irreversible");
        assert_eq!(v["committed"], false);
        assert!(v.get("error").is_none());
        let back: ScriptDispatchRecord = serde_json::from_value(v).unwrap();
        assert_eq!(back.side_effect, SideEffectClass::Irreversible);
    }

    #[test]
    fn script_exec_result_defaults_on_missing_fields() {
        // Hand-written payload without the defaulted fields: the
        // `#[serde(default)]` forward-compatibility path (a producer that
        // knows `outcome` but predates value/truncation/dispatches).
        let v: serde_json::Value = serde_json::json!({
            "language": "Javascript",
            "execution_id": Uuid::new_v4().to_string(),
            "outcome": "Completed",
            "duration_ms": 42,
            "stdout": "ok",
            "stderr": ""
        });
        let back: ScriptExecResult = serde_json::from_value(v).unwrap();
        assert!(back.value.is_none());
        assert!(!back.stdout_truncated);
        assert!(back.dispatches.is_empty());
    }

    #[test]
    fn script_exec_result_stdout_truncated_wire() {
        let r = ScriptExecResult {
            language: ScriptLanguage::Javascript,
            execution_id: Uuid::new_v4(),
            outcome: ScriptExecutionOutcome::OutputLimit,
            duration_ms: 5,
            stdout: "... [truncated]".into(),
            stderr: String::new(),
            value: None,
            stdout_truncated: true,
            dispatches: Vec::new(),
        };
        let v = serde_json::to_value(&r).unwrap();
        // The marker is always on the wire, true in the truncated case —
        // pinning the OutputLimit => stdout_truncated producer contract.
        assert_eq!(v["stdout_truncated"], true);
        let back: ScriptExecResult = serde_json::from_value(v).unwrap();
        assert!(back.stdout_truncated);
    }

    #[test]
    fn dispatch_record_undeclared_maps_irreversible() {
        let rec = ScriptDispatchRecord {
            call_id: "x:js:2".into(),
            tool: "mystery_tool".into(),
            started_at_ms: 10,
            settled_at_ms: 30,
            ok: true,
            error: None,
            side_effect: SideEffectClass::Undeclared,
            committed: false,
        };
        let v = serde_json::to_value(&rec).unwrap();
        assert_eq!(v["side_effect"], "Undeclared");
        let back: ScriptDispatchRecord = serde_json::from_value(v).unwrap();
        // Fail-closed canonical mapping: Undeclared acts as Irreversible.
        assert_eq!(back.side_effect, SideEffectClass::Undeclared);
        assert!(back.side_effect.is_irreversible());
        assert_eq!(back.side_effect.effective(), SideEffectClass::Irreversible);
        assert!(SideEffectClass::Irreversible.is_irreversible());
        assert!(!SideEffectClass::Pure.is_irreversible());
        assert!(!SideEffectClass::Stateful.is_irreversible());
        assert_eq!(SideEffectClass::Pure.effective(), SideEffectClass::Pure);
    }

    #[test]
    fn layer2_script_exec_result_round_trip() {
        let r = Layer2ScriptExecResult {
            language: ScriptLanguage::Layer2,
            agent: "web_automation".into(),
            tool: "browser_navigate".into(),
            execution_id: Uuid::new_v4(),
            outcome: ScriptExecutionOutcome::Exception,
            duration_ms: 800,
            output: "navigation failed".into(),
            value: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["outcome"], "Exception");
        assert!(v.get("value").is_none());
        let back: Layer2ScriptExecResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.outcome, ScriptExecutionOutcome::Exception);
    }

    #[test]
    fn remote_connection_info_round_trip() {
        let r = RemoteConnectionInfo {
            id: "ssh-1".into(),
            host: "192.168.1.10".into(),
            port: 22,
            protocol: "ssh".into(),
            connected: true,
            connected_at: Some("2026-01-01T00:00:00Z".into()),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["port"], 22);
        assert_eq!(v["connected_at"], "2026-01-01T00:00:00Z");
        let back: RemoteConnectionInfo = serde_json::from_value(v).unwrap();
        assert!(back.connected);
    }

    #[test]
    fn remote_connection_info_no_connected_at() {
        let r = RemoteConnectionInfo {
            id: "x".into(),
            host: "h".into(),
            port: 22,
            protocol: "ssh".into(),
            connected: false,
            connected_at: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["connected_at"], serde_json::Value::Null);
    }

    #[test]
    fn screenshot_result_round_trip() {
        let r = ScreenshotResult {
            remote_id: "ssh-1".into(),
            width: 1920,
            height: 1080,
            format: "png".into(),
            data_base64: "iVBORw0KGgo=".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["width"], 1920);
        assert_eq!(v["format"], "png");
        let back: ScreenshotResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.data_base64, "iVBORw0KGgo=");
    }

    #[test]
    fn modbus_read_result_round_trip() {
        let r = ModbusReadResult {
            station: 1,
            transport: "tcp".into(),
            endpoint: "192.168.1.5:502".into(),
            results: vec![RegisterRangeResult {
                register_type: "holding".into(),
                start_address: 0,
                count: 4,
                values: vec![100, 200, 300, 400],
                raw_bytes: vec!["0x0064".into(), "0x00C8".into()],
            }],
            total_registers: 4,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["station"], 1);
        assert_eq!(v["results"][0]["values"][1], 200);
        assert_eq!(v["total_registers"], 4);
        let back: ModbusReadResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.results[0].values.len(), 4);
    }

    #[test]
    fn modbus_write_result_round_trip() {
        let r = ModbusWriteResult {
            station: 1,
            transport: "tcp".into(),
            endpoint: "192.168.1.5:502".into(),
            writes: vec![WriteRangeResult {
                register_type: "holding".into(),
                start_address: 0,
                count: 2,
                confirmed: true,
            }],
            total_written: 2,
            all_confirmed: true,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["all_confirmed"], true);
        let back: ModbusWriteResult = serde_json::from_value(v).unwrap();
        assert!(back.all_confirmed);
    }

    #[test]
    fn signal_normalize_result_with_stats() {
        let r = SignalNormalizeResult {
            method: "min-max".into(),
            input_count: 5,
            output: vec![0.0, 0.25, 0.5, 0.75, 1.0],
            stats: SignalStats {
                min: 0.0,
                max: 100.0,
                mean: 50.0,
                std_dev: 31.62,
            },
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["stats"]["mean"], 50.0);
        assert_eq!(v["output"].as_array().unwrap().len(), 5);
        let back: SignalNormalizeResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.stats.min, 0.0);
    }

    #[test]
    fn mouse_operate_result_round_trip() {
        let r = MouseOperateResult {
            remote_id: "ssh-1".into(),
            action: "click".into(),
            x: 100,
            y: 200,
            button: "left".into(),
            success: true,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["x"], 100);
        assert_eq!(v["button"], "left");
    }

    #[test]
    fn modbus_scan_config_optional_function_code() {
        let c = ModbusScanConfig {
            address: 0,
            count: 10,
            function_code: None,
        };
        let v = serde_json::to_value(&c).unwrap();
        // function_code uses skip_serializing_if.
        assert!(v.get("function_code").is_none());
    }

    #[test]
    fn modbus_scan_config_with_function_code() {
        let c = ModbusScanConfig {
            address: 0,
            count: 10,
            function_code: Some(3),
        };
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["function_code"], 3);
    }
}
