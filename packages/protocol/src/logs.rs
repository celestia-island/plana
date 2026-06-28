//! Log entry types — server / container log streaming.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct LogEntryData {
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub instance_uuid: Option<String>,
    pub level: String,
    #[serde(default)]
    #[ts(optional)]
    pub target: Option<String>,
    pub message: String,
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub fields: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ServerLogEntryParams {
    pub entry: LogEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct ContainerLogEntryParams {
    pub instance_uuid: String,
    pub entry: LogEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SubscribeLogsResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    pub entries: Vec<LogEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct UnsubscribeLogsResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}
