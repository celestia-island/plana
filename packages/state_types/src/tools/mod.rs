//! Tool definitions and the prompt-side machinery around them.
//!
//! `ToolInfo` / `ToolParameters` describe a tool to the model and to the
//! router, `ToolCallRequest` / `ToolCallResponse` are the invocation envelopes,
//! and `ToolConfig` / `ToolPromptInjector` decide which mandatory prompt text
//! reaches the system prompt. Todo markers (`MarkedTodoItem`, `TodoMarker`) and
//! the compression payload (`CompressedContext`, `PreserveState`) live here as
//! well, because they travel with the same tool-calling state.

/// JSON Schema plumbing for tool parameters: the `type` literals, plus the
/// strict-mode rewrite that `ToolInfo::to_tool_definition` applies before a
/// schema is handed to the model.
pub mod schema;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::ModelTier;
use plana_core::ToolDefinition;

fn default_param_type() -> String {
    schema::default_param_type()
}

/// Where a tool (or skill) executes: on the node-side scepter runtime, or in
/// the Cosmos microkernel that hosts the agent-agnostic `exec` /
/// `write_to_var` tools. Wire form is the lower-case variant name
/// (`rename_all = "snake_case"`), and `Scepter` is what an absent key means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolLocation {
    /// Node-side scepter runtime: the default, and the host of every tool that is
    /// not explicitly routed to Cosmos.
    #[default]
    Scepter,
    /// Cosmos microkernel (its own nested container runtime): the agent-agnostic
    /// tool surface that other tools are invoked through.
    Cosmos,
}

/// Skills reuse the tool location enum, so a skill declares its host with the
/// same two values; `plana_domain_skills` falls back to `Scepter` when the
/// skill metadata leaves it unset.
pub type SkillLocation = ToolLocation;

/// How a tool call is dispatched: `Blocking` (the default), `FireAndForget`
/// or `AsyncCallback`. Defined in `plana_core` and re-exported here so tool
/// specs only need this crate.
pub use plana_core::ToolCallMode;

/// Which gate a tool must clear before it reaches the model.
///
/// Nothing in this crate applies the gate: the runtime resolves the allow-flags
/// for the current turn and asks `ToolVisibility::is_visible_with`. The field
/// that carries it defaults to `Always`, and the wire form is the lower-case
/// variant name (`always`, `skill_whitelist`, `tool_explore`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolVisibility {
    /// Offered whenever the caller allows always-visible tools; the default.
    #[default]
    Always,
    /// Offered only when the caller reports the skill whitelist as allowed, that is
    /// while the tool is on the active skill's list.
    SkillWhitelist,
    /// Offered only when the caller reports tool-explore mode as allowed.
    ToolExplore,
}

impl ToolVisibility {
    /// Whether a tool carrying this visibility may be offered, given the three
    /// allow-flags the caller resolved for the current turn. Returns the flag
    /// matching this variant and ignores the other two; the flags are not stored,
    /// so the caller owns both the policy and the default.
    pub fn is_visible_with(
        &self,
        allow_always: bool,
        allow_skill_whitelist: bool,
        allow_tool_explore: bool,
    ) -> bool {
        match self {
            Self::Always => allow_always,
            Self::SkillWhitelist => allow_skill_whitelist,
            Self::ToolExplore => allow_tool_explore,
        }
    }
}

/// How far along a tool is in its lifecycle. Operator-facing metadata only:
/// `to_tool_definition` does not filter on it, so a `Stub` or `Deprecated` tool
/// is still offered to the model. Specs declared through `agent_tool_module!`
/// fill it from their `maturity` key; the wire form is the lower-case name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolMaturity {
    /// Supported and expected to stay compatible; the default.
    #[default]
    Stable,
    /// Usable but still changing; arguments or results may shift.
    Experimental,
    /// Registered for routing or documentation purposes, with no working
    /// implementation behind it.
    Stub,
    /// Superseded: consumers should stop offering it. The variant still parses,
    /// so registrations that name it keep deserializing.
    Deprecated,
}

