use std::collections::HashMap;
use serde_json::Value;
use tokio::sync::oneshot;
use uuid::Uuid;
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

/// Three kinds of JSON-RPC messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    OneWay,
    SyncReq,
    AsyncReq,
}

// ────────────────────────────────────────────────────────────────────
// Method enum — every JSON-RPC wire name is a variant, powered by strum
// ────────────────────────────────────────────────────────────────────

/// Every known JSON-RPC method as a strum enum.
///
/// The `#[strum(serialize)]` attribute defines the exact wire-format string
/// (e.g. `"Tui.ServerVersion"`).  `Display` and `EnumString` are derived
/// automatically — no scattered `pub const` declarations.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
    Display, EnumString, EnumIter, IntoStaticStr,
)]
#[strum(serialize_all = "PascalCase")]
pub enum Method {
    // ── Handshake ───────────────────────────────────────────
    #[strum(serialize = "Tui.ServerVersion")]
    TuiServerVersion,
    #[strum(serialize = "Tui.ConnectHandshake")]
    TuiConnectHandshake,
    #[strum(serialize = "Tui.HandshakeAck")]
    TuiHandshakeAck,
    #[strum(serialize = "Tui.VersionMismatch")]
    TuiVersionMismatch,
    #[strum(serialize = "Tui.ScepterIdentity")]
    TuiScepterIdentity,

    // ── Heartbeat / Base ────────────────────────────────────
    #[strum(serialize = "Base.Heartbeat")]
    BaseHeartbeat,
    #[strum(serialize = "Base.HeartbeatAck")]
    BaseHeartbeatAck,
    #[strum(serialize = "Base.Error")]
    BaseError,
    #[strum(serialize = "Base.Ack")]
    BaseAck,

    // ── State Sync (server push) ────────────────────────────
    #[strum(serialize = "Tui.StatePatch")]
    TuiStatePatch,
    #[strum(serialize = "Tui.StateSnapshot")]
    TuiStateSnapshot,
    #[strum(serialize = "Tui.ChannelEvent")]
    TuiChannelEvent,

    // ── Global / Container / Task Snapshots ─────────────────
    #[strum(serialize = "Tui.RequestGlobalSnapshot")]
    TuiRequestGlobalSnapshot,
    #[strum(serialize = "Tui.GlobalSnapshot")]
    TuiGlobalSnapshot,
    #[strum(serialize = "Tui.RequestContainerSnapshot")]
    TuiRequestContainerSnapshot,
    #[strum(serialize = "Tui.ContainerSnapshot")]
    TuiContainerSnapshot,
    #[strum(serialize = "Tui.RequestTasksSnapshot")]
    TuiRequestTasksSnapshot,
    #[strum(serialize = "Tui.TasksSnapshot")]
    TuiTasksSnapshot,
    #[strum(serialize = "Tui.RequestVmSnapshot")]
    TuiRequestVmSnapshot,
    #[strum(serialize = "Tui.VmSnapshot")]
    TuiVmSnapshot,
    #[strum(serialize = "Tui.RequestFullSnapshot")]
    TuiRequestFullSnapshot,
    #[strum(serialize = "Tui.FullSnapshot")]
    TuiFullSnapshot,

    // ── Provider / Model Config ─────────────────────────────
    #[strum(serialize = "Tui.GetProvidersFromFs")]
    TuiGetProvidersFromFs,
    #[strum(serialize = "Tui.ProvidersFromFsResponse")]
    TuiProvidersFromFsResponse,
    #[strum(serialize = "Tui.GetModelsFromFs")]
    TuiGetModelsFromFs,
    #[strum(serialize = "Tui.ModelsFromFsResponse")]
    TuiModelsFromFsResponse,
    #[strum(serialize = "Tui.GetUserConfig")]
    TuiGetUserConfig,
    #[strum(serialize = "Tui.UserConfigResponse")]
    TuiUserConfigResponse,
    #[strum(serialize = "Tui.ModelsSnapshot")]
    TuiModelsSnapshot,
    #[strum(serialize = "Tui.ProvidersSnapshot")]
    TuiProvidersSnapshot,

