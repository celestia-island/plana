//! Tool-name truth source for the domain agents.
//!
//! Every constant here is one agent capability's tool name. The string is the
//! single source of truth: it is what the tool registry registers as a tool's
//! `NAME`, what the LLM sees as the callable function name, and what the
//! Cosmos `exec` sandbox resolves as an ES-module import
//! (`import { report } from 'hubris'; report({ content: ... })` — see
//! `packages/core/src/var_namespace.rs`). Many of these strings are also keys
//! of `res/i18n/locales/*/tools.toml` (the UI labels), but only a subset — a
//! rename must grep that directory rather than assume a label exists.
//!
//! Names are scoped per agent, not globally unique: `web_automation::SCREENSHOT`
//! and `remote_operations::SCREENSHOT` are both `"screenshot"`, and only the
//! agent namespace tells them apart. Under the microkernel architecture an agent
//! may call just three tools directly (`exec`, `write_to_var`,
//! `write_to_var_json` — see `agent_allowed_tools`); every other name in this
//! file is reached from inside `exec` JavaScript, subject to the skill's
//! `[[related_tools]]` allowlist.
//!
//! Most constants have no call site in this repository: the consumers are the
//! downstream services (arona / scepter / evernight / ...) and the skill prompt
//! files, so an unused-looking name here is not dead code.

use anyhow::Result;

/// A tool call parsed out of the compact string form the LLM emits.
///
/// Accepted shapes (see the unit benches): `file_read`, `navigate[2]`,
/// `llm_chat.content`, `navigate[3].content`. Anything that does not match
/// is kept whole in `base_name`, so parsing never loses the raw text.
#[derive(Default)]
pub struct ParsedToolCall {
    /// Tool name with the optional `[tag]` and `.field` suffixes removed.
    pub base_name: String,
    /// Bracketed call index when the emitter disambiguated repeated calls
    /// (`2` for `navigate[2]`); `None` when it was omitted.
    pub call_tag: Option<String>,
    /// Dotted sub-field of the payload (`code` for `exec[0].code`); `None`
    /// means the call targets the whole payload.
    pub field: Option<String>,
}

impl ParsedToolCall {
    /// Parse `raw` into its name / call-tag / field parts.
    ///
    /// Never fails on malformed input — a non-matching string is returned
    /// verbatim as `base_name`. `Err` only if the static regex itself
    /// cannot be compiled.
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

    /// True when the call addresses a sub-field rather than the whole
    /// payload; the streaming layer uses this to decide whether the call
    /// carries incrementally-arriving content.
    pub fn is_content_call(&self) -> bool {
        self.field.is_some()
    }
}

/// ApoRia — storage and LLM hub (Layer 1).
pub mod aporia {
    /// Run one chat completion through ApoRia's LLM hub — the agent-side way
    /// to ask another model.
    pub const LLM_CHAT: &str = "llm_chat";
    /// Persist a document (or its chunks) into the RAG store for later retrieval.
    pub const RAG_DB_WRITE: &str = "rag_db_write";
    /// Query the RAG store and return the matching chunks; read-only, ranked hits.
    pub const RAG_DB_READ: &str = "rag_db_read";
    /// Drop a document and its vectors from the RAG store. Irreversible, so the
    /// collection must be named explicitly rather than inferred.
    pub const RAG_DB_DELETE: &str = "rag_db_delete";
    /// Read RAG store counters (documents/chunks per collection) without returning content.
    pub const RAG_DB_STATS: &str = "rag_db_stats";
    /// Translate a generated report into the requested locale, preserving its section structure.
    pub const TRANSLATE_REPORT: &str = "translate_report";
    /// Scan a metric/series window for anomalies and return the flagged points.
    pub const ANOMALY_DETECT: &str = "anomaly_detect";
    /// Explain an observed outcome by reasoning over the supplied variables and evidence.
    pub const CAUSAL_REASON: &str = "causal_reason";
    /// Build or refresh the searchable index of a workspace's files (the write
    /// side of workspace search).
    pub const WORKSPACE_INDEX: &str = "workspace_index";
    /// Search the index built by `WORKSPACE_INDEX` and return matching files/snippets.
    pub const WORKSPACE_SEARCH: &str = "workspace_search";
    /// Report index freshness/coverage for a workspace — callers use it to decide
    /// whether a search needs a re-index first.
    pub const WORKSPACE_STATUS: &str = "workspace_status";
}

