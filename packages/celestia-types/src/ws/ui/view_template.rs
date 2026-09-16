//! Panel view templates — the declarative format behind workspace panels.
//!
//! A template names the **base view** it is drawn on and fills that base's
//! spec. The webui resolves `render_engine` through its plugin registry and
//! reads the spec to configure the base component; nothing described here
//! executes code, so an author can only combine vocabulary this build
//! already draws.
//!
//! The five bases mirror the components the library ships:
//!
//!   `waterfall`      bucketed card stream over independent columns
//!   `node-canvas`    pannable / zoomable node-and-edge surface
//!   `data-grid`      typed tabular grid
//!   `kanban`         draggable typed cards in lanes (the lane header is
//!                    itself a card)
//!   `blank-canvas`   a bare 100% × 100% mount (3D scenes, embeds)

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ── Base view ─────────────────────────────────────────────────────────

/// The base view a template is built on — the spec arm that must be set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum BaseViewKind {
    /// Bucketed card stream laid out into independent columns.
    Waterfall,
    /// Pannable / zoomable canvas of nodes and edges.
    NodeCanvas,
    /// Typed tabular grid.
    DataGrid,
    /// Draggable typed cards in lanes.
    Kanban,
    /// A bare full-bleed mount.
    BlankCanvas,
}

// ── Waterfall ─────────────────────────────────────────────────────────

/// How a waterfall splits its item stream into columns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case", tag = "kind")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum WaterfallColumns {
    /// One full-width column.
    Single,
    /// A fixed column count.
    Fixed {
        /// Number of columns (>= 1).
        count: u32,
    },
    /// As many columns as fit, capped.
    Adaptive {
        /// Upper bound on the derived column count.
        max: u32,
        /// Minimum column width in pixels.
        min_width: u32,
    },
}

/// Where the windowed rows come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum WindowingSource {
    /// The client windows a fully loaded list.
    Client,
    /// The host pages the list in on demand.
    Host,
}

/// Windowing parameters for a scrolling base view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct WindowingSpec {
    /// Which side owns the window.
    pub source: WindowingSource,
    /// Estimated row height in pixels before measurement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub estimated_item_height: Option<u32>,
    /// Screens of overscan kept mounted on either side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub overscan_screens: Option<f64>,
}

/// How a waterfall groups its items into buckets (day sections, lanes, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct GroupingSpec {
    /// Field of the item the bucket key is read from.
    pub field: String,
    /// Whether the buckets are ordered newest-first.
    #[serde(default)]
    pub newest_first: bool,
}

/// The waterfall base's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct WaterfallSpec {
    /// Column layout.
    pub columns: WaterfallColumns,
    /// Optional bucketing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub grouping: Option<GroupingSpec>,
    /// Optional windowing (absent = render the whole list).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub windowing: Option<WindowingSpec>,
    /// Card kinds this view may render, in picker order.
    #[serde(default)]
    pub card_kinds: Vec<String>,
    /// Optional rail widget drawn beside the stream (a timeline).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rail: Option<String>,
    /// Optional search affordance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub search: Option<String>,
}

// ── Node canvas ───────────────────────────────────────────────────────

/// How a node canvas draws its layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum CanvasBackend {
    /// Every layer is SVG.
    Svg,
    /// DOM nodes over an SVG edge layer.
    DomOverlaySvg,
    /// A 2D-canvas edge layer under DOM nodes.
    CanvasEdgesDom,
    /// A single full-bleed canvas the host paints itself.
    Blank,
}

/// Camera behaviour of a node canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct CameraSpec {
    /// Smallest allowed zoom factor.
    pub min_zoom: f64,
    /// Largest allowed zoom factor.
    pub max_zoom: f64,
    /// Zoom step as a fraction (0.05 = 5%).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub zoom_step: Option<f64>,
    /// Whether the camera fits the content on first paint.
    #[serde(default)]
    pub fit_on_load: bool,
}

/// The built-in minimap, which is on by default for node canvases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct MinimapSpec {
    /// Whether the minimap is drawn.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Where it is anchored.
    #[serde(default = "default_minimap_placement")]
    pub placement: DockPlacement,
}

fn default_true() -> bool {
    true
}

fn default_minimap_placement() -> DockPlacement {
    DockPlacement::BottomRight
}

/// The node-canvas base's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct NodeCanvasSpec {
    /// Which rendering backend draws the layers.
    pub backend: CanvasBackend,
    /// Camera bounds and first-paint behaviour.
    pub camera: CameraSpec,
    /// The minimap, on by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub minimap: Option<MinimapSpec>,
    /// Node kinds this canvas may render.
    #[serde(default)]
    pub node_kinds: Vec<String>,
    /// Optional left rail (a node palette).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub palette: Option<String>,
    /// Optional right rail (an inspector).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub inspector: Option<String>,
    /// Optional auto-layout algorithm id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auto_layout: Option<String>,
}

// ── Data grid ─────────────────────────────────────────────────────────

