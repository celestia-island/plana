//! HTTP DTOs of the celestia-island platform surface, exported by ts-rs to
//! `bindings/httpTypes.ts` and published as `@celestia-island/plana-types`.
//!
//! These structs are wire contracts, and the producers live in the service
//! repos (today mostly `shittim-chest` `packages/core`), so each field doc
//! states its JSON key, unit, value set and whether an optional field is
//! omitted or sent as `null` — rather than pointing at one implementation.
//! A few shapes have no producer left in the workspace; those say so and stay
//! for wire compatibility.

use plana::protocol_core::http::RbacGroup;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use ts_rs::TS;
/// One configured LLM provider as the console's Providers page reads it: the stored row with
/// its API key reduced to a masked hint — the raw key never crosses the wire.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ProviderPublic {
    /// Provider row id (UUID rendered as a string); every `ModelInfo` row of this provider
    /// repeats it as `provider_id`.
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    /// Operator-assigned display name. The router keys the model→provider map by this name, so
    /// two providers sharing a name collapse into one entry.
    pub name: String,
    /// OpenAI-compatible base URL requests are sent to; loopback and cloud-metadata hosts are
    /// rejected when a provider is saved.
    pub endpoint: String,
    /// Stored API key masked server-side — never the raw secret. The JSON-RPC path keeps
    /// `****` plus the last 4 characters (`****` alone for keys of 8 characters or fewer), the
    /// REST path serves a fixed `••••••••`; an empty string means no key is stored.
    pub api_key_masked: String,
    /// Model ids this provider serves; each one becomes a `ModelInfo` row in the model picker.
    pub models: Vec<String>,
    /// Capability class of the provider, `chat` for conversational endpoints; copied to every
    /// model row of this provider.
    pub category: String,
    /// Whether this provider answers for model ids no provider lists; when no row is marked,
    /// the first loaded provider is used instead.
    pub is_default: bool,
    /// Whether the provider takes part in routing; disabled rows stay in the list but are
    /// skipped when the registry is loaded.
    pub enabled: bool,
    /// Ascending sort key: lower sorts first. The console numbers providers in this order for
    /// the `#N` suffix on model tags.
    pub priority: i32,
}

/// One (provider, model) pair flattened for the model picker — a model id is only unique
/// together with the provider that serves it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ModelInfo {
    /// Model id exactly as the endpoint spells it (e.g. `deepseek-v4-flash`).
    pub id: String,
    /// Display name of the serving provider — the `ProviderPublic.name` the router keys on.
    pub provider_name: String,
    /// Id of the serving provider as a string, so the picker can deep-link to its row.
    pub provider_id: String,
    /// Capability class copied from the provider; `chat` for conversational endpoints.
    pub category: String,
}

/// One vendor preset of the create-provider wizard's catalogue (generated from the
/// provider-registry snapshot) — a template for a new provider, not a configured one.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct VendorInfo {
    /// Stable vendor slug (`anthropic`, `openai`, …) used as the wizard's preset key.
    pub id: String,
    /// Vendor name shown in the wizard.
    pub name: String,
    /// API base URL prefilled when this vendor is chosen.
    pub endpoint: String,
    /// Catalogue bucket from the registry snapshot (e.g. `cloud`); not the provider's own
    /// `category` capability class.
    pub category: String,
    /// Free-text blurb; the served catalogue sends an empty string today and the wizard does
    /// not render it.
    pub description: String,
    /// Newest-generation-first shortlist the wizard preselects: flagship chat models of this
    /// vendor that the live endpoint actually serves.
    pub recommended_models: Vec<String>,
    /// Per-plan model allowlists keyed by plan name; the served catalogue sends an empty object
    /// today.
    pub plan_models: HashMap<String, Vec<String>>,
}

/// Result of the wizard's live key check: the endpoint is dialled with the candidate key and
/// its model list comes back.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ValidateKeyResponse {
    /// Whether the dial succeeded — HTTP 2xx with a parseable model list.
    pub valid: bool,
    /// Model ids the endpoint reported; empty when the check failed.
    pub models: Vec<String>,
    /// Preselection shortlist drawn from `models` (flagship chat models, newest first); empty
    /// when the endpoint's order gives nothing to rank.
    pub recommended_models: Vec<String>,
    /// Failure text the wizard shows when `valid` is false; serialized as `null` on success
    /// because the field has no `skip_serializing_if`.
    pub error: Option<String>,
}
// ── Token usage ────────────────────────────────────────────

/// Reply of the token-usage query (`GET /chat/token-usage?since=&until=`): one bucket per
/// grouping value plus the window total.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct TokenUsageResponse {
    /// One bucket per grouping dimension value; empty when nothing was metered in the window.
    pub usage: Vec<UsageEntry>,
    /// Sum of tokens over all buckets, as a signed 64-bit count (the aggregation is an SQL SUM,
    /// hence i64 rather than the u64 used elsewhere).
    pub total_tokens: i64,
}

/// One usage bucket. `model` stays the bucket-key carrier whatever the grouping is, so the
/// group-by-user and group-by-workspace replies reuse this exact shape.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UsageEntry {
    /// Bucket key: normally a model id, but the user id or workspace id under those groupings;
    /// unattributed rows read `unknown`.
    pub model: String,
    /// Tokens attributed to this bucket; the TS mirror renders it as `bigint`.
    pub token_count: u64,
}

/// Usage dashboard payload: window totals plus per-model and per-day breakdowns.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UsageDataResponse {
    /// Window label the figures cover, free-form (`30d`, `monthly`) — the client echoes what it
    /// asked for and no producer normalises it.
    pub period: String,
    /// Tokens consumed in the window, across all models.
    pub total_tokens: u64,
    /// Spend attributable to those tokens, in US dollars — not cents.
    pub total_cost_usd: f64,
    /// Per-model rows, unordered.
    pub by_model: Vec<UsageModelEntry>,
    /// Per-day rows for the trend chart, unordered.
    pub by_day: Vec<UsageDayEntry>,
}

