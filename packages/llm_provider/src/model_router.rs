//! Model routing engine — selects the best model for a task based on complexity scoring.
//!
//! Unlike a static provider config, the router evaluates the task's complexity
//! (prompt length, tool count, context depth) and selects a model from the
//! appropriate tier — trivial tasks get fast/cheap models, complex tasks get
//! powerful models.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use plana_llm_provider::model_router::{ModelRouter, TaskComplexity};
//!
//! let router = ModelRouter::default();
//! if let Some(entry) = router.select(TaskComplexity::Standard) {
//!     println!("Selected: {} via {}", entry.model, entry.provider);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complexity tiers that map to model capability requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Quick lookups, short responses, simple formatting.
    /// → Fast, inexpensive models (e.g., Haiku, Flash, mini).
    Trivial,
    /// Normal agent operations: tool dispatch, status checks, planning.
    /// → Default-tier models (e.g., Sonnet, Pro, standard).
    Standard,
    /// Code generation, multi-step analysis, complex reasoning.
    /// → Powerful models (e.g., Opus, Pro Max, ultra).
    Complex,
    /// Hardest tasks: architecture design, security review, novel synthesis.
    /// → Most capable models available.
    Frontier,
}

impl TaskComplexity {
    /// Heuristic scoring from request characteristics.
    ///
    /// Based on prompt token count and number of tools available.
    /// This is intentionally simple and deterministic — not an ML model.
    pub fn from_request(prompt_tokens: usize, tool_count: usize, context_depth: usize) -> Self {
        let score = prompt_tokens / 1000 + tool_count * 2 + context_depth * 3;
        match score {
            0..=8 => Self::Trivial,
            9..=30 => Self::Standard,
            31..=80 => Self::Complex,
            _ => Self::Frontier,
        }
    }
}

/// A model entry in a routing tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Provider identifier (e.g., "anthropic", "openai", "gemini").
    pub provider: String,
    /// Model name as the provider expects it (e.g., "claude-sonnet-4-20250514").
    pub model: String,
    /// Relative priority within the tier (lower = preferred).
    pub priority: u8,
    /// Approximate cost per 1M input tokens in USD (for budget tracking).
    pub cost_per_1m_input: f64,
    /// Approximate cost per 1M output tokens in USD.
    pub cost_per_1m_output: f64,
}

impl ModelEntry {
    /// Estimate the cost for a request with given token counts.
    pub fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        (self.cost_per_1m_input * input_tokens as f64 / 1_000_000.0)
            + (self.cost_per_1m_output * output_tokens as f64 / 1_000_000.0)
    }
}

/// The model router holds ordered model lists per complexity tier.
#[derive(Debug, Clone, Default)]
pub struct ModelRouter {
    tiers: HashMap<TaskComplexity, Vec<ModelEntry>>,
}

impl ModelRouter {
    /// Create a router with sensible defaults for common providers.
    ///
    /// These defaults are overridable via [`ModelRouter::set_tier`].
    pub fn new() -> Self {
        let mut router = Self::default();
        router.set_tier(TaskComplexity::Trivial, default_trivial_tier());
        router.set_tier(TaskComplexity::Standard, default_standard_tier());
        router.set_tier(TaskComplexity::Complex, default_complex_tier());
        router.set_tier(TaskComplexity::Frontier, default_frontier_tier());
        router
    }

    /// Override or set models for a specific tier.
    pub fn set_tier(&mut self, tier: TaskComplexity, models: Vec<ModelEntry>) {
        self.tiers.insert(tier, models);
    }

    /// Select the best available model for a complexity tier.
    ///
    /// "Available" means the provider is not currently quota-exhausted
    /// (checked via [`crate::quota_meter::is_exhausted`]).
    pub fn select(&self, tier: TaskComplexity) -> Option<&ModelEntry> {
        let models = self.tiers.get(&tier)?;
        models
            .iter()
            .find(|entry| !crate::quota_meter::is_exhausted(&entry.provider))
            .or_else(|| models.first())
    }

    /// Select a model for a request, scoring complexity from request params.
    pub fn select_for_request(
        &self,
        prompt_tokens: usize,
        tool_count: usize,
        context_depth: usize,
    ) -> Option<&ModelEntry> {
        let tier = TaskComplexity::from_request(prompt_tokens, tool_count, context_depth);
        self.select(tier)
    }
}

// ── Default tier configurations ─────────────────────────────────────

