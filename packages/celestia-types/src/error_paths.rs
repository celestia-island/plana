//! Semantic error-path trees — the family's error identity (2026-10-11
//! user ruling, direction C).
//!
//! The JSON-RPC numeric error code is ALWAYS the -32000 placeholder for
//! application errors (the spec requires a code; the -32000..-32099 band
//! is its "implementation-defined server-errors" slot). The application's
//! error IDENTITY is a METHOD-STYLE PATH STRING (`Usage.Forbidden`,
//! `Scheduler.QueryFailed`), declared here as yuuka `derive_enum!`
//! trees — one per big namespace — and shipped to TypeScript consumers
//! through the generated `bindings/errorPaths.ts` (the export test in
//! this module regenerates it; `cargo test` is the generator, CI is the
//! drift gate).
//!
//! Wire shape: the path rides as the message prefix —
//! `"<Path.Here>: <human prose>"` — the channel every family client
//! already carries end-to-end (the plana rpc client transports the
//! message verbatim and discards the numeric code). Denials end in
//! `.Forbidden`.
//!
//! Family services host their domain trees HERE (the unified table) and
//! send errors through a typed `&impl AppErrorPath` helper so a bare
//! string path cannot compile (chest's `send_app_error` is the
//! reference). New namespaces get a new tree + a `*_PATHS` const + a
//! pin test + a writer entry — do NOT mint new numeric codes.

use yuuka::derive_enum;

/// The one code application errors carry. Placeholder by design — see
/// the module docs; the semantic identity lives in the path.
pub const APP_ERROR_CODE: i64 = -32000;

/// The typed channel for error paths: only the trees in this module
/// implement it, so a caller cannot pass a bare string (a
/// numeric-code-shaped "path" is a compile error, not a silent drift).
pub trait AppErrorPath {
    /// The method-style semantic path ("Usage.Forbidden").
    fn error_path(&self) -> &'static str;
}

// ── Usage (chest's usage.get seam) ───────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum UsageErrors {
        Forbidden,
        Internal,
        GroupByInvalid,
    }
);

impl UsageErrors {
    /// The arms draw from `USAGE_PATHS` slots BY DESIGN (R1 MAJOR-1's
    /// drift guard): a new variant forces a new match arm (compile
    /// error without one), and the per-tree pin test asserts the exact
    /// literal — going green REQUIRES the const to carry the new path.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => USAGE_PATHS[0],
            Self::Internal => USAGE_PATHS[1],
            Self::GroupByInvalid => USAGE_PATHS[2],
        }
    }
}

impl AppErrorPath for UsageErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Usage tree, in declaration order (the TS export's
/// source of truth for this namespace).
pub const USAGE_PATHS: &[&str] = &["Usage.Forbidden", "Usage.Internal", "Usage.GroupByInvalid"];

// ── Scheduler (the arona metering plane — shared by usage.get and
//    chest's self-hosted registry) ────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum SchedulerErrors {
        Unavailable,
        QueryFailed,
    }
);

impl SchedulerErrors {
    /// See UsageErrors::path — arms draw from `SCHEDULER_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Unavailable => SCHEDULER_PATHS[0],
            Self::QueryFailed => SCHEDULER_PATHS[1],
        }
    }
}

impl AppErrorPath for SchedulerErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Scheduler tree. The two paths must stay distinct:
/// the split exists so a scheduler-side HTTP 403 (stale admin token)
/// classifies as `QueryFailed`, never as a caller denial.
pub const SCHEDULER_PATHS: &[&str] = &["Scheduler.Unavailable", "Scheduler.QueryFailed"];

// ── Platform (the P86 platform-pinning gate) ─────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum PlatformErrors {
        Forbidden,
        IdInvalid,
        Internal,
    }
);

impl PlatformErrors {
    /// See UsageErrors::path — arms draw from `PLATFORM_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => PLATFORM_PATHS[0],
            Self::IdInvalid => PLATFORM_PATHS[1],
            Self::Internal => PLATFORM_PATHS[2],
        }
    }
}

impl AppErrorPath for PlatformErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Platform tree.
pub const PLATFORM_PATHS: &[&str] = &[
    "Platform.Forbidden",
    "Platform.IdInvalid",
    "Platform.Internal",
];

// ── Models.SelfHosted (chest's self-hosted backend registry) ─────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum SelfHostedErrors {
        Forbidden,
        KindInvalid,
        Params,
        UrlInvalid,
        ModelsInvalid,
        ModelsRequired,
        IndexRequired,
    }
);

impl SelfHostedErrors {
    /// See UsageErrors::path — arms draw from `SELF_HOSTED_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => SELF_HOSTED_PATHS[0],
            Self::KindInvalid => SELF_HOSTED_PATHS[1],
            Self::Params => SELF_HOSTED_PATHS[2],
            Self::UrlInvalid => SELF_HOSTED_PATHS[3],
            Self::ModelsInvalid => SELF_HOSTED_PATHS[4],
            Self::ModelsRequired => SELF_HOSTED_PATHS[5],
            Self::IndexRequired => SELF_HOSTED_PATHS[6],
        }
    }
}

impl AppErrorPath for SelfHostedErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Models.SelfHosted tree.
pub const SELF_HOSTED_PATHS: &[&str] = &[
    "Models.SelfHosted.Forbidden",
    "Models.SelfHosted.KindInvalid",
    "Models.SelfHosted.Params",
    "Models.SelfHosted.UrlInvalid",
    "Models.SelfHosted.ModelsInvalid",
    "Models.SelfHosted.ModelsRequired",
    "Models.SelfHosted.IndexRequired",
];

// ── Authority (chest's shared authority gates — check_admin,
//    require_maintenance_admin, rbac manage, device enroll, panel write:
//    every missing-authority denial lands HERE, never as a per-domain
//    Forbidden) ────────────────────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum AuthorityErrors {
        SystemWrite,
        SystemRead,
        RbacManage,
        DeviceEnroll,
        MaintenanceAdmin,
        PanelWrite,
        Forbidden,
    }
);

impl AuthorityErrors {
    /// See UsageErrors::path — arms draw from `AUTHORITY_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::SystemWrite => AUTHORITY_PATHS[0],
            Self::SystemRead => AUTHORITY_PATHS[1],
            Self::RbacManage => AUTHORITY_PATHS[2],
            Self::DeviceEnroll => AUTHORITY_PATHS[3],
            Self::MaintenanceAdmin => AUTHORITY_PATHS[4],
            Self::PanelWrite => AUTHORITY_PATHS[5],
            Self::Forbidden => AUTHORITY_PATHS[6],
        }
    }
}

impl AppErrorPath for AuthorityErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Authority tree.
pub const AUTHORITY_PATHS: &[&str] = &[
    "Authority.SystemWrite",
    "Authority.SystemRead",
    "Authority.RbacManage",
    "Authority.DeviceEnroll",
    "Authority.MaintenanceAdmin",
    "Authority.PanelWrite",
    "Authority.Forbidden",
];

// ── Upstream (chest's calls out to the evernight registry, arona
//    catalog, llm gateway, engine peers: transport failures of an
//    upstream dependency — which upstream is detail prose, not identity) ──
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum UpstreamErrors {
        Unreachable,
        Malformed,
        Timeout,
        Auth,
        Internal,
    }
);

impl UpstreamErrors {
    /// See UsageErrors::path — arms draw from `UPSTREAM_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Unreachable => UPSTREAM_PATHS[0],
            Self::Malformed => UPSTREAM_PATHS[1],
            Self::Timeout => UPSTREAM_PATHS[2],
            Self::Auth => UPSTREAM_PATHS[3],
            Self::Internal => UPSTREAM_PATHS[4],
        }
    }
}

impl AppErrorPath for UpstreamErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Upstream tree.
pub const UPSTREAM_PATHS: &[&str] = &[
    "Upstream.Unreachable",
    "Upstream.Malformed",
    "Upstream.Timeout",
    "Upstream.Auth",
    "Upstream.Internal",
];

// ── Topic (chest's topics/chat handler faces: topic fetch/list/messages/
//    archive/edit/undo, conversation resolution, model selection) ─────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum TopicErrors {
        GetFailed,
        ListFailed,
        MessagesFailed,
        ArchiveFailed,
        EditFailed,
        UndoFailed,
        ConversationNotFound,
        ConversationIdRequired,
        ConversationAccessDenied,
        NoModelSelected,
        ParamsRequired,
        CreateFailed,
        UpdateFailed,
        DeleteFailed,
        ReportsFailed,
        MessageNotFound,
    }
);

