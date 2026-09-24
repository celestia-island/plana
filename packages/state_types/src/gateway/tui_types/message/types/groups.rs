//! Domain-grouped view of [`super::SyncMessage`] variants.
//!
//! This module provides categorized documentation and re-exports of
//! `SyncMessage` variant *names* (as constants) so that developers can
//! quickly discover which variants belong to which functional domain.
//!
//! The full enum remains in [`super::SyncMessage`]; nothing here changes
//! the public API. This is the first step toward eventual full
//! decomposition of the 150-variant enum into separate types.

/// String constants for connection / handshake variants.
///
/// Covers the initial wire protocol: ping, version negotiation, and the
/// token-based handshake.
pub mod protocol {
    /// `Sync.Ping` - client -> server keepalive request carrying `timestamp` (epoch millis).
    /// Answered by the payload-free `Sync.Pong` variant, which has no constant here.
    pub const PING: &str = "Ping";
    /// `Sync.ServerVersion` - server -> client greeting: the gateway's `version` string plus a
    /// `build_info` string, sent once so the client can surface and compare the peer build.
    pub const SERVER_VERSION: &str = "ServerVersion";
    /// `Sync.ConnectHandshake` - client -> server; presents the auth `token`, an optional
    /// `session_id`, the declared `capabilities` / `node_info` / `workspace_id`, and
    /// `client_type` (cli or tui). Answered by `HandshakeAck`; a cli client makes the
    /// server skip the PubSub bridge.
    pub const CONNECT_HANDSHAKE: &str = "ConnectHandshake";
    /// `Sync.HandshakeAck` - server -> client reply to `ConnectHandshake`: `ok`, optional `error`,
    /// the assigned `session_id`, and a `reconnect` flag marking a resumed session.
    pub const HANDSHAKE_ACK: &str = "HandshakeAck";
    /// `Sync.VersionMismatch` - server -> client handshake rejection carrying both
    /// `server_version` and `client_version`, so only the gateway, which knows both, can
    /// raise it.
    pub const VERSION_MISMATCH: &str = "VersionMismatch";

    /// The 5 `protocol` names as one slice, in declaration order.
    ///
    /// This slice is the whole initial handshake vocabulary.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        PING,
        SERVER_VERSION,
        CONNECT_HANDSHAKE,
        HANDSHAKE_ACK,
        VERSION_MISMATCH,
    ];
}

/// String constants for Layer-2 / custom-agent marketplace variants.
///
/// These deal with discovering, subscribing to, and querying external
/// (Layer-2) agents, their tools, skills, and prompt templates.
pub mod layer2 {
    /// `Sync.Layer2AgentList` - client -> server request (`AsyncReq`) for the installed Layer-2
    /// agents; answered by `Layer2AgentListResponse`.
    pub const LAYER2_AGENT_LIST: &str = "Layer2AgentList";
    /// `Sync.Layer2AgentListResponse` - server -> client answer carrying `agents`, each a Layer-2
    /// agent with name, description, tool/skill counts, languages and an enabled flag.
    pub const LAYER2_AGENT_LIST_RESPONSE: &str = "Layer2AgentListResponse";
    /// `Sync.Layer2AgentToolTools` - client -> server; asks which tools the Layer-2 agent
    /// `agent_name` exposes. Answered by `Layer2AgentToolResponse`.
    pub const LAYER2_AGENT_TOOL_TOOLS: &str = "Layer2AgentToolTools";
    /// `Sync.Layer2AgentToolResponse` - server -> client; echoes `agent_name` and returns `tools`
    /// (`Layer2ToolInfo` descriptors) for the client's tool picker.
    pub const LAYER2_AGENT_TOOL_RESPONSE: &str = "Layer2AgentToolResponse";
    /// `Sync.Layer2AgentSkills` - client -> server; asks for the skills of Layer-2 agent
    /// `agent_name`. Answered by `Layer2AgentSkillsResponse`.
    pub const LAYER2_AGENT_SKILLS: &str = "Layer2AgentSkills";
    /// `Sync.Layer2AgentSkillsResponse` - server -> client; echoes `agent_name` and returns
    /// `skills` (`Layer2SkillInfo` descriptors).
    pub const LAYER2_AGENT_SKILLS_RESPONSE: &str = "Layer2AgentSkillsResponse";
    /// `Sync.Layer2AgentToolPrompt` - client -> server; requests the prompt body of one `tool` of
    /// `agent_name`, optionally in `lang`. Answered by `Layer2AgentToolPromptResponse`.
    pub const LAYER2_AGENT_TOOL_PROMPT: &str = "Layer2AgentToolPrompt";
    /// `Sync.Layer2AgentToolPromptResponse` - server -> client; returns `agent_name`, `tool`, the
    /// resolved `lang`, the prompt `content`, and its display `name`.
    pub const LAYER2_AGENT_TOOL_PROMPT_RESPONSE: &str = "Layer2AgentToolPromptResponse";
    /// `Sync.Layer2AgentSkillPrompt` - client -> server; same request shape for one `skill` of
    /// `agent_name` (optional `lang`). Answered by `Layer2AgentSkillPromptResponse`.
    pub const LAYER2_AGENT_SKILL_PROMPT: &str = "Layer2AgentSkillPrompt";
    /// `Sync.Layer2AgentSkillPromptResponse` - server -> client; returns `agent_name`, `skill`,
    /// `lang`, the prompt `content`, and its display `name`.
    pub const LAYER2_AGENT_SKILL_PROMPT_RESPONSE: &str = "Layer2AgentSkillPromptResponse";
    /// `Sync.CustomAgentList` - client -> server request for the custom agents this client has
    /// subscribed to; answered by `CustomAgentListResponse`.
    pub const CUSTOM_AGENT_LIST: &str = "CustomAgentList";
    /// `Sync.CustomAgentListResponse` - server -> client; `agents` as `CustomAgentInfo` entries
    /// (name, display name, description, skill count, source, version, last update).
    pub const CUSTOM_AGENT_LIST_RESPONSE: &str = "CustomAgentListResponse";
    /// `Sync.SubscribeCustomAgent` - client -> server; installs a custom agent from `source` plus
    /// optional `repository` / `url`. Answered by `SubscribeCustomAgentResponse`.
    pub const SUBSCRIBE_CUSTOM_AGENT: &str = "SubscribeCustomAgent";
    /// `Sync.SubscribeCustomAgentResponse` - server -> client; `success`, optional `error`, the
    /// installed `agent`, and the granted `skills` / `permissions` lists.
    pub const SUBSCRIBE_CUSTOM_AGENT_RESPONSE: &str = "SubscribeCustomAgentResponse";
    /// `Sync.UnsubscribeCustomAgent` - client -> server; removes the custom agent named `name`.
    /// Answered by `UnsubscribeCustomAgentResponse`.
    pub const UNSUBSCRIBE_CUSTOM_AGENT: &str = "UnsubscribeCustomAgent";
    /// `Sync.UnsubscribeCustomAgentResponse` - server -> client; `success` plus optional `error`
    /// when the named agent was not subscribed.
    pub const UNSUBSCRIBE_CUSTOM_AGENT_RESPONSE: &str = "UnsubscribeCustomAgentResponse";

    /// The 16 `layer2` names as one slice, in declaration order.
    ///
    /// It mixes the Layer-2 request/reply pairs with the custom-agent ones.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        LAYER2_AGENT_LIST,
        LAYER2_AGENT_LIST_RESPONSE,
        LAYER2_AGENT_TOOL_TOOLS,
        LAYER2_AGENT_TOOL_RESPONSE,
        LAYER2_AGENT_SKILLS,
        LAYER2_AGENT_SKILLS_RESPONSE,
        LAYER2_AGENT_TOOL_PROMPT,
        LAYER2_AGENT_TOOL_PROMPT_RESPONSE,
        LAYER2_AGENT_SKILL_PROMPT,
        LAYER2_AGENT_SKILL_PROMPT_RESPONSE,
        CUSTOM_AGENT_LIST,
        CUSTOM_AGENT_LIST_RESPONSE,
        SUBSCRIBE_CUSTOM_AGENT,
        SUBSCRIBE_CUSTOM_AGENT_RESPONSE,
        UNSUBSCRIBE_CUSTOM_AGENT,
        UNSUBSCRIBE_CUSTOM_AGENT_RESPONSE,
    ];
}

