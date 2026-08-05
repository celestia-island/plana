//! Foundation domain types shared across the entire entelecheia platform.
//!
//! This crate is the system's **single source of truth** for the core vocabulary: what an
//! agent *is*, how errors are structured, which execution modes exist, and how tools are
//! defined. Every other crate (storage, container, TUI, infra) depends on these types so
//! that cross-cutting concepts are defined exactly once and everyone speaks the same
//! language.
//!
//! Key abstractions:
//! - **Identity** — [`AgentId`], [`ContainerId`], [`LlmSessionId`] — strongly-typed wrappers
//!   that prevent ID confusion at compile time.
//! - **Errors** — [`AgentErrorCode`] + [`StructuredAgentError`] — a taxonomy of failure modes
//!   (model selection, rate-limiting, chaining, skills) serializable across process boundaries.
//! - **Execution mode** — `Read`/`Write`/`Edge` — controls the level of side-effect authority
//!   granted to an agent run.
//! - **Tool definition** — [`ToolDefinition`] — the canonical shape for MCP-style tool schemas.
//! - **Safety & workspace** — shell-metacharacter detection, workspace identity/registry,
//!   credential storage, model-tier gating.
//!
//! The crate has **zero internal dependencies** on other entelecheia crates; it is a
//! leaf-level library that only pulls from the Rust ecosystem (`serde`, `thiserror`,
//! `tracing`). This ensures circular-dependency protection and makes it cheap to compile.
#![allow(clippy::type_complexity)]

pub mod agent_badge;
pub mod cli;
pub mod constants;
pub mod credentials;
pub mod enums;
pub mod errors;
pub mod execution_mode;
pub mod identity;
pub mod llm_image_content;
pub mod logger;
pub mod mcp_call_mode;
pub mod model_tier;
pub mod ref_namespace;
pub mod shell_safety;
pub mod state_tree;
pub mod thread_types;
pub mod tool_definition;
pub mod usage_aggregation;
pub mod utils;
pub mod var_namespace;
pub mod version;
pub mod workspace;
pub mod workspace_ref;

pub use agent_badge::AgentBadge;
pub use constants::{
    AgentAction, BaseAction, CONFIG, DEFAULT_NETWORK, HttpMethod, LlmProvider, LogLevel, McpAction,
    MessageType, MonitorAction, NodeAction, SkillAction, UnknownLogLevelError, is_invalid_api_key,
};
pub use credentials::CredentialStorage;
pub use enums::{
    AlignmentStatus, CheckStatus, CheckType, ComplianceStatus, ConversationMessageType,
    ObservationType, Priority, ScanType, Severity, TaskStatus, TriggerType,
};
pub use errors::{AgentErrorCode, CredentialError, StructuredAgentError};
pub use execution_mode::{ExecutionMode, UnknownExecutionModeError};
pub use identity::{AgentId, AgentIdentity, ContainerId, LlmSessionId};
pub use llm_image_content::{LlmAudioContent, LlmImageContent};
pub use logger::{init_logger, init_logger_text, init_logger_tui};
pub use mcp_call_mode::McpToolCallMode;
pub use model_tier::ModelTier;
pub use shell_safety::contains_shell_metacharacters;
pub use tool_definition::ToolDefinition;
pub use utils::{
    bytes_base64, detect_platform_metadata, detect_wsl, format_timestamp, generate_id, is_blank,
    now, now_timestamp, simplify_platform_title, truncate,
};
pub use version::{VERSION, check_version_compatibility, is_compatible};
pub use workspace::{
    WorkspaceConnectionKind, WorkspaceDescriptor, WorkspaceIdentity, WorkspaceRegistry,
    WritebackMode,
};
pub use workspace_ref::{WorkspaceRef, WorkspaceScopedBadge, WorkspaceScopedSessionId};