impl TopicErrors {
    /// See UsageErrors::path — arms draw from `TOPIC_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::GetFailed => TOPIC_PATHS[0],
            Self::ListFailed => TOPIC_PATHS[1],
            Self::MessagesFailed => TOPIC_PATHS[2],
            Self::ArchiveFailed => TOPIC_PATHS[3],
            Self::EditFailed => TOPIC_PATHS[4],
            Self::UndoFailed => TOPIC_PATHS[5],
            Self::ConversationNotFound => TOPIC_PATHS[6],
            Self::ConversationIdRequired => TOPIC_PATHS[7],
            Self::ConversationAccessDenied => TOPIC_PATHS[8],
            Self::NoModelSelected => TOPIC_PATHS[9],
            Self::ParamsRequired => TOPIC_PATHS[10],
            Self::CreateFailed => TOPIC_PATHS[11],
            Self::UpdateFailed => TOPIC_PATHS[12],
            Self::DeleteFailed => TOPIC_PATHS[13],
            Self::ReportsFailed => TOPIC_PATHS[14],
            Self::MessageNotFound => TOPIC_PATHS[15],
        }
    }
}

impl AppErrorPath for TopicErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Topic tree.
pub const TOPIC_PATHS: &[&str] = &[
    "Topic.GetFailed",
    "Topic.ListFailed",
    "Topic.MessagesFailed",
    "Topic.ArchiveFailed",
    "Topic.EditFailed",
    "Topic.UndoFailed",
    "Topic.ConversationNotFound",
    "Topic.ConversationIdRequired",
    "Topic.ConversationAccessDenied",
    "Topic.NoModelSelected",
    "Topic.ParamsRequired",
    "Topic.CreateFailed",
    "Topic.UpdateFailed",
    "Topic.DeleteFailed",
    "Topic.ReportsFailed",
    "Topic.MessageNotFound",
];

// ── Workspace (chest's workspace/tenancy handler faces: role gates,
//    membership lookups, workspace_id plumbing, tabs, shares) ─────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum WorkspaceErrors {
        Forbidden,
        RoleInsufficient,
        NotFound,
        /// "No access to this workspace" (chest `workspace.rs:91`, `:238`
        /// — the `check_workspace_access` denial) and "Not your
        /// workspace" (`workspace.rs:1111`, `:1193` — the
        /// `may_manage_workspace_alias` denial).
        AccessDenied,
        WorkspaceIdRequired,
        WorkspaceIdInvalid,
        TabNotFound,
        ShareNotFound,
        ShareUnavailable,
    }
);

impl WorkspaceErrors {
    /// See UsageErrors::path — arms draw from `WORKSPACE_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => WORKSPACE_PATHS[0],
            Self::RoleInsufficient => WORKSPACE_PATHS[1],
            Self::NotFound => WORKSPACE_PATHS[2],
            Self::AccessDenied => WORKSPACE_PATHS[3],
            Self::WorkspaceIdRequired => WORKSPACE_PATHS[4],
            Self::WorkspaceIdInvalid => WORKSPACE_PATHS[5],
            Self::TabNotFound => WORKSPACE_PATHS[6],
            Self::ShareNotFound => WORKSPACE_PATHS[7],
            Self::ShareUnavailable => WORKSPACE_PATHS[8],
        }
    }
}

impl AppErrorPath for WorkspaceErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Workspace tree.
pub const WORKSPACE_PATHS: &[&str] = &[
    "Workspace.Forbidden",
    "Workspace.RoleInsufficient",
    "Workspace.NotFound",
    "Workspace.AccessDenied",
    "Workspace.WorkspaceIdRequired",
    "Workspace.WorkspaceIdInvalid",
    "Workspace.TabNotFound",
    "Workspace.ShareNotFound",
    "Workspace.ShareUnavailable",
];

// ── Auth (chest's account/session faces: registration, deletion,
//    realname, sms challenges, invitations) ───────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum AuthErrors {
        /// "username already taken" (chest
        /// `handlers/invitations.rs:614`, `:675` — the directed-invitation
        /// account-precreate guard, `-32602`).
        UsernameTaken,
        EmailInUse,
        PasswordTooShort,
        UserNotFound,
        NoUserWithEmail,
        SelfDeleteDenied,
        RealNameDisabled,
        SmsDisabled,
        SmsProviderMissing,
        WrongCode,
        NoChallenge,
        /// "invite code pre-check failed" (chest `handlers/invitations.rs:376`),
        /// "could not mint a unique invite code" (`invitations.rs:385`),
        /// "failed to create invitation" (`invitations.rs:456`).
        InvitationMintFailed,
        MobileInvalid,
        ParamsRequired,
        IdInvalid,
        SaveFailed,
    }
);

impl AuthErrors {
    /// See UsageErrors::path — arms draw from `AUTH_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::UsernameTaken => AUTH_PATHS[0],
            Self::EmailInUse => AUTH_PATHS[1],
            Self::PasswordTooShort => AUTH_PATHS[2],
            Self::UserNotFound => AUTH_PATHS[3],
            Self::NoUserWithEmail => AUTH_PATHS[4],
            Self::SelfDeleteDenied => AUTH_PATHS[5],
            Self::RealNameDisabled => AUTH_PATHS[6],
            Self::SmsDisabled => AUTH_PATHS[7],
            Self::SmsProviderMissing => AUTH_PATHS[8],
            Self::WrongCode => AUTH_PATHS[9],
            Self::NoChallenge => AUTH_PATHS[10],
            Self::InvitationMintFailed => AUTH_PATHS[11],
            Self::MobileInvalid => AUTH_PATHS[12],
            Self::ParamsRequired => AUTH_PATHS[13],
            Self::IdInvalid => AUTH_PATHS[14],
            Self::SaveFailed => AUTH_PATHS[15],
        }
    }
}

impl AppErrorPath for AuthErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Auth tree.
pub const AUTH_PATHS: &[&str] = &[
    "Auth.UsernameTaken",
    "Auth.EmailInUse",
    "Auth.PasswordTooShort",
    "Auth.UserNotFound",
    "Auth.NoUserWithEmail",
    "Auth.SelfDeleteDenied",
    "Auth.RealNameDisabled",
    "Auth.SmsDisabled",
    "Auth.SmsProviderMissing",
    "Auth.WrongCode",
    "Auth.NoChallenge",
    "Auth.InvitationMintFailed",
    "Auth.MobileInvalid",
    "Auth.ParamsRequired",
    "Auth.IdInvalid",
    "Auth.SaveFailed",
];

// ── Provider (chest's model provider/vendor config faces: naming, URLs,
//    model lists, quotas, workflow wiring) ────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum ProviderErrors {
        NotFound,
        NameTaken,
        NameAndUrlRequired,
        UrlInvalid,
        ModelsRequired,
        ModelsInvalid,
        QuotaReached,
        KindInvalid,
        WorkflowRequired,
        WorkflowInvalid,
        NoActiveAdapter,
        NoConfig,
    }
);

impl ProviderErrors {
    /// See UsageErrors::path — arms draw from `PROVIDER_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::NotFound => PROVIDER_PATHS[0],
            Self::NameTaken => PROVIDER_PATHS[1],
            Self::NameAndUrlRequired => PROVIDER_PATHS[2],
            Self::UrlInvalid => PROVIDER_PATHS[3],
            Self::ModelsRequired => PROVIDER_PATHS[4],
            Self::ModelsInvalid => PROVIDER_PATHS[5],
            Self::QuotaReached => PROVIDER_PATHS[6],
            Self::KindInvalid => PROVIDER_PATHS[7],
            Self::WorkflowRequired => PROVIDER_PATHS[8],
            Self::WorkflowInvalid => PROVIDER_PATHS[9],
            Self::NoActiveAdapter => PROVIDER_PATHS[10],
            Self::NoConfig => PROVIDER_PATHS[11],
        }
    }
}

impl AppErrorPath for ProviderErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Provider tree.
pub const PROVIDER_PATHS: &[&str] = &[
    "Provider.NotFound",
    "Provider.NameTaken",
    "Provider.NameAndUrlRequired",
    "Provider.UrlInvalid",
    "Provider.ModelsRequired",
    "Provider.ModelsInvalid",
    "Provider.QuotaReached",
    "Provider.KindInvalid",
    "Provider.WorkflowRequired",
    "Provider.WorkflowInvalid",
    "Provider.NoActiveAdapter",
    "Provider.NoConfig",
];

// ── Media (chest's audio/video/voice/media handler faces: account
//    lookup, upload size limits, transcription) ───────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum MediaErrors {
        AccountMissing,
        ServiceUnavailable,
        AudioTooLarge,
        TranscribeRateLimited,
        AudioMissing,
        Base64ImageInvalid,
        FileTooLarge,
        TranscribeFailed,
        ParamsRequired,
        IdInvalid,
        KindRequired,
    }
);