/// One tool as advertised by an agent: the model-facing name, description and
/// parameter schema, plus the routing metadata the runtime needs in order to
/// decide whether and where to offer it. Built by `ToolInfo::simple` or by
/// `agent_tool_module!`, and turned into the model-facing definition with
/// `to_tool_definition`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Name the model must call; also the lookup key for the tool's markdown doc
    /// (`ToolDocLoader::load`) and the prefix of the follow-up syntax
    /// `name.param("value")` that `to_tool_definition` advertises for
    /// `separate_call_keys`.
    pub name: String,
    /// Model-facing description; `ToolDocLoader::enrich_tool_info` may replace it
    /// with the localized text from the tool's doc file.
    pub description: String,
    /// Agent kind that owns and serves the tool. A request is routed by this field
    /// as well as by the tool name.
    pub agent_type: super::agent::Agent,
    /// Parameter schema; see `ToolParameters`.
    pub parameters: ToolParameters,
    /// Minimum model tier the tool needs, when a spec pins one; `serde(default)`
    /// means an absent key and an explicit `null` both decode to `None`.
    #[serde(default)]
    pub tier: Option<ModelTier>,
    /// Runtime host for the tool; `serde(default)` = `Scepter` when absent.
    #[serde(default)]
    pub location: ToolLocation,
    /// How the runtime dispatches the call: `Blocking` (the default),
    /// `FireAndForget`, or `AsyncCallback` (`plana_core::ToolCallMode`).
    #[serde(default)]
    pub call_mode: ToolCallMode,
    /// Gate this tool must clear before it is offered; `serde(default)` = `Always`.
    #[serde(default)]
    pub visibility: ToolVisibility,
    /// Marks a long-running tool (set by `async_tool`). Independent of `call_mode`
    /// and absent from the generated schema, so only the runtime reads it.
    #[serde(default)]
    pub is_async: bool,
    /// Lifecycle marker; `serde(default)` = `Stable`, and nothing filters on it.
    #[serde(default)]
    pub maturity: ToolMaturity,
}

/// Parameter schema for a tool, shaped like the OpenAI-compatible tool-calling
/// contract: a `type` / `properties` / `required` object, plus the list of keys
/// that may instead arrive in a follow-up call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolParameters {
    /// The schema's `type` key (renamed to `type` on the wire); an absent key
    /// decodes to `object` through `default_param_type`.
    #[serde(rename = "type", default = "default_param_type")]
    pub param_type: String,
    /// Names of the parameters the model must supply. They are written into the
    /// schema's `required` array only on the separate-call path; the strict
    /// normalizer otherwise derives `required` from the property keys.
    #[serde(default)]
    pub required: Vec<String>,
    /// Parameter name to JSON Schema for that parameter; kept as raw
    /// `serde_json::Value` so tool specs can use any schema feature.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    /// Parameters the model may pass either in the JSON arguments or in a follow-up
    /// `tool.key(...)` call. A non-empty list switches `to_tool_definition` into
    /// non-strict mode and appends the syntax hint to the description.
    #[serde(default)]
    pub separate_call_keys: Vec<String>,
}

impl ToolParameters {
    /// Build a schema from an explicit type, required names and properties;
    /// `separate_call_keys` starts empty and has to be filled on the value.
    pub fn new(
        param_type: &str,
        required: Vec<String>,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            param_type: param_type.to_string(),
            required,
            properties,
            separate_call_keys: Vec::new(),
        }
    }
}

