use serde_json::Value;
use std::collections::HashMap;
use strum::{Display, EnumIter, EnumString};
use tokio::sync::oneshot;
use uuid::Uuid;

// ══════════════════════════════════════════════════════════════
// namespace!  macro  —  single source of truth for each namespace.
//
// Each line:   Variant  [as Kind]  [=> ResponseVariant]
//   - "as Kind"  defaults to SyncReq;  OneWay / AsyncReq for others.
//   - "=> Resp"  pairs request→response.
//   - No hand-written wire strings — derived from enum path.
//
// Generates:
//   1.  Inner enum (TuiMethod, CliMethod, …) with strum derives
//   2.  wire()  —  "{Prefix}.{Variant}"  via concat!
//   3.  kind() / is_one_way() / response()  for the inner enum
//   4.  Paste-based flat aliases on Method  (Method::TuiServerVersion)
// ══════════════════════════════════════════════════════════════

macro_rules! namespace {
    (
        $prefix:literal, $ns:ident, $inner:ident,
        $( $variant:ident $(as $kind:ident)? $(=> $response:ident)? ),* $(,)?
    ) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Display,EnumString,EnumIter)]
        pub enum $inner { $( $variant, )* }

        impl $inner {
            pub fn wire(self) -> &'static str {
                match self { $(Self::$variant => concat!($prefix,".",stringify!($variant)),)* }
            }
            pub fn kind(self) -> MessageKind {
                match self {
                    $( Self::$variant => { let _ = stringify!($($kind)?); namespace_kind!($($kind)?) }, )*
                }
            }
            pub fn is_one_way(self) -> bool { matches!(self.kind(), MessageKind::OneWay) }
            pub fn response(self) -> Option<Self> {
                match self {
                    $( Self::$variant => { let _ = stringify!($($response)?); namespace_resp!($($response)?) }, )*
                }
            }
        }
    };
}

macro_rules! namespace_kind {
    ($k:ident) => {
        MessageKind::$k
    };
    () => {
        MessageKind::SyncReq
    };
}
macro_rules! namespace_resp {
    ($r:ident) => {
        Some(Self::$r)
    };
    () => {
        None
    };
}

// ══════════════════════════════════════════════════════════════
// Method enum — namespace wrapper
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Tui(TuiMethod),
    Cli(CliMethod),
    Mcp(McpMethod),
    Skill(SkillMethod),
    Base(BaseMethod),
    Device(DeviceMethod),
    Screen(ScreenMethod),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    OneWay,
    SyncReq,
    AsyncReq,
}

// ── Per-namespace definitions ─────────────────────────────────

