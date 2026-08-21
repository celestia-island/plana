//! Usage metering and budget enforcement for LLM calls.
//!
//! Tracks token usage and estimated costs per agent, per workspace, and
//! globally.  When a budget is set, calls that would exceed it are rejected
//! before the LLM request is dispatched.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use plana_llm_provider::metering::{MeteringEngine, Budget};
//!
//! let engine = MeteringEngine::global();
//!
//! // Set a budget for an agent
//! engine.set_budget("demiurge.001", Budget::daily_usd(5.0));
//!
//! // Record usage after an LLM call
//! engine.record_usage("demiurge.001", "anthropic", "claude-sonnet-4-20250514", 1200, 450);
//!
//! // Check budget before next call
//! if engine.is_over_budget("demiurge.001") {
//!     // Skip or use a cheaper model
//! }
//! ```

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

// ── Types ──────────────────────────────────────────────────────────

/// A single usage record for one LLM call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub agent_badge: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub timestamp: DateTime<Utc>,
}

/// Budget period for cost enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Total,
}

/// A budget for an agent or workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_cost_usd: f64,
    pub period: BudgetPeriod,
}

impl Budget {
    pub fn daily_usd(amount: f64) -> Self {
        Self {
            max_cost_usd: amount,
            period: BudgetPeriod::Daily,
        }
    }

    pub fn monthly_usd(amount: f64) -> Self {
        Self {
            max_cost_usd: amount,
            period: BudgetPeriod::Monthly,
        }
    }

    pub fn total_usd(amount: f64) -> Self {
        Self {
            max_cost_usd: amount,
            period: BudgetPeriod::Total,
        }
    }
}

/// Aggregate usage summary for a scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub request_count: u64,
}

// ── Engine ─────────────────────────────────────────────────────────

/// Global metering engine — singleton accessible from anywhere.
pub struct MeteringEngine {
    records: RwLock<Vec<UsageRecord>>,
    budgets: RwLock<HashMap<String, Budget>>,
}