impl ToolInfo {
    /// Minimal constructor used by `agent_tool_module!`: an empty `object` schema
    /// requiring the names passed in, with everything else at its default (scepter
    /// host, blocking call, always visible, stable, no tier, synchronous).
    pub fn simple(
        name: &str,
        description: &str,
        agent_type: super::agent::Agent,
        required: Vec<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            agent_type,
            parameters: ToolParameters {
                param_type: "object".to_string(),
                required: required.iter().map(|s| s.to_string()).collect(),
                properties: HashMap::new(),
                separate_call_keys: Vec::new(),
            },
            tier: None,
            location: ToolLocation::default(),
            call_mode: ToolCallMode::default(),
            visibility: ToolVisibility::default(),
            is_async: false,
            maturity: ToolMaturity::default(),
        }
    }

    /// Replace the parameter schema wholesale, e.g. with one built by `ToolDocLoader`;
    /// it is normalized for strict mode later, inside `to_tool_definition`.
    pub fn with_params(mut self, parameters: ToolParameters) -> Self {
        self.parameters = parameters;
        self
    }

    /// Route the tool to another runtime host; `Scepter` is the value it starts with.
    pub fn with_location(mut self, location: ToolLocation) -> Self {
        self.location = location;
        self
    }

    /// Choose a non-blocking dispatch mode for this tool; `Blocking` is the value it
    /// starts with.
    pub fn with_call_mode(mut self, mode: ToolCallMode) -> Self {
        self.call_mode = mode;
        self
    }

    /// Record the model tier this tool needs; `tier` stays `None` until this is
    /// called.
    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = Some(tier);
        self
    }

    /// Set the gate the tool must clear before it is offered; `Always` unless called.
    pub fn with_visibility(mut self, visibility: ToolVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set the `is_async` flag. There is no inverse builder, so a tool marked async
    /// cannot be made synchronous again.
    pub fn async_tool(mut self) -> Self {
        self.is_async = true;
        self
    }

    /// Set the lifecycle marker; `Stable` unless called.
    pub fn with_maturity(mut self, maturity: ToolMaturity) -> Self {
        self.maturity = maturity;
        self
    }

    /// Turn this entry into the model-facing `plana_core::ToolDefinition`.
    ///
    /// With no `separate_call_keys` the parameter object is normalized for strict
    /// mode (every property becomes required and nullable) and `strict` is set to
    /// `true`. With them, `additionalProperties` is forced to `false` and `strict`
    /// is `false`, because the follow-up call supplies only the remaining arguments
    /// instead of one complete object.
    pub fn to_tool_definition(&self) -> ToolDefinition {
        let properties = self.parameters.properties.clone();
        let separate_keys = &self.parameters.separate_call_keys;
        let has_separate = !separate_keys.is_empty();

        let mut params_map = serde_json::Map::new();
        params_map.insert(
            "type".to_string(),
            serde_json::Value::String(self.parameters.param_type.clone()),
        );
        params_map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties.into_iter().collect()),
        );
        let mut params = serde_json::Value::Object(params_map);

        if has_separate {
            let required: Vec<serde_json::Value> = self
                .parameters
                .required
                .iter()
                .map(|r| serde_json::Value::String(r.clone()))
                .collect();
            if !required.is_empty()
                && let Some(obj) = params.as_object_mut()
            {
                obj.insert("required".to_string(), serde_json::Value::Array(required));
            }
            if let Some(obj) = params.as_object_mut() {
                obj.insert(
                    "additionalProperties".to_string(),
                    serde_json::Value::Bool(false),
                );
            }
        } else {
            schema::normalize_schema_for_strict(&mut params);
        }

        let mut description = self.description.clone();
        if has_separate {
            let fields: Vec<String> = separate_keys
                .iter()
                .map(|k| format!("{}.{}(\"...\")", self.name, k))
                .collect();
            description.push_str(&format!(
                "\n\nParameters [{}] can be provided either in the JSON arguments or via a separate follow-up call using the syntax tool_name.param_name(\"value\").",
                fields.join(", ")
            ));
        }

        ToolDefinition {
            name: self.name.clone(),
            description,
            parameters: params,
            call_mode: self.call_mode,
            strict: Some(!has_separate),
        }
    }
}

/// One tool invocation, addressed to whichever host serves the tool. The
/// `call_id` is the correlation key that the matching `ToolCallResponse` echoes
/// back, so concurrent calls on one connection stay distinguishable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Name of the tool to run, as advertised in `ToolInfo::name`.
    pub tool_name: String,
    /// Agent kind that owns the tool; lets a shared host route the call without a
    /// registry lookup.
    pub agent_type: super::agent::Agent,
    /// Raw JSON arguments as produced by the model; validating them against the tool
    /// schema is the host's job.
    pub parameters: serde_json::Value,
    /// Correlation id for this invocation, echoed by `ToolCallResponse::call_id`.
    pub call_id: Uuid,
    /// Workspace the call should run in. `None` means the host default, and
    /// `skip_serializing_if` keeps the key off the wire entirely in that case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_uri: Option<String>,
}

