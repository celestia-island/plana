//! Domain-grouped view of [`super::TuiMessage`] variants.
//!
//! This module provides categorized documentation and re-exports of
//! `TuiMessage` variant *names* (as constants) so that developers can
//! quickly discover which variants belong to which functional domain.
//!
//! The full enum remains in [`super::TuiMessage`]; nothing here changes
//! the public API. This is the first step toward eventual full
//! decomposition of the 150-variant enum into separate types.

/// String constants for connection / handshake variants.
///
/// Covers the initial wire protocol: ping, version negotiation, and the
/// token-based handshake.
pub mod protocol {
    pub const PING: &str = "Ping";
    pub const SERVER_VERSION: &str = "ServerVersion";
    pub const CONNECT_HANDSHAKE: &str = "ConnectHandshake";
    pub const HANDSHAKE_ACK: &str = "HandshakeAck";
    pub const VERSION_MISMATCH: &str = "VersionMismatch";

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
/// (Layer-2) agents, their MCP tools, skills, and prompt templates.
pub mod layer2 {
    pub const LAYER2_AGENT_LIST: &str = "Layer2AgentList";
    pub const LAYER2_AGENT_LIST_RESPONSE: &str = "Layer2AgentListResponse";
    pub const LAYER2_AGENT_MCP_TOOLS: &str = "Layer2AgentMcpTools";
    pub const LAYER2_AGENT_MCP_RESPONSE: &str = "Layer2AgentMcpResponse";
    pub const LAYER2_AGENT_SKILLS: &str = "Layer2AgentSkills";
    pub const LAYER2_AGENT_SKILLS_RESPONSE: &str = "Layer2AgentSkillsResponse";
    pub const LAYER2_AGENT_MCP_PROMPT: &str = "Layer2AgentMcpPrompt";
    pub const LAYER2_AGENT_MCP_PROMPT_RESPONSE: &str = "Layer2AgentMcpPromptResponse";
    pub const LAYER2_AGENT_SKILL_PROMPT: &str = "Layer2AgentSkillPrompt";
    pub const LAYER2_AGENT_SKILL_PROMPT_RESPONSE: &str = "Layer2AgentSkillPromptResponse";
    pub const CUSTOM_AGENT_LIST: &str = "CustomAgentList";
    pub const CUSTOM_AGENT_LIST_RESPONSE: &str = "CustomAgentListResponse";
    pub const SUBSCRIBE_CUSTOM_AGENT: &str = "SubscribeCustomAgent";
    pub const SUBSCRIBE_CUSTOM_AGENT_RESPONSE: &str = "SubscribeCustomAgentResponse";
    pub const UNSUBSCRIBE_CUSTOM_AGENT: &str = "UnsubscribeCustomAgent";
    pub const UNSUBSCRIBE_CUSTOM_AGENT_RESPONSE: &str = "UnsubscribeCustomAgentResponse";