    // ── Agent Interaction ───────────────────────────────────
    #[strum(serialize = "Tui.UserMessage")]
    TuiUserMessage,
    #[strum(serialize = "Tui.AgentResponse")]
    TuiAgentResponse,
    #[strum(serialize = "Tui.AgentStreamingChunk")]
    TuiAgentStreamingChunk,
    #[strum(serialize = "Tui.AgentThinkingStep")]
    TuiAgentThinkingStep,
    #[strum(serialize = "Tui.AgentReport")]
    TuiAgentReport,
    #[strum(serialize = "Tui.AgentReportReply")]
    TuiAgentReportReply,
    #[strum(serialize = "Tui.AgentToolCall")]
    TuiAgentToolCall,
    #[strum(serialize = "Tui.AgentTransfer")]
    TuiAgentTransfer,
    #[strum(serialize = "Tui.AgentPatch")]
    TuiAgentPatch,
    #[strum(serialize = "Tui.AgentUpdate")]
    TuiAgentUpdate,
    #[strum(serialize = "Tui.AgentListResponse")]
    TuiAgentListResponse,
    #[strum(serialize = "Tui.AgentSnapshot")]
    TuiAgentSnapshot,
    #[strum(serialize = "Tui.OrchestrationStatus")]
    TuiOrchestrationStatus,
    #[strum(serialize = "Tui.McpToolResult")]
    TuiMcpToolResult,
    #[strum(serialize = "Tui.TaskCreated")]
    TuiTaskCreated,
    #[strum(serialize = "Tui.TaskStatusUpdate")]
    TuiTaskStatusUpdate,
    #[strum(serialize = "Tui.TaskPatch")]
    TuiTaskPatch,
    #[strum(serialize = "Tui.ContainerPatch")]
    TuiContainerPatch,
    #[strum(serialize = "Tui.SystemMessage")]
    TuiSystemMessage,

    // ── Ask Human ───────────────────────────────────────────
    #[strum(serialize = "Tui.AskHumanRequest")]
    TuiAskHumanRequest,
    #[strum(serialize = "Tui.AskHumanReply")]
    TuiAskHumanReply,
    #[strum(serialize = "Tui.AskHumanReplyResponse")]
    TuiAskHumanReplyResponse,
    #[strum(serialize = "Tui.HumanReviewRequest")]
    TuiHumanReviewRequest,
    #[strum(serialize = "Tui.HumanReviewResponse")]
    TuiHumanReviewResponse,

    // ── Skill Chain ─────────────────────────────────────────
    #[strum(serialize = "Tui.SkillChainStart")]
    TuiSkillChainStart,
    #[strum(serialize = "Tui.SkillChainStep")]
    TuiSkillChainStep,
    #[strum(serialize = "Tui.SkillChainComplete")]
    TuiSkillChainComplete,

    // ── YOLO ────────────────────────────────────────────────
    #[strum(serialize = "Tui.YoloStart")]
    TuiYoloStart,
    #[strum(serialize = "Tui.YoloStartResponse")]
    TuiYoloStartResponse,
    #[strum(serialize = "Tui.YoloStop")]
    TuiYoloStop,
    #[strum(serialize = "Tui.YoloStopResponse")]
    TuiYoloStopResponse,
    #[strum(serialize = "Tui.YoloTerminate")]
    TuiYoloTerminate,
    #[strum(serialize = "Tui.YoloTerminateResponse")]
    TuiYoloTerminateResponse,
    #[strum(serialize = "Tui.YoloStatus")]
    TuiYoloStatus,
    #[strum(serialize = "Tui.YoloStatusResponse")]
    TuiYoloStatusResponse,
    #[strum(serialize = "Tui.YoloGetConfig")]
    TuiYoloGetConfig,
    #[strum(serialize = "Tui.YoloConfigResponse")]
    TuiYoloConfigResponse,
    #[strum(serialize = "Tui.YoloUpdateTask")]
    TuiYoloUpdateTask,
    #[strum(serialize = "Tui.YoloUpdateTaskResponse")]
    TuiYoloUpdateTaskResponse,
    #[strum(serialize = "Tui.YoloSetTierInterval")]
    TuiYoloSetTierInterval,
    #[strum(serialize = "Tui.YoloSetTierIntervalResponse")]
    TuiYoloSetTierIntervalResponse,
    #[strum(serialize = "Tui.YoloRunTierNow")]
    TuiYoloRunTierNow,
    #[strum(serialize = "Tui.YoloRunTierNowResponse")]
    TuiYoloRunTierNowResponse,
    #[strum(serialize = "Tui.YoloCycleStep")]
    TuiYoloCycleStep,
    #[strum(serialize = "Tui.YoloCycleComplete")]
    TuiYoloCycleComplete,
    #[strum(serialize = "Tui.YoloTaskStart")]
    TuiYoloTaskStart,
    #[strum(serialize = "Tui.YoloTaskDone")]
    TuiYoloTaskDone,
    #[strum(serialize = "Tui.YoloTaskError")]
    TuiYoloTaskError,

