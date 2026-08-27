use anyhow::Result;

// Tool name constants

#[derive(Default)]
pub struct ParsedToolCall {
    pub base_name: String,
    pub call_tag: Option<String>,
    pub field: Option<String>,
}

impl ParsedToolCall {
    pub fn parse(raw: &str) -> Result<Self> {
        static TOOL_CALL_REGEX: std::sync::OnceLock<Result<regex::Regex, regex::Error>> =
            std::sync::OnceLock::new();
        let re = TOOL_CALL_REGEX
            .get_or_init(|| regex::Regex::new(r"^([\w:]+?)(?:\[(\d+)\])?(?:\.(\w+))?$"))
            .as_ref()
            .map_err(|e| anyhow::anyhow!("invalid TOOL_CALL_REGEX: {e}"))?;
        if let Some(caps) = re.captures(raw) {
            Ok(Self {
                base_name: caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                call_tag: caps.get(2).map(|m| m.as_str().to_string()),
                field: caps.get(3).map(|m| m.as_str().to_string()),
            })
        } else {
            Ok(Self {
                base_name: raw.to_string(),
                call_tag: None,
                field: None,
            })
        }
    }

    pub fn is_content_call(&self) -> bool {
        self.field.is_some()
    }
}

/// ApoRia tool names
pub mod aporia {
    pub const LLM_CHAT: &str = "llm_chat";
    pub const RAG_DB_WRITE: &str = "rag_db_write";
    pub const RAG_DB_READ: &str = "rag_db_read";
    pub const RAG_DB_DELETE: &str = "rag_db_delete";
    pub const RAG_DB_STATS: &str = "rag_db_stats";
    pub const TRANSLATE_REPORT: &str = "translate_report";
    pub const ANOMALY_DETECT: &str = "anomaly_detect";
    pub const CAUSAL_REASON: &str = "causal_reason";
    pub const WORKSPACE_INDEX: &str = "workspace_index";
    pub const WORKSPACE_SEARCH: &str = "workspace_search";
    pub const WORKSPACE_STATUS: &str = "workspace_status";
}

/// SkoPeo tool names
pub mod skopeo {
    pub const GOAL_CREATE: &str = "goal_create";
    pub const GOAL_UPDATE: &str = "goal_update";
    pub const GOAL_CLOSE: &str = "goal_close";
    pub const GOAL_LIST: &str = "goal_list";
    pub const TRACK_CREATE: &str = "track_create";
    pub const TRACK_UPDATE: &str = "track_update";
    pub const TRACK_CLOSE: &str = "track_close";
    pub const GOAL_TASK_CREATE: &str = "goal_task_create";
    pub const GOAL_TASK_UPDATE: &str = "goal_task_update";
    pub const GOAL_TASK_COMPLETE: &str = "goal_task_complete";
    pub const GOAL_TASK_LIST: &str = "goal_task_list";
    pub const ALIGNMENT_CHECK: &str = "alignment_check";
}

/// HubRis tool names
pub mod hubris {
    pub const CREATE_TODO: &str = "create_todo";
    pub const LIST_TODO: &str = "list_todo";
    pub const UPDATE_TODO: &str = "update_todo";
    pub const DELETE_TODO: &str = "delete_todo";
    pub const CLEAR_TODO: &str = "clear_todo";
    pub const MOVE_TODO: &str = "move_todo";
    pub const REPORT: &str = "report";
    pub const REPORT_HUMAN: &str = "report_human";
}

/// KaLos tool names
pub mod kalos {
    pub const FILE_READ: &str = "file_read";
    pub const FILE_WRITE: &str = "file_write";
    pub const FILE_EDIT: &str = "file_edit";
    pub const FILE_DELETE: &str = "file_delete";
    pub const FILE_EXISTS: &str = "file_exists";
    pub const FILE_LIST: &str = "file_list";
    pub const FILE_GET_INFO: &str = "file_get_info";
    pub const FILE_CREATE_DIR: &str = "file_create_dir";
}

/// EleOs tool names
pub mod eleos {
    pub const WEB_FETCH: &str = "web_fetch";
    pub const WEB_SEARCH: &str = "web_search";
}

/// Cosmos microkernel tool names (injected into LLM tools, agent-agnostic)
pub mod cosmos {
    pub const EXEC: &str = "exec";
    pub const WRITE_TO_VAR: &str = "write_to_var";
    pub const WRITE_TO_VAR_JSON: &str = "write_to_var_json";

    pub const EXEC_CODE_SOFT_LIMIT: usize = 0;
}

