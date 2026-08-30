//! RBAC permission and role types for the generic protocol core.
//!
//! The permission vocabulary ([`Permission`]) is a v1 baseline **closed set**
//! of 31 typed variants. Extensibility is planned via a string-permission
//! mechanism, and the seam already exists: permissions travel over the wire
//! as dotted strings (`"agent.read"`, …), and the string-level APIs
//! (`Permission::from_path`, `parse_permission`, `validate_grant_permissions`,
//! `GrantItem::validate_permission`) already accept and validate arbitrary
//! dotted paths against the closed vocabulary. The typed enum stays closed
//! until the opaque string-permission mechanism lands; profiles must use the
//! 33 built-in leaves (or their domain prefixes) today.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// The three dimensions of access control.
///
/// (1) User        — individual account
/// (2) UserGroup   — collection of users sharing permissions
/// (3) ModelGroup  — collection of models with access policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessDimension {
    User,
    UserGroup,
    ModelGroup,
}

/// The scope in which a permission is evaluated.
/// Some permissions are system-wide, others are tied to a model group,
/// and yet others are user-personal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum PermissionScope {
    Global,
    ModelGroup(String),
    Personal,
}

/// Every discrete operation that can be authorised by a platform profile.
///
/// Serialised as a dotted path (`gateway.chat`, `model.deploy`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
pub enum Permission {
    // ── Gateway & Proxy ──────────────────────────────────────────
    #[serde(rename = "gateway.chat")]
    GatewayChat,
    #[serde(rename = "gateway.models")]
    GatewayListModels,
    #[serde(rename = "gateway.stream")]
    GatewayStream,

    // ── Conversations ────────────────────────────────────────────
    #[serde(rename = "conversation.create")]
    ConversationCreate,
    #[serde(rename = "conversation.read")]
    ConversationRead,
    #[serde(rename = "conversation.delete")]
    ConversationDelete,

    // ── API Keys ─────────────────────────────────────────────────
    #[serde(rename = "apikey.create")]
    ApiKeyCreate,
    #[serde(rename = "apikey.read")]
    ApiKeyRead,
    #[serde(rename = "apikey.revoke")]
    ApiKeyRevoke,

    // ── Model Management ─────────────────────────────────────────
    #[serde(rename = "model.deploy")]
    ModelDeploy,
    #[serde(rename = "model.stop")]
    ModelStop,
    #[serde(rename = "model.read")]
    ModelRead,
    #[serde(rename = "model.download")]
    ModelDownload,

    // ── Provider Management ──────────────────────────────────────
    #[serde(rename = "provider.create")]
    ProviderCreate,
    #[serde(rename = "provider.read")]
    ProviderRead,
    #[serde(rename = "provider.update")]
    ProviderUpdate,
    #[serde(rename = "provider.delete")]
    ProviderDelete,
    #[serde(rename = "provider.test")]
    ProviderTest,

    // ── Agent Management ─────────────────────────────────────────
    #[serde(rename = "agent.register")]
    AgentRegister,
    #[serde(rename = "agent.read")]
    AgentRead,
    #[serde(rename = "agent.deregister")]
    AgentDeregister,

    // ── User & Group Management ──────────────────────────────────
    #[serde(rename = "user.create")]
    UserCreate,
    #[serde(rename = "user.read")]
    UserRead,
    #[serde(rename = "user.update")]
    UserUpdate,
    #[serde(rename = "user.delete")]
    UserDelete,
    #[serde(rename = "group.create")]
    GroupCreate,
    #[serde(rename = "group.read")]
    GroupRead,
    #[serde(rename = "group.manage")]
    GroupManage,

    // ── System ───────────────────────────────────────────────────
    #[serde(rename = "system.settings")]
    SystemSettings,
    #[serde(rename = "system.metrics")]
    SystemMetrics,
    #[serde(rename = "system.admin")]
    SystemAdmin,
    // ── Quota & Credits ──────────────────────────────────────────
    #[serde(rename = "quota.read")]
    QuotaRead,
    #[serde(rename = "quota.manage")]
    QuotaManage,
}

