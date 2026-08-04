//! System / UI — web-UI control & status.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/systemUi.ts")]
pub struct WebUiControlResponseParams {
    pub command: String,
    pub success: bool,
    pub message: String,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/systemUi.ts")]
pub struct WebUiStatusParams {
    pub running: bool,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub container_id: Option<String>,
}
