//! TUI snapshot / patch types for the dashboard wire.
//!
//! Snapshots (`AgentSnapshot`, `ContainerSnapshot`, `TasksSnapshot`,
//! `GlobalSnapshot`) replace a whole client-side collection; the `*Patch`
//! types beside them carry field-level increments instead. Both travel as
//! `Sync.*` JSON-RPC payloads, which `SyncMessage` tags with `action`.
//!
//! These are pure data types: no producer or consumer lives in this crate,
//! and no `Vec` field promises an order beyond the sender's enumeration.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent::{CompletionOutcome, RequestState, TuiAgentInfo};
use crate::agent::{Agent, WorkStatus};
use cherino::ContainerStatus;
use plana_core::{AgentBadge, AgentId};

/// Field-level incremental update for one agent - not a complete agent object.
///
/// Sent as `Sync.AgentPatch`, whose params carry the patch array under
/// `patches`. A receiver applies every non-null field as a whole-value replace
/// and leaves absent fields untouched, so `None` never means "clear this"
/// (`upsert_agent_patches`, `packages/sync/src/domains/agents.rs:56`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPatch {
    /// Identity of the agent this patch targets; the only non-optional field. A
    /// receiver without it falls back to `agent_number`, then `agent_type`, as
    /// the tree key (`patch_agent_key`, `packages/sync/src/domains/agents.rs:81`).
    pub agent_id: AgentId,
    /// Badge number of the agent in the panel (`001`-style, `demiurge` for the
    /// root) - also the first fallback key when `agent_id` is unusable.
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    /// Agent kind (for example `HubRis` or `SkeMma`); written through to the client
    /// and used as the last-resort patch key.
    #[serde(default)]
    pub agent_type: Option<Agent>,
    /// Sender-assigned revision for this agent, stored verbatim as
    /// `state.agents.<id>.version` - the only ordering signal a field-level
    /// patch carries (`upsert_patches_field_level_incremental`,
    /// `packages/sync/src/domains/agents.rs:232`).
    pub version: u64,
    /// New value of the agent's LLM-working flag when it flipped. The `_changed`
    /// suffix means "new value, sent only on change", not a delta.
    #[serde(default)]
    pub llm_working_changed: Option<bool>,
    /// Replacement `WorkStatus` - the tagged execution phase, such as `Thinking`
    /// or `Executing { skill_name }`.
    #[serde(default)]
    pub work_status: Option<WorkStatus>,
    /// Id of the model the agent is running on right now.
    #[serde(default)]
    pub current_model: Option<String>,
    /// Tier of `current_model` (`Deep`, `Normal` or `Basic`).
    #[serde(default)]
    pub model_tier: Option<crate::types::ModelTier>,
    /// Opaque handle of the agent's in-flight LLM call, for correlating its logs.
    #[serde(default)]
    pub llm_handle: Option<String>,
    /// Token increment `(input, output)` to add to the client's running counters;
    /// the pair order matches `WorkspaceTokenUsage`
    /// (`message/types/mod.rs:350`).
    #[serde(default)]
    pub token_usage_delta: Option<(u32, u32)>,
    /// Full `(input, output)` token totals, in the same order - the
    /// non-incremental counterpart of `token_usage_delta`.
    #[serde(default)]
    pub token_usage_absolute: Option<(u32, u32)>,
    /// Replacement `RequestState` of the agent's current turn (`Idle`, `Waiting`,
    /// `Streaming`, `Retrying` or `WaitingTool`).
    #[serde(default)]
    pub request_state: Option<RequestState>,
    /// Increment to add to the agent's cumulative tool-call counter.
    #[serde(default)]
    pub tool_calls_delta: Option<u32>,
    /// Increment to add to the agent's cumulative skill-call counter.
    #[serde(default)]
    pub skill_calls_delta: Option<u32>,
    /// CPU load of the agent; the unit is not pinned by this crate (see
    /// `ContainerInfo.cpu_usage`, which carries the same measure).
    #[serde(default)]
    pub cpu_usage: Option<f64>,
    /// Resident memory of the agent in MiB - the unit is in the field name.
    #[serde(default)]
    pub memory_mb: Option<u64>,
    /// Replacement `CompletionOutcome` (`None`, `Reported` or `Failed`): how the
    /// agent's turn ended.
    #[serde(default)]
    pub completion_outcome: Option<CompletionOutcome>,
    /// Retries already consumed on the current turn.
    #[serde(default)]
    pub retry_count: Option<u32>,
    /// Retry budget for the current turn, the ceiling `retry_count` is measured
    /// against.
    #[serde(default)]
    pub max_retries: Option<u32>,
    /// Name of the skill-chain stage the agent is executing now, matching the stage
    /// names `SkillStage` events carry.
    #[serde(default)]
    pub current_stage: Option<String>,
    /// Name of the stage the chain is expected to run next.
    #[serde(default)]
    pub next_stage: Option<String>,
    /// Tool the agent is executing, rendered next to the agent type while
    /// `work_status` is `CallingTool` (`WorkStatus::display_tag`, `agent.rs:309`).
    #[serde(default)]
    pub current_tool_name: Option<String>,
    /// Id of the agent that spawned this one; `None` for a root agent.
    #[serde(default)]
    pub parent_id: Option<String>,
}