/// String constants for core agent lifecycle variants.
///
/// User ↔ agent messaging, streaming, reporting, orchestration status,
/// tool results, human-in-the-loop reviews/consultations, and
/// agent CRUD.
pub mod agent {
    /// `Sync.UserMessage` - client -> server; one user turn: `sender_id`, `content`, `timestamp`
    /// (string, or epoch millis as a number on the wire), optional `language`, `images`,
    /// `workspace_id`, `conversation_id` (topic chaining) and `actor` (RBAC claims).
    pub const USER_MESSAGE: &str = "UserMessage";
    /// `Sync.AgentResponse` - server -> client; a finished agent answer carrying `agent_type`,
    /// `agent_id`, optional `agent_number` badge, `content`, `timestamp` and `parent_id`.
    pub const AGENT_RESPONSE: &str = "AgentResponse";
    /// `Sync.AgentStreamingChunk` - server -> client; one incremental `chunk` for `agent_id` with
    /// `is_done` and an optional `chunk_kind` (Text / Thinking / DeepThinking).
    pub const AGENT_STREAMING_CHUNK: &str = "AgentStreamingChunk";
    /// `Sync.AgentReport` - server -> client; the report card: `report_type`, `title`, `content`,
    /// `preset_options` (with selection mode and custom-reply flags), plus routing, stream, usage
    /// and error metadata. A query report expects an `AgentReportReply` back from the client.
    pub const AGENT_REPORT: &str = "AgentReport";
    /// `Sync.AgentTransfer` - server -> client; announces a skill hand-off `from_skill` to
    /// `to_skill` for `agent_id`, with optional summary, model name and token usage.
    pub const AGENT_TRANSFER: &str = "AgentTransfer";
    /// `Sync.OrchestrationStatus` - server -> client; one orchestration `stage` (`SkillStage`) for
    /// `agent`, with optional `tool_name`, `call_id`, `parent_agent` and `parameters_summary`.
    pub const ORCHESTRATION_STATUS: &str = "OrchestrationStatus";
    /// `Sync.ToolResult` - server -> client; the outcome of one tool call: `tool_name`, `call_id`,
    /// `result`, `success`, optional `parameters_summary` and `duration_ms`.
    pub const TOOL_RESULT: &str = "ToolResult";
    /// `Sync.StreamingTail` - server -> client one-way carry of the trailing `tail` text for
    /// `agent_id`, distinct from the incremental `AgentStreamingChunk` deltas.
    pub const STREAMING_TAIL: &str = "StreamingTail";
    /// `Sync.HumanReviewRequest` - server -> client; asks the operator to decide on `review_id`
    /// (title, content, timestamp, originating agent). Answered by `HumanReviewResponse`.
    pub const HUMAN_REVIEW_REQUEST: &str = "HumanReviewRequest";
    /// `Sync.HumanReviewResponse` - client -> server; the operator's `choice` and free-form
    /// `comment` for a previously received `review_id`.
    pub const HUMAN_REVIEW_RESPONSE: &str = "HumanReviewResponse";
    /// `Sync.AskHumanRequest` - server -> client; consultation `consultation_id` carrying the
    /// `question` and `question_localized` text, candidate `options`, and an optional
    /// `recommended` pick.
    pub const ASK_HUMAN_REQUEST: &str = "AskHumanRequest";
    /// `Sync.AskHumanReply` - client -> server (`AsyncReq`); the answer to a consultation:
    /// `selected_options`, optional `custom_answer`, and `answered_by` naming the answer source.
    pub const ASK_HUMAN_REPLY: &str = "AskHumanReply";
    /// `Sync.AutoModeUpdate` - server -> client auto-mode state (`enabled`, optional
    /// `timeout_secs`), the same payload as `SystemNotification::AutoModeChanged`.
    pub const AUTO_MODE_UPDATE: &str = "AutoModeUpdate";
    /// `Sync.ScepterIdentity` - server -> client identity announcement carrying the gateway's own
    /// `device_id`, so a connected client can tell which scepter instance it reached.
    pub const SCEPTER_IDENTITY: &str = "ScepterIdentity";
    /// `Sync.ListAgents` - client -> server (`SyncReq`) roster request; answered by
    /// `AgentListResponse` (or superseded by the `state.agents` viewport).
    pub const LIST_AGENTS: &str = "ListAgents";
    /// `Sync.AgentListResponse` - server -> client; `agents` as `TuiAgentInfo` entries, the full
    /// roster that the `state.agents` viewport upserts keyed by `agent_id`.
    pub const AGENT_LIST_RESPONSE: &str = "AgentListResponse";
    /// `Sync.AgentUpdate` - server -> client one-way upsert of a single `TuiAgentInfo`; the
    /// `state.agents` domain merges it by `agent_id` instead of reloading the whole roster.
    pub const AGENT_UPDATE: &str = "AgentUpdate";
    /// `Sync.RetryAgentRequest` - client -> server; asks `agent_id` to run again, carrying the
    /// `attempt` counter the server uses to bound retries.
    pub const RETRY_AGENT_REQUEST: &str = "RetryAgentRequest";
    /// `Sync.UndoRequest` - client -> server with no payload; asks the server to undo the last
    /// agent turn of the session.
    pub const UNDO_REQUEST: &str = "UndoRequest";
    /// `Sync.ReloadAllAgentConfigs` - client -> server with no payload; asks the server to re-read
    /// every agent config from disk.
    pub const RELOAD_ALL_AGENT_CONFIGS: &str = "ReloadAllAgentConfigs";
    /// `Sync.ReloadAgentConfig` - client -> server; re-reads the config of the single agent named
    /// `agent_name`, leaving the other agents untouched.
    pub const RELOAD_AGENT_CONFIG: &str = "ReloadAgentConfig";

    /// `Sync.AgentReportReply` - client -> server answer to an `AgentReport` whose
    /// `report_type` is Query; `report_id` mirrors the original `agent_id`, and
    /// `selected_options` / `custom_answer` carry the human's choice.
    pub const AGENT_REPORT_REPLY: &str = "AgentReportReply";
    /// `Sync.AgentThinkingStep` - server -> client; one `ThinkingStepEntry` (id, content, status,
    /// timestamp) appended to an agent's thinking timeline.
    pub const AGENT_THINKING_STEP: &str = "AgentThinkingStep";
    /// `Sync.AgentToolCall` - server -> client; a tool invocation by `agent_id`: `tool`, optional
    /// `params` and `result`, and a `status` string.
    pub const AGENT_TOOL_CALL: &str = "AgentToolCall";
    /// The 24 `agent` names as one slice, in declaration order.
    ///
    /// The last three entries are the late additions `AgentReportReply`,
    /// `AgentThinkingStep` and `AgentToolCall`.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        USER_MESSAGE,
        AGENT_RESPONSE,
        AGENT_STREAMING_CHUNK,
        AGENT_REPORT,
        AGENT_TRANSFER,
        ORCHESTRATION_STATUS,
        TOOL_RESULT,
        STREAMING_TAIL,
        HUMAN_REVIEW_REQUEST,
        HUMAN_REVIEW_RESPONSE,
        ASK_HUMAN_REQUEST,
        ASK_HUMAN_REPLY,
        AUTO_MODE_UPDATE,
        SCEPTER_IDENTITY,
        LIST_AGENTS,
        AGENT_LIST_RESPONSE,
        AGENT_UPDATE,
        RETRY_AGENT_REQUEST,
        UNDO_REQUEST,
        RELOAD_ALL_AGENT_CONFIGS,
        RELOAD_AGENT_CONFIG,
        AGENT_REPORT_REPLY,
        AGENT_THINKING_STEP,
        AGENT_TOOL_CALL,
    ];
}

/// String constants for task / issue management variants.
pub mod task {
    /// `Sync.TaskCreated` - server -> client broadcast; a new task (`task_id`, `issue_id`,
    /// `title`, optional description, assigned agent, parent task, badge, conversation and
    /// degree estimate).
    pub const TASK_CREATED: &str = "TaskCreated";
    /// `Sync.TaskStatusUpdate` - server -> client broadcast; moves `task_id` to `status`
    /// (`crate::TaskStatus`) with a `progress` byte.
    pub const TASK_STATUS_UPDATE: &str = "TaskStatusUpdate";

    /// The 2 `task` names as one slice, in declaration order.
    ///
    /// Only two names: `TaskEstimateUpdated` is in the enum but has no constant here.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[TASK_CREATED, TASK_STATUS_UPDATE];
}