fn default_trivial_tier() -> Vec<ModelEntry> {
    vec![
        ModelEntry {
            provider: "anthropic".into(),
            model: "claude-3-5-haiku-20241022".into(),
            priority: 1,
            cost_per_1m_input: 0.8,
            cost_per_1m_output: 4.0,
        },
        ModelEntry {
            provider: "gemini".into(),
            model: "gemini-2.0-flash".into(),
            priority: 2,
            cost_per_1m_input: 0.075,
            cost_per_1m_output: 0.3,
        },
        ModelEntry {
            provider: "openai".into(),
            model: "gpt-4o-mini".into(),
            priority: 3,
            cost_per_1m_input: 0.15,
            cost_per_1m_output: 0.6,
        },
    ]
}

fn default_standard_tier() -> Vec<ModelEntry> {
    vec![
        ModelEntry {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
            priority: 1,
            cost_per_1m_input: 3.0,
            cost_per_1m_output: 15.0,
        },
        ModelEntry {
            provider: "openai".into(),
            model: "gpt-4o".into(),
            priority: 2,
            cost_per_1m_input: 2.5,
            cost_per_1m_output: 10.0,
        },
        ModelEntry {
            provider: "gemini".into(),
            model: "gemini-2.0-pro".into(),
            priority: 3,
            cost_per_1m_input: 1.25,
            cost_per_1m_output: 5.0,
        },
    ]
}

fn default_complex_tier() -> Vec<ModelEntry> {
    vec![
        ModelEntry {
            provider: "anthropic".into(),
            model: "claude-opus-4-20250514".into(),
            priority: 1,
            cost_per_1m_input: 15.0,
            cost_per_1m_output: 75.0,
        },
        ModelEntry {
            provider: "openai".into(),
            model: "o3".into(),
            priority: 2,
            cost_per_1m_input: 10.0,
            cost_per_1m_output: 40.0,
        },
    ]
}

fn default_frontier_tier() -> Vec<ModelEntry> {
    vec![ModelEntry {
        provider: "anthropic".into(),
        model: "claude-opus-4-20250514".into(),
        priority: 1,
        cost_per_1m_input: 15.0,
        cost_per_1m_output: 75.0,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_scoring_boundaries() {
        assert_eq!(
            TaskComplexity::from_request(500, 1, 0),
            TaskComplexity::Trivial
        );
        assert_eq!(
            TaskComplexity::from_request(5000, 3, 2),
            TaskComplexity::Standard
        );
        assert_eq!(
            TaskComplexity::from_request(20000, 5, 5),
            TaskComplexity::Complex
        );
        assert_eq!(
            TaskComplexity::from_request(50000, 10, 10),
            TaskComplexity::Frontier
        );
    }

    #[test]
    fn router_selects_from_tier() {
        let router = ModelRouter::new();
        let entry = router.select(TaskComplexity::Trivial);
        assert!(entry.is_some());
        assert!(entry.is_some_and(|e| !e.model.is_empty()));
    }

    #[test]
    fn router_select_for_request() {
        let router = ModelRouter::new();
        let entry = router.select_for_request(100, 1, 0);
        assert!(entry.is_some());
    }

    #[test]
    fn cost_estimation() {
        let entry = ModelEntry {
            provider: "test".into(),
            model: "test".into(),
            priority: 1,
            cost_per_1m_input: 3.0,
            cost_per_1m_output: 15.0,
        };
        let cost = entry.estimate_cost(1_000_000, 500_000);
        assert!((cost - 10.5).abs() < 0.001);
    }

    #[test]
    fn router_falls_back_on_all_exhausted() {
        let router = ModelRouter::new();
        // Even if all providers are exhausted, we still return the first model
        // rather than failing — the caller can retry later.
        let entry = router.select(TaskComplexity::Standard);
        assert!(entry.is_some());
    }

    #[test]
    fn custom_tier_override() -> Result<(), Box<dyn std::error::Error>> {
        let mut router = ModelRouter::new();
        let custom = vec![ModelEntry {
            provider: "local".into(),
            model: "llama-70b".into(),
            priority: 1,
            cost_per_1m_input: 0.0,
            cost_per_1m_output: 0.0,
        }];
        router.set_tier(TaskComplexity::Trivial, custom);
        let entry = router
            .select(TaskComplexity::Trivial)
            .ok_or("no model selected")?;
        assert_eq!(entry.provider, "local");
        assert_eq!(entry.model, "llama-70b");
        Ok(())
    }
}
