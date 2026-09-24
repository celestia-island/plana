//! Agent telemetry for the TUI: the parameters an engine publishes for one
//! agent update, and the `TuiAgentInfo` row the client renders from it.
use serde::{Deserialize, Serialize};

use crate::agent::{Agent, AgentStatus, WorkStatus};
use plana_core::{AgentBadge, AgentId};

/// What the agent's current LLM request is doing; drives the TUI status
/// indicator. Serialized with the variant name verbatim (`"Idle"`, ...) — the
/// enum declares no `rename_all`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RequestState {
    /// No request in flight; the agent is between turns (the `Default`).
    #[default]
    Idle,
    /// A request was issued and the agent is waiting for the first response.
    Waiting,
    /// Response content is streaming back right now.
    Streaming,
    /// The previous attempt failed and the agent is retrying.
    Retrying,
    /// Paused until the tool call the agent issued returns.
    WaitingTool,
}

/// How the agent's last finished request ended. `None` is the initial value,
/// not a failure, and the wire form is the variant name verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompletionOutcome {
    /// No completion recorded yet (fresh agent or default-constructed value).
    #[default]
    None,
    /// The agent finished the request and reported its result.
    Reported,
    /// The request ended in failure.
    Failed,
}

/// Parameter bundle for publishing one agent update: engines fill in only what
/// they know and leave the rest at `Default` (a `system`-id, idle row).
#[derive(Debug, Clone)]
pub struct AgentUpdateParams {
    /// Identity of the agent this update is about; serialized as a plain string
    /// because `AgentId` is serde-transparent. `Default` uses `AgentId::system()`.
    pub agent_id: AgentId,
    /// Panel badge in its raw string form (e.g. `123` or `123.456`, without the
    /// `#`); `None` when the agent has no badge.
    pub agent_number: Option<String>,
    /// Agent kind; `None` when the caller does not pin one.
    pub agent_type: Option<Agent>,
    /// Whether an LLM request is currently in flight for this agent.
    pub llm_working: bool,
    /// Fine-grained work phase (`Thinking`, `Executing`, ...) when the engine
    /// reports one.
    pub work_status: Option<WorkStatus>,
    /// Model id currently serving the agent; `None` when unset.
    pub current_model: Option<String>,
    /// Depth tier of the model in use, used by tier cascades and cost logic.
    pub model_tier: Option<crate::types::ModelTier>,
    /// Opaque handle of the live LLM stream, for correlating or cancelling it.
    pub llm_handle: Option<String>,
    /// `(input, output)` token counts for the current or last request.
    pub token_usage: Option<(u32, u32)>,
    /// Current request state; `RequestState::Idle` by default.
    pub request_state: RequestState,
    /// CPU load reported for the agent's container; the wire fixes no unit, so
    /// consumers should not assume a percentage scale.
    pub cpu_usage: f64,
    /// Resident memory of the agent's container, in megabytes.
    pub memory_mb: u64,
    /// Retries already spent on the current request; `None` when unreported.
    pub retry_count: Option<u32>,
    /// Retry budget for the current request; `None` when unset.
    pub max_retries: Option<u32>,
    /// Id of the agent that spawned this one; `None` for top-level agents.
    pub parent_id: Option<String>,
}

impl Default for AgentUpdateParams {
    fn default() -> Self {
        Self {
            agent_id: AgentId::system(),
            agent_number: None,
            agent_type: None,
            llm_working: false,
            work_status: None,
            current_model: None,
            model_tier: None,
            llm_handle: None,
            token_usage: None,
            request_state: RequestState::Idle,
            cpu_usage: 0.0,
            memory_mb: 0,
            retry_count: None,
            max_retries: None,
            parent_id: None,
        }
    }
}

impl AgentUpdateParams {
    /// Builds an update for `agent_id` (raw string) carrying only the agent
    /// kind; every other field stays at `Default`.
    pub fn new(agent_id: impl AsRef<str>, agent_type: Agent) -> Self {
        Self {
            agent_id: AgentId::from_raw(agent_id.as_ref()),
            agent_type: Some(agent_type),
            ..Default::default()
        }
    }