/// String constants for LLM provider configuration variants.
///
/// CRUD operations on providers, model-level configuration, endpoint
/// validation, and usage-period accounting.
pub mod llm {
    /// `Sync.ConfigureLlmProvider` - client -> server; registers a provider from `provider_name`,
    /// `api_key`, optional `api_endpoint`, a `default_model` and a `provider_type`.
    pub const CONFIGURE_LLM_PROVIDER: &str = "ConfigureLlmProvider";
    /// `Sync.LlmProviderConfigured` - server -> client reply carrying `provider_name`, `success`
    /// and an optional `error`.
    pub const LLM_PROVIDER_CONFIGURED: &str = "LlmProviderConfigured";
    /// `Sync.RenameProvider` - client -> server; sets the display name of `provider_name` to
    /// `new_display_name`. Answered by `ProviderRenamed`.
    pub const RENAME_PROVIDER: &str = "RenameProvider";
    /// `Sync.ProviderRenamed` - server -> client reply: `provider_name`, `new_display_name`,
    /// `success` and an optional `error`.
    pub const PROVIDER_RENAMED: &str = "ProviderRenamed";
    /// `Sync.EditProvider` - client -> server; patches the stored `api_key` and/or `api_endpoint`
    /// of `provider_name` (both optional, omitted fields stay unchanged). Answered by
    /// `ProviderEdited`.
    pub const EDIT_PROVIDER: &str = "EditProvider";
    /// `Sync.ProviderEdited` - server -> client reply: `provider_name`, `success`, and an
    /// optional `error`.
    pub const PROVIDER_EDITED: &str = "ProviderEdited";
    /// `Sync.DeleteProvider` - client -> server; removes `provider_name` and its stored config.
    /// Answered by `ProviderDeleted`.
    pub const DELETE_PROVIDER: &str = "DeleteProvider";
    /// `Sync.ProviderDeleted` - server -> client reply: `provider_name`, `success`, and an
    /// optional `error`.
    pub const PROVIDER_DELETED: &str = "ProviderDeleted";
    /// `Sync.ListConfiguredProviders` - client -> server request; answered by
    /// `ConfiguredProvidersList`.
    pub const LIST_CONFIGURED_PROVIDERS: &str = "ListConfiguredProviders";
    /// `Sync.ConfiguredProvidersList` - server -> client; `providers` as `ConfiguredProvider`
    /// entries for the provider settings page.
    pub const CONFIGURED_PROVIDERS_LIST: &str = "ConfiguredProvidersList";
    /// `Sync.UpdateModelProviderConfig` - client -> server; one model's config knobs:
    /// display name, endpoint url, model id, context window, compression threshold, usage type,
    /// input / cache / output prices, period limits, and the image / audio / video /
    /// reasoning capability flags.
    pub const UPDATE_MODEL_PROVIDER_CONFIG: &str = "UpdateModelProviderConfig";
    /// `Sync.ModelProviderConfigUpdated` - server -> client reply: `provider_name`, `success`,
    /// optional `error`.
    pub const MODEL_PROVIDER_CONFIG_UPDATED: &str = "ModelProviderConfigUpdated";
    /// `Sync.ValidateEndpoint` - client -> server; probes `api_endpoint` for `provider_name`,
    /// optionally with an `api_key`, speaking `protocol` (`GenProtocol`).
    pub const VALIDATE_ENDPOINT: &str = "ValidateEndpoint";
    /// `Sync.EndpointValidated` - server -> client probe result: `provider_name`, `is_reachable`,
    /// optional `latency_ms` and `error`.
    pub const ENDPOINT_VALIDATED: &str = "EndpointValidated";
    /// `Sync.UsagePeriodQuery` - client -> server (`SyncReq`); asks for usage of `user_id` (all
    /// users when absent) over the given `period_types`. Answered by `UsagePeriodResponse`.
    pub const USAGE_PERIOD_QUERY: &str = "UsagePeriodQuery";
    /// `Sync.UsagePeriodResponse` - server -> client answer to `UsagePeriodQuery`, carrying `data`
    /// as `UsagePeriodData` entries.
    pub const USAGE_PERIOD_RESPONSE: &str = "UsagePeriodResponse";
    /// `Sync.UsagePeriodUpdate` - server -> client push of refreshed `data` (`UsagePeriodData`),
    /// so an open usage view updates without re-querying.
    pub const USAGE_PERIOD_UPDATE: &str = "UsagePeriodUpdate";

    /// The 17 `llm` names as one slice, in declaration order.
    ///
    /// Provider, model-config and usage requests alternate with their replies.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        CONFIGURE_LLM_PROVIDER,
        LLM_PROVIDER_CONFIGURED,
        RENAME_PROVIDER,
        PROVIDER_RENAMED,
        EDIT_PROVIDER,
        PROVIDER_EDITED,
        DELETE_PROVIDER,
        PROVIDER_DELETED,
        LIST_CONFIGURED_PROVIDERS,
        CONFIGURED_PROVIDERS_LIST,
        UPDATE_MODEL_PROVIDER_CONFIG,
        MODEL_PROVIDER_CONFIG_UPDATED,
        VALIDATE_ENDPOINT,
        ENDPOINT_VALIDATED,
        USAGE_PERIOD_QUERY,
        USAGE_PERIOD_RESPONSE,
        USAGE_PERIOD_UPDATE,
    ];
}

/// String constants for state-sync / snapshot variants.
///
/// Full snapshots, incremental patches, and VM introspection for agents,
/// containers, tasks, and global state.
pub mod snapshot {
    /// `Sync.RequestFullSnapshot` - client -> server (`SyncReq`) asks for a full state dump; the
    /// enum has no `FullSnapshot` arm, so only the per-domain snapshot variants can answer it.
    pub const REQUEST_FULL_SNAPSHOT: &str = "RequestFullSnapshot";
    /// `Sync.AgentPatch` - server -> client one-way; `patches` (`AgentPatch` entries) are
    /// field-level agent deltas, typically produced by entelecheia, applied keyed by `agent_id`.
    pub const AGENT_PATCH: &str = "AgentPatch";
    /// `Sync.AgentSnapshot` - server -> client one-way; wraps a full `AgentSnapshot` that a
    /// receiver takes as the new baseline instead of merging an `AgentPatch`.
    pub const AGENT_SNAPSHOT: &str = "AgentSnapshot";
    /// `Sync.RequestGlobalSnapshot` - client -> server (`SyncReq`); answered by `GlobalSnapshot`
    /// (agents, containers and active tasks in one frame).
    pub const REQUEST_GLOBAL_SNAPSHOT: &str = "RequestGlobalSnapshot";
    /// `Sync.GlobalSnapshot` - server -> client one-way; wraps a `GlobalSnapshot` with version,
    /// timestamp, agents, containers and active tasks.
    pub const GLOBAL_SNAPSHOT: &str = "GlobalSnapshot";
    /// `Sync.ModelsSnapshot` - server -> client one-way; `models` as `ModelInfo` entries backing
    /// the model picker.
    pub const MODELS_SNAPSHOT: &str = "ModelsSnapshot";
    /// `Sync.ProvidersSnapshot` - server -> client one-way; `providers` as `ProviderInfo` entries.
    pub const PROVIDERS_SNAPSHOT: &str = "ProvidersSnapshot";
    /// `Sync.ContainerPatch` - server -> client one-way; `patches` (`ContainerPatch` entries) are
    /// container deltas that the state tree upserts.
    pub const CONTAINER_PATCH: &str = "ContainerPatch";
    /// `Sync.RequestContainerSnapshot` - client -> server (`SyncReq`); answered by
    /// `ContainerSnapshot`.
    pub const REQUEST_CONTAINER_SNAPSHOT: &str = "RequestContainerSnapshot";
    /// `Sync.ContainerSnapshot` - server -> client one-way; wraps a `ContainerSnapshot` with
    /// version, timestamp and containers.
    pub const CONTAINER_SNAPSHOT: &str = "ContainerSnapshot";
    /// `Sync.TaskPatch` - server -> client one-way; `patches` (`TaskPatch` entries) are
    /// task deltas.
    pub const TASK_PATCH: &str = "TaskPatch";
    /// `Sync.RequestTasksSnapshot` - client -> server (`SyncReq`); answered by `TasksSnapshot`.
    pub const REQUEST_TASKS_SNAPSHOT: &str = "RequestTasksSnapshot";
    /// `Sync.TasksSnapshot` - server -> client one-way; wraps a `TasksSnapshot` with version,
    /// timestamp and tasks.
    pub const TASKS_SNAPSHOT: &str = "TasksSnapshot";
    /// `Sync.RequestVmSnapshot` - client -> server (`SyncReq`) introspection request for one
    /// agent's VM, identified by `agent_type` plus `agent_id`. Answered by `VmSnapshot`.
    pub const REQUEST_VM_SNAPSHOT: &str = "RequestVmSnapshot";
    /// `Sync.VmSnapshot` - server -> client one-way; one agent's VM view: `globals`, optional
    /// `container_info`, `tool_list`, and the `op_log` (`CosmosOperationLogEntry` entries).
    pub const VM_SNAPSHOT: &str = "VmSnapshot";

    /// The 15 `snapshot` names as one slice, in declaration order.
    ///
    /// It interleaves requests, patches and snapshots per state domain.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        REQUEST_FULL_SNAPSHOT,
        AGENT_PATCH,
        AGENT_SNAPSHOT,
        REQUEST_GLOBAL_SNAPSHOT,
        GLOBAL_SNAPSHOT,
        MODELS_SNAPSHOT,
        PROVIDERS_SNAPSHOT,
        CONTAINER_PATCH,
        REQUEST_CONTAINER_SNAPSHOT,
        CONTAINER_SNAPSHOT,
        TASK_PATCH,
        REQUEST_TASKS_SNAPSHOT,
        TASKS_SNAPSHOT,
        REQUEST_VM_SNAPSHOT,
        VM_SNAPSHOT,
    ];
}