impl MeteringEngine {
    /// Access the global singleton instance.
    pub fn global() -> &'static MeteringEngine {
        static INSTANCE: OnceLock<MeteringEngine> = OnceLock::new();
        INSTANCE.get_or_init(|| MeteringEngine {
            records: RwLock::new(Vec::new()),
            budgets: RwLock::new(HashMap::new()),
        })
    }

    /// Construct a fresh, isolated engine (for tests; avoids cross-test
    /// interference on the global singleton).
    #[cfg(test)]
    fn new() -> Self {
        MeteringEngine {
            records: RwLock::new(Vec::new()),
            budgets: RwLock::new(HashMap::new()),
        }
    }

    /// Set or update a budget for an agent badge or workspace ID.
    pub fn set_budget(&self, scope: &str, budget: Budget) {
        let mut budgets = self.budgets.write().unwrap_or_else(|e| e.into_inner());
        budgets.insert(scope.to_string(), budget);
    }

    /// Remove a budget.
    pub fn remove_budget(&self, scope: &str) {
        let mut budgets = self.budgets.write().unwrap_or_else(|e| e.into_inner());
        budgets.remove(scope);
    }

    /// Record a usage entry after an LLM call completes.
    pub fn record_usage(
        &self,
        agent_badge: &str,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) {
        let cost = estimate_cost(provider, model, input_tokens, output_tokens);
        let record = UsageRecord {
            agent_badge: agent_badge.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens,
            output_tokens,
            cost_usd: cost,
            timestamp: Utc::now(),
        };

        let mut records = self.records.write().unwrap_or_else(|e| e.into_inner());
        records.push(record);

        // Prevent unbounded growth — keep last 100,000 records
        if records.len() > 100_000 {
            let drain_count = records.len() - 80_000;
            records.drain(0..drain_count);
        }
    }

    /// Check if a scope has exceeded its budget.
    pub fn is_over_budget(&self, scope: &str) -> bool {
        // Drop the budgets guard before touching `records` to avoid a
        // lock-ordering inversion with `clear()` (records -> budgets).
        let budget = {
            let budgets = self.budgets.read().unwrap_or_else(|e| e.into_inner());
            budgets.get(scope).cloned()
        };
        let Some(budget) = budget else {
            return false;
        };

        let summary = self.summarize_in_period(scope, budget.period);
        summary.total_cost_usd >= budget.max_cost_usd
    }

    /// Get remaining budget for a scope.
    pub fn remaining_budget(&self, scope: &str) -> Option<f64> {
        // Drop the budgets guard before touching `records` (see is_over_budget).
        let budget = {
            let budgets = self.budgets.read().unwrap_or_else(|e| e.into_inner());
            budgets.get(scope).cloned()
        };
        let budget = budget?;
        let summary = self.summarize_in_period(scope, budget.period);
        Some((budget.max_cost_usd - summary.total_cost_usd).max(0.0))
    }

    /// Get a usage summary for a scope over all time.
    pub fn summarize(&self, scope: &str) -> UsageSummary {
        let records = self.records.read().unwrap_or_else(|e| e.into_inner());
        aggregate(&records, |r| r.agent_badge == scope)
    }

    /// Get a usage summary for a scope within a budget period.
    fn summarize_in_period(&self, scope: &str, period: BudgetPeriod) -> UsageSummary {
        let records = self.records.read().unwrap_or_else(|e| e.into_inner());
        let cutoff = period_cutoff(period);
        aggregate(&records, |r| {
            r.agent_badge == scope && r.timestamp >= cutoff
        })
    }

    /// Get a usage summary across all agents.
    pub fn summarize_all(&self) -> UsageSummary {
        let records = self.records.read().unwrap_or_else(|e| e.into_inner());
        aggregate(&records, |_| true)
    }

    /// Get per-provider breakdown.
    pub fn by_provider(&self) -> HashMap<String, UsageSummary> {
        let records = self.records.read().unwrap_or_else(|e| e.into_inner());
        let mut map: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
        for r in records.iter() {
            map.entry(r.provider.clone()).or_default().push(r);
        }
        map.into_iter()
            .map(|(k, v)| {
                let refs: Vec<&UsageRecord> = v;
                (k, aggregate_from_refs(&refs))
            })
            .collect()
    }

    /// Clear all records (for testing).
    #[cfg(test)]
    fn clear(&self) {
        let mut records = self.records.write().unwrap_or_else(|e| e.into_inner());
        records.clear();
        let mut budgets = self.budgets.write().unwrap_or_else(|e| e.into_inner());
        budgets.clear();
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn aggregate(records: &[UsageRecord], filter: impl Fn(&UsageRecord) -> bool) -> UsageSummary {
    let filtered: Vec<&UsageRecord> = records.iter().filter(|r| filter(r)).collect();
    aggregate_from_refs(&filtered)
}

fn aggregate_from_refs(records: &[&UsageRecord]) -> UsageSummary {
    let mut summary = UsageSummary::default();
    for r in records {
        summary.total_input_tokens += r.input_tokens;
        summary.total_output_tokens += r.output_tokens;
        summary.total_cost_usd += r.cost_usd;
        summary.request_count += 1;
    }
    summary
}

fn period_cutoff(period: BudgetPeriod) -> DateTime<Utc> {
    let now = Utc::now();
    match period {
        BudgetPeriod::Daily => {
            let date =
                NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap_or_default();
            DateTime::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap_or_default(), Utc)
        }
        BudgetPeriod::Weekly => now - Duration::days(7),
        BudgetPeriod::Monthly => {
            let date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or_default();
            DateTime::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap_or_default(), Utc)
        }
        BudgetPeriod::Total => DateTime::UNIX_EPOCH,
    }
}

