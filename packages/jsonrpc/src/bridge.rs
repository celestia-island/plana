use serde_json::Value;
use std::fmt;

use super::{json_keys::BridgeKey, types::*};
use _state_sync::gateway::Message as CoreMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayMethod {
    Sync(&'static str),
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
    pub const SYNC_PING: Self = Self::Sync("Ping");
    pub const SYNC_AGENT_PATCH: Self = Self::Sync("AgentPatch");
    pub const SYNC_ORCHESTRATION_STATUS: Self = Self::Sync("OrchestrationStatus");
    pub const SYNC_MCP_TOOL_RESULT: Self = Self::Sync("McpToolResult");
    pub const SYNC_AGENT_STREAMING_CHUNK: Self = Self::Sync("AgentStreamingChunk");
    pub const SYNC_AGENT_REPORT: Self = Self::Sync("AgentReport");
    pub const SYNC_AGENT_TRANSFER: Self = Self::Sync("AgentTransfer");
    pub const SYNC_ASK_HUMAN_REQUEST: Self = Self::Sync("AskHumanRequest");
    pub const SYNC_USER_MESSAGE: Self = Self::Sync("UserMessage");
    pub const SYNC_AGENT_RESPONSE: Self = Self::Sync("AgentResponse");
    pub const SYNC_REQUEST_FULL_SNAPSHOT: Self = Self::Sync("RequestFullSnapshot");
    pub const SYNC_REQUEST_GLOBAL_SNAPSHOT: Self = Self::Sync("RequestGlobalSnapshot");
    pub const SYNC_GLOBAL_SNAPSHOT: Self = Self::Sync("GlobalSnapshot");
    pub const SYNC_MODELS_SNAPSHOT: Self = Self::Sync("ModelsSnapshot");
    pub const SYNC_PROVIDERS_SNAPSHOT: Self = Self::Sync("ProvidersSnapshot");
    pub const SYNC_CONTAINER_SNAPSHOT: Self = Self::Sync("ContainerSnapshot");
    pub const SYNC_CONTAINER_PATCH: Self = Self::Sync("ContainerPatch");
    pub const SYNC_TASK_PATCH: Self = Self::Sync("TaskPatch");
    pub const SYNC_TASKS_SNAPSHOT: Self = Self::Sync("TasksSnapshot");
    pub const SYNC_LIST_AGENTS: Self = Self::Sync("ListAgents");
    pub const SYNC_SERVER_VERSION: Self = Self::Sync("ServerVersion");
    pub const SYNC_OPEN_WORKSPACE: Self = Self::Sync("OpenWorkspace");
    pub const SYNC_WORKSPACE_STATUS: Self = Self::Sync("WorkspaceStatus");
    pub const SYNC_REQUEST_WORKSPACE_STATUS: Self = Self::Sync("RequestWorkspaceStatus");
    pub const SYNC_SYSTEM_MESSAGE: Self = Self::Sync("SystemMessage");
    pub const SYNC_WEBUI_CONTROL: Self = Self::Sync("WebUiControl");
    pub const SYNC_WEBUI_CONTROL_RESPONSE: Self = Self::Sync("WebUiControlResponse");
    pub const SYNC_WEBUI_STATUS: Self = Self::Sync("WebUiStatus");
    pub const SYNC_REQUEST_WEBUI_STATUS: Self = Self::Sync("RequestWebUiStatus");

    pub const SYNC_AUTH_LOGIN: Self = Self::Sync("AuthLogin");
    pub const SYNC_AUTH_LOGIN_RESPONSE: Self = Self::Sync("AuthLoginResponse");
    pub const SYNC_AUTH_REGISTER: Self = Self::Sync("AuthRegister");
    pub const SYNC_AUTH_REGISTER_RESPONSE: Self = Self::Sync("AuthRegisterResponse");
    pub const SYNC_AUTH_LIST_USERS: Self = Self::Sync("AuthListUsers");
    pub const SYNC_AUTH_LIST_USERS_RESPONSE: Self = Self::Sync("AuthListUsersResponse");
    pub const SYNC_AUTH_GET_USER: Self = Self::Sync("AuthGetUser");
    pub const SYNC_AUTH_GET_USER_RESPONSE: Self = Self::Sync("AuthGetUserResponse");
    pub const SYNC_AUTH_DELETE_USER: Self = Self::Sync("AuthDeleteUser");
    pub const SYNC_AUTH_DELETE_USER_RESPONSE: Self = Self::Sync("AuthDeleteUserResponse");
    pub const SYNC_AUTH_CHANGE_PASSWORD: Self = Self::Sync("AuthChangePassword");
    pub const SYNC_AUTH_CHANGE_PASSWORD_RESPONSE: Self = Self::Sync("AuthChangePasswordResponse");

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
            Self::Sync(action) => format!("Sync.{}", action),
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
            Self::Sync(_) => "Sync",
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
            Self::Sync(a)
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
            "Sync.Ping" => Ok(Self::SYNC_PING),
            "Sync.AgentPatch" => Ok(Self::SYNC_AGENT_PATCH),
            "Sync.OrchestrationStatus" => Ok(Self::SYNC_ORCHESTRATION_STATUS),
            "Sync.McpToolResult" => Ok(Self::SYNC_MCP_TOOL_RESULT),
            "Sync.AgentStreamingChunk" => Ok(Self::SYNC_AGENT_STREAMING_CHUNK),
            "Sync.AgentReport" => Ok(Self::SYNC_AGENT_REPORT),
            "Sync.AskHumanRequest" => Ok(Self::SYNC_ASK_HUMAN_REQUEST),
            "Sync.UserMessage" => Ok(Self::SYNC_USER_MESSAGE),
            "Sync.AgentResponse" => Ok(Self::SYNC_AGENT_RESPONSE),
            "Sync.RequestFullSnapshot" => Ok(Self::SYNC_REQUEST_FULL_SNAPSHOT),
            "Sync.RequestGlobalSnapshot" => Ok(Self::SYNC_REQUEST_GLOBAL_SNAPSHOT),
            "Sync.GlobalSnapshot" => Ok(Self::SYNC_GLOBAL_SNAPSHOT),
            "Sync.ModelsSnapshot" => Ok(Self::SYNC_MODELS_SNAPSHOT),
            "Sync.ProvidersSnapshot" => Ok(Self::SYNC_PROVIDERS_SNAPSHOT),
            "Sync.ContainerSnapshot" => Ok(Self::SYNC_CONTAINER_SNAPSHOT),
            "Sync.ContainerPatch" => Ok(Self::SYNC_CONTAINER_PATCH),
            "Sync.TaskPatch" => Ok(Self::SYNC_TASK_PATCH),
            "Sync.TasksSnapshot" => Ok(Self::SYNC_TASKS_SNAPSHOT),
            "Sync.ListAgents" => Ok(Self::SYNC_LIST_AGENTS),
            "Sync.ServerVersion" => Ok(Self::SYNC_SERVER_VERSION),
            "Sync.OpenGitWorkspace" | "Sync.OpenWorkspace" => Ok(Self::SYNC_OPEN_WORKSPACE),
            "Sync.WorkspaceStatus" => Ok(Self::SYNC_WORKSPACE_STATUS),
            "Sync.RequestWorkspaceStatus" => Ok(Self::SYNC_REQUEST_WORKSPACE_STATUS),
            "Sync.SystemMessage" => Ok(Self::SYNC_SYSTEM_MESSAGE),
            "Sync.WebUiControl" => Ok(Self::SYNC_WEBUI_CONTROL),
            "Sync.WebUiControlResponse" => Ok(Self::SYNC_WEBUI_CONTROL_RESPONSE),
            "Sync.WebUiStatus" => Ok(Self::SYNC_WEBUI_STATUS),
            "Sync.RequestWebUiStatus" => Ok(Self::SYNC_REQUEST_WEBUI_STATUS),
            "Sync.AuthLogin" => Ok(Self::SYNC_AUTH_LOGIN),
            "Sync.AuthLoginResponse" => Ok(Self::SYNC_AUTH_LOGIN_RESPONSE),
            "Sync.AuthRegister" => Ok(Self::SYNC_AUTH_REGISTER),
            "Sync.AuthRegisterResponse" => Ok(Self::SYNC_AUTH_REGISTER_RESPONSE),
            "Sync.AuthListUsers" => Ok(Self::SYNC_AUTH_LIST_USERS),
            "Sync.AuthListUsersResponse" => Ok(Self::SYNC_AUTH_LIST_USERS_RESPONSE),
            "Sync.AuthGetUser" => Ok(Self::SYNC_AUTH_GET_USER),
            "Sync.AuthGetUserResponse" => Ok(Self::SYNC_AUTH_GET_USER_RESPONSE),
            "Sync.AuthDeleteUser" => Ok(Self::SYNC_AUTH_DELETE_USER),
            "Sync.AuthDeleteUserResponse" => Ok(Self::SYNC_AUTH_DELETE_USER_RESPONSE),
            "Sync.AuthChangePassword" => Ok(Self::SYNC_AUTH_CHANGE_PASSWORD),
            "Sync.AuthChangePasswordResponse" => Ok(Self::SYNC_AUTH_CHANGE_PASSWORD_RESPONSE),
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

                let wire_prefix = type_name;
                let method = format!("{}.{}", wire_prefix, action);

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
        }
        _ => ("Unknown.Unknown".to_string(), Some(json)),
    }
}

