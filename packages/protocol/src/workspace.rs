//! Workspace & Polemos device registry types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceStatusParams {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub connection_kind: String,
    #[serde(default)]
    #[ts(optional)]
    pub resolved_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub remote_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub branch: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub host_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PolemosDeviceInfo {
    #[ts(type = "string")]
    pub node_id: uuid::Uuid,
    pub name: String,
    pub address: String,
    pub status: String,
    #[serde(default)]
    #[ts(optional)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ide_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct PolemosDeviceListParams {
    pub devices: Vec<PolemosDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RegisterPolemosDeviceResponseParams {
    pub success: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub device: Option<PolemosDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct SwitchWorkspaceResponseParams {
    pub success: bool,
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}
