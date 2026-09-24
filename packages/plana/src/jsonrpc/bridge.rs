//! Bridge between the gateway method names and the JSON-RPC wire form.
//!
//! A fixed gateway method is `{namespace}.{action}` (`Sync.Ping`,
//! `Tool.CallTool`, ...), the namespace being one of the `GatewayMethod`
//! variants and the action the serde `action` tag of the payload enum. The
//! helpers here convert the platform-internal tagged envelope
//! `{"type": "<Namespace>", "data": {"action": "<Action>", ...}}` to and from
//! a JSON-RPC 2.0 request or notification, so a consumer service can keep one
//! dispatch table per namespace instead of hand-building method strings.
//!
//! The method strings are stable wire vocabulary: renaming one breaks every
//! peer at once. Consumer-private methods travel unchanged in the `Extension`
//! variant, which is why parsing never fails (see `UnknownGatewayMethodError`).
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt;

use super::{json_keys::BridgeKey, types::*};

/// One gateway method: a fixed namespace plus the action name inside it.
///
/// The namespace selects the method prefix that `as_str` emits and the `type`
/// field the tagged envelope must carry. This is the entelecheia
/// workspace-sync dialect, not the `lowercase.dotted` service profile
/// (`docs/en/rpc/service-profile.md` §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayMethod {
    /// `Sync.*` — the workspace-sync dialect: client turns and
    /// server-authoritative state (patches, snapshots, reports, notifications).
    Sync(&'static str),
    /// `Base.*` — liveness and transport-level notices (heartbeat, error, ack),
    /// valid in both directions and carrying no domain payload.
    Base(&'static str),
    /// `Agent.*` — agent-registry CRUD on the control plane. No constant and no
    /// `FromStr` arm exist, so the variant is reachable only in process.
    Agent(&'static str),
    /// `Tool.*` — tool catalog and invocation, dispatched to the agent host that
    /// owns the tool.
    Tool(&'static str),
    /// `Skill.*` — skill catalog and invocation; a skill name is the entry point
    /// of a server-side chain.
    Skill(&'static str),
    /// `Node.*` — peer-node discovery and per-node info. Like `Agent` it has no
    /// constant or parse arm and is constructible only in process.
    Node(&'static str),
    /// `Monitor.*` — host and per-agent metrics sampling. Also without constants
    /// or parse arms; `MonitorMessage` supplies the action names.
    Monitor(&'static str),
    /// `Conversation.*` — agent-to-agent consultations about one file: ask,
    /// reply, escalate to a human, resolve.
    Conversation(&'static str),
    /// `Device.*` — polemos hardware terminals: registration, heartbeats, PTY
    /// session control, file transfer, and WebRTC negotiation.
    Device(&'static str),
    /// `Screen.*` — WebRTC screen-stream signalling (SDP offer/answer and ICE
    /// candidates), separate from the `Device.*` control surface.
    Screen(&'static str),
    /// `Cli.*` — operator queries (status, chat history, timeline, search,
    /// listings). Overlaps `Tool.ListTools` and `Sync.OpenWorkspace` by name but
    /// is a distinct method string.
    Cli(&'static str),
    /// `Trigger.*` — hardware trigger events; `Trigger.Event` is the only action
    /// the parser accepts.
    Trigger(&'static str),
    /// `Sensor.*` — batched industrial sensor readings; the parser accepts only
    /// `Sensor.Batch`.
    Sensor(&'static str),
    /// `Discovery.*` — industrial discovery-scan progress; the parser accepts
    /// only `Discovery.Progress`.
    Discovery(&'static str),
    /// `Command.*` — one-shot shell execution on a broker host; `Command.Exec`
    /// runs a single command string and answers with exit code plus stdout and
    /// stderr.
    Command(&'static str),
    /// DLC extension method — arbitrary string for consumer-specific RPCs
    /// (e.g. shittim-chest's auth.*/channels.*/topology.* etc.)
    Extension(String),
}

impl GatewayMethod {
    /// `Sync.Ping` — keepalive the client sends with a `timestamp` (epoch
    /// millis); the peer answers `Sync.Pong`, for which this crate defines no
    /// payload type.
    pub const SYNC_PING: Self = Self::Sync("Ping");
    /// `Sync.AgentPatch` — server push of field-level agent deltas (`patches`),
    /// not whole agent objects; the client upserts each non-null field into its
    /// `state.agents.<agent_id>` viewport.
    pub const SYNC_AGENT_PATCH: Self = Self::Sync("AgentPatch");
    /// `Sync.OrchestrationStatus` — server push of one orchestration `stage`
    /// (`SkillStage`) of a running chain, so a UI can show progress without
    /// polling.
    pub const SYNC_ORCHESTRATION_STATUS: Self = Self::Sync("OrchestrationStatus");
    /// `Sync.ToolResult` — server push reporting one settled tool call
    /// (`tool_name`, `call_id`, `result`, `success`, optional `duration_ms`);
    /// `call_id` is what pairs it with the invocation.
    pub const SYNC_TOOL_RESULT: Self = Self::Sync("ToolResult");
    /// `Sync.AgentStreamingChunk` — server push of one incremental LLM `chunk`
    /// for `agent_id`; `is_done` marks the last chunk of the turn.
    pub const SYNC_AGENT_STREAMING_CHUNK: Self = Self::Sync("AgentStreamingChunk");
    /// `Sync.AgentReport` — server push of a report card (`report_type`, `title`,
    /// `content`, optional `preset_options`); a `query` report is answered by the
    /// client with `Sync.AgentReportReply`.
    pub const SYNC_AGENT_REPORT: Self = Self::Sync("AgentReport");
    /// `Sync.AgentTransfer` — server push announcing a skill hand-off from
    /// `from_skill` to `to_skill` inside one agent. This is the only constant
    /// whose string the parser does not map back, so `"Sync.AgentTransfer"`
    /// round-trips as `Extension` (see the `FromStr` impl).
    pub const SYNC_AGENT_TRANSFER: Self = Self::Sync("AgentTransfer");
    /// `Sync.AskHumanRequest` — server push asking a human to answer `question`
    /// for `consultation_id` from `options`; the client replies
    /// `Sync.AskHumanReply`.
    pub const SYNC_ASK_HUMAN_REQUEST: Self = Self::Sync("AskHumanRequest");
    /// `Sync.UserMessage` — the client-side user turn (`sender_id`, `content`,
    /// `timestamp`); the server mints or reuses the `conversation_id` it carries
    /// and drives the chain from there.
    pub const SYNC_USER_MESSAGE: Self = Self::Sync("UserMessage");
    /// `Sync.AgentResponse` — server push of one finished agent answer
    /// (`agent_type`, `agent_id`, `content`); `parent_id` names what it answers.
    pub const SYNC_AGENT_RESPONSE: Self = Self::Sync("AgentResponse");
    /// `Sync.RequestFullSnapshot` — client request without payload for a
    /// complete state dump; the payload enum has no `FullSnapshot` arm, so only
    /// the per-domain snapshots can answer it.
    pub const SYNC_REQUEST_FULL_SNAPSHOT: Self = Self::Sync("RequestFullSnapshot");
    /// `Sync.RequestGlobalSnapshot` — client request for one frame holding
    /// agents, containers and active tasks; answered by `Sync.GlobalSnapshot`.
    pub const SYNC_REQUEST_GLOBAL_SNAPSHOT: Self = Self::Sync("RequestGlobalSnapshot");
    /// `Sync.GlobalSnapshot` — server push wrapping a `GlobalSnapshot`
    /// (`version`, `timestamp`, `agents`, `containers`, `active_tasks`), the
    /// baseline a client resumes from.
    pub const SYNC_GLOBAL_SNAPSHOT: Self = Self::Sync("GlobalSnapshot");
    /// `Sync.ModelsSnapshot` — server push of the whole model catalog as
    /// `ModelInfo` entries, replacing the client-side model list outright.
    pub const SYNC_MODELS_SNAPSHOT: Self = Self::Sync("ModelsSnapshot");
    /// `Sync.ProvidersSnapshot` — server push of the provider catalog as
    /// `ProviderInfo` entries; it reports `has_api_key` rather than any key.
    pub const SYNC_PROVIDERS_SNAPSHOT: Self = Self::Sync("ProvidersSnapshot");
    /// `Sync.ContainerSnapshot` — server push wrapping a full
    /// `ContainerSnapshot` (`version`, `timestamp`, `containers`); the reply to
    /// `Sync.RequestContainerSnapshot`.
    pub const SYNC_CONTAINER_SNAPSHOT: Self = Self::Sync("ContainerSnapshot");
    /// `Sync.ContainerPatch` — server push of container deltas that the client
    /// upserts into its container viewport, the incremental counterpart of
    /// `Sync.ContainerSnapshot`.
    pub const SYNC_CONTAINER_PATCH: Self = Self::Sync("ContainerPatch");
    /// `Sync.TaskPatch` — server push of task deltas (`status`, `progress`) that
    /// the client upserts into its task viewport.
    pub const SYNC_TASK_PATCH: Self = Self::Sync("TaskPatch");
    /// `Sync.TasksSnapshot` — server push wrapping the full task list; the reply
    /// to `Sync.RequestTasksSnapshot`.
    pub const SYNC_TASKS_SNAPSHOT: Self = Self::Sync("TasksSnapshot");
    /// `Sync.ListAgents` — client request for the agent roster, answered by
    /// `Sync.AgentListResponse`. A client that subscribes to the `state.agents`
    /// viewport no longer needs to send it on reconnect.
    pub const SYNC_LIST_AGENTS: Self = Self::Sync("ListAgents");
    /// `Sync.ServerVersion` — server greeting: the gateway `version` plus a
    /// free-form `build_info` string, so the client can show and compare the
    /// peer build.
    pub const SYNC_SERVER_VERSION: Self = Self::Sync("ServerVersion");
    /// `Sync.OpenWorkspace` — client request to open a workspace `uri`, answered
    /// by `Sync.OpenWorkspaceResponse`. The legacy string
    /// `Sync.OpenGitWorkspace` parses to this same method.
    pub const SYNC_OPEN_WORKSPACE: Self = Self::Sync("OpenWorkspace");
    /// `Sync.WorkspaceStatus` — server push describing the open workspace: id,
    /// display name, connection kind, resolved path, remote url, branch, host id.
    /// Every one of those except the id and the connection kind is optional, so
    /// an absent field means no remote, no checkout branch, no resolved path or
    /// no recorded host.
    pub const SYNC_WORKSPACE_STATUS: Self = Self::Sync("WorkspaceStatus");
    /// `Sync.RequestWorkspaceStatus` — client poll without payload; the server
    /// answers `Sync.WorkspaceStatus`.
    pub const SYNC_REQUEST_WORKSPACE_STATUS: Self = Self::Sync("RequestWorkspaceStatus");
    /// `Sync.SystemMessage` — server push of one `SystemNotification` plus its
    /// `timestamp`; the client renders it (i18n key and params come from the
    /// notification) and takes no protocol action. Published on the
    /// `system_notification` channel topic.
    pub const SYNC_SYSTEM_MESSAGE: Self = Self::Sync("SystemMessage");
    /// `Sync.WebUiControl` — request carrying one `command` for the host that
    /// owns the embedded Web UI; that host answers
    /// `Sync.WebUiControlResponse` echoing the command with its outcome.
    pub const SYNC_WEBUI_CONTROL: Self = Self::Sync("WebUiControl");
    /// `Sync.WebUiControlResponse` — the owning host answer to
    /// `Sync.WebUiControl`: the same `command`, a `success` flag, a human
    /// `message`, and `url` once the Web UI is reachable.
    pub const SYNC_WEBUI_CONTROL_RESPONSE: Self = Self::Sync("WebUiControlResponse");
    /// `Sync.WebUiStatus` — server push of the Web UI state: `running`, its
    /// `url`, and the `container_id` it runs in; an absent optional field means
    /// not running or not known.
    pub const SYNC_WEBUI_STATUS: Self = Self::Sync("WebUiStatus");
    /// `Sync.RequestWebUiStatus` — client poll without payload; the server
    /// answers `Sync.WebUiStatus`.
    pub const SYNC_REQUEST_WEBUI_STATUS: Self = Self::Sync("RequestWebUiStatus");

    /// `Sync.AuthLogin` — client request with `username` and `password`; the
    /// server answers `Sync.AuthLoginResponse`, which carries the session on
    /// success. Credentials travel in params, never in the envelope.
    pub const SYNC_AUTH_LOGIN: Self = Self::Sync("AuthLogin");
    /// `Sync.AuthLoginResponse` — the server answer to `Sync.AuthLogin`: `ok`
    /// plus, on success, `token`, `session_id`, `user_id`, `username`,
    /// `display_name` and `role`; on failure `ok = false`, `error`, no token.
    pub const SYNC_AUTH_LOGIN_RESPONSE: Self = Self::Sync("AuthLoginResponse");
    /// `Sync.AuthRegister` — client request creating an account from `username`,
    /// `password` and an optional `display_name`; answered by
    /// `Sync.AuthRegisterResponse`.
    pub const SYNC_AUTH_REGISTER: Self = Self::Sync("AuthRegister");
    /// `Sync.AuthRegisterResponse` — `ok` with `user_id` and `username` on
    /// success, or `ok = false` with `error`; the password is never echoed.
    pub const SYNC_AUTH_REGISTER_RESPONSE: Self = Self::Sync("AuthRegisterResponse");
    /// `Sync.AuthListUsers` — client request without payload for the user
    /// roster; answered by `Sync.AuthListUsersResponse`.
    pub const SYNC_AUTH_LIST_USERS: Self = Self::Sync("AuthListUsers");
    /// `Sync.AuthListUsersResponse` — `ok` plus `users` (`AuthUserInfo`) or an
    /// `error`; both payload fields are optional, so read `ok` first.
    pub const SYNC_AUTH_LIST_USERS_RESPONSE: Self = Self::Sync("AuthListUsersResponse");
    /// `Sync.AuthGetUser` — client request for the single `user_id`; answered by
    /// `Sync.AuthGetUserResponse`.
    pub const SYNC_AUTH_GET_USER: Self = Self::Sync("AuthGetUser");
    /// `Sync.AuthGetUserResponse` — `ok` with the resolved `user`, or
    /// `ok = false` plus `error` when no such user exists.
    pub const SYNC_AUTH_GET_USER_RESPONSE: Self = Self::Sync("AuthGetUserResponse");
    /// `Sync.AuthDeleteUser` — client request deleting `user_id`; answered by
    /// `Sync.AuthDeleteUserResponse`.
    pub const SYNC_AUTH_DELETE_USER: Self = Self::Sync("AuthDeleteUser");
    /// `Sync.AuthDeleteUserResponse` — `ok` plus an optional `error`; the deleted
    /// user is not echoed back.
    pub const SYNC_AUTH_DELETE_USER_RESPONSE: Self = Self::Sync("AuthDeleteUserResponse");
    /// `Sync.AuthChangePassword` — client request carrying `old_password` so the
    /// server can verify the caller before storing `new_password` for `user_id`.
    pub const SYNC_AUTH_CHANGE_PASSWORD: Self = Self::Sync("AuthChangePassword");
    /// `Sync.AuthChangePasswordResponse` — `ok` plus an optional `error`; neither
    /// password appears in the answer.
    pub const SYNC_AUTH_CHANGE_PASSWORD_RESPONSE: Self = Self::Sync("AuthChangePasswordResponse");

    /// `Base.Heartbeat` — client keepalive notification with no JSON-RPC id,
    /// sent at the client cadence (the `plana-rpc-client` default is 15 s). The
    /// server answers `Base.HeartbeatAck` on the control lane and drops a
    /// connection idle for about three beats.
    pub const BASE_HEARTBEAT: Self = Self::Base("Heartbeat");
    /// `Base.Error` — one-way notice carrying a machine-readable `code` and a
    /// human `message` (`BaseMessage::Error`); declared `OneWay`, so the peer
    /// sends no answer to it.
    pub const BASE_ERROR: Self = Self::Base("Error");
    /// `Base.Ack` — one-way acknowledgement of an earlier message, named by
    /// `message_id` (`BaseMessage::Ack`); distinct from the heartbeat ack
    /// `Base.HeartbeatAck`.
    pub const BASE_ACK: Self = Self::Base("Ack");

    /// `Tool.CallTool` — dispatched request to run `tool_name` with `parameters`
    /// on behalf of `agent_type`. Declared `AsyncReq`, so the result arrives on a
    /// later method rather than on the request id.
    pub const TOOL_CALL: Self = Self::Tool("CallTool");
    /// `Tool.ListTools` — request for the tool catalog of one `agent_type`, or of
    /// every agent when the field is absent; answered by
    /// `Tool.ToolsListResponse`.
    pub const TOOL_LIST_TOOLS: Self = Self::Tool("ListTools");
    /// `Tool.ToolsListResponse` — the answer to `Tool.ListTools`, carrying `tools`
    /// as `ToolInfo` entries (name, description, owning agent, parameter schema,
    /// tier, visibility).
    pub const TOOL_TOOLS_LIST_RESPONSE: Self = Self::Tool("ToolsListResponse");

    /// `Skill.CallSkill` — dispatched request to start `skill_name` with
    /// `parameters` on behalf of `agent_type`; declared `AsyncReq`, so the caller
    /// is answered out of band.
    pub const SKILL_CALL: Self = Self::Skill("CallSkill");
    /// `Skill.ListSkills` — request for the skill catalog of one `agent_type`, or
    /// of every agent when absent; answered by `Skill.SkillsListResponse`.
    pub const SKILL_LIST_SKILLS: Self = Self::Skill("ListSkills");
    /// `Skill.SkillsListResponse` — the answer to `Skill.ListSkills`, carrying
    /// `skills` as `SkillInfo` entries (name, per-language descriptions, owning
    /// agent, required tools).
    pub const SKILL_LIST_SKILLS_RESPONSE: Self = Self::Skill("SkillsListResponse");

    /// Canonical wire method string: `{namespace}.{action}` for the fixed
    /// namespaces, the raw consumer string for `Extension`. Owned rather than
    /// `&'static str` because an extension method string is not static.
    pub fn as_str(&self) -> String {
        match self {
            Self::Sync(action) => format!("Sync.{}", action),
            Self::Base(action) => format!("Base.{}", action),
            Self::Agent(action) => format!("Agent.{}", action),
            Self::Tool(action) => format!("Tool.{}", action),
            Self::Skill(action) => format!("Skill.{}", action),
            Self::Node(action) => format!("Node.{}", action),
            Self::Monitor(action) => format!("Monitor.{}", action),
            Self::Conversation(action) => format!("Conversation.{}", action),
            Self::Device(action) => format!("Device.{}", action),
            Self::Screen(action) => format!("Screen.{}", action),
            Self::Cli(action) => format!("Cli.{}", action),
            Self::Trigger(action) => format!("Trigger.{}", action),
            Self::Sensor(action) => format!("Sensor.{}", action),
            Self::Discovery(action) => format!("Discovery.{}", action),
            Self::Command(action) => format!("Command.{}", action),
            Self::Extension(s) => s.clone(),
        }
    }

    /// Namespace label alone (`Sync`, `Base`, `Tool`, ...), matching the `type`
    /// field of the tagged envelope. For `Extension` this is the literal
    /// `Extension` — a classifier, not a wire prefix, since an extension method
    /// keeps its own string.
    pub fn type_prefix(&self) -> &'static str {
        match self {
            Self::Sync(_) => "Sync",
            Self::Base(_) => "Base",
            Self::Agent(_) => "Agent",
            Self::Tool(_) => "Tool",
            Self::Skill(_) => "Skill",
            Self::Node(_) => "Node",
            Self::Monitor(_) => "Monitor",
            Self::Conversation(_) => "Conversation",
            Self::Device(_) => "Device",
            Self::Screen(_) => "Screen",
            Self::Cli(_) => "Cli",
            Self::Trigger(_) => "Trigger",
            Self::Sensor(_) => "Sensor",
            Self::Discovery(_) => "Discovery",
            Self::Command(_) => "Command",
            Self::Extension(_) => "Extension",
        }
    }

    /// The action segment after the dot, i.e. the method name without its
    /// namespace. For `Extension` it is the whole consumer string, which may
    /// itself be dotted (`auth.login`), so it is not guaranteed to be a bare
    /// action name.
    pub fn action(&self) -> &str {
        match self {
            Self::Sync(a)
            | Self::Base(a)
            | Self::Agent(a)
            | Self::Tool(a)
            | Self::Skill(a)
            | Self::Node(a)
            | Self::Monitor(a)
            | Self::Conversation(a)
            | Self::Device(a)
            | Self::Screen(a)
            | Self::Cli(a)
            | Self::Trigger(a)
            | Self::Sensor(a)
            | Self::Discovery(a)
            | Self::Command(a) => a,
            Self::Extension(s) => s.as_str(),
        }
    }
}

impl fmt::Display for GatewayMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

/// Error type of the `GatewayMethod` `FromStr` impl, holding the rejected
/// input in its tuple field. As written the parser never returns it: an
/// unknown method becomes `Extension` instead, so the type only fills the
/// trait `Err` slot (message: `unknown gateway method: <input>`).
#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown gateway method: {0}")]
pub struct UnknownGatewayMethodError(pub String);

impl std::str::FromStr for GatewayMethod {
    type Err = UnknownGatewayMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sync.Ping" => Ok(Self::SYNC_PING),
            "Sync.AgentPatch" => Ok(Self::SYNC_AGENT_PATCH),
            "Sync.OrchestrationStatus" => Ok(Self::SYNC_ORCHESTRATION_STATUS),
            "Sync.ToolResult" => Ok(Self::SYNC_TOOL_RESULT),
            "Sync.AgentStreamingChunk" => Ok(Self::SYNC_AGENT_STREAMING_CHUNK),
            "Sync.AgentReport" => Ok(Self::SYNC_AGENT_REPORT),
            "Sync.AskHumanRequest" => Ok(Self::SYNC_ASK_HUMAN_REQUEST),
            "Sync.UserMessage" => Ok(Self::SYNC_USER_MESSAGE),
            "Sync.AgentResponse" => Ok(Self::SYNC_AGENT_RESPONSE),
            "Sync.RequestFullSnapshot" => Ok(Self::SYNC_REQUEST_FULL_SNAPSHOT),
            "Sync.RequestGlobalSnapshot" => Ok(Self::SYNC_REQUEST_GLOBAL_SNAPSHOT),
            "Sync.GlobalSnapshot" => Ok(Self::SYNC_GLOBAL_SNAPSHOT),
            "Sync.ModelsSnapshot" => Ok(Self::SYNC_MODELS_SNAPSHOT),
            "Sync.ProvidersSnapshot" => Ok(Self::SYNC_PROVIDERS_SNAPSHOT),
            "Sync.ContainerSnapshot" => Ok(Self::SYNC_CONTAINER_SNAPSHOT),
            "Sync.ContainerPatch" => Ok(Self::SYNC_CONTAINER_PATCH),
            "Sync.TaskPatch" => Ok(Self::SYNC_TASK_PATCH),
            "Sync.TasksSnapshot" => Ok(Self::SYNC_TASKS_SNAPSHOT),
            "Sync.ListAgents" => Ok(Self::SYNC_LIST_AGENTS),
            "Sync.ServerVersion" => Ok(Self::SYNC_SERVER_VERSION),
            "Sync.OpenGitWorkspace" | "Sync.OpenWorkspace" => Ok(Self::SYNC_OPEN_WORKSPACE),
            "Sync.WorkspaceStatus" => Ok(Self::SYNC_WORKSPACE_STATUS),
            "Sync.RequestWorkspaceStatus" => Ok(Self::SYNC_REQUEST_WORKSPACE_STATUS),
            "Sync.SystemMessage" => Ok(Self::SYNC_SYSTEM_MESSAGE),
            "Sync.WebUiControl" => Ok(Self::SYNC_WEBUI_CONTROL),
            "Sync.WebUiControlResponse" => Ok(Self::SYNC_WEBUI_CONTROL_RESPONSE),
            "Sync.WebUiStatus" => Ok(Self::SYNC_WEBUI_STATUS),
            "Sync.RequestWebUiStatus" => Ok(Self::SYNC_REQUEST_WEBUI_STATUS),
            "Sync.AuthLogin" => Ok(Self::SYNC_AUTH_LOGIN),
            "Sync.AuthLoginResponse" => Ok(Self::SYNC_AUTH_LOGIN_RESPONSE),
            "Sync.AuthRegister" => Ok(Self::SYNC_AUTH_REGISTER),
            "Sync.AuthRegisterResponse" => Ok(Self::SYNC_AUTH_REGISTER_RESPONSE),
            "Sync.AuthListUsers" => Ok(Self::SYNC_AUTH_LIST_USERS),
            "Sync.AuthListUsersResponse" => Ok(Self::SYNC_AUTH_LIST_USERS_RESPONSE),
            "Sync.AuthGetUser" => Ok(Self::SYNC_AUTH_GET_USER),
            "Sync.AuthGetUserResponse" => Ok(Self::SYNC_AUTH_GET_USER_RESPONSE),
            "Sync.AuthDeleteUser" => Ok(Self::SYNC_AUTH_DELETE_USER),
            "Sync.AuthDeleteUserResponse" => Ok(Self::SYNC_AUTH_DELETE_USER_RESPONSE),
            "Sync.AuthChangePassword" => Ok(Self::SYNC_AUTH_CHANGE_PASSWORD),
            "Sync.AuthChangePasswordResponse" => Ok(Self::SYNC_AUTH_CHANGE_PASSWORD_RESPONSE),
            "Base.Heartbeat" => Ok(Self::BASE_HEARTBEAT),
            "Base.Error" => Ok(Self::BASE_ERROR),
            "Base.Ack" => Ok(Self::BASE_ACK),
            "Tool.CallTool" => Ok(Self::TOOL_CALL),
            "Tool.ListTools" => Ok(Self::TOOL_LIST_TOOLS),
            "Tool.ToolsListResponse" => Ok(Self::TOOL_TOOLS_LIST_RESPONSE),
            "Skill.CallSkill" => Ok(Self::SKILL_CALL),
            "Skill.ListSkills" => Ok(Self::SKILL_LIST_SKILLS),
            "Skill.SkillsListResponse" => Ok(Self::SKILL_LIST_SKILLS_RESPONSE),
            // ── Device / Screen (hardware terminal + WebRTC) ──
            "Device.PolemosRegister" => Ok(Self::Device("PolemosRegister")),
            "Device.Heartbeat" => Ok(Self::Device("Heartbeat")),
            "Device.TerminalOpen" => Ok(Self::Device("TerminalOpen")),
            "Device.TerminalInput" => Ok(Self::Device("TerminalInput")),
            "Device.TerminalResize" => Ok(Self::Device("TerminalResize")),
            "Device.TerminalPoll" => Ok(Self::Device("TerminalPoll")),
            "Device.TerminalClose" => Ok(Self::Device("TerminalClose")),
            "Device.TerminalList" => Ok(Self::Device("TerminalList")),
            "Device.SubscribeOutput" => Ok(Self::Device("SubscribeOutput")),
            "Device.FileList" => Ok(Self::Device("FileList")),
            "Device.FileDownload" => Ok(Self::Device("FileDownload")),
            "Device.FileUpload" => Ok(Self::Device("FileUpload")),
            "Device.Ping" => Ok(Self::Device("Ping")),
            "Device.WebrtcOffer" => Ok(Self::Device("WebrtcOffer")),
            "Device.WebrtcIce" => Ok(Self::Device("WebrtcIce")),
            "Screen.Offer" => Ok(Self::Screen("Offer")),
            "Screen.Answer" => Ok(Self::Screen("Answer")),
            "Screen.Ice" => Ok(Self::Screen("Ice")),
            "Screen.IceCandidate" => Ok(Self::Screen("IceCandidate")),
            // ── Cli query methods ──
            "Cli.Status" => Ok(Self::Cli("Status")),
            "Cli.ChatHistory" => Ok(Self::Cli("ChatHistory")),
            "Cli.TimelineList" => Ok(Self::Cli("TimelineList")),
            "Cli.TimelineShow" => Ok(Self::Cli("TimelineShow")),
            "Cli.RecentChats" => Ok(Self::Cli("RecentChats")),
            "Cli.SessionStats" => Ok(Self::Cli("SessionStats")),
            "Cli.SessionPurge" => Ok(Self::Cli("SessionPurge")),
            "Cli.SessionVacuum" => Ok(Self::Cli("SessionVacuum")),
            "Cli.Search" => Ok(Self::Cli("Search")),
            "Cli.TraceChain" => Ok(Self::Cli("TraceChain")),
            "Cli.ListPolemosDevices" => Ok(Self::Cli("ListPolemosDevices")),
            "Cli.ListTools" => Ok(Self::Cli("ListTools")),
            "Cli.ListSkills" => Ok(Self::Cli("ListSkills")),
            "Cli.ListWorkspaces" => Ok(Self::Cli("ListWorkspaces")),
            "Cli.OpenWorkspace" => Ok(Self::Cli("OpenWorkspace")),
            "Cli.SwitchWorkspace" => Ok(Self::Cli("SwitchWorkspace")),
            // ── Hardware trigger/sensor/discovery/command ──
            "Trigger.Event" => Ok(Self::Trigger("Event")),
            "Sensor.Batch" => Ok(Self::Sensor("Batch")),
            "Discovery.Progress" => Ok(Self::Discovery("Progress")),
            "Command.Exec" => Ok(Self::Command("Exec")),
            // ── Conversation internal messages ──
            "Conversation.AskAgent" => Ok(Self::Conversation("AskAgent")),
            "Conversation.ReplyAgent" => Ok(Self::Conversation("ReplyAgent")),
            "Conversation.Escalated" => Ok(Self::Conversation("Escalated")),
            "Conversation.Resolved" => Ok(Self::Conversation("Resolved")),
            // ── Extension fallback: unknown methods become Extension ──
            other => Ok(Self::Extension(other.to_string())),
        }
    }
}

/// Serialize a tagged gateway envelope and split it into a JSON-RPC method
/// string and params: `method = "{type}.{action}"`, params = the `data`
/// object minus its `action` key.
///
/// `params` is `None` when the action carries no other field, which keeps a
/// unit action free of an empty object. An envelope whose `data` is not an
/// object, or which serializes to a non-object, yields the fallback pair
/// `("Unknown.Unknown", Some(json))` rather than an error.
pub fn core_message_to_method_and_params<T: Serialize>(msg: &T) -> (String, Option<Value>) {
    let json = serde_json::to_value(msg).unwrap_or(Value::Null);

    match json {
        Value::Object(ref map) => {
            let type_name = map
                .get(BridgeKey::Type.as_ref())
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let data = map
                .get(BridgeKey::Data.as_ref())
                .cloned()
                .unwrap_or(Value::Null);

            if let Value::Object(data_map) = data {
                let action = data_map
                    .get(BridgeKey::Action.as_ref())
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let wire_prefix = type_name;
                let method = format!("{}.{}", wire_prefix, action);

                let params: serde_json::Map<String, Value> = data_map
                    .into_iter()
                    .filter(|(k, _)| k != BridgeKey::Action.as_ref())
                    .collect();

                let params_value = if params.is_empty() {
                    None
                } else {
                    Some(Value::Object(params))
                };

                return (method, params_value);
            }

            ("Unknown.Unknown".to_string(), Some(json))
        }
        _ => ("Unknown.Unknown".to_string(), Some(json)),
    }
}

/// Inverse of `core_message_to_method_and_params`: split `method` at its
/// first dot and deserialize the rebuilt `{type, data}` envelope into `T`,
/// with the second segment restored as the `action` field.
///
/// For `Tool` and `Skill` methods it fills `agent_type` (`"SkoPeo"`) and an
/// empty `parameters` object when the caller omitted them, because those
/// payloads require both. Returns `None` when the value does not fit `T`, so
/// a shape mismatch is indistinguishable from a missing method.
pub fn from_jsonrpc_method<T: DeserializeOwned>(method: &str, params: Option<Value>) -> Option<T> {
    let (wire_prefix, action) = method.split_once('.')?;
    let type_name = wire_prefix;

    let mut data = match params {
        Some(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    data.insert(
        BridgeKey::Action.as_ref().to_string(),
        Value::String(action.to_string()),
    );

    // Ensure required fields for Tool/Skill messages are present
    if type_name == "Tool" {
        if !data.contains_key("agent_type") {
            data.insert("agent_type".into(), Value::String("SkoPeo".into()));
        }
        if !data.contains_key("parameters") {
            data.insert("parameters".into(), Value::Object(serde_json::Map::new()));
        }
    }
    if type_name == "Skill" {
        if !data.contains_key("agent_type") {
            data.insert("agent_type".into(), Value::String("SkoPeo".into()));
        }
        if !data.contains_key("parameters") {
            data.insert("parameters".into(), Value::Object(serde_json::Map::new()));
        }
    }

    let mut reconstructed_map = serde_json::Map::new();
    reconstructed_map.insert(
        BridgeKey::Type.as_ref().to_string(),
        Value::String(type_name.to_string()),
    );
    reconstructed_map.insert(BridgeKey::Data.as_ref().to_string(), Value::Object(data));
    let reconstructed = Value::Object(reconstructed_map);

    serde_json::from_value::<T>(reconstructed).ok()
}

/// Serialize a gateway message to a JSON-RPC frame: a notification without an
/// `id` when `is_notification`, otherwise a request carrying a freshly minted
/// UUID `id`. The caller cannot choose that id, so a request built here is
/// correlated by the transport rather than by the caller.
pub fn serialize_to_jsonrpc<T: Serialize>(
    msg: &T,
    is_notification: bool,
) -> Result<String, serde_json::Error> {
    if is_notification {
        let (method, params) = core_message_to_method_and_params(msg);
        let notif = JsonRpcNotification::new_raw(method, params);
        serde_json::to_string(&notif)
    } else {
        let (method, params) = core_message_to_method_and_params(msg);
        let req = JsonRpcRequest::new_raw(method, params);
        serde_json::to_string(&req)
    }
}

/// Parse a JSON-RPC frame into the gateway type `T`.
///
/// A frame that is neither request nor notification (a response) yields
/// `Ok(None)`, and so does a payload that fails to deserialize — the caller
/// cannot tell the two apart. A JSON syntax error yields a `JsonRpcError`
/// with code `-32700` and the parser message appended.
pub fn deserialize_from_jsonrpc<T: DeserializeOwned>(
    json: &str,
) -> Result<Option<T>, JsonRpcError> {
    let rpc_msg: JsonRpcMessage = serde_json::from_str(json)
        .map_err(|e| JsonRpcError::new(-32700, format!("Parse error: {}", e)))?;

    match rpc_msg {
        JsonRpcMessage::Request(req) => Ok(from_jsonrpc_method(&req.method, req.params)),
        JsonRpcMessage::Notification(notif) => Ok(from_jsonrpc_method(&notif.method, notif.params)),
        JsonRpcMessage::Response(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use serde::{Deserialize, Serialize};

    // Minimal stand-in for the gateway message envelope: mirrors the wire
    // shape (`{"type": "...", "data": {"action": "...", ...}}`) so the bridge
    // roundtrips can be exercised against a concrete typed message without
    // depending on any host platform's message catalog.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "data")]
    enum TestMessage {
        Sync(SyncMessage),
        Base(BaseMessage),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "action")]
    enum SyncMessage {
        Ping {
            timestamp: u64,
        },
        OpenWorkspace {
            uri: String,
        },
        WorkspaceStatus {
            workspace_id: uuid::Uuid,
            display_name: Option<String>,
            connection_kind: String,
            resolved_path: Option<String>,
            remote_url: Option<String>,
            branch: Option<String>,
            host_id: Option<String>,
        },
        SystemMessage {
            notification: SystemNotification,
            timestamp: String,
        },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "action")]
    enum BaseMessage {
        Heartbeat { timestamp: i64 },
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum SystemNotification {
        WebUiStarted,
    }

    #[test]
    fn roundtrip_user_message() -> anyhow::Result<()> {
        let msg = TestMessage::Sync(SyncMessage::Ping {
            timestamp: 12345u64,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_PING.as_str());

        let reconstructed: Option<TestMessage> =
            from_jsonrpc_method(&method, params).context("failed to reconstruct Ping message")?;
        match reconstructed {
            Some(TestMessage::Sync(SyncMessage::Ping { timestamp })) => {
                assert_eq!(timestamp, 12345u64);
            }
            other => anyhow::bail!("Expected Ping, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_heartbeat() -> anyhow::Result<()> {
        let msg = TestMessage::Base(BaseMessage::Heartbeat { timestamp: 999 });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::BASE_HEARTBEAT.as_str());

        let reconstructed: Option<TestMessage> = from_jsonrpc_method(&method, params)
            .context("failed to reconstruct Heartbeat message")?;
        match reconstructed {
            Some(TestMessage::Base(BaseMessage::Heartbeat { timestamp })) => {
                assert_eq!(timestamp, 999);
            }
            other => anyhow::bail!("Expected Heartbeat, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_open_workspace() -> anyhow::Result<()> {
        let msg = TestMessage::Sync(SyncMessage::OpenWorkspace {
            uri: "git://https://github.com/org/repo.git".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_OPEN_WORKSPACE.as_str());

        let reconstructed: Option<TestMessage> =
            from_jsonrpc_method(&method, params).context("failed to reconstruct OpenWorkspace")?;
        match reconstructed {
            Some(TestMessage::Sync(SyncMessage::OpenWorkspace { uri })) => {
                assert_eq!(uri, "git://https://github.com/org/repo.git");
            }
            other => anyhow::bail!("Expected OpenWorkspace, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_workspace_status() -> anyhow::Result<()> {
        let test_ws_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000123")
            .context("test precondition")?;
        let msg = TestMessage::Sync(SyncMessage::WorkspaceStatus {
            workspace_id: test_ws_id,
            display_name: Some("my-repo".to_string()),
            connection_kind: "git".to_string(),
            resolved_path: Some("/tmp/workspaces/ws-123".to_string()),
            remote_url: Some("https://github.com/org/repo.git".to_string()),
            branch: Some("main".to_string()),
            host_id: None,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_WORKSPACE_STATUS.as_str());

        let reconstructed: Option<TestMessage> = from_jsonrpc_method(&method, params)
            .context("failed to reconstruct WorkspaceStatus")?;
        match reconstructed {
            Some(TestMessage::Sync(SyncMessage::WorkspaceStatus {
                workspace_id,
                connection_kind,
                remote_url,
                ..
            })) => {
                assert_eq!(workspace_id, test_ws_id);
                assert_eq!(connection_kind, "git");
                assert_eq!(
                    remote_url,
                    Some("https://github.com/org/repo.git".to_string())
                );
            }
            other => anyhow::bail!("Expected WorkspaceStatus, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_open_workspace() -> anyhow::Result<()> {
        let method: GatewayMethod = "Sync.OpenWorkspace".parse()?;
        assert_eq!(method.as_str(), GatewayMethod::SYNC_OPEN_WORKSPACE.as_str());
        Ok(())
    }

    #[test]
    fn gateway_method_parse_workspace_status() -> anyhow::Result<()> {
        let method: GatewayMethod = "Sync.WorkspaceStatus".parse()?;
        assert_eq!(
            method.as_str(),
            GatewayMethod::SYNC_WORKSPACE_STATUS.as_str()
        );
        Ok(())
    }

    #[test]
    fn roundtrip_system_message() -> anyhow::Result<()> {
        let msg = TestMessage::Sync(SyncMessage::SystemMessage {
            notification: SystemNotification::WebUiStarted,
            timestamp: "2026-05-11T12:00:00Z".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_SYSTEM_MESSAGE.as_str());
        let reconstructed: Option<TestMessage> =
            from_jsonrpc_method(&method, params).context("system message roundtrip")?;
        match reconstructed {
            Some(TestMessage::Sync(SyncMessage::SystemMessage {
                notification,
                timestamp,
            })) => {
                assert_eq!(notification, SystemNotification::WebUiStarted);
                assert_eq!(timestamp, "2026-05-11T12:00:00Z");
            }
            other => anyhow::bail!("Expected SystemMessage, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_methods() -> anyhow::Result<()> {
        for (s, expected) in [
            ("Sync.SystemMessage", GatewayMethod::SYNC_SYSTEM_MESSAGE),
            ("Sync.AuthLogin", GatewayMethod::SYNC_AUTH_LOGIN),
            (
                "Sync.AuthLoginResponse",
                GatewayMethod::SYNC_AUTH_LOGIN_RESPONSE,
            ),
        ] {
            let method: GatewayMethod = s.parse()?;
            assert_eq!(method.as_str(), expected.as_str());
        }
        Ok(())
    }
}