impl MediaErrors {
    /// See UsageErrors::path — arms draw from `MEDIA_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::AccountMissing => MEDIA_PATHS[0],
            Self::ServiceUnavailable => MEDIA_PATHS[1],
            Self::AudioTooLarge => MEDIA_PATHS[2],
            Self::TranscribeRateLimited => MEDIA_PATHS[3],
            Self::AudioMissing => MEDIA_PATHS[4],
            Self::Base64ImageInvalid => MEDIA_PATHS[5],
            Self::FileTooLarge => MEDIA_PATHS[6],
            Self::TranscribeFailed => MEDIA_PATHS[7],
            Self::ParamsRequired => MEDIA_PATHS[8],
            Self::IdInvalid => MEDIA_PATHS[9],
            Self::KindRequired => MEDIA_PATHS[10],
        }
    }
}

impl AppErrorPath for MediaErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Media tree. `AccountLookupFailed` was removed in
/// the fix round: its only chest references were dynamic (`format!`
/// interpolation at `billing_quota.rs:153`, `billing_premium.rs:65`,
/// `video_results.rs:259`) — a later chest PR wires those sites to
/// `Media.AccountLookupFailed` when it lands.
pub const MEDIA_PATHS: &[&str] = &[
    "Media.AccountMissing",
    "Media.ServiceUnavailable",
    "Media.AudioTooLarge",
    "Media.TranscribeRateLimited",
    "Media.AudioMissing",
    "Media.Base64ImageInvalid",
    "Media.FileTooLarge",
    "Media.TranscribeFailed",
    "Media.ParamsRequired",
    "Media.IdInvalid",
    "Media.KindRequired",
];

// ── Topology (chest's topology/connections handler faces: gateway dial,
//    targets, node ids) ────────────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum TopologyErrors {
        GatewayTimeout,
        TargetDuplicate,
        RemoteTargetMissing,
        NoDialPath,
        ConnectionMissing,
        ClaimedGatewayNoNodeId,
        TargetKindMismatch,
        NodeIdShapeInvalid,
        NodeIdModeRequired,
        HostRequired,
    }
);

impl TopologyErrors {
    /// See UsageErrors::path — arms draw from `TOPOLOGY_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::GatewayTimeout => TOPOLOGY_PATHS[0],
            Self::TargetDuplicate => TOPOLOGY_PATHS[1],
            Self::RemoteTargetMissing => TOPOLOGY_PATHS[2],
            Self::NoDialPath => TOPOLOGY_PATHS[3],
            Self::ConnectionMissing => TOPOLOGY_PATHS[4],
            Self::ClaimedGatewayNoNodeId => TOPOLOGY_PATHS[5],
            Self::TargetKindMismatch => TOPOLOGY_PATHS[6],
            Self::NodeIdShapeInvalid => TOPOLOGY_PATHS[7],
            Self::NodeIdModeRequired => TOPOLOGY_PATHS[8],
            Self::HostRequired => TOPOLOGY_PATHS[9],
        }
    }
}

impl AppErrorPath for TopologyErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Topology tree.
pub const TOPOLOGY_PATHS: &[&str] = &[
    "Topology.GatewayTimeout",
    "Topology.TargetDuplicate",
    "Topology.RemoteTargetMissing",
    "Topology.NoDialPath",
    "Topology.ConnectionMissing",
    "Topology.ClaimedGatewayNoNodeId",
    "Topology.TargetKindMismatch",
    "Topology.NodeIdShapeInvalid",
    "Topology.NodeIdModeRequired",
    "Topology.HostRequired",
];

// ── Console (chest's operator/ops faces: danger, infra_repair, ci_ops,
//    cd_ops, noa_ops, office_ops, deploy_scripts, selftrain, selfheal,
//    system_info) ──────────────────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum ConsoleErrors {
        CiNotConfigured,
        CiWatchlistEmpty,
        CdNotConfigured,
        EngineNotConfigured,
        RepairTargetNotConfigured,
        ParamsRequired,
    }
);

impl ConsoleErrors {
    /// See UsageErrors::path — arms draw from `CONSOLE_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::CiNotConfigured => CONSOLE_PATHS[0],
            Self::CiWatchlistEmpty => CONSOLE_PATHS[1],
            Self::CdNotConfigured => CONSOLE_PATHS[2],
            Self::EngineNotConfigured => CONSOLE_PATHS[3],
            Self::RepairTargetNotConfigured => CONSOLE_PATHS[4],
            Self::ParamsRequired => CONSOLE_PATHS[5],
        }
    }
}

impl AppErrorPath for ConsoleErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Console tree.
pub const CONSOLE_PATHS: &[&str] = &[
    "Console.CiNotConfigured",
    "Console.CiWatchlistEmpty",
    "Console.CdNotConfigured",
    "Console.EngineNotConfigured",
    "Console.RepairTargetNotConfigured",
    "Console.ParamsRequired",
];

// ── Webhook (chest's webhook config face) ────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum WebhookErrors {
        IdRequired,
        ScopeTypeInvalid,
        ScopeIdInvalid,
        ServiceAuthOnly,
        EventsInvalid,
        ScopesInvalid,
        ParamsRequired,
        NameRequired,
        NotFound,
        IdInvalid,
        InsertFailed,
        UpdateFailed,
        DeleteFailed,
        RotateFailed,
        QueryFailed,
    }
);

impl WebhookErrors {
    /// See UsageErrors::path — arms draw from `WEBHOOK_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::IdRequired => WEBHOOK_PATHS[0],
            Self::ScopeTypeInvalid => WEBHOOK_PATHS[1],
            Self::ScopeIdInvalid => WEBHOOK_PATHS[2],
            Self::ServiceAuthOnly => WEBHOOK_PATHS[3],
            Self::EventsInvalid => WEBHOOK_PATHS[4],
            Self::ScopesInvalid => WEBHOOK_PATHS[5],
            Self::ParamsRequired => WEBHOOK_PATHS[6],
            Self::NameRequired => WEBHOOK_PATHS[7],
            Self::NotFound => WEBHOOK_PATHS[8],
            Self::IdInvalid => WEBHOOK_PATHS[9],
            Self::InsertFailed => WEBHOOK_PATHS[10],
            Self::UpdateFailed => WEBHOOK_PATHS[11],
            Self::DeleteFailed => WEBHOOK_PATHS[12],
            Self::RotateFailed => WEBHOOK_PATHS[13],
            Self::QueryFailed => WEBHOOK_PATHS[14],
        }
    }
}

impl AppErrorPath for WebhookErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Webhook tree.
pub const WEBHOOK_PATHS: &[&str] = &[
    "Webhook.IdRequired",
    "Webhook.ScopeTypeInvalid",
    "Webhook.ScopeIdInvalid",
    "Webhook.ServiceAuthOnly",
    "Webhook.EventsInvalid",
    "Webhook.ScopesInvalid",
    "Webhook.ParamsRequired",
    "Webhook.NameRequired",
    "Webhook.NotFound",
    "Webhook.IdInvalid",
    "Webhook.InsertFailed",
    "Webhook.UpdateFailed",
    "Webhook.DeleteFailed",
    "Webhook.RotateFailed",
    "Webhook.QueryFailed",
];

// ── Iam (chest's iam + oauth_clients faces: clients, tokens, group
//    sources, membership consent) ──────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum IamErrors {
        ClientIdRequired,
        ClientNotFound,
        ClientNotActive,
        TokenEncryptFailed,
        SecretSealFailed,
        GroupSourceRequiresSystemWrite,
        BuiltinGroupImmutable,
        MembershipRequired,
        ConsentRequiresMembership,
        GroupNotFound,
        NameRequired,
    }
);

impl IamErrors {
    /// See UsageErrors::path — arms draw from `IAM_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::ClientIdRequired => IAM_PATHS[0],
            Self::ClientNotFound => IAM_PATHS[1],
            Self::ClientNotActive => IAM_PATHS[2],
            Self::TokenEncryptFailed => IAM_PATHS[3],
            Self::SecretSealFailed => IAM_PATHS[4],
            Self::GroupSourceRequiresSystemWrite => IAM_PATHS[5],
            Self::BuiltinGroupImmutable => IAM_PATHS[6],
            Self::MembershipRequired => IAM_PATHS[7],
            Self::ConsentRequiresMembership => IAM_PATHS[8],
            Self::GroupNotFound => IAM_PATHS[9],
            Self::NameRequired => IAM_PATHS[10],
        }
    }
}

impl AppErrorPath for IamErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Iam tree.
pub const IAM_PATHS: &[&str] = &[
    "Iam.ClientIdRequired",
    "Iam.ClientNotFound",
    "Iam.ClientNotActive",
    "Iam.TokenEncryptFailed",
    "Iam.SecretSealFailed",
    "Iam.GroupSourceRequiresSystemWrite",
    "Iam.BuiltinGroupImmutable",
    "Iam.MembershipRequired",
    "Iam.ConsentRequiresMembership",
    "Iam.GroupNotFound",
    "Iam.NameRequired",
];