/// One row of the per-model usage breakdown.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UsageModelEntry {
    /// Model id this row aggregates.
    pub model: String,
    /// Tokens consumed by this model in the window.
    pub tokens: u64,
    /// Spend for this model in US dollars.
    pub cost_usd: f64,
    /// Requests charged to this model in the window.
    pub requests: u32,
}

/// One row of the per-day usage breakdown; rows carry no cost split, only volume.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UsageDayEntry {
    /// Day bucket key as a calendar date string; the producer decides its exact format.
    pub date: String,
    /// Tokens consumed on that day.
    pub tokens: u64,
    /// Requests served on that day.
    pub requests: u32,
}
// ── Proxy / System info ────────────────────────────────────

/// Host snapshot behind the console's System info tab (`admin.system.getInfo`), computed on
/// demand from the `sysinfo` crate; every usage field is a percentage, not an absolute.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ProxySystemInfo {
    /// Version of the serving backend build.
    pub version: String,
    /// Runtime identifier string, e.g. `rust 1.8x.y`; the field name survives from the
    /// Node-based panel and does not mean a Node runtime. Serialized as `nodeVersion`.
    #[serde(rename = "nodeVersion")]
    pub node_version: String,
    /// Composite host description: long OS version, CPU architecture and kernel version
    /// (e.g. `Ubuntu 24.04.1 LTS x86_64 (6.8.0-45-generic)`) — not a bare OS name.
    pub platform: String,
    /// Aggregate CPU load as a percentage, 0–100, rounded to one decimal; serialized as
    /// `cpuUsage`.
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f64,
    /// Used physical memory as a percentage of the total, 0–100, rounded to one decimal;
    /// serialized as `memoryUsage`.
    #[serde(rename = "memoryUsage")]
    pub memory_usage: f64,
    /// Used space of the root mount as a percentage of its size, 0–100, rounded to one
    /// decimal; serialized as `diskUsage`. Reads 0 when no size is available.
    #[serde(rename = "diskUsage")]
    pub disk_usage: f64,
}

/// Legacy rich host snapshot. No handler in the workspace serves this shape — the System
/// info tab reads `ProxySystemInfo` — so it is kept for wire compatibility only.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SystemInfoResponse {
    /// Version string of the serving instance, in the same format as `ProxySystemInfo.version`.
    pub version: String,
    /// Seconds the backend process has been running, counted from process start.
    pub uptime_secs: u64,
    /// Agent-fleet rollup; see `SystemInfoAgents`.
    pub agents: SystemInfoAgents,
    /// Host CPU, memory and disk counters, in the units the field names carry.
    pub resources: SystemInfoResources,
    /// Live connection counters, split by transport.
    pub connections: SystemInfoConnections,
    /// Database identity and load counters.
    pub database: SystemInfoDatabase,
}

/// Agent-fleet rollup of the legacy snapshot: one count per lifecycle bucket. No producer
/// fixes whether the buckets partition `total` or count containers rather than sessions.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SystemInfoAgents {
    /// Size of the fleet the snapshot covers; the per-state counts below are its breakdown.
    pub total: u32,
    /// Semantics TBC — see packages/celestia-types/bindings/httpTypes.ts:87 (the generated
    /// mirror is the only remaining definition of this rollup).
    pub running: u32,
    /// Semantics TBC — see packages/celestia-types/bindings/httpTypes.ts:87 (the generated
    /// mirror is the only remaining definition of this rollup).
    pub idle: u32,
    /// Semantics TBC — see packages/celestia-types/bindings/httpTypes.ts:87 (the generated
    /// mirror is the only remaining definition of this rollup).
    pub stopped: u32,
}

/// Host resource counters of the legacy snapshot; the units are the ones the field names
/// carry, and the CPU figure is the same quantity `ProxySystemInfo.cpuUsage` reports.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SystemInfoResources {
    /// Aggregate CPU load as a percentage, 0–100.
    pub cpu_usage_pct: f64,
    /// Physical memory in use, in gigabytes.
    pub memory_used_gb: f64,
    /// Physical memory installed, in gigabytes — the denominator for `memory_used_gb`.
    pub memory_total_gb: f64,
    /// Disk space in use, in gigabytes.
    pub disk_used_gb: f64,
    /// Total disk space, in gigabytes — the denominator for `disk_used_gb`.
    pub disk_total_gb: f64,
}

/// Live connection counters of the legacy snapshot, split by transport.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SystemInfoConnections {
    /// WebSocket connections the instance held at snapshot time.
    pub active_ws: u32,
    /// HTTP requests in flight at snapshot time (not a cumulative count).
    pub active_http: u32,
}

/// Database identity and load counters of the legacy snapshot.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SystemInfoDatabase {
    /// Database engine name, e.g. `postgresql`.
    pub engine: String,
    /// Database size in megabytes, as a whole number.
    pub size_mb: u32,
    /// Connections open to the database at snapshot time.
    pub connections: u32,
}
// ── Workspace / Project ────────────────────────────────────
// `WorkspaceItem` and `WorkspaceResolveResponse` used to live here and
// described a chest roster shape (`path` / `editor` / `git_branch` /
// `connected`) that no chest endpoint ever produced — reading them is what
// blanked the admin console's workspace roster for months (chest #884, which
// also removed their last would-be consumers). The one real chest wire type
// in this section, `AliasRegistryEntry`, stays: it matches the
// `workspace.registry` payload exactly. The authoritative roster row type now
// lives with its producer (`shittim-chest` `packages/webui/src/types/
// workspace.ts`), next to the handler that serialises it.