namespace!("Tui", Tui, TuiMethod,
    ServerVersion              as OneWay,
    ConnectHandshake           as SyncReq   => HandshakeAck,
    HandshakeAck               as OneWay,
    VersionMismatch            as OneWay,
    ScepterIdentity            as OneWay,
    StatePatch                 as OneWay,
    StateSnapshot              as OneWay,
    ChannelEvent               as OneWay,
    RequestGlobalSnapshot      as SyncReq   => GlobalSnapshot,
    GlobalSnapshot             as OneWay,
    RequestContainerSnapshot   as SyncReq   => ContainerSnapshot,
    ContainerSnapshot          as OneWay,
    RequestTasksSnapshot       as SyncReq   => TasksSnapshot,
    TasksSnapshot              as OneWay,
    RequestVmSnapshot          as SyncReq   => VmSnapshot,
    VmSnapshot                 as OneWay,
    RequestFullSnapshot        as SyncReq   => FullSnapshot,
    FullSnapshot               as OneWay,
    GetProvidersFromFs         as SyncReq   => ProvidersFromFsResponse,
    ProvidersFromFsResponse    as OneWay,
    GetModelsFromFs            as SyncReq   => ModelsFromFsResponse,
    ModelsFromFsResponse       as OneWay,
    GetUserConfig              as SyncReq   => UserConfigResponse,
    UserConfigResponse         as OneWay,
    ModelsSnapshot             as OneWay,
    ProvidersSnapshot          as OneWay,
    UserMessage                as OneWay,
    AgentResponse              as OneWay,
    AgentStreamingChunk        as OneWay,
    AgentThinkingStep          as OneWay,
    AgentReport                as OneWay,
    AgentReportReply           as OneWay,
    AgentToolCall              as OneWay,
    AgentTransfer              as OneWay,
    AgentPatch                 as OneWay,
    AgentUpdate                as OneWay,
    AgentListResponse          as OneWay,
    AgentSnapshot              as OneWay,
    OrchestrationStatus        as OneWay,
    McpToolResult              as OneWay,
    TaskCreated                as OneWay,
    TaskStatusUpdate           as OneWay,
    TaskPatch                  as OneWay,
    ContainerPatch             as OneWay,
    SystemMessage              as OneWay,
    AskHumanRequest            as OneWay,
    AskHumanReply              as AsyncReq  => AskHumanReplyResponse,
    AskHumanReplyResponse      as OneWay,
    HumanReviewRequest         as OneWay,
    HumanReviewResponse        as OneWay,
    SkillChainStart            as OneWay,
    SkillChainStep             as OneWay,
    SkillChainComplete         as OneWay,
    YoloStart                  as AsyncReq  => YoloStartResponse,
    YoloStartResponse          as OneWay,
    YoloStop                   as AsyncReq  => YoloStopResponse,
    YoloStopResponse           as OneWay,
    YoloTerminate              as AsyncReq  => YoloTerminateResponse,
    YoloTerminateResponse      as OneWay,
    YoloStatus                 as AsyncReq  => YoloStatusResponse,
    YoloStatusResponse         as OneWay,
    YoloGetConfig              as AsyncReq  => YoloConfigResponse,
    YoloConfigResponse         as OneWay,
    YoloUpdateTask             as AsyncReq  => YoloUpdateTaskResponse,
    YoloUpdateTaskResponse     as OneWay,
    YoloSetTierInterval        as AsyncReq  => YoloSetTierIntervalResponse,
    YoloSetTierIntervalResponse as OneWay,
    YoloRunTierNow             as AsyncReq  => YoloRunTierNowResponse,
    YoloRunTierNowResponse     as OneWay,
    YoloCycleStep              as OneWay,
    YoloCycleComplete          as OneWay,
    YoloTaskStart              as OneWay,
    YoloTaskDone               as OneWay,
    YoloTaskError              as OneWay,
    OpenWorkspace              as SyncReq   => OpenWorkspaceResponse,
    OpenWorkspaceResponse      as OneWay,
    RequestWorkspaceStatus     as SyncReq   => WorkspaceStatus,
    WorkspaceStatus            as OneWay,
    ListAgents                 as SyncReq   => AgentListResponse,
    PolemosDeviceList          as OneWay,
    ListPolemosDevices         as SyncReq,
    RegisterPolemosDevice      as SyncReq   => RegisterPolemosDeviceResponse,
    RegisterPolemosDeviceResponse as OneWay,
    AuthLogin                  as SyncReq   => AuthLoginResponse,
    AuthLoginResponse          as OneWay,
    AuthRegister               as SyncReq   => AuthRegisterResponse,
    AuthRegisterResponse       as OneWay,
    AuthListUsers              as SyncReq   => AuthListUsersResponse,
    AuthListUsersResponse      as OneWay,
    AuthGetUser                as SyncReq   => AuthGetUserResponse,
    AuthGetUserResponse        as OneWay,
    AuthDeleteUser             as SyncReq   => AuthDeleteUserResponse,
    AuthDeleteUserResponse     as OneWay,
    AuthChangePassword         as SyncReq   => AuthChangePasswordResponse,
    AuthChangePasswordResponse as OneWay,
    Ping                       as SyncReq   => Pong,
    Pong                       as OneWay,
    UsagePeriodQuery           as SyncReq   => UsagePeriodResponse,
    UsagePeriodResponse        as OneWay,
    Layer2AgentList            as AsyncReq  => Layer2AgentListResponse,
    Layer2AgentListResponse    as OneWay,
    Layer2AgentMcpTools        as AsyncReq  => Layer2AgentMcpResponse,
    Layer2AgentMcpResponse     as OneWay,
    Layer2AgentSkills          as AsyncReq  => Layer2AgentSkillsResponse,
    Layer2AgentSkillsResponse  as OneWay,
    GetUserPreferences         as SyncReq,
    SyncPreferences            as SyncReq,
    AudioPullProgress          as OneWay,
    AudioStatusChanged         as OneWay,
    ServerInfo                 as OneWay,
    CrossWorkspaceDenied       as OneWay,
    RequestBridgeNetwork       as SyncReq,
    RequestFileTree            as SyncReq,
    RequestFileRead            as SyncReq,
    RequestModelList           as SyncReq,
    RequestModelServerAction   as SyncReq,
    AgentChunkRange            as OneWay,
    AgentChunkCount            as OneWay,
    IndustrialTelemetryPush    as OneWay,
    IndustrialAlarmPush        as OneWay,
    IndustrialWriteApprovalPush as OneWay,
    ServerLogEntry             as OneWay,
    ContainerLogEntry          as OneWay,
);