    // ── MCP / Skill ─────────────────────────────────────────
    #[strum(serialize = "Mcp.CallTool")]
    McpCallTool,
    #[strum(serialize = "Mcp.ToolCallResult")]
    McpToolCallResult,
    #[strum(serialize = "Mcp.ListTools")]
    McpListTools,
    #[strum(serialize = "Mcp.ToolsListResponse")]
    McpToolsListResponse,
    #[strum(serialize = "Skill.CallSkill")]
    SkillCallSkill,
    #[strum(serialize = "Skill.SkillCallResult")]
    SkillSkillCallResult,
    #[strum(serialize = "Skill.ListSkills")]
    SkillListSkills,
    #[strum(serialize = "Skill.SkillsListResponse")]
    SkillSkillsListResponse,

    // ── CLI ─────────────────────────────────────────────────
    #[strum(serialize = "Cli.Status")]
    CliStatus,
    #[strum(serialize = "Cli.ChatHistory")]
    CliChatHistory,
    #[strum(serialize = "Cli.TimelineList")]
    CliTimelineList,
    #[strum(serialize = "Cli.TimelineShow")]
    CliTimelineShow,
    #[strum(serialize = "Cli.RecentChats")]
    CliRecentChats,
    #[strum(serialize = "Cli.ListPolemosDevices")]
    CliListPolemosDevices,
    #[strum(serialize = "Cli.SessionStats")]
    CliSessionStats,
    #[strum(serialize = "Cli.SessionPurge")]
    CliSessionPurge,
    #[strum(serialize = "Cli.SessionVacuum")]
    CliSessionVacuum,
    #[strum(serialize = "Cli.Search")]
    CliSearch,
    #[strum(serialize = "Cli.TraceChain")]
    CliTraceChain,
    #[strum(serialize = "Cli.ListTools")]
    CliListTools,
    #[strum(serialize = "Cli.ListSkills")]
    CliListSkills,
    #[strum(serialize = "Cli.ListWorkspaces")]
    CliListWorkspaces,
    #[strum(serialize = "Cli.OpenWorkspace")]
    CliOpenWorkspace,
    #[strum(serialize = "Cli.SwitchWorkspace")]
    CliSwitchWorkspace,

    // ── Workspace ───────────────────────────────────────────
    #[strum(serialize = "Tui.OpenWorkspace")]
    TuiOpenWorkspace,
    #[strum(serialize = "Tui.OpenWorkspaceResponse")]
    TuiOpenWorkspaceResponse,
    #[strum(serialize = "Tui.RequestWorkspaceStatus")]
    TuiRequestWorkspaceStatus,
    #[strum(serialize = "Tui.WorkspaceStatus")]
    TuiWorkspaceStatus,
    #[strum(serialize = "Tui.ListAgents")]
    TuiListAgents,
    #[strum(serialize = "Tui.PolemosDeviceList")]
    TuiPolemosDeviceList,
    #[strum(serialize = "Tui.ListPolemosDevices")]
    TuiListPolemosDevices,

    // ── Polemos ─────────────────────────────────────────────
    #[strum(serialize = "Tui.RegisterPolemosDevice")]
    TuiRegisterPolemosDevice,
    #[strum(serialize = "Tui.RegisterPolemosDeviceResponse")]
    TuiRegisterPolemosDeviceResponse,
    #[strum(serialize = "Tui.AuthLogin")]
    TuiAuthLogin,
    #[strum(serialize = "Tui.AuthLoginResponse")]
    TuiAuthLoginResponse,
    #[strum(serialize = "Tui.AuthRegister")]
    TuiAuthRegister,
    #[strum(serialize = "Tui.AuthRegisterResponse")]
    TuiAuthRegisterResponse,
    #[strum(serialize = "Tui.AuthListUsers")]
    TuiAuthListUsers,
    #[strum(serialize = "Tui.AuthListUsersResponse")]
    TuiAuthListUsersResponse,
    #[strum(serialize = "Tui.AuthGetUser")]
    TuiAuthGetUser,
    #[strum(serialize = "Tui.AuthGetUserResponse")]
    TuiAuthGetUserResponse,
    #[strum(serialize = "Tui.AuthDeleteUser")]
    TuiAuthDeleteUser,
    #[strum(serialize = "Tui.AuthDeleteUserResponse")]
    TuiAuthDeleteUserResponse,
    #[strum(serialize = "Tui.AuthChangePassword")]
    TuiAuthChangePassword,
    #[strum(serialize = "Tui.AuthChangePasswordResponse")]
    TuiAuthChangePasswordResponse,