/// One entry of the per-user workspace alias registry (`workspace.registry`): the address a
/// user reaches a workspace by.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AliasRegistryEntry {
    /// Workspace id (UUID string) the alias resolves to.
    pub workspace_uuid: String,
    /// Human alias of the workspace, unique within its identity scope (one user, or one group
    /// when the entry is group-scoped); a colliding group alias supersedes the user's.
    pub alias: String,
    /// Short display id — the last 6 hex characters of the workspace UUID; globally unique and
    /// resolvable on its own.
    pub short_id: String,
}

/// One project row in the project picker: display metadata plus the manual ordering key.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ProjectItem {
    /// Project id (UUID string); scene configs and device models hang off it.
    pub id: String,
    /// Project name shown in the picker.
    pub name: String,
    /// Free-text description; `null` when unset.
    pub description: Option<String>,
    /// Ascending manual order the project list is sorted by; new projects default to 0.
    pub sort_order: u32,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
}
// ── Scene ──────────────────────────────────────────────────

/// Scene configuration of one project: background, optional ground and lighting, grid,
/// camera, bloom and the ambient level — the descriptor the preview renderer applies.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneConfigItem {
    /// Project this scene belongs to (UUID string); the scene config is keyed by it.
    pub project_id: String,
    /// Background colour as a hex string (e.g. `#0a0a1a`).
    pub background_color: String,
    /// Ground plane settings; serialized as `null` (not omitted) when the scene has none.
    pub ground: Option<SceneGround>,
    /// Ambient and directional light settings; serialized as `null` when the scene has none.
    pub lighting: Option<SceneLighting>,
    /// Grid helper settings; always present.
    pub grid: SceneGrid,
    /// Initial camera placement and its named bookmarks.
    pub camera: SceneCamera,
    /// Bloom post-processing parameters; always present.
    pub bloom: SceneBloom,
    /// Linear ambient-light multiplier the renderer applies (built-in scene: 0.6). Distinct
    /// from `lighting.ambient_intensity`, which the live renderer ignores.
    pub ambient_light_intensity: f64,
}

/// Ground plane of the scene, along with the grid overlay drawn on it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneGround {
    /// Whether the ground plane is drawn at all; the grid overlay follows `grid_visible`.
    pub enabled: bool,
    /// Ground extent along X, in scene units.
    pub size_x: f64,
    /// Ground extent along Z, in scene units.
    pub size_z: f64,
    /// Ground colour as a hex string.
    pub color: String,
    /// Height of the ground plane on the Y axis, in scene units; negative sinks it below the
    /// scene origin.
    pub y: f64,
    /// Whether the ground's own grid overlay is drawn (independent of `SceneGrid.visible`).
    pub grid_visible: bool,
    /// Extent of that grid overlay, in scene units.
    pub grid_size: u32,
    /// Number of cells the overlay is divided into.
    pub grid_divisions: u32,
    /// Grid line colour as a hex string.
    pub grid_color: String,
    /// Grid line opacity, 0–1 (built-in scene: 0.25).
    pub grid_opacity: f64,
}

/// Ambient and directional light settings of the scene.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneLighting {
    /// Ambient light colour as an RGB triple; components are linear, 0–1.
    pub ambient_color: [f64; 3],
    /// Ambient intensity as stored (built-in scene: 500). The live preview renderer takes the
    /// ambient level from `SceneConfigItem.ambient_light_intensity` and ignores this field.
    pub ambient_intensity: f64,
    /// Directional light colour as an RGB triple; components are linear, 0–1.
    pub directional_color: [f64; 3],
    /// Directional light intensity passed to the renderer; the preview falls back to 1 when the
    /// producer leaves it at its default.
    pub directional_intensity: f64,
    /// Light position as an XYZ triple in scene units (built-in scene: (15, 80, 30)).
    pub directional_position: [f64; 3],
}

/// The scene's standalone grid helper; `SceneGround` carries a second, independent grid.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneGrid {
    /// Whether the grid is drawn.
    pub visible: bool,
    /// Grid extent in scene units.
    pub size: u32,
    /// Number of cells the grid is divided into (built-in scene: 80 units over 40 cells).
    pub divisions: u32,
}

/// Camera placement of the scene: where it looks from, what it looks at, and optional named
/// bookmarks.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneCamera {
    /// Camera position in scene units.
    pub position: SceneVec3,
    /// Point the camera looks at, in scene units.
    pub target: SceneVec3,
    /// Named camera presets keyed by bookmark name; omitted (not null) when the scene has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bookmarks: Option<std::collections::HashMap<String, SceneCameraBookmark>>,
}

/// One named camera preset: the position and target restored when the bookmark is picked.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneCameraBookmark {
    /// Saved camera position, in scene units.
    pub position: SceneVec3,
    /// Saved look-at point, in scene units.
    pub target: SceneVec3,
}

/// A point or direction in scene space; the same three numbers mean different things
/// depending on which field carries them.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneVec3 {
    /// X component, in scene units.
    pub x: f64,
    /// Y component, in scene units — up in the renderer's frame.
    pub y: f64,
    /// Z component, in scene units.
    pub z: f64,
}

/// Bloom (glow) post-processing parameters.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SceneBloom {
    /// Bloom intensity (built-in scene: 0.5).
    pub strength: f64,
    /// Bloom spread radius (built-in scene: 0.4).
    pub radius: f64,
    /// Luminance above which pixels bloom (built-in scene: 0.85).
    pub threshold: f64,
}
// ── Channel ────────────────────────────────────────────────