namespace!(
    "Cli",
    Cli,
    CliMethod,
    Status,
    ChatHistory,
    TimelineList,
    TimelineShow,
    RecentChats,
    ListPolemosDevices,
    SessionStats,
    SessionPurge,
    SessionVacuum,
    Search,
    TraceChain,
    ListTools,
    ListSkills,
    ListWorkspaces,
    OpenWorkspace,
    SwitchWorkspace,
);

namespace!("Mcp", Mcp, McpMethod,
    CallTool         as AsyncReq => ToolCallResult,
    ToolCallResult   as OneWay,
    ListTools        as SyncReq  => ToolsListResponse,
    ToolsListResponse as OneWay,
);

namespace!("Skill", Skill, SkillMethod,
    CallSkill         as AsyncReq => SkillCallResult,
    SkillCallResult   as OneWay,
    ListSkills        as SyncReq  => SkillsListResponse,
    SkillsListResponse as OneWay,
);

namespace!("Base", Base, BaseMethod,
    Heartbeat        as SyncReq => HeartbeatAck,
    HeartbeatAck     as OneWay,
    Error            as OneWay,
    Ack              as OneWay,
);

namespace!("Device", Device, DeviceMethod,
    PolemosRegister   as SyncReq => PolemosRegisterAck,
    PolemosRegisterAck as OneWay,
    Heartbeat         as SyncReq => HeartbeatAck,
    HeartbeatAck      as OneWay,
    TerminalOpen      as SyncReq => TerminalReady,
    TerminalReady     as OneWay,
    TerminalInput     as OneWay,
    TerminalResize    as OneWay,
    TerminalPoll      as SyncReq => TerminalPollResult,
    TerminalPollResult as OneWay,
    TerminalClose     as SyncReq => TerminalCloseAck,
    TerminalCloseAck  as OneWay,
    FileList          as SyncReq => FileListResult,
    FileListResult    as OneWay,
    FileDownload      as SyncReq => FileDownloadResult,
    FileDownloadResult as OneWay,
    FileUpload        as SyncReq => FileUploadResult,
    FileUploadResult  as OneWay,
    Ping              as SyncReq => Pong,
    Pong              as OneWay,
    WebrtcOffer       as SyncReq => WebrtcAnswer,
    WebrtcAnswer      as OneWay,
    WebrtcIce         as OneWay,
    SubscribeOutput   as OneWay,
    TerminalList      as SyncReq,
    TerminalOutput    as OneWay,
    Error             as OneWay,
);

namespace!(
    "Screen",
    Screen,
    ScreenMethod,
    Offer as OneWay,
    Answer as OneWay,
    Ice as OneWay,
    IceCandidate as OneWay,
);