/// SkoPeo — central coordinator (Layer 1).
pub mod skopeo {
    /// Create a goal — the durable objective the coordinator tracks across sessions.
    pub const GOAL_CREATE: &str = "goal_create";
    /// Amend a goal (title, description, progress); the goal id selects the target.
    pub const GOAL_UPDATE: &str = "goal_update";
    /// Close a goal as finished or abandoned; closed goals stay readable but
    /// stop counting as active.
    pub const GOAL_CLOSE: &str = "goal_close";
    /// Enumerate goals with their status — the read side of goal management.
    pub const GOAL_LIST: &str = "goal_list";
    /// Create a track: a monitored workstream that groups goal tasks over time.
    pub const TRACK_CREATE: &str = "track_create";
    /// Update a track's state/metrics without touching the goal it belongs to.
    pub const TRACK_UPDATE: &str = "track_update";
    /// Close a track so no further updates are accepted against it.
    pub const TRACK_CLOSE: &str = "track_close";
    /// Create a goal task — the atomic unit of work under a goal.
    pub const GOAL_TASK_CREATE: &str = "goal_task_create";
    /// Update a goal task's description/owner/dates, leaving its completion state alone.
    pub const GOAL_TASK_UPDATE: &str = "goal_task_update";
    /// Mark a goal task complete. Kept separate from `GOAL_TASK_UPDATE` so
    /// completion is its own auditable event.
    pub const GOAL_TASK_COMPLETE: &str = "goal_task_complete";
    /// List a goal's tasks, typically filtered by status or assignee.
    pub const GOAL_TASK_LIST: &str = "goal_task_list";
    /// Check that goals/tasks still align with the declared OKRs and report the deviations.
    pub const ALIGNMENT_CHECK: &str = "alignment_check";
}

/// HubRis — work planning engine (Layer 1).
pub mod hubris {
    /// Add an item to the plan/todo list the agent maintains for the current task.
    pub const CREATE_TODO: &str = "create_todo";
    /// Read the current todo list (ids, text, completion state).
    pub const LIST_TODO: &str = "list_todo";
    /// Rewrite a todo item's text/priority; the item id selects the target.
    pub const UPDATE_TODO: &str = "update_todo";
    /// Remove a single todo item by id.
    pub const DELETE_TODO: &str = "delete_todo";
    /// Remove every todo item — used when a plan is superseded wholesale.
    pub const CLEAR_TODO: &str = "clear_todo";
    /// Reorder a todo item, moving it within the list.
    pub const MOVE_TODO: &str = "move_todo";
    /// Submit the agent's report payload (`{ content: ... }`) to the runtime;
    /// the result is rendered as an accepted tool result on the timeline.
    pub const REPORT: &str = "report";
    /// Deliver a message to the human operator (shown in the chat UI) — use
    /// when a person must decide or answer, as opposed to `REPORT`.
    pub const REPORT_HUMAN: &str = "report_human";
}

/// KaLos — file operations (Layer 1).
pub mod kalos {
    /// Read a workspace file's contents.
    pub const FILE_READ: &str = "file_read";
    /// Write a file, creating it when absent; the whole content is replaced, not appended.
    pub const FILE_WRITE: &str = "file_write";
    /// Apply a targeted edit to an existing file instead of rewriting it whole.
    pub const FILE_EDIT: &str = "file_edit";
    /// Delete a workspace file.
    pub const FILE_DELETE: &str = "file_delete";
    /// Cheap existence probe — returns a boolean instead of paying for a full read.
    pub const FILE_EXISTS: &str = "file_exists";
    /// List a directory's entries.
    pub const FILE_LIST: &str = "file_list";
    /// Stat a path: size, mtime and type, without reading content.
    pub const FILE_GET_INFO: &str = "file_get_info";
    /// Create a directory (and any missing parents) inside the workspace sandbox.
    pub const FILE_CREATE_DIR: &str = "file_create_dir";
}