/// One channel adapter in the runtime list: its platform key, state and inbound webhook
/// path. This is adapter state, not the stored config (`ChannelConfigDetail`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelListItem {
    /// Platform key: `telegram`, `discord`, `qqbot`, `lark`, `slack`, `wecom`, `teams`,
    /// `matrix`, `google_chat`, `line`, `irc` or `mattermost`.
    pub platform: String,
    /// Whether an adapter is running for this platform.
    pub enabled: bool,
    /// Bot display name the adapter reports; every built-in adapter currently reports none.
    pub bot_name: String,
    /// Path of the platform's inbound webhook on this instance, e.g. `/api/webhook/telegram`.
    pub webhook_path: String,
}

/// Reply of the channel list: one entry per running adapter.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelListResponse {
    /// Running adapters, in registry order.
    pub channels: Vec<ChannelListItem>,
}

/// Stored configuration of one channel adapter as the admin console reads it back — secrets
/// masked, ready to be edited in place.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelConfigDetail {
    /// Config row id (UUID string).
    pub id: String,
    /// Platform key the config belongs to (same value set as `ChannelListItem.platform`).
    pub platform: String,
    /// Whether the adapter may start for this config.
    pub enabled: bool,
    /// Operator-assigned name of the config.
    pub name: String,
    /// Free-text description; omitted (not null) when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// Bot token, masked server-side before serialization (`****` plus the last 4 characters,
    /// or `****` alone for short secrets); omitted when no token is stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bot_token: Option<String>,
    /// Application/client id of the bot — not a secret, so it travels unmasked; omitted when
    /// unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub app_id: Option<String>,
    /// Application secret, masked exactly like `bot_token`; omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub app_secret: Option<String>,
    /// Webhook verification token, masked like `bot_token`; omitted when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub verify_token: Option<String>,
    /// Endpoint base override for platforms that have a sandbox and a production host (e.g. QQ
    /// bot); omitted when the adapter uses its built-in default.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_base: Option<String>,
    /// Platform-specific extras; omitted (not null) when unset. Typed as a plain object in TS
    /// because each platform defines its own keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub extra_config: Option<serde_json::Value>,
    /// Inbound webhook path this config answers on.
    pub webhook_path: String,
    /// Text of the last failed connect or health check; omitted when the last attempt succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_error: Option<String>,
    /// RFC 3339 time of the last successful test connection; omitted when the config has never
    /// been tested.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_tested_at: Option<String>,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
}

/// One channel config plus whether a live adapter is currently registered for it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelConfigResponse {
    /// The stored config, with secret fields masked.
    pub config: ChannelConfigDetail,
    /// True when the registry holds a running adapter for this platform (or platform:instance)
    /// key; a disabled config is never active.
    pub active: bool,
}

/// Reply that wraps the channel config list.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelConfigsResponse {
    /// Config envelopes, one per stored row.
    pub configs: Vec<ChannelConfigResponse>,
}

/// One logged channel message (inbound intake or outbound send) in the admin message log.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelMessageItem {
    /// Log row id (UUID string).
    pub id: String,
    /// Platform the message travelled over; same value set as `ChannelListItem.platform`, plus
    /// `webhook` for ingress rows.
    pub platform: String,
    /// Flow direction. Every row the current logging paths write is `inbound`; outbound sends
    /// are not recorded in this log.
    pub direction: String,
    /// Platform-side message id, used to correlate replies.
    pub message_id: String,
    /// Platform-side conversation id the message belongs to.
    pub chat_id: String,
    /// Platform-side sender id; `null` for messages with no identifiable sender.
    pub sender_id: Option<String>,
    /// Message body as received or sent.
    pub text: String,
    /// Whether the conversation is a group chat rather than a direct message.
    pub is_group: bool,
    /// Group or conversation id when `is_group` is true, `null` otherwise; ingress rows reuse
    /// it to carry the webhook id.
    pub group_id: Option<String>,
    /// Error text when the message failed to deliver; `null` on success.
    pub error: Option<String>,
    /// RFC 3339 timestamp the row was written.
    pub created_at: String,
}

/// Reply of the channel message log (`admin.channels.messages`): newest first, capped.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ChannelMessageListResponse {
    /// Log rows, newest first, capped by the caller's limit (100 default, 500 max).
    pub messages: Vec<ChannelMessageItem>,
    /// Number of rows returned after that cap — not the total number of logged messages.
    pub count: usize,
}
// ── Agent ──────────────────────────────────────────────────

/// Roster row of one platform agent as the retired panel served it: identity, lifecycle
/// status, its tool list and container, plus the per-user marketplace flags. No handler in
/// the workspace produces this shape any more — the live fleet DTO is `TuiAgentInfo` in the
/// ws bindings (`ws/agentLifecycle.ts`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AgentItem {
    /// Agent identity string; the row's tool list, skills and container hang off it.
    pub id: String,
    /// Agent name shown in the roster.
    pub name: String,
    /// What the agent does; non-optional on the wire, so the producer sends an empty string
    /// rather than null when it has no description.
    pub description: String,
    /// Agent family key — the `Agent` enum's wire value (e.g. `HubRis`) in the live fleet DTOs;
    /// a plain string here because this shape predates the enum.
    pub agent_type: String,
    /// Platform layer the agent sits on (0 = foundation primitives … 3 = service layer).
    pub layer: u32,
    /// Lifecycle status string; the live fleet types this as `AgentStatus`, whose wire values
    /// are `Initializing`, `Online`, `Busy`, `Offline` and `Error`.
    pub status: String,
    /// Whether the agent may be started at all; a disabled agent stays in the roster.
    pub enabled: bool,
    /// Tools the agent exposes, each with its own enable flag.
    pub tools: Vec<AgentTool>,
    /// Size of `tools`, serialized as `toolsCount`; carried alongside the array so a roster
    /// can render the count without walking it.
    #[serde(rename = "toolsCount")]
    pub tools_count: usize,
    /// Semantics TBC — see packages/celestia-types/tests/surface.rs:29 (the only site that
    /// still names this DTO).
    pub subscribed: bool,
    /// Semantics TBC — see packages/celestia-types/tests/surface.rs:29 (the only site that
    /// still names this DTO).
    pub installed: bool,
    /// Version string of the agent package this row describes.
    pub version: String,
    /// Runtime configuration of the agent: concurrency, timeout, retry, model and prompt.
    pub config: AgentConfig,
    /// Container backing the agent, or `null` when it has none running.
    pub container: Option<AgentContainer>,
    /// Semantics TBC — see packages/celestia-types/tests/surface.rs:29 (the only remaining
    /// site; the producer is gone, so the skill-id format is unfixed).
    pub skills: Vec<String>,
    /// Registration timestamp, serialized as `createdAt`.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Last-modified timestamp, serialized as `updatedAt`.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// One tool exposed by an agent in the roster.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AgentTool {
    /// Tool identity string that the agent's configuration references.
    pub id: String,
    /// Tool name shown in the roster's tool list.
    pub name: String,
    /// What the tool does, shown next to the name.
    pub description: String,
    /// Whether the agent may call this tool; disabled tools stay listed.
    pub enabled: bool,
}