// ══════════════════════════════════════════════════════════════
// Generated flat aliases on Method (via paste)
// ══════════════════════════════════════════════════════════════

macro_rules! flat_aliases {
    ($ns:ident, $inner:ident, $($variant:ident),* $(,)?) => {
        impl Method {
            $( paste::paste! { pub const [<$ns $variant>]: Method = Method::$ns($inner::$variant); } )*
        }
    };
}

flat_aliases!(
    Tui,
    TuiMethod,
    ServerVersion,
    ConnectHandshake,
    HandshakeAck,
    VersionMismatch,
    ScepterIdentity,
    StatePatch,
    StateSnapshot,
    ChannelEvent,
    RequestGlobalSnapshot,
    GlobalSnapshot,
    RequestContainerSnapshot,
    ContainerSnapshot,
    RequestTasksSnapshot,
    TasksSnapshot,
    RequestVmSnapshot,
    VmSnapshot,
    RequestFullSnapshot,
    FullSnapshot,
    GetProvidersFromFs,
    ProvidersFromFsResponse,
    GetModelsFromFs,
    ModelsFromFsResponse,
    GetUserConfig,
    UserConfigResponse,
    ModelsSnapshot,
    ProvidersSnapshot,
    UserMessage,
    AgentResponse,
    AgentStreamingChunk,
    AgentThinkingStep,
    AgentReport,
    AgentReportReply,
    AgentToolCall,
    AgentTransfer,
    AgentPatch,
    AgentUpdate,
    AgentListResponse,
    AgentSnapshot,
    OrchestrationStatus,
    McpToolResult,
    TaskCreated,
    TaskStatusUpdate,
    TaskPatch,
    ContainerPatch,
    SystemMessage,
    AskHumanRequest,
    AskHumanReply,
    AskHumanReplyResponse,
    HumanReviewRequest,
    HumanReviewResponse,
    SkillChainStart,
    SkillChainStep,
    SkillChainComplete,
    YoloStart,
    YoloStartResponse,
    YoloStop,
    YoloStopResponse,
    YoloTerminate,
    YoloTerminateResponse,
    YoloStatus,
    YoloStatusResponse,
    YoloGetConfig,
    YoloConfigResponse,
    YoloUpdateTask,
    YoloUpdateTaskResponse,
    YoloSetTierInterval,
    YoloSetTierIntervalResponse,
    YoloRunTierNow,
    YoloRunTierNowResponse,
    YoloCycleStep,
    YoloCycleComplete,
    YoloTaskStart,
    YoloTaskDone,
    YoloTaskError,
    OpenWorkspace,
    OpenWorkspaceResponse,
    RequestWorkspaceStatus,
    WorkspaceStatus,
    ListAgents,
    PolemosDeviceList,
    ListPolemosDevices,
    RegisterPolemosDevice,
    RegisterPolemosDeviceResponse,
    AuthLogin,
    AuthLoginResponse,
    AuthRegister,
    AuthRegisterResponse,
    AuthListUsers,
    AuthListUsersResponse,
    AuthGetUser,
    AuthGetUserResponse,
    AuthDeleteUser,
    AuthDeleteUserResponse,
    AuthChangePassword,
    AuthChangePasswordResponse,
    Ping,
    Pong,
    UsagePeriodQuery,
    UsagePeriodResponse,
    Layer2AgentList,
    Layer2AgentListResponse,
    Layer2AgentMcpTools,
    Layer2AgentMcpResponse,
    Layer2AgentSkills,
    Layer2AgentSkillsResponse,
    GetUserPreferences,
    SyncPreferences,
    AudioPullProgress,
    AudioStatusChanged,
    ServerInfo,
    CrossWorkspaceDenied,
    RequestBridgeNetwork,
    RequestFileTree,
    RequestFileRead,
    RequestModelList,
    RequestModelServerAction,
    AgentChunkRange,
    AgentChunkCount,
    IndustrialTelemetryPush,
    IndustrialAlarmPush,
    IndustrialWriteApprovalPush,
    ServerLogEntry,
    ContainerLogEntry,
);
flat_aliases!(
    Cli,
    CliMethod,
    Status,
    ChatHistory,
    TimelineList,
    TimelineShow,
    RecentChats,
    ListPolemosDevices,
    SessionStats,
    SessionPurge,
    SessionVacuum,
    Search,
    TraceChain,
    ListTools,
    ListSkills,
    ListWorkspaces,
    OpenWorkspace,
    SwitchWorkspace
);
flat_aliases!(
    Mcp,
    McpMethod,
    CallTool,
    ToolCallResult,
    ListTools,
    ToolsListResponse
);
flat_aliases!(
    Skill,
    SkillMethod,
    CallSkill,
    SkillCallResult,
    ListSkills,
    SkillsListResponse
);
flat_aliases!(Base, BaseMethod, Heartbeat, HeartbeatAck, Error, Ack);
flat_aliases!(
    Device,
    DeviceMethod,
    PolemosRegister,
    PolemosRegisterAck,
    Heartbeat,
    HeartbeatAck,
    TerminalOpen,
    TerminalReady,
    TerminalInput,
    TerminalResize,
    TerminalPoll,
    TerminalPollResult,
    TerminalClose,
    TerminalCloseAck,
    FileList,
    FileListResult,
    FileDownload,
    FileDownloadResult,
    FileUpload,
    FileUploadResult,
    Ping,
    Pong,
    WebrtcOffer,
    WebrtcAnswer,
    WebrtcIce,
    SubscribeOutput,
    TerminalList,
    TerminalOutput,
    Error,
);
flat_aliases!(Screen, ScreenMethod, Offer, Answer, Ice, IceCandidate);