/// Outcome of a `ToolCallRequest`. `success` is authoritative: `result` is only
/// meaningful when it is `true`, and `error` only when it is `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// Echo of the request's `call_id`, so a host may answer calls out of order.
    pub call_id: Uuid,
    /// Tool result as JSON; the shape is tool-specific and may be `null` on failure.
    pub result: serde_json::Value,
    /// `true` when the tool ran to completion; `false` means `error` carries the
    /// reason.
    pub success: bool,
    /// Failure text when `success` is `false`. It has neither `serde(default)` nor
    /// `skip_serializing_if`, so it is always emitted (`null` when unset) and the
    /// derived decoder requires the key on input.
    pub error: Option<String>,
}

/// A skill as advertised by an agent: identity, localized descriptions, and the
/// tools it needs. Metadata only -- the prompt body lives in the skill's markdown
/// doc, looked up by `name` under the agent's `skills/` directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill identifier, unique within the agent; `serde(default)` = empty string,
    /// which marks an unnamed skill.
    #[serde(default)]
    pub name: String,
    /// Language code to description, `en` being the fallback; `serde(default)` =
    /// empty map. One skill is registered once but rendered in several UI languages,
    /// which is why the description is a map rather than a string.
    #[serde(default)]
    pub description: HashMap<String, String>,
    /// Agent that provides the skill.
    pub agent_type: super::agent::Agent,
    /// Names of the tools the skill needs in order to run; callers use the list to
    /// check the skill against the tools an agent actually offers.
    #[serde(default)]
    pub required_tools: Vec<String>,
    /// Minimum model tier the skill needs; `serde(default)` = `None` (no pin).
    #[serde(default)]
    pub tier: Option<ModelTier>,
    /// Runtime host for the skill, the same enum as for tools; `serde(default)` =
    /// `Scepter`.
    #[serde(default)]
    pub location: SkillLocation,
}

impl Default for SkillInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: HashMap::new(),
            agent_type: super::agent::Agent::ApoRia,
            required_tools: Vec::new(),
            tier: None,
            location: SkillLocation::default(),
        }
    }
}

impl SkillInfo {
    /// Wrap a single description into the language map used by `description`, keyed
    /// `en`; the constructor path for skills defined in source code.
    pub fn desc_from_str(s: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("en".to_string(), s.to_string());
        m
    }
}

/// When a tool's mandatory prompt is injected into the system prompt. The wire
/// form is the lower-case variant name (`always`, `on_first_use`,
/// `on_every_use`), and the default is `OnFirstUse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptInjectionPolicy {
    /// Inject on every turn, whether or not the tool has been used.
    Always,
    /// Inject until the tool has been used once, then stop; the default.
    #[default]
    OnFirstUse,
    /// Intended to re-inject on every turn once the tool has been used; today
    /// `should_inject` answers `true` unconditionally, exactly like `Always`.
    OnEveryUse,
}

/// Prompt-side configuration for a tool: the text that must appear in the system
/// prompt while the tool is available, and the policy for when it is injected.
/// Unlike `ToolInfo` it is never converted into a model tool schema; it only
/// feeds `ToolPromptInjector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Name of the tool this config belongs to; also the key recorded in the
    /// injector's used-tool set.
    pub tool_name: String,
    /// Human-readable description kept alongside the injected prompt.
    pub description: String,
    /// Text injected verbatim as the body of the `<tool_constraints>` section of the
    /// system prompt.
    pub mandatory_prompt: String,
    /// When `mandatory_prompt` is injected; `serde(default)` = `OnFirstUse`.
    #[serde(default)]
    pub injection_policy: PromptInjectionPolicy,
    /// Agent the config applies to; `ToolConfig::new` starts at `Agent::ApoRia`.
    pub agent_type: super::agent::Agent,
    /// Parameter schema. `serde(default)` builds `ToolParameters::default()`, that
    /// is an empty `param_type` with no properties -- not the `object` default that
    /// the schema's own `type` key carries.
    #[serde(default)]
    pub parameters: ToolParameters,
    /// Minimum model tier, when a spec pins one; `serde(default)` = `None`.
    #[serde(default)]
    pub tier: Option<ModelTier>,
    /// Runtime host for the tool; `serde(default)` = `Scepter`.
    #[serde(default)]
    pub location: ToolLocation,
}