/// String constants for config-filesystem variants.
///
/// Reading/writing provider, model, user, and API-key configuration
/// from the on-disk config store.
pub mod config_fs {
    /// `Sync.GetProvidersFromFs` - client -> server (`SyncReq`); reads provider definitions
    /// straight from the on-disk config store. Answered by `ProvidersFromFsResponse`.
    pub const GET_PROVIDERS_FROM_FS: &str = "GetProvidersFromFs";
    /// `Sync.ProvidersFromFsResponse` - server -> client; `providers` as `ProviderFsInfo` entries
    /// read from disk.
    pub const PROVIDERS_FROM_FS_RESPONSE: &str = "ProvidersFromFsResponse";
    /// `Sync.GetModelsFromFs` - client -> server (`SyncReq`); reads model definitions from disk.
    /// Answered by `ModelsFromFsResponse`.
    pub const GET_MODELS_FROM_FS: &str = "GetModelsFromFs";
    /// `Sync.ModelsFromFsResponse` - server -> client; `models` as `ModelFsInfo` entries.
    pub const MODELS_FROM_FS_RESPONSE: &str = "ModelsFromFsResponse";
    /// `Sync.ReloadProviderConfig` - client -> server; tells the config store to re-read the
    /// provider identified by `provider_id`.
    pub const RELOAD_PROVIDER_CONFIG: &str = "ReloadProviderConfig";
    /// `Sync.ReloadModelConfig` - client -> server; re-reads one model (`provider_id` plus
    /// `model_id`) from the config store.
    pub const RELOAD_MODEL_CONFIG: &str = "ReloadModelConfig";
    /// `Sync.GetUserConfig` - client -> server (`SyncReq`); answered by `UserConfigResponse`.
    pub const GET_USER_CONFIG: &str = "GetUserConfig";
    /// `Sync.UserConfigResponse` - server -> client; wraps the stored `UserInfo`.
    pub const USER_CONFIG_RESPONSE: &str = "UserConfigResponse";
    /// `Sync.UpdateUserConfig` - client -> server; writes the supplied `UserInfo` back to the
    /// config store.
    pub const UPDATE_USER_CONFIG: &str = "UpdateUserConfig";
    /// `Sync.ReloadUserConfig` - client -> server with no payload; re-reads the user config from
    /// disk.
    pub const RELOAD_USER_CONFIG: &str = "ReloadUserConfig";
    /// `Sync.ListKeys` - client -> server request for the stored API keys; answered by
    /// `KeysListResponse`.
    pub const LIST_KEYS: &str = "ListKeys";
    /// `Sync.KeysListResponse` - server -> client; `keys` as `KeyInfo` entries.
    pub const KEYS_LIST_RESPONSE: &str = "KeysListResponse";
    /// `Sync.SaveApiKey` - client -> server; stores `api_key` under `provider`, with optional
    /// `metadata` (`KeyMetadata`).
    pub const SAVE_API_KEY: &str = "SaveApiKey";
    /// `Sync.DeleteApiKey` - client -> server; removes the stored key of `provider`.
    pub const DELETE_API_KEY: &str = "DeleteApiKey";
    /// `Sync.GetApiKeyInfo` - client -> server; asks for the stored key metadata of `provider`.
    /// Answered by `ApiKeyInfoResponse`.
    pub const GET_API_KEY_INFO: &str = "GetApiKeyInfo";
    /// `Sync.ApiKeyInfoResponse` - server -> client; wraps `info` (`KeyInfo`).
    pub const API_KEY_INFO_RESPONSE: &str = "ApiKeyInfoResponse";

    /// The 16 `config_fs` names as one slice, in declaration order.
    ///
    /// Disk-read requests alternate with their `*Response` replies.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        GET_PROVIDERS_FROM_FS,
        PROVIDERS_FROM_FS_RESPONSE,
        GET_MODELS_FROM_FS,
        MODELS_FROM_FS_RESPONSE,
        RELOAD_PROVIDER_CONFIG,
        RELOAD_MODEL_CONFIG,
        GET_USER_CONFIG,
        USER_CONFIG_RESPONSE,
        UPDATE_USER_CONFIG,
        RELOAD_USER_CONFIG,
        LIST_KEYS,
        KEYS_LIST_RESPONSE,
        SAVE_API_KEY,
        DELETE_API_KEY,
        GET_API_KEY_INFO,
        API_KEY_INFO_RESPONSE,
    ];
}

/// String constants for knowledge-base variants.
///
/// CRUD for knowledge bases, documents, and subscriptions.
pub mod knowledge_base {
    /// `Sync.CreateKnowledgeBase` - client -> server; carries a `CreateKnowledgeBaseRequest`.
    /// Answered by `CreateKnowledgeBaseResponse`.
    pub const CREATE_KNOWLEDGE_BASE: &str = "CreateKnowledgeBase";
    /// `Sync.CreateKnowledgeBaseResponse` - server -> client; wraps a
    /// `CreateKnowledgeBaseResponse` payload for the new base.
    pub const CREATE_KNOWLEDGE_BASE_RESPONSE: &str = "CreateKnowledgeBaseResponse";
    /// `Sync.AddDocument` - client -> server; adds a document to a base via `AddDocumentRequest`.
    /// Answered by `AddDocumentResponse`.
    pub const ADD_DOCUMENT: &str = "AddDocument";
    /// `Sync.AddDocumentResponse` - server -> client; wraps an `AddDocumentResponse` payload.
    pub const ADD_DOCUMENT_RESPONSE: &str = "AddDocumentResponse";
    /// `Sync.QueryKnowledgeBase` - client -> server; runs a `QueryKnowledgeBaseRequest` against a
    /// base. Answered by `QueryKnowledgeBaseResponse`.
    pub const QUERY_KNOWLEDGE_BASE: &str = "QueryKnowledgeBase";
    /// `Sync.QueryKnowledgeBaseResponse` - server -> client; wraps a `QueryKnowledgeBaseResponse`.
    pub const QUERY_KNOWLEDGE_BASE_RESPONSE: &str = "QueryKnowledgeBaseResponse";
    /// `Sync.CreateSubscription` - client -> server; subscribes a base using a
    /// `CreateSubscriptionRequest`. Answered by `CreateSubscriptionResponse`.
    pub const CREATE_SUBSCRIPTION: &str = "CreateSubscription";
    /// `Sync.CreateSubscriptionResponse` - server -> client; wraps a `CreateSubscriptionResponse`.
    pub const CREATE_SUBSCRIPTION_RESPONSE: &str = "CreateSubscriptionResponse";
    /// `Sync.SyncSubscription` - client -> server; re-syncs the subscription described by a
    /// `SyncSubscriptionRequest`. Answered by `SyncSubscriptionResponse`.
    pub const SYNC_SUBSCRIPTION: &str = "SyncSubscription";
    /// `Sync.SyncSubscriptionResponse` - server -> client; wraps a `SyncSubscriptionResponse`.
    pub const SYNC_SUBSCRIPTION_RESPONSE: &str = "SyncSubscriptionResponse";
    /// `Sync.DeleteSubscription` - client -> server; drops one subscription by `subscription_id`.
    /// Answered by `DeleteSubscriptionResponse`.
    pub const DELETE_SUBSCRIPTION: &str = "DeleteSubscription";
    /// `Sync.DeleteSubscriptionResponse` - server -> client; wraps a `DeleteSubscriptionResponse`.
    pub const DELETE_SUBSCRIPTION_RESPONSE: &str = "DeleteSubscriptionResponse";
    /// `Sync.GetKnowledgeBase` - client -> server; fetches `knowledge_base_id`; the reply carries
    /// `knowledge_base: Option<KnowledgeBaseInfo>`.
    pub const GET_KNOWLEDGE_BASE: &str = "GetKnowledgeBase";
    /// `Sync.GetKnowledgeBaseResponse` - server -> client; `knowledge_base` as an optional
    /// `KnowledgeBaseInfo` for the requested id.
    pub const GET_KNOWLEDGE_BASE_RESPONSE: &str = "GetKnowledgeBaseResponse";
    /// `Sync.ListKnowledgeBases` - client -> server; lists bases matching the optional `filters`
    /// (`KnowledgeBaseFilters`). Answered by `ListKnowledgeBasesResponse`.
    pub const LIST_KNOWLEDGE_BASES: &str = "ListKnowledgeBases";
    /// `Sync.ListKnowledgeBasesResponse` - server -> client; `knowledge_bases` as
    /// `KnowledgeBaseInfo` entries.
    pub const LIST_KNOWLEDGE_BASES_RESPONSE: &str = "ListKnowledgeBasesResponse";
    /// `Sync.DeleteKnowledgeBase` - client -> server; deletes `knowledge_base_id`. Answered by
    /// `DeleteKnowledgeBaseResponse`.
    pub const DELETE_KNOWLEDGE_BASE: &str = "DeleteKnowledgeBase";
    /// `Sync.DeleteKnowledgeBaseResponse` - server -> client; wraps a
    /// `DeleteKnowledgeBaseResponse`.
    pub const DELETE_KNOWLEDGE_BASE_RESPONSE: &str = "DeleteKnowledgeBaseResponse";

    /// The 18 `knowledge_base` names as one slice, in declaration order.
    ///
    /// Nine request/reply pairs in CRUD order.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        CREATE_KNOWLEDGE_BASE,
        CREATE_KNOWLEDGE_BASE_RESPONSE,
        ADD_DOCUMENT,
        ADD_DOCUMENT_RESPONSE,
        QUERY_KNOWLEDGE_BASE,
        QUERY_KNOWLEDGE_BASE_RESPONSE,
        CREATE_SUBSCRIPTION,
        CREATE_SUBSCRIPTION_RESPONSE,
        SYNC_SUBSCRIPTION,
        SYNC_SUBSCRIPTION_RESPONSE,
        DELETE_SUBSCRIPTION,
        DELETE_SUBSCRIPTION_RESPONSE,
        GET_KNOWLEDGE_BASE,
        GET_KNOWLEDGE_BASE_RESPONSE,
        LIST_KNOWLEDGE_BASES,
        LIST_KNOWLEDGE_BASES_RESPONSE,
        DELETE_KNOWLEDGE_BASE,
        DELETE_KNOWLEDGE_BASE_RESPONSE,
    ];
}