// ══════════════════════════════════════════════════════════════
// Method impl — delegates to inner
// ══════════════════════════════════════════════════════════════

impl Method {
    pub fn method_name(self) -> &'static str {
        match self {
            Method::Tui(m) => m.wire(),
            Method::Cli(m) => m.wire(),
            Method::Mcp(m) => m.wire(),
            Method::Skill(m) => m.wire(),
            Method::Base(m) => m.wire(),
            Method::Device(m) => m.wire(),
            Method::Screen(m) => m.wire(),
        }
    }
    pub fn kind(self) -> MessageKind {
        match self {
            Method::Tui(m) => m.kind(),
            Method::Cli(m) => m.kind(),
            Method::Mcp(m) => m.kind(),
            Method::Skill(m) => m.kind(),
            Method::Base(m) => m.kind(),
            Method::Device(m) => m.kind(),
            Method::Screen(m) => m.kind(),
        }
    }
    pub fn is_one_way(self) -> bool {
        matches!(self.kind(), MessageKind::OneWay)
    }
    pub fn response(self) -> Option<Method> {
        match self {
            Method::Tui(m) => m.response().map(Method::Tui),
            Method::Cli(m) => m.response().map(Method::Cli),
            Method::Mcp(m) => m.response().map(Method::Mcp),
            Method::Skill(m) => m.response().map(Method::Skill),
            Method::Base(m) => m.response().map(Method::Base),
            Method::Device(m) => m.response().map(Method::Device),
            Method::Screen(m) => m.response().map(Method::Screen),
        }
    }
}

impl std::str::FromStr for Method {
    type Err = strum::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ns, action) = s
            .split_once('.')
            .ok_or(strum::ParseError::VariantNotFound)?;
        Ok(match ns {
            "Tui" => Method::Tui(action.parse()?),
            "Cli" => Method::Cli(action.parse()?),
            "Mcp" => Method::Mcp(action.parse()?),
            "Skill" => Method::Skill(action.parse()?),
            "Base" => Method::Base(action.parse()?),
            "Device" => Method::Device(action.parse()?),
            "Screen" => Method::Screen(action.parse()?),
            _ => return Err(strum::ParseError::VariantNotFound),
        })
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.method_name())
    }
}

// ══════════════════════════════════════════════════════════════
// PendingRegistry
// ══════════════════════════════════════════════════════════════

