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
    /// Format version. A reader that does not know the version must reject
    /// the file rather than guess — the same explicit stance the MDD
    /// descriptor takes for its `schema_version`.
    pub version: u32,
    /// The templates the file declares.
    #[serde(default)]
    pub template: Vec<PanelTemplate>,
}

/// The plugin-id shape the workspace-module contract fixes for render
/// engines (`^[a-z][a-z0-9-]*$`), reused for template ids so the two
/// vocabularies cannot drift apart.
fn is_kebab_id(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The i18n-key shape the admin catalog accepts: dotted, identifier-ish
/// segments (`footer.addPanel.views.kanban.title`).
fn is_i18n_key(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) if first.is_ascii_alphabetic() => {}
                _ => return false,
            }
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
}

impl ViewTemplateSpec {
    /// Check that exactly the arm named by `base` is set.
    ///
    /// The type cannot express this — the arms are five independent options —
    /// so a loader has to ask: a template whose arm disagrees with its base
    /// would otherwise reach the renderer and be silently ignored.
    pub fn check_arms(&self) -> Result<(), String> {
        let settings = [
            (BaseViewKind::Waterfall, self.waterfall.is_some()),
            (BaseViewKind::NodeCanvas, self.node_canvas.is_some()),
            (BaseViewKind::DataGrid, self.data_grid.is_some()),
            (BaseViewKind::Kanban, self.kanban.is_some()),
            (BaseViewKind::BlankCanvas, self.blank_canvas.is_some()),
        ];
        let set: Vec<BaseViewKind> = settings
            .iter()
            .filter(|(_, on)| *on)
            .map(|(kind, _)| *kind)
            .collect();
        match set.as_slice() {
            [only] if *only == self.base => Ok(()),
            [only] => Err(format!(
                "spec arm {only:?} does not match base {:?}",
                self.base
            )),
            other => Err(format!(
                "exactly one spec arm must be set, found {}",
                other.len()
            )),
        }
    }

    /// Check the numeric bounds the types cannot carry.
    fn check_bounds(&self) -> Result<(), String> {
        if let Some(spec) = &self.waterfall {
            match &spec.columns {
                WaterfallColumns::Fixed { count } if *count == 0 => {
                    return Err("waterfall columns: fixed count must be >= 1".to_string());
                }
                WaterfallColumns::Adaptive { max, min_width } if *max == 0 || *min_width == 0 => {
                    return Err(
                        "waterfall columns: adaptive max and min_width must be >= 1".to_string()
                    );
                }
                _ => {}
            }
        }
        if let Some(spec) = &self.node_canvas {
            let camera = &spec.camera;
            if !camera.min_zoom.is_finite()
                || !camera.max_zoom.is_finite()
                || camera.min_zoom <= 0.0
                || camera.min_zoom > camera.max_zoom
            {
                return Err(format!(
                    "node-canvas camera: need 0 < min_zoom <= max_zoom, got {}..{}",
                    camera.min_zoom, camera.max_zoom
                ));
            }
            if let Some(step) = camera.zoom_step {
                if !step.is_finite() || step <= 0.0 {
                    return Err("node-canvas camera: zoom_step must be finite and > 0".to_string());
                }
            }
        }
        Ok(())
    }
}

impl PanelTemplate {
    /// Validate everything the types cannot: the id and key shapes, the arm
    /// matching the base, and the numeric bounds.
    ///
    /// Call this at the loading boundary (TOML -> validate -> renderer),
    /// with the discipline the admin catalog already applies to untrusted
    /// module payloads: reject and say why, never hand the renderer
    /// something it will silently ignore.
    pub fn validate(&self) -> Result<(), String> {
        if !is_kebab_id(&self.id) {
            return Err(format!(
                "template id '{}' must match ^[a-z][a-z0-9-]*$",
                self.id
            ));
        }
        if !is_kebab_id(&self.render_engine) {
            return Err(format!(
                "render_engine '{}' must match ^[a-z][a-z0-9-]*$",
                self.render_engine
            ));
        }
        for (what, key) in [
            ("title_key", &self.title_key),
            ("description_key", &self.description_key),
        ] {
            if let Some(key) = key {
                if !is_i18n_key(key) {
                    return Err(format!("{what} '{key}' is not a dotted i18n key"));
                }
            }
        }
        self.spec.check_arms()?;
        self.spec.check_bounds()?;
        Ok(())
    }
}

impl ViewTemplateFile {
    /// Validate every template the file declares.
    pub fn validate(&self) -> Result<(), String> {
        for template in &self.template {
            template.validate()?;
        }
        Ok(())
    }
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
            version: 1,
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
    fn the_wire_spellings_of_the_base_views_are_pinned() {
        // The kebab spellings ARE the wire contract: switching to snake_case
        // would silently break every `base = "node-canvas"` a config file or
        // plugin writes, and the round-trip test cannot see it (kebab and
        // snake agree on the single-word variants).
        for (kind, expected) in [
            (BaseViewKind::Waterfall, "\"waterfall\""),
            (BaseViewKind::NodeCanvas, "\"node-canvas\""),
            (BaseViewKind::DataGrid, "\"data-grid\""),
            (BaseViewKind::Kanban, "\"kanban\""),
            (BaseViewKind::BlankCanvas, "\"blank-canvas\""),
        ] {
            assert_eq!(serde_json::to_string(&kind).expect("serialises"), expected);
        }
    }