/// String constants for workspace management variants.
///
/// Opening workspaces via URI and querying workspace status.
pub mod workspace {
    /// `Sync.OpenWorkspace` - client -> server (`SyncReq`); opens the workspace at `uri`.
    /// Answered by `OpenWorkspaceResponse`.
    pub const OPEN_WORKSPACE: &str = "OpenWorkspace";
    /// `Sync.WorkspaceStatus` - server -> client answer to `RequestWorkspaceStatus`:
    /// `workspace_id`, display name, `connection_kind`, resolved path, remote url, branch,
    /// and host id.
    pub const WORKSPACE_STATUS: &str = "WorkspaceStatus";
    /// `Sync.RequestWorkspaceStatus` - client -> server (`SyncReq`); asks for the active
    /// workspace's status. Answered by `WorkspaceStatus`.
    pub const REQUEST_WORKSPACE_STATUS: &str = "RequestWorkspaceStatus";

    /// `Sync.OpenWorkspaceResponse` - server -> client; `success`, optional `workspace_id` and
    /// optional `error`.
    pub const OPEN_WORKSPACE_RESPONSE: &str = "OpenWorkspaceResponse";
    /// `Sync.ListPolemosDevices` - client -> server (`SyncReq`) request for the Polemos device
    /// roster; the devices themselves arrive as the separate `PolemosDeviceList` notification.
    pub const LIST_POLEMOS_DEVICES: &str = "ListPolemosDevices";
    /// `Sync.PolemosDeviceList` - server -> client one-way roster push produced by scepter;
    /// consumers upsert each device into `state.devices.<node_id>`.
    pub const POLEMOS_DEVICE_LIST: &str = "PolemosDeviceList";
    /// `Sync.RegisterPolemosDevice` - client -> server (`SyncReq`); registers a device by
    /// `host_id` and `address`, with an optional `workspace_path`.
    pub const REGISTER_POLEMOS_DEVICE: &str = "RegisterPolemosDevice";
    /// `Sync.RegisterPolemosDeviceResponse` - server -> client; `success`, optional `error`, and
    /// the registered `device` (`PolemosDeviceInfo`).
    pub const REGISTER_POLEMOS_DEVICE_RESPONSE: &str = "RegisterPolemosDeviceResponse";
    /// `Sync.SwitchWorkspace` - client -> server; switches the active workspace to `workspace_id`.
    /// Answered by `SwitchWorkspaceResponse`.
    pub const SWITCH_WORKSPACE: &str = "SwitchWorkspace";
    /// `Sync.SwitchWorkspaceResponse` - server -> client; `success`, the `workspace_id`
    /// switched to, and an optional `error`.
    pub const SWITCH_WORKSPACE_RESPONSE: &str = "SwitchWorkspaceResponse";
    /// `Sync.SetClientCwd` - client -> server; reports the client's current directory `path`,
    /// optionally scoped by `device_id` and `device_type`.
    pub const SET_CLIENT_CWD: &str = "SetClientCwd";
    /// `Sync.PushWorkspaceFiles` - client -> server; uploads `files` (`FilePayload`) under
    /// `base_path` for `workspace_id`, chunked by `batch_index` of `batch_total`.
    pub const PUSH_WORKSPACE_FILES: &str = "PushWorkspaceFiles";
    /// `Sync.PushWorkspaceFilesAck` - server -> client; per-batch receipt naming `workspace_id`,
    /// `batch_index`, the accepted count and an optional `error`.
    pub const PUSH_WORKSPACE_FILES_ACK: &str = "PushWorkspaceFilesAck";
    /// `Sync.RequestWorkspaceFiles` - client -> server; asks for the `workspace_id` files matching
    /// `glob_patterns`, minus `exclude_patterns`.
    pub const REQUEST_WORKSPACE_FILES: &str = "RequestWorkspaceFiles";
    /// `Sync.WorkspaceReady` - server -> client; announces that `workspace_id` is ready and names
    /// the `container_id` backing it.
    pub const WORKSPACE_READY: &str = "WorkspaceReady";
    /// The 15 `workspace` names as one slice, in declaration order.
    ///
    /// The Polemos device names follow the workspace-status ones in the slice.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        OPEN_WORKSPACE,
        WORKSPACE_STATUS,
        REQUEST_WORKSPACE_STATUS,
        OPEN_WORKSPACE_RESPONSE,
        LIST_POLEMOS_DEVICES,
        POLEMOS_DEVICE_LIST,
        REGISTER_POLEMOS_DEVICE,
        REGISTER_POLEMOS_DEVICE_RESPONSE,
        SWITCH_WORKSPACE,
        SWITCH_WORKSPACE_RESPONSE,
        SET_CLIENT_CWD,
        PUSH_WORKSPACE_FILES,
        PUSH_WORKSPACE_FILES_ACK,
        REQUEST_WORKSPACE_FILES,
        WORKSPACE_READY,
    ];
}

/// String constants for system / UI control variants.
pub mod system {
    /// `Sync.SystemMessage` - server -> client one-way notification: a `SystemNotification` plus a
    /// `timestamp`; the `system_notification` channel topic fans it out to subscribers.
    pub const SYSTEM_MESSAGE: &str = "SystemMessage";
    /// `Sync.WebUiControl` - client -> server; asks the server to run the web-UI `command`.
    /// Answered by `WebUiControlResponse`.
    pub const WEB_UI_CONTROL: &str = "WebUiControl";
    /// `Sync.WebUiControlResponse` - server -> client; echoes `command` with `success`, a human
    /// `message`, and an optional `url`.
    pub const WEB_UI_CONTROL_RESPONSE: &str = "WebUiControlResponse";
    /// `Sync.WebUiStatus` - server -> client snapshot of the embedded web UI: `running`, optional
    /// `url` and `container_id`.
    pub const WEB_UI_STATUS: &str = "WebUiStatus";
    /// `Sync.RequestWebUiStatus` - client -> server; asks for the current web-UI status. Answered
    /// by `WebUiStatus`.
    pub const REQUEST_WEB_UI_STATUS: &str = "RequestWebUiStatus";

    /// `Sync.BadgeTransition` - server -> client; records a badge moving from
    /// `previous_llm_session_id` / `previous_container_id` to the current pair, correlated by
    /// `transition_uuid` and an optional `linked_session_uuid`.
    pub const BADGE_TRANSITION: &str = "BadgeTransition";
    /// The 6 `system` names as one slice, in declaration order.
    ///
    /// `BadgeTransition` closes the slice after the web-UI control names.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        SYSTEM_MESSAGE,
        WEB_UI_CONTROL,
        WEB_UI_CONTROL_RESPONSE,
        WEB_UI_STATUS,
        REQUEST_WEB_UI_STATUS,
        BADGE_TRANSITION,
    ];
}

