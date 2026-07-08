//! Skill Feature Registry
//!
//! Provides a trait-based system for declaring and checking skill execution features.
//! Each feature type (boolean, enum, tool list) implements `FeatureRequirement`.
//! The `FeatureRegistry` aggregates all requirements for a skill and checks them
//! against a `FeatureContext` after execution.

use std::collections::HashSet;

use _core::ExecutionMode;

/// Context provided to feature checkers after skill execution.
#[derive(Debug, Clone)]
pub struct FeatureContext {
    /// All tools called during execution, in `agent::tool` format (e.g. `kalos::file_write`)
    pub called_tools: HashSet<String>,
    /// Whether any file write occurred
    pub has_file_write: bool,
    /// Whether any write dispatch (host_command_exec with writes) occurred
    pub has_write_dispatch: bool,
    /// Whether verification (cargo check etc.) ran
    pub has_verification: bool,
    /// Execution mode of the skill
    pub execution_mode: ExecutionMode,
}

/// Result of a feature check.
#[derive(Debug, Clone)]
pub enum FeatureCheckResult {
    /// Requirement satisfied.
    Satisfied,
    /// Requirement not satisfied, with a human-readable reason.
    Failed(String),
}

/// A single feature requirement that can be checked against a context.
pub trait FeatureRequirement: Send + Sync {
    /// Unique feature name (e.g. `must_use_at_least_once`, `execution_mode`).
    fn name(&self) -> &str;
    /// Check this requirement against the execution context.
    fn check(&self, ctx: &FeatureContext) -> FeatureCheckResult;
}

// ─── Concrete feature implementations ───

/// Requires that at least one of the listed tools was called.
/// Tool names use `agent::tool` format (e.g. `kalos::file_write`).
pub struct MustUseAtLeastOnce(pub Vec<String>);

impl FeatureRequirement for MustUseAtLeastOnce {
    fn name(&self) -> &str {
        "must_use_at_least_once"
    }

    fn check(&self, ctx: &FeatureContext) -> FeatureCheckResult {
        let missing: Vec<&str> = self
            .0
            .iter()
            .filter(|req| !ctx.called_tools.contains(*req))
            .map(|s| s.as_str())
            .collect();
        if missing.is_empty() {
            FeatureCheckResult::Satisfied
        } else {
            FeatureCheckResult::Failed(format!("Required tools not used: {}", missing.join(", ")))
        }
    }
}

/// Validates that the execution mode matches expectation.
pub struct RequireExecutionMode(pub ExecutionMode);

impl FeatureRequirement for RequireExecutionMode {
    fn name(&self) -> &str {
        "execution_mode"
    }

    fn check(&self, ctx: &FeatureContext) -> FeatureCheckResult {
        if ctx.execution_mode == self.0 {
            FeatureCheckResult::Satisfied
        } else {
            FeatureCheckResult::Failed(format!(
                "Expected execution_mode {:?}, got {:?}",
                self.0, ctx.execution_mode
            ))
        }
    }
}

/// Validates that file write occurred.
pub struct RequireFileWrite;

impl FeatureRequirement for RequireFileWrite {
    fn name(&self) -> &str {
        "require_file_write"
    }

    fn check(&self, ctx: &FeatureContext) -> FeatureCheckResult {
        if ctx.has_file_write || ctx.has_write_dispatch {
            FeatureCheckResult::Satisfied
        } else {
            FeatureCheckResult::Failed("No file write or write dispatch detected".to_string())
        }
    }
}

/// Validates that verification ran after writes.
pub struct RequireVerification;

impl FeatureRequirement for RequireVerification {
    fn name(&self) -> &str {
        "require_verification"
    }

    fn check(&self, ctx: &FeatureContext) -> FeatureCheckResult {
        if ctx.has_verification {
            FeatureCheckResult::Satisfied
        } else {
            FeatureCheckResult::Failed("No verification command detected after writes".to_string())
        }
    }
}

/// Registry holding all feature requirements for a skill.
pub struct FeatureRegistry {
    requirements: Vec<Box<dyn FeatureRequirement>>,
}

impl FeatureRegistry {
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    pub fn with(mut self, req: Box<dyn FeatureRequirement>) -> Self {
        self.requirements.push(req);
        self
    }

    /// Build a registry from parsed `PromptFeatures`.
    pub fn from_features(features: &crate::PromptFeatures) -> Self {
        let mut reg = Self::new();
        if !features.must_use_at_least_once.is_empty() {
            reg = reg.with(Box::new(MustUseAtLeastOnce(
                features.must_use_at_least_once.clone(),
            )));
        }
        reg
    }

    /// Check all requirements. Returns all failures (empty = all satisfied).
    pub fn check_all(&self, ctx: &FeatureContext) -> Vec<FeatureCheckResult> {
        self.requirements
            .iter()
            .filter_map(|req| match req.check(ctx) {
                FeatureCheckResult::Satisfied => None,
                failed => Some(failed),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}
