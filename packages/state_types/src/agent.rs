//! Agent descriptors and the per-instance agent state that travels with them.
//!
//! `Agent` is the state-layer mirror of `plana_domain_agent::AgentKind` and
//! delegates every string form (folder, friendly name, description, layer) to
//! that registry. `AgentInfo` and the register/unregister requests are the
//! registration protocol, `AgentStatus` and `WorkStatus` are the two status
//! axes (process lifecycle versus current turn), and `AgentCategory` /
//! `CustomAgentId` carry the identity extras for multi-instance and custom
//! agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::tools::{SkillInfo, ToolInfo};
use plana_core::{AgentBadge, AgentId};
use plana_domain_agent::AgentKind;

/// Id of a user-defined (custom) agent, as opposed to the built-in `Agent`
/// kinds. Newtype over the name string: `Display`, `From<String>` and serde's
/// newtype form all pass that string through unchanged, while `Ord`/`Hash` let
/// it key sets and maps of custom agents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CustomAgentId(pub String);

impl CustomAgentId {
    /// Wrap an owned or borrowed name into an id.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The id as a string slice, without allocating.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CustomAgentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CustomAgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Distinguishes a single-instance tool agent from a multi-instance one that
/// needs a badge. Sent in `AgentRegisterRequest`, where the registry reads it to
/// decide whether to allocate an instance number. `Display` renders
/// `simple_tool`, or `complex_tool#<badge>` / `complex_tool#<nnn>` /
/// `complex_tool#pending`; serde writes the variant names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    /// The agent is a single tool: one registration, no instance number, no badge.
    SimpleTool,
    /// The agent may run as several instances, each identified by a badge;
    /// serialized as an object carrying the two fields below.
    ComplexTool {
        /// Instance slot as a plain integer. It has no `serde(default)`, so the key has
        /// to be sent explicitly (`null` while the registry has not assigned one).
        agent_number: Option<u16>,
        /// Badge the caller chose up front, e.g. for a pre-planned instance;
        /// `serde(default)` = `None`, meaning the registry assigns one.
        #[serde(default)]
        preassigned_badge: Option<String>,
    },
}

impl Display for AgentCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCategory::SimpleTool => write!(f, "simple_tool"),
            AgentCategory::ComplexTool {
                agent_number,
                preassigned_badge,
            } => {
                if let Some(badge) = preassigned_badge {
                    write!(f, "complex_tool#{}", badge)
                } else {
                    match agent_number {
                        Some(n) => write!(f, "complex_tool#{:03}", n),
                        None => write!(f, "complex_tool#pending"),
                    }
                }
            }
        }
    }
}

macro_rules! agent_variants {
    ($($name:ident),* $(,)?) => {
        /// Agent kind as seen by the state and wire layers: one variant per entry of
        /// `plana_domain_agent::AgentKind`, in the same order, with total conversions in
        /// both directions.
        ///
        /// Serializes as the bare variant identifier (`HapLotes`, ...), and
        /// `Deserialize` accepts only that spelling; `FromStr` is looser and also takes
        /// the folder name. `Display` yields the folder name.
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
        pub enum Agent {
            $($name,)*
        }

        impl Agent {
            /// Convert from the canonical `AgentKind`; total, so no failure case.
            pub fn from_kind(kind: AgentKind) -> Self {
                match kind {
                    $(AgentKind::$name => Agent::$name,)*
                }
            }

            /// Convert back into `AgentKind`, consuming the value.
            pub fn into_kind(self) -> AgentKind {
                match self {
                    $(Agent::$name => AgentKind::$name,)*
                }
            }

            /// Convert back into `AgentKind` by copy (the kind is `Copy`), leaving the value
            /// usable.
            pub fn as_kind(&self) -> AgentKind {
                match self {
                    $(Agent::$name => AgentKind::$name,)*
                }
            }

            /// Lower-case folder name used for prompt and doc paths (e.g. `web_automation`).
            pub fn folder_name(&self) -> &'static str {
                self.as_kind().folder_name()
            }

            /// Human-readable name taken from the agent registry (e.g. `Web Automation`).
            pub fn friendly_name(&self) -> &'static str {
                self.as_kind().friendly_name()
            }

            /// One-line description of the agent's role, taken from the agent registry.
            pub fn description(&self) -> &'static str {
                self.as_kind().description()
            }

            /// Every variant in declaration order: the Layer 1 agents first, then the
            /// Layer 2 ones.
            pub fn all() -> Vec<Self> {
                vec![$(Agent::$name,)*]
            }
        }

        impl Display for Agent {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_kind().folder_name())
            }
        }

        impl std::str::FromStr for Agent {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                AgentKind::from_str(s)
                    .map(Agent::from_kind)
                    .map_err(|_| format!("unknown agent: {}", s))
            }
        }

        impl From<AgentKind> for Agent {
            fn from(kind: AgentKind) -> Self {
                Agent::from_kind(kind)
            }
        }

        impl From<Agent> for AgentKind {
            fn from(agent: Agent) -> Self {
                agent.into_kind()
            }
        }
    };
}

