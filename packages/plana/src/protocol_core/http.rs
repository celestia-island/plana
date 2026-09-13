use serde::Serialize;
use ts_rs::TS;

// ── Generic / Status ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct OkIdResponse {
    pub ok: bool,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct IdResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct CreatedResponse {
    pub created: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
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

/// Service health status. Serde uses lowercase so JSON returns "ok" etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Ok,
    Degraded,
    Unhealthy,
}

/// Backend build profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Dev,
    Nightly,
    Prod,
    Mock,
}

/// Standard /api/health response for all plana backends.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct HealthResponse {
    pub status: ServiceStatus,
    pub version: String,
    pub kind: BackendKind,
    pub uptime: u64,
    pub network: NetworkInfo,
    pub build_hash: Option<String>,
    pub engine_version: Option<String>,
}

impl HealthResponse {
    pub fn ok(
        version: impl Into<String>,
        kind: BackendKind,
        uptime: u64,
        network: NetworkInfo,
    ) -> Self {
        Self {
            status: ServiceStatus::Ok,
            version: version.into(),
            kind,
            uptime,
            network,
            build_hash: None,
            engine_version: None,
        }
    }

    /// Attach the build identity to a health payload.
    ///
    /// [`HealthResponse::ok`] deliberately leaves the hash empty, which is why
    /// every backend used to report `build_hash: null` on `/health` while its
    /// JSON-RPC twin said nothing at all. Services that know their revision
    /// chain this (typically with `plana_build_info::build_hash!()` and
    /// `build_kind!()`) so HTTP and RPC answer with the same triple.
    pub fn with_build(
        mut self,
        build_hash: impl Into<String>,
        engine_version: Option<String>,
    ) -> Self {
        let hash = build_hash.into();
        if !hash.is_empty() {
            self.build_hash = Some(hash);
        }
        self.engine_version = engine_version;
        self
    }

    /// The version identity of this service, in the one shape every backend
    /// must expose on both `/health` and its `Service.Info` JSON-RPC method.
    pub fn version_report(&self) -> VersionReport {
        VersionReport {
            version: self.version.clone(),
            build_hash: self.build_hash.clone(),
            kind: self.kind,
            engine_version: self.engine_version.clone(),
        }
    }
}

/// The version half of a health payload, shared by HTTP and JSON-RPC.
///
/// Backends answer `Service.Info` with exactly this struct, built from the
/// same [`HealthResponse`] they serve on `/health`, so the two transports can
/// never drift apart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct VersionReport {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub build_hash: Option<String>,
    pub kind: BackendKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub engine_version: Option<String>,
}

/// Wire method name of the JSON-RPC twin of `/health`. Every backend
/// registers this method and answers with [`VersionReport`].
pub const SERVICE_INFO_METHOD: &str = "Service.Info";

/// Network context from the incoming request.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct NetworkInfo {
    pub transport: String,
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<u32>,
}

impl NetworkInfo {
    pub fn unknown() -> Self {
        Self {
            transport: "sse".into(),
            region: "XX".into(),
            asn: None,
        }
    }
}

/// Connection status of a dependency, as reported in health payloads.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub latency: u64,
    #[serde(rename = "lastCheck")]
    pub last_check: String,
}

// ── RBAC ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
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
#[ts(export, export_to = "httpTypes.ts")]
pub struct RbacUsersResponse {
    pub users: Vec<RbacUser>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct RbacGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct RbacGroupsResponse {
    pub groups: Vec<RbacGroup>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct MyPermissions {
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct PermissionsResponse {
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
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

impl GrantItem {
    /// Validate that the `permission` field holds a valid Permission path
    /// (leaf node like `"agent.read"`) or domain name (branch like `"agent"`).
    /// Returns `None` when valid; otherwise returns the invalid path.
    #[must_use]
    pub fn validate_permission(&self) -> Option<&str> {
        let valid = crate::protocol_core::rbac::Permission::from_path(&self.permission).is_some()
            || !crate::protocol_core::rbac::Permission::expand_domain(&self.permission).is_empty();
        if valid {
            None
        } else {
            Some(&self.permission)
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct GrantListResponse {
    pub grants: Vec<GrantItem>,
}

// ── OAuth ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct OAuthProvider {
    pub provider: String,
    pub client_id: String,
    pub client_secret_masked: String,
    pub public_domain: String,
    pub enabled: bool,
}

// ── Common response helpers ────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct DeletedResponse {
    pub deleted: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct OkMessageResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "httpTypes.ts")]
pub struct ReadinessResponse {
    pub status: String,
    pub database: bool,
}
