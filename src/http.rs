use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use ts_rs::TS;

// ── Provider / Model / Vendor ──────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ProviderPublic {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    pub name: String,
    pub endpoint: String,
    pub api_key_masked: String,
    pub models: Vec<String>,
    pub category: String,
    pub is_default: bool,
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ModelInfo {
    pub id: String,
    pub provider_name: String,
    pub provider_id: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct VendorInfo {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub category: String,
    pub description: String,
    pub recommended_models: Vec<String>,
    pub plan_models: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ValidateKeyResponse {
    pub valid: bool,
    pub models: Vec<String>,
    pub recommended_models: Vec<String>,
    pub error: Option<String>,
}

// ── Generic / Status ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct OkIdResponse {
    pub ok: bool,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct IdResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct CreatedResponse {
    pub created: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct StatusResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plugin_id: Option<String>,
}

// ── Health ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub mock: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct HealthDetailed {
    pub shittimChest: ConnectionStatus,
    pub scepter: ConnectionStatus,
    pub database: ConnectionStatus,
    pub activeSessions: u32,
    pub uptime: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub latency: u64,
    #[serde(rename = "lastCheck")]
    pub last_check: String,
}

// ── Token usage ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct TokenUsageResponse {
    pub usage: Vec<UsageEntry>,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UsageEntry {
    pub model: String,
    pub token_count: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UsageDataResponse {
    pub period: String,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub by_model: Vec<UsageModelEntry>,
    pub by_day: Vec<UsageDayEntry>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UsageModelEntry {
    pub model: String,
    pub tokens: u64,
    pub cost_usd: f64,
    pub requests: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UsageDayEntry {
    pub date: String,
    pub tokens: u64,
    pub requests: u32,
}

// ── Proxy / System info ────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ProxySystemInfo {
    pub version: String,
    pub nodeVersion: String,
    pub platform: String,
    pub cpuUsage: f64,
    pub memoryUsage: f64,
    pub diskUsage: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SystemInfoResponse {
    pub version: String,
    pub uptime_secs: u64,
    pub agents: SystemInfoAgents,
    pub resources: SystemInfoResources,
    pub connections: SystemInfoConnections,
    pub database: SystemInfoDatabase,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SystemInfoAgents {
    pub total: u32,
    pub running: u32,
    pub idle: u32,
    pub stopped: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SystemInfoResources {
    pub cpu_usage_pct: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SystemInfoConnections {
    pub active_ws: u32,
    pub active_http: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SystemInfoDatabase {
    pub engine: String,
    pub size_mb: u32,
    pub connections: u32,
}

// ── RBAC ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct RbacUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub role: String,
    pub tier: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct RbacUsersResponse {
    pub users: Vec<RbacUser>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct RbacGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct RbacGroupsResponse {
    pub groups: Vec<RbacGroup>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct MyPermissions {
    pub role: String,
    pub permissions: Vec<String>,
}

// ── OAuth ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct OAuthProvider {
    pub provider: String,
    pub client_id: String,
    pub client_secret_masked: String,
    pub public_domain: String,
    pub enabled: bool,
}

// ── Workspace / Project ────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WorkspaceItem {
    pub id: String,
    pub path: String,
    pub editor: String,
    pub git_branch: String,
    pub status: String,
    pub connected: bool,
    pub short_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default = "default_connection_kind")]
    pub connection_kind: String,
}

fn default_connection_kind() -> String {
    "local_filesystem".to_string()
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AliasRegistryEntry {
    pub workspace_uuid: String,
    pub alias: String,
    pub short_id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WorkspaceResolveResponse {
    pub workspace_uuid: String,
    pub short_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ProjectItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: u32,
    pub created_at: String,
    pub updated_at: String,
}

// ── Scene ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneConfigItem {
    pub project_id: String,
    pub background_color: String,
    pub ground: Option<SceneGround>,
    pub lighting: Option<SceneLighting>,
    pub grid: SceneGrid,
    pub camera: SceneCamera,
    pub bloom: SceneBloom,
    pub ambient_light_intensity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneGround {
    pub enabled: bool,
    pub size_x: f64,
    pub size_z: f64,
    pub color: String,
    pub y: f64,
    pub grid_visible: bool,
    pub grid_size: u32,
    pub grid_divisions: u32,
    pub grid_color: String,
    pub grid_opacity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneLighting {
    pub ambient_color: [f64; 3],
    pub ambient_intensity: f64,
    pub directional_color: [f64; 3],
    pub directional_intensity: f64,
    pub directional_position: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneGrid {
    pub visible: bool,
    pub size: u32,
    pub divisions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneCamera {
    pub position: SceneVec3,
    pub target: SceneVec3,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SceneBloom {
    pub strength: f64,
    pub radius: f64,
    pub threshold: f64,
}

// ── Channel ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelListItem {
    pub platform: String,
    pub enabled: bool,
    pub bot_name: String,
    pub webhook_path: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelListResponse {
    pub channels: Vec<ChannelListItem>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelConfigDetail {
    pub id: String,
    pub platform: String,
    pub enabled: bool,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub app_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub verify_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub extra_config: Option<serde_json::Value>,
    pub webhook_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_tested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelConfigResponse {
    pub config: ChannelConfigDetail,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelConfigsResponse {
    pub configs: Vec<ChannelConfigResponse>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelMessageItem {
    pub id: String,
    pub platform: String,
    pub direction: String,
    pub message_id: String,
    pub chat_id: String,
    pub sender_id: Option<String>,
    pub text: String,
    pub is_group: bool,
    pub group_id: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ChannelMessageListResponse {
    pub messages: Vec<ChannelMessageItem>,
    pub count: usize,
}

// ── Agent ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AgentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_type: String,
    pub layer: u32,
    pub status: String,
    pub enabled: bool,
    pub tools: Vec<AgentTool>,
    pub toolsCount: usize,
    pub subscribed: bool,
    pub installed: bool,
    pub version: String,
    pub config: AgentConfig,
    pub container: Option<AgentContainer>,
    pub skills: Vec<String>,
    pub createdAt: String,
    pub updatedAt: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AgentTool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AgentConfig {
    pub max_concurrent_tasks: u32,
    pub timeout_secs: u32,
    pub retry_on_failure: bool,
    pub model: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AgentContainer {
    pub id: String,
    pub image: String,
    pub status: String,
    pub uptime_secs: u64,
}

// ── Webhook ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WebhookItem {
    pub id: String,
    pub name: String,
    pub url: String,
    pub platform: String,
    pub secret: String,
    pub events: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub lastDeliveryAt: Option<String>,
    pub createdAt: String,
    pub updatedAt: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WebhookDeliveryItem {
    pub id: String,
    pub webhookId: String,
    pub event: String,
    pub statusCode: u16,
    pub requestHeaders: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    pub requestBody: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub responseHeaders: Option<HashMap<String, String>>,
    pub responseBody: String,
    pub duration: u64,
    pub success: bool,
    pub deliveredAt: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WebhookDeliveryGenItem {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: String,
    pub request_headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    pub response_body: serde_json::Value,
}

// ── Skill ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SkillParameterItem {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SkillItem {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub agent: String,
    pub agent_types: Vec<String>,
    pub parameters: Vec<SkillParameterItem>,
    pub estimated_duration_secs: u64,
}

// ── Tool ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ToolItem {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub agent: String,
    #[ts(type = "Record<string, unknown>")]
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub output_schema: Option<serde_json::Value>,
}

// ── Common response helpers ────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct DeletedResponse {
    pub deleted: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct OkMessageResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ReadinessResponse {
    pub status: String,
    pub database: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AvatarPlatformResponse {
    pub id: String,
    pub slug: String,
    pub label: String,
    pub url_template: String,
    pub hint: Option<String>,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SetupCheckResponse {
    pub needs_setup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub locale: Option<String>,
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UserPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub theme: Option<String>,
    #[serde(rename = "themeMode", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub theme_mode: Option<String>,
    #[serde(rename = "chatMode", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chat_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UserProfileResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub role: String,
    pub groups: Vec<RbacGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferences: Option<UserPreferences>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct AvatarUpdateResponse {
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct PermissionsResponse {
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct GrantItem {
    pub id: String,
    pub scope: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub permission: String,
    pub resource_id: Option<String>,
    pub granted: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct GrantListResponse {
    pub grants: Vec<GrantItem>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct DeviceResponse {
    pub id: String,
    pub device_id: String,
    pub name: String,
    pub device_type: String,
    pub status: String,
    pub last_seen_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "Record<string, unknown>")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct SessionCreateResponse {
    pub session_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    #[ts(type = "Record<string, unknown>")]
    pub signaling: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct FileListingResponse {
    pub path: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct FileEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WebhookInfoItem {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WebhookListResponse {
    pub webhooks: Vec<WebhookInfoItem>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct DeliveryListResponse {
    #[ts(type = "Array<Record<string, unknown>>")]
    pub deliveries: Vec<serde_json::Value>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct IpWhitelistResponse {
    pub enabled: bool,
    #[ts(type = "Array<Record<string, unknown>>")]
    pub whitelist: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct CursorVisibleRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct CursorState {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub file: Option<String>,
    pub line: u32,
    pub column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub total_lines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub visible_range: Option<CursorVisibleRange>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct WorkspaceSessionResponse {
    pub workspace_id: String,
    pub workspace_path: String,
    pub editor_name: String,
    pub editor_version: String,
    pub git_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cursor: Option<CursorState>,
    pub connected_at: String,
    pub last_heartbeat: String,
}

// ── Resource Quotas / Allocation ────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ResourceQuota {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    #[ts(type = "number")]
    pub limit_value: f64,
    pub limit_unit: String,
    #[ts(type = "number")]
    pub used_value: f64,
    pub period: String,
    pub tier: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ResourceQuotaListResponse {
    pub quotas: Vec<ResourceQuota>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ResourceUsageSummary {
    pub resource_type: String,
    #[ts(type = "number")]
    pub current_usage: f64,
    pub unit: String,
    #[ts(type = "number")]
    pub limit: f64,
    pub period: String,
    #[ts(type = "number")]
    pub utilization_pct: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct ResourceUsageResponse {
    pub summary: Vec<ResourceUsageSummary>,
}

// ── User Tier / Payment ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UserTierInfo {
    pub user_id: String,
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tier_expires_at: Option<String>,
    #[ts(type = "number")]
    pub daily_quota_used: f64,
    #[ts(type = "number")]
    pub monthly_token_used: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_quota_reset_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct TierDefinition {
    pub tier: String,
    #[ts(type = "number")]
    pub daily_request_limit: f64,
    #[ts(type = "number")]
    pub monthly_token_limit: f64,
    #[ts(type = "number")]
    pub max_sessions: f64,
    pub price: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct TierListResponse {
    pub tiers: Vec<TierDefinition>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "HttpTypes.ts")]
pub struct UpdateUserTierPayload {
    pub user_id: String,
    pub tier: String,
}