    /// Builds an update keyed by a panel id instead of a spawned agent: both
    /// `agent_id` and the badge number come from `panel_id`.
    pub fn for_skill(panel_id: impl AsRef<str>, agent_type: Agent) -> Self {
        let panel_id = AgentId::from_raw(panel_id.as_ref());
        Self {
            agent_number: Some(panel_id.to_string()),
            agent_type: Some(agent_type),
            agent_id: panel_id,
            ..Default::default()
        }
    }

    /// Sets whether an LLM request is currently in flight.
    pub fn llm_working(mut self, working: bool) -> Self {
        self.llm_working = working;
        self
    }

    /// Sets the request state shown by the TUI.
    pub fn request_state(mut self, state: RequestState) -> Self {
        self.request_state = state;
        self
    }

    /// Sets the serving model id, overwriting any previous value.
    pub fn current_model(mut self, model: impl Into<String>) -> Self {
        self.current_model = Some(model.into());
        self
    }

    /// Sets the serving model id from an `Option`, keeping `None` as unset.
    pub fn maybe_current_model(mut self, model: Option<String>) -> Self {
        self.current_model = model;
        self
    }

    /// Sets the fine-grained work phase.
    pub fn work_status(mut self, status: WorkStatus) -> Self {
        self.work_status = Some(status);
        self
    }

    /// Sets the work phase from an `Option`, keeping `None` as unset.
    pub fn maybe_work_status(mut self, status: Option<WorkStatus>) -> Self {
        self.work_status = status;
        self
    }

    /// Sets the token counts from an `Option`, keeping `None` as unreported.
    pub fn maybe_token_usage(mut self, usage: Option<(u32, u32)>) -> Self {
        self.token_usage = usage;
        self
    }

    /// Records how many retries the current request has already spent.
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }

    /// Records the retry budget the current request may spend.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Sets the spawning agent's id from an `Option`, keeping `None` for
    /// top-level agents.
    pub fn maybe_parent_id(mut self, id: Option<String>) -> Self {
        self.parent_id = id;
        self
    }
}

/// One agent list row as the TUI renders it: identity, live status, resource
/// usage and the progress of the current request. Carried by
/// `Sync.AgentListResponse`, `Sync.AgentUpdate` and the snapshot payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiAgentInfo {
    /// Agent kind this row describes.
    pub agent_type: Agent,
    /// Badge number assigned to the agent; `None` for agents without one (e.g.
    /// skill panels).
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    /// Structured `AgentId` of the agent; `None` when only the plain-string
    /// `agent_id` is known.
    #[serde(default)]
    pub agent_uuid: Option<AgentId>,
    /// Plain-string key of this row, used to match list entries and patches.
    pub agent_id: String,
    /// Lifecycle status (`Initializing` / `Online` / `Busy` / `Offline` /
    /// `Error`).
    pub status: AgentStatus,
    /// Whether an LLM request is in flight for the agent.
    pub llm_working: bool,
    /// CPU load reported for the agent's container; the wire fixes no unit, so
    /// consumers should not assume a percentage scale.
    pub cpu_usage: f64,
    /// Resident memory of the agent's container, in megabytes.
    pub memory_mb: u64,
    /// Id of the spawning agent; `None` for top-level agents.
    pub parent_id: Option<String>,
    /// Fine-grained work phase, when the engine reports one.
    #[serde(default)]
    pub work_status: Option<WorkStatus>,
    /// Model currently serving the agent, when known.
    #[serde(default)]
    pub current_model: Option<String>,
    /// Depth tier of the current model, when known.
    #[serde(default)]
    pub model_tier: Option<crate::types::ModelTier>,
    /// Handle of the live LLM stream, when one is open.
    #[serde(default)]
    pub llm_handle: Option<String>,
    /// `(input, output)` token counts for the current or last request; `None`
    /// when unreported.
    #[serde(default)]
    pub token_usage: Option<(u32, u32)>,
    /// Number of tool invocations so far; `0` when the producer omits the
    /// counter.
    #[serde(default)]
    pub tool_calls: u32,
    /// Current request state; `RequestState::Idle` when the producer omits it.
    #[serde(default)]
    pub request_state: RequestState,
    /// How the last finished request ended; `CompletionOutcome::None` when
    /// omitted.
    #[serde(default)]
    pub completion_outcome: CompletionOutcome,
    /// Retries spent on the current request; `0` when omitted.
    #[serde(default)]
    pub retry_count: u32,
    /// Retry budget for the current request; `0` when omitted.
    #[serde(default)]
    pub max_retries: u32,
}