// ── Billing (chest's billing authority/quota/reconcile faces) ────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum BillingErrors {
        ReservationLost,
        WalletVanished,
        ModeInvalid,
        QuotaExceeded,
        ParamsRequired,
        SenderIdRequired,
        SenderIdInvalid,
        Forbidden,
    }
);

impl BillingErrors {
    /// See UsageErrors::path — arms draw from `BILLING_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::ReservationLost => BILLING_PATHS[0],
            Self::WalletVanished => BILLING_PATHS[1],
            Self::ModeInvalid => BILLING_PATHS[2],
            Self::QuotaExceeded => BILLING_PATHS[3],
            Self::ParamsRequired => BILLING_PATHS[4],
            Self::SenderIdRequired => BILLING_PATHS[5],
            Self::SenderIdInvalid => BILLING_PATHS[6],
            Self::Forbidden => BILLING_PATHS[7],
        }
    }
}

impl AppErrorPath for BillingErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Billing tree.
pub const BILLING_PATHS: &[&str] = &[
    "Billing.ReservationLost",
    "Billing.WalletVanished",
    "Billing.ModeInvalid",
    "Billing.QuotaExceeded",
    "Billing.ParamsRequired",
    "Billing.SenderIdRequired",
    "Billing.SenderIdInvalid",
    "Billing.Forbidden",
];

// ── Edge (chest's edge_flash/edge_admission/edge_enroll/edge_proxy
//    faces). `AdmissionForbidden` was removed in the fix round: its only
//    referent is edge_admission's admin-gate prose ("admin required —
//    device admission management is an operator surface"), and per this
//    table's Authority rule every missing-authority denial lands in
//    Authority.* — never as a per-domain Forbidden. ──────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum EdgeErrors {
        CatalogEntryNotFound,
        IntentIdRequired,
        DeviceIdRequired,
        EnrollmentBudgetExhausted,
        SerialOrEnrollmentIdRequired,
    }
);

impl EdgeErrors {
    /// See UsageErrors::path — arms draw from `EDGE_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::CatalogEntryNotFound => EDGE_PATHS[0],
            Self::IntentIdRequired => EDGE_PATHS[1],
            Self::DeviceIdRequired => EDGE_PATHS[2],
            Self::EnrollmentBudgetExhausted => EDGE_PATHS[3],
            Self::SerialOrEnrollmentIdRequired => EDGE_PATHS[4],
        }
    }
}

impl AppErrorPath for EdgeErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Edge tree.
pub const EDGE_PATHS: &[&str] = &[
    "Edge.CatalogEntryNotFound",
    "Edge.IntentIdRequired",
    "Edge.DeviceIdRequired",
    "Edge.EnrollmentBudgetExhausted",
    "Edge.SerialOrEnrollmentIdRequired",
];

// ── Visitor (chest's visitor/visitor_observations_admin/tour_access
//    faces) ────────────────────────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum VisitorErrors {
        TicketRequired,
        TicketNotFound,
        ObservationNotFound,
        TicketIdRequired,
    }
);

impl VisitorErrors {
    /// See UsageErrors::path — arms draw from `VISITOR_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::TicketRequired => VISITOR_PATHS[0],
            Self::TicketNotFound => VISITOR_PATHS[1],
            Self::ObservationNotFound => VISITOR_PATHS[2],
            Self::TicketIdRequired => VISITOR_PATHS[3],
        }
    }
}

impl AppErrorPath for VisitorErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Visitor tree.
pub const VISITOR_PATHS: &[&str] = &[
    "Visitor.TicketRequired",
    "Visitor.TicketNotFound",
    "Visitor.ObservationNotFound",
    "Visitor.TicketIdRequired",
];

// ── Panel (chest's panel_revision_ledger/panel_shares/projects/
//    site_documents/mcp_tools faces) ───────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum PanelErrors {
        RevisionInvalid,
        ViewRefInvalid,
        ExpectedRevisionRequired,
        RoleRequired,
        ReadOnly,
        McpHubRequired,
        McpIdToolRequired,
        ArgumentsInvalid,
        IdRequired,
        NotFound,
    }
);

impl PanelErrors {
    /// See UsageErrors::path — arms draw from `PANEL_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::RevisionInvalid => PANEL_PATHS[0],
            Self::ViewRefInvalid => PANEL_PATHS[1],
            Self::ExpectedRevisionRequired => PANEL_PATHS[2],
            Self::RoleRequired => PANEL_PATHS[3],
            Self::ReadOnly => PANEL_PATHS[4],
            Self::McpHubRequired => PANEL_PATHS[5],
            Self::McpIdToolRequired => PANEL_PATHS[6],
            Self::ArgumentsInvalid => PANEL_PATHS[7],
            Self::IdRequired => PANEL_PATHS[8],
            Self::NotFound => PANEL_PATHS[9],
        }
    }
}

impl AppErrorPath for PanelErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Panel tree.
pub const PANEL_PATHS: &[&str] = &[
    "Panel.RevisionInvalid",
    "Panel.ViewRefInvalid",
    "Panel.ExpectedRevisionRequired",
    "Panel.RoleRequired",
    "Panel.ReadOnly",
    "Panel.McpHubRequired",
    "Panel.McpIdToolRequired",
    "Panel.ArgumentsInvalid",
    "Panel.IdRequired",
    "Panel.NotFound",
];

// ── Audit (chest's audit_query/audit_forward/usage_facts faces) ──────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum AuditErrors {
        ExportRecordWriteFailed,
        NothingToPurge,
        BatchTooLarge,
        ServiceAuthOnly,
        ParamsRequired,
        FactsRequired,
        QueryFailed,
        ExportFailed,
        ExportedBeforeRequired,
    }
);

impl AuditErrors {
    /// See UsageErrors::path — arms draw from `AUDIT_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::ExportRecordWriteFailed => AUDIT_PATHS[0],
            Self::NothingToPurge => AUDIT_PATHS[1],
            Self::BatchTooLarge => AUDIT_PATHS[2],
            Self::ServiceAuthOnly => AUDIT_PATHS[3],
            Self::ParamsRequired => AUDIT_PATHS[4],
            Self::FactsRequired => AUDIT_PATHS[5],
            Self::QueryFailed => AUDIT_PATHS[6],
            Self::ExportFailed => AUDIT_PATHS[7],
            Self::ExportedBeforeRequired => AUDIT_PATHS[8],
        }
    }
}

impl AppErrorPath for AuditErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Audit tree.
pub const AUDIT_PATHS: &[&str] = &[
    "Audit.ExportRecordWriteFailed",
    "Audit.NothingToPurge",
    "Audit.BatchTooLarge",
    "Audit.ServiceAuthOnly",
    "Audit.ParamsRequired",
    "Audit.FactsRequired",
    "Audit.QueryFailed",
    "Audit.ExportFailed",
    "Audit.ExportedBeforeRequired",
];

// ── Plugin (chest's plugins.rs face — domain-specific conditions only;
//    plugin-store/trust general rejections live in their own domains) ─────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum PluginErrors {
        IdRequired,
        TargetTypeRequired,
        PointRequired,
    }
);

impl PluginErrors {
    /// See UsageErrors::path — arms draw from `PLUGIN_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::IdRequired => PLUGIN_PATHS[0],
            Self::TargetTypeRequired => PLUGIN_PATHS[1],
            Self::PointRequired => PLUGIN_PATHS[2],
        }
    }
}

impl AppErrorPath for PluginErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Plugin tree.
pub const PLUGIN_PATHS: &[&str] = &[
    "Plugin.IdRequired",
    "Plugin.TargetTypeRequired",
    "Plugin.PointRequired",
];

// ── Channel (chest's channel faces: admin.channels.*, admin.policy.*,
//    admin.credentials.*, channel.self.* — the admin CRUD buckets ride
//    this tree by name; the shared missing-authority gates themselves
//    still land in Authority.*) ────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum ChannelErrors {
        SaveFailed,
        NotFound,
        InvalidParams,
        ParamsRequired,
        NoConfig,
        NoActiveAdapter,
        ReloadFailed,
        QueryFailed,
        SendFailed,
        InsertFailed,
        DeleteFailed,
        NameRequired,
        IdRequired,
        UnsupportedPlatform,
        QuotaExceeded,
        CategoryInvalid,
    }
);