/// Look up per-1M-token pricing `(input, output)` in USD for a model family.
///
/// This is the canonical pricing table shared across celestia-island services
/// (arona, entelecheia, evernight, ...).  Model matching is substring-based on
/// the lowercased model id; more specific families are matched before broader
/// ones.  Returns `None` when the model family is not in the table.
pub fn lookup_pricing(model: &str) -> Option<(f64, f64)> {
    let lower = model.to_lowercase();
    if lower.contains("claude") {
        if lower.contains("haiku") {
            return Some((0.80, 4.00));
        }
        if lower.contains("opus") {
            return Some((15.00, 75.00));
        }
        return Some((3.00, 15.00));
    }
    if lower.contains("gemini") {
        if lower.contains("flash") {
            return Some((0.075, 0.30));
        }
        return Some((1.25, 5.00));
    }
    if lower.contains("o1") || lower.contains("o3") {
        return Some((10.00, 40.00));
    }
    if lower.contains("gpt") && lower.contains("mini") {
        return Some((0.15, 0.60));
    }
    if lower.contains("gpt-4o") {
        return Some((2.50, 10.00));
    }
    if lower.contains("gpt-4") {
        return Some((30.00, 60.00));
    }
    if lower.contains("gpt-3.5") {
        return Some((0.50, 1.50));
    }
    if lower.contains("deepseek") {
        return Some((0.14, 0.28));
    }
    if lower.contains("qwen") {
        return Some((0.50, 2.00));
    }
    if lower.contains("llama-3") || lower.contains("llama3") {
        return Some((0.20, 0.80));
    }
    if lower.contains("mistral") {
        return Some((0.20, 0.80));
    }
    None
}