/// EleOs — web search and fetch (Layer 1).
pub mod eleos {
    /// Fetch a single URL and return its body as text (no JS execution, no crawl).
    pub const WEB_FETCH: &str = "web_fetch";
    /// Run a web search query and return the ranked results.
    pub const WEB_SEARCH: &str = "web_search";
}

/// Cosmos microkernel tool names — agent-agnostic, injected into every LLM
/// tool set. These three are the only tools an agent calls directly; every
/// other tool name in this file is resolved as an ES import inside `exec`.
pub mod cosmos {
    /// The only code-execution entry point: run JavaScript in the Cosmos
    /// sandbox, where every other tool is an ES-module import.
    pub const EXEC: &str = "exec";
    /// Store a text value in the run's variable namespace so later steps can reference it.
    pub const WRITE_TO_VAR: &str = "write_to_var";
    /// Store a structured JSON value in the variable namespace (kept parsed, not stringified).
    pub const WRITE_TO_VAR_JSON: &str = "write_to_var_json";

    /// Semantics TBC — defined here but read nowhere in this repo (no consumer,
    /// no test). Named like an advisory cap on `exec` payload size, where `0`
    /// would plausibly mean unlimited; confirm with the Cosmos exec router.
    pub const EXEC_CODE_SOFT_LIMIT: usize = 0;
}

/// NeiKos — container management (Layer 1).
pub mod neikos {
    /// Create a container from an image/spec; the returned id is the handle for
    /// every later lifecycle call.
    pub const CONTAINER_CREATE: &str = "container_create";
    /// Start a stopped container.
    pub const CONTAINER_START: &str = "container_start";
    /// Stop a running container, preserving its state for a later start.
    pub const CONTAINER_STOP: &str = "container_stop";
    /// Delete a container and its writable layer — irreversible.
    pub const CONTAINER_REMOVE: &str = "container_remove";
    /// Branch a container into a copy so an experiment cannot disturb the original.
    pub const CONTAINER_FORK: &str = "container_fork";
    /// Capture a container snapshot (state/image) for rollback or reuse.
    pub const CONTAINER_SNAPSHOT: &str = "container_snapshot";
    /// List the containers known to the runtime together with their state.
    pub const CONTAINER_LIST: &str = "container_list";
    /// Inspect one container: configuration, state and mounts.
    pub const CONTAINER_INFO: &str = "container_info";
    /// Run a command inside an existing container and return its output.
    pub const EXEC_ON_CONTAINER: &str = "exec_on_container";
    /// Push a branch to its remote from the container's git workspace.
    pub const GIT_PUSH_BRANCH: &str = "git_push_branch";
    /// List the toolchains available inside a container image.
    pub const TOOLCHAIN_LIST: &str = "toolchain_list";
    /// Ensure a toolchain (rust/node/...) is installed before a build or exec step.
    pub const TOOLCHAIN_ENSURE: &str = "toolchain_ensure";
    /// Start a long-lived sidecar process beside the container (e.g. a dev server).
    pub const SIDECAR_SPAWN: &str = "sidecar_spawn";
    /// Send input to a running sidecar process.
    pub const SIDECAR_SEND: &str = "sidecar_send";
    /// Terminate a sidecar process.
    pub const SIDECAR_KILL: &str = "sidecar_kill";
    /// Block until the awaited operation finishes.
    pub const WAIT: &str = "wait";
    /// Poll a pending wait without blocking, for callers that must stay responsive.
    pub const CHECK_WAIT: &str = "check_wait";
}

