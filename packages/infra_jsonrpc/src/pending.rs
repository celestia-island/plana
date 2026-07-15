use std::collections::HashMap;
use serde_json::Value;
use strum::{Display, EnumIter, EnumString};
use tokio::sync::oneshot;
use uuid::Uuid;

// ──────────────────────────────────────────────────────────────
// Macro: define every JSON-RPC method in a single declarative table.
// Each line is:  (Variant, "wire.name", Kind [, ResponseVariant])
//
// Wire strings are hand-written but validated at compile time:
//   - roundtrip: parse(wire) → variant   +   variant.method_name() → wire
//   - pattern:  wire must be "<Namespace>.<CamelCaseAction>"
// ──────────────────────────────────────────────────────────────

macro_rules! define_methods {
    (
        $(
            $(#[$attrs:meta])*
            ($variant:ident, $wire:literal, $kind:ident $(, $response:ident)?)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, EnumIter)]
        pub enum Method {
            $(
                $(#[$attrs])*
                #[strum(serialize = $wire)]
                $variant,
            )*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum MessageKind { OneWay, SyncReq, AsyncReq }

        impl Method {
            pub fn method_name(self) -> &'static str {
                match self { $(Self::$variant => $wire,)* }
            }
            pub fn kind(self) -> MessageKind {
                match self { $(Self::$variant => MessageKind::$kind,)* }
            }
            pub fn is_one_way(self) -> bool {
                matches!(self.kind(), MessageKind::OneWay)
            }
        }
    };
}

// response() — maintained by hand alongside the table above.
// Keep this section in sync with the define_methods!() call.
impl Method {
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
            TuiAskHumanReply => Some(TuiAskHumanReplyResponse),
            TuiYoloStart => Some(TuiYoloStartResponse),
            TuiYoloStop => Some(TuiYoloStopResponse),
            TuiYoloTerminate => Some(TuiYoloTerminateResponse),
            TuiYoloStatus => Some(TuiYoloStatusResponse),
            TuiYoloGetConfig => Some(TuiYoloConfigResponse),
            TuiYoloUpdateTask => Some(TuiYoloUpdateTaskResponse),
            TuiYoloSetTierInterval => Some(TuiYoloSetTierIntervalResponse),
            TuiYoloRunTierNow => Some(TuiYoloRunTierNowResponse),
            McpCallTool => Some(McpToolCallResult),
            McpListTools => Some(McpToolsListResponse),
            SkillCallSkill => Some(SkillSkillCallResult),
            SkillListSkills => Some(SkillSkillsListResponse),
            TuiOpenWorkspace => Some(TuiOpenWorkspaceResponse),
            TuiRequestWorkspaceStatus => Some(TuiWorkspaceStatus),
            TuiListAgents => Some(TuiAgentListResponse),
            TuiRegisterPolemosDevice => Some(TuiRegisterPolemosDeviceResponse),
            TuiAuthLogin => Some(TuiAuthLoginResponse),
            TuiAuthRegister => Some(TuiAuthRegisterResponse),
            TuiAuthListUsers => Some(TuiAuthListUsersResponse),
            TuiAuthGetUser => Some(TuiAuthGetUserResponse),
            TuiAuthDeleteUser => Some(TuiAuthDeleteUserResponse),
            TuiAuthChangePassword => Some(TuiAuthChangePasswordResponse),
            TuiPing => Some(TuiPong),
            TuiUsagePeriodQuery => Some(TuiUsagePeriodResponse),
            TuiLayer2AgentList => Some(TuiLayer2AgentListResponse),
            TuiLayer2AgentMcpTools => Some(TuiLayer2AgentMcpResponse),
            TuiLayer2AgentSkills => Some(TuiLayer2AgentSkillsResponse),
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
            _ => None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// THE TABLE — single source of truth for every method
// ──────────────────────────────────────────────────────────────

define_methods! {
    // ── Handshake ──────────────────────────────────────
    (TuiServerVersion, "Tui.ServerVersion", OneWay),
    (TuiConnectHandshake, "Tui.ConnectHandshake", SyncReq, TuiHandshakeAck),
    (TuiHandshakeAck, "Tui.HandshakeAck", OneWay),
    (TuiVersionMismatch, "Tui.VersionMismatch", OneWay),
    (TuiScepterIdentity, "Tui.ScepterIdentity", OneWay),

    // ── Heartbeat / Base ───────────────────────────────
    (BaseHeartbeat, "Base.Heartbeat", SyncReq, BaseHeartbeatAck),
    (BaseHeartbeatAck, "Base.HeartbeatAck", OneWay),
    (BaseError, "Base.Error", OneWay),
    (BaseAck, "Base.Ack", OneWay),

    // ── State Sync ─────────────────────────────────────
    (TuiStatePatch, "Tui.StatePatch", OneWay),
    (TuiStateSnapshot, "Tui.StateSnapshot", OneWay),
    (TuiChannelEvent, "Tui.ChannelEvent", OneWay),

    // ── Snapshots ──────────────────────────────────────
    (TuiRequestGlobalSnapshot, "Tui.RequestGlobalSnapshot", SyncReq, TuiGlobalSnapshot),
    (TuiGlobalSnapshot, "Tui.GlobalSnapshot", OneWay),
    (TuiRequestContainerSnapshot, "Tui.RequestContainerSnapshot", SyncReq, TuiContainerSnapshot),
    (TuiContainerSnapshot, "Tui.ContainerSnapshot", OneWay),
    (TuiRequestTasksSnapshot, "Tui.RequestTasksSnapshot", SyncReq, TuiTasksSnapshot),
    (TuiTasksSnapshot, "Tui.TasksSnapshot", OneWay),
    (TuiRequestVmSnapshot, "Tui.RequestVmSnapshot", SyncReq, TuiVmSnapshot),
    (TuiVmSnapshot, "Tui.VmSnapshot", OneWay),
    (TuiRequestFullSnapshot, "Tui.RequestFullSnapshot", SyncReq, TuiFullSnapshot),
    (TuiFullSnapshot, "Tui.FullSnapshot", OneWay),

    // ── Provider / Model Config ────────────────────────
    (TuiGetProvidersFromFs, "Tui.GetProvidersFromFs", SyncReq, TuiProvidersFromFsResponse),
    (TuiProvidersFromFsResponse, "Tui.ProvidersFromFsResponse", OneWay),
    (TuiGetModelsFromFs, "Tui.GetModelsFromFs", SyncReq, TuiModelsFromFsResponse),
    (TuiModelsFromFsResponse, "Tui.ModelsFromFsResponse", OneWay),
    (TuiGetUserConfig, "Tui.GetUserConfig", SyncReq, TuiUserConfigResponse),
    (TuiUserConfigResponse, "Tui.UserConfigResponse", OneWay),
    (TuiModelsSnapshot, "Tui.ModelsSnapshot", OneWay),
    (TuiProvidersSnapshot, "Tui.ProvidersSnapshot", OneWay),

    // ── Agent Interaction ──────────────────────────────
    (TuiUserMessage, "Tui.UserMessage", OneWay),
    (TuiAgentResponse, "Tui.AgentResponse", OneWay),
    (TuiAgentStreamingChunk, "Tui.AgentStreamingChunk", OneWay),
    (TuiAgentThinkingStep, "Tui.AgentThinkingStep", OneWay),
    (TuiAgentReport, "Tui.AgentReport", OneWay),
    (TuiAgentReportReply, "Tui.AgentReportReply", OneWay),
    (TuiAgentToolCall, "Tui.AgentToolCall", OneWay),
    (TuiAgentTransfer, "Tui.AgentTransfer", OneWay),
    (TuiAgentPatch, "Tui.AgentPatch", OneWay),
    (TuiAgentUpdate, "Tui.AgentUpdate", OneWay),
    (TuiAgentListResponse, "Tui.AgentListResponse", OneWay),
    (TuiAgentSnapshot, "Tui.AgentSnapshot", OneWay),
    (TuiOrchestrationStatus, "Tui.OrchestrationStatus", OneWay),
    (TuiMcpToolResult, "Tui.McpToolResult", OneWay),
    (TuiTaskCreated, "Tui.TaskCreated", OneWay),
    (TuiTaskStatusUpdate, "Tui.TaskStatusUpdate", OneWay),
    (TuiTaskPatch, "Tui.TaskPatch", OneWay),
    (TuiContainerPatch, "Tui.ContainerPatch", OneWay),
    (TuiSystemMessage, "Tui.SystemMessage", OneWay),

    // ── Ask Human ──────────────────────────────────────
    (TuiAskHumanRequest, "Tui.AskHumanRequest", OneWay),
    (TuiAskHumanReply, "Tui.AskHumanReply", AsyncReq, TuiAskHumanReplyResponse),
    (TuiAskHumanReplyResponse, "Tui.AskHumanReplyResponse", OneWay),
    (TuiHumanReviewRequest, "Tui.HumanReviewRequest", OneWay),
    (TuiHumanReviewResponse, "Tui.HumanReviewResponse", OneWay),

    // ── Skill Chain ────────────────────────────────────
    (TuiSkillChainStart, "Tui.SkillChainStart", OneWay),
    (TuiSkillChainStep, "Tui.SkillChainStep", OneWay),
    (TuiSkillChainComplete, "Tui.SkillChainComplete", OneWay),

    // ── YOLO ───────────────────────────────────────────
    (TuiYoloStart, "Tui.YoloStart", AsyncReq, TuiYoloStartResponse),
    (TuiYoloStartResponse, "Tui.YoloStartResponse", OneWay),
    (TuiYoloStop, "Tui.YoloStop", AsyncReq, TuiYoloStopResponse),
    (TuiYoloStopResponse, "Tui.YoloStopResponse", OneWay),
    (TuiYoloTerminate, "Tui.YoloTerminate", AsyncReq, TuiYoloTerminateResponse),
    (TuiYoloTerminateResponse, "Tui.YoloTerminateResponse", OneWay),
    (TuiYoloStatus, "Tui.YoloStatus", AsyncReq, TuiYoloStatusResponse),
    (TuiYoloStatusResponse, "Tui.YoloStatusResponse", OneWay),
    (TuiYoloGetConfig, "Tui.YoloGetConfig", AsyncReq, TuiYoloConfigResponse),
    (TuiYoloConfigResponse, "Tui.YoloConfigResponse", OneWay),
    (TuiYoloUpdateTask, "Tui.YoloUpdateTask", AsyncReq, TuiYoloUpdateTaskResponse),
    (TuiYoloUpdateTaskResponse, "Tui.YoloUpdateTaskResponse", OneWay),
    (TuiYoloSetTierInterval, "Tui.YoloSetTierInterval", AsyncReq, TuiYoloSetTierIntervalResponse),
    (TuiYoloSetTierIntervalResponse, "Tui.YoloSetTierIntervalResponse", OneWay),
    (TuiYoloRunTierNow, "Tui.YoloRunTierNow", AsyncReq, TuiYoloRunTierNowResponse),
    (TuiYoloRunTierNowResponse, "Tui.YoloRunTierNowResponse", OneWay),
    (TuiYoloCycleStep, "Tui.YoloCycleStep", OneWay),
    (TuiYoloCycleComplete, "Tui.YoloCycleComplete", OneWay),
    (TuiYoloTaskStart, "Tui.YoloTaskStart", OneWay),
    (TuiYoloTaskDone, "Tui.YoloTaskDone", OneWay),
    (TuiYoloTaskError, "Tui.YoloTaskError", OneWay),

    // ── MCP / Skill ────────────────────────────────────
    (McpCallTool, "Mcp.CallTool", AsyncReq, McpToolCallResult),
    (McpToolCallResult, "Mcp.ToolCallResult", OneWay),
    (McpListTools, "Mcp.ListTools", SyncReq, McpToolsListResponse),
    (McpToolsListResponse, "Mcp.ToolsListResponse", OneWay),
    (SkillCallSkill, "Skill.CallSkill", AsyncReq, SkillSkillCallResult),
    (SkillSkillCallResult, "Skill.SkillCallResult", OneWay),
    (SkillListSkills, "Skill.ListSkills", SyncReq, SkillSkillsListResponse),
    (SkillSkillsListResponse, "Skill.SkillsListResponse", OneWay),

    // ── CLI ────────────────────────────────────────────
    (CliStatus, "Cli.Status", SyncReq),
    (CliChatHistory, "Cli.ChatHistory", SyncReq),
    (CliTimelineList, "Cli.TimelineList", SyncReq),
    (CliTimelineShow, "Cli.TimelineShow", SyncReq),
    (CliRecentChats, "Cli.RecentChats", SyncReq),
    (CliListPolemosDevices, "Cli.ListPolemosDevices", SyncReq),
    (CliSessionStats, "Cli.SessionStats", SyncReq),
    (CliSessionPurge, "Cli.SessionPurge", SyncReq),
    (CliSessionVacuum, "Cli.SessionVacuum", SyncReq),
    (CliSearch, "Cli.Search", SyncReq),
    (CliTraceChain, "Cli.TraceChain", SyncReq),
    (CliListTools, "Cli.ListTools", SyncReq),
    (CliListSkills, "Cli.ListSkills", SyncReq),
    (CliListWorkspaces, "Cli.ListWorkspaces", SyncReq),
    (CliOpenWorkspace, "Cli.OpenWorkspace", SyncReq),
    (CliSwitchWorkspace, "Cli.SwitchWorkspace", SyncReq),

    // ── Workspace ──────────────────────────────────────
    (TuiOpenWorkspace, "Tui.OpenWorkspace", SyncReq, TuiOpenWorkspaceResponse),
    (TuiOpenWorkspaceResponse, "Tui.OpenWorkspaceResponse", OneWay),
    (TuiRequestWorkspaceStatus, "Tui.RequestWorkspaceStatus", SyncReq, TuiWorkspaceStatus),
    (TuiWorkspaceStatus, "Tui.WorkspaceStatus", OneWay),
    (TuiListAgents, "Tui.ListAgents", SyncReq, TuiAgentListResponse),
    (TuiPolemosDeviceList, "Tui.PolemosDeviceList", OneWay),
    (TuiListPolemosDevices, "Tui.ListPolemosDevices", SyncReq),

    // ── Polemos ────────────────────────────────────────
    (TuiRegisterPolemosDevice, "Tui.RegisterPolemosDevice", SyncReq, TuiRegisterPolemosDeviceResponse),
    (TuiRegisterPolemosDeviceResponse, "Tui.RegisterPolemosDeviceResponse", OneWay),

    // ── Auth ───────────────────────────────────────────
    (TuiAuthLogin, "Tui.AuthLogin", SyncReq, TuiAuthLoginResponse),
    (TuiAuthLoginResponse, "Tui.AuthLoginResponse", OneWay),
    (TuiAuthRegister, "Tui.AuthRegister", SyncReq, TuiAuthRegisterResponse),
    (TuiAuthRegisterResponse, "Tui.AuthRegisterResponse", OneWay),
    (TuiAuthListUsers, "Tui.AuthListUsers", SyncReq, TuiAuthListUsersResponse),
    (TuiAuthListUsersResponse, "Tui.AuthListUsersResponse", OneWay),
    (TuiAuthGetUser, "Tui.AuthGetUser", SyncReq, TuiAuthGetUserResponse),
    (TuiAuthGetUserResponse, "Tui.AuthGetUserResponse", OneWay),
    (TuiAuthDeleteUser, "Tui.AuthDeleteUser", SyncReq, TuiAuthDeleteUserResponse),
    (TuiAuthDeleteUserResponse, "Tui.AuthDeleteUserResponse", OneWay),
    (TuiAuthChangePassword, "Tui.AuthChangePassword", SyncReq, TuiAuthChangePasswordResponse),
    (TuiAuthChangePasswordResponse, "Tui.AuthChangePasswordResponse", OneWay),

    // ── Ping ───────────────────────────────────────────
    (TuiPing, "Tui.Ping", SyncReq, TuiPong),
    (TuiPong, "Tui.Pong", OneWay),

    // ── Usage ──────────────────────────────────────────
    (TuiUsagePeriodQuery, "Tui.UsagePeriodQuery", SyncReq, TuiUsagePeriodResponse),
    (TuiUsagePeriodResponse, "Tui.UsagePeriodResponse", OneWay),

    // ── Layer2 ─────────────────────────────────────────
    (TuiLayer2AgentList, "Tui.Layer2AgentList", AsyncReq, TuiLayer2AgentListResponse),
    (TuiLayer2AgentListResponse, "Tui.Layer2AgentListResponse", OneWay),
    (TuiLayer2AgentMcpTools, "Tui.Layer2AgentMcpTools", AsyncReq, TuiLayer2AgentMcpResponse),
    (TuiLayer2AgentMcpResponse, "Tui.Layer2AgentMcpResponse", OneWay),
    (TuiLayer2AgentSkills, "Tui.Layer2AgentSkills", AsyncReq, TuiLayer2AgentSkillsResponse),
    (TuiLayer2AgentSkillsResponse, "Tui.Layer2AgentSkillsResponse", OneWay),

    // ── User Preferences ───────────────────────────────
    (TuiGetUserPreferences, "Tui.GetUserPreferences", SyncReq),
    (TuiSyncPreferences, "Tui.SyncPreferences", SyncReq),

    // ── Audio ──────────────────────────────────────────
    (TuiAudioPullProgress, "Tui.AudioPullProgress", OneWay),
    (TuiAudioStatusChanged, "Tui.AudioStatusChanged", OneWay),

    // ── Device ─────────────────────────────────────────
    (DevicePolemosRegister, "Device.PolemosRegister", SyncReq, DevicePolemosRegisterAck),
    (DevicePolemosRegisterAck, "Device.PolemosRegisterAck", OneWay),
    (DeviceHeartbeat, "Device.Heartbeat", SyncReq, DeviceHeartbeatAck),
    (DeviceHeartbeatAck, "Device.HeartbeatAck", OneWay),
    (DeviceTerminalOpen, "Device.TerminalOpen", SyncReq, DeviceTerminalReady),
    (DeviceTerminalReady, "Device.TerminalReady", OneWay),
    (DeviceTerminalInput, "Device.TerminalInput", OneWay),
    (DeviceTerminalResize, "Device.TerminalResize", OneWay),
    (DeviceTerminalPoll, "Device.TerminalPoll", SyncReq, DeviceTerminalPollResult),
    (DeviceTerminalPollResult, "Device.TerminalPollResult", OneWay),
    (DeviceTerminalClose, "Device.TerminalClose", SyncReq, DeviceTerminalCloseAck),
    (DeviceTerminalCloseAck, "Device.TerminalCloseAck", OneWay),
    (DeviceFileList, "Device.FileList", SyncReq, DeviceFileListResult),
    (DeviceFileListResult, "Device.FileListResult", OneWay),
    (DeviceFileDownload, "Device.FileDownload", SyncReq, DeviceFileDownloadResult),
    (DeviceFileDownloadResult, "Device.FileDownloadResult", OneWay),
    (DeviceFileUpload, "Device.FileUpload", SyncReq, DeviceFileUploadResult),
    (DeviceFileUploadResult, "Device.FileUploadResult", OneWay),
    (DevicePing, "Device.Ping", SyncReq, DevicePong),
    (DevicePong, "Device.Pong", OneWay),
    (DeviceWebrtcOffer, "Device.WebrtcOffer", SyncReq, DeviceWebrtcAnswer),
    (DeviceWebrtcAnswer, "Device.WebrtcAnswer", OneWay),
    (DeviceWebrtcIce, "Device.WebrtcIce", OneWay),
    (DeviceSubscribeOutput, "Device.SubscribeOutput", OneWay),
    (DeviceTerminalList, "Device.TerminalList", SyncReq),
    (DeviceTerminalOutput, "Device.TerminalOutput", OneWay),
    (DeviceError, "Device.Error", OneWay),

    // ── Screen ─────────────────────────────────────────
    (ScreenOffer, "Screen.Offer", OneWay),
    (ScreenAnswer, "Screen.Answer", OneWay),
    (ScreenIce, "Screen.Ice", OneWay),
    (ScreenIceCandidate, "Screen.IceCandidate", OneWay),

    // ── Server / Chest-local ───────────────────────────
    (TuiServerInfo, "Tui.ServerInfo", OneWay),
    (TuiCrossWorkspaceDenied, "Tui.CrossWorkspaceDenied", OneWay),
    (TuiRequestBridgeNetwork, "Tui.RequestBridgeNetwork", SyncReq),
    (TuiRequestFileTree, "Tui.RequestFileTree", SyncReq),
    (TuiRequestFileRead, "Tui.RequestFileRead", SyncReq),
    (TuiRequestModelList, "Tui.RequestModelList", SyncReq),
    (TuiRequestModelServerAction, "Tui.RequestModelServerAction", SyncReq),
    (TuiAgentChunkRange, "Tui.AgentChunkRange", OneWay),
    (TuiAgentChunkCount, "Tui.AgentChunkCount", OneWay),
    (TuiIndustrialTelemetryPush, "Tui.IndustrialTelemetryPush", OneWay),
    (TuiIndustrialAlarmPush, "Tui.IndustrialAlarmPush", OneWay),
    (TuiIndustrialWriteApprovalPush, "Tui.IndustrialWriteApprovalPush", OneWay),
    (TuiServerLogEntry, "Tui.ServerLogEntry", OneWay),
    (TuiContainerLogEntry, "Tui.ContainerLogEntry", OneWay),
}

// ──────────────────────────────────────────────────────────────
// PendingRegistry — UUID-based JSON-RPC correlation
// ──────────────────────────────────────────────────────────────

pub struct PendingHandle {
    pub id: Uuid,
    rx: oneshot::Receiver<Value>,
}

impl PendingHandle {
    pub async fn wait(self) -> Result<Value, oneshot::error::RecvError> { self.rx.await }
}

pub struct PendingRegistry {
    pending: HashMap<Uuid, oneshot::Sender<Value>>,
}

impl PendingRegistry {
    pub fn new() -> Self { Self { pending: HashMap::new() } }

    pub fn prepare_notify(method: Method, params: Value) -> Value {
        serde_json::json!({"jsonrpc":"2.0","method":method.method_name(),"params":params})
    }

    pub fn request(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        let (handle, id) = self.register_pending();
        (serde_json::json!({"jsonrpc":"2.0","id":id.to_string(),"method":method.method_name(),"params":params}), handle)
    }

    pub fn request_async(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        self.request(method, params)
    }

    pub fn on_response(&mut self, id: &str, result: Value) -> bool {
        Uuid::parse_str(id).ok().and_then(|u| self.pending.remove(&u)).map(|tx| { let _ = tx.send(result); }).is_some()
    }

    pub fn on_error(&mut self, id: &str, error: Value) -> bool {
        self.on_response(id, serde_json::json!({"__error":error}))
    }

    pub fn pending_count(&self) -> usize { self.pending.len() }

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
        for m in Method::iter() {
            let n = m.method_name();
            assert!(!n.is_empty(), "{m:?} has empty wire name");
            assert!(n.contains('.'), "{m:?} missing dot: {n}");
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
    async fn test_pending_registry_sync() {
        let mut reg = PendingRegistry::new();
        let (frame, handle) = reg.request(Method::CliStatus, Value::Null);
        let id = frame["id"].as_str().unwrap();
        reg.on_response(id, serde_json::json!({"ok":true}));
        assert_eq!(handle.wait().await.unwrap(), serde_json::json!({"ok":true}));
    }

    #[tokio::test]
    async fn test_pending_registry_async() {
        let mut reg = PendingRegistry::new();
        let (f1, h1) = reg.request_async(Method::TuiYoloStart, Value::Null);
        let (f2, h2) = reg.request_async(Method::TuiYoloStop, Value::Null);
        assert_eq!(reg.pending_count(), 2);
        reg.on_response(f2["id"].as_str().unwrap(), serde_json::json!({"stopped":true}));
        assert_eq!(reg.pending_count(), 1);
        assert_eq!(h2.wait().await.unwrap(), serde_json::json!({"stopped":true}));
    }

    #[test]
    fn test_prepare_notify() {
        let f = PendingRegistry::prepare_notify(Method::TuiAgentReport, serde_json::json!({"x":1}));
        assert!(f.get("id").is_none());
        assert_eq!(f["method"], "Tui.AgentReport");
    }

    #[test]
    fn test_unsolicited_response_ignored() {
        let mut reg = PendingRegistry::new();
        assert!(!reg.on_response("nonexistent", Value::Null));
    }

    // ── Wire format integrity tests ──────────────────────────
    // These guarantee that every hand-written wire string in the
    // macro table is consistent with its enum variant name.

    const PREFIXES: &[&str] = &["Tui", "Cli", "Mcp", "Skill", "Base", "Device", "Screen"];

    #[test]
    fn test_no_duplicate_wire_names() {
        let mut seen = std::collections::HashSet::new();
        for m in Method::iter() {
            let wire = m.method_name();
            assert!(
                seen.insert(wire),
                "duplicate wire name: {wire}"
            );
        }
    }

    #[test]
    fn test_wire_name_roundtrip() {
        for m in Method::iter() {
            let wire = m.method_name();
            let parsed: Method = wire.parse()
                .unwrap_or_else(|_| panic!("wire '{wire}' (from {m:?}) cannot parse back"));
            assert_eq!(parsed, m,
                "roundtrip failed: {m:?} → '{wire}' → {parsed:?}"
            );
        }
    }

    #[test]
    fn test_wire_name_matches_variant_pattern() {
        for m in Method::iter() {
            let wire = m.method_name();
            let vname = format!("{m:?}");
            // Wire must be <Prefix>.<Action> with a known prefix
            let has_dot = wire.contains('.');
            assert!(has_dot, "{m:?} wire '{wire}' has no dot separator");

            let (prefix, action) = wire.split_once('.').unwrap();
            assert!(
                PREFIXES.contains(&prefix),
                "{m:?} wire '{wire}' has unknown prefix '{prefix}'"
            );

            // Variant name must be prefix + action (without the dot)
            let expected_variant = format!("{prefix}{action}");
            assert_eq!(
                vname, expected_variant,
                "{m:?} variant name '{vname}' should equal prefix+action '{expected_variant}'"
            );
        }
    }
}