    pub const VARIANTS: &[&str] = &[
        LAYER2_AGENT_LIST,
        LAYER2_AGENT_LIST_RESPONSE,
        LAYER2_AGENT_MCP_TOOLS,
        LAYER2_AGENT_MCP_RESPONSE,
        LAYER2_AGENT_SKILLS,
        LAYER2_AGENT_SKILLS_RESPONSE,
        LAYER2_AGENT_MCP_PROMPT,
        LAYER2_AGENT_MCP_PROMPT_RESPONSE,
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
/// MCP tool results, human-in-the-loop reviews/consultations, and
/// agent CRUD.
pub mod agent {
    pub const USER_MESSAGE: &str = "UserMessage";
    pub const AGENT_RESPONSE: &str = "AgentResponse";
    pub const AGENT_STREAMING_CHUNK: &str = "AgentStreamingChunk";
    pub const AGENT_REPORT: &str = "AgentReport";
    pub const AGENT_TRANSFER: &str = "AgentTransfer";
    pub const ORCHESTRATION_STATUS: &str = "OrchestrationStatus";
    pub const MCP_TOOL_RESULT: &str = "McpToolResult";
    pub const STREAMING_TAIL: &str = "StreamingTail";
    pub const HUMAN_REVIEW_REQUEST: &str = "HumanReviewRequest";
    pub const HUMAN_REVIEW_RESPONSE: &str = "HumanReviewResponse";
    pub const ASK_HUMAN_REQUEST: &str = "AskHumanRequest";
    pub const ASK_HUMAN_REPLY: &str = "AskHumanReply";
    pub const AUTO_MODE_UPDATE: &str = "AutoModeUpdate";
    pub const SCEPTER_IDENTITY: &str = "ScepterIdentity";
    pub const LIST_AGENTS: &str = "ListAgents";
    pub const AGENT_LIST_RESPONSE: &str = "AgentListResponse";
    pub const AGENT_UPDATE: &str = "AgentUpdate";
    pub const RETRY_AGENT_REQUEST: &str = "RetryAgentRequest";
    pub const UNDO_REQUEST: &str = "UndoRequest";
    pub const RELOAD_ALL_AGENT_CONFIGS: &str = "ReloadAllAgentConfigs";
    pub const RELOAD_AGENT_CONFIG: &str = "ReloadAgentConfig";

    pub const AGENT_REPORT_REPLY: &str = "AgentReportReply";
    pub const AGENT_THINKING_STEP: &str = "AgentThinkingStep";
    pub const AGENT_TOOL_CALL: &str = "AgentToolCall";
    pub const VARIANTS: &[&str] = &[
        USER_MESSAGE,
        AGENT_RESPONSE,
        AGENT_STREAMING_CHUNK,
        AGENT_REPORT,
        AGENT_TRANSFER,
        ORCHESTRATION_STATUS,
        MCP_TOOL_RESULT,
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
    pub const TASK_CREATED: &str = "TaskCreated";
    pub const TASK_STATUS_UPDATE: &str = "TaskStatusUpdate";

    pub const VARIANTS: &[&str] = &[TASK_CREATED, TASK_STATUS_UPDATE];
}

/// String constants for LLM provider configuration variants.
///
/// CRUD operations on providers, model-level configuration, endpoint
/// validation, and usage-period accounting.
pub mod llm {
    pub const CONFIGURE_LLM_PROVIDER: &str = "ConfigureLlmProvider";
    pub const LLM_PROVIDER_CONFIGURED: &str = "LlmProviderConfigured";
    pub const RENAME_PROVIDER: &str = "RenameProvider";
    pub const PROVIDER_RENAMED: &str = "ProviderRenamed";
    pub const EDIT_PROVIDER: &str = "EditProvider";
    pub const PROVIDER_EDITED: &str = "ProviderEdited";
    pub const DELETE_PROVIDER: &str = "DeleteProvider";
    pub const PROVIDER_DELETED: &str = "ProviderDeleted";
    pub const LIST_CONFIGURED_PROVIDERS: &str = "ListConfiguredProviders";
    pub const CONFIGURED_PROVIDERS_LIST: &str = "ConfiguredProvidersList";
    pub const UPDATE_MODEL_PROVIDER_CONFIG: &str = "UpdateModelProviderConfig";
    pub const MODEL_PROVIDER_CONFIG_UPDATED: &str = "ModelProviderConfigUpdated";
    pub const VALIDATE_ENDPOINT: &str = "ValidateEndpoint";
    pub const ENDPOINT_VALIDATED: &str = "EndpointValidated";
    pub const USAGE_PERIOD_QUERY: &str = "UsagePeriodQuery";
    pub const USAGE_PERIOD_RESPONSE: &str = "UsagePeriodResponse";
    pub const USAGE_PERIOD_UPDATE: &str = "UsagePeriodUpdate";

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
    pub const REQUEST_FULL_SNAPSHOT: &str = "RequestFullSnapshot";
    pub const AGENT_PATCH: &str = "AgentPatch";
    pub const AGENT_SNAPSHOT: &str = "AgentSnapshot";
    pub const REQUEST_GLOBAL_SNAPSHOT: &str = "RequestGlobalSnapshot";
    pub const GLOBAL_SNAPSHOT: &str = "GlobalSnapshot";
    pub const MODELS_SNAPSHOT: &str = "ModelsSnapshot";
    pub const PROVIDERS_SNAPSHOT: &str = "ProvidersSnapshot";
    pub const CONTAINER_PATCH: &str = "ContainerPatch";
    pub const REQUEST_CONTAINER_SNAPSHOT: &str = "RequestContainerSnapshot";
    pub const CONTAINER_SNAPSHOT: &str = "ContainerSnapshot";
    pub const TASK_PATCH: &str = "TaskPatch";
    pub const REQUEST_TASKS_SNAPSHOT: &str = "RequestTasksSnapshot";
    pub const TASKS_SNAPSHOT: &str = "TasksSnapshot";
    pub const REQUEST_VM_SNAPSHOT: &str = "RequestVmSnapshot";
    pub const VM_SNAPSHOT: &str = "VmSnapshot";

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
    pub const GET_PROVIDERS_FROM_FS: &str = "GetProvidersFromFs";
    pub const PROVIDERS_FROM_FS_RESPONSE: &str = "ProvidersFromFsResponse";
    pub const GET_MODELS_FROM_FS: &str = "GetModelsFromFs";
    pub const MODELS_FROM_FS_RESPONSE: &str = "ModelsFromFsResponse";
    pub const RELOAD_PROVIDER_CONFIG: &str = "ReloadProviderConfig";
    pub const RELOAD_MODEL_CONFIG: &str = "ReloadModelConfig";
    pub const GET_USER_CONFIG: &str = "GetUserConfig";
    pub const USER_CONFIG_RESPONSE: &str = "UserConfigResponse";
    pub const UPDATE_USER_CONFIG: &str = "UpdateUserConfig";
    pub const RELOAD_USER_CONFIG: &str = "ReloadUserConfig";
    pub const LIST_KEYS: &str = "ListKeys";
    pub const KEYS_LIST_RESPONSE: &str = "KeysListResponse";
    pub const SAVE_API_KEY: &str = "SaveApiKey";
    pub const DELETE_API_KEY: &str = "DeleteApiKey";
    pub const GET_API_KEY_INFO: &str = "GetApiKeyInfo";
    pub const API_KEY_INFO_RESPONSE: &str = "ApiKeyInfoResponse";

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
    pub const CREATE_KNOWLEDGE_BASE: &str = "CreateKnowledgeBase";
    pub const CREATE_KNOWLEDGE_BASE_RESPONSE: &str = "CreateKnowledgeBaseResponse";
    pub const ADD_DOCUMENT: &str = "AddDocument";
    pub const ADD_DOCUMENT_RESPONSE: &str = "AddDocumentResponse";
    pub const QUERY_KNOWLEDGE_BASE: &str = "QueryKnowledgeBase";
    pub const QUERY_KNOWLEDGE_BASE_RESPONSE: &str = "QueryKnowledgeBaseResponse";
    pub const CREATE_SUBSCRIPTION: &str = "CreateSubscription";
    pub const CREATE_SUBSCRIPTION_RESPONSE: &str = "CreateSubscriptionResponse";
    pub const SYNC_SUBSCRIPTION: &str = "SyncSubscription";
    pub const SYNC_SUBSCRIPTION_RESPONSE: &str = "SyncSubscriptionResponse";
    pub const DELETE_SUBSCRIPTION: &str = "DeleteSubscription";
    pub const DELETE_SUBSCRIPTION_RESPONSE: &str = "DeleteSubscriptionResponse";
    pub const GET_KNOWLEDGE_BASE: &str = "GetKnowledgeBase";
    pub const GET_KNOWLEDGE_BASE_RESPONSE: &str = "GetKnowledgeBaseResponse";
    pub const LIST_KNOWLEDGE_BASES: &str = "ListKnowledgeBases";
    pub const LIST_KNOWLEDGE_BASES_RESPONSE: &str = "ListKnowledgeBasesResponse";
    pub const DELETE_KNOWLEDGE_BASE: &str = "DeleteKnowledgeBase";
    pub const DELETE_KNOWLEDGE_BASE_RESPONSE: &str = "DeleteKnowledgeBaseResponse";

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
    pub const OPEN_WORKSPACE: &str = "OpenWorkspace";
    pub const WORKSPACE_STATUS: &str = "WorkspaceStatus";
    pub const REQUEST_WORKSPACE_STATUS: &str = "RequestWorkspaceStatus";

    pub const OPEN_WORKSPACE_RESPONSE: &str = "OpenWorkspaceResponse";
    pub const LIST_POLEMOS_DEVICES: &str = "ListPolemosDevices";
    pub const POLEMOS_DEVICE_LIST: &str = "PolemosDeviceList";
    pub const REGISTER_POLEMOS_DEVICE: &str = "RegisterPolemosDevice";
    pub const REGISTER_POLEMOS_DEVICE_RESPONSE: &str = "RegisterPolemosDeviceResponse";
    pub const SWITCH_WORKSPACE: &str = "SwitchWorkspace";
    pub const SWITCH_WORKSPACE_RESPONSE: &str = "SwitchWorkspaceResponse";
    pub const SET_CLIENT_CWD: &str = "SetClientCwd";
    pub const PUSH_WORKSPACE_FILES: &str = "PushWorkspaceFiles";
    pub const PUSH_WORKSPACE_FILES_ACK: &str = "PushWorkspaceFilesAck";
    pub const REQUEST_WORKSPACE_FILES: &str = "RequestWorkspaceFiles";
    pub const WORKSPACE_READY: &str = "WorkspaceReady";
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
    pub const SYSTEM_MESSAGE: &str = "SystemMessage";
    pub const WEB_UI_CONTROL: &str = "WebUiControl";
    pub const WEB_UI_CONTROL_RESPONSE: &str = "WebUiControlResponse";
    pub const WEB_UI_STATUS: &str = "WebUiStatus";
    pub const REQUEST_WEB_UI_STATUS: &str = "RequestWebUiStatus";

    pub const BADGE_TRANSITION: &str = "BadgeTransition";
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
    pub const AUTH_LOGIN: &str = "AuthLogin";
    pub const AUTH_LOGIN_RESPONSE: &str = "AuthLoginResponse";
    pub const AUTH_REGISTER: &str = "AuthRegister";
    pub const AUTH_REGISTER_RESPONSE: &str = "AuthRegisterResponse";
    pub const AUTH_LIST_USERS: &str = "AuthListUsers";
    pub const AUTH_LIST_USERS_RESPONSE: &str = "AuthListUsersResponse";
    pub const AUTH_GET_USER: &str = "AuthGetUser";
    pub const AUTH_GET_USER_RESPONSE: &str = "AuthGetUserResponse";
    pub const AUTH_DELETE_USER: &str = "AuthDeleteUser";
    pub const AUTH_DELETE_USER_RESPONSE: &str = "AuthDeleteUserResponse";
    pub const AUTH_CHANGE_PASSWORD: &str = "AuthChangePassword";
    pub const AUTH_CHANGE_PASSWORD_RESPONSE: &str = "AuthChangePasswordResponse";
    pub const ARBITER_STATUS: &str = "ArbiterStatus";
    pub const ARBITER_STATUS_RESPONSE: &str = "ArbiterStatusResponse";
    pub const ARBITER_LOCKDOWN: &str = "ArbiterLockdown";
    pub const ARBITER_LOCKDOWN_RESPONSE: &str = "ArbiterLockdownResponse";
    pub const ARBITER_RESTORE: &str = "ArbiterRestore";
    pub const ARBITER_RESTORE_RESPONSE: &str = "ArbiterRestoreResponse";

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

pub mod log_subscription {
    pub const SUBSCRIBE_CONTAINER_LOGS: &str = "SubscribeContainerLogs";
    pub const SUBSCRIBE_CONTAINER_LOGS_RESPONSE: &str = "SubscribeContainerLogsResponse";
    pub const UNSUBSCRIBE_CONTAINER_LOGS: &str = "UnsubscribeContainerLogs";
    pub const UNSUBSCRIBE_CONTAINER_LOGS_RESPONSE: &str = "UnsubscribeContainerLogsResponse";
    pub const CONTAINER_LOG_ENTRY: &str = "ContainerLogEntry";
    pub const SUBSCRIBE_SERVER_LOGS: &str = "SubscribeServerLogs";
    pub const SUBSCRIBE_SERVER_LOGS_RESPONSE: &str = "SubscribeServerLogsResponse";
    pub const UNSUBSCRIBE_SERVER_LOGS: &str = "UnsubscribeServerLogs";
    pub const UNSUBSCRIBE_SERVER_LOGS_RESPONSE: &str = "UnsubscribeServerLogsResponse";
    pub const SERVER_LOG_ENTRY: &str = "ServerLogEntry";

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

pub mod yolo {
    pub const YOLO_START: &str = "YoloStart";
    pub const YOLO_START_RESPONSE: &str = "YoloStartResponse";
    pub const YOLO_STOP: &str = "YoloStop";
    pub const YOLO_STOP_RESPONSE: &str = "YoloStopResponse";
    pub const YOLO_TERMINATE: &str = "YoloTerminate";
    pub const YOLO_TERMINATE_RESPONSE: &str = "YoloTerminateResponse";
    pub const YOLO_STATUS: &str = "YoloStatus";
    pub const YOLO_STATUS_RESPONSE: &str = "YoloStatusResponse";
    pub const YOLO_GET_CONFIG: &str = "YoloGetConfig";
    pub const YOLO_CONFIG_RESPONSE: &str = "YoloConfigResponse";
    pub const YOLO_UPDATE_TASK: &str = "YoloUpdateTask";
    pub const YOLO_UPDATE_TASK_RESPONSE: &str = "YoloUpdateTaskResponse";
    pub const YOLO_SET_TIER_INTERVAL: &str = "YoloSetTierInterval";
    pub const YOLO_SET_TIER_INTERVAL_RESPONSE: &str = "YoloSetTierIntervalResponse";
    pub const YOLO_RUN_TIER_NOW: &str = "YoloRunTierNow";
    pub const YOLO_RUN_TIER_NOW_RESPONSE: &str = "YoloRunTierNowResponse";
    pub const YOLO_CYCLE_STEP: &str = "YoloCycleStep";
    pub const YOLO_CYCLE_COMPLETE: &str = "YoloCycleComplete";
    pub const YOLO_TASK_START: &str = "YoloTaskStart";
    pub const YOLO_TASK_DONE: &str = "YoloTaskDone";
    pub const YOLO_TASK_ERROR: &str = "YoloTaskError";

    pub const SKILL_CHAIN_START: &str = "SkillChainStart";
    pub const SKILL_CHAIN_STEP: &str = "SkillChainStep";
    pub const SKILL_CHAIN_COMPLETE: &str = "SkillChainComplete";
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

pub mod noa {
    pub const REQUEST_NOA_HANDSHAKE: &str = "RequestNoaHandshake";
    pub const NOA_HANDSHAKE_RESPONSE: &str = "NoaHandshakeResponse";
    pub const NOA_AUTH_REQUEST: &str = "NoaAuthRequest";
    pub const NOA_AUTH_RESPONSE: &str = "NoaAuthResponse";
    pub const NOA_READY: &str = "NoaReady";
    pub const NOA_EVENT_SYNC: &str = "NoaEventSync";
    pub const NOA_EVENT_SYNC_ACK: &str = "NoaEventSyncAck";

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

pub mod file {
    pub const REQUEST_FILE_TREE: &str = "RequestFileTree";
    pub const FILE_TREE: &str = "FileTree";
    pub const REQUEST_FILE_READ: &str = "RequestFileRead";
    pub const FILE_READ: &str = "FileRead";

    pub const VARIANTS: &[&str] = &[REQUEST_FILE_TREE, FILE_TREE, REQUEST_FILE_READ, FILE_READ];
}

pub mod bridge {
    pub const REQUEST_BRIDGE_NETWORK: &str = "RequestBridgeNetwork";
    pub const BRIDGE_NETWORK: &str = "BridgeNetwork";

    pub const VARIANTS: &[&str] = &[REQUEST_BRIDGE_NETWORK, BRIDGE_NETWORK];
}

pub mod industrial {
    pub const INDUSTRIAL_TELEMETRY_PUSH: &str = "IndustrialTelemetryPush";
    pub const INDUSTRIAL_ALARM_PUSH: &str = "IndustrialAlarmPush";
    pub const INDUSTRIAL_DISCOVERY_PUSH: &str = "IndustrialDiscoveryPush";
    pub const INDUSTRIAL_WRITE_APPROVAL_PUSH: &str = "IndustrialWriteApprovalPush";

    pub const VARIANTS: &[&str] = &[
        INDUSTRIAL_TELEMETRY_PUSH,
        INDUSTRIAL_ALARM_PUSH,
        INDUSTRIAL_DISCOVERY_PUSH,
        INDUSTRIAL_WRITE_APPROVAL_PUSH,
    ];
}

pub mod search {
    pub const SEARCH_REQUEST: &str = "SearchRequest";
    pub const SEARCH_RESPONSE: &str = "SearchResponse";

    pub const VARIANTS: &[&str] = &[SEARCH_REQUEST, SEARCH_RESPONSE];
}

pub mod conversation {
    pub const CONVERSATION_STARTED: &str = "ConversationStarted";
    pub const REQUEST_RECENT_MESSAGES: &str = "RequestRecentMessages";
    pub const REQUEST_OLDER_MESSAGES: &str = "RequestOlderMessages";
    pub const MESSAGES_RESPONSE: &str = "MessagesResponse";

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
        ]
        .iter()
        .sum();
        assert_eq!(total, 209, "expected 209 grouped variants, got {total}");
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
        all.sort();
        let dupes: Vec<&str> = all
            .windows(2)
            .filter_map(|w| if w[0] == w[1] { Some(w[0]) } else { None })
            .collect();
        assert!(dupes.is_empty(), "duplicate variant names: {dupes:?}");
    }
}
