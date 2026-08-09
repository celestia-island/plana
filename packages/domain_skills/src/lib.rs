//! Agent skill framework — tools, skill types, SOC executor, registry.
//!
//! This crate defines the runtime model for agent skills: what a skill is, how
//! it is registered, how its tools are invoked, and how execution is wrapped
//! in SOC (Security Operations Center) process management.
//!
//! Key abstractions:
//! - [`Skill`] / [`SkillRegistry`] — prompt-file-based skill definitions with
//!   front-matter metadata parsing and required-tool validation.
//! - [`Tool`] / [`ToolRegistry<M>`] — per-agent (phantom-typed) tool registry;
//!   each tool has a name, schema, and async invoke method. The
//!   [`GlobalToolRegistry`] aggregates across all agents.
//! - [`ToolResult`] / [`ToolInvoker`] — standardized tool invocation result
//!   and the invoker trait that bridges tools to the host runtime.
//! - [`SOCSkillExecutor`] — wraps skill execution with SOC stage tracking
//!   (prepare, verify, archive) for audit and compliance.
//! - [`define_agent_skills!`] macro — compile-time skill initialization from
//!   embedded prompt files, with tool-availability filtering.
//!
//! Design philosophy: skills are declarative (prompts + metadata), tools are
//! trait-objects behind phantom-type safety, and the SOC layer adds process
//! governance without coupling to the skill implementation.
#![allow(clippy::type_complexity)]

pub mod llm_subcall;
pub mod skill_types;
pub mod soc_executor;
pub mod tool_macros;
pub mod tool_names;
pub mod tool_permissions;
pub mod tool_registry;
pub mod tool_trait;
pub mod tools;
pub mod trigger_types;

pub use skill_types::{Skill, SkillInvoker, SkillRegistry, SkillResult};
pub use tool_names::{
    ParsedToolCall, agent_allowed_tools, agent_tools, aporia, cosmos, eleos, epieikeia, haplotes,
    hubris, kalos, neikos, orexis, philia, polemos, skemma, skopeo, web_automation,
};
pub use tool_permissions::{CommandSafety, TrustLevel, check_command_safety, classify_command};
pub use tool_registry::{GlobalToolRegistry, ToolRegistry};
pub use tool_trait::{ErasedTool, Tool, ToolDescriptor, ToolSchema};
pub use tools::{
    NaturalLanguageFormatter, SnapshotPolicy, ToolInvoker, ToolResult, validate_required_params,
};

#[macro_export]
macro_rules! define_agent_skills {
    ($agent:expr, $($name:ident => $path:literal),* $(,)?) => {
        pub async fn initialize_skills(registry: &$crate::SkillRegistry) -> ::anyhow::Result<()> {
            let available_tools = $crate::agent_tools($agent);
            $(
                let skill = $crate::Skill::from_include_str(include_str!($path), &available_tools)?;
                registry.register(skill).await;
            )*
            Ok(())
        }
    }
}