    // ── Ping ────────────────────────────────────────────────
    #[strum(serialize = "Tui.Ping")]
    TuiPing,
    #[strum(serialize = "Tui.Pong")]
    TuiPong,

    // ── Usage ───────────────────────────────────────────────
    #[strum(serialize = "Tui.UsagePeriodQuery")]
    TuiUsagePeriodQuery,
    #[strum(serialize = "Tui.UsagePeriodResponse")]
    TuiUsagePeriodResponse,

    // ── Layer2 Domain Agents ────────────────────────────────
    #[strum(serialize = "Tui.Layer2AgentList")]
    TuiLayer2AgentList,
    #[strum(serialize = "Tui.Layer2AgentListResponse")]
    TuiLayer2AgentListResponse,
    #[strum(serialize = "Tui.Layer2AgentMcpTools")]
    TuiLayer2AgentMcpTools,
    #[strum(serialize = "Tui.Layer2AgentMcpResponse")]
    TuiLayer2AgentMcpResponse,
    #[strum(serialize = "Tui.Layer2AgentSkills")]
    TuiLayer2AgentSkills,
    #[strum(serialize = "Tui.Layer2AgentSkillsResponse")]
    TuiLayer2AgentSkillsResponse,

    // ── User Preferences ────────────────────────────────────
    #[strum(serialize = "Tui.GetUserPreferences")]
    TuiGetUserPreferences,
    #[strum(serialize = "Tui.SyncPreferences")]
    TuiSyncPreferences,

    // ── Audio ───────────────────────────────────────────────
    #[strum(serialize = "Tui.AudioPullProgress")]
    TuiAudioPullProgress,
    #[strum(serialize = "Tui.AudioStatusChanged")]
    TuiAudioStatusChanged,

    // ── Device ──────────────────────────────────────────────
    #[strum(serialize = "Device.PolemosRegister")]
    DevicePolemosRegister,
    #[strum(serialize = "Device.PolemosRegisterAck")]
    DevicePolemosRegisterAck,
    #[strum(serialize = "Device.Heartbeat")]
    DeviceHeartbeat,
    #[strum(serialize = "Device.HeartbeatAck")]
    DeviceHeartbeatAck,
    #[strum(serialize = "Device.TerminalOpen")]
    DeviceTerminalOpen,
    #[strum(serialize = "Device.TerminalReady")]
    DeviceTerminalReady,
    #[strum(serialize = "Device.TerminalInput")]
    DeviceTerminalInput,
    #[strum(serialize = "Device.TerminalResize")]
    DeviceTerminalResize,
    #[strum(serialize = "Device.TerminalPoll")]
    DeviceTerminalPoll,
    #[strum(serialize = "Device.TerminalPollResult")]
    DeviceTerminalPollResult,
    #[strum(serialize = "Device.TerminalClose")]
    DeviceTerminalClose,
    #[strum(serialize = "Device.TerminalCloseAck")]
    DeviceTerminalCloseAck,
    #[strum(serialize = "Device.FileList")]
    DeviceFileList,
    #[strum(serialize = "Device.FileListResult")]
    DeviceFileListResult,
    #[strum(serialize = "Device.FileDownload")]
    DeviceFileDownload,
    #[strum(serialize = "Device.FileDownloadResult")]
    DeviceFileDownloadResult,
    #[strum(serialize = "Device.FileUpload")]
    DeviceFileUpload,
    #[strum(serialize = "Device.FileUploadResult")]
    DeviceFileUploadResult,
    #[strum(serialize = "Device.Ping")]
    DevicePing,
    #[strum(serialize = "Device.Pong")]
    DevicePong,
    #[strum(serialize = "Device.WebrtcOffer")]
    DeviceWebrtcOffer,
    #[strum(serialize = "Device.WebrtcAnswer")]
    DeviceWebrtcAnswer,
    #[strum(serialize = "Device.WebrtcIce")]
    DeviceWebrtcIce,
    #[strum(serialize = "Device.SubscribeOutput")]
    DeviceSubscribeOutput,
    #[strum(serialize = "Device.TerminalList")]
    DeviceTerminalList,
    #[strum(serialize = "Device.TerminalOutput")]
    DeviceTerminalOutput,
    #[strum(serialize = "Device.Error")]
    DeviceError,