/// Rough cost estimation for metering when actual cost isn't provided.
/// Consults the canonical [`lookup_pricing`] table first, then falls back to
/// provider-keyed pricing for model families not in the table.
pub fn estimate_cost(provider: &str, model: &str, input: u64, output: u64) -> f64 {
    let (in_price, out_price) =
        lookup_pricing(model).unwrap_or_else(|| match (provider, model.contains("haiku")) {
            ("anthropic", true) => (0.8, 4.0),
            ("anthropic", _) if model.contains("opus") => (15.0, 75.0),
            ("anthropic", _) => (3.0, 15.0),
            ("openai", _) if model.contains("mini") => (0.15, 0.6),
            ("openai", _) if model.contains("o3") || model.contains("o1") => (10.0, 40.0),
            ("openai", _) => (2.5, 10.0),
            ("gemini", _) if model.contains("flash") => (0.075, 0.3),
            ("gemini", _) => (1.25, 5.0),
            _ => (3.0, 15.0),
        });
    (in_price * input as f64 / 1_000_000.0) + (out_price * output as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_summarize() {
        let engine = MeteringEngine::new();
        engine.clear();

        engine.record_usage(
            "test.001",
            "anthropic",
            "claude-sonnet-4-20250514",
            1000,
            500,
        );
        engine.record_usage(
            "test.001",
            "anthropic",
            "claude-sonnet-4-20250514",
            2000,
            1000,
        );

        let summary = engine.summarize("test.001");
        assert_eq!(summary.request_count, 2);
        assert_eq!(summary.total_input_tokens, 3000);
        assert_eq!(summary.total_output_tokens, 1500);
        assert!(summary.total_cost_usd > 0.0);
    }

    #[test]
    fn budget_enforcement() {
        let engine = MeteringEngine::new();
        engine.clear();

        engine.set_budget("budget-test", Budget::total_usd(0.01));

        assert!(!engine.is_over_budget("budget-test"));

        // Record enough usage to exceed $0.01
        engine.record_usage(
            "budget-test",
            "anthropic",
            "claude-opus-4-20250514",
            100_000,
            50_000,
        );

        assert!(engine.is_over_budget("budget-test"));
        assert_eq!(engine.remaining_budget("budget-test"), Some(0.0));
    }

    #[test]
    fn daily_budget_resets() {
        let engine = MeteringEngine::new();
        engine.clear();

        engine.set_budget("daily-test", Budget::daily_usd(100.0));

        // Record old usage (yesterday)
        let old_record = UsageRecord {
            agent_badge: "daily-test".into(),
            provider: "anthropic".into(),
            model: "claude-opus-4-20250514".into(),
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cost_usd: 50.0,
            timestamp: Utc::now() - Duration::days(2),
        };
        {
            let mut records = engine.records.write().unwrap_or_else(|e| e.into_inner());
            records.push(old_record);
        }

        // Old usage should not count toward today's budget
        assert!(!engine.is_over_budget("daily-test"));
    }

    #[test]
    fn no_budget_means_no_limit() {
        let engine = MeteringEngine::new();
        engine.clear();

        engine.record_usage("unlimited", "openai", "gpt-4o", 999_999_999, 999_999_999);
        assert!(!engine.is_over_budget("unlimited"));
    }

    #[test]
    fn by_provider_breakdown() -> Result<(), Box<dyn std::error::Error>> {
        let engine = MeteringEngine::new();
        engine.clear();

        engine.record_usage("a", "anthropic", "claude", 100, 50);
        engine.record_usage("b", "openai", "gpt-4o", 200, 100);
        engine.record_usage("c", "anthropic", "claude", 300, 150);

        let by_prov = engine.by_provider();
        let anthropic = by_prov
            .get("anthropic")
            .ok_or("anthropic provider missing")?;
        let openai = by_prov.get("openai").ok_or("openai provider missing")?;
        assert_eq!(anthropic.request_count, 2);
        assert_eq!(openai.request_count, 1);
        Ok(())
    }

    #[test]
    fn cost_estimation_scales() {
        let cost1 = estimate_cost("anthropic", "claude-sonnet-4-20250514", 1000, 500);
        let cost2 = estimate_cost("anthropic", "claude-sonnet-4-20250514", 2000, 1000);
        assert!((cost2 / cost1 - 2.0).abs() < 0.01, "should scale linearly");
    }

    #[test]
    fn canonical_pricing_lookup() {
        assert_eq!(lookup_pricing("gpt-4o"), Some((2.50, 10.00)));
        assert_eq!(lookup_pricing("gpt-4o-mini"), Some((0.15, 0.60)));
        assert_eq!(lookup_pricing("gpt-4-turbo"), Some((30.00, 60.00)));
        assert_eq!(lookup_pricing("gpt-3.5-turbo"), Some((0.50, 1.50)));
        assert_eq!(lookup_pricing("o3-mini"), Some((10.00, 40.00)));
        assert_eq!(
            lookup_pricing("claude-opus-4-20250514"),
            Some((15.00, 75.00))
        );
        assert_eq!(
            lookup_pricing("claude-sonnet-4-20250514"),
            Some((3.00, 15.00))
        );
        assert_eq!(lookup_pricing("claude-3-5-haiku"), Some((0.80, 4.00)));
        assert_eq!(lookup_pricing("gemini-2.5-flash"), Some((0.075, 0.30)));
        assert_eq!(lookup_pricing("gemini-2.5-pro"), Some((1.25, 5.00)));
        assert_eq!(lookup_pricing("deepseek-v4-flash"), Some((0.14, 0.28)));
        assert_eq!(
            lookup_pricing("deepseek-ai/deepseek-v4-pro"),
            Some((0.14, 0.28))
        );
        assert_eq!(lookup_pricing("Qwen/Qwen3-1.7B"), Some((0.50, 2.00)));
        assert_eq!(lookup_pricing("llama-3-8b-instruct"), Some((0.20, 0.80)));
        assert_eq!(lookup_pricing("mistral-7b-instruct"), Some((0.20, 0.80)));
        assert_eq!(lookup_pricing("google/gemma-3-1b-it"), None);
        assert_eq!(lookup_pricing("HuggingFaceTB/SmolLM2-1.7B-Instruct"), None);
    }
}