/// Full agent roster: one `TuiAgentInfo` per live agent. The list replaces the
/// client's whole agent view and is never merged element-wise.
///
/// Pushed one-way as `Sync.AgentSnapshot`, with no request of its own in the
/// `Sync` namespace (`packages/plana/src/jsonrpc/pending.rs:149`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    /// Revision of the sender's agent store this roster was cut from - the
    /// snapshot-level counterpart of `AgentPatch.version`.
    pub version: u64,
    /// Wall-clock time the snapshot was produced, epoch-millis `i64` (the
    /// convention of `Sync.UserMessage.timestamp`, `message/types/mod.rs:28`).
    pub timestamp: i64,
    /// The roster itself, in the sender's enumeration order.
    pub agents: Vec<TuiAgentInfo>,
}

/// Whole-dashboard snapshot: agents, containers and active tasks in one
/// payload, replacing all three client collections at once.
///
/// Pushed as `Sync.GlobalSnapshot` and as the one-way answer to
/// `Sync.RequestGlobalSnapshot` (`packages/plana/src/jsonrpc/pending.rs:120`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSnapshot {
    /// Revision of the sender's state store that all three lists were cut from.
    pub version: u64,
    /// Wall-clock time the snapshot was produced, epoch-millis `i64`.
    pub timestamp: i64,
    /// Full agent roster, the same rows as `AgentSnapshot.agents`.
    pub agents: Vec<TuiAgentInfo>,
    /// Full container roster, the same rows as `ContainerSnapshot.containers`.
    pub containers: Vec<ContainerInfo>,
    /// Tasks the sender counts as live - a working set, not the full task history a
    /// sender may have stored.
    pub active_tasks: Vec<TaskInfo>,
}

/// One model row inside `Sync.ModelsSnapshot`, which pushes the entire list in a
/// single message (`packages/plana/src/jsonrpc/pending.rs:136`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Identifier the rest of the protocol selects this model by, as carried by
    /// `AgentPatch.current_model`.
    pub id: String,
    /// Human-readable label for the model in the dashboard.
    pub name: String,
    /// Provider this model belongs to, matching `ProviderInfo.name`.
    pub provider_name: String,
    /// Semantics TBC — see `Sync.ModelsSnapshot`, message/types/mod.rs:866.
    pub model_type: String,
    /// Context window of the model in tokens; `None` when the sender does not know
    /// it (the FS mirror calls the same value `context_window`, `config_fs.rs:114`).
    pub context_length: Option<u32>,
    /// Whether the model is currently selectable; configured-but-disabled models
    /// report `false`.
    pub is_active: bool,
}

/// One configured LLM provider inside `Sync.ProvidersSnapshot`, which pushes the
/// entire provider list in a single message
/// (`packages/plana/src/jsonrpc/pending.rs:137`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider key used to match models and config entries; not necessarily
    /// human-readable.
    pub name: String,
    /// Label shown for the provider in the dashboard.
    pub display_name: String,
    /// Base URL of the provider API; no credential material is part of it.
    pub api_endpoint: String,
    /// Whether the server holds a stored key for this provider - the key value
    /// itself never travels on this wire.
    pub has_api_key: bool,
    /// Model id used when a request names none; `None` when the provider has no
    /// default configured.
    pub default_model: Option<String>,
    /// Whether the provider is currently enabled for model selection.
    pub is_active: bool,
}

/// One task row inside `TasksSnapshot` and `GlobalSnapshot.active_tasks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task identity, matching the `task_id` of `Sync.TaskStatusUpdate` and
    /// `TaskPatch`.
    pub id: Uuid,
    /// Issue (goal group) the task was spawned for - several tasks can share one id
    /// (`Sync.TaskCreated`, `message/types/mod.rs:726`).
    pub issue_id: Uuid,
    /// Title set at creation, shown as the task's label.
    pub title: String,
    /// Lifecycle status; snake_case strings (`not_started`, `in_progress`, ...) plus
    /// the externally tagged `{"waiting": {...}}` form (`crate::TaskStatus`, `types.rs:11`).
    pub status: crate::TaskStatus,
    /// Coarse completion counter, the snapshot twin of `progress` on
    /// `Sync.TaskStatusUpdate` (`message/types/mod.rs:748`); the scale is not
    /// pinned by this crate.
    pub progress: u8,
    /// Id of the agent working the task; `None` while it is unassigned.
    pub assigned_agent: Option<String>,
}