/// String constants for authentication variants.
///
/// Login, registration, user CRUD, and password management.
pub mod auth {
    /// `Sync.AuthLogin` - client -> server (`SyncReq`); submits `username` and `password`.
    /// Answered by `AuthLoginResponse`.
    pub const AUTH_LOGIN: &str = "AuthLogin";
    /// `Sync.AuthLoginResponse` - server -> client; `ok` plus optional `token`, `session_id`,
    /// `user_id`, `username`, `display_name`, `role` and `error`.
    pub const AUTH_LOGIN_RESPONSE: &str = "AuthLoginResponse";
    /// `Sync.AuthRegister` - client -> server (`SyncReq`); creates a user from `username`,
    /// `password` and an optional `display_name`. Answered by `AuthRegisterResponse`.
    pub const AUTH_REGISTER: &str = "AuthRegister";
    /// `Sync.AuthRegisterResponse` - server -> client; `ok`, optional `user_id`, `username` and
    /// `error`.
    pub const AUTH_REGISTER_RESPONSE: &str = "AuthRegisterResponse";
    /// `Sync.AuthListUsers` - client -> server (`SyncReq`) admin listing. Answered by
    /// `AuthListUsersResponse`.
    pub const AUTH_LIST_USERS: &str = "AuthListUsers";
    /// `Sync.AuthListUsersResponse` - server -> client; `ok`, optional `users` (`AuthUserInfo`)
    /// and `error`.
    pub const AUTH_LIST_USERS_RESPONSE: &str = "AuthListUsersResponse";
    /// `Sync.AuthGetUser` - client -> server (`SyncReq`); fetches one user by `user_id`. Answered
    /// by `AuthGetUserResponse`.
    pub const AUTH_GET_USER: &str = "AuthGetUser";
    /// `Sync.AuthGetUserResponse` - server -> client; `ok`, optional `user` (`AuthUserInfo`) and
    /// `error`.
    pub const AUTH_GET_USER_RESPONSE: &str = "AuthGetUserResponse";
    /// `Sync.AuthDeleteUser` - client -> server (`SyncReq`); deletes `user_id`. Answered by
    /// `AuthDeleteUserResponse`.
    pub const AUTH_DELETE_USER: &str = "AuthDeleteUser";
    /// `Sync.AuthDeleteUserResponse` - server -> client; `ok` and an optional `error`.
    pub const AUTH_DELETE_USER_RESPONSE: &str = "AuthDeleteUserResponse";
    /// `Sync.AuthChangePassword` - client -> server (`SyncReq`); rotates `user_id`'s password from
    /// `old_password` to `new_password`. Answered by `AuthChangePasswordResponse`.
    pub const AUTH_CHANGE_PASSWORD: &str = "AuthChangePassword";
    /// `Sync.AuthChangePasswordResponse` - server -> client; `ok` plus an optional `error`.
    pub const AUTH_CHANGE_PASSWORD_RESPONSE: &str = "AuthChangePasswordResponse";
    /// `Sync.ArbiterStatus` - client -> server; asks the arbiter supervising `instance_uuid` for
    /// its state. Answered by `ArbiterStatusResponse`.
    pub const ARBITER_STATUS: &str = "ArbiterStatus";
    /// `Sync.ArbiterStatusResponse` - server -> client; `ok`, an optional free-form JSON `status`
    /// and `error`.
    pub const ARBITER_STATUS_RESPONSE: &str = "ArbiterStatusResponse";
    /// `Sync.ArbiterLockdown` - client -> server; commands the arbiter of `instance_uuid` to lock
    /// down, attributing the action to `delegator_id` with a `reason`.
    pub const ARBITER_LOCKDOWN: &str = "ArbiterLockdown";
    /// `Sync.ArbiterLockdownResponse` - server -> client; `ok` and an optional `error`.
    pub const ARBITER_LOCKDOWN_RESPONSE: &str = "ArbiterLockdownResponse";
    /// `Sync.ArbiterRestore` - client -> server; restores `instance_uuid` to `target_level`,
    /// attributed to `delegator_id`.
    pub const ARBITER_RESTORE: &str = "ArbiterRestore";
    /// `Sync.ArbiterRestoreResponse` - server -> client; `ok` and an optional `error`.
    pub const ARBITER_RESTORE_RESPONSE: &str = "ArbiterRestoreResponse";

    /// The 18 `auth` names as one slice, in declaration order.
    ///
    /// The slice closes with the six arbiter request/reply names.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        AUTH_LOGIN,
        AUTH_LOGIN_RESPONSE,
        AUTH_REGISTER,
        AUTH_REGISTER_RESPONSE,
        AUTH_LIST_USERS,
        AUTH_LIST_USERS_RESPONSE,
        AUTH_GET_USER,
        AUTH_GET_USER_RESPONSE,
        AUTH_DELETE_USER,
        AUTH_DELETE_USER_RESPONSE,
        AUTH_CHANGE_PASSWORD,
        AUTH_CHANGE_PASSWORD_RESPONSE,
        ARBITER_STATUS,
        ARBITER_STATUS_RESPONSE,
        ARBITER_LOCKDOWN,
        ARBITER_LOCKDOWN_RESPONSE,
        ARBITER_RESTORE,
        ARBITER_RESTORE_RESPONSE,
    ];
}

/// String constants for container / server log-streaming variants.
///
/// `Subscribe*` / `Unsubscribe*` are client request/reply pairs; `ContainerLogEntry` and
/// `ServerLogEntry` are the one-way pushes that follow a subscription. The two pushes are
/// the `container_logs` / `server_logs` channel topics in `packages/sync/src/lib.rs`.
pub mod log_subscription {
    /// `Sync.SubscribeContainerLogs` - client -> server; starts streaming the logs of container
    /// `instance_uuid`, backfilling `tail` lines in the reply.
    pub const SUBSCRIBE_CONTAINER_LOGS: &str = "SubscribeContainerLogs";
    /// `Sync.SubscribeContainerLogsResponse` - server -> client; `ok`, optional `error`, and the
    /// backfilled `entries` (`LogEntryData`).
    pub const SUBSCRIBE_CONTAINER_LOGS_RESPONSE: &str = "SubscribeContainerLogsResponse";
    /// `Sync.UnsubscribeContainerLogs` - client -> server; stops the log stream of container
    /// `instance_uuid`. Answered by `UnsubscribeContainerLogsResponse`.
    pub const UNSUBSCRIBE_CONTAINER_LOGS: &str = "UnsubscribeContainerLogs";
    /// `Sync.UnsubscribeContainerLogsResponse` - server -> client; `ok` and an optional `error`.
    pub const UNSUBSCRIBE_CONTAINER_LOGS_RESPONSE: &str = "UnsubscribeContainerLogsResponse";
    /// `Sync.ContainerLogEntry` - server -> client one-way; one `entry` (`LogEntryData`) for
    /// `instance_uuid`, the payload of the `container_logs` topic.
    pub const CONTAINER_LOG_ENTRY: &str = "ContainerLogEntry";
    /// `Sync.SubscribeServerLogs` - client -> server; subscribes to the gateway's own log stream,
    /// backfilling `tail` lines. Answered by `SubscribeServerLogsResponse`.
    pub const SUBSCRIBE_SERVER_LOGS: &str = "SubscribeServerLogs";
    /// `Sync.SubscribeServerLogsResponse` - server -> client; `ok`, optional `error`, and the
    /// backfilled `entries`.
    pub const SUBSCRIBE_SERVER_LOGS_RESPONSE: &str = "SubscribeServerLogsResponse";
    /// `Sync.UnsubscribeServerLogs` - client -> server with no payload; stops the server log
    /// stream. Answered by `UnsubscribeServerLogsResponse`.
    pub const UNSUBSCRIBE_SERVER_LOGS: &str = "UnsubscribeServerLogs";
    /// `Sync.UnsubscribeServerLogsResponse` - server -> client; `ok` and an optional `error`.
    pub const UNSUBSCRIBE_SERVER_LOGS_RESPONSE: &str = "UnsubscribeServerLogsResponse";
    /// `Sync.ServerLogEntry` - server -> client one-way; one `entry` (`LogEntryData`) of the
    /// gateway log, the payload of the `server_logs` topic.
    pub const SERVER_LOG_ENTRY: &str = "ServerLogEntry";

    /// The 10 `log_subscription` names as one slice, in declaration order.
    ///
    /// The container names come first, then the server-log names.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        SUBSCRIBE_CONTAINER_LOGS,
        SUBSCRIBE_CONTAINER_LOGS_RESPONSE,
        UNSUBSCRIBE_CONTAINER_LOGS,
        UNSUBSCRIBE_CONTAINER_LOGS_RESPONSE,
        CONTAINER_LOG_ENTRY,
        SUBSCRIBE_SERVER_LOGS,
        SUBSCRIBE_SERVER_LOGS_RESPONSE,
        UNSUBSCRIBE_SERVER_LOGS,
        UNSUBSCRIBE_SERVER_LOGS_RESPONSE,
        SERVER_LOG_ENTRY,
    ];
}

