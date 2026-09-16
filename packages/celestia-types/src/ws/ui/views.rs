//! Workspace & UI message params — dashboard view pushes.
//!
//! What lives here is the widget-level push contract: the webui composes a
//! panel from widgets ([`DashboardWidget`]), and the panel vocabulary itself
//! is `view_template` (the five base views).
//!
//! The earlier grid-dashboard model — `ViewKind` / `ViewInstance` /
//! `ViewLayout` / `DashboardLayout` and their push wrappers — is gone. It
//! described a layout the webui never adopted (panels are tabs of sub-views,
//! not a view grid), disagreed with the widget array the live
//! `Sync.DashboardLayoutPush` event actually carries, and had no Rust
//! consumer; dropping it leaves the crate with ONE panel vocabulary instead
//! of two whose serde values collided (`"kanban"` meant different things in
//! each).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A widget inside a webui dashboard descriptor (P3#A4).
///
/// Mirrors the shittim-chest `WidgetDescriptor` (dashboard.ts) so the
/// agent-side push tool can construct widgets the webui understands
/// without knowing its internals.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct DashboardWidget {
    /// Unique widget id within the layout.
    pub id: String,
    /// Renderer discriminator ("gauge-row", "node-graph", "data-table", …).
    #[serde(rename = "type")]
    pub widget_type: String,
    /// Optional display title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    /// Data-source label (free-form).
    #[serde(default)]
    pub source: String,
    /// Grid span hint ("full" | "half").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub span: Option<String>,
    /// Widget-specific configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub config: Option<serde_json::Value>,
    /// Initial data payload (shape depends on widget type).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub data: Option<serde_json::Value>,
}

/// Widget create/update/delete on an existing layout (P3#A4).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/views.ts")]
pub struct ViewInstancePushParams {
    /// Target layout (panel instance) id.
    pub layout_id: String,
    /// Mutation: "create" | "update" | "delete".
    pub op: String,
    /// Widget descriptor to create/update (ignored for delete).
    pub widget: DashboardWidget,
}