pub fn from_jsonrpc_method(method: &str, params: Option<Value>) -> Option<CoreMessage> {
    let (wire_prefix, action) = method.split_once('.')?;
    let type_name = wire_prefix;

    let data = match params {
        Some(Value::Object(mut map)) => {
            map.insert(
                BridgeKey::Action.as_ref().to_string(),
                Value::String(action.to_string()),
            );
            Value::Object(map)
        }
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
    use _state_sync::gateway::{BaseMessage, SyncMessage};
    use anyhow::Context;

    #[test]
    fn roundtrip_user_message() -> anyhow::Result<()> {
        let msg = CoreMessage::Sync(SyncMessage::Ping {
            timestamp: 12345u64,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_PING.as_str());

        let reconstructed =
            from_jsonrpc_method(&method, params).context("failed to reconstruct Ping message")?;
        match reconstructed {
            CoreMessage::Sync(SyncMessage::Ping { timestamp }) => {
                assert_eq!(timestamp, 12345u64);
            }
            other => anyhow::bail!("Expected {}, got {:?}", GatewayMethod::SYNC_PING, other),
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
            }
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
        let msg = CoreMessage::Sync(SyncMessage::OpenWorkspace {
            uri: "git://https://github.com/org/repo.git".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_OPEN_WORKSPACE.as_str());

        let reconstructed =
            from_jsonrpc_method(&method, params).context("failed to reconstruct OpenWorkspace")?;
        match reconstructed {
            CoreMessage::Sync(SyncMessage::OpenWorkspace { uri }) => {
                assert_eq!(uri, "git://https://github.com/org/repo.git");
            }
            other => anyhow::bail!("Expected OpenWorkspace, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn roundtrip_workspace_status() -> anyhow::Result<()> {
        let test_ws_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000123")
            .context("test precondition")?;
        let msg = CoreMessage::Sync(SyncMessage::WorkspaceStatus {
            workspace_id: test_ws_id,
            display_name: Some("my-repo".to_string()),
            connection_kind: "git".to_string(),
            resolved_path: Some("/tmp/workspaces/ws-123".to_string()),
            remote_url: Some("https://github.com/org/repo.git".to_string()),
            branch: Some("main".to_string()),
            host_id: None,
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_WORKSPACE_STATUS.as_str());

        let reconstructed = from_jsonrpc_method(&method, params)
            .context("failed to reconstruct WorkspaceStatus")?;
        match reconstructed {
            CoreMessage::Sync(SyncMessage::WorkspaceStatus {
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
            }
            other => anyhow::bail!("Expected WorkspaceStatus, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_open_workspace() -> anyhow::Result<()> {
        let method: GatewayMethod = "Sync.OpenWorkspace".parse()?;
        assert_eq!(method.as_str(), GatewayMethod::SYNC_OPEN_WORKSPACE.as_str());
        Ok(())
    }

    #[test]
    fn gateway_method_parse_workspace_status() -> anyhow::Result<()> {
        let method: GatewayMethod = "Sync.WorkspaceStatus".parse()?;
        assert_eq!(
            method.as_str(),
            GatewayMethod::SYNC_WORKSPACE_STATUS.as_str()
        );
        Ok(())
    }

    #[test]
    fn roundtrip_system_message() -> anyhow::Result<()> {
        let msg = CoreMessage::Sync(SyncMessage::SystemMessage {
            notification: _state_sync::SystemNotification::WebUiStarted,
            timestamp: "2026-05-11T12:00:00Z".to_string(),
        });
        let (method, params) = core_message_to_method_and_params(&msg);
        assert_eq!(method, GatewayMethod::SYNC_SYSTEM_MESSAGE.as_str());
        let reconstructed =
            from_jsonrpc_method(&method, params).context("system message roundtrip")?;
        match reconstructed {
            CoreMessage::Sync(SyncMessage::SystemMessage {
                notification,
                timestamp,
            }) => {
                assert_eq!(notification, _state_sync::SystemNotification::WebUiStarted);
                assert_eq!(timestamp, "2026-05-11T12:00:00Z");
            }
            other => anyhow::bail!("Expected SystemMessage, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn gateway_method_parse_methods() -> anyhow::Result<()> {
        for (s, expected) in [
            ("Sync.SystemMessage", GatewayMethod::SYNC_SYSTEM_MESSAGE),
            ("Sync.AuthLogin", GatewayMethod::SYNC_AUTH_LOGIN),
            (
                "Sync.AuthLoginResponse",
                GatewayMethod::SYNC_AUTH_LOGIN_RESPONSE,
            ),
        ] {
            let method: GatewayMethod = s.parse()?;
            assert_eq!(method.as_str(), expected.as_str());
        }
        Ok(())
    }
}