impl ToolConfig {
    /// Build a config for a tool with the default policy (`OnFirstUse`), `ApoRia` as
    /// the agent, an empty parameter schema, no tier and the `Scepter` location; use
    /// the `with_*` steps to override any of them.
    pub fn new(tool_name: &str, description: &str, mandatory_prompt: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            description: description.to_string(),
            mandatory_prompt: mandatory_prompt.to_string(),
            injection_policy: PromptInjectionPolicy::default(),
            agent_type: super::agent::Agent::ApoRia,
            parameters: ToolParameters {
                param_type: "object".to_string(),
                required: vec![],
                properties: std::collections::HashMap::new(),
                separate_call_keys: Vec::new(),
            },
            tier: None,
            location: ToolLocation::default(),
        }
    }

    /// Set when the mandatory prompt is injected; `OnFirstUse` unless called.
    pub fn with_injection_policy(mut self, policy: PromptInjectionPolicy) -> Self {
        self.injection_policy = policy;
        self
    }

    /// Attribute the config to a specific agent. The kind is stored as given, without
    /// checking it against the agent registry.
    pub fn with_agent_type(mut self, agent_type: super::agent::Agent) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Record the model tier this tool needs; `tier` stays `None` until this is
    /// called.
    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = Some(tier);
        self
    }

    /// Route the tool to another runtime host; `Scepter` is the value it starts with.
    pub fn with_location(mut self, location: ToolLocation) -> Self {
        self.location = location;
        self
    }
}

/// Per-conversation bookkeeping for mandatory tool prompts; not serialized. It
/// remembers which tools already had their prompt injected so an `OnFirstUse`
/// policy fires at most once, which means it must be reset when the conversation
/// restarts or the prompts stay suppressed.
#[derive(Debug, Clone, Default)]
pub struct ToolPromptInjector {
    used_tools: HashSet<String>,
    // Only ever written as `None` (by `new` and `reset`); nothing reads it, so
    // no behaviour depends on this field today.
    current_prompt_section: Option<String>,
}

impl ToolPromptInjector {
    /// Empty injector: no tool marked as used.
    pub fn new() -> Self {
        Self {
            used_tools: HashSet::new(),
            current_prompt_section: None,
        }
    }

    /// Whether `config`'s mandatory prompt belongs in the system prompt right now.
    /// `Always` and `OnEveryUse` both answer `true` without consulting state;
    /// `OnFirstUse` answers `true` only while the tool is not marked used.
    pub fn should_inject(&self, config: &ToolConfig) -> bool {
        match config.injection_policy {
            PromptInjectionPolicy::Always => true,
            PromptInjectionPolicy::OnFirstUse => !self.used_tools.contains(&config.tool_name),
            PromptInjectionPolicy::OnEveryUse => true,
        }
    }

    /// The prompt text to inject for `config`, or `None` when `should_inject` says
    /// no. A `Some` answer marks the tool used as a side effect, so the next
    /// `OnFirstUse` check for it fails.
    pub fn prepare_tool_context(&mut self, config: &ToolConfig) -> Option<String> {
        if self.should_inject(config) {
            self.mark_tool_used(&config.tool_name);
            Some(config.mandatory_prompt.clone())
        } else {
            None
        }
    }

    /// Mark `tool_name` as used without injecting anything, which suppresses later
    /// `OnFirstUse` injections for it.
    pub fn mark_tool_used(&mut self, tool_name: &str) {
        self.used_tools.insert(tool_name.to_string());
    }

