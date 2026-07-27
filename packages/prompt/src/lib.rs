//! Prompt loading, templating, and agent soul management.
//!
//! [`PromptLoader`] discovers and parses markdown front-matter prompts from the
//! filesystem, yielding [`PromptMetadata`] (name, description, tags, execution
//! mode) and [`PromptTemplate`] for variable interpolation.
//!
//! [`SoulLoader`] loads per-agent per-language soul documents ("soul.md") through
//! a pluggable [`ContentProvider`] trait, with a global singleton set at startup.
//! [`PromptTemplateService`] renders templates, merging agent context, skills,
//! and execution-mode constraints. Together these form the content layer that
//! constructs the system prompt and personality of every agent.
#![allow(clippy::type_complexity)]

pub mod features;
pub mod front_matter;
pub mod prompt_loader;
pub mod prompt_template;
pub mod soul_loader;

pub use features::{
    FeatureCheckResult, FeatureContext, FeatureRegistry, FeatureRequirement, MustUseAtLeastOnce,
    RequireExecutionMode, RequireFileWrite, RequireVerification,
};
pub use prompt_loader::{PromptFeatures, PromptLoader, PromptMetadata, PromptTemplate};
pub use prompt_template::PromptTemplateService;
