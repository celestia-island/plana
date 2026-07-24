use uuid::Uuid;

use crate::enums::ScriptLanguage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ScriptExecResult {
    pub language: ScriptLanguage,
    pub execution_id: Uuid,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct Layer2ScriptExecResult {
    pub language: ScriptLanguage,
    pub agent: String,
    pub tool: String,
    pub execution_id: Uuid,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub output: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct RemoteConnectionInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub connected: bool,
    pub connected_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ListRemotesResult {
    pub remotes: Vec<RemoteConnectionInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ConnectRemoteResult {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct DisconnectRemoteResult {
    pub disconnected: bool,
    pub remote_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ExecOnRemoteResult {
    pub remote_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ScreenshotResult {
    pub remote_id: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct MouseOperateResult {
    pub remote_id: String,
    pub action: String,
    pub x: i32,
    pub y: i32,
    pub button: String,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct KeyboardOperateResult {
    pub remote_id: String,
    pub action: String,
    pub keys: Vec<String>,
    pub success: bool,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ModbusScanConfig {
    pub address: u16,
    pub count: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_code: Option<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ModbusWriteConfig {
    pub address: u16,
    pub value: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_code: Option<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ScriptExecParams {
    pub code: String,
    pub language: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ModbusReadParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub scan: Option<Vec<ModbusScanConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub count: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ModbusWriteParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub writes: Option<Vec<ModbusWriteConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub values: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct SignalNormalizeParams {
    pub values: Vec<f64>,
    pub method: Option<String>,
    pub signed: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ConnectRemoteViaSshParams {
    pub host: String,
    pub port: Option<u64>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ExecOnRemoteParams {
    pub remote_id: String,
    pub command: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ScreenshotParams {
    pub remote_id: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct MouseOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub button: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct KeyboardOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct DisconnectRemoteParams {
    pub remote_id: String,
}

// ── Tool result structs (signal/modbus/opcua) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct SignalStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct SignalNormalizeResult {
    pub method: String,
    pub input_count: usize,
    pub output: Vec<f64>,
    pub stats: SignalStats,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct RegisterRangeResult {
    pub register_type: String,
    pub start_address: u16,
    pub count: u16,
    pub values: Vec<u16>,
    pub raw_bytes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct ModbusReadResult {
    pub station: u16,
    pub transport: String,
    pub endpoint: String,
    pub results: Vec<RegisterRangeResult>,
    pub total_registers: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
pub struct WriteRangeResult {
    pub register_type: String,
    pub start_address: u16,
    pub count: u16,
    pub confirmed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/skemma.ts")]
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
    use serde_json::json;

    #[test]
    fn script_exec_result_round_trip() {
        let r = ScriptExecResult {
            language: ScriptLanguage::Bash,
            execution_id: Uuid::new_v4(),
            exit_code: 0,
            duration_ms: 1500,
            stdout: "hello\n".into(),
            stderr: String::new(),
        };
        let v = serde_json::to_value(&r).unwrap();
        // ScriptLanguage serializes as PascalCase variant name (serde default).
        assert_eq!(v["language"], "Bash");
        assert_eq!(v["exit_code"], 0);
        assert_eq!(v["duration_ms"], 1500);
        let back: ScriptExecResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.language, ScriptLanguage::Bash);
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