impl ChannelErrors {
    /// See UsageErrors::path — arms draw from `CHANNEL_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::SaveFailed => CHANNEL_PATHS[0],
            Self::NotFound => CHANNEL_PATHS[1],
            Self::InvalidParams => CHANNEL_PATHS[2],
            Self::ParamsRequired => CHANNEL_PATHS[3],
            Self::NoConfig => CHANNEL_PATHS[4],
            Self::NoActiveAdapter => CHANNEL_PATHS[5],
            Self::ReloadFailed => CHANNEL_PATHS[6],
            Self::QueryFailed => CHANNEL_PATHS[7],
            Self::SendFailed => CHANNEL_PATHS[8],
            Self::InsertFailed => CHANNEL_PATHS[9],
            Self::DeleteFailed => CHANNEL_PATHS[10],
            Self::NameRequired => CHANNEL_PATHS[11],
            Self::IdRequired => CHANNEL_PATHS[12],
            Self::UnsupportedPlatform => CHANNEL_PATHS[13],
            Self::QuotaExceeded => CHANNEL_PATHS[14],
            Self::CategoryInvalid => CHANNEL_PATHS[15],
        }
    }
}

impl AppErrorPath for ChannelErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Channel tree.
pub const CHANNEL_PATHS: &[&str] = &[
    "Channel.SaveFailed",
    "Channel.NotFound",
    "Channel.InvalidParams",
    "Channel.ParamsRequired",
    "Channel.NoConfig",
    "Channel.NoActiveAdapter",
    "Channel.ReloadFailed",
    "Channel.QueryFailed",
    "Channel.SendFailed",
    "Channel.InsertFailed",
    "Channel.DeleteFailed",
    "Channel.NameRequired",
    "Channel.IdRequired",
    "Channel.UnsupportedPlatform",
    "Channel.QuotaExceeded",
    "Channel.CategoryInvalid",
];

// ── Extensions (chest's extensions/RAG face: rag.search/upsert/embed,
//    payment.tier(s), invalid document params, and the pipeline.execute
//    param guard) ──────────────────────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum ExtensionsErrors {
        SearchFailed,
        UpsertFailed,
        EmbedFailed,
        DocumentParamsInvalid,
        TierLookupFailed,
        ParamsRequired,
    }
);

impl ExtensionsErrors {
    /// See UsageErrors::path — arms draw from `EXTENSIONS_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::SearchFailed => EXTENSIONS_PATHS[0],
            Self::UpsertFailed => EXTENSIONS_PATHS[1],
            Self::EmbedFailed => EXTENSIONS_PATHS[2],
            Self::DocumentParamsInvalid => EXTENSIONS_PATHS[3],
            Self::TierLookupFailed => EXTENSIONS_PATHS[4],
            Self::ParamsRequired => EXTENSIONS_PATHS[5],
        }
    }
}

impl AppErrorPath for ExtensionsErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Extensions tree.
pub const EXTENSIONS_PATHS: &[&str] = &[
    "Extensions.SearchFailed",
    "Extensions.UpsertFailed",
    "Extensions.EmbedFailed",
    "Extensions.DocumentParamsInvalid",
    "Extensions.TierLookupFailed",
    "Extensions.ParamsRequired",
];