/// Runtime limits and prompt defaults of one agent.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AgentConfig {
    /// How many tasks the agent runs at the same time.
    pub max_concurrent_tasks: u32,
    /// Per-task timeout in seconds.
    pub timeout_secs: u32,
    /// Whether a failed task is retried automatically before it is reported.
    pub retry_on_failure: bool,
    /// Model id the agent runs on.
    pub model: String,
    /// System prompt prepended to the agent's conversations.
    pub system_prompt: String,
}

/// Container instance backing a running agent.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AgentContainer {
    /// Container id as the container runtime reports it.
    pub id: String,
    /// Image reference the container was created from.
    pub image: String,
    /// Container lifecycle status; platform container operations use `Created`, `Running`,
    /// `Stopped`, `Removed` and `Forked`.
    pub status: String,
    /// Seconds the container has been up; the TS mirror renders it as `bigint`.
    pub uptime_secs: u64,
}
// ── Webhook ────────────────────────────────────────────────

/// One outbound webhook subscription (`admin.webhooks.*`) as the admin console reads it
/// back, with its signing secret masked.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WebhookItem {
    /// Subscription id (UUID string) that delivery records reference as `webhookId`.
    pub id: String,
    /// Operator-assigned name of the subscription.
    pub name: String,
    /// Target URL deliveries are posted to.
    pub url: String,
    /// Platform tag the subscription routes for; free-form string.
    pub platform: String,
    /// HMAC-SHA256 signing secret reduced to `****` plus its last 4 characters (`****` alone
    /// for short secrets) — the stored secret is never echoed.
    pub secret: String,
    /// Event names the subscription fires on; an empty array is served when the row stores none.
    pub events: Vec<String>,
    /// Subscription status string (`active`, `disabled`, …) exactly as stored.
    pub status: String,
    /// RFC 3339 time of the last delivery attempt, serialized as `lastDeliveryAt`; omitted
    /// (not null) when none has happened.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[serde(rename = "lastDeliveryAt")]
    pub last_delivery_at: Option<String>,
    /// RFC 3339 creation timestamp, serialized as `createdAt`.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// RFC 3339 last-update timestamp, serialized as `updatedAt`.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// One row of the webhook delivery log: what was sent, what came back and how long it took.
/// The live ingress log in the chest console carries the same data under slightly
/// different key names.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WebhookDeliveryItem {
    /// Delivery record id.
    pub id: String,
    /// Id of the `WebhookItem` subscription this delivery belongs to; serialized as `webhookId`.
    #[serde(rename = "webhookId")]
    pub webhook_id: String,
    /// Event that triggered the delivery; the live ingress path records the literal `ingress`.
    pub event: String,
    /// HTTP status the receiver returned, serialized as `statusCode`; the live ingress log
    /// records 200 on success and 401 when the signature check failed.
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    /// Headers sent with the request, serialized as `requestHeaders`.
    #[serde(rename = "requestHeaders")]
    pub request_headers: HashMap<String, String>,
    /// JSON body that was sent, serialized as `requestBody`; omitted entirely when the value
    /// is JSON `null`.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    #[serde(rename = "requestBody")]
    pub request_body: serde_json::Value,
    /// Headers the receiver answered with, serialized as `responseHeaders`; omitted (not null)
    /// when the delivery never got a response.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[serde(rename = "responseHeaders")]
    pub response_headers: Option<HashMap<String, String>>,
    /// Receiver response body as text, serialized as `responseBody` — a string, not parsed JSON.
    #[serde(rename = "responseBody")]
    pub response_body: String,
    /// Semantics TBC — see the `durationMs` sibling in
    /// shittim-chest packages/core/src/proxy/handlers/channels.rs:1055, the only place the
    /// unit is named.
    pub duration: u64,
    /// Whether the receiver accepted the delivery; the live ingress log derives it from the
    /// signature check.
    pub success: bool,
    /// RFC 3339 timestamp of the attempt, serialized as `deliveredAt`.
    #[serde(rename = "deliveredAt")]
    pub delivered_at: String,
}

/// Provider-agnostic webhook delivery row; the `Gen` variant is the one that names its
/// units, unlike `WebhookDeliveryItem`.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WebhookDeliveryGenItem {
    /// Delivery record id.
    pub id: String,
    /// Id of the subscription the delivery belongs to.
    pub webhook_id: String,
    /// Event name that triggered the delivery.
    pub event: String,
    /// HTTP status code the receiver returned; named `status` because this shape predates
    /// `status_code`.
    pub status: u16,
    /// Delivery duration in milliseconds.
    pub duration_ms: u64,
    /// RFC 3339 time of the delivery attempt.
    pub timestamp: String,
    /// Headers sent with the request.
    pub request_headers: HashMap<String, String>,
    /// Parsed JSON body the receiver answered with; omitted entirely when it is JSON `null`.
    /// The request side is not recorded in this shape.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    pub response_body: serde_json::Value,
}
// ── Skill ──────────────────────────────────────────────────