pub struct PendingHandle {
    pub id: Uuid,
    rx: oneshot::Receiver<Value>,
}
impl PendingHandle {
    pub async fn wait(self) -> Result<Value, oneshot::error::RecvError> {
        self.rx.await
    }
}

pub struct PendingRegistry {
    pending: HashMap<Uuid, oneshot::Sender<Value>>,
}

impl PendingRegistry {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }
    pub fn prepare_notify(method: Method, params: Value) -> Value {
        serde_json::json!({"jsonrpc":"2.0","method":method.method_name(),"params":params})
    }
    pub fn request(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        let (h, id) = self.register_pending();
        (
            serde_json::json!({"jsonrpc":"2.0","id":id.to_string(),"method":method.method_name(),"params":params}),
            h,
        )
    }
    pub fn request_async(&mut self, method: Method, params: Value) -> (Value, PendingHandle) {
        self.request(method, params)
    }
    pub fn on_response(&mut self, id: &str, result: Value) -> bool {
        Uuid::parse_str(id)
            .ok()
            .and_then(|u| self.pending.remove(&u))
            .map(|tx| {
                let _ = tx.send(result);
            })
            .is_some()
    }
    pub fn on_error(&mut self, id: &str, error: Value) -> bool {
        self.on_response(id, serde_json::json!({"__error":error}))
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
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_wire_from_enum_path() {
        assert_eq!(
            Method::Tui(TuiMethod::ServerVersion).method_name(),
            "Sync.ServerVersion"
        );
        assert_eq!(Method::TuiServerVersion.method_name(), "Sync.ServerVersion"); // flat alias
        assert_eq!(Method::Cli(CliMethod::Status).method_name(), "Cli.Status");
        assert_eq!(Method::CliStatus.method_name(), "Cli.Status");
        assert_eq!(
            Method::Mcp(McpMethod::CallTool).method_name(),
            "Mcp.CallTool"
        );
    }
    #[test]
    fn test_parse_roundtrip() {
        assert_eq!(
            "Sync.ServerVersion"
                .parse::<Method>()
                .unwrap()
                .method_name(),
            "Sync.ServerVersion"
        );
        assert_eq!(
            "Cli.Status".parse::<Method>().unwrap().method_name(),
            "Cli.Status"
        );
        assert_eq!(
            "Device.TerminalOpen"
                .parse::<Method>()
                .unwrap()
                .method_name(),
            "Device.TerminalOpen"
        );
    }
    #[test]
    fn test_kind() {
        assert_eq!(TuiMethod::ServerVersion.kind(), MessageKind::OneWay);
        assert_eq!(TuiMethod::ConnectHandshake.kind(), MessageKind::SyncReq);
        assert_eq!(TuiMethod::YoloStart.kind(), MessageKind::AsyncReq);
        assert_eq!(Method::TuiServerVersion.kind(), MessageKind::OneWay);
    }
    #[test]
    fn test_response() {
        assert_eq!(
            TuiMethod::ConnectHandshake.response(),
            Some(TuiMethod::HandshakeAck)
        );
        assert_eq!(TuiMethod::Ping.response(), Some(TuiMethod::Pong));
        assert_eq!(TuiMethod::ServerVersion.response(), None);
        assert_eq!(Method::TuiServerVersion.response(), None);
    }
    #[tokio::test]
    async fn test_pending() {
        let mut r = PendingRegistry::new();
        let (f, h) = r.request(Method::CliStatus, Value::Null);
        r.on_response(f["id"].as_str().unwrap(), serde_json::json!({"ok":true}));
        assert_eq!(h.wait().await.unwrap(), serde_json::json!({"ok":true}));
    }
    #[test]
    fn test_prepare_notify() {
        let f = PendingRegistry::prepare_notify(Method::TuiAgentReport, serde_json::json!({"x":1}));
        assert!(f.get("id").is_none());
        assert_eq!(f["method"], "Sync.AgentReport");
    }
}
