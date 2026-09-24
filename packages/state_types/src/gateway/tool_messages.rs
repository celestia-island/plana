//! Tool and skill invocation DTOs (`Tool.*`, `Skill.*`).
//!
//! Both families share one shape: call with parameters and an owning
//! `agent_type`, list without naming an agent, and answer with a payload of
//! `ToolInfo`/`SkillInfo` records or an opaque JSON result.

use serde::{Deserialize, Serialize};

use crate::{
    agent::Agent,
    tools::{SkillInfo, ToolInfo},
};

/// One `Tool`-namespace action, serde-tagged by `action`.
///
/// Wire shape: `{"type": "Tool", "data": {"action": "CallTool", ...}}`. The
/// bridge has constants for `CallTool`, `ListTools` and `ToolsListResponse`;
/// `ToolResponse` has none and parses back as `GatewayMethod::Extension`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ToolMessage {
    /// `Tool.CallTool` — run `tool_name`, owned by `agent_type`, with an opaque
    /// `parameters` object. The typed method catalog declares it `AsyncReq`, so
    /// the result does not come back on the request id.
    CallTool {
        tool_name: String,
        agent_type: Agent,
        parameters: serde_json::Value,
    },
    /// `Tool.ToolResponse` — the call result as opaque JSON. Its inner shape is
    /// tool-defined; `json_keys::ResponseKey` names the conventional
    /// `success`/`error`/`data`/`stdout`/`path` keys an implementation may use.
    ToolResponse {
        /// Tool-defined result payload, passed through as opaque JSON.
        result: serde_json::Value,
    },
    /// `Tool.ListTools` — list the tools of one `agent_type`, or of every agent
    /// when the field is `None` or absent.
    ListTools {
        /// Restrict the listing to one agent; `None` or absent lists every
        /// agent.
        agent_type: Option<Agent>,
    },
    /// `Tool.ToolsListResponse` — the catalog as `ToolInfo` records (name,
    /// description, owning agent, parameter schema, tier, visibility).
    ToolsListResponse {
        /// The matching tool records, in the order the serving agent returned
        /// them.
        tools: Vec<ToolInfo>,
    },
}

/// One `Skill`-namespace action, serde-tagged by `action`.
///
/// The mirror image of `ToolMessage` with the same four verbs: a call carries
/// an owning `agent_type` too, because one skill name can exist under several
/// agents. Constrained like the tool side: `Skill.SkillResponse` has no bridge
/// constant and parses back as `GatewayMethod::Extension`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SkillMessage {
    /// `Skill.CallSkill` — start `skill_name` owned by `agent_type` with an opaque
    /// `parameters` object; declared `AsyncReq`, so the result is not returned on
    /// the request id.
    CallSkill {
        skill_name: String,
        agent_type: Agent,
        parameters: serde_json::Value,
    },
    /// `Skill.SkillResponse` — the skill run result as opaque JSON, shaped by the
    /// skill rather than by this DTO.
    SkillResponse {
        /// Skill-defined result payload, opaque to this DTO.
        result: serde_json::Value,
    },
    /// `Skill.ListSkills` — list the skills of one `agent_type`, or of every
    /// agent when the field is `None` or absent.
    ListSkills {
        /// Restrict the listing to one agent; `None` or absent lists every
        /// agent.
        agent_type: Option<Agent>,
    },
    /// `Skill.SkillsListResponse` — the catalog as `SkillInfo` records (name,
    /// per-language descriptions, owning agent, required tools).
    SkillsListResponse {
        /// The matching skill records, in the order the serving agent returned
        /// them.
        skills: Vec<SkillInfo>,
    },
}