    // ── Screen ──────────────────────────────────────────────
    #[strum(serialize = "Screen.Offer")]
    ScreenOffer,
    #[strum(serialize = "Screen.Answer")]
    ScreenAnswer,
    #[strum(serialize = "Screen.Ice")]
    ScreenIce,
    #[strum(serialize = "Screen.IceCandidate")]
    ScreenIceCandidate,

    // ── Server Info ─────────────────────────────────────────
    #[strum(serialize = "Tui.ServerInfo")]
    TuiServerInfo,

    // ── Chest-local push notifications ───────────────────────
    #[strum(serialize = "Tui.CrossWorkspaceDenied")]
    TuiCrossWorkspaceDenied,

    // ── Ad-hoc request methods (scepter tui_connection) ──────
    #[strum(serialize = "Tui.RequestBridgeNetwork")]
    TuiRequestBridgeNetwork,
    #[strum(serialize = "Tui.RequestFileTree")]
    TuiRequestFileTree,
    #[strum(serialize = "Tui.RequestFileRead")]
    TuiRequestFileRead,
    #[strum(serialize = "Tui.RequestModelList")]
    TuiRequestModelList,
    #[strum(serialize = "Tui.RequestModelServerAction")]
    TuiRequestModelServerAction,

    // ── Chest-local chunk history ────────────────────────────
    #[strum(serialize = "Tui.AgentChunkRange")]
    TuiAgentChunkRange,
    #[strum(serialize = "Tui.AgentChunkCount")]
    TuiAgentChunkCount,
    #[strum(serialize = "Tui.IndustrialTelemetryPush")]
    TuiIndustrialTelemetryPush,
    #[strum(serialize = "Tui.IndustrialAlarmPush")]
    TuiIndustrialAlarmPush,
    #[strum(serialize = "Tui.IndustrialWriteApprovalPush")]
    TuiIndustrialWriteApprovalPush,
    #[strum(serialize = "Tui.ServerLogEntry")]
    TuiServerLogEntry,
    #[strum(serialize = "Tui.ContainerLogEntry")]
    TuiContainerLogEntry,
}

