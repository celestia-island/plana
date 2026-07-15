use std::collections::HashMap;
use serde_json::Value;
use tokio::sync::oneshot;
use uuid::Uuid;

/// Three kinds of JSON-RPC messages in the protocol registry.
///
/// Every known method is registered with its kind.
/// Sync and Async variants always come as `(Request, Response)` pairs
/// where both sides carry a [`Uuid`] for correlation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    /// Fire-and-forget notification — no response expected, no UUID.
    OneWay(&'static str),
    /// Synchronous request-response pair.
    /// Sender calls [`PendingRegistry::request`], awaits the response.
    SyncReq {
        request: &'static str,
        response: &'static str,
    },
    /// Asynchronous request-response pair.
    /// Sender calls [`PendingRegistry::request_async`], gets a handle back.
    AsyncReq {
        request: &'static str,
        response: &'static str,
    },
}

impl MessageKind {
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::OneWay(m) => m,
            Self::SyncReq { request, .. } | Self::AsyncReq { request, .. } => request,
        }
    }

    pub fn is_one_way(&self) -> bool {
        matches!(self, Self::OneWay(_))
    }

    pub fn response_name(&self) -> Option<&'static str> {
        match self {
            Self::OneWay(_) => None,
            Self::SyncReq { response, .. } | Self::AsyncReq { response, .. } => Some(response),
        }
    }
}

/// A pending request handle returned by [`PendingRegistry::request_async`].
///
/// Call [`PendingHandle::wait`] to block until the response arrives
/// (or the registry is dropped without a match).
pub struct PendingHandle {
    pub id: Uuid,
    rx: oneshot::Receiver<Value>,
}

impl PendingHandle {
    pub async fn wait(self) -> Result<Value, oneshot::error::RecvError> {
        self.rx.await
    }
}

/// UUID-based JSON-RPC request/response correlation.
///
/// The registry tracks in-flight requests by [`Uuid`].
/// When a JSON-RPC response frame arrives (matched by `id`),
/// [`on_response`] routes the result to the waiting handle.
///
/// # Usage
///
/// ```ignore
/// let mut reg = PendingRegistry::new();
///
/// // Sync: send request, block until response
/// let result = reg.request("Cli.Status", Value::Null).await?;
///
/// // Async: get a handle, do other work, await later
/// let handle = reg.request_async("Tui.YoloStart", Value::Null);
/// // ... other work ...
/// let result = handle.wait().await?;
///
/// // One-way: just send, no registration
/// reg.prepare_notify("Tui.AgentReport", json!({...}));
/// ```
pub struct PendingRegistry {
    pending: HashMap<Uuid, oneshot::Sender<Value>>,
}

impl PendingRegistry {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Build a JSON-RPC notification frame (no `id` field).
    /// One-way messages do not register in the pending map.
    pub fn prepare_notify(method: &str, params: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        })
    }

    /// Build a JSON-RPC request frame (with `id`), register the UUID,
    /// and return both the frame and a handle that will receive the response.
    ///
    /// This is the **synchronous** path — the caller should `.await` the handle
    /// immediately (or shortly after sending the frame).
    pub fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> (Value, PendingHandle) {
        let (handle, id) = self.register_pending();
        let frame = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id.to_string(),
            "method": method,
            "params": params,
        });
        (frame, handle)
    }

    /// Build a JSON-RPC request frame, register the UUID,
    /// and return both the frame and an async handle.
    ///
    /// This is the **asynchronous** path — the caller sends the frame, does
    /// other work, and calls [`PendingHandle::wait`] when ready.
    pub fn request_async(
        &mut self,
        method: &str,
        params: Value,
    ) -> (Value, PendingHandle) {
        self.request(method, params)
    }

    /// Route an incoming response to its waiting handle.
    ///
    /// Returns `true` if a handle was found and notified, `false` if the
    /// response was unsolicited (no matching request UUID).
    pub fn on_response(&mut self, id: &str, result: Value) -> bool {
        let uuid = match Uuid::parse_str(id) {
            Ok(u) => u,
            Err(_) => return false,
        };
        if let Some(tx) = self.pending.remove(&uuid) {
            let _ = tx.send(result);
            true
        } else {
            false
        }
    }

    /// Route an error response to its waiting handle.
    pub fn on_error(&mut self, id: &str, error: Value) -> bool {
        self.on_response(id, serde_json::json!({ "__error": error }))
    }

    /// How many requests are currently in-flight.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    fn register_pending(&mut self) -> (PendingHandle, Uuid) {
        let id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        (PendingHandle { id, rx }, id)
    }
}