/// The data-grid base's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct DataGridSpec {
    /// Field schema ids this grid binds to.
    #[serde(default)]
    pub fields: Vec<String>,
    /// Whether rows can be edited in place.
    #[serde(default)]
    pub editable: bool,
    /// Optional overlay widget drawn above the grid (trend, alarms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub overlay: Option<String>,
}

// ── Kanban ────────────────────────────────────────────────────────────

/// Which way a kanban scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum ScrollAxes {
    /// Lanes side by side, scrolling across.
    Horizontal,
    /// Lanes stacked, scrolling down.
    Vertical,
    /// Both directions.
    Both,
}

/// One lane of a kanban.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct LaneSpec {
    /// Field of the item the lane key is read from.
    pub field: String,
    /// Whether the lane header is drawn as a card of its own.
    #[serde(default)]
    pub header_is_card: bool,
}

/// Drag-and-drop behaviour of a kanban.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct DndSpec {
    /// Whether cards can move between lanes.
    #[serde(default)]
    pub across_lanes: bool,
    /// Whether cards can be reordered inside a lane.
    #[serde(default)]
    pub reorder: bool,
}

/// The kanban base's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct KanbanSpec {
    /// Scroll directions the board supports.
    pub axes: ScrollAxes,
    /// Lane definition.
    pub lanes: LaneSpec,
    /// Draggable card kinds, in picker order.
    #[serde(default)]
    pub card_kinds: Vec<String>,
    /// Drag-and-drop behaviour.
    pub dnd: DndSpec,
    /// Optional windowing for long boards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub windowing: Option<WindowingSpec>,
}

// ── Blank canvas ──────────────────────────────────────────────────────

/// The blank-canvas base's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct BlankCanvasSpec {
    /// What mounts into the canvas (a renderer id the host resolves).
    pub mount: String,
    /// Optional DOM overlay drawn above the canvas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub overlay: Option<String>,
}

// ── Chrome & docks ────────────────────────────────────────────────────

/// Where a dock surface hangs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum DockPlacement {
    /// Top edge, centred.
    Top,
    /// Bottom edge, centred.
    Bottom,
    /// Left edge, centred.
    Left,
    /// Right edge, centred.
    Right,
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// The plane a dock surface sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum DockAnchor {
    /// Floating above the page content.
    Page,
    /// In flow inside a host band.
    Plane,
}

/// The finish of a dock surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub enum DockSurface {
    /// Translucent with backdrop blur.
    Glass,
    /// Opaque, for moving canvases.
    Solid,
}

/// One dock widget a template hangs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct DockContribution {
    /// Widget id the host resolves.
    pub widget: String,
    /// Where it hangs.
    pub placement: DockPlacement,
    /// Which plane carries it.
    #[serde(default = "default_dock_anchor")]
    pub anchor: DockAnchor,
    /// Surface finish.
    #[serde(default = "default_dock_surface")]
    pub surface: DockSurface,
    /// Stacking order among contributions sharing a placement.
    #[serde(default)]
    pub order: i32,
}

fn default_dock_anchor() -> DockAnchor {
    DockAnchor::Page
}

fn default_dock_surface() -> DockSurface {
    DockSurface::Glass
}

/// The chrome a template asks the shell to draw around it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct ChromeSpec {
    /// Docked widgets this view contributes.
    #[serde(default)]
    pub docks: Vec<DockContribution>,
    /// Whether the shell draws its breadcrumb bar.
    #[serde(default)]
    pub breadcrumb: bool,
}

// ── The template ──────────────────────────────────────────────────────

/// The spec arm matching a template's base view.
///
/// Exactly one arm is set — the one named by [`ViewTemplateSpec::base`] —
/// so a reader never has to guess which of the five it is looking at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct ViewTemplateSpec {
    /// The base view this template is drawn on.
    pub base: BaseViewKind,
    /// Set iff `base` is `waterfall`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub waterfall: Option<WaterfallSpec>,
    /// Set iff `base` is `node-canvas`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub node_canvas: Option<NodeCanvasSpec>,
    /// Set iff `base` is `data-grid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub data_grid: Option<DataGridSpec>,
    /// Set iff `base` is `kanban`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kanban: Option<KanbanSpec>,
    /// Set iff `base` is `blank-canvas`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub blank_canvas: Option<BlankCanvasSpec>,
    /// Optional chrome (docks, breadcrumb).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chrome: Option<ChromeSpec>,
}

/// One panel template: identity, presentation and the base view's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct PanelTemplate {
    /// Template id — kebab-case, stable across config layers.
    pub id: String,
    /// Display title (the author's language).
    pub title: String,
    /// Optional i18n key; when present the shell translates it instead of
    /// using `title`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title_key: Option<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// Optional description i18n key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description_key: Option<String>,
    /// Render-engine (plugin) id that draws this template.
    pub render_engine: String,
    /// The base view and its configuration.
    pub spec: ViewTemplateSpec,
}