    /// Whether `tool_name` is marked used, either by `mark_tool_used` or by an
    /// injection through `prepare_tool_context`.
    pub fn has_used_tool(&self, tool_name: &str) -> bool {
        self.used_tools.contains(tool_name)
    }

    /// Forget every used tool and drop the remembered section; call this between
    /// conversations.
    pub fn reset(&mut self) {
        self.used_tools.clear();
        self.current_prompt_section = None;
    }

    /// Replace the `<tool_constraints>` section of `system_prompt` with
    /// `mandatory_prompt`, appending a fresh section when the prompt has none. A
    /// start marker without a matching end marker is left untouched, and the
    /// argument is modified in place.
    pub fn inject_to_system_prompt(&self, system_prompt: &mut String, mandatory_prompt: &str) {
        let section_name = "tool_constraints";
        let section_marker_start = format!("<{}>", section_name);
        let section_marker_end = format!("</{}>", section_name);

        let new_section = format!(
            "\n{}\n{}\n{}\n",
            section_marker_start, mandatory_prompt, section_marker_end
        );

        if let Some(start) = system_prompt.find(&section_marker_start) {
            if let Some(end) = system_prompt.find(&section_marker_end) {
                system_prompt.replace_range(start..end + section_marker_end.len(), &new_section);
            }
        } else {
            system_prompt.push_str(&new_section);
        }
    }

    /// The same replacement as `inject_to_system_prompt`, but on a copy: returns the
    /// rewritten prompt and never mutates the argument. When either marker is
    /// missing it falls back to appending the section.
    pub fn replace_tool_section(&self, system_prompt: &str, mandatory_prompt: &str) -> String {
        let section_name = "tool_constraints";
        let section_marker_start = format!("<{}>", section_name);
        let section_marker_end = format!("</{}>", section_name);

        let start_idx = system_prompt.find(&section_marker_start);
        let end_idx = system_prompt.find(&section_marker_end);

        match (start_idx, end_idx) {
            (Some(start), Some(end)) => {
                let new_section = format!(
                    "{}\n{}\n{}",
                    section_marker_start, mandatory_prompt, section_marker_end
                );
                format!(
                    "{}{}{}",
                    &system_prompt[..start],
                    new_section,
                    &system_prompt[end + section_marker_end.len()..]
                )
            }
            _ => {
                let mut result = system_prompt.to_string();
                self.inject_to_system_prompt(&mut result, mandatory_prompt);
                result
            }
        }
    }
}

/// One mark: which todo/task item to keep in view, how deep below it to look, and
/// which extras to pull in with it. Stored inside `TodoMarker` and serialized by
/// field name; walking the todo tree is the consumer's job, not this crate's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkedTodoItem {
    /// Id of the todo/task item the mark points at.
    pub todo_id: Uuid,
    /// How many descendant levels below `todo_id` to include; `serde(default)` = 1,
    /// so payloads without the key keep exactly one level.
    #[serde(default = "default_include_depth")]
    pub include_depth: u32,
    /// Also include the chain of ancestors above `todo_id`; `serde(default)` =
    /// `false`.
    #[serde(default)]
    pub include_ancestors: bool,
    /// Also include artifacts attached to the marked items; `serde(default)` =
    /// `false`.
    #[serde(default)]
    pub include_artifacts: bool,
}

fn default_include_depth() -> u32 {
    1
}

impl MarkedTodoItem {
    /// Mark `todo_id` with depth 1 and neither ancestors nor artifacts.
    pub fn new(todo_id: &Uuid) -> Self {
        Self {
            todo_id: *todo_id,
            include_depth: 1,
            include_ancestors: false,
            include_artifacts: false,
        }
    }

    /// Override the default depth of one descendant level.
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.include_depth = depth;
        self
    }

    /// Turn on `include_ancestors`; no builder turns it back off.
    pub fn with_ancestors(mut self) -> Self {
        self.include_ancestors = true;
        self
    }

    /// Turn on `include_artifacts`; no builder turns it back off.
    pub fn with_artifacts(mut self) -> Self {
        self.include_artifacts = true;
        self
    }

    /// Alias of `new`: a mark that pulls in nothing beyond the item's own id.
    pub fn simple(todo_id: &Uuid) -> Self {
        Self::new(todo_id)
    }
}