agent_variants!(
    HapLotes,
    SkoPeo,
    HubRis,
    KaLos,
    NeiKos,
    SkeMma,
    ApoRia,
    EleOs,
    Epieikeia,
    OreXis,
    PhiLia,
    PoleMos,
    WebAutomation,
    ClassicSoftwareEngineering,
    DigitalTwin,
    DataGrid,
    MediaFlow,
    IndustrialIoT,
    RemoteOperations,
    PlatformAdmin,
);

impl Agent {
    /// Human-readable name for UI use; the same string as `friendly_name`.
    pub fn display_name(&self) -> &str {
        self.friendly_name()
    }

    /// Name to show for one instance: the badge parsed out of `agent_id` when it
    /// carries one, otherwise the plain friendly name.
    pub fn formatted_name(&self, agent_id: &str) -> String {
        AgentBadge::new(agent_id)
            .map(|b| b.to_string())
            .unwrap_or_else(|| self.friendly_name().to_string())
    }

    /// Whether the kind is one of the Layer 2 specialist agents, as recorded by the
    /// registry (`layer == 2`).
    pub fn is_layer2(&self) -> bool {
        self.as_kind().is_layer2()
    }
}

/// Process-level status of one agent instance, reported to the control plane and
/// mirrored into the TUI snapshot. Serializes as the variant identifier
/// (`Online`, ...). Distinct from `WorkStatus`, which describes what a live agent
/// is doing inside its current turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// The process is starting up and cannot serve work yet.
    Initializing,
    /// Registered, idle and ready to accept work.
    Online,
    /// Registered and currently occupied.
    Busy,
    /// Not reachable: heartbeats stopped or the process shut down.
    Offline,
    /// The agent reported a failure of its own runtime.
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Initializing => write!(f, "Initializing"),
            AgentStatus::Online => write!(f, "Online"),
            AgentStatus::Busy => write!(f, "Busy"),
            AgentStatus::Offline => write!(f, "Offline"),
            AgentStatus::Error => write!(f, "Error"),
        }
    }
}

/// What an agent is doing inside its current turn, pushed to the TUI and the
/// control plane (`TuiAgentInfo::work_status`). Serializes as the variant
/// identifier, so unit variants are bare strings while the two payload variants
/// become objects (`{"Executing":{"skill_name":...}}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStatus {
    /// Waiting for the model's next output.
    Thinking,
    /// Receiving streamed model output.
    StreamingResponse,
    /// Running a skill; carries its name so the UI can label the turn.
    Executing { skill_name: String },
    /// Retrying the model call; carries the attempt number and its ceiling.
    Retrying { retry_count: u32, max_retries: u32 },
    /// Prompting the agent again because it has not reported yet.
    Nudging,
    /// The turn ended with a report.
    Completed,
    /// The model request itself failed, at transport or provider level.
    RequestFailed,
    /// The turn failed after the request went through.
    Failed,
    /// The tool loop reached its iteration limit and was stopped.
    ToolLoopTerminated,
    /// A tool call is in flight.
    CallingTool,
}

impl std::fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkStatus::Thinking => write!(f, "Thinking"),
            WorkStatus::StreamingResponse => write!(f, "Streaming response"),
            WorkStatus::Executing { .. } => write!(f, "Executing"),
            WorkStatus::Retrying {
                retry_count,
                max_retries,
            } => {
                write!(f, "Retrying ({}/{})", retry_count, max_retries)
            }
            WorkStatus::Nudging => write!(f, "Nudging for report"),
            WorkStatus::Completed => write!(f, "Completed"),
            WorkStatus::RequestFailed => write!(f, "Request Failed"),
            WorkStatus::Failed => write!(f, "Failed"),
            WorkStatus::ToolLoopTerminated => write!(f, "Tool loop terminated"),
            WorkStatus::CallingTool => write!(f, "Calling tool"),
        }
    }
}