/// NeiKos tool names
pub mod neikos {
    pub const CONTAINER_CREATE: &str = "container_create";
    pub const CONTAINER_START: &str = "container_start";
    pub const CONTAINER_STOP: &str = "container_stop";
    pub const CONTAINER_REMOVE: &str = "container_remove";
    pub const CONTAINER_FORK: &str = "container_fork";
    pub const CONTAINER_SNAPSHOT: &str = "container_snapshot";
    pub const CONTAINER_LIST: &str = "container_list";
    pub const CONTAINER_INFO: &str = "container_info";
    pub const EXEC_ON_CONTAINER: &str = "exec_on_container";
    pub const GIT_PUSH_BRANCH: &str = "git_push_branch";
    pub const TOOLCHAIN_LIST: &str = "toolchain_list";
    pub const TOOLCHAIN_ENSURE: &str = "toolchain_ensure";
    pub const SIDECAR_SPAWN: &str = "sidecar_spawn";
    pub const SIDECAR_SEND: &str = "sidecar_send";
    pub const SIDECAR_KILL: &str = "sidecar_kill";
    pub const WAIT: &str = "wait";
    pub const CHECK_WAIT: &str = "check_wait";
}

/// OreXis tool names
pub mod orexis {
    pub const STANDARD_CHECK: &str = "standard_check";
    pub const COMPLIANCE_REPORT: &str = "compliance_report";
    pub const AUDIT_ALIGNMENT: &str = "audit_alignment";
    pub const AUDIT_LEGALITY: &str = "audit_legality";
    pub const AGENT_INTEGRITY: &str = "agent_integrity";
    pub const SECURITY_AUDIT: &str = "security_audit";
    pub const BLOCK_TOOL: &str = "block_tool";
    pub const UNBLOCK_TOOL: &str = "unblock_tool";
    pub const SET_SECURITY_POLICY: &str = "set_security_policy";
    pub const SET_RISK_THRESHOLD: &str = "set_risk_threshold";
    pub const INSPECT_TOOL_CALL: &str = "inspect_tool_call";
    pub const SECURITY_STATUS: &str = "security_status";
    pub const SET_NETWORK_POLICY: &str = "set_network_policy";
    pub const SECURITY_SUGGESTIONS: &str = "security_suggestions";
    pub const MANAGE_SENSITIVITY_RULES: &str = "manage_sensitivity_rules";
    pub const SET_ALARM_RULE: &str = "set_alarm_rule";
    pub const REMOVE_ALARM_RULE: &str = "remove_alarm_rule";
    pub const ACKNOWLEDGE_ALARM: &str = "acknowledge_alarm";
    pub const ALARM_STATUS: &str = "alarm_status";
    pub const ALARM_MUTE: &str = "alarm_mute";
    pub const SET_DEFAULT_WRITE_POLICY: &str = "set_default_write_policy";
    pub const WHITELIST_WRITE_ADDRESS: &str = "whitelist_write_address";
    pub const VERIFY_WRITE_SAFETY: &str = "verify_write_safety";
    pub const REQUEST_WRITE_APPROVAL: &str = "request_write_approval";
    pub const REQUEST_HUMAN_REVIEW: &str = "request_human_review";
}

/// PhiLia tool names
pub mod philia {
    pub const MEMORY_STORE: &str = "memory_store";
    pub const MEMORY_QUERY: &str = "memory_query";
    pub const MEMORY_CONSOLIDATE: &str = "memory_consolidate";
    pub const CONTEXT_PREPARE: &str = "context_prepare";
    pub const TIMESERIES_QUERY: &str = "timeseries_query";
    pub const DATA_QUALITY_CHECK: &str = "data_quality_check";
    pub const TOOL_SCHEMA_GET: &str = "tool_schema_get";
}

/// PoleMos tool names
pub mod polemos {
    pub const CPU_INFO: &str = "cpu_info";
    pub const MEMORY_INFO: &str = "memory_info";
    pub const STORAGE_INFO: &str = "storage_info";
    pub const PCI_DEVICES: &str = "pci_devices";
    pub const GPU_INFO: &str = "gpu_info";
    pub const HOST_FILE_READ: &str = "host_file_read";
    pub const HOST_FILE_WRITE: &str = "host_file_write";
    pub const HOST_FILE_EDIT: &str = "host_file_edit";
    pub const HOST_COMMAND_EXEC: &str = "host_command_exec";
}

/// HapLotes tool names
pub mod haplotes {
    pub const LLM_PROVIDER_CALL: &str = "llm_provider_call";
    pub const SUBSCRIBE_TRIGGER: &str = "subscribe_trigger";
}

/// Epieikeia tool names — event/message dispatch and async operations
pub mod epieikeia {
    pub const DELIVER_MESSAGE: &str = "deliver_message";
    pub const INJECT_USER_PROMPT: &str = "inject_user_prompt";
    pub const CONSUME_INJECTED_PROMPTS: &str = "consume_injected_prompts";
    pub const FORK_CONTAINER_ON_NEXT_ACTION: &str = "fork_container_on_next_action";
    pub const NOTIFY_FILE_OPERATION: &str = "notify_file_operation";
    pub const LIST_FILE_OBSERVERS: &str = "list_file_observers";
    pub const UNREGISTER_FILE_OPERATION: &str = "unregister_file_operation";
    pub const DISCOVER_HOOKS: &str = "discover_hooks";
}