/// Which items `TodoMarker` should treat as marked. Only the choice is stored
/// here -- `get_strategy` hands it to the caller and no code in this crate reads
/// it, so the automatic variants describe intent the consumer implements. Wire
/// form is the lower-case variant name, default `hybrid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarkerStrategy {
    /// Only items explicitly passed to `mark` / `mark_multiple` count as marked.
    Manual,
    /// Intended to also treat critical items as marked.
    AutoCritical,
    /// Intended to also treat unfinished items as marked.
    AutoUnfinished,
    /// Default: the automatic rules combined with explicit marks.
    #[default]
    Hybrid,
}

/// Collector for the items a caller wants treated as marked, plus the strategy
/// that qualifies them. Marks are kept in insertion order and are not
/// de-duplicated, so the same id can appear more than once.
#[derive(Debug, Clone, Default)]
pub struct TodoMarker {
    marked_items: Vec<MarkedTodoItem>,
    marker_strategy: MarkerStrategy,
}

impl TodoMarker {
    /// Empty marker using the given strategy.
    pub fn new(strategy: MarkerStrategy) -> Self {
        Self {
            marked_items: Vec::new(),
            marker_strategy: strategy,
        }
    }

    /// Append one mark, without checking whether the id is already present.
    pub fn mark(&mut self, item: MarkedTodoItem) {
        self.marked_items.push(item);
    }

    /// Append every mark in order, without de-duplication.
    pub fn mark_multiple(&mut self, items: Vec<MarkedTodoItem>) {
        self.marked_items.extend(items);
    }

    /// The marks in insertion order, for the consumer to walk.
    pub fn get_marked_items(&self) -> &[MarkedTodoItem] {
        &self.marked_items
    }

    /// Drop every mark; the strategy is left as it was.
    pub fn clear(&mut self) {
        self.marked_items.clear();
    }

    /// The strategy this marker was built with (the enum is `Copy`).
    pub fn get_strategy(&self) -> MarkerStrategy {
        self.marker_strategy
    }

    /// Replace the strategy without touching the marks.
    pub fn set_strategy(&mut self, strategy: MarkerStrategy) {
        self.marker_strategy = strategy;
    }
}

/// What survives when a conversation is compressed: the two system prompts and
/// the original user input are carried over verbatim, and anything that must not
/// be summarized away travels in `preserved_state`. Serialized by field name,
/// with every field required on input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedContext {
    /// The agent's soul/system prompt as of the compression point.
    pub soul_prompt: String,
    /// The active skill's prompt section as of the compression point.
    pub skill_prompt: String,
    /// The conversation's first user message, preserved verbatim.
    pub initial_user_input: String,
    /// State lifted out of the conversation that has to outlive the compression.
    pub preserved_state: PreserveState,
}

/// State carried across a context compression: the todo tree that was active and
/// the decisions that must not be summarized away. `Default` is the empty state,
/// and neither field is optional on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreserveState {
    /// Id of the active todo tree root, or `None` when no tree was in play; the key
    /// is always serialized (`null` when unset).
    pub todo_tree_root: Option<String>,
    /// Decisions to keep verbatim, in insertion order; may be empty.
    pub critical_decisions: Vec<String>,
}

impl Default for PreserveState {
    fn default() -> Self {
        Self::new()
    }
}

impl PreserveState {
    /// Empty state: no todo root and no decisions.
    pub fn new() -> Self {
        Self {
            todo_tree_root: None,
            critical_decisions: Vec::new(),
        }
    }

    /// Set `todo_tree_root`; called twice, the last root wins.
    pub fn with_todo_root(mut self, root: &str) -> Self {
        self.todo_tree_root = Some(root.to_string());
        self
    }

    /// Append one decision; repeated calls accumulate them in insertion order.
    pub fn add_decision(mut self, decision: &str) -> Self {
        self.critical_decisions.push(decision.to_string());
        self
    }
}