impl WorkStatus {
    /// Short label for the TUI: the skill name while `Executing`, `agent::tool`
    /// while `CallingTool` (falling back to the agent name when no tool name is
    /// given), and the agent name for every other status.
    pub fn display_tag(&self, agent_type: &Agent, tool_name: Option<&str>) -> String {
        match self {
            WorkStatus::Executing { skill_name } => skill_name.clone(),
            WorkStatus::CallingTool => tool_name
                .map(|t| format!("{}::{}", agent_type, t))
                .unwrap_or_else(|| agent_type.to_string()),
            _ => agent_type.to_string(),
        }
    }
}

/// Snapshot of one registered agent instance: identity, status, liveness
/// timestamps, the host it runs on, and the tools and skills it advertises.
/// `AgentRegisterRequest` carries the same identity minus the runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Which agent kind this instance is.
    pub agent_type: Agent,
    /// Instance id. `AgentId` is a transparent string newtype, normally formatted as
    /// `<kind>-<uuidv7>`.
    pub agent_id: AgentId,
    /// Badge assigned to this instance of a complex agent; `serde(default)` = `None`
    /// for simple agents or before assignment.
    #[serde(default)]
    pub agent_number: Option<AgentBadge>,
    /// Lifecycle status; see `AgentStatus`.
    pub status: AgentStatus,
    /// When the agent process started, in UTC.
    pub started_at: DateTime<Utc>,
    /// Timestamp of the latest heartbeat, in UTC; judging staleness is the reader's
    /// job, not this type's.
    pub last_heartbeat: DateTime<Utc>,
    /// Host name the agent reported; `serde(default)` = `None` when unknown.
    #[serde(default)]
    pub hostname: Option<String>,
    /// Address the agent is reachable at; `serde(default)` = `None`.
    #[serde(default)]
    pub ip_address: Option<String>,
    /// Port the agent's own endpoint listens on; `serde(default)` = `None`.
    #[serde(default)]
    pub agent_port: Option<u16>,
    /// Platform label such as `linux` or `windows`; `serde(default)` = `None`.
    #[serde(default)]
    pub platform: Option<String>,
    /// Platform version string; `serde(default)` = `None`.
    #[serde(default)]
    pub platform_version: Option<String>,
    /// Tools this instance advertises; may be empty.
    pub tools: Vec<ToolInfo>,
    /// Skills this instance advertises; may be empty.
    pub skills: Vec<SkillInfo>,
    /// Agent that spawned this instance, for child agents; `serde(default)` = `None`
    /// for top-level ones.
    #[serde(default)]
    pub parent_agent_id: Option<AgentId>,
}

/// Registration payload an agent sends when it comes online: identity, category
/// and the capabilities it offers. Identity, category, tools and skills are
/// required keys; the host and parent fields are `serde(default)` and may be
/// omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegisterRequest {
    /// Agent kind being registered.
    pub agent_type: Agent,
    /// Instance id chosen by the agent or by its supervisor.
    pub agent_id: AgentId,
    /// Whether the instance is a single tool or one instance of a complex agent.
    pub category: AgentCategory,
    /// Host name the agent runs on; `serde(default)` = `None`.
    #[serde(default)]
    pub hostname: Option<String>,
    /// Address the agent is reachable at; `serde(default)` = `None`.
    #[serde(default)]
    pub ip_address: Option<String>,
    /// Port the agent's endpoint listens on; `serde(default)` = `None`.
    #[serde(default)]
    pub agent_port: Option<u16>,
    /// Platform label; `serde(default)` = `None`.
    #[serde(default)]
    pub platform: Option<String>,
    /// Platform version string; `serde(default)` = `None`.
    #[serde(default)]
    pub platform_version: Option<String>,
    /// Tools the agent offers once it is registered; may be empty.
    pub tools: Vec<ToolInfo>,
    /// Skills the agent can run once it is registered; may be empty.
    pub skills: Vec<SkillInfo>,
    /// Spawning agent's id when this instance is a child; `serde(default)` = `None`.
    #[serde(default)]
    pub parent_agent_id: Option<AgentId>,
}

/// Deregistration payload: names the instance that is going away, so the control
/// plane can drop it and, for complex agents, release its badge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUnregisterRequest {
    /// Agent kind of the instance being removed.
    pub agent_type: Agent,
    /// Instance id being removed.
    pub agent_id: AgentId,
}
