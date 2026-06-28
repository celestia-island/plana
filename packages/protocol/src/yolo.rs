//! YOLO Cruise Control — autonomous loop status & configuration.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::YoloTaskTier;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStartResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStopResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTerminateResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTaskResult {
    pub success: bool,
    pub duration_ms: u64,
    pub completed_at: String,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTaskStatus {
    pub agent: String,
    pub skill: String,
    pub enabled: bool,
    #[serde(default)]
    #[ts(optional)]
    pub last_result: Option<YoloTaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierStatus {
    pub tier: YoloTaskTier,
    pub enabled: bool,
    pub interval_secs: u64,
    #[serde(default)]
    #[ts(optional)]
    pub last_run_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub next_run_at: Option<String>,
    #[serde(default)]
    pub tasks: Vec<YoloTaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloStatusResponseParams {
    pub active: bool,
    pub loop_count: u64,
    #[serde(default)]
    #[ts(optional)]
    pub started_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub current_cycle: Option<String>,
    #[serde(default)]
    pub tiers: Vec<YoloTierStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloCycleStepParams {
    pub skill: String,
    pub loop_count: u64,
    pub status: String,
    #[serde(default)]
    #[ts(optional)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    #[ts(optional)]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloCycleCompleteParams {
    pub loop_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierTaskConfig {
    pub agent: String,
    pub skill: String,
    #[serde(default = "crate::default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloTierConfig {
    pub tier: YoloTaskTier,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub interval_secs: u64,
    #[serde(default)]
    pub tasks: Vec<YoloTierTaskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloConfigResponseParams {
    pub tiers: Vec<YoloTierConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloUpdateTaskResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloSetTierIntervalResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct YoloRunTierNowResponseParams {
    pub ok: bool,
    #[serde(default)]
    #[ts(optional)]
    pub error: Option<String>,
}