/// OreXis — security audit and external integration (Layer 1).
pub mod orexis {
    /// Audit a target against the declared standard.
    pub const STANDARD_CHECK: &str = "standard_check";
    /// Produce the compliance report backing an audit verdict.
    pub const COMPLIANCE_REPORT: &str = "compliance_report";
    /// Audit whether an action or state conforms to the declared plan/policy.
    pub const AUDIT_ALIGNMENT: &str = "audit_alignment";
    /// Audit an action for legal/regulatory acceptability (i18n label: 合法性审计).
    pub const AUDIT_LEGALITY: &str = "audit_legality";
    /// Verify agent integrity — that the running agent and its prompt are the
    /// ones that were loaded, not a tampered version.
    pub const AGENT_INTEGRITY: &str = "agent_integrity";
    /// Run the general security audit over a target.
    pub const SECURITY_AUDIT: &str = "security_audit";
    /// Block a tool for the current scope so later calls are denied before dispatch.
    pub const BLOCK_TOOL: &str = "block_tool";
    /// Lift a previously applied tool block.
    pub const UNBLOCK_TOOL: &str = "unblock_tool";
    /// Replace the active security policy (which tools and scopes are allowed).
    pub const SET_SECURITY_POLICY: &str = "set_security_policy";
    /// Set the risk score above which a call requires human approval.
    pub const SET_RISK_THRESHOLD: &str = "set_risk_threshold";
    /// Inspect a proposed tool call and return its risk verdict before it executes.
    pub const INSPECT_TOOL_CALL: &str = "inspect_tool_call";
    /// Read the current security state: active blocks, policy and pending approvals.
    pub const SECURITY_STATUS: &str = "security_status";
    /// Set the egress/network policy that the agent's tools must obey.
    pub const SET_NETWORK_POLICY: &str = "set_network_policy";
    /// Ask for hardening suggestions derived from the current security posture.
    pub const SECURITY_SUGGESTIONS: &str = "security_suggestions";
    /// Create/update the data-sensitivity rules that classify content for
    /// redaction and access decisions.
    pub const MANAGE_SENSITIVITY_RULES: &str = "manage_sensitivity_rules";
    /// Define an alarm rule: the condition that raises an alarm.
    pub const SET_ALARM_RULE: &str = "set_alarm_rule";
    /// Delete an alarm rule.
    pub const REMOVE_ALARM_RULE: &str = "remove_alarm_rule";
    /// Acknowledge a raised alarm so it stops re-notifying; the record stays.
    pub const ACKNOWLEDGE_ALARM: &str = "acknowledge_alarm";
    /// Read alarm state: what is raised, acknowledged or muted.
    pub const ALARM_STATUS: &str = "alarm_status";
    /// Silence notifications from an alarm source without clearing the alarm.
    pub const ALARM_MUTE: &str = "alarm_mute";
    /// Set the default policy for writes that no specific rule covers.
    pub const SET_DEFAULT_WRITE_POLICY: &str = "set_default_write_policy";
    /// Add a write target (address/register) to the known-safe whitelist.
    pub const WHITELIST_WRITE_ADDRESS: &str = "whitelist_write_address";
    /// Verify a proposed write is safe before it is issued to the device.
    pub const VERIFY_WRITE_SAFETY: &str = "verify_write_safety";
    /// Escalate a write to a human approver when policy requires it.
    pub const REQUEST_WRITE_APPROVAL: &str = "request_write_approval";
    /// Escalate a decision (not necessarily a write) to a human reviewer.
    pub const REQUEST_HUMAN_REVIEW: &str = "request_human_review";
}

/// PhiLia — data storage and system integration (Layer 1).
pub mod philia {
    /// Persist a memory entry for later recall.
    pub const MEMORY_STORE: &str = "memory_store";
    /// Recall stored memories matching a query.
    pub const MEMORY_QUERY: &str = "memory_query";
    /// Consolidate/compact stored memories; the same string is also used as the
    /// `skill` id in Yolo tier-task configs (see `YoloTierTaskConfig` in
    /// plana_state_sync).
    pub const MEMORY_CONSOLIDATE: &str = "memory_consolidate";
    /// Assemble the context payload for the next LLM call (retrieval plus trimming).
    pub const CONTEXT_PREPARE: &str = "context_prepare";
    /// Query a timeseries store over a time range and return the points.
    pub const TIMESERIES_QUERY: &str = "timeseries_query";
    /// Check a dataset for gaps, outliers and schema drift, and report the findings.
    pub const DATA_QUALITY_CHECK: &str = "data_quality_check";
    /// Fetch a tool's JSON schema so a caller can build a valid invocation.
    pub const TOOL_SCHEMA_GET: &str = "tool_schema_get";
}

