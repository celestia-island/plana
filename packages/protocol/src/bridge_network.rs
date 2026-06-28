//! Bridge Network — host machines + their workspaces.
//!
//! The 3rd chat sub-page ("桥接网络"): the left column lists host machines
//! (localhost + remote polemos devices) with live performance; the right
//! column lists the workspaces attached to the selected host with their
//! noa-git status + token usage. Clicking a host opens its file browser
//! (default /home); clicking a workspace opens its on-disk directory.
//!
//! Mirrors `entelecheia/.../tui_types/message/types/mod.rs`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Live performance snapshot for one host machine.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct HostMetrics {
    /// Stable host id ("localhost" for self, or a polemos device id).
    pub host_id: String,
    pub hostname: String,
    pub os: String,
    /// CPU utilisation, 0..100.
    pub cpu_usage_percent: f64,
    /// Logical CPU core count (shown with an i18n "cores" unit).
    pub cpu_cores: u32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    /// Outbound network rate (bytes/sec). Omitted when unknown.
    #[serde(default)]
    #[ts(optional)]
    pub net_up_bps: Option<u64>,
    /// Inbound network rate (bytes/sec). Omitted when unknown.
    #[serde(default)]
    #[ts(optional)]
    pub net_down_bps: Option<u64>,
}

/// noa-git status for a workspace checkout (branch / dirty / ahead / behind).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceGitStatus {
    pub branch: String,
    /// Modified/untracked file count.
    #[serde(default)]
    pub modified: u32,
    /// Commits ahead of upstream.
    #[serde(default)]
    pub ahead: u32,
    /// Commits behind upstream.
    #[serde(default)]
    pub behind: u32,
    /// `true` when there are uncommitted changes.
    #[serde(default)]
    pub dirty: bool,
}

/// One agent's token usage within a workspace (top-N entries).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceTokenUsage {
    pub agent: String,
    pub input: u64,
    pub output: u64,
}

/// A workspace attached to a host, with its git + token-usage summary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct WorkspaceNode {
    #[ts(type = "string")]
    pub workspace_id: uuid::Uuid,
    pub host_id: String,
    pub path: String,
    #[serde(default)]
    #[ts(optional)]
    pub alias: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub git: Option<WorkspaceGitStatus>,
    /// Top token consumers in this workspace (max 3).
    #[serde(default)]
    pub token_usage: Vec<WorkspaceTokenUsage>,
}

/// `Tui.RequestBridgeNetwork` — request the host/workspace roster.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct RequestBridgeNetworkParams {}

/// `Tui.BridgeNetwork` — the host/workspace roster response/push.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "WsTypes.ts")]
pub struct BridgeNetworkParams {
    pub hosts: Vec<HostMetrics>,
    pub workspaces: Vec<WorkspaceNode>,
}