/// One parameter of a skill's signature, as the skill catalogue declares it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SkillParameterItem {
    /// Parameter name as the skill expects it on the call.
    pub name: String,
    /// Declared parameter type, serialized as `type` (renamed because `type` is a Rust keyword).
    #[serde(rename = "type")]
    pub param_type: String,
    /// Value applied when the caller omits the parameter; omitted (not null) when there is none.
    /// Typed `unknown` in TS because the JSON type follows `param_type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub default: Option<serde_json::Value>,
    /// Free-text description; serialized as `null` when absent (no `skip_serializing_if`).
    pub description: Option<String>,
    /// Whether the caller must supply the parameter.
    pub required: bool,
}

/// One skill in the catalogue (`skills.list`): who may run it, what it needs and how long
/// it typically takes.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SkillItem {
    /// Stable skill id the agent tooling dispatches on.
    pub skill_id: String,
    /// Skill name shown in the picker.
    pub name: String,
    /// What the skill does, shown beside the name.
    pub description: String,
    /// Catalogue grouping of the skill.
    pub category: String,
    /// Agent family that owns the skill.
    pub agent: String,
    /// Agent families allowed to run the skill; more than one when it is shared.
    pub agent_types: Vec<String>,
    /// Declared parameters, in signature order.
    pub parameters: Vec<SkillParameterItem>,
    /// Typical wall-clock duration in seconds; the TS mirror renders it as `bigint`.
    pub estimated_duration_secs: u64,
}
// ── Tool ───────────────────────────────────────────────────

/// One tool in the platform tool catalogue: its JSON-Schema contract and the agent that
/// owns it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ToolItem {
    /// Stable tool id the agent dispatches on.
    pub tool_id: String,
    /// Tool name shown in the picker.
    pub name: String,
    /// What the tool does, shown beside the name.
    pub description: String,
    /// Catalogue grouping of the tool.
    pub category: String,
    /// Agent family that owns the tool.
    pub agent: String,
    /// JSON Schema of the tool's arguments; typed as a plain object in TS and never null.
    #[ts(type = "Record<string, unknown>")]
    pub input_schema: serde_json::Value,
    /// JSON Schema of the tool's result; omitted (not null) when the tool declares none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub output_schema: Option<serde_json::Value>,
}

// ── User / Device / Session / File ──────────────────────────

/// One avatar source the picker offers: a label plus a URL template the server fills in
/// (`auth.avatarPlatforms.list`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AvatarPlatformResponse {
    /// Row id (UUID string).
    pub id: String,
    /// Stable slug derived from the label (lowercase, dashes); the admin UI keys rows by it.
    pub slug: String,
    /// Display name shown in the picker.
    pub label: String,
    /// Tera template carrying the domain and its placeholders (`{{ username }}`,
    /// `{{ md5_email }}`, …); rendered per user server-side whenever an avatar URL is needed.
    pub url_template: String,
    /// Short usage hint for the picker; `null` when the source has none.
    pub hint: Option<String>,
    /// Whether users may pick the source; the user-facing list filters disabled rows out.
    pub enabled: bool,
    /// Ascending display order — the picker sorts by it.
    pub sort_order: i32,
}

/// First-run probe the client boots through: whether the instance still needs its initial
/// administrator, its default locale and whether self-registration is open.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SetupCheckResponse {
    /// True while the instance has no accounts at all; the client then routes to setup.
    pub needs_setup: bool,
    /// Instance default locale from system settings; omitted (not null) when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub locale: Option<String>,
    /// Whether self-registration is open. The server ANDs the stored toggle with "at least one
    /// account exists", so a fresh instance always reports false here.
    pub registration_enabled: bool,
}

/// Per-user UI preferences, stored under `auth_users.preferences` with camelCase keys and
/// echoed to the client in that same spelling.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UserPreferences {
    /// Theme id the user picked (e.g. `shittim`); omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub theme: Option<String>,
    /// Light/dark mode, serialized as `themeMode`; omitted when unset.
    #[serde(default, rename = "themeMode", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub theme_mode: Option<String>,
    /// Chat layout mode the user selected (the console's chat-mode tab key), serialized as
    /// `chatMode`; omitted when unset.
    #[serde(default, rename = "chatMode", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chat_mode: Option<String>,
    /// UI language tag (e.g. `zh-Hans`); omitted when unset, and the client then falls back to
    /// the instance default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub locale: Option<String>,
}

/// The signed-in account as `auth.getMe` returns it: account fields, the derived role, RBAC
/// groups and stored preferences.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UserProfileResponse {
    /// Account id (UUID string).
    pub id: String,
    /// Login name; unique per instance.
    pub username: String,
    /// Account email, or an empty string when the account has none.
    pub email: String,
    /// Name shown in the UI, or an empty string when unset.
    pub display_name: String,
    /// Resolved avatar URL; `null` when the account has no avatar.
    pub avatar_url: Option<String>,
    /// Whether the account may sign in; a deactivated account stays visible to admins.
    pub is_active: bool,
    /// Effective role derived from built-in group membership: `admin`, `operator`, `viewer` or
    /// `member`.
    pub role: String,
    /// Explicit RBAC group memberships, each with id, name, description and timestamps.
    pub groups: Vec<RbacGroup>,
    /// Stored UI preferences; omitted (not null) when the account has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferences: Option<UserPreferences>,
    /// Account creation timestamp as this handler renders it — the database timestamp's plain
    /// string form, not RFC 3339. Every other timestamp on this DTO family is RFC 3339.
    pub created_at: String,
}