/// PoleMos — edge computing, hardware and vision (Layer 1). Host inventory
/// probes plus privileged host file/command access on the edge node.
pub mod polemos {
    /// Read host CPU inventory and utilisation.
    pub const CPU_INFO: &str = "cpu_info";
    /// Read host memory inventory and usage.
    pub const MEMORY_INFO: &str = "memory_info";
    /// Read host storage inventory and free space.
    pub const STORAGE_INFO: &str = "storage_info";
    /// List host PCI devices — the discovery step before GPU/NIC
    /// passthrough or driver work.
    pub const PCI_DEVICES: &str = "pci_devices";
    /// Read GPU inventory and state on the edge host.
    pub const GPU_INFO: &str = "gpu_info";
    /// Read a file on the host, i.e. outside the agent's container.
    pub const HOST_FILE_READ: &str = "host_file_read";
    /// Write a file on the host — a privileged escape from the container sandbox.
    pub const HOST_FILE_WRITE: &str = "host_file_write";
    /// Edit a host file in place rather than rewriting it whole.
    pub const HOST_FILE_EDIT: &str = "host_file_edit";
    /// Run a command on the host. Its writes are what the prompt-layer feature
    /// flag tracks (see `packages/prompt/src/features.rs`).
    pub const HOST_COMMAND_EXEC: &str = "host_command_exec";
}

/// HapLotes — communication gateway (Layer 1).
///
/// The webhook family (B3b, user-panel redesign) manages the running
/// agent owner's SCOPED webhook subscriptions on shittim-chest: the
/// scepter service credential reaches chest's `webhook.workspace.*`
/// service family, where chest resolves the real authority from the
/// workspace ownership (the subscription's scope bounds the tools to
/// exactly what the workspace controls). The workspace identity rides
/// the service credential — it is never a tool parameter (an
/// LLM-authored parameter could impersonate any workspace).
pub mod haplotes {
    /// Call a named LLM provider directly, with the provider/model chosen by
    /// the caller rather than the hub default.
    pub const LLM_PROVIDER_CALL: &str = "llm_provider_call";
    /// Subscribe the agent to a trigger so matching events are delivered to it
    /// as tool invocations.
    pub const SUBSCRIBE_TRIGGER: &str = "subscribe_trigger";
    /// Register a webhook subscription. The workspace identity rides the
    /// service credential and is never a tool parameter — an LLM-authored
    /// parameter could impersonate any workspace.
    pub const WEBHOOK_SUBSCRIBE: &str = "webhook_subscribe";
    /// Remove a webhook subscription owned by the acting workspace.
    pub const WEBHOOK_UNSUBSCRIBE: &str = "webhook_unsubscribe";
    /// List the running agent owner's webhook subscriptions.
    pub const WEBHOOK_LIST: &str = "webhook_list";
}

/// Epieikeia — event/message dispatch and async operations (Layer 1).
pub mod epieikeia {
    /// Deliver a message to another agent/session — the dispatch primitive
    /// behind agent-to-agent traffic.
    pub const DELIVER_MESSAGE: &str = "deliver_message";
    /// Queue a synthetic user prompt for the running agent; it takes effect on the next turn.
    pub const INJECT_USER_PROMPT: &str = "inject_user_prompt";
    /// Drain the injected-prompt queue — the counterpart of `INJECT_USER_PROMPT`.
    pub const CONSUME_INJECTED_PROMPTS: &str = "consume_injected_prompts";
    /// Arm a one-shot container fork so the next action runs in a branch,
    /// leaving the original container state untouched.
    pub const FORK_CONTAINER_ON_NEXT_ACTION: &str = "fork_container_on_next_action";
    /// Register interest in a file path so writes to it notify the caller.
    pub const NOTIFY_FILE_OPERATION: &str = "notify_file_operation";
    /// List the file observers currently registered.
    pub const LIST_FILE_OBSERVERS: &str = "list_file_observers";
    /// Remove a file observer.
    pub const UNREGISTER_FILE_OPERATION: &str = "unregister_file_operation";
    /// Discover the hooks available to the agent (the hook registry index).
    pub const DISCOVER_HOOKS: &str = "discover_hooks";
}