/// Field-level incremental update for one container, sent as
/// `Sync.ContainerPatch`. The `*_changed` fields carry the new value and are
/// omitted when nothing moved; absent never means "clear".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPatch {
    /// Identity of the container this patch targets, as the runtime reports it (a
    /// string, not a UUID).
    pub container_id: String,
    /// Sender-assigned revision for this container, the ordering field mirroring
    /// `AgentPatch.version`.
    pub version: u64,
    /// New `ContainerStatus` (`running`, `exited`, ...) when it changed.
    #[serde(default)]
    pub status_changed: Option<ContainerStatus>,
    /// New CPU load value when it changed; same measure as
    /// `ContainerInfo.cpu_usage`.
    #[serde(default)]
    pub cpu_usage_changed: Option<f64>,
    /// New memory usage in MiB when it changed; `ContainerInfo` names the same
    /// measure `memory_mb`.
    #[serde(default)]
    pub memory_usage_changed: Option<u64>,
    /// New branch of the container when it changed.
    #[serde(default)]
    pub branch_changed: Option<String>,
    /// New read-only flag when it flipped.
    #[serde(default)]
    pub is_read_only_changed: Option<bool>,
    /// New panel badge when it was assigned or renamed.
    #[serde(default)]
    pub badge_changed: Option<AgentBadge>,
    /// New skill the container is executing when it changed.
    #[serde(default)]
    pub current_skill_changed: Option<String>,
}

/// Field-level incremental update for one task, sent as `Sync.TaskPatch`; `task_id`
/// is the only field a sender must provide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPatch {
    /// Identity of the task this patch targets.
    pub task_id: Uuid,
    /// Sender-assigned revision for this task, mirroring `AgentPatch.version`.
    pub version: u64,
    /// New `crate::TaskStatus` when the task moved.
    #[serde(default)]
    pub status_changed: Option<crate::TaskStatus>,
    /// New progress value when it moved; same unpinned scale as
    /// `TaskInfo.progress`.
    #[serde(default)]
    pub progress_changed: Option<u8>,
}

/// One container row inside `ContainerSnapshot` and `GlobalSnapshot`.
///
/// Carries the runtime state plus the workspace/git binding of the container.
/// The four `workspace_*` / `git_*` fields are the ones older payloads omit;
/// `test_container_info_backward_compatible` pins that they still parse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// Container id as the runtime reports it (a string, not a UUID).
    pub id: String,
    /// Container name as created, for example `e-skemma-abc12345`.
    pub name: String,
    /// Runtime status; the wire literals come from `cherino::ContainerStatus`
    /// (`running`, `exited`, ...).
    pub status: ContainerStatus,
    /// CPU load of the container; the unit is not pinned by this crate
    /// (`ContainerPatch.cpu_usage_changed` carries the same measure).
    pub cpu_usage: f64,
    /// Resident memory in MiB - the unit is in the field name.
    pub memory_mb: u64,
    /// Image reference the container was started from; empty string when the sender
    /// omits it, which `serde(default)` turns into the same value.
    #[serde(default)]
    pub image: String,
    /// Agent kind as a plain string (the runtime's label), not the typed `Agent`
    /// enum that `AgentPatch.agent_type` carries.
    #[serde(default)]
    pub agent_type: Option<String>,
    /// Id of the container this one was forked from; `None` for a root.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Fork depth: `0` for a root container, plus one per fork
    /// (`ContainerForkResult.branch_level`,
    /// `packages/celestia-types/src/tools/neikos.rs:90`); absent means `0`.
    #[serde(default)]
    pub branch_level: u32,
    /// Whether this is a cosmos (git-branching sandbox) container rather than a
    /// plain one; absent means `false`.
    #[serde(default)]
    pub is_cosmos: bool,
    /// Cosmos branch the container belongs to, for example
    /// `cosmos/auto-plan_execute`; `None` when it has no branch.
    #[serde(default)]
    pub branch: Option<String>,
    /// Whether the container's workspace is mounted read-only; absent means
    /// `false`.
    #[serde(default)]
    pub is_read_only: bool,
    /// Panel badge assigned to the container's agent, when it has one.
    #[serde(default)]
    pub badge: Option<AgentBadge>,
    /// Skill the container is executing now; `None` when it is idle.
    #[serde(default)]
    pub current_skill: Option<String>,
    /// Workspace the container is bound to, as an opaque id string; `None` on
    /// payloads from before the workspace fields existed.
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// Host path of that workspace as mounted into the container (the compat
    /// fixture uses `/home/user/project`); `None` on legacy payloads.
    #[serde(default)]
    pub workspace_path: Option<String>,
    /// Origin remote of the container's workspace checkout; `None` on legacy
    /// payloads or when the workspace has no remote.
    #[serde(default)]
    pub git_remote_url: Option<String>,
    /// Branch checked out in the workspace named by `git_remote_url`; `None` on
    /// legacy payloads.
    #[serde(default)]
    pub git_branch: Option<String>,
}