/// SkeMma tool names
pub mod skemma {
    pub const SCRIPT_EXEC: &str = "script_exec";
    pub const SIGNAL_NORMALIZE: &str = "signal_normalize";
}

/// Classic Software Engineering tool names
pub mod classic_software_engineering {
    pub const STATIC_ANALYZE: &str = "static_analyze";
    pub const CODE_REVIEW: &str = "code_review";
    pub const QUALITY_CHECK: &str = "quality_check";
    pub const REFACTOR_SUGGEST: &str = "refactor_suggest";
    pub const LSP_DIAGNOSE: &str = "lsp_diagnose";
    pub const LSP_SYMBOLS: &str = "lsp_symbols";
    pub const LSP_REFACTOR: &str = "lsp_refactor";
}

/// Web Automation tool names
pub mod web_automation {
    pub const CREATE: &str = "create";
    pub const CLOSE: &str = "close";
    pub const NAVIGATE: &str = "navigate";
    pub const SCREENSHOT: &str = "screenshot";
    pub const EXECUTE_SCRIPT: &str = "execute_script";
    pub const GET_CONSOLE_LOGS: &str = "get_console_logs";
    pub const GET_NETWORK_LOGS: &str = "get_network_logs";
    pub const KEYPRESS: &str = "keypress";
    pub const MOUSE_CLICK: &str = "mouse_click";
    pub const MOUSE_MOVE: &str = "mouse_move";
    pub const RECORD: &str = "record";
}

/// Industrial IoT tool names — domain-specific industrial protocol tools
/// migrated from SkeMma (modbus) and PoleMos (protocol discovery) to this
/// Layer 2 domain agent.
pub mod industrial_iot {
    // From SkeMma
    pub const MODBUS_READ: &str = "modbus_read";
    pub const MODBUS_WRITE: &str = "modbus_write";

    // From PoleMos
    pub const SERIAL_DISCOVER: &str = "serial_discover";
    pub const S7COMM_DISCOVER: &str = "s7comm_discover";
    pub const PROTOCOL_AUTO_DETECT: &str = "protocol_auto_detect";
    pub const PROTOCOL_PROBE: &str = "protocol_probe";
    pub const DEVICE_SELF_TEST: &str = "device_self_test";
}

/// Remote Operations tool names — Layer 2 remote access agent
/// absorbing SSH, remote terminal, GUI automation, and file transfer tools
/// from SkeMma (6 tools) and PoleMos (10 tools).
pub mod remote_operations {
    // From SkeMma
    pub const CONNECT_REMOTE_VIA_SSH: &str = "connect_remote_via_ssh";
    pub const DISCONNECT_REMOTE: &str = "disconnect_remote";
    pub const EXEC_ON_REMOTE: &str = "exec_on_remote";
    pub const SCREENSHOT: &str = "screenshot";
    pub const MOUSE_OPERATE: &str = "mouse_operate";
    pub const KEYBOARD_OPERATE: &str = "keyboard_operate";

    // From PoleMos
    pub const NODE_DISCOVER: &str = "node_discover";
    pub const NODE_CONNECT: &str = "node_connect";
    pub const NODE_EXECUTE: &str = "node_execute";
    pub const NODE_TERMINAL_OPEN: &str = "node_terminal_open";
    pub const NODE_TERMINAL_WRITE: &str = "node_terminal_write";
    pub const NODE_TERMINAL_RESIZE: &str = "node_terminal_resize";
    pub const NODE_TERMINAL_CLOSE: &str = "node_terminal_close";
    pub const NODE_FILE_LIST: &str = "node_file_list";
    pub const NODE_FILE_DOWNLOAD: &str = "node_file_download";
    pub const NODE_FILE_UPLOAD: &str = "node_file_upload";
    pub const NODE_SCREEN_OFFER: &str = "node_screen_offer";
}

/// Returns the LLM-visible tool surface for the given agent.
///
/// Under the microkernel architecture, ALL agents expose exactly three tools:
/// `exec`, `write_to_var`, and `write_to_var_json`. All other tools are
/// accessed indirectly through ES-imported tool functions (e.g. `report()`,
/// `file_read()`) inside JavaScript executed by Cosmos's `exec`. Per-skill
/// permission enforcement is handled by the `[[related_tools]]` TOML frontmatter
/// in each skill markdown file and the Cosmos tool router`s `allowed_tools`
/// allowlist.
///
/// If the architecture ever needs to grant specific agents additional
/// direct tool access, add a `match` on `agent` here.
pub fn agent_allowed_tools(_agent: _state_sync::Agent) -> &'static [&'static str] {
    &[
        cosmos::EXEC,
        cosmos::WRITE_TO_VAR,
        cosmos::WRITE_TO_VAR_JSON,
    ]
}

pub fn agent_tools(agent: _state_sync::Agent) -> Vec<String> {
    agent_allowed_tools(agent)
        .iter()
        .map(|s| s.to_string())
        .collect()
}