impl Method {
    /// Wire-format method name (e.g. `"Tui.ServerVersion"`).
    /// Delegates to `Display` (driven by strum).
    pub fn method_name(self) -> &'static str {
        self.into()
    }

    /// Whether this method is a one-way notification (no response expected).
    pub fn is_one_way(self) -> bool {
        self.kind() == MessageKind::OneWay
    }

    /// The MessageKind classification.
    pub fn kind(self) -> MessageKind {
        use Method::*;
        match self {
            // ── One-way server push notifications ──
            TuiServerVersion
            | TuiHandshakeAck
            | TuiVersionMismatch
            | TuiScepterIdentity
            | BaseError | BaseAck | BaseHeartbeatAck
            | TuiStatePatch | TuiStateSnapshot | TuiChannelEvent
            | TuiGlobalSnapshot | TuiContainerSnapshot | TuiTasksSnapshot
            | TuiVmSnapshot | TuiFullSnapshot
            | TuiProvidersFromFsResponse | TuiModelsFromFsResponse | TuiUserConfigResponse
            | TuiModelsSnapshot | TuiProvidersSnapshot
            | TuiAgentResponse | TuiAgentStreamingChunk | TuiAgentThinkingStep
            | TuiAgentReport | TuiAgentReportReply | TuiAgentToolCall | TuiAgentTransfer
            | TuiAgentPatch | TuiAgentUpdate | TuiAgentListResponse | TuiAgentSnapshot
            | TuiOrchestrationStatus | TuiMcpToolResult
            | TuiTaskCreated | TuiTaskStatusUpdate | TuiTaskPatch | TuiContainerPatch
            | TuiSystemMessage
            | TuiAskHumanRequest | TuiHumanReviewRequest | TuiHumanReviewResponse
            | TuiSkillChainStart | TuiSkillChainStep | TuiSkillChainComplete
            | TuiYoloCycleStep | TuiYoloCycleComplete
            | TuiYoloTaskStart | TuiYoloTaskDone | TuiYoloTaskError
            | TuiUserMessage
            | McpToolCallResult | McpToolsListResponse
            | SkillSkillCallResult | SkillSkillsListResponse
            | TuiPolemosDeviceList
            | TuiAudioPullProgress | TuiAudioStatusChanged
            | TuiServerInfo
            | TuiCrossWorkspaceDenied
            | TuiAgentChunkRange | TuiAgentChunkCount
            | TuiIndustrialTelemetryPush | TuiIndustrialAlarmPush
            | TuiIndustrialWriteApprovalPush
            | TuiServerLogEntry | TuiContainerLogEntry
            | DevicePolemosRegisterAck | DeviceHeartbeatAck
            | DeviceTerminalReady | DeviceTerminalInput | DeviceTerminalResize
            | DeviceTerminalPollResult | DeviceTerminalCloseAck
            | DeviceFileListResult | DeviceFileDownloadResult | DeviceFileUploadResult
            | DevicePong | DeviceWebrtcAnswer | DeviceWebrtcIce
            | DeviceSubscribeOutput | DeviceTerminalOutput | DeviceError
            | TuiRegisterPolemosDeviceResponse
            | ScreenOffer | ScreenAnswer | ScreenIce | ScreenIceCandidate => MessageKind::OneWay,

            // ── Sync request-response pairs ──
            TuiConnectHandshake | BaseHeartbeat
            | TuiRequestGlobalSnapshot | TuiRequestContainerSnapshot
            | TuiRequestTasksSnapshot | TuiRequestVmSnapshot | TuiRequestFullSnapshot
            | TuiGetProvidersFromFs | TuiGetModelsFromFs | TuiGetUserConfig
            | McpListTools | SkillListSkills
            | CliStatus | CliChatHistory | CliTimelineList | CliTimelineShow
            | CliRecentChats | CliListPolemosDevices | CliSessionStats | CliSessionPurge
            | CliSessionVacuum | CliSearch | CliTraceChain
            | CliListTools | CliListSkills | CliListWorkspaces | CliOpenWorkspace
            | CliSwitchWorkspace
            | TuiOpenWorkspace | TuiRequestWorkspaceStatus | TuiListAgents
            | TuiListPolemosDevices
            | TuiAuthLogin | TuiAuthRegister | TuiAuthListUsers
            | TuiAuthGetUser | TuiAuthDeleteUser | TuiAuthChangePassword
            | TuiPing
            | TuiUsagePeriodQuery
            | TuiGetUserPreferences | TuiSyncPreferences
            | TuiRequestBridgeNetwork | TuiRequestFileTree | TuiRequestFileRead
            | TuiRequestModelList | TuiRequestModelServerAction
            | TuiRegisterPolemosDevice
            | DevicePolemosRegister | DeviceHeartbeat
            | DeviceTerminalOpen | DeviceTerminalPoll | DeviceTerminalClose
            | DeviceFileList | DeviceFileDownload | DeviceFileUpload
            | DevicePing | DeviceWebrtcOffer | DeviceTerminalList => MessageKind::SyncReq,

            // ── Async request-response pairs ──
            TuiAskHumanReply
            | TuiYoloStart | TuiYoloStop | TuiYoloTerminate
            | TuiYoloStatus | TuiYoloGetConfig | TuiYoloUpdateTask
            | TuiYoloSetTierInterval | TuiYoloRunTierNow
            | McpCallTool | SkillCallSkill
            | TuiLayer2AgentList | TuiLayer2AgentMcpTools | TuiLayer2AgentSkills => MessageKind::AsyncReq,

            // ── Response-only variants (paired with their request) ──
            TuiHandshakeAck => MessageKind::OneWay, // already covered
            TuiPong
            | TuiOpenWorkspaceResponse | TuiWorkspaceStatus
            | TuiYoloStartResponse | TuiYoloStopResponse | TuiYoloTerminateResponse
            | TuiYoloStatusResponse | TuiYoloConfigResponse
            | TuiYoloUpdateTaskResponse | TuiYoloSetTierIntervalResponse
            | TuiYoloRunTierNowResponse
            | TuiAskHumanReplyResponse
            | TuiAuthLoginResponse | TuiAuthRegisterResponse | TuiAuthListUsersResponse
            | TuiAuthGetUserResponse | TuiAuthDeleteUserResponse | TuiAuthChangePasswordResponse
            | TuiUsagePeriodResponse
            | TuiLayer2AgentListResponse | TuiLayer2AgentMcpResponse
            | TuiLayer2AgentSkillsResponse => MessageKind::OneWay,
        }
    }

    /// For a request method, returns the expected response method.
    /// Returns `None` for one-way notifications and response-only methods.
    pub fn response(self) -> Option<Method> {
        use Method::*;
        match self {
            TuiConnectHandshake => Some(TuiHandshakeAck),
            BaseHeartbeat => Some(BaseHeartbeatAck),
            TuiRequestGlobalSnapshot => Some(TuiGlobalSnapshot),
            TuiRequestContainerSnapshot => Some(TuiContainerSnapshot),
            TuiRequestTasksSnapshot => Some(TuiTasksSnapshot),
            TuiRequestVmSnapshot => Some(TuiVmSnapshot),
            TuiRequestFullSnapshot => Some(TuiFullSnapshot),
            TuiGetProvidersFromFs => Some(TuiProvidersFromFsResponse),
            TuiGetModelsFromFs => Some(TuiModelsFromFsResponse),
            TuiGetUserConfig => Some(TuiUserConfigResponse),
            McpListTools => Some(McpToolsListResponse),
            SkillListSkills => Some(SkillSkillsListResponse),
            McpCallTool => Some(McpToolCallResult),
            SkillCallSkill => Some(SkillSkillCallResult),
            CliStatus => Some(TuiPolemosDeviceList), // Cli.Status doesn't have a defined response pair — server returns result directly
            CliSearch => Some(TuiPolemosDeviceList), // placeholders
            CliTraceChain => Some(TuiPolemosDeviceList),
            TuiOpenWorkspace => Some(TuiOpenWorkspaceResponse),
            TuiRequestWorkspaceStatus => Some(TuiWorkspaceStatus),
            TuiListAgents => Some(TuiAgentListResponse),
            TuiPing => Some(TuiPong),
            TuiUsagePeriodQuery => Some(TuiUsagePeriodResponse),
            TuiGetUserPreferences => Some(TuiPolemosDeviceList), // placeholder — response is inline
            TuiSyncPreferences => Some(TuiPolemosDeviceList),
            TuiAskHumanReply => Some(TuiAskHumanReplyResponse),
            TuiYoloStart => Some(TuiYoloStartResponse),
            TuiYoloStop => Some(TuiYoloStopResponse),
            TuiYoloTerminate => Some(TuiYoloTerminateResponse),
            TuiYoloStatus => Some(TuiYoloStatusResponse),
            TuiYoloGetConfig => Some(TuiYoloConfigResponse),
            TuiYoloUpdateTask => Some(TuiYoloUpdateTaskResponse),
            TuiYoloSetTierInterval => Some(TuiYoloSetTierIntervalResponse),
            TuiYoloRunTierNow => Some(TuiYoloRunTierNowResponse),
            TuiAuthLogin => Some(TuiAuthLoginResponse),
            TuiAuthRegister => Some(TuiAuthRegisterResponse),
            TuiAuthListUsers => Some(TuiAuthListUsersResponse),
            TuiAuthGetUser => Some(TuiAuthGetUserResponse),
            TuiAuthDeleteUser => Some(TuiAuthDeleteUserResponse),
            TuiAuthChangePassword => Some(TuiAuthChangePasswordResponse),
            TuiLayer2AgentList => Some(TuiLayer2AgentListResponse),
            TuiLayer2AgentMcpTools => Some(TuiLayer2AgentMcpResponse),
            TuiLayer2AgentSkills => Some(TuiLayer2AgentSkillsResponse),
            TuiRegisterPolemosDevice => Some(TuiRegisterPolemosDeviceResponse),
            DevicePolemosRegister => Some(DevicePolemosRegisterAck),
            DeviceHeartbeat => Some(DeviceHeartbeatAck),
            DeviceTerminalOpen => Some(DeviceTerminalReady),
            DeviceTerminalPoll => Some(DeviceTerminalPollResult),
            DeviceTerminalClose => Some(DeviceTerminalCloseAck),
            DeviceFileList => Some(DeviceFileListResult),
            DeviceFileDownload => Some(DeviceFileDownloadResult),
            DeviceFileUpload => Some(DeviceFileUploadResult),
            DevicePing => Some(DevicePong),
            DeviceWebrtcOffer => Some(DeviceWebrtcAnswer),
            DeviceTerminalList => Some(DeviceFileListResult),
            // Ad-hoc methods — response is inline JSON-RPC result
            TuiRequestBridgeNetwork | TuiRequestFileTree | TuiRequestFileRead
            | TuiRequestModelList | TuiRequestModelServerAction => None,
            _ => None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// PendingRegistry — unchanged from before
// ──────────────────────────────────────────────────────────────

/// A pending request handle.
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
pub struct PendingRegistry {
    pending: HashMap<Uuid, oneshot::Sender<Value>>,
}

impl PendingRegistry {
    pub fn new() -> Self {
        Self { pending: HashMap::new() }
    }

    pub fn prepare_notify(method: Method, params: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": method.method_name(),
            "params": params,
        })
    }

    pub fn request(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        let (handle, id) = self.register_pending();
        let frame = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id.to_string(),
            "method": method.method_name(),
            "params": params,
        });
        (frame, handle)
    }

    pub fn request_async(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        self.request(method, params)
    }

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

    pub fn on_error(&mut self, id: &str, error: Value) -> bool {
        self.on_response(id, serde_json::json!({ "__error": error }))
    }

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
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_every_variant_has_wire_name() {
        for method in Method::iter() {
            let name = method.method_name();
            assert!(!name.is_empty(), "{method:?} has empty wire name");
            assert!(name.contains('.'), "{method:?} missing dot: {name}");
        }
    }

    #[test]
    fn test_roundtrip_display_parse() {
        let name = Method::TuiServerVersion.method_name();
        assert_eq!(name, "Tui.ServerVersion");
        let parsed: Method = name.parse().unwrap();
        assert_eq!(parsed, Method::TuiServerVersion);
    }

    #[test]
    fn test_kind_classification() {
        assert_eq!(Method::TuiServerVersion.kind(), MessageKind::OneWay);
        assert_eq!(Method::TuiConnectHandshake.kind(), MessageKind::SyncReq);
        assert_eq!(Method::TuiYoloStart.kind(), MessageKind::AsyncReq);
    }

    #[test]
    fn test_request_response_pairs() {
        assert_eq!(Method::TuiConnectHandshake.response(), Some(Method::TuiHandshakeAck));
        assert_eq!(Method::TuiPing.response(), Some(Method::TuiPong));
        assert_eq!(Method::TuiYoloStart.response(), Some(Method::TuiYoloStartResponse));
        assert_eq!(Method::TuiServerVersion.response(), None);
    }

    #[test]
    fn test_one_way_has_no_response() {
        assert!(Method::TuiAgentReport.is_one_way());
        assert!(Method::TuiStatePatch.is_one_way());
        assert!(!Method::TuiConnectHandshake.is_one_way());
    }

    #[tokio::test]
    async fn test_pending_registry_sync_flow() {
        let mut reg = PendingRegistry::new();
        let (frame, handle) = reg.request(Method::CliStatus, Value::Null);
        let id = frame["id"].as_str().unwrap();
        reg.on_response(id, serde_json::json!({"ok": true}));
        assert_eq!(handle.wait().await.unwrap(), serde_json::json!({"ok": true}));
    }

    #[tokio::test]
    async fn test_pending_registry_async_flow() {
        let mut reg = PendingRegistry::new();
        let (f1, h1) = reg.request_async(Method::TuiYoloStart, Value::Null);
        let (f2, h2) = reg.request_async(Method::TuiYoloStop, Value::Null);
        assert_eq!(reg.pending_count(), 2);
        let id1 = f1["id"].as_str().unwrap();
        let id2 = f2["id"].as_str().unwrap();
        assert_ne!(id1, id2);
        reg.on_response(id2, serde_json::json!({"stopped":true}));
        assert_eq!(reg.pending_count(), 1);
        assert_eq!(h2.wait().await.unwrap(), serde_json::json!({"stopped":true}));
        reg.on_response(id1, serde_json::json!({"started":true}));
        assert_eq!(h1.wait().await.unwrap(), serde_json::json!({"started":true}));
    }

    #[test]
    fn test_prepare_notify() {
        let frame = PendingRegistry::prepare_notify(Method::TuiAgentReport, serde_json::json!({"text":"hi"}));
        assert!(frame.get("id").is_none());
        assert_eq!(frame["method"], "Tui.AgentReport");
    }

    #[test]
    fn test_unsolicited_response() {
        let mut reg = PendingRegistry::new();
        assert!(!reg.on_response("nonexistent", Value::Null));
    }
}