/// A template file — the TOML document a workspace or plugin ships.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "ws/viewTemplate.ts")]
pub struct ViewTemplateFile {
    /// Format version.
    pub version: String,
    /// The templates the file declares.
    #[serde(default)]
    pub template: Vec<PanelTemplate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn waterfall_template() -> PanelTemplate {
        PanelTemplate {
            id: "reports".to_string(),
            title: "Reports".to_string(),
            title_key: None,
            description: None,
            description_key: None,
            render_engine: "celestia-waterfall".to_string(),
            spec: ViewTemplateSpec {
                base: BaseViewKind::Waterfall,
                waterfall: Some(WaterfallSpec {
                    columns: WaterfallColumns::Adaptive {
                        max: 2,
                        min_width: 320,
                    },
                    grouping: Some(GroupingSpec {
                        field: "timestamp".to_string(),
                        newest_first: true,
                    }),
                    windowing: Some(WindowingSpec {
                        source: WindowingSource::Client,
                        estimated_item_height: Some(150),
                        overscan_screens: Some(1.0),
                    }),
                    card_kinds: vec!["report-card".to_string()],
                    rail: Some("day-rail".to_string()),
                    search: None,
                }),
                node_canvas: None,
                data_grid: None,
                kanban: None,
                blank_canvas: None,
                chrome: None,
            },
        }
    }

    #[test]
    fn round_trips_a_waterfall_template_through_toml() {
        let file = ViewTemplateFile {
            version: "1".to_string(),
            template: vec![waterfall_template()],
        };
        let toml_text = toml::to_string(&file).expect("serialises");
        let back: ViewTemplateFile = toml::from_str(&toml_text).expect("parses");
        assert_eq!(back, file);
    }

    #[test]
    fn keeps_the_base_view_arms_apart() {
        // A template names exactly one arm; the others stay absent so a
        // reader never has to guess which spec applies.
        let spec = &waterfall_template().spec;
        assert_eq!(spec.base, BaseViewKind::Waterfall);
        assert!(spec.waterfall.is_some());
        assert!(spec.node_canvas.is_none());
        assert!(spec.data_grid.is_none());
        assert!(spec.kanban.is_none());
        assert!(spec.blank_canvas.is_none());
    }

    #[test]
    fn the_minimap_defaults_to_the_bottom_right_corner() {
        let spec: MinimapSpec = serde_json::from_str("{}").expect("defaults deserialise");
        assert!(spec.enabled);
        assert_eq!(spec.placement, DockPlacement::BottomRight);
    }

    #[test]
    fn a_dock_contribution_defaults_to_the_page_plane() {
        let dock: DockContribution =
            serde_json::from_str(r#"{"widget":"chat-bar","placement":"bottom"}"#)
                .expect("defaults deserialise");
        assert_eq!(dock.anchor, DockAnchor::Page);
        assert_eq!(dock.surface, DockSurface::Glass);
        assert_eq!(dock.order, 0);
    }

    #[test]
    fn every_dock_placement_round_trips() {
        for placement in [
            DockPlacement::Top,
            DockPlacement::Bottom,
            DockPlacement::Left,
            DockPlacement::Right,
            DockPlacement::TopLeft,
            DockPlacement::TopRight,
            DockPlacement::BottomLeft,
            DockPlacement::BottomRight,
        ] {
            let text = serde_json::to_string(&placement).expect("serialises");
            let back: DockPlacement = serde_json::from_str(&text).expect("parses");
            assert_eq!(back, placement);
        }
    }

    #[test]
    fn a_kanban_template_carries_its_lanes_and_drag_rules() {
        let template = PanelTemplate {
            id: "node-list".to_string(),
            title: "Nodes".to_string(),
            title_key: None,
            description: None,
            description_key: None,
            render_engine: "celestia-kanban".to_string(),
            spec: ViewTemplateSpec {
                base: BaseViewKind::Kanban,
                waterfall: None,
                node_canvas: None,
                data_grid: None,
                kanban: Some(KanbanSpec {
                    axes: ScrollAxes::Both,
                    lanes: LaneSpec {
                        field: "parent".to_string(),
                        header_is_card: true,
                    },
                    card_kinds: vec!["node-card".to_string()],
                    dnd: DndSpec {
                        across_lanes: true,
                        reorder: true,
                    },
                    windowing: Some(WindowingSpec {
                        source: WindowingSource::Host,
                        estimated_item_height: Some(150),
                        overscan_screens: None,
                    }),
                }),
                blank_canvas: None,
                chrome: Some(ChromeSpec {
                    docks: vec![DockContribution {
                        widget: "stats-card".to_string(),
                        placement: DockPlacement::Bottom,
                        anchor: DockAnchor::Plane,
                        surface: DockSurface::Glass,
                        order: 10,
                    }],
                    breadcrumb: true,
                }),
            },
        };
        let text = toml::to_string(&ViewTemplateFile {
            version: "1".to_string(),
            template: vec![template.clone()],
        })
        .expect("serialises");
        let back: ViewTemplateFile = toml::from_str(&text).expect("parses");
        assert_eq!(back.template[0], template);
    }
}
