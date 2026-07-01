#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct NodeDiscoverResult {
    pub host: String,
    pub port: u16,
    pub node_id: Option<String>,
    pub total_nodes: usize,
    pub status: String,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct NodeConnectResult {
    pub node_id: String,
    pub node_name: String,
    pub address: String,
    pub status: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct NodeExecuteResult {
    pub node_id: String,
    pub node_name: String,
    pub host: String,
    pub command: String,
    pub exit_code: Option<i64>,
    pub stdout: String,
    pub stderr: String,
    pub status: String,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeDiscoverParams {
    pub auto_register: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeConnectParams {
    pub node_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeExecuteParams {
    pub node_id: String,
    pub command: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ProtocolProbeParams {
    pub host: String,
    pub ports: Option<Vec<u64>>,
    pub protocols: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct DeviceSelfTestParams {
    pub host: String,
    pub device_id: Option<String>,
    pub modbus_port: Option<u64>,
    pub mqtt_port: Option<u64>,
    pub http_port: Option<u64>,
    pub skip_adaptive: Option<bool>,
    pub register_ranges: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct EmptyParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeTerminalOpenParams {
    pub node_id: String,
    pub cols: Option<u64>,
    pub rows: Option<u64>,
    pub shell: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeTerminalWriteParams {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeTerminalResizeParams {
    pub session_id: String,
    pub cols: u64,
    pub rows: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeTerminalCloseParams {
    pub session_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeFileListParams {
    pub node_id: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeFileDownloadParams {
    pub node_id: String,
    pub remote_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeFileUploadParams {
    pub node_id: String,
    pub remote_path: String,
    pub data_base64: String,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct NodeScreenOfferParams {
    pub node_id: String,
}

// ── Tool result structs (network tools) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct ProtocolProbeResult {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub confidence: f64,
    pub banner: String,
    pub details: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct ProtocolProbeResponse {
    pub host: String,
    pub probes: Vec<ProtocolProbeResult>,
    pub total_found: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct KneeJerkTest {
    pub test_name: String,
    pub protocol: String,
    pub passed: bool,
    pub latency_ms: u64,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct DeviceRegisterRangeResult {
    pub start: u16,
    pub end: u16,
    pub function_code: u8,
    pub bytes_readable: u16,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct AdaptiveProbeResult {
    pub register_range: String,
    pub function_codes_tested: Vec<u8>,
    pub readable_ranges: Vec<DeviceRegisterRangeResult>,
    pub latency_avg_ms: u64,
    pub latency_max_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct DeviceCapability {
    pub protocols: Vec<String>,
    pub register_maps: Vec<DeviceRegisterRangeResult>,
    pub estimated_device_type: String,
    pub function_codes_supported: Vec<u8>,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct Phase1Result {
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub results: Vec<KneeJerkTest>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct Phase2Result {
    pub register_ranges_scanned: usize,
    pub function_codes_probed: Vec<u8>,
    pub readable_registers: usize,
    pub probe_results: Vec<AdaptiveProbeResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/polemos.ts")]
pub struct DeviceSelfTestResponse {
    pub device_id: String,
    pub host: String,
    pub overall_status: String,
    pub phase1_self_sensing: Phase1Result,
    pub phase2_adaptive: Phase2Result,
    pub capability_profile: DeviceCapability,
    pub registered_to_node_graph: bool,
    pub timestamp: String,
}