/// String constants for the YOLO cruise-control and skill-chain variants.
///
/// The `Yolo*` control names are request/reply pairs in plana's `Sync` method table; the
/// `YoloCycle*`, `YoloTask*` and `SkillChain*` names are one-way progress pushes. Those
/// pushes are the `yolo_cycle` / `skill_chain` channel topics in `packages/sync/src/lib.rs`.
pub mod yolo {
    /// `Sync.YoloStart` - client -> server (`AsyncReq`), payload-free; starts the YOLO
    /// cruise-control loop. Answered by `YoloStartResponse`.
    pub const YOLO_START: &str = "YoloStart";
    /// `Sync.YoloStartResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_START_RESPONSE: &str = "YoloStartResponse";
    /// `Sync.YoloStop` - client -> server (`AsyncReq`), payload-free; stops the loop. Answered by
    /// `YoloStopResponse`.
    pub const YOLO_STOP: &str = "YoloStop";
    /// `Sync.YoloStopResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_STOP_RESPONSE: &str = "YoloStopResponse";
    /// `Sync.YoloTerminate` - client -> server (`AsyncReq`), payload-free; tears the loop down.
    /// Answered by `YoloTerminateResponse`.
    pub const YOLO_TERMINATE: &str = "YoloTerminate";
    /// `Sync.YoloTerminateResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_TERMINATE_RESPONSE: &str = "YoloTerminateResponse";
    /// `Sync.YoloStatus` - client -> server (`AsyncReq`), payload-free; asks whether the loop is
    /// running. Answered by `YoloStatusResponse`.
    pub const YOLO_STATUS: &str = "YoloStatus";
    /// `Sync.YoloStatusResponse` - server -> client; `active`, `loop_count`, optional `started_at`
    /// and `current_cycle`, plus the per-tier `tiers` (`YoloTierStatus`).
    pub const YOLO_STATUS_RESPONSE: &str = "YoloStatusResponse";
    /// `Sync.YoloGetConfig` - client -> server (`AsyncReq`), payload-free; asks for the tier
    /// configuration. Answered by `YoloConfigResponse`.
    pub const YOLO_GET_CONFIG: &str = "YoloGetConfig";
    /// `Sync.YoloConfigResponse` - server -> client; `tiers` as `YoloTierConfig` entries.
    pub const YOLO_CONFIG_RESPONSE: &str = "YoloConfigResponse";
    /// `Sync.YoloUpdateTask` - client -> server (`AsyncReq`); enables or disables the `agent` plus
    /// `skill` task assigned to `tier`. Answered by `YoloUpdateTaskResponse`.
    pub const YOLO_UPDATE_TASK: &str = "YoloUpdateTask";
    /// `Sync.YoloUpdateTaskResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_UPDATE_TASK_RESPONSE: &str = "YoloUpdateTaskResponse";
    /// `Sync.YoloSetTierInterval` - client -> server (`AsyncReq`); sets `tier`'s run interval to
    /// `interval_secs`. Answered by `YoloSetTierIntervalResponse`.
    pub const YOLO_SET_TIER_INTERVAL: &str = "YoloSetTierInterval";
    /// `Sync.YoloSetTierIntervalResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_SET_TIER_INTERVAL_RESPONSE: &str = "YoloSetTierIntervalResponse";
    /// `Sync.YoloRunTierNow` - client -> server (`AsyncReq`); triggers `tier` immediately.
    /// Answered by `YoloRunTierNowResponse`.
    pub const YOLO_RUN_TIER_NOW: &str = "YoloRunTierNow";
    /// `Sync.YoloRunTierNowResponse` - server -> client; `ok` and an optional `error`.
    pub const YOLO_RUN_TIER_NOW_RESPONSE: &str = "YoloRunTierNowResponse";
    /// `Sync.YoloCycleStep` - server -> client one-way cycle progress: `skill`, `loop_count`,
    /// `status`, and optional `token_usage` / `model_name`.
    pub const YOLO_CYCLE_STEP: &str = "YoloCycleStep";
    /// `Sync.YoloCycleComplete` - server -> client one-way; a cycle finished after `loop_count`
    /// iterations in `duration_ms`.
    pub const YOLO_CYCLE_COMPLETE: &str = "YoloCycleComplete";
    /// `Sync.YoloTaskStart` - server -> client one-way; the `agent` plus `skill` task of `tier`
    /// just started.
    pub const YOLO_TASK_START: &str = "YoloTaskStart";
    /// `Sync.YoloTaskDone` - server -> client one-way; that task finished, with `duration_ms` and
    /// optional `token_usage` / `model_name`.
    pub const YOLO_TASK_DONE: &str = "YoloTaskDone";
    /// `Sync.YoloTaskError` - server -> client one-way; that task failed, with the `error` text.
    pub const YOLO_TASK_ERROR: &str = "YoloTaskError";

    /// `Sync.SkillChainStart` - server -> client one-way; a skill chain for the current
    /// turn began.
    pub const SKILL_CHAIN_START: &str = "SkillChainStart";
    /// `Sync.SkillChainStep` - server -> client one-way; `skill` advanced to `status`.
    pub const SKILL_CHAIN_STEP: &str = "SkillChainStep";
    /// `Sync.SkillChainComplete` - server -> client one-way; the chain reached its final `skill`.
    pub const SKILL_CHAIN_COMPLETE: &str = "SkillChainComplete";
    /// The 24 `yolo` names as one slice, in declaration order.
    ///
    /// The three `SkillChain*` names share this slice with the `Yolo*` ones.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        YOLO_START,
        YOLO_START_RESPONSE,
        YOLO_STOP,
        YOLO_STOP_RESPONSE,
        YOLO_TERMINATE,
        YOLO_TERMINATE_RESPONSE,
        YOLO_STATUS,
        YOLO_STATUS_RESPONSE,
        YOLO_GET_CONFIG,
        YOLO_CONFIG_RESPONSE,
        YOLO_UPDATE_TASK,
        YOLO_UPDATE_TASK_RESPONSE,
        YOLO_SET_TIER_INTERVAL,
        YOLO_SET_TIER_INTERVAL_RESPONSE,
        YOLO_RUN_TIER_NOW,
        YOLO_RUN_TIER_NOW_RESPONSE,
        YOLO_CYCLE_STEP,
        YOLO_CYCLE_COMPLETE,
        YOLO_TASK_START,
        YOLO_TASK_DONE,
        YOLO_TASK_ERROR,
        SKILL_CHAIN_START,
        SKILL_CHAIN_STEP,
        SKILL_CHAIN_COMPLETE,
    ];
}

/// String constants for the NOA workspace variants.
///
/// The handshake is a fixed round trip documented in
/// `packages/celestia-types/src/ws/ui/noa.rs`: scepter -> client `RequestNoaHandshake`,
/// client -> scepter `NoaHandshakeResponse`, scepter -> client `NoaAuthRequest` (the branch
/// picker), client -> scepter `NoaAuthResponse`, then scepter -> client `NoaReady`.
/// `NoaEventSync` / `NoaEventSyncAck` are the event pair used after `NoaReady`.
pub mod noa {
    /// `Sync.RequestNoaHandshake` - scepter (server) -> client; asks a client that declared the
    /// noa_workspace capability to set up `remote_name` at `remote_path` for `workspace_id`.
    pub const REQUEST_NOA_HANDSHAKE: &str = "RequestNoaHandshake";
    /// `Sync.NoaHandshakeResponse` - client -> server; reports `repo_id`, `current_branch` and the
    /// `noa_initialized` / `gitignore_updated` flags of that workspace.
    pub const NOA_HANDSHAKE_RESPONSE: &str = "NoaHandshakeResponse";
    /// `Sync.NoaAuthRequest` - scepter (server) -> client; the branch picker: candidate
    /// `branches`, a `suggested_branch` and the `reason` for the choice.
    pub const NOA_AUTH_REQUEST: &str = "NoaAuthRequest";
    /// `Sync.NoaAuthResponse` - client -> server; the operator's `selected_branch` (with an
    /// optional `branch_base`) and the `approved` flag.
    pub const NOA_AUTH_RESPONSE: &str = "NoaAuthResponse";
    /// `Sync.NoaReady` - scepter (server) -> client; terminal handshake event naming the `branch`
    /// and `snapshot_id` of the ready workspace.
    pub const NOA_READY: &str = "NoaReady";
    /// `Sync.NoaEventSync` - bidirectional event batch (client <-> server) for `workspace_id`:
    /// `events` (`NoaEvent` entries) plus an optional `direction` tag naming which way it went.
    pub const NOA_EVENT_SYNC: &str = "NoaEventSync";
    /// `Sync.NoaEventSyncAck` - the receiving peer's answer to a `NoaEventSync` batch for
    /// `workspace_id`, returning the `last_event_id` the sender should resume from.
    pub const NOA_EVENT_SYNC_ACK: &str = "NoaEventSyncAck";

    /// The 7 `noa` names as one slice, in declaration order.
    ///
    /// Both directions of the handshake and the event-sync pair are in the slice.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        REQUEST_NOA_HANDSHAKE,
        NOA_HANDSHAKE_RESPONSE,
        NOA_AUTH_REQUEST,
        NOA_AUTH_RESPONSE,
        NOA_READY,
        NOA_EVENT_SYNC,
        NOA_EVENT_SYNC_ACK,
    ];
}

/// String constants for the file-browsing variants.
///
/// `RequestFileTree` / `RequestFileRead` are client requests (`SyncReq` in plana's table);
/// `FileTree` / `FileRead` are their server replies. A target is a container badge, a host id
/// or a workspace id, carried as `FileTarget` in the sibling `types/mod.rs`.
pub mod file {
    /// `Sync.RequestFileTree` - client -> server (`SyncReq`); lists the directory `path` inside
    /// `target` (`FileTarget`: container, host or workspace).
    pub const REQUEST_FILE_TREE: &str = "RequestFileTree";
    /// `Sync.FileTree` - server -> client reply to `RequestFileTree`; echoes `target` and `path`
    /// and returns `entries` (`FileTreeEntry`: name, kind, size).
    pub const FILE_TREE: &str = "FileTree";
    /// `Sync.RequestFileRead` - client -> server (`SyncReq`); reads the file at `path` inside
    /// `target`.
    pub const REQUEST_FILE_READ: &str = "RequestFileRead";
    /// `Sync.FileRead` - server -> client reply to `RequestFileRead`; `content`, `size` and a
    /// `truncated` flag for the requested `target` and `path`.
    pub const FILE_READ: &str = "FileRead";

    /// The 4 `file` names as one slice, in declaration order.
    ///
    /// Only four names: two requests and their two replies.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[REQUEST_FILE_TREE, FILE_TREE, REQUEST_FILE_READ, FILE_READ];
}

/// String constants for the Bridge Network variants.
///
/// They back the chat sub-page that lists host machines with live metrics and the workspaces
/// attached to the selected host. `RequestBridgeNetwork` is a client `SyncReq` with an empty
/// payload; `BridgeNetwork` is the server reply.
pub mod bridge {
    /// `Sync.RequestBridgeNetwork` - client -> server (`SyncReq`) with an empty payload; asks for
    /// the host / workspace topology.
    pub const REQUEST_BRIDGE_NETWORK: &str = "RequestBridgeNetwork";
    /// `Sync.BridgeNetwork` - server -> client; `hosts` (`HostMetrics`) and `workspaces`
    /// (`WorkspaceNode`, each with noa-git status and token usage).
    pub const BRIDGE_NETWORK: &str = "BridgeNetwork";

