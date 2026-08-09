pub fn agent_update_topic(agent_type: &str) -> String {
    format!("agent.{}.update", agent_type)
}

pub fn agent_complete_topic(agent_type: &str) -> String {
    format!("agent.{}.complete", agent_type)
}

pub fn task_status_topic(task_id: &str) -> String {
    format!("task.{}.status", task_id)
}

pub fn log_entry_topic(source: &str, instance_uuid: Option<&str>) -> String {
    match instance_uuid {
        Some(uuid) => format!("log.{}.{}", source, uuid),
        None => format!("log.{}", source),
    }
}

pub mod tui {
    pub mod ws {
        pub const BATCH_PROCESSED: &str = "tui.ws.batch_processed";
        pub const CONNECTED: &str = "tui.ws.connected";
        pub const DISCONNECTED: &str = "tui.ws.disconnected";
        pub const SERVER_VERSION: &str = "tui.ws.server_version";
        pub const VERSION_MISMATCH: &str = "tui.ws.version_mismatch";
        pub const AGENT_PATCH: &str = "tui.ws.agent_patch";
        pub const AGENT_INFO: &str = "tui.ws.agent_info";
        pub const AGENT_REPORT: &str = "tui.ws.agent_report";
        pub const AGENT_STREAMING_CHUNK: &str = "tui.ws.agent_streaming_chunk";
        pub const AGENT_STREAMING_DONE: &str = "tui.ws.agent_streaming_done";
        pub const STREAMING_TAIL: &str = "tui.ws.streaming_tail";
        pub const ORCHESTRATION_STATUS: &str = "tui.ws.orchestration_status";
        pub const TOOL_RESULT: &str = "tui.ws.mcp_tool_result";
        pub const ASK_HUMAN_REQUEST: &str = "tui.ws.ask_human_request";
        pub const ASK_HUMAN_REPLY: &str = "tui.ws.ask_human_reply";
        pub const SYSTEM_MESSAGE: &str = "tui.ws.system_message";
        pub const TASK_CREATED: &str = "tui.ws.task_created";
        pub const TASK_STATUS_UPDATE: &str = "tui.ws.task_status_update";
        pub const TASK_PATCH: &str = "tui.ws.task_patch";
        pub const SNAPSHOT_SYNC: &str = "tui.ws.snapshot_sync";
        pub const CONTAINER_SNAPSHOT: &str = "tui.ws.container_snapshot";
        pub const TASKS_SNAPSHOT: &str = "tui.ws.tasks_snapshot";
        pub const USAGE_PERIOD_UPDATE: &str = "tui.ws.usage_period_update";
        pub const WEBUI_STATUS: &str = "tui.ws.webui_status";
        pub const PROVIDERS_FROM_FS: &str = "tui.ws.providers_from_fs";
        pub const MODELS_FROM_FS: &str = "tui.ws.models_from_fs";
        pub const USER_CONFIG: &str = "tui.ws.user_config";
        pub const KEYS_LIST: &str = "tui.ws.keys_list";
        pub const API_KEY_INFO: &str = "tui.ws.api_key_info";
        pub const TOOLS_LIST: &str = "tui.ws.tools_list";
        pub const SKILLS_LIST: &str = "tui.ws.skills_list";
        pub const NODE_LIST: &str = "tui.ws.node_list";
        pub const AGENT_LIST: &str = "tui.ws.agent_list";
        pub const AUTO_MODE_UPDATE: &str = "tui.ws.auto_mode_update";
        pub const SCEPTER_IDENTITY: &str = "tui.ws.scepter_identity";
        pub const LLM_PROVIDER_CONFIGURED: &str = "tui.ws.llm_provider_configured";
    }

    pub mod state {
        pub const MEMORY_REFRESH: &str = "tui.state.memory_refresh";
        pub const DEVICE_USAGE_UPDATE: &str = "tui.state.device_usage_update";
        pub const PLATFORM_UPDATE: &str = "tui.state.platform_update";
        pub const CONNECTION_STATE_CHANGED: &str = "tui.state.connection_state_changed";
        pub const OFFLINE_MODAL_CHANGED: &str = "tui.state.offline_modal_changed";
        pub const WEBUI_CHANGED: &str = "tui.state.webui_changed";
    }
}
