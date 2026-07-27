use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserInstanceInfo {
    pub browser_id: String,
    pub browser_type: String,
    pub headless: bool,
    pub window_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserCreateResult {
    pub browser_id: String,
    pub browser_type: String,
    pub headless: bool,
    pub window_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserCloseResult {
    pub browser_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserListResult {
    pub instances: Vec<BrowserInstanceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserNavigateResult {
    pub browser_id: String,
    pub final_url: String,
    pub title: String,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserScreenshotResult {
    pub browser_id: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserScriptResult {
    pub browser_id: String,
    #[ts(type = "Record<string, unknown>")]
    pub result: serde_json::Value,
    pub return_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserConsoleLogEntry {
    pub level: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserConsoleLogsResult {
    pub browser_id: String,
    pub entries: Vec<BrowserConsoleLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserNetworkEntry {
    pub method: String,
    pub url: String,
    pub status: String,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserNetworkLogsResult {
    pub browser_id: String,
    pub entries: Vec<BrowserNetworkEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserKeypressResult {
    pub browser_id: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserMouseClickResult {
    pub browser_id: String,
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserMouseMoveResult {
    pub browser_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/webAutomation.ts")]
pub struct BrowserRecordResult {
    pub browser_id: String,
    pub action: String,
    pub status: String,
    pub file_path: Option<String>,
}