impl Default for PendingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile-time registry of all known JSON-RPC request-response pairs.
///
/// Each entry is a [`MessageKind`] — one-way notifications, sync pairs,
/// or async pairs. This is the **authoritative** list; all three consumer
/// repos (scriptum, shittim-chest, entelecheia) should reference these
/// constants rather than hard-coding method name strings.
#[rustfmt::skip]
pub mod methods {
    use super::MessageKind;

    // ── Handshake ────────────────────────────────────────────────
    pub const SERVER_VERSION: MessageKind       = MessageKind::OneWay("Tui.ServerVersion");
    pub const HANDSHAKE: MessageKind            = MessageKind::SyncReq {
        request: "Tui.ConnectHandshake",
        response: "Tui.HandshakeAck",
    };
    pub const VERSION_MISMATCH: MessageKind     = MessageKind::OneWay("Tui.VersionMismatch");
    pub const SCEPTER_IDENTITY: MessageKind      = MessageKind::OneWay("Tui.ScepterIdentity");

    // ── Heartbeat / Base ─────────────────────────────────────────
    pub const HEARTBEAT: MessageKind            = MessageKind::SyncReq {
        request: "Base.Heartbeat",
        response: "Base.HeartbeatAck",
    };
    pub const BASE_ERROR: MessageKind           = MessageKind::OneWay("Base.Error");
    pub const BASE_ACK: MessageKind             = MessageKind::OneWay("Base.Ack");

    // ── State Sync (server push, one-way) ───────────────────────
    pub const STATE_PATCH: MessageKind          = MessageKind::OneWay("Tui.StatePatch");
    pub const STATE_SNAPSHOT: MessageKind       = MessageKind::OneWay("Tui.StateSnapshot");
    pub const CHANNEL_EVENT: MessageKind        = MessageKind::OneWay("Tui.ChannelEvent");