/// Reply of `auth.setAvatar` / `auth.removeAvatar`: the account's avatar URL after the change.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct AvatarUpdateResponse {
    /// New avatar URL; `null` after a successful removal.
    pub avatar_url: Option<String>,
}
/// One device registered to an account, plus the last-known presence state of its row.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct DeviceResponse {
    /// Row id (UUID string); device sessions point at it through their own `device_id` column.
    pub id: String,
    /// Device-supplied identifier of the machine or browser, unique per account: re-registering
    /// the same device resolves to the same row rather than adding one.
    pub device_id: String,
    /// Operator-visible device name.
    pub name: String,
    /// Device class stored on the row; the schema default is `both`.
    pub device_type: String,
    /// Presence status stored on the row; the schema default is `unknown`.
    pub status: String,
    /// Time of the last contact; `null` when the device has never connected.
    pub last_seen_at: Option<String>,
    /// Free-form device metadata (arch, GPUs, …); omitted (not null) when the row has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub metadata: Option<serde_json::Value>,
    /// Registration timestamp.
    pub created_at: String,
}

/// Reply of the WebRTC device-session create: the session handle plus its first signaling
/// payload.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct SessionCreateResponse {
    /// Session id (UUID string) that every later offer, answer and ICE call carries.
    pub session_id: String,
    /// Session status: a freshly created session is `pending`, turns `active` on the first
    /// offer and `closed` when torn down.
    pub status: String,
    /// WebRTC signaling payload (SDP offer/answer or ICE candidate) to hand to the peer; omitted
    /// (not null) when there is nothing to relay yet.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    pub signaling: serde_json::Value,
}

/// Legacy directory-listing payload: a directory path plus the entries found in it.
/// No handler in the workspace serves this shape any more — the unified file picker
/// resolves directories through `Sync.RequestFileTree` — so it is kept for wire
/// compatibility only.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct FileListingResponse {
    /// Directory the listing was taken from, spelled the way the producer reported it.
    pub path: String,
    /// One row per directory entry; order is whatever the producer walked.
    pub entries: Vec<FileEntry>,
}

/// One directory entry of the legacy listing. No producer in the workspace emits it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct FileEntry {
    /// Entry name only — the parent directory lives on `FileListingResponse.path`.
    pub name: String,
    /// Entry kind, serialized as `type` (renamed because `type` is a Rust keyword).
    #[serde(rename = "type")]
    pub entry_type: String,
    /// Size in bytes, `null` for entries that have no size; the TS mirror renders it
    /// as `bigint | null`.
    pub size: Option<i64>,
}

/// One inbound webhook endpoint this instance exposes (`GET /api/webhook`). An entry
/// exists only for a provider whose signing secret is configured, plus the
/// always-present `custom` catch-all.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WebhookInfoItem {
    /// Provider key: `github`, `gitlab`, `gitee` or `custom`.
    pub name: String,
    /// Absolute ingress URL built from the configured public webhook base (`{base}/{name}`);
    /// the `custom` entry keeps a `{name}` placeholder for the caller to fill in.
    pub url: String,
    /// Provider event names the endpoint accepts (`push`, `pull_request`, `issues`, …);
    /// the `custom` endpoint advertises `any`.
    pub events: Vec<String>,
}

/// Reply of the inbound-webhook listing: every endpoint the instance currently accepts.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WebhookListResponse {
    /// Available endpoints, in provider order (github, gitlab, gitee, then custom).
    pub webhooks: Vec<WebhookInfoItem>,
}

/// Reply of the webhook delivery log (`GET /api/webhook/deliveries?limit=`); rows are
/// free-form because the log is generic over ingress sources.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct DeliveryListResponse {
    /// Delivery records as stored — the live log writes `{id, source, event_type,
    /// payload_hash, success, error, created_at}` — but the field stays opaque JSON so new
    /// sources can add keys without a type change.
    #[ts(type = "Array<Record<string, unknown>>")]
    pub deliveries: Vec<serde_json::Value>,
    /// Number of rows actually returned after the limit clamp (100 default, 500 max) — not the
    /// log's total size.
    pub count: usize,
}

/// State of the webhook IP allowlist: the enforcement toggle plus the per-source CIDR sets
/// it evaluates.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct IpWhitelistResponse {
    /// Whether the allowlist is enforced; when false every caller passes the check.
    pub enabled: bool,
    /// Per-source entries — `{source, cidrs}` from the built-in github/gitlab/gitee sets — left
    /// as opaque JSON so operators can extend them.
    #[ts(type = "Array<Record<string, unknown>>")]
    pub whitelist: Vec<serde_json::Value>,
}

/// First visible line range of an editor viewport, as the IDE extension reports it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct CursorVisibleRange {
    /// First visible line, 1-based inclusive.
    pub start: u32,
    /// Last visible line, 1-based inclusive.
    pub end: u32,
}

/// Editor cursor snapshot: the IDE extensions POST one to `/api/workspace/cursor` on a
/// debounce, and the console replays the last one as the live-cursor indicator.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct CursorState {
    /// Workspace the cursor belongs to (the id handed out at session create); omitted, not
    /// null, when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub workspace_id: Option<String>,
    /// Edited file, workspace-relative; omitted when no file is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub file: Option<String>,
    /// Caret line, 1-based.
    pub line: u32,
    /// Caret column, 1-based.
    pub column: u32,
    /// Line count of the file, for the minimap; omitted when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub total_lines: Option<u32>,
    /// Editor language id (`typescript`, `rust`, or a vim filetype); omitted when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language: Option<String>,
    /// Viewport range, drawn as the peer's visible band; omitted when the editor reports none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub visible_range: Option<CursorVisibleRange>,
}