/// Full container roster, replacing the client's whole container view.
///
/// One-way answer to `Sync.RequestContainerSnapshot`
/// (`packages/plana/src/jsonrpc/pending.rs:122`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSnapshot {
    /// Revision of the sender's container store this roster was cut from.
    pub version: u64,
    /// Wall-clock time the snapshot was produced, epoch-millis `i64`.
    pub timestamp: i64,
    /// The roster itself, in the sender's enumeration order.
    pub containers: Vec<ContainerInfo>,
}

/// Full task roster, replacing the client's whole task view.
///
/// One-way answer to `Sync.RequestTasksSnapshot`
/// (`packages/plana/src/jsonrpc/pending.rs:124`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksSnapshot {
    /// Revision of the sender's task store this roster was cut from.
    pub version: u64,
    /// Wall-clock time the snapshot was produced, epoch-millis `i64`.
    pub timestamp: i64,
    /// The roster itself, in the sender's enumeration order.
    pub tasks: Vec<TaskInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn test_container_info_with_workspace_fields() -> Result<()> {
        let info = ContainerInfo {
            id: "container-123".to_string(),
            name: "e-skemma-abc12345".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 0.5,
            memory_mb: 256,
            image: "127.0.0.1:5000/entelecheia".to_string(),
            agent_type: Some("SkeMma".to_string()),
            parent_id: None,
            branch_level: 0,
            is_cosmos: true,
            branch: Some("cosmos/auto-plan_execute".to_string()),
            is_read_only: false,
            badge: Some(AgentBadge::new("001").context("invalid badge")?),
            current_skill: Some("plan_execute".to_string()),
            workspace_id: Some("ws-uuid-here".to_string()),
            workspace_path: Some("/home/user/project".to_string()),
            git_remote_url: Some("https://github.com/org/repo.git".to_string()),
            git_branch: Some("main".to_string()),
        };

        let json = serde_json::to_string(&info)?;
        let deserialized: ContainerInfo = serde_json::from_str(&json)?;

        assert_eq!(deserialized.workspace_id, Some("ws-uuid-here".to_string()));
        assert_eq!(
            deserialized.workspace_path,
            Some("/home/user/project".to_string())
        );
        assert_eq!(
            deserialized.git_remote_url,
            Some("https://github.com/org/repo.git".to_string())
        );
        assert_eq!(deserialized.git_branch, Some("main".to_string()));
        Ok(())
    }

    #[test]
    fn test_container_info_backward_compatible() -> Result<()> {
        let old_json = r#"{
            "id": "c1",
            "name": "e-test",
            "status": "running",
            "cpu_usage": 0.0,
            "memory_mb": 0,
            "image": "",
            "agent_type": null,
            "parent_id": null,
            "branch_level": 0,
            "is_cosmos": false,
            "branch": null,
            "is_read_only": false,
            "badge": null,
            "current_skill": null
        }"#;
        let info: ContainerInfo = serde_json::from_str(old_json)?;
        assert!(info.workspace_id.is_none());
        assert!(info.workspace_path.is_none());
        assert!(info.git_remote_url.is_none());
        assert!(info.git_branch.is_none());
        Ok(())
    }
}

/// One tracing record forwarded to the dashboard by the log subscription
/// (`Sync.SubscribeServerLogs` / `SubscribeContainerLogs` and their live
/// `ServerLogEntry` / `ContainerLogEntry` pushes).
///
/// The fields mirror the `log.entries` row written by
/// `packages/infra_services/src/pg_log_writer.rs:11`, whose values are built by
/// `PgLogLayer` (`packages/infra_services/src/pg_log_layer.rs:38`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryData {
    /// Label of the emitting service, fixed when the log layer was installed
    /// (`PgLogLayer::new(tx, source)`).
    pub source: String,
    /// Container instance the record came from; `None` for server-wide records -
    /// the layer itself always sets `None`.
    pub instance_uuid: Option<String>,
    /// Lowercase level as produced by the layer: `error`, `warn`, `info`, `debug` or
    /// `trace` (`packages/infra_services/src/pg_log_layer.rs:60`).
    pub level: String,
    /// Tracing target of the record, i.e. the module path that emitted it.
    pub target: Option<String>,
    /// Rendered message text, taken from the event's `message` field.
    pub message: String,
    /// Remaining structured event fields as a JSON object of stringified values;
    /// an empty object when the event carried none.
    pub fields: serde_json::Value,
    /// RFC 3339 UTC creation time with millisecond precision and a `Z` suffix
    /// (`packages/infra_services/src/pg_log_layer.rs:53`).
    pub created_at: String,
}