    /// The 2 `bridge` names as one slice, in declaration order.
    ///
    /// One request plus its one reply.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[REQUEST_BRIDGE_NETWORK, BRIDGE_NETWORK];
}

/// String constants for the industrial push variants.
///
/// All four are one-way pushes produced by scepter and consumed by the shittim-chest webui
/// (the wire types sit in the sibling `types/mod.rs`). Telemetry, alarm and write-approval
/// also have channel topics in `packages/sync/src/lib.rs`; discovery does not.
pub mod industrial {
    /// `Sync.IndustrialTelemetryPush` - server -> client one-way; live readings, either a single
    /// `reading` or a batch in `readings` (`IndustrialSensorReading`).
    pub const INDUSTRIAL_TELEMETRY_PUSH: &str = "IndustrialTelemetryPush";
    /// `Sync.IndustrialAlarmPush` - server -> client one-way; one `event` (`IndustrialAlarmEvent`)
    /// fired on a threshold breach or clear.
    pub const INDUSTRIAL_ALARM_PUSH: &str = "IndustrialAlarmPush";
    /// `Sync.IndustrialDiscoveryPush` - server -> client one-way; evernight discovery progress for
    /// `session_id`: `phase`, `message`, `found_devices`, `progress_percent`, `raw_findings`.
    pub const INDUSTRIAL_DISCOVERY_PUSH: &str = "IndustrialDiscoveryPush";
    /// `Sync.IndustrialWriteApprovalPush` - server -> client one-way; a safety-critical PLC write
    /// awaiting operator approval, carrying the `request_id` the UI must echo in
    /// `industrial.approveWrite`, plus `station_id`, `address`, the values, reason and
    /// `risk_level`.
    pub const INDUSTRIAL_WRITE_APPROVAL_PUSH: &str = "IndustrialWriteApprovalPush";

    /// The 4 `industrial` names as one slice, in declaration order.
    ///
    /// All four entries are one-way pushes; discovery has no channel topic.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        INDUSTRIAL_TELEMETRY_PUSH,
        INDUSTRIAL_ALARM_PUSH,
        INDUSTRIAL_DISCOVERY_PUSH,
        INDUSTRIAL_WRITE_APPROVAL_PUSH,
    ];
}

/// String constants for the semantic-search variants.
///
/// `SearchRequest` is the client query (default `limit` of 10, optional `source` filter and
/// `min_score` floor); `SearchResponse` is the server reply carrying the search payload.
pub mod search {
    /// `Sync.SearchRequest` - client -> server; semantic query text with a `limit`, an optional
    /// `source` filter and a `min_score` floor.
    pub const SEARCH_REQUEST: &str = "SearchRequest";
    /// `Sync.SearchResponse` - server -> client; wraps the search result payload
    /// (`super::super::search::SearchResponse`).
    pub const SEARCH_RESPONSE: &str = "SearchResponse";

    /// The 2 `search` names as one slice, in declaration order.
    ///
    /// One request plus its one reply.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[SEARCH_REQUEST, SEARCH_RESPONSE];
}

/// String constants for the long-term memory (Philia) variants.
///
/// Three request/reply pairs - store, semantic query and delete - in that order. The requests
/// are client -> server, the replies server -> client.
pub mod memory {
    /// `Sync.MemoryStoreRequest` - client -> server; stores `text` as a `node_type` node, with
    /// optional `source_episode_id`, `related_node_ids` and `properties`.
    pub const MEMORY_STORE_REQUEST: &str = "MemoryStoreRequest";
    /// `Sync.MemoryStoreResponse` - server -> client; returns the new `node_id`.
    pub const MEMORY_STORE_RESPONSE: &str = "MemoryStoreResponse";
    /// `Sync.MemoryQueryRequest` - client -> server; semantic memory query with a `limit`, plus
    /// optional `graph_depth`, `node_type_filter` and `subgraph` expansion.
    pub const MEMORY_QUERY_REQUEST: &str = "MemoryQueryRequest";
    /// `Sync.MemoryQueryResponse` - server -> client; echoes `query` and returns `total` with
    /// `results` (`MemoryQueryItem` entries).
    pub const MEMORY_QUERY_RESPONSE: &str = "MemoryQueryResponse";
    /// `Sync.MemoryDeleteRequest` - client -> server; deletes the memory node identified by
    /// `node_id`.
    pub const MEMORY_DELETE_REQUEST: &str = "MemoryDeleteRequest";
    /// `Sync.MemoryDeleteResponse` - server -> client; `deleted` reports whether the node existed.
    pub const MEMORY_DELETE_RESPONSE: &str = "MemoryDeleteResponse";

    /// The 6 `memory` names as one slice, in declaration order.
    ///
    /// Three request/reply pairs in store / query / delete order.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        MEMORY_STORE_REQUEST,
        MEMORY_STORE_RESPONSE,
        MEMORY_QUERY_REQUEST,
        MEMORY_QUERY_RESPONSE,
        MEMORY_DELETE_REQUEST,
        MEMORY_DELETE_RESPONSE,
    ];
}

/// String constants for the conversation-history variants.
///
/// `ConversationStarted` is a server push that opens a lazy-loaded history session;
/// `RequestRecentMessages` / `RequestOlderMessages` are client requests, both answered by
/// `MessagesResponse`.
pub mod conversation {
    /// `Sync.ConversationStarted` - server -> client, emitted once when the server creates a
    /// conversation for the active task so the client can page its history later.
    pub const CONVERSATION_STARTED: &str = "ConversationStarted";
    /// `Sync.RequestRecentMessages` - client -> server; asks for the newest `limit` messages of
    /// `conversation_id`. Answered by `MessagesResponse`.
    pub const REQUEST_RECENT_MESSAGES: &str = "RequestRecentMessages";
    /// `Sync.RequestOlderMessages` - client -> server; pages backwards from the ISO-8601
    /// `before_created_at` cursor for `conversation_id`. Answered by `MessagesResponse`.
    pub const REQUEST_OLDER_MESSAGES: &str = "RequestOlderMessages";
    /// `Sync.MessagesResponse` - server -> client; one history `page` (`MessagesPage`) for
    /// `conversation_id`, answering either message request.
    pub const MESSAGES_RESPONSE: &str = "MessagesResponse";

    /// The 4 `conversation` names as one slice, in declaration order.
    ///
    /// One push, two requests and the reply they share.
    /// Consumed only by this module's `all_variant_counts_match` and
    /// `no_duplicates_across_groups` tests - `groups` is compiled for tests only
    /// (`pub mod groups;` under the test cfg at `types/mod.rs:2`).
    pub const VARIANTS: &[&str] = &[
        CONVERSATION_STARTED,
        REQUEST_RECENT_MESSAGES,
        REQUEST_OLDER_MESSAGES,
        MESSAGES_RESPONSE,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variant_counts_match() {
        let total: usize = [
            protocol::VARIANTS.len(),
            layer2::VARIANTS.len(),
            agent::VARIANTS.len(),
            task::VARIANTS.len(),
            llm::VARIANTS.len(),
            snapshot::VARIANTS.len(),
            config_fs::VARIANTS.len(),
            knowledge_base::VARIANTS.len(),
            workspace::VARIANTS.len(),
            system::VARIANTS.len(),
            auth::VARIANTS.len(),
            log_subscription::VARIANTS.len(),
            yolo::VARIANTS.len(),
            noa::VARIANTS.len(),
            file::VARIANTS.len(),
            bridge::VARIANTS.len(),
            industrial::VARIANTS.len(),
            search::VARIANTS.len(),
            conversation::VARIANTS.len(),
            memory::VARIANTS.len(),
        ]
        .iter()
        .sum();
        assert_eq!(total, 215, "expected 215 grouped variants, got {total}");
    }

    #[test]
    fn no_duplicates_across_groups() {
        let mut all: Vec<&str> = Vec::new();
        all.extend(protocol::VARIANTS);
        all.extend(layer2::VARIANTS);
        all.extend(agent::VARIANTS);
        all.extend(task::VARIANTS);
        all.extend(llm::VARIANTS);
        all.extend(snapshot::VARIANTS);
        all.extend(config_fs::VARIANTS);
        all.extend(knowledge_base::VARIANTS);
        all.extend(workspace::VARIANTS);
        all.extend(system::VARIANTS);
        all.extend(auth::VARIANTS);
        all.extend(log_subscription::VARIANTS);
        all.extend(yolo::VARIANTS);
        all.extend(noa::VARIANTS);
        all.extend(file::VARIANTS);
        all.extend(bridge::VARIANTS);
        all.extend(industrial::VARIANTS);
        all.extend(search::VARIANTS);
        all.extend(conversation::VARIANTS);
        all.extend(memory::VARIANTS);
        all.sort();
        let dupes: Vec<&str> = all
            .windows(2)
            .filter_map(|w| if w[0] == w[1] { Some(w[0]) } else { None })
            .collect();
        assert!(dupes.is_empty(), "duplicate variant names: {dupes:?}");
    }
}