/// SkeMma — script execution and microservice runtime (Layer 1).
pub mod skemma {
    /// Execute a script (bash/python/...) in the agent's runtime.
    pub const SCRIPT_EXEC: &str = "script_exec";
    /// Normalise a raw signal (scaling, units, denoise) before it is analysed.
    pub const SIGNAL_NORMALIZE: &str = "signal_normalize";
}

/// Classic Software Engineering — code review, LSP and refactoring (Layer 2).
pub mod classic_software_engineering {
    /// Run static analysis over a codebase and return the findings.
    pub const STATIC_ANALYZE: &str = "static_analyze";
    /// Review a change set and return review comments.
    pub const CODE_REVIEW: &str = "code_review";
    /// Run the project's quality gate (lint/format/tests) and report pass/fail.
    pub const QUALITY_CHECK: &str = "quality_check";
    /// Propose refactorings for a target without applying them.
    pub const REFACTOR_SUGGEST: &str = "refactor_suggest";
    /// Ask the language server for diagnostics on a file or workspace.
    pub const LSP_DIAGNOSE: &str = "lsp_diagnose";
    /// Query the language server's symbol index (definitions, references).
    pub const LSP_SYMBOLS: &str = "lsp_symbols";
    /// Apply an LSP-backed refactor — unlike `REFACTOR_SUGGEST`, this mutates code.
    pub const LSP_REFACTOR: &str = "lsp_refactor";
}

/// Web Automation — browser automation and testing (Layer 2). Session-scoped:
/// `CREATE` opens a browser session and every other tool needs its handle.
pub mod web_automation {
    /// Open a browser session; the returned handle is what every other
    /// web-automation tool operates on. Wire name is the bare `"create"`.
    pub const CREATE: &str = "create";
    /// Close a browser session and release its resources.
    pub const CLOSE: &str = "close";
    /// Point the session at a URL and wait for the page to load.
    pub const NAVIGATE: &str = "navigate";
    /// Capture a screenshot of the current page.
    pub const SCREENSHOT: &str = "screenshot";
    /// Evaluate JavaScript in the page context.
    pub const EXECUTE_SCRIPT: &str = "execute_script";
    /// Read the page's console log buffer.
    pub const GET_CONSOLE_LOGS: &str = "get_console_logs";
    /// Read the page's network request log.
    pub const GET_NETWORK_LOGS: &str = "get_network_logs";
    /// Send a key (with modifiers) to the focused element.
    pub const KEYPRESS: &str = "keypress";
    /// Click at a page coordinate or on an element.
    pub const MOUSE_CLICK: &str = "mouse_click";
    /// Move the pointer to a page coordinate (hover states, drag preparation).
    pub const MOUSE_MOVE: &str = "mouse_move";
    /// Record the session (video/trace) for later inspection.
    pub const RECORD: &str = "record";
}

/// Industrial IoT tool names — domain-specific industrial protocol tools
/// migrated from SkeMma (modbus) and PoleMos (protocol discovery) to this
/// Layer 2 domain agent.
pub mod industrial_iot {
    // From SkeMma
    /// Read Modbus coils/registers from a PLC.
    pub const MODBUS_READ: &str = "modbus_read";
    /// Write Modbus coils/registers — the write path OreXis' approval gate guards.
    pub const MODBUS_WRITE: &str = "modbus_write";

    // From PoleMos
    /// Enumerate the serial ports/interfaces available for protocol probing.
    pub const SERIAL_DISCOVER: &str = "serial_discover";
    /// Discover Siemens S7comm devices.
    pub const S7COMM_DISCOVER: &str = "s7comm_discover";
    /// Fingerprint which industrial protocol a device speaks.
    pub const PROTOCOL_AUTO_DETECT: &str = "protocol_auto_detect";
    /// Probe a device with protocol-specific requests to confirm and
    /// parametrise it.
    pub const PROTOCOL_PROBE: &str = "protocol_probe";
    /// Run a device self-test and return its report.
    pub const DEVICE_SELF_TEST: &str = "device_self_test";
}

