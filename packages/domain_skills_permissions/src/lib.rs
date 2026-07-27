//! Permission and security model for agent skills and tool execution.
//!
//! This crate defines the authorization, risk-classification, and runtime
//! security checks that gate every tool invocation in the Entelecheia platform.
//!
//! Key abstractions:
//! - [`AccessMode`] (Read/Write/Execute), [`RiskLevel`] (Info through Critical),
//!   [`TrustLevel`] — the core permission taxonomy.
//! - [`PermissionDecision`] — the outcome of evaluating a [`SkillToolRequest`]
//!   against access mode, execution mode, tool scope, and trust level.
//! - [`ToolSecurityPipeline`] — runtime pipeline that applies rate limiting,
//!   invocation-depth caps, dangerous-parameter detection, and dual-authorization
//!   checks before a tool executes.
//! - [`ToolAuditLog`] / [`ToolAuditEntry`] — append-only audit trail with
//!   parameter hashing for tamper detection.
//! - [`ToolIdentity`] — origin identity (agent, skill, session) propagated
//!   through the call chain for attribution.
//!
//! Design philosophy: defense in depth — static classification (compile-time)
//! feeds into runtime policy evaluation, with every decision auditable.
//! Security is enforced at the boundary, never inside individual tools.
#![allow(clippy::type_complexity)]

pub mod tool_audit;
pub mod tool_identity;
pub mod tool_permissions;
pub mod tool_security;

pub use tool_audit::{
    AuditSink, JsonlAuditSink, SharedAuditLog, ToolAuditEntry, ToolAuditLog, compute_params_hash,
    shared_audit_log,
};
pub use tool_identity::ToolIdentity;
pub use tool_permissions::{
    AccessMode, CommandSafety, DenialKind, ExecutionMode, PermissionDecision, RiskLevel,
    SkillToolRequest, ToolCapability, ToolScope, TrustLevel, UnknownAccessModeError,
    UnknownExecutionModeError, check_command_safety, check_dual_authorization,
    check_execution_location_gate, classify_command, permission_name_to_access_mode,
};
pub use tool_security::{
    RateLimitConfig, SecurityAction, ToolSecurityPipeline, ToolSecurityVerdict,
    check_dangerous_params,
};