/// Every path in the table, namespace by namespace — the TS export's
/// flat surface.
pub const ALL_ERROR_PATHS: &[&[&str]] = &[
    USAGE_PATHS,
    SCHEDULER_PATHS,
    PLATFORM_PATHS,
    SELF_HOSTED_PATHS,
    AUTHORITY_PATHS,
    UPSTREAM_PATHS,
    TOPIC_PATHS,
    WORKSPACE_PATHS,
    AUTH_PATHS,
    PROVIDER_PATHS,
    MEDIA_PATHS,
    TOPOLOGY_PATHS,
    CONSOLE_PATHS,
    WEBHOOK_PATHS,
    IAM_PATHS,
    BILLING_PATHS,
    EDGE_PATHS,
    VISITOR_PATHS,
    PANEL_PATHS,
    AUDIT_PATHS,
    PLUGIN_PATHS,
    CHANNEL_PATHS,
    EXTENSIONS_PATHS,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_paths_pinned() {
        assert_eq!(UsageErrors::Forbidden.path(), "Usage.Forbidden");
        assert_eq!(UsageErrors::Internal.path(), "Usage.Internal");
        assert_eq!(UsageErrors::GroupByInvalid.path(), "Usage.GroupByInvalid");
        assert_eq!(USAGE_PATHS.len(), 3);
    }

    #[test]
    fn scheduler_paths_pinned() {
        assert_eq!(SchedulerErrors::Unavailable.path(), "Scheduler.Unavailable");
        assert_eq!(SchedulerErrors::QueryFailed.path(), "Scheduler.QueryFailed");
        assert_ne!(
            SchedulerErrors::Unavailable.path(),
            SchedulerErrors::QueryFailed.path()
        );
        assert_eq!(SCHEDULER_PATHS.len(), 2);
    }

    #[test]
    fn platform_paths_pinned() {
        assert_eq!(PlatformErrors::Forbidden.path(), "Platform.Forbidden");
        assert_eq!(PlatformErrors::IdInvalid.path(), "Platform.IdInvalid");
        assert_eq!(PlatformErrors::Internal.path(), "Platform.Internal");
        assert_eq!(PLATFORM_PATHS.len(), 3);
    }

    #[test]
    fn self_hosted_paths_pinned() {
        assert_eq!(
            SelfHostedErrors::Forbidden.path(),
            "Models.SelfHosted.Forbidden"
        );
        assert_eq!(
            SelfHostedErrors::KindInvalid.path(),
            "Models.SelfHosted.KindInvalid"
        );
        assert_eq!(SelfHostedErrors::Params.path(), "Models.SelfHosted.Params");
        assert_eq!(
            SelfHostedErrors::UrlInvalid.path(),
            "Models.SelfHosted.UrlInvalid"
        );
        assert_eq!(
            SelfHostedErrors::ModelsInvalid.path(),
            "Models.SelfHosted.ModelsInvalid"
        );
        assert_eq!(
            SelfHostedErrors::ModelsRequired.path(),
            "Models.SelfHosted.ModelsRequired"
        );
        assert_eq!(
            SelfHostedErrors::IndexRequired.path(),
            "Models.SelfHosted.IndexRequired"
        );
        assert_eq!(SELF_HOSTED_PATHS.len(), 7);
    }

    #[test]
    fn authority_paths_pinned() {
        assert_eq!(AuthorityErrors::SystemWrite.path(), "Authority.SystemWrite");
        assert_eq!(AuthorityErrors::SystemRead.path(), "Authority.SystemRead");
        assert_eq!(AuthorityErrors::RbacManage.path(), "Authority.RbacManage");
        assert_eq!(
            AuthorityErrors::DeviceEnroll.path(),
            "Authority.DeviceEnroll"
        );
        assert_eq!(
            AuthorityErrors::MaintenanceAdmin.path(),
            "Authority.MaintenanceAdmin"
        );
        assert_eq!(AuthorityErrors::PanelWrite.path(), "Authority.PanelWrite");
        assert_eq!(AuthorityErrors::Forbidden.path(), "Authority.Forbidden");
        assert_eq!(AUTHORITY_PATHS.len(), 7);
    }

    #[test]
    fn upstream_paths_pinned() {
        assert_eq!(UpstreamErrors::Unreachable.path(), "Upstream.Unreachable");
        assert_eq!(UpstreamErrors::Malformed.path(), "Upstream.Malformed");
        assert_eq!(UpstreamErrors::Timeout.path(), "Upstream.Timeout");
        assert_eq!(UpstreamErrors::Auth.path(), "Upstream.Auth");
        assert_eq!(UpstreamErrors::Internal.path(), "Upstream.Internal");
        assert_eq!(UPSTREAM_PATHS.len(), 5);
    }

    #[test]
    fn topic_paths_pinned() {
        assert_eq!(TopicErrors::GetFailed.path(), "Topic.GetFailed");
        assert_eq!(TopicErrors::ListFailed.path(), "Topic.ListFailed");
        assert_eq!(TopicErrors::MessagesFailed.path(), "Topic.MessagesFailed");
        assert_eq!(TopicErrors::ArchiveFailed.path(), "Topic.ArchiveFailed");
        assert_eq!(TopicErrors::EditFailed.path(), "Topic.EditFailed");
        assert_eq!(TopicErrors::UndoFailed.path(), "Topic.UndoFailed");
        assert_eq!(
            TopicErrors::ConversationNotFound.path(),
            "Topic.ConversationNotFound"
        );
        assert_eq!(
            TopicErrors::ConversationIdRequired.path(),
            "Topic.ConversationIdRequired"
        );
        assert_eq!(
            TopicErrors::ConversationAccessDenied.path(),
            "Topic.ConversationAccessDenied"
        );
        assert_eq!(TopicErrors::NoModelSelected.path(), "Topic.NoModelSelected");
        assert_eq!(TopicErrors::ParamsRequired.path(), "Topic.ParamsRequired");
        assert_eq!(TopicErrors::CreateFailed.path(), "Topic.CreateFailed");
        assert_eq!(TopicErrors::UpdateFailed.path(), "Topic.UpdateFailed");
        assert_eq!(TopicErrors::DeleteFailed.path(), "Topic.DeleteFailed");
        assert_eq!(TopicErrors::ReportsFailed.path(), "Topic.ReportsFailed");
        assert_eq!(TopicErrors::MessageNotFound.path(), "Topic.MessageNotFound");
        assert_eq!(TOPIC_PATHS.len(), 16);
    }

    #[test]
    fn workspace_paths_pinned() {
        assert_eq!(WorkspaceErrors::Forbidden.path(), "Workspace.Forbidden");
        assert_eq!(
            WorkspaceErrors::RoleInsufficient.path(),
            "Workspace.RoleInsufficient"
        );
        assert_eq!(WorkspaceErrors::NotFound.path(), "Workspace.NotFound");
        assert_eq!(
            WorkspaceErrors::AccessDenied.path(),
            "Workspace.AccessDenied"
        );
        assert_eq!(
            WorkspaceErrors::WorkspaceIdRequired.path(),
            "Workspace.WorkspaceIdRequired"
        );
        assert_eq!(
            WorkspaceErrors::WorkspaceIdInvalid.path(),
            "Workspace.WorkspaceIdInvalid"
        );
        assert_eq!(WorkspaceErrors::TabNotFound.path(), "Workspace.TabNotFound");
        assert_eq!(
            WorkspaceErrors::ShareNotFound.path(),
            "Workspace.ShareNotFound"
        );
        assert_eq!(
            WorkspaceErrors::ShareUnavailable.path(),
            "Workspace.ShareUnavailable"
        );
        assert_eq!(WORKSPACE_PATHS.len(), 9);
    }

    #[test]
    fn auth_paths_pinned() {
        assert_eq!(AuthErrors::UsernameTaken.path(), "Auth.UsernameTaken");
        assert_eq!(AuthErrors::EmailInUse.path(), "Auth.EmailInUse");
        assert_eq!(AuthErrors::PasswordTooShort.path(), "Auth.PasswordTooShort");
        assert_eq!(AuthErrors::UserNotFound.path(), "Auth.UserNotFound");
        assert_eq!(AuthErrors::NoUserWithEmail.path(), "Auth.NoUserWithEmail");
        assert_eq!(AuthErrors::SelfDeleteDenied.path(), "Auth.SelfDeleteDenied");
        assert_eq!(AuthErrors::RealNameDisabled.path(), "Auth.RealNameDisabled");
        assert_eq!(AuthErrors::SmsDisabled.path(), "Auth.SmsDisabled");
        assert_eq!(
            AuthErrors::SmsProviderMissing.path(),
            "Auth.SmsProviderMissing"
        );
        assert_eq!(AuthErrors::WrongCode.path(), "Auth.WrongCode");
        assert_eq!(AuthErrors::NoChallenge.path(), "Auth.NoChallenge");
        assert_eq!(
            AuthErrors::InvitationMintFailed.path(),
            "Auth.InvitationMintFailed"
        );
        assert_eq!(AuthErrors::MobileInvalid.path(), "Auth.MobileInvalid");
        assert_eq!(AuthErrors::ParamsRequired.path(), "Auth.ParamsRequired");
        assert_eq!(AuthErrors::IdInvalid.path(), "Auth.IdInvalid");
        assert_eq!(AuthErrors::SaveFailed.path(), "Auth.SaveFailed");
        assert_eq!(AUTH_PATHS.len(), 16);
    }

    #[test]
    fn provider_paths_pinned() {
        assert_eq!(ProviderErrors::NotFound.path(), "Provider.NotFound");
        assert_eq!(ProviderErrors::NameTaken.path(), "Provider.NameTaken");
        assert_eq!(
            ProviderErrors::NameAndUrlRequired.path(),
            "Provider.NameAndUrlRequired"
        );
        assert_eq!(ProviderErrors::UrlInvalid.path(), "Provider.UrlInvalid");
        assert_eq!(
            ProviderErrors::ModelsRequired.path(),
            "Provider.ModelsRequired"
        );
        assert_eq!(
            ProviderErrors::ModelsInvalid.path(),
            "Provider.ModelsInvalid"
        );
        assert_eq!(ProviderErrors::QuotaReached.path(), "Provider.QuotaReached");
        assert_eq!(ProviderErrors::KindInvalid.path(), "Provider.KindInvalid");
        assert_eq!(
            ProviderErrors::WorkflowRequired.path(),
            "Provider.WorkflowRequired"
        );
        assert_eq!(
            ProviderErrors::WorkflowInvalid.path(),
            "Provider.WorkflowInvalid"
        );
        assert_eq!(
            ProviderErrors::NoActiveAdapter.path(),
            "Provider.NoActiveAdapter"
        );
        assert_eq!(ProviderErrors::NoConfig.path(), "Provider.NoConfig");
        assert_eq!(PROVIDER_PATHS.len(), 12);
    }

    #[test]
    fn media_paths_pinned() {
        assert_eq!(MediaErrors::AccountMissing.path(), "Media.AccountMissing");
        assert_eq!(
            MediaErrors::ServiceUnavailable.path(),
            "Media.ServiceUnavailable"
        );
        assert_eq!(MediaErrors::AudioTooLarge.path(), "Media.AudioTooLarge");
        assert_eq!(
            MediaErrors::TranscribeRateLimited.path(),
            "Media.TranscribeRateLimited"
        );
        assert_eq!(MediaErrors::AudioMissing.path(), "Media.AudioMissing");
        assert_eq!(
            MediaErrors::Base64ImageInvalid.path(),
            "Media.Base64ImageInvalid"
        );
        assert_eq!(MediaErrors::FileTooLarge.path(), "Media.FileTooLarge");
        assert_eq!(
            MediaErrors::TranscribeFailed.path(),
            "Media.TranscribeFailed"
        );
        assert_eq!(MediaErrors::ParamsRequired.path(), "Media.ParamsRequired");
        assert_eq!(MediaErrors::IdInvalid.path(), "Media.IdInvalid");
        assert_eq!(MediaErrors::KindRequired.path(), "Media.KindRequired");
        assert_eq!(MEDIA_PATHS.len(), 11);
    }

    #[test]
    fn topology_paths_pinned() {
        assert_eq!(
            TopologyErrors::GatewayTimeout.path(),
            "Topology.GatewayTimeout"
        );
        assert_eq!(
            TopologyErrors::TargetDuplicate.path(),
            "Topology.TargetDuplicate"
        );
        assert_eq!(
            TopologyErrors::RemoteTargetMissing.path(),
            "Topology.RemoteTargetMissing"
        );
        assert_eq!(TopologyErrors::NoDialPath.path(), "Topology.NoDialPath");
        assert_eq!(
            TopologyErrors::ConnectionMissing.path(),
            "Topology.ConnectionMissing"
        );
        assert_eq!(
            TopologyErrors::ClaimedGatewayNoNodeId.path(),
            "Topology.ClaimedGatewayNoNodeId"
        );
        assert_eq!(
            TopologyErrors::TargetKindMismatch.path(),
            "Topology.TargetKindMismatch"
        );
        assert_eq!(
            TopologyErrors::NodeIdShapeInvalid.path(),
            "Topology.NodeIdShapeInvalid"
        );
        assert_eq!(
            TopologyErrors::NodeIdModeRequired.path(),
            "Topology.NodeIdModeRequired"
        );
        assert_eq!(TopologyErrors::HostRequired.path(), "Topology.HostRequired");
        assert_eq!(TOPOLOGY_PATHS.len(), 10);
    }

    #[test]
    fn console_paths_pinned() {
        assert_eq!(
            ConsoleErrors::CiNotConfigured.path(),
            "Console.CiNotConfigured"
        );
        assert_eq!(
            ConsoleErrors::CiWatchlistEmpty.path(),
            "Console.CiWatchlistEmpty"
        );
        assert_eq!(
            ConsoleErrors::CdNotConfigured.path(),
            "Console.CdNotConfigured"
        );
        assert_eq!(
            ConsoleErrors::EngineNotConfigured.path(),
            "Console.EngineNotConfigured"
        );
        assert_eq!(
            ConsoleErrors::RepairTargetNotConfigured.path(),
            "Console.RepairTargetNotConfigured"
        );
        assert_eq!(
            ConsoleErrors::ParamsRequired.path(),
            "Console.ParamsRequired"
        );
        assert_eq!(CONSOLE_PATHS.len(), 6);
    }

    #[test]
    fn channel_paths_pinned() {
        assert_eq!(ChannelErrors::SaveFailed.path(), "Channel.SaveFailed");
        assert_eq!(ChannelErrors::NotFound.path(), "Channel.NotFound");
        assert_eq!(ChannelErrors::InvalidParams.path(), "Channel.InvalidParams");
        assert_eq!(
            ChannelErrors::ParamsRequired.path(),
            "Channel.ParamsRequired"
        );
        assert_eq!(ChannelErrors::NoConfig.path(), "Channel.NoConfig");
        assert_eq!(
            ChannelErrors::NoActiveAdapter.path(),
            "Channel.NoActiveAdapter"
        );
        assert_eq!(ChannelErrors::ReloadFailed.path(), "Channel.ReloadFailed");
        assert_eq!(ChannelErrors::QueryFailed.path(), "Channel.QueryFailed");
        assert_eq!(ChannelErrors::SendFailed.path(), "Channel.SendFailed");
        assert_eq!(ChannelErrors::InsertFailed.path(), "Channel.InsertFailed");
        assert_eq!(ChannelErrors::DeleteFailed.path(), "Channel.DeleteFailed");
        assert_eq!(ChannelErrors::NameRequired.path(), "Channel.NameRequired");
        assert_eq!(ChannelErrors::IdRequired.path(), "Channel.IdRequired");
        assert_eq!(
            ChannelErrors::UnsupportedPlatform.path(),
            "Channel.UnsupportedPlatform"
        );
        assert_eq!(ChannelErrors::QuotaExceeded.path(), "Channel.QuotaExceeded");
        assert_eq!(
            ChannelErrors::CategoryInvalid.path(),
            "Channel.CategoryInvalid"
        );
        assert_eq!(CHANNEL_PATHS.len(), 16);
    }

    #[test]
    fn extensions_paths_pinned() {
        assert_eq!(
            ExtensionsErrors::SearchFailed.path(),
            "Extensions.SearchFailed"
        );
        assert_eq!(
            ExtensionsErrors::UpsertFailed.path(),
            "Extensions.UpsertFailed"
        );
        assert_eq!(
            ExtensionsErrors::EmbedFailed.path(),
            "Extensions.EmbedFailed"
        );
        assert_eq!(
            ExtensionsErrors::DocumentParamsInvalid.path(),
            "Extensions.DocumentParamsInvalid"
        );
        assert_eq!(
            ExtensionsErrors::TierLookupFailed.path(),
            "Extensions.TierLookupFailed"
        );
        assert_eq!(
            ExtensionsErrors::ParamsRequired.path(),
            "Extensions.ParamsRequired"
        );
        assert_eq!(EXTENSIONS_PATHS.len(), 6);
    }

    #[test]
    fn webhook_paths_pinned() {
        assert_eq!(WebhookErrors::IdRequired.path(), "Webhook.IdRequired");
        assert_eq!(
            WebhookErrors::ScopeTypeInvalid.path(),
            "Webhook.ScopeTypeInvalid"
        );
        assert_eq!(
            WebhookErrors::ScopeIdInvalid.path(),
            "Webhook.ScopeIdInvalid"
        );
        assert_eq!(
            WebhookErrors::ServiceAuthOnly.path(),
            "Webhook.ServiceAuthOnly"
        );
        assert_eq!(WebhookErrors::EventsInvalid.path(), "Webhook.EventsInvalid");
        assert_eq!(WebhookErrors::ScopesInvalid.path(), "Webhook.ScopesInvalid");
        assert_eq!(
            WebhookErrors::ParamsRequired.path(),
            "Webhook.ParamsRequired"
        );
        assert_eq!(WebhookErrors::NameRequired.path(), "Webhook.NameRequired");
        assert_eq!(WebhookErrors::NotFound.path(), "Webhook.NotFound");
        assert_eq!(WebhookErrors::IdInvalid.path(), "Webhook.IdInvalid");
        assert_eq!(WebhookErrors::InsertFailed.path(), "Webhook.InsertFailed");
        assert_eq!(WebhookErrors::UpdateFailed.path(), "Webhook.UpdateFailed");
        assert_eq!(WebhookErrors::DeleteFailed.path(), "Webhook.DeleteFailed");
        assert_eq!(WebhookErrors::RotateFailed.path(), "Webhook.RotateFailed");
        assert_eq!(WebhookErrors::QueryFailed.path(), "Webhook.QueryFailed");
        assert_eq!(WEBHOOK_PATHS.len(), 15);
    }

    #[test]
    fn iam_paths_pinned() {
        assert_eq!(IamErrors::ClientIdRequired.path(), "Iam.ClientIdRequired");
        assert_eq!(IamErrors::ClientNotFound.path(), "Iam.ClientNotFound");
        assert_eq!(IamErrors::ClientNotActive.path(), "Iam.ClientNotActive");
        assert_eq!(
            IamErrors::TokenEncryptFailed.path(),
            "Iam.TokenEncryptFailed"
        );
        assert_eq!(IamErrors::SecretSealFailed.path(), "Iam.SecretSealFailed");
        assert_eq!(
            IamErrors::GroupSourceRequiresSystemWrite.path(),
            "Iam.GroupSourceRequiresSystemWrite"
        );
        assert_eq!(
            IamErrors::BuiltinGroupImmutable.path(),
            "Iam.BuiltinGroupImmutable"
        );
        assert_eq!(
            IamErrors::MembershipRequired.path(),
            "Iam.MembershipRequired"
        );
        assert_eq!(
            IamErrors::ConsentRequiresMembership.path(),
            "Iam.ConsentRequiresMembership"
        );
        assert_eq!(IamErrors::GroupNotFound.path(), "Iam.GroupNotFound");
        assert_eq!(IamErrors::NameRequired.path(), "Iam.NameRequired");
        assert_eq!(IAM_PATHS.len(), 11);
    }

    #[test]
    fn billing_paths_pinned() {
        assert_eq!(
            BillingErrors::ReservationLost.path(),
            "Billing.ReservationLost"
        );
        assert_eq!(
            BillingErrors::WalletVanished.path(),
            "Billing.WalletVanished"
        );
        assert_eq!(BillingErrors::ModeInvalid.path(), "Billing.ModeInvalid");
        assert_eq!(BillingErrors::QuotaExceeded.path(), "Billing.QuotaExceeded");
        assert_eq!(
            BillingErrors::ParamsRequired.path(),
            "Billing.ParamsRequired"
        );
        assert_eq!(
            BillingErrors::SenderIdRequired.path(),
            "Billing.SenderIdRequired"
        );
        assert_eq!(
            BillingErrors::SenderIdInvalid.path(),
            "Billing.SenderIdInvalid"
        );
        assert_eq!(BillingErrors::Forbidden.path(), "Billing.Forbidden");
        assert_eq!(BILLING_PATHS.len(), 8);
    }

    #[test]
    fn edge_paths_pinned() {
        assert_eq!(
            EdgeErrors::CatalogEntryNotFound.path(),
            "Edge.CatalogEntryNotFound"
        );
        assert_eq!(EdgeErrors::IntentIdRequired.path(), "Edge.IntentIdRequired");
        assert_eq!(EdgeErrors::DeviceIdRequired.path(), "Edge.DeviceIdRequired");
        assert_eq!(
            EdgeErrors::EnrollmentBudgetExhausted.path(),
            "Edge.EnrollmentBudgetExhausted"
        );
        assert_eq!(
            EdgeErrors::SerialOrEnrollmentIdRequired.path(),
            "Edge.SerialOrEnrollmentIdRequired"
        );
        assert_eq!(EDGE_PATHS.len(), 5);
    }

    #[test]
    fn visitor_paths_pinned() {
        assert_eq!(
            VisitorErrors::TicketRequired.path(),
            "Visitor.TicketRequired"
        );
        assert_eq!(
            VisitorErrors::TicketNotFound.path(),
            "Visitor.TicketNotFound"
        );
        assert_eq!(
            VisitorErrors::ObservationNotFound.path(),
            "Visitor.ObservationNotFound"
        );
        assert_eq!(
            VisitorErrors::TicketIdRequired.path(),
            "Visitor.TicketIdRequired"
        );
        assert_eq!(VISITOR_PATHS.len(), 4);
    }

    #[test]
    fn panel_paths_pinned() {
        assert_eq!(PanelErrors::RevisionInvalid.path(), "Panel.RevisionInvalid");
        assert_eq!(PanelErrors::ViewRefInvalid.path(), "Panel.ViewRefInvalid");
        assert_eq!(
            PanelErrors::ExpectedRevisionRequired.path(),
            "Panel.ExpectedRevisionRequired"
        );
        assert_eq!(PanelErrors::RoleRequired.path(), "Panel.RoleRequired");
        assert_eq!(PanelErrors::ReadOnly.path(), "Panel.ReadOnly");
        assert_eq!(PanelErrors::McpHubRequired.path(), "Panel.McpHubRequired");
        assert_eq!(
            PanelErrors::McpIdToolRequired.path(),
            "Panel.McpIdToolRequired"
        );
        assert_eq!(
            PanelErrors::ArgumentsInvalid.path(),
            "Panel.ArgumentsInvalid"
        );
        assert_eq!(PanelErrors::IdRequired.path(), "Panel.IdRequired");
        assert_eq!(PanelErrors::NotFound.path(), "Panel.NotFound");
        assert_eq!(PANEL_PATHS.len(), 10);
    }

    #[test]
    fn audit_paths_pinned() {
        assert_eq!(
            AuditErrors::ExportRecordWriteFailed.path(),
            "Audit.ExportRecordWriteFailed"
        );
        assert_eq!(AuditErrors::NothingToPurge.path(), "Audit.NothingToPurge");
        assert_eq!(AuditErrors::BatchTooLarge.path(), "Audit.BatchTooLarge");
        assert_eq!(AuditErrors::ServiceAuthOnly.path(), "Audit.ServiceAuthOnly");
        assert_eq!(AuditErrors::ParamsRequired.path(), "Audit.ParamsRequired");
        assert_eq!(AuditErrors::FactsRequired.path(), "Audit.FactsRequired");
        assert_eq!(AuditErrors::QueryFailed.path(), "Audit.QueryFailed");
        assert_eq!(AuditErrors::ExportFailed.path(), "Audit.ExportFailed");
        assert_eq!(
            AuditErrors::ExportedBeforeRequired.path(),
            "Audit.ExportedBeforeRequired"
        );
        assert_eq!(AUDIT_PATHS.len(), 9);
    }

    #[test]
    fn plugin_paths_pinned() {
        assert_eq!(PluginErrors::IdRequired.path(), "Plugin.IdRequired");
        assert_eq!(
            PluginErrors::TargetTypeRequired.path(),
            "Plugin.TargetTypeRequired"
        );
        assert_eq!(PluginErrors::PointRequired.path(), "Plugin.PointRequired");
        assert_eq!(PLUGIN_PATHS.len(), 3);
    }

    #[test]
    fn the_code_is_the_placeholder_and_paths_are_the_identity() {
        assert_eq!(
            APP_ERROR_CODE, -32000,
            "the app-error code is the placeholder by ruling; paths are the identity"
        );
        // Pairwise-unique across the whole table — an error path names
        // exactly one condition.
        let flat: Vec<&str> = ALL_ERROR_PATHS.concat();
        let mut sorted = flat.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            flat.len(),
            sorted.len(),
            "duplicate error path in the table"
        );
    }

    /// Regenerate `bindings/errorPaths.ts` from the trees' path tables.
    /// Runs under the plain `cargo test` regime like the ts-rs export
    /// tests, so CI's regeneration pass is the drift gate: a tree change
    /// without regenerated bindings fails the suite on the diff.
    #[test]
    fn export_error_paths_bindings() {
        fn tree_const(name: &str, doc: &str, paths: &[&str]) -> String {
            let entries: Vec<String> = paths
                .iter()
                .map(|p| {
                    let variant = p.rsplit('.').next().unwrap();
                    format!("  {variant}: \"{p}\",")
                })
                .collect();
            // MINOR-1 guard: two paths in ONE tree sharing a last segment
            // would collide as a duplicate TS object key (silent
            // last-wins in JS object literals).
            let mut keys: Vec<&str> = paths
                .iter()
                .map(|p| p.rsplit('.').next().unwrap())
                .collect();
            let before = keys.len();
            keys.sort_unstable();
            keys.dedup();
            assert_eq!(before, keys.len(), "duplicate TS object key in {name}");
            format!(
                "/** {doc} */\nexport const {name} = {{\n{}\n}} as const;\n",
                entries.join("\n")
            )
        }

        let trees = [
            tree_const(
                "USAGE_ERROR_PATHS",
                "Usage tree paths (chest's usage.get seam).",
                USAGE_PATHS,
            ),
            tree_const(
                "SCHEDULER_ERROR_PATHS",
                "Scheduler tree paths (the arona metering plane).",
                SCHEDULER_PATHS,
            ),
            tree_const(
                "PLATFORM_ERROR_PATHS",
                "Platform tree paths (the platform-pinning gate).",
                PLATFORM_PATHS,
            ),
            tree_const(
                "SELF_HOSTED_ERROR_PATHS",
                "Models.SelfHosted tree paths (the self-hosted backend registry).",
                SELF_HOSTED_PATHS,
            ),
            tree_const(
                "AUTHORITY_ERROR_PATHS",
                "Authority tree paths (chest's shared authority gates).",
                AUTHORITY_PATHS,
            ),
            tree_const(
                "UPSTREAM_ERROR_PATHS",
                "Upstream tree paths (chest's upstream transport failures).",
                UPSTREAM_PATHS,
            ),
            tree_const(
                "TOPIC_ERROR_PATHS",
                "Topic tree paths (chest's topics/chat faces).",
                TOPIC_PATHS,
            ),
            tree_const(
                "WORKSPACE_ERROR_PATHS",
                "Workspace tree paths (chest's workspace/tenancy faces).",
                WORKSPACE_PATHS,
            ),
            tree_const(
                "AUTH_ERROR_PATHS",
                "Auth tree paths (chest's account/session faces).",
                AUTH_PATHS,
            ),
            tree_const(
                "PROVIDER_ERROR_PATHS",
                "Provider tree paths (chest's provider config faces).",
                PROVIDER_PATHS,
            ),
            tree_const(
                "MEDIA_ERROR_PATHS",
                "Media tree paths (chest's audio/video/voice/media faces).",
                MEDIA_PATHS,
            ),
            tree_const(
                "TOPOLOGY_ERROR_PATHS",
                "Topology tree paths (chest's topology/connections faces).",
                TOPOLOGY_PATHS,
            ),
            tree_const(
                "CONSOLE_ERROR_PATHS",
                "Console tree paths (chest's operator/ops faces).",
                CONSOLE_PATHS,
            ),
            tree_const(
                "WEBHOOK_ERROR_PATHS",
                "Webhook tree paths (chest's webhook config face).",
                WEBHOOK_PATHS,
            ),
            tree_const(
                "IAM_ERROR_PATHS",
                "Iam tree paths (chest's iam + oauth_clients faces).",
                IAM_PATHS,
            ),
            tree_const(
                "BILLING_ERROR_PATHS",
                "Billing tree paths (chest's billing authority/quota faces).",
                BILLING_PATHS,
            ),
            tree_const(
                "EDGE_ERROR_PATHS",
                "Edge tree paths (chest's edge_flash/admission/enroll/proxy faces).",
                EDGE_PATHS,
            ),
            tree_const(
                "VISITOR_ERROR_PATHS",
                "Visitor tree paths (chest's visitor faces).",
                VISITOR_PATHS,
            ),
            tree_const(
                "PANEL_ERROR_PATHS",
                "Panel tree paths (chest's panel/projects/mcp_tools faces).",
                PANEL_PATHS,
            ),
            tree_const(
                "AUDIT_ERROR_PATHS",
                "Audit tree paths (chest's audit/usage_facts faces).",
                AUDIT_PATHS,
            ),
            tree_const(
                "PLUGIN_ERROR_PATHS",
                "Plugin tree paths (chest's plugins.rs face).",
                PLUGIN_PATHS,
            ),
            tree_const(
                "CHANNEL_ERROR_PATHS",
                "Channel tree paths (chest's channel admin + personal-channel faces).",
                CHANNEL_PATHS,
            ),
            tree_const(
                "EXTENSIONS_ERROR_PATHS",
                "Extensions tree paths (chest's extensions/RAG face).",
                EXTENSIONS_PATHS,
            ),
        ]
        .join("\n");
        let flat: Vec<&str> = ALL_ERROR_PATHS.concat();
        let flat_entries: Vec<String> = flat.iter().map(|p| format!("  \"{p}\",")).collect();
        let content = format!(
            "// This file is generated by the error-path export test (cargo test). Do not edit manually.\n\
             // Source of truth: packages/celestia-types/src/error_paths.rs (yuuka trees).\n\n\
             {trees}\n\
             /** Every path in the table, in declaration order. */\n\
             export const ERROR_PATHS = [\n{}\n] as const;\n\n\
             export type ErrorPath = (typeof ERROR_PATHS)[number];\n\n\
             /** Exact-match helper: does an error message carry this path prefix?\n \
             * Server messages are `\"<Path>: <prose>\"`; matching on the delimited\n \
             * prefix keeps `Usage.Forbidden` from matching `Usage.ForbiddenExtra`. */\n\
             export function errorPathMatches(message: string, path: string): boolean {{\n  \
             return message === path || message.startsWith(`${{path}}:`);\n}}\n",
            flat_entries.join("\n"),
        );

        let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/bindings");
        let out_path = format!("{out_dir}/errorPaths.ts");
        let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
        if existing != content {
            std::fs::write(&out_path, &content).expect("write bindings/errorPaths.ts");
        }
    }
}