/// Remote Operations tool names — Layer 2 remote access agent
/// absorbing SSH, remote terminal, GUI automation, and file transfer tools
/// from SkeMma (6 tools) and PoleMos (10 tools).
pub mod remote_operations {
    // From SkeMma
    /// Open an SSH connection to a remote host and return a session handle.
    pub const CONNECT_REMOTE_VIA_SSH: &str = "connect_remote_via_ssh";
    /// Tear down a remote connection/session.
    pub const DISCONNECT_REMOTE: &str = "disconnect_remote";
    /// Run a command over an established remote session.
    pub const EXEC_ON_REMOTE: &str = "exec_on_remote";
    /// Capture the remote screen. Shares the wire name `"screenshot"` with
    /// `web_automation::SCREENSHOT`; only the agent namespace separates them.
    pub const SCREENSHOT: &str = "screenshot";
    /// Drive the remote pointer — GUI automation on the remote host.
    pub const MOUSE_OPERATE: &str = "mouse_operate";
    /// Send keystrokes to the remote host.
    pub const KEYBOARD_OPERATE: &str = "keyboard_operate";

    // From PoleMos
    /// Discover nodes reachable for remote operations.
    pub const NODE_DISCOVER: &str = "node_discover";
    /// Connect to a discovered node and open the control channel.
    pub const NODE_CONNECT: &str = "node_connect";
    /// Execute a command on a connected node.
    pub const NODE_EXECUTE: &str = "node_execute";
    /// Open an interactive terminal on a node; the returned id is used by the
    /// write/resize/close terminal tools.
    pub const NODE_TERMINAL_OPEN: &str = "node_terminal_open";
    /// Send input to an open node terminal.
    pub const NODE_TERMINAL_WRITE: &str = "node_terminal_write";
    /// Resize a node terminal (the PTY window size).
    pub const NODE_TERMINAL_RESIZE: &str = "node_terminal_resize";
    /// Close a node terminal session.
    pub const NODE_TERMINAL_CLOSE: &str = "node_terminal_close";
    /// List files on a node.
    pub const NODE_FILE_LIST: &str = "node_file_list";
    /// Download a file from a node to the caller.
    pub const NODE_FILE_DOWNLOAD: &str = "node_file_download";
    /// Upload a file to a node.
    pub const NODE_FILE_UPLOAD: &str = "node_file_upload";
    /// Offer the node's screen stream to the operator — the node side of
    /// starting screen sharing.
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
pub fn agent_allowed_tools(_agent: plana_state_sync::Agent) -> &'static [&'static str] {
    &[
        cosmos::EXEC,
        cosmos::WRITE_TO_VAR,
        cosmos::WRITE_TO_VAR_JSON,
    ]
}

/// Tool names `agent` may call directly, as owned strings.
///
/// Thin wrapper over `agent_allowed_tools`, which the registry/skill
/// macro wants as owned `String`s while the allowlist itself is static.
pub fn agent_tools(agent: plana_state_sync::Agent) -> Vec<String> {
    agent_allowed_tools(agent)
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Platform Admin — platform governance (Layer 2).
///
/// RBAC partitioning admin across the engine family, driven
/// conversationally; the per-tool semantics are documented on each
/// constant below.
pub mod platform_admin {
    /// Inspect RBAC state across the engine family (chest grants/groups,
    /// arona groups, evernight node tables) — the read-side fan-out.
    pub const RBAC_INSPECT: &str = "rbac_inspect";
    /// Grant a permission at a scope (global / user-group / workspace)
    /// on chest; workspace-qualified grants carry the workspace uuid.
    pub const RBAC_GRANT: &str = "rbac_grant";
    /// Revoke grants / file denies (the same replace semantics the
    /// chest admin surface applies, with its escalation guards).
    pub const RBAC_REVOKE: &str = "rbac_revoke";
    /// Manage user groups (create / rename / membership) on chest and
    /// arona — the dedicated-group mint high-privilege operators use.
    pub const GROUP_MANAGE: &str = "group_manage";
    /// Allow/deny model access for an arona group (the cloud-resource
    /// partition).
    pub const GROUP_MODELS: &str = "group_models";
    /// Credit top-up / balance for an arona group.
    pub const GROUP_CREDITS: &str = "group_credits";
    /// Grant/revoke on a node's evernight RBAC table (device-plane
    /// authority).
    pub const EVERNIGHT_GRANT: &str = "evernight_grant";
}
