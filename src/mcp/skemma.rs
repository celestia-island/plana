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
pub struct ScriptExecParams {
    pub code: String,
    pub language: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ModbusReadParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub scan: Option<Vec<ModbusScanConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub count: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ModbusWriteParams {
    pub endpoint: String,
    pub station: Option<u16>,
    pub writes: Option<Vec<ModbusWriteConfig>>,
    pub register_type: Option<String>,
    pub start_address: Option<u64>,
    pub values: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SignalNormalizeParams {
    pub values: Vec<f64>,
    pub method: Option<String>,
    pub signed: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ConnectRemoteViaSshParams {
    pub host: String,
    pub port: Option<u64>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ExecOnRemoteParams {
    pub remote_id: String,
    pub command: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ScreenshotParams {
    pub remote_id: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct MouseOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub button: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct KeyboardOperateParams {
    pub remote_id: String,
    pub action: Option<String>,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
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
