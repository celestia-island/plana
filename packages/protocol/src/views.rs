//! Abstract View Interface — pluggable dashboard views.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// View type identifier — determines which frontend renderer handles the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub enum ViewKind {
    /// Industrial SCADA / HMI panel (P&ID, gauges, alarm panel, trend charts)
    IndustrialScada,
    /// Chat / conversation interface (default demiurge view)
    Chat,
    /// Kanban board (task cards, columns, drag-drop)
    Kanban,
    /// Gantt chart (timeline, milestones, dependencies)
    Gantt,
    /// Data table / spreadsheet (like Feishu multi-dimensional table)
    DataTable,
    /// Audio/video generation flow (node graph like ComfyUI)
    MediaFlow,
    /// File explorer / code browser
    FileExplorer,
    /// Custom (plugin-rendered)
    Custom,
}

/// A view instance — one panel in the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct ViewInstance {
    /// Unique view ID within the workspace.
    pub view_id: String,
    /// What kind of renderer to use.
    pub kind: ViewKind,
    /// Display title.
    pub title: String,
    /// Data source identifier — what the view is bound to.
    /// Examples: "industrial:station:19", "chat:conversation:abc",
    /// "kanban:project:xyz", "media:flow:comfyui"
    pub data_source: String,
    /// View-specific configuration (JSON, interpreted by the renderer).
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub config: serde_json::Value,
    /// Layout position (grid area, tab order, etc.).
    #[serde(default)]
    #[ts(optional)]
    pub layout: Option<ViewLayout>,
}

/// Layout descriptor for a view within the dashboard grid.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct ViewLayout {
    /// Grid column start (1-based).
    #[serde(default)]
    pub col: u32,
    /// Grid row start (1-based).
    #[serde(default)]
    pub row: u32,
    /// Column span.
    #[serde(default)]
    pub col_span: u32,
    /// Row span.
    #[serde(default)]
    pub row_span: u32,
    /// Minimum width in pixels.
    #[serde(default)]
    #[ts(optional)]
    pub min_width: Option<u32>,
    /// Minimum height in pixels.
    #[serde(default)]
    #[ts(optional)]
    pub min_height: Option<u32>,
}

/// Dashboard layout — a collection of views arranged in a grid.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct DashboardLayout {
    /// Workspace ID this dashboard belongs to.
    pub workspace_id: String,
    /// Dashboard name.
    pub name: String,
    /// All view instances in this dashboard.
    pub views: Vec<ViewInstance>,
    /// Grid columns count (0 = auto).
    #[serde(default)]
    pub grid_columns: u32,
}

/// Push a dashboard layout update to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct DashboardLayoutPushParams {
    pub layout: DashboardLayout,
}

/// View data update — incremental data push for a specific view.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct ViewDataPushParams {
    /// Target view ID.
    pub view_id: String,
    /// Data payload (format depends on ViewKind).
    #[ts(type = "Record<string, unknown>")]
    pub data: serde_json::Value,
    /// Whether this is a full replacement or incremental update.
    #[serde(default)]
    pub full_replace: bool,
}