/// Roster row of one editor/agent workspace session, as `workspace.createSession` echoes it
/// and as the workspace registry reads it back.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct WorkspaceSessionResponse {
    /// Session id (the workspace UUID as a string); every cursor and heartbeat call carries it.
    pub workspace_id: String,
    /// Absolute path of the workspace root on the machine that opened the session.
    pub workspace_path: String,
    /// Editor or agent that owns the session (`vscode`, `neovim`, …).
    pub editor_name: String,
    /// Version string of that editor build, used to spot outdated extensions.
    pub editor_version: String,
    /// Branch checked out in the workspace, as reported when the session connected.
    pub git_branch: String,
    /// Latest cursor snapshot; omitted (not null) until the client has reported one.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cursor: Option<CursorState>,
    /// RFC 3339 time the session was created.
    pub connected_at: String,
    /// RFC 3339 time of the last heartbeat; a stale value is what marks the session offline.
    pub last_heartbeat: String,
}

// ── Resource Quotas / Allocation ────────────────────────────

/// One resource quota row: a ceiling plus the usage booked against it for a single
/// resource dimension.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ResourceQuota {
    /// Stable row id (e.g. `quota-cpu`).
    pub id: String,
    /// Display name of the quota row.
    pub name: String,
    /// Metered dimension; the served defaults are `cpu`, `memory`, `disk` and `agents`.
    pub resource_type: String,
    /// Ceiling expressed in `limit_unit`. A non-positive value yields 0% utilization rather
    /// than an infinite or NaN percentage.
    #[ts(type = "number")]
    pub limit_value: f64,
    /// Unit both the ceiling and the usage are expressed in (`cores`, `MB`, `count`); free-form.
    pub limit_unit: String,
    /// Usage booked against the ceiling, in `limit_unit`.
    #[ts(type = "number")]
    pub used_value: f64,
    /// Reset window label; the served quota defaults use `monthly`, the live token view `30d`.
    pub period: String,
    /// Tier the quota belongs to (`standard`); a quota-only field that the usage summary drops.
    pub tier: String,
    /// Whether the row is enforced.
    pub enabled: bool,
}

/// Reply of `usage.quotas.list`: the quota rows the console renders as cards.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ResourceQuotaListResponse {
    /// Quota rows; the served default set is cpu, memory, disk and agents.
    pub quotas: Vec<ResourceQuota>,
}

/// One quota-gauge card. Derived from a quota row: id, name and tier are deliberately
/// dropped, so this is the shape the dashboard renders rather than the stored row.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ResourceUsageSummary {
    /// Metered dimension, falling back to `unknown` for partial rows.
    pub resource_type: String,
    /// Usage booked against the quota, in `unit`.
    #[ts(type = "number")]
    pub current_usage: f64,
    /// Unit of both `current_usage` and `limit`.
    pub unit: String,
    /// Ceiling in `unit`; 0 for partial rows.
    #[ts(type = "number")]
    pub limit: f64,
    /// Reset window label, defaulting to `monthly`.
    pub period: String,
    /// `current_usage / limit` as a percentage rounded to two decimals; 0 when the limit is not
    /// positive, so it is never infinite or NaN.
    #[ts(type = "number")]
    pub utilization_pct: f64,
}

/// Reply of the resource-usage gauge endpoint.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ResourceUsageResponse {
    /// Gauge rows — one per quota row, or one per metered subject in the live 30-day token view.
    pub summary: Vec<ResourceUsageSummary>,
}

// ── User Tier / Payment ──────────────────────────────────────

/// Legacy per-user tier snapshot: the tier, its expiry and the quota consumed so far.
/// No handler in the workspace emits this shape — the quota views read the billing
/// self-overview — so it is kept for wire compatibility only.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UserTierInfo {
    /// Account the snapshot describes, as a string UUID.
    pub user_id: String,
    /// Tier slug the account is on; `TierDefinition.tier` supplies the value set.
    pub tier: String,
    /// RFC 3339 expiry of the tier; omitted (not null) when the tier does not expire.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tier_expires_at: Option<String>,
    /// Quota consumed in the current day, in the tier's own quota unit.
    #[ts(type = "number")]
    pub daily_quota_used: f64,
    /// Tokens consumed in the current month.
    #[ts(type = "number")]
    pub monthly_token_used: f64,
    /// RFC 3339 time of the last quota reset; omitted when the account has never been reset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_quota_reset_at: Option<String>,
}

/// One entry of the tier catalogue the instance serves; the console renders these as plan
/// cards and `UpdateUserTierPayload` moves accounts between them.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct TierDefinition {
    /// Tier slug — `free`, `pro` and `enterprise` in the built-in catalogue.
    pub tier: String,
    /// Requests allowed per day; `-1` means unlimited (the built-in enterprise row).
    #[ts(type = "number")]
    pub daily_request_limit: f64,
    /// Tokens allowed per month.
    #[ts(type = "number")]
    pub monthly_token_limit: f64,
    /// Concurrent sessions allowed; `-1` means unlimited.
    #[ts(type = "number")]
    pub max_sessions: f64,
    /// Display price string (`$0`, `$29/mo`, `Custom`) — presentation text, not a number to
    /// compute with.
    pub price: String,
}

/// Reply of the tier catalogue endpoint.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct TierListResponse {
    /// Served tiers in catalogue order (free, pro, enterprise).
    pub tiers: Vec<TierDefinition>,
}

/// Admin request body that moves one account onto another tier.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct UpdateUserTierPayload {
    /// Account to move, as a string UUID.
    pub user_id: String,
    /// Target tier slug; must be one of the catalogue's `TierDefinition.tier` values.
    pub tier: String,
}