    // ── Global Snapshot ──────────────────────────────────────────
    pub const REQUEST_GLOBAL_SNAPSHOT: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestGlobalSnapshot",
        response: "Tui.GlobalSnapshot",
    };
    pub const REQUEST_CONTAINER_SNAPSHOT: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestContainerSnapshot",
        response: "Tui.ContainerSnapshot",
    };
    pub const REQUEST_TASKS_SNAPSHOT: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestTasksSnapshot",
        response: "Tui.TasksSnapshot",
    };
    pub const REQUEST_VM_SNAPSHOT: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestVmSnapshot",
        response: "Tui.VmSnapshot",
    };
    pub const REQUEST_FULL_SNAPSHOT: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestFullSnapshot",
        response: "Tui.FullSnapshot",
    };

    // ── Provider / Model Config ─────────────────────────────────
    pub const GET_PROVIDERS_FROM_FS: MessageKind = MessageKind::SyncReq {
        request: "Tui.GetProvidersFromFs",
        response: "Tui.ProvidersFromFsResponse",
    };
    pub const GET_MODELS_FROM_FS: MessageKind = MessageKind::SyncReq {
        request: "Tui.GetModelsFromFs",
        response: "Tui.ModelsFromFsResponse",
    };
    pub const GET_USER_CONFIG: MessageKind = MessageKind::SyncReq {
        request: "Tui.GetUserConfig",
        response: "Tui.UserConfigResponse",
    };
    pub const MODELS_SNAPSHOT: MessageKind       = MessageKind::OneWay("Tui.ModelsSnapshot");
    pub const PROVIDERS_SNAPSHOT: MessageKind    = MessageKind::OneWay("Tui.ProvidersSnapshot");

    // ── Agent Interaction ───────────────────────────────────────
    pub const USER_MESSAGE: MessageKind          = MessageKind::OneWay("Tui.UserMessage");
    pub const AGENT_RESPONSE: MessageKind        = MessageKind::OneWay("Tui.AgentResponse");
    pub const AGENT_STREAMING_CHUNK: MessageKind = MessageKind::OneWay("Tui.AgentStreamingChunk");
    pub const AGENT_THINKING_STEP: MessageKind   = MessageKind::OneWay("Tui.AgentThinkingStep");
    pub const AGENT_REPORT: MessageKind          = MessageKind::OneWay("Tui.AgentReport");
    pub const AGENT_REPORT_REPLY: MessageKind    = MessageKind::OneWay("Tui.AgentReportReply");
    pub const AGENT_TOOL_CALL: MessageKind       = MessageKind::OneWay("Tui.AgentToolCall");
    pub const AGENT_TRANSFER: MessageKind        = MessageKind::OneWay("Tui.AgentTransfer");
    pub const AGENT_PATCH: MessageKind           = MessageKind::OneWay("Tui.AgentPatch");
    pub const AGENT_UPDATE: MessageKind          = MessageKind::OneWay("Tui.AgentUpdate");
    pub const AGENT_LIST_RESPONSE: MessageKind   = MessageKind::OneWay("Tui.AgentListResponse");
    pub const ORCHESTRATION_STATUS: MessageKind  = MessageKind::OneWay("Tui.OrchestrationStatus");
    pub const MCP_TOOL_RESULT: MessageKind       = MessageKind::OneWay("Tui.McpToolResult");
    pub const TASK_CREATED: MessageKind          = MessageKind::OneWay("Tui.TaskCreated");
    pub const TASK_STATUS_UPDATE: MessageKind    = MessageKind::OneWay("Tui.TaskStatusUpdate");
    pub const TASK_PATCH: MessageKind            = MessageKind::OneWay("Tui.TaskPatch");
    pub const CONTAINER_PATCH: MessageKind       = MessageKind::OneWay("Tui.ContainerPatch");

    // ── Ask Human ───────────────────────────────────────────────
    pub const ASK_HUMAN_REQUEST: MessageKind     = MessageKind::OneWay("Tui.AskHumanRequest");
    pub const ASK_HUMAN_REPLY: MessageKind       = MessageKind::AsyncReq {
        request: "Tui.AskHumanReply",
        response: "Tui.AskHumanReplyResponse",
    };
    pub const HUMAN_REVIEW_REQUEST: MessageKind  = MessageKind::OneWay("Tui.HumanReviewRequest");
    pub const HUMAN_REVIEW_RESPONSE: MessageKind = MessageKind::OneWay("Tui.HumanReviewResponse");

    // ── Skill Chain (server push) ───────────────────────────────
    pub const SKILL_CHAIN_START: MessageKind     = MessageKind::OneWay("Tui.SkillChainStart");
    pub const SKILL_CHAIN_STEP: MessageKind      = MessageKind::OneWay("Tui.SkillChainStep");
    pub const SKILL_CHAIN_COMPLETE: MessageKind  = MessageKind::OneWay("Tui.SkillChainComplete");

    // ── YOLO ────────────────────────────────────────────────────
    pub const YOLO_START: MessageKind            = MessageKind::AsyncReq {
        request: "Tui.YoloStart",
        response: "Tui.YoloStartResponse",
    };
    pub const YOLO_STOP: MessageKind             = MessageKind::AsyncReq {
        request: "Tui.YoloStop",
        response: "Tui.YoloStopResponse",
    };
    pub const YOLO_TERMINATE: MessageKind        = MessageKind::AsyncReq {
        request: "Tui.YoloTerminate",
        response: "Tui.YoloTerminateResponse",
    };
    pub const YOLO_STATUS: MessageKind           = MessageKind::AsyncReq {
        request: "Tui.YoloStatus",
        response: "Tui.YoloStatusResponse",
    };
    pub const YOLO_GET_CONFIG: MessageKind       = MessageKind::AsyncReq {
        request: "Tui.YoloGetConfig",
        response: "Tui.YoloConfigResponse",
    };
    pub const YOLO_UPDATE_TASK: MessageKind      = MessageKind::AsyncReq {
        request: "Tui.YoloUpdateTask",
        response: "Tui.YoloUpdateTaskResponse",
    };
    pub const YOLO_SET_TIER_INTERVAL: MessageKind = MessageKind::AsyncReq {
        request: "Tui.YoloSetTierInterval",
        response: "Tui.YoloSetTierIntervalResponse",
    };
    pub const YOLO_RUN_TIER_NOW: MessageKind     = MessageKind::AsyncReq {
        request: "Tui.YoloRunTierNow",
        response: "Tui.YoloRunTierNowResponse",
    };
    pub const YOLO_CYCLE_STEP: MessageKind       = MessageKind::OneWay("Tui.YoloCycleStep");
    pub const YOLO_CYCLE_COMPLETE: MessageKind   = MessageKind::OneWay("Tui.YoloCycleComplete");
    pub const YOLO_TASK_START: MessageKind       = MessageKind::OneWay("Tui.YoloTaskStart");
    pub const YOLO_TASK_DONE: MessageKind        = MessageKind::OneWay("Tui.YoloTaskDone");
    pub const YOLO_TASK_ERROR: MessageKind       = MessageKind::OneWay("Tui.YoloTaskError");

    // ── MCP / Skill ─────────────────────────────────────────────
    pub const MCP_CALL_TOOL: MessageKind         = MessageKind::AsyncReq {
        request: "Mcp.CallTool",
        response: "Mcp.ToolCallResult",
    };
    pub const MCP_LIST_TOOLS: MessageKind        = MessageKind::SyncReq {
        request: "Mcp.ListTools",
        response: "Mcp.ToolsListResponse",
    };
    pub const SKILL_CALL: MessageKind            = MessageKind::AsyncReq {
        request: "Skill.CallSkill",
        response: "Skill.SkillCallResult",
    };
    pub const SKILL_LIST: MessageKind            = MessageKind::SyncReq {
        request: "Skill.ListSkills",
        response: "Skill.SkillsListResponse",
    };

    // ── CLI ─────────────────────────────────────────────────────
    pub const CLI_STATUS: MessageKind            = MessageKind::SyncReq {
        request: "Cli.Status",
        response: "Cli.StatusResponse",
    };
    pub const CLI_CHAT_HISTORY: MessageKind       = MessageKind::SyncReq {
        request: "Cli.ChatHistory",
        response: "Cli.ChatHistoryResponse",
    };
    pub const CLI_TIMELINE_LIST: MessageKind      = MessageKind::SyncReq {
        request: "Cli.TimelineList",
        response: "Cli.TimelineListResponse",
    };
    pub const CLI_TIMELINE_SHOW: MessageKind      = MessageKind::SyncReq {
        request: "Cli.TimelineShow",
        response: "Cli.TimelineShowResponse",
    };
    pub const CLI_RECENT_CHATS: MessageKind       = MessageKind::SyncReq {
        request: "Cli.RecentChats",
        response: "Cli.RecentChatsResponse",
    };
    pub const CLI_SESSION_STATS: MessageKind      = MessageKind::SyncReq {
        request: "Cli.SessionStats",
        response: "Cli.SessionStatsResponse",
    };
    pub const CLI_SESSION_PURGE: MessageKind      = MessageKind::SyncReq {
        request: "Cli.SessionPurge",
        response: "Cli.SessionPurgeResponse",
    };
    pub const CLI_SESSION_VACUUM: MessageKind     = MessageKind::SyncReq {
        request: "Cli.SessionVacuum",
        response: "Cli.SessionVacuumResponse",
    };
    pub const CLI_SEARCH: MessageKind             = MessageKind::SyncReq {
        request: "Cli.Search",
        response: "Cli.SearchResponse",
    };
    pub const CLI_TRACE_CHAIN: MessageKind        = MessageKind::SyncReq {
        request: "Cli.TraceChain",
        response: "Cli.TraceChainResponse",
    };
    pub const CLI_LIST_POLEMOS_DEVICES: MessageKind = MessageKind::SyncReq {
        request: "Cli.ListPolemosDevices",
        response: "Cli.PolemosDeviceListResponse",
    };
    pub const CLI_LIST_TOOLS: MessageKind         = MessageKind::SyncReq {
        request: "Cli.ListTools",
        response: "Cli.ToolsListResponse",
    };
    pub const CLI_LIST_SKILLS: MessageKind        = MessageKind::SyncReq {
        request: "Cli.ListSkills",
        response: "Cli.SkillsListResponse",
    };
    pub const CLI_LIST_WORKSPACES: MessageKind    = MessageKind::SyncReq {
        request: "Cli.ListWorkspaces",
        response: "Cli.WorkspacesListResponse",
    };
    pub const CLI_OPEN_WORKSPACE: MessageKind     = MessageKind::SyncReq {
        request: "Cli.OpenWorkspace",
        response: "Cli.OpenWorkspaceResponse",
    };
    pub const CLI_SWITCH_WORKSPACE: MessageKind   = MessageKind::SyncReq {
        request: "Cli.SwitchWorkspace",
        response: "Cli.SwitchWorkspaceResponse",
    };

    // ── Workspace ───────────────────────────────────────────────
    pub const OPEN_WORKSPACE: MessageKind         = MessageKind::SyncReq {
        request: "Tui.OpenWorkspace",
        response: "Tui.OpenWorkspaceResponse",
    };
    pub const REQUEST_WORKSPACE_STATUS: MessageKind = MessageKind::SyncReq {
        request: "Tui.RequestWorkspaceStatus",
        response: "Tui.WorkspaceStatus",
    };
    pub const LIST_AGENTS: MessageKind            = MessageKind::SyncReq {
        request: "Tui.ListAgents",
        response: "Tui.AgentListResponse",
    };

    // ── Auth ────────────────────────────────────────────────────
    pub const AUTH_LOGIN: MessageKind             = MessageKind::SyncReq {
        request: "Tui.AuthLogin",
        response: "Tui.AuthLoginResponse",
    };
    pub const AUTH_REGISTER: MessageKind          = MessageKind::SyncReq {
        request: "Tui.AuthRegister",
        response: "Tui.AuthRegisterResponse",
    };
    pub const AUTH_LIST_USERS: MessageKind        = MessageKind::SyncReq {
        request: "Tui.AuthListUsers",
        response: "Tui.AuthListUsersResponse",
    };
    pub const AUTH_GET_USER: MessageKind          = MessageKind::SyncReq {
        request: "Tui.AuthGetUser",
        response: "Tui.AuthGetUserResponse",
    };
    pub const AUTH_DELETE_USER: MessageKind       = MessageKind::SyncReq {
        request: "Tui.AuthDeleteUser",
        response: "Tui.AuthDeleteUserResponse",
    };
    pub const AUTH_CHANGE_PASSWORD: MessageKind   = MessageKind::SyncReq {
        request: "Tui.AuthChangePassword",
        response: "Tui.AuthChangePasswordResponse",
    };

    // ── Ping ────────────────────────────────────────────────────
    pub const PING: MessageKind                  = MessageKind::SyncReq {
        request: "Tui.Ping",
        response: "Tui.Pong",
    };

    // ── System ──────────────────────────────────────────────────
    pub const SYSTEM_MESSAGE: MessageKind         = MessageKind::OneWay("Tui.SystemMessage");
    pub const USAGE_PERIOD_QUERY: MessageKind     = MessageKind::SyncReq {
        request: "Tui.UsagePeriodQuery",
        response: "Tui.UsagePeriodResponse",
    };

    // ── Device ──────────────────────────────────────────────────
    pub const DEVICE_POLEMOS_REGISTER: MessageKind = MessageKind::SyncReq {
        request: "Device.PolemosRegister",
        response: "Device.PolemosRegisterAck",
    };
    pub const DEVICE_HEARTBEAT: MessageKind       = MessageKind::SyncReq {
        request: "Device.Heartbeat",
        response: "Device.HeartbeatAck",
    };
    pub const DEVICE_TERMINAL_OPEN: MessageKind   = MessageKind::SyncReq {
        request: "Device.TerminalOpen",
        response: "Device.TerminalReady",
    };
    pub const DEVICE_TERMINAL_INPUT: MessageKind  = MessageKind::OneWay("Device.TerminalInput");
    pub const DEVICE_TERMINAL_RESIZE: MessageKind = MessageKind::OneWay("Device.TerminalResize");
    pub const DEVICE_TERMINAL_POLL: MessageKind   = MessageKind::SyncReq {
        request: "Device.TerminalPoll",
        response: "Device.TerminalPollResult",
    };
    pub const DEVICE_TERMINAL_CLOSE: MessageKind  = MessageKind::SyncReq {
        request: "Device.TerminalClose",
        response: "Device.TerminalCloseAck",
    };
    pub const DEVICE_FILE_LIST: MessageKind       = MessageKind::SyncReq {
        request: "Device.FileList",
        response: "Device.FileListResult",
    };
    pub const DEVICE_FILE_DOWNLOAD: MessageKind   = MessageKind::SyncReq {
        request: "Device.FileDownload",
        response: "Device.FileDownloadResult",
    };
    pub const DEVICE_FILE_UPLOAD: MessageKind     = MessageKind::SyncReq {
        request: "Device.FileUpload",
        response: "Device.FileUploadResult",
    };
    pub const DEVICE_PING: MessageKind            = MessageKind::SyncReq {
        request: "Device.Ping",
        response: "Device.Pong",
    };
    pub const DEVICE_WEBRTC_OFFER: MessageKind    = MessageKind::SyncReq {
        request: "Device.WebrtcOffer",
        response: "Device.WebrtcAnswer",
    };
    pub const DEVICE_WEBRTC_ICE: MessageKind      = MessageKind::OneWay("Device.WebrtcIce");
    pub const DEVICE_SUBSCRIBE_OUTPUT: MessageKind = MessageKind::OneWay("Device.SubscribeOutput");
    pub const DEVICE_TERMINAL_LIST: MessageKind   = MessageKind::SyncReq {
        request: "Device.TerminalList",
        response: "Device.TerminalListResult",
    };

    // ── Screen ──────────────────────────────────────────────────
    pub const SCREEN_OFFER: MessageKind           = MessageKind::OneWay("Screen.Offer");
    pub const SCREEN_ANSWER: MessageKind          = MessageKind::OneWay("Screen.Answer");
    pub const SCREEN_ICE: MessageKind             = MessageKind::OneWay("Screen.Ice");
    pub const SCREEN_ICE_CANDIDATE: MessageKind   = MessageKind::OneWay("Screen.IceCandidate");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_way_no_response_name() {
        assert_eq!(methods::SERVER_VERSION.response_name(), None);
        assert_eq!(methods::USER_MESSAGE.response_name(), None);
        assert!(methods::AGENT_REPORT.is_one_way());
    }

    #[test]
    fn test_sync_req_has_response() {
        let pair = methods::CLI_STATUS;
        assert_eq!(pair.method_name(), "Cli.Status");
        assert_eq!(pair.response_name(), Some("Cli.StatusResponse"));
        assert!(!pair.is_one_way());
    }

    #[test]
    fn test_async_req_has_response() {
        let pair = methods::YOLO_START;
        assert_eq!(pair.method_name(), "Tui.YoloStart");
        assert_eq!(pair.response_name(), Some("Tui.YoloStartResponse"));
        assert!(!pair.is_one_way());
    }

    #[test]
    fn test_handshake_pair() {
        assert_eq!(methods::HANDSHAKE.method_name(), "Tui.ConnectHandshake");
        assert_eq!(methods::HANDSHAKE.response_name(), Some("Tui.HandshakeAck"));
    }

    #[tokio::test]
    async fn test_pending_registry_sync_flow() {
        let mut reg = PendingRegistry::new();
        assert_eq!(reg.pending_count(), 0);

        let (frame, handle) = reg.request("Cli.Status", Value::Null);
        assert_eq!(reg.pending_count(), 1);
        assert!(frame.get("id").is_some());

        let id = frame["id"].as_str().unwrap();
        reg.on_response(id, serde_json::json!({"ok": true}));
        assert_eq!(reg.pending_count(), 0);

        let result = handle.wait().await.unwrap();
        assert_eq!(result, serde_json::json!({"ok": true}));
    }

    #[tokio::test]
    async fn test_pending_registry_async_flow() {
        let mut reg = PendingRegistry::new();

        let (frame1, handle1) = reg.request_async("Tui.YoloStart", Value::Null);
        let (frame2, handle2) = reg.request_async("Tui.YoloStop", Value::Null);
        assert_eq!(reg.pending_count(), 2);

        let id1 = frame1["id"].as_str().unwrap();
        let id2 = frame2["id"].as_str().unwrap();
        assert_ne!(id1, id2);

        // Response arrives for second request first
        reg.on_response(id2, serde_json::json!({"stopped": true}));
        assert_eq!(reg.pending_count(), 1);
        let r2 = handle2.wait().await.unwrap();
        assert_eq!(r2, serde_json::json!({"stopped": true}));

        reg.on_response(id1, serde_json::json!({"started": true}));
        assert_eq!(reg.pending_count(), 0);
        let r1 = handle1.wait().await.unwrap();
        assert_eq!(r1, serde_json::json!({"started": true}));
    }

    #[test]
    fn test_prepare_notify_no_id() {
        let frame = PendingRegistry::prepare_notify("Tui.AgentReport", serde_json::json!({"text": "hi"}));
        assert!(frame.get("id").is_none());
        assert_eq!(frame["method"], "Tui.AgentReport");
    }

    #[test]
    fn test_unsolicited_response_ignored() {
        let mut reg = PendingRegistry::new();
        assert!(!reg.on_response("nonexistent-uuid", Value::Null));
    }
}