    #[test]
    fn the_waterfall_columns_stay_internally_tagged() {
        // `{"kind": …}` is the shape a consumer discriminates on. Dropping
        // the tag turns the union into an externally tagged one, and every
        // `columns.kind` read misses without an error.
        assert_eq!(
            serde_json::to_string(&WaterfallColumns::Single).expect("serialises"),
            "{\"kind\":\"single\"}"
        );
        assert_eq!(
            serde_json::to_string(&WaterfallColumns::Fixed { count: 2 }).expect("serialises"),
            "{\"kind\":\"fixed\",\"count\":2}"
        );
        assert_eq!(
            serde_json::to_string(&WaterfallColumns::Adaptive {
                max: 3,
                min_width: 320
            })
            .expect("serialises"),
            "{\"kind\":\"adaptive\",\"max\":3,\"min_width\":320}"
        );
    }

    #[test]
    fn the_dock_placements_are_exactly_the_eight_anchors() {
        // Pinned as a SET, not merely "each variant round-trips": removing a
        // variant together with its entry in a round-trip list used to leave
        // the suite green while the published union silently lost an anchor.
        let mut spelled: Vec<String> = [
            DockPlacement::Top,
            DockPlacement::Bottom,
            DockPlacement::Left,
            DockPlacement::Right,
            DockPlacement::TopLeft,
            DockPlacement::TopRight,
            DockPlacement::BottomLeft,
            DockPlacement::BottomRight,
        ]
        .iter()
        .map(|placement| serde_json::to_string(placement).expect("serialises"))
        .collect();
        spelled.sort();
        assert_eq!(
            spelled,
            vec![
                "\"bottom\"",
                "\"bottom-left\"",
                "\"bottom-right\"",
                "\"left\"",
                "\"right\"",
                "\"top\"",
                "\"top-left\"",
                "\"top-right\"",
            ]
        );
    }

    #[test]
    fn validation_rejects_an_arm_that_contradicts_its_base() {
        let mut template = waterfall_template();
        template.spec.base = BaseViewKind::Kanban;
        let error = template.validate().expect_err("must be rejected");
        assert!(error.contains("does not match base"), "{error}");
    }

    #[test]
    fn validation_rejects_a_missing_arm() {
        let mut template = waterfall_template();
        template.spec.waterfall = None;
        let error = template.validate().expect_err("must be rejected");
        assert!(error.contains("exactly one spec arm"), "{error}");
    }

    #[test]
    fn validation_rejects_malformed_ids_and_keys() {
        for (label, mutate) in [
            (
                "id",
                Box::new(|t: &mut PanelTemplate| t.id = "Not An Id".to_string())
                    as Box<dyn Fn(&mut PanelTemplate)>,
            ),
            (
                "render_engine",
                Box::new(|t: &mut PanelTemplate| t.render_engine = "Celestia".to_string()),
            ),
            (
                "title_key",
                // A single bare segment IS a valid key; the malformed shape
                // is a segment that does not start with a letter.
                Box::new(|t: &mut PanelTemplate| t.title_key = Some("1bad.key".to_string())),
            ),
        ] {
            let mut template = waterfall_template();
            mutate(&mut template);
            assert!(template.validate().is_err(), "{label} must be rejected");
        }
        // The untouched template passes, so the rejections above are not the
        // validator refusing everything.
        assert!(waterfall_template().validate().is_ok());
    }

    #[test]
    fn validation_rejects_out_of_range_numbers() {
        let mut zero_columns = waterfall_template();
        if let Some(spec) = zero_columns.spec.waterfall.as_mut() {
            spec.columns = WaterfallColumns::Fixed { count: 0 };
        }
        assert!(zero_columns.validate().is_err());

        let inverted_zoom = PanelTemplate {
            id: "canvas".to_string(),
            title: "Canvas".to_string(),
            title_key: None,
            description: None,
            description_key: None,
            render_engine: "celestia-node-graph".to_string(),
            spec: ViewTemplateSpec {
                base: BaseViewKind::NodeCanvas,
                waterfall: None,
                node_canvas: Some(NodeCanvasSpec {
                    backend: CanvasBackend::Svg,
                    camera: CameraSpec {
                        min_zoom: 4.0,
                        max_zoom: 0.5,
                        zoom_step: None,
                        fit_on_load: true,
                    },
                    minimap: None,
                    node_kinds: Vec::new(),
                    palette: None,
                    inspector: None,
                    auto_layout: None,
                }),
                data_grid: None,
                kanban: None,
                blank_canvas: None,
                chrome: None,
            },
        };
        assert!(inverted_zoom.validate().is_err());
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
            version: 1,
            template: vec![template.clone()],
        })
        .expect("serialises");
        let back: ViewTemplateFile = toml::from_str(&text).expect("parses");
        assert_eq!(back.template[0], template);
    }
}
