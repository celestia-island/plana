use serde_json::Value;
use std::fmt;

use super::{json_keys::BridgeKey, types::*};
use _state_sync::gateway::Message as CoreMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayMethod {
    Tui(&'static str),
    Base(&'static str),
    Agent(&'static str),
    Mcp(&'static str),
    Skill(&'static str),
    Node(&'static str),
    Monitor(&'static str),
    Conversation(&'static str),
    Device(&'static str),
    Screen(&'static str),
    Cli(&'static str),
    Trigger(&'static str),
    Sensor(&'static str),
    Discovery(&'static str),
    Command(&'static str),
    /// DLC extension method — arbitrary string for consumer-specific RPCs
    /// (e.g. shittim-chest's auth.*/channels.*/topology.* etc.)
    Extension(String),
}

impl GatewayMethod {
    pub const TUI_PING: Self = Self::Tui("Ping");
    pub const TUI_AGENT_PATCH: Self = Self::Tui("AgentPatch");
    pub const TUI_ORCHESTRATION_STATUS: Self = Self::Tui("OrchestrationStatus");
    pub const TUI_MCP_TOOL_RESULT: Self = Self::Tui("McpToolResult");
    pub const TUI_AGENT_STREAMING_CHUNK: Self = Self::Tui("AgentStreamingChunk");
    pub const TUI_AGENT_REPORT: Self = Self::Tui("AgentReport");
    pub const TUI_AGENT_TRANSFER: Self = Self::Tui("AgentTransfer");
    pub const TUI_ASK_HUMAN_REQUEST: Self = Self::Tui("AskHumanRequest");
    pub const TUI_USER_MESSAGE: Self = Self::Tui("UserMessage");
    pub const TUI_AGENT_RESPONSE: Self = Self::Tui("AgentResponse");
    pub const TUI_REQUEST_FULL_SNAPSHOT: Self = Self::Tui("RequestFullSnapshot");
    pub const TUI_REQUEST_GLOBAL_SNAPSHOT: Self = Self::Tui("RequestGlobalSnapshot");
    pub const TUI_GLOBAL_SNAPSHOT: Self = Self::Tui("GlobalSnapshot");
    pub const TUI_MODELS_SNAPSHOT: Self = Self::Tui("ModelsSnapshot");
    pub const TUI_PROVIDERS_SNAPSHOT: Self = Self::Tui("ProvidersSnapshot");
    pub const TUI_CONTAINER_SNAPSHOT: Self = Self::Tui("ContainerSnapshot");
    pub const TUI_CONTAINER_PATCH: Self = Self::Tui("ContainerPatch");
    pub const TUI_TASK_PATCH: Self = Self::Tui("TaskPatch");
    pub const TUI_TASKS_SNAPSHOT: Self = Self::Tui("TasksSnapshot");
    pub const TUI_LIST_AGENTS: Self = Self::Tui("ListAgents");
    pub const TUI_SERVER_VERSION: Self = Self::Tui("ServerVersion");
    pub const TUI_OPEN_WORKSPACE: Self = Self::Tui("OpenWorkspace");
    pub const TUI_WORKSPACE_STATUS: Self = Self::Tui("WorkspaceStatus");
    pub const TUI_REQUEST_WORKSPACE_STATUS: Self = Self::Tui("RequestWorkspaceStatus");
    pub const TUI_SYSTEM_MESSAGE: Self = Self::Tui("SystemMessage");
    pub const TUI_WEBUI_CONTROL: Self = Self::Tui("WebUiControl");
    pub const TUI_WEBUI_CONTROL_RESPONSE: Self = Self::Tui("WebUiControlResponse");
    pub const TUI_WEBUI_STATUS: Self = Self::Tui("WebUiStatus");
    pub const TUI_REQUEST_WEBUI_STATUS: Self = Self::Tui("RequestWebUiStatus");

    pub const TUI_AUTH_LOGIN: Self = Self::Tui("AuthLogin");
    pub const TUI_AUTH_LOGIN_RESPONSE: Self = Self::Tui("AuthLoginResponse");
    pub const TUI_AUTH_REGISTER: Self = Self::Tui("AuthRegister");
    pub const TUI_AUTH_REGISTER_RESPONSE: Self = Self::Tui("AuthRegisterResponse");
    pub const TUI_AUTH_LIST_USERS: Self = Self::Tui("AuthListUsers");
    pub const TUI_AUTH_LIST_USERS_RESPONSE: Self = Self::Tui("AuthListUsersResponse");
    pub const TUI_AUTH_GET_USER: Self = Self::Tui("AuthGetUser");
    pub const TUI_AUTH_GET_USER_RESPONSE: Self = Self::Tui("AuthGetUserResponse");
    pub const TUI_AUTH_DELETE_USER: Self = Self::Tui("AuthDeleteUser");
    pub const TUI_AUTH_DELETE_USER_RESPONSE: Self = Self::Tui("AuthDeleteUserResponse");
    pub const TUI_AUTH_CHANGE_PASSWORD: Self = Self::Tui("AuthChangePassword");
    pub const TUI_AUTH_CHANGE_PASSWORD_RESPONSE: Self = Self::Tui("AuthChangePasswordResponse");

    pub const BASE_HEARTBEAT: Self = Self::Base("Heartbeat");
    pub const BASE_ERROR: Self = Self::Base("Error");
    pub const BASE_ACK: Self = Self::Base("Ack");

    pub const MCP_CALL: Self = Self::Mcp("CallTool");
    pub const MCP_LIST_TOOLS: Self = Self::Mcp("ListTools");
    pub const MCP_TOOLS_LIST_RESPONSE: Self = Self::Mcp("ToolsListResponse");

    pub const SKILL_CALL: Self = Self::Skill("CallSkill");
    pub const SKILL_LIST_SKILLS: Self = Self::Skill("ListSkills");
    pub const SKILL_LIST_SKILLS_RESPONSE: Self = Self::Skill("SkillsListResponse");

    pub fn as_str(&self) -> String {
        match self {
            Self::Tui(action) => format!("Tui.{}", action),
            Self::Base(action) => format!("Base.{}", action),
            Self::Agent(action) => format!("Agent.{}", action),
            Self::Mcp(action) => format!("Mcp.{}", action),
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

    pub fn type_prefix(&self) -> &'static str {
        match self {
            Self::Tui(_) => "Tui",
            Self::Base(_) => "Base",
            Self::Agent(_) => "Agent",
            Self::Mcp(_) => "Mcp",
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

    pub fn action(&self) -> &str {
        match self {
            Self::Tui(a)
            | Self::Base(a)
            | Self::Agent(a)
            | Self::Mcp(a)
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown gateway method: {0}")]
pub struct UnknownGatewayMethodError(pub String);

impl std::str::FromStr for GatewayMethod {
    type Err = UnknownGatewayMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tui.Ping" => Ok(Self::TUI_PING),
            "Tui.AgentPatch" => Ok(Self::TUI_AGENT_PATCH),
            "Tui.OrchestrationStatus" => Ok(Self::TUI_ORCHESTRATION_STATUS),
            "Tui.McpToolResult" => Ok(Self::TUI_MCP_TOOL_RESULT),
            "Tui.AgentStreamingChunk" => Ok(Self::TUI_AGENT_STREAMING_CHUNK),
            "Tui.AgentReport" => Ok(Self::TUI_AGENT_REPORT),
            "Tui.AskHumanRequest" => Ok(Self::TUI_ASK_HUMAN_REQUEST),
            "Tui.UserMessage" => Ok(Self::TUI_USER_MESSAGE),
            "Tui.AgentResponse" => Ok(Self::TUI_AGENT_RESPONSE),
            "Tui.RequestFullSnapshot" => Ok(Self::TUI_REQUEST_FULL_SNAPSHOT),
            "Tui.RequestGlobalSnapshot" => Ok(Self::TUI_REQUEST_GLOBAL_SNAPSHOT),
            "Tui.GlobalSnapshot" => Ok(Self::TUI_GLOBAL_SNAPSHOT),
            "Tui.ModelsSnapshot" => Ok(Self::TUI_MODELS_SNAPSHOT),
            "Tui.ProvidersSnapshot" => Ok(Self::TUI_PROVIDERS_SNAPSHOT),
            "Tui.ContainerSnapshot" => Ok(Self::TUI_CONTAINER_SNAPSHOT),
            "Tui.ContainerPatch" => Ok(Self::TUI_CONTAINER_PATCH),
            "Tui.TaskPatch" => Ok(Self::TUI_TASK_PATCH),
            "Tui.TasksSnapshot" => Ok(Self::TUI_TASKS_SNAPSHOT),
            "Tui.ListAgents" => Ok(Self::TUI_LIST_AGENTS),
            "Tui.ServerVersion" => Ok(Self::TUI_SERVER_VERSION),
            "Tui.OpenGitWorkspace" | "Tui.OpenWorkspace" => Ok(Self::TUI_OPEN_WORKSPACE),
            "Tui.WorkspaceStatus" => Ok(Self::TUI_WORKSPACE_STATUS),
            "Tui.RequestWorkspaceStatus" => Ok(Self::TUI_REQUEST_WORKSPACE_STATUS),
            "Tui.SystemMessage" => Ok(Self::TUI_SYSTEM_MESSAGE),
            "Tui.WebUiControl" => Ok(Self::TUI_WEBUI_CONTROL),
            "Tui.WebUiControlResponse" => Ok(Self::TUI_WEBUI_CONTROL_RESPONSE),
            "Tui.WebUiStatus" => Ok(Self::TUI_WEBUI_STATUS),
            "Tui.RequestWebUiStatus" => Ok(Self::TUI_REQUEST_WEBUI_STATUS),
            "Tui.AuthLogin" => Ok(Self::TUI_AUTH_LOGIN),
            "Tui.AuthLoginResponse" => Ok(Self::TUI_AUTH_LOGIN_RESPONSE),
            "Tui.AuthRegister" => Ok(Self::TUI_AUTH_REGISTER),
            "Tui.AuthRegisterResponse" => Ok(Self::TUI_AUTH_REGISTER_RESPONSE),
            "Tui.AuthListUsers" => Ok(Self::TUI_AUTH_LIST_USERS),
            "Tui.AuthListUsersResponse" => Ok(Self::TUI_AUTH_LIST_USERS_RESPONSE),
            "Tui.AuthGetUser" => Ok(Self::TUI_AUTH_GET_USER),
            "Tui.AuthGetUserResponse" => Ok(Self::TUI_AUTH_GET_USER_RESPONSE),
            "Tui.AuthDeleteUser" => Ok(Self::TUI_AUTH_DELETE_USER),
            "Tui.AuthDeleteUserResponse" => Ok(Self::TUI_AUTH_DELETE_USER_RESPONSE),
            "Tui.AuthChangePassword" => Ok(Self::TUI_AUTH_CHANGE_PASSWORD),
            "Tui.AuthChangePasswordResponse" => Ok(Self::TUI_AUTH_CHANGE_PASSWORD_RESPONSE),
            "Base.Heartbeat" => Ok(Self::BASE_HEARTBEAT),
            "Base.Error" => Ok(Self::BASE_ERROR),
            "Base.Ack" => Ok(Self::BASE_ACK),
            "Mcp.CallTool" => Ok(Self::MCP_CALL),
            "Mcp.ListTools" => Ok(Self::MCP_LIST_TOOLS),
            "Mcp.ToolsListResponse" => Ok(Self::MCP_TOOLS_LIST_RESPONSE),
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

pub fn core_message_to_method_and_params(msg: &CoreMessage) -> (String, Option<Value>) {
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

                let method = format!("{}.{}", type_name, action);

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
        },
        _ => ("Unknown.Unknown".to_string(), Some(json)),
    }
}

pub fn from_jsonrpc_method(method: &str, params: Option<Value>) -> Option<CoreMessage> {
    let (type_name, action) = method.split_once('.')?;

    let data = match params {
        Some(Value::Object(mut map)) => {
            map.insert(
                BridgeKey::Action.as_ref().to_string(),
                Value::String(action.to_string()),
            );
            Value::Object(map)
        },
        _ => Value::Object({
            let mut map = serde_json::Map::new();
            map.insert(
                BridgeKey::Action.as_ref().to_string(),
                Value::String(action.to_string()),
            );
            map
        }),
    };

    let mut reconstructed_map = serde_json::Map::new();
    reconstructed_map.insert(
        BridgeKey::Type.as_ref().to_string(),
        Value::String(type_name.to_string()),
    );
    reconstructed_map.insert(BridgeKey::Data.as_ref().to_string(), data);
    let reconstructed = Value::Object(reconstructed_map);

    serde_json::from_value::<CoreMessage>(reconstructed).ok()
}

pub fn serialize_to_jsonrpc(
    msg: &CoreMessage,
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

pub fn deserialize_from_jsonrpc(json: &str) -> Result<Option<CoreMessage>, JsonRpcError> {
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
    use _state_sync::gateway::{BaseMessage, TuiMessage};
    use anyhow::Context;

    #[test]
    fn roundtrip_user_message() -> anyhow::Result<()> {
        let msg = CoreMessage::Tui(TuiMessage::Ping {
            timestamp: 12345u64,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::TUI_PING.as_str());

        let reconstructed =
            from_jsonrpc_method(&method, params).context("failed to reconstruct Ping message")?;
        match reconstructed {
            CoreMessage::Tui(TuiMessage::Ping { timestamp }) => {
                assert_eq!(timestamp, 12345u64);
            },
            other => anyhow::bail!("Expected {}, got {:?}", GatewayMethod::TUI_PING, other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_heartbeat() -> anyhow::Result<()> {
        let msg = CoreMessage::Base(BaseMessage::Heartbeat { timestamp: 999 });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::BASE_HEARTBEAT.as_str());

        let reconstructed = from_jsonrpc_method(&method, params)
            .context("failed to reconstruct Heartbeat message")?;
        match reconstructed {
            CoreMessage::Base(BaseMessage::Heartbeat { timestamp }) => {
                assert_eq!(timestamp, 999);
            },
            other => anyhow::bail!(
                "Expected {}, got {:?}",
                GatewayMethod::BASE_HEARTBEAT,
                other
            ),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_open_workspace() -> anyhow::Result<()> {
        let msg = CoreMessage::Tui(TuiMessage::OpenWorkspace {
            uri: "git://https://github.com/org/repo.git".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::TUI_OPEN_WORKSPACE.as_str());

        let reconstructed =
            from_jsonrpc_method(&method, params).context("failed to reconstruct OpenWorkspace")?;
        match reconstructed {
            CoreMessage::Tui(TuiMessage::OpenWorkspace { uri }) => {
                assert_eq!(uri, "git://https://github.com/org/repo.git");
            },
            other => anyhow::bail!("Expected OpenWorkspace, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_workspace_status() -> anyhow::Result<()> {
        let test_ws_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000123")
            .context("test precondition")?;
        let msg = CoreMessage::Tui(TuiMessage::WorkspaceStatus {
            workspace_id: test_ws_id,
            display_name: Some("my-repo".to_string()),
            connection_kind: "git".to_string(),
            resolved_path: Some("/tmp/workspaces/ws-123".to_string()),
            remote_url: Some("https://github.com/org/repo.git".to_string()),
            branch: Some("main".to_string()),
            host_id: None,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::TUI_WORKSPACE_STATUS.as_str());

        let reconstructed = from_jsonrpc_method(&method, params)
            .context("failed to reconstruct WorkspaceStatus")?;
        match reconstructed {
            CoreMessage::Tui(TuiMessage::WorkspaceStatus {
                workspace_id,
                connection_kind,
                remote_url,
                ..
            }) => {
                assert_eq!(workspace_id, test_ws_id);
                assert_eq!(connection_kind, "git");
                assert_eq!(
                    remote_url,
                    Some("https://github.com/org/repo.git".to_string())
                );
            },
            other => anyhow::bail!("Expected WorkspaceStatus, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_open_workspace() -> anyhow::Result<()> {
        let method: GatewayMethod = "Tui.OpenWorkspace".parse()?;
        assert_eq!(method.as_str(), GatewayMethod::TUI_OPEN_WORKSPACE.as_str());
        Ok(())
    }

    #[test]
    fn gateway_method_parse_workspace_status() -> anyhow::Result<()> {
        let method: GatewayMethod = "Tui.WorkspaceStatus".parse()?;
        assert_eq!(
            method.as_str(),
            GatewayMethod::TUI_WORKSPACE_STATUS.as_str()
        );
        Ok(())
    }

    #[test]
    fn roundtrip_system_message() -> anyhow::Result<()> {
        let msg = CoreMessage::Tui(TuiMessage::SystemMessage {
            notification: _state_sync::SystemNotification::WebUiStarted,
            timestamp: "2026-05-11T12:00:00Z".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::TUI_SYSTEM_MESSAGE.as_str());
        let reconstructed =
            from_jsonrpc_method(&method, params).context("system message roundtrip")?;
        match reconstructed {
            CoreMessage::Tui(TuiMessage::SystemMessage {
                notification,
                timestamp,
            }) => {
                assert_eq!(
                    notification,
                    _state_sync::SystemNotification::WebUiStarted
                );
                assert_eq!(timestamp, "2026-05-11T12:00:00Z");
            },
            other => anyhow::bail!("Expected SystemMessage, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_methods() -> anyhow::Result<()> {
        for (s, expected) in [
            ("Tui.SystemMessage", GatewayMethod::TUI_SYSTEM_MESSAGE),
            ("Tui.AuthLogin", GatewayMethod::TUI_AUTH_LOGIN),
            (
                "Tui.AuthLoginResponse",
                GatewayMethod::TUI_AUTH_LOGIN_RESPONSE,
            ),
        ] {
            let method: GatewayMethod = s.parse()?;
            assert_eq!(method.as_str(), expected.as_str());
        }
        Ok(())
    }
}