impl Permission {
    /// Human-readable English description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::GatewayChat => "Use /v1/chat/completions",
            Self::GatewayListModels => "List available models",
            Self::GatewayStream => "Use streaming responses",
            Self::ConversationCreate => "Create conversations",
            Self::ConversationRead => "Read own conversations",
            Self::ConversationDelete => "Delete conversations",
            Self::ApiKeyCreate => "Create API keys",
            Self::ApiKeyRead => "List API keys",
            Self::ApiKeyRevoke => "Revoke API keys",
            Self::ModelDeploy => "Deploy models",
            Self::ModelStop => "Stop models",
            Self::ModelRead => "List/search models",
            Self::ModelDownload => "Download models from registry",
            Self::ProviderCreate => "Add providers",
            Self::ProviderRead => "List providers",
            Self::ProviderUpdate => "Update providers",
            Self::ProviderDelete => "Remove providers",
            Self::ProviderTest => "Test provider connection",
            Self::AgentRegister => "Register agents",
            Self::AgentRead => "List agents",
            Self::AgentDeregister => "Deregister agents",
            Self::UserCreate => "Create users",
            Self::UserRead => "List users",
            Self::UserUpdate => "Update users",
            Self::UserDelete => "Delete users",
            Self::GroupCreate => "Create groups",
            Self::GroupRead => "List groups",
            Self::GroupManage => "Add/remove members, change group permissions",
            Self::SystemSettings => "Read/update system settings",
            Self::SystemMetrics => "View metrics and logs",
            Self::SystemAdmin => "Full admin access (god mode)",
            Self::QuotaRead => "View quota and points balances",
            Self::QuotaManage => "Top up, allocate and adjust points",
        }
    }

    /// i18n key for localisation (e.g. `"permission.gateway.chat"`).
    pub fn i18n_key(&self) -> String {
        format!("permission.{}", self.as_str())
    }

    /// Return the scope in which this permission is evaluated.
    pub fn scope(&self) -> PermissionScope {
        match self {
            Self::GatewayChat
            | Self::GatewayListModels
            | Self::GatewayStream
            | Self::ModelDeploy
            | Self::ModelStop
            | Self::ModelRead
            | Self::ModelDownload
            | Self::ProviderCreate
            | Self::ProviderRead
            | Self::ProviderUpdate
            | Self::ProviderDelete
            | Self::ProviderTest => PermissionScope::ModelGroup(String::new()),

            Self::ConversationCreate
            | Self::ConversationRead
            | Self::ConversationDelete
            | Self::ApiKeyCreate
            | Self::ApiKeyRead
            | Self::ApiKeyRevoke => PermissionScope::Personal,

            Self::AgentRegister
            | Self::AgentRead
            | Self::AgentDeregister
            | Self::UserCreate
            | Self::UserRead
            | Self::UserUpdate
            | Self::UserDelete
            | Self::GroupCreate
            | Self::GroupRead
            | Self::GroupManage
            | Self::SystemSettings
            | Self::SystemMetrics
            | Self::SystemAdmin
            | Self::QuotaRead
            | Self::QuotaManage => PermissionScope::Global,
        }
    }

    /// Return the serde wire name (e.g. `"gateway.chat"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GatewayChat => "gateway.chat",
            Self::GatewayListModels => "gateway.models",
            Self::GatewayStream => "gateway.stream",
            Self::ConversationCreate => "conversation.create",
            Self::ConversationRead => "conversation.read",
            Self::ConversationDelete => "conversation.delete",
            Self::ApiKeyCreate => "apikey.create",
            Self::ApiKeyRead => "apikey.read",
            Self::ApiKeyRevoke => "apikey.revoke",
            Self::ModelDeploy => "model.deploy",
            Self::ModelStop => "model.stop",
            Self::ModelRead => "model.read",
            Self::ModelDownload => "model.download",
            Self::ProviderCreate => "provider.create",
            Self::ProviderRead => "provider.read",
            Self::ProviderUpdate => "provider.update",
            Self::ProviderDelete => "provider.delete",
            Self::ProviderTest => "provider.test",
            Self::AgentRegister => "agent.register",
            Self::AgentRead => "agent.read",
            Self::AgentDeregister => "agent.deregister",
            Self::UserCreate => "user.create",
            Self::UserRead => "user.read",
            Self::UserUpdate => "user.update",
            Self::UserDelete => "user.delete",
            Self::GroupCreate => "group.create",
            Self::GroupRead => "group.read",
            Self::GroupManage => "group.manage",
            Self::SystemSettings => "system.settings",
            Self::SystemMetrics => "system.metrics",
            Self::SystemAdmin => "system.admin",
            Self::QuotaRead => "quota.read",
            Self::QuotaManage => "quota.manage",
        }
    }

    /// Parse a dotted permission path (e.g. `"agent.read"`) back into a variant.
    pub fn from_path(s: &str) -> Option<Self> {
        match s {
            "gateway.chat" => Some(Self::GatewayChat),
            "gateway.models" => Some(Self::GatewayListModels),
            "gateway.stream" => Some(Self::GatewayStream),
            "conversation.create" => Some(Self::ConversationCreate),
            "conversation.read" => Some(Self::ConversationRead),
            "conversation.delete" => Some(Self::ConversationDelete),
            "apikey.create" => Some(Self::ApiKeyCreate),
            "apikey.read" => Some(Self::ApiKeyRead),
            "apikey.revoke" => Some(Self::ApiKeyRevoke),
            "model.deploy" => Some(Self::ModelDeploy),
            "model.stop" => Some(Self::ModelStop),
            "model.read" => Some(Self::ModelRead),
            "model.download" => Some(Self::ModelDownload),
            "provider.create" => Some(Self::ProviderCreate),
            "provider.read" => Some(Self::ProviderRead),
            "provider.update" => Some(Self::ProviderUpdate),
            "provider.delete" => Some(Self::ProviderDelete),
            "provider.test" => Some(Self::ProviderTest),
            "agent.register" => Some(Self::AgentRegister),
            "agent.read" => Some(Self::AgentRead),
            "agent.deregister" => Some(Self::AgentDeregister),
            "user.create" => Some(Self::UserCreate),
            "user.read" => Some(Self::UserRead),
            "user.update" => Some(Self::UserUpdate),
            "user.delete" => Some(Self::UserDelete),
            "group.create" => Some(Self::GroupCreate),
            "group.read" => Some(Self::GroupRead),
            "group.manage" => Some(Self::GroupManage),
            "system.settings" => Some(Self::SystemSettings),
            "system.metrics" => Some(Self::SystemMetrics),
            "system.admin" => Some(Self::SystemAdmin),
            "quota.read" => Some(Self::QuotaRead),
            "quota.manage" => Some(Self::QuotaManage),
            _ => None,
        }
    }

    /// Return all 31 leaf permissions.
    pub fn all() -> Vec<Self> {
        vec![
            Self::GatewayChat,
            Self::GatewayListModels,
            Self::GatewayStream,
            Self::ConversationCreate,
            Self::ConversationRead,
            Self::ConversationDelete,
            Self::ApiKeyCreate,
            Self::ApiKeyRead,
            Self::ApiKeyRevoke,
            Self::ModelDeploy,
            Self::ModelStop,
            Self::ModelRead,
            Self::ModelDownload,
            Self::ProviderCreate,
            Self::ProviderRead,
            Self::ProviderUpdate,
            Self::ProviderDelete,
            Self::ProviderTest,
            Self::AgentRegister,
            Self::AgentRead,
            Self::AgentDeregister,
            Self::UserCreate,
            Self::UserRead,
            Self::UserUpdate,
            Self::UserDelete,
            Self::GroupCreate,
            Self::GroupRead,
            Self::GroupManage,
            Self::SystemSettings,
            Self::SystemMetrics,
            Self::SystemAdmin,
            Self::QuotaRead,
            Self::QuotaManage,
        ]
    }

    /// Return all unique domain prefixes (first segment of the dotted path).
    pub fn all_domains() -> Vec<&'static str> {
        vec![
            "gateway",
            "conversation",
            "apikey",
            "model",
            "provider",
            "agent",
            "user",
            "group",
            "system",
            "quota",
        ]
    }

    /// Expand a domain name into all leaf permissions under it.
    /// `"agent"` → `[AgentRegister, AgentRead, AgentDeregister]`.
    pub fn expand_domain(domain: &str) -> Vec<Self> {
        Self::all()
            .into_iter()
            .filter(|p| p.as_str().starts_with(domain) && p.as_str() != domain)
            .collect()
    }

    /// Group all permissions by category (domain) for organised UI display.
    /// Returns `Vec<(category_label, permissions)>`.
    pub fn categories() -> Vec<(String, Vec<Self>)> {
        vec![
            (
                "gateway".into(),
                vec![
                    Self::GatewayChat,
                    Self::GatewayListModels,
                    Self::GatewayStream,
                ],
            ),
            (
                "conversation".into(),
                vec![
                    Self::ConversationCreate,
                    Self::ConversationRead,
                    Self::ConversationDelete,
                ],
            ),
            (
                "apikey".into(),
                vec![Self::ApiKeyCreate, Self::ApiKeyRead, Self::ApiKeyRevoke],
            ),
            (
                "model".into(),
                vec![
                    Self::ModelDeploy,
                    Self::ModelStop,
                    Self::ModelRead,
                    Self::ModelDownload,
                ],
            ),
            (
                "provider".into(),
                vec![
                    Self::ProviderCreate,
                    Self::ProviderRead,
                    Self::ProviderUpdate,
                    Self::ProviderDelete,
                    Self::ProviderTest,
                ],
            ),
            (
                "agent".into(),
                vec![Self::AgentRegister, Self::AgentRead, Self::AgentDeregister],
            ),
            (
                "user".into(),
                vec![
                    Self::UserCreate,
                    Self::UserRead,
                    Self::UserUpdate,
                    Self::UserDelete,
                ],
            ),
            (
                "group".into(),
                vec![Self::GroupCreate, Self::GroupRead, Self::GroupManage],
            ),
            (
                "system".into(),
                vec![Self::SystemSettings, Self::SystemMetrics, Self::SystemAdmin],
            ),
            (
                "quota".into(),
                vec![Self::QuotaRead, Self::QuotaManage],
            ),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════
// Role definitions
// ═══════════════════════════════════════════════════════════════

/// A named collection of permissions that can be assigned to a user.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub is_default: bool,
}

impl Role {
    /// Admin — every permission in the system.
    pub fn admin() -> Self {
        Self {
            name: "admin".into(),
            description: "Full system access — all permissions granted.".into(),
            permissions: Permission::all(),
            is_default: false,
        }
    }

    /// Developer — gateway access, model operations, and personal resources.
    pub fn developer() -> Self {
        Self {
            name: "developer".into(),
            description: "Can chat, manage models, and handle own conversations & API keys.".into(),
            permissions: vec![
                Permission::GatewayChat,
                Permission::GatewayListModels,
                Permission::GatewayStream,
                Permission::ConversationCreate,
                Permission::ConversationRead,
                Permission::ConversationDelete,
                Permission::ApiKeyCreate,
                Permission::ApiKeyRead,
                Permission::ApiKeyRevoke,
                Permission::ModelRead,
                Permission::ProviderRead,
                Permission::AgentRead,
            ],
            is_default: true,
        }
    }

    /// Viewer — read-only access across the platform.
    pub fn viewer() -> Self {
        Self {
            name: "viewer".into(),
            description: "Read-only access to models, providers, agents, and own resources.".into(),
            permissions: vec![
                Permission::GatewayListModels,
                Permission::ConversationRead,
                Permission::ApiKeyRead,
                Permission::ModelRead,
                Permission::ProviderRead,
                Permission::AgentRead,
            ],
            is_default: false,
        }
    }

    /// All three built-in roles.
    pub fn builtin_roles() -> Vec<Self> {
        vec![Self::admin(), Self::developer(), Self::viewer()]
    }
}

// ═══════════════════════════════════════════════════════════════
// i18n key constants
// ═══════════════════════════════════════════════════════════════

pub mod i18n {
    use super::Permission;

    pub const PERMISSION_PREFIX: &str = "permission.";
    pub const ROLE_PREFIX: &str = "role.";

    pub const ROLE_ADMIN: &str = "role.admin";
    pub const ROLE_DEVELOPER: &str = "role.developer";
    pub const ROLE_VIEWER: &str = "role.viewer";

    pub const DIM_USER: &str = "dimension.user";
    pub const DIM_USER_GROUP: &str = "dimension.user_group";
    pub const DIM_MODEL_GROUP: &str = "dimension.model_group";

    pub const SCOPE_GLOBAL: &str = "scope.global";
    pub const SCOPE_PERSONAL: &str = "scope.personal";
    pub const SCOPE_MODEL_GROUP: &str = "scope.model_group";

    /// Build an i18n key from a permission, e.g. `"permission.gateway.chat"`.
    pub fn permission_key(p: &Permission) -> String {
        format!("{}{}", PERMISSION_PREFIX, p.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════
// Convenience free functions (mirror the old rbac module's API)
// ═══════════════════════════════════════════════════════════════

/// Parse a dotted permission path string into a `Permission` variant.
pub fn parse_permission(s: &str) -> Option<Permission> {
    Permission::from_path(s)
}

/// Validate a list of grant strings. Returns `Ok(())` if all are valid
/// leaf permissions or domain prefixes; returns `Err(invalid)` otherwise.
pub fn validate_grant_permissions(grants: &[String]) -> Result<(), Vec<String>> {
    let invalid: Vec<String> = grants
        .iter()
        .filter(|g| Permission::from_path(g).is_none() && Permission::expand_domain(g).is_empty())
        .cloned()
        .collect();
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(invalid)
    }
}

/// Return the serde names of all leaf permissions.
pub fn list_all_permission_names() -> Vec<String> {
    Permission::all()
        .iter()
        .map(|p| p.as_str().to_string())
        .collect()
}

/// Return all domain prefix strings.
pub fn list_all_domain_names() -> Vec<&'static str> {
    Permission::all_domains()
}

/// Expand a domain name into the names of all leaf permissions under it.
pub fn expand_domain(domain: &str) -> Vec<String> {
    Permission::expand_domain(domain)
        .iter()
        .map(|p| p.as_str().to_string())
        .collect()
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Permission::from_path / parse_permission ────────────────

    #[test]
    fn parse_valid_leaf() {
        assert!(parse_permission("agent.read").is_some());
        assert!(parse_permission("model.deploy").is_some());
        assert!(parse_permission("provider.test").is_some());
        assert!(parse_permission("system.admin").is_some());
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_permission("nonexistent.action").is_none());
        assert!(parse_permission("").is_none());
        assert!(parse_permission("agent.invalid").is_none());
    }

    // ── validate_grant_permissions ──────────────────────────────

    #[test]
    fn validate_grants_ok() {
        let grants = vec!["agent.read".into(), "model.deploy".into()];
        assert!(validate_grant_permissions(&grants).is_ok());
    }

    #[test]
    fn validate_grants_rejects_invalid() {
        let grants = vec!["agent.read".into(), "invalid.permission".into()];
        assert!(validate_grant_permissions(&grants).is_err());
    }

    #[test]
    fn validate_grants_allows_domain_wildcards() {
        let grants = vec!["agent".into()];
        assert!(validate_grant_permissions(&grants).is_ok());
    }

    // ── counts ──────────────────────────────────────────────────

    #[test]
    fn all_permissions_count() {
        let names = list_all_permission_names();
        assert_eq!(names.len(), 33, "expected 33 leaf permissions");
    }

    #[test]
    fn all_domains_count() {
        let domains = list_all_domain_names();
        assert_eq!(domains.len(), 10);
    }

    // ── Permission::expand_domain ───────────────────────────────

    #[test]
    fn expand_agent_domain() {
        let perms = Permission::expand_domain("agent");
        assert_eq!(perms.len(), 3);
        assert!(perms.contains(&Permission::AgentRegister));
        assert!(perms.contains(&Permission::AgentRead));
        assert!(perms.contains(&Permission::AgentDeregister));
    }

    #[test]
    fn expand_nonexistent_domain() {
        let perms = Permission::expand_domain("nope");
        assert!(perms.is_empty());
    }

    // ── Permission::scope ───────────────────────────────────────

    #[test]
    fn system_admin_is_global() {
        assert_eq!(Permission::SystemAdmin.scope(), PermissionScope::Global);
    }

    #[test]
    fn model_deploy_is_model_group_scoped() {
        assert_eq!(
            Permission::ModelDeploy.scope(),
            PermissionScope::ModelGroup(String::new())
        );
    }

    #[test]
    fn conversation_create_is_personal() {
        assert_eq!(
            Permission::ConversationCreate.scope(),
            PermissionScope::Personal
        );
    }

    // ── Permission serde round-trip ─────────────────────────────

    #[test]
    fn permission_serde_as_string() {
        let json = serde_json::to_string(&Permission::GatewayChat).unwrap();
        assert_eq!(json, r#""gateway.chat""#);
        let back: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Permission::GatewayChat);
    }

    #[test]
    fn all_permissions_round_trip() {
        for p in Permission::all() {
            let s = serde_json::to_string(&p).unwrap();
            let back: Permission = serde_json::from_str(&s).unwrap();
            assert_eq!(back, p, "round-trip failed for {p:?}");
        }
    }

    // ── Role definitions ────────────────────────────────────────

    #[test]
    fn admin_has_all_permissions() {
        let admin = Role::admin();
        assert_eq!(admin.permissions.len(), Permission::all().len());
    }

    #[test]
    fn developer_has_subset() {
        let dev = Role::developer();
        assert!(dev.permissions.len() < Permission::all().len());
        assert!(dev.is_default);
    }

    #[test]
    fn viewer_is_read_only() {
        let viewer = Role::viewer();
        // viewer shouldn't have any create/delete/deploy/stop permissions
        let write_perms: Vec<_> = viewer
            .permissions
            .iter()
            .filter(|p| {
                p.as_str().contains(".create")
                    || p.as_str().contains(".delete")
                    || p.as_str().contains(".deploy")
                    || p.as_str().contains(".stop")
                    || p.as_str().contains(".revoke")
                    || p.as_str().contains(".register")
                    || p.as_str().contains(".deregister")
                    || p.as_str().contains(".update")
            })
            .collect();
        assert!(
            write_perms.is_empty(),
            "viewer should have no write permissions, got: {write_perms:?}"
        );
    }

    #[test]
    fn three_builtin_roles() {
        assert_eq!(Role::builtin_roles().len(), 3);
    }

    // ── i18n keys ───────────────────────────────────────────────

    #[test]
    fn i18n_permission_key() {
        assert_eq!(
            i18n::permission_key(&Permission::GatewayChat),
            "permission.gateway.chat"
        );
        assert_eq!(
            i18n::permission_key(&Permission::SystemAdmin),
            "permission.system.admin"
        );
    }

    // ── AccessDimension serde ───────────────────────────────────

    #[test]
    fn access_dimension_serde_snake_case() {
        let json = serde_json::to_string(&AccessDimension::UserGroup).unwrap();
        assert_eq!(json, r#""user_group""#);
        let back: AccessDimension = serde_json::from_str(&json).unwrap();
        assert_eq!(back, AccessDimension::UserGroup);
    }

    // ── PermissionScope serde ───────────────────────────────────

    #[test]
    fn permission_scope_global_serde() {
        let json = serde_json::to_string(&PermissionScope::Global).unwrap();
        assert_eq!(json, r#"{"type":"global"}"#);
        let back: PermissionScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PermissionScope::Global);
    }

    #[test]
    fn permission_scope_model_group_serde() {
        let scope = PermissionScope::ModelGroup("mg-42".into());
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, r#"{"type":"model_group","value":"mg-42"}"#);
        let back: PermissionScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, scope);
    }

    #[test]
    fn permission_scope_personal_serde() {
        let json = serde_json::to_string(&PermissionScope::Personal).unwrap();
        assert_eq!(json, r#"{"type":"personal"}"#);
        let back: PermissionScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PermissionScope::Personal);
    }

    // ── categories ──────────────────────────────────────────────

    #[test]
    fn categories_count() {
        let cats = Permission::categories();
        assert_eq!(cats.len(), 10);
    }

    #[test]
    fn all_permissions_in_categories() {
        let mut seen = std::collections::HashSet::new();
        for (_, perms) in Permission::categories() {
            for p in perms {
                seen.insert(p);
            }
        }
        assert_eq!(seen.len(), Permission::all().len());
    }

    // ── description / as_str exhaustiveness ─────────────────────

    #[test]
    fn every_permission_has_description_and_str() {
        for p in Permission::all() {
            let desc = p.description();
            assert!(!desc.is_empty(), "missing description for {p:?}");
            let s = p.as_str();
            assert!(!s.is_empty(), "missing as_str for {p:?}");
            assert!(s.contains('.'), "as_str must be dotted: {s}");
        }
    }
}
