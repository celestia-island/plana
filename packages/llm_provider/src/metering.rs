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
    /// Which model tier the request was dispatched under (`deep` / `normal` /
    /// `basic`). `None` on records written before the tier dimension existed
    /// or by callers that have no tier context. Wire form is the lowercase
    /// tier string (matches `ModelTier::as_tier_str`).
    #[serde(default)]
    pub tier: Option<String>,
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
    /// Audit R1-LP-4: per-scope lifetime spend that NEVER gets evicted —
    /// the records Vec drains at 100k, which silently shrank Total-period
    /// budget checks. This map is the authoritative Total-spend source.
    lifetime_cost: RwLock<HashMap<String, f64>>,
}

impl MeteringEngine {
    /// Access the global singleton instance.
    pub fn global() -> &'static MeteringEngine {
        static INSTANCE: OnceLock<MeteringEngine> = OnceLock::new();
        INSTANCE.get_or_init(|| MeteringEngine {
            records: RwLock::new(Vec::new()),
            budgets: RwLock::new(HashMap::new()),
            lifetime_cost: RwLock::new(HashMap::new()),
        })
    }

    /// Construct a fresh, isolated engine (for tests; avoids cross-test
    /// interference on the global singleton).
    #[cfg(test)]
    fn new() -> Self {
        MeteringEngine {
            records: RwLock::new(Vec::new()),
            budgets: RwLock::new(HashMap::new()),
            lifetime_cost: RwLock::new(HashMap::new()),
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
        self.record_usage_with_tier(
            agent_badge,
            provider,
            model,
            input_tokens,
            output_tokens,
            None,
        );
    }

    /// [`Self::record_usage`] with the dispatch tier attached, so the cost
    /// ledger can answer "what did each tier actually cost" — the first
    /// column of per-task cost accounting. `tier` takes the lowercase tier
    /// string (`deep` / `normal` / `basic`).
    pub fn record_usage_with_tier(
        &self,
        agent_badge: &str,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        tier: Option<&str>,
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
            tier: tier.map(str::to_string),
        };

        // Audit R1-LP-4: accumulate lifetime cost per scope — this map
        // is never evicted, so Total-period budget checks remain accurate
        // even after the records Vec drains.
        {
            let mut lifetime = self
                .lifetime_cost
                .write()
                .unwrap_or_else(|e| e.into_inner());
            *lifetime.entry(agent_badge.to_string()).or_default() += cost;
        }

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

        // Audit R1-LP-4: for Total period, use the never-evicted
        // lifetime cost instead of re-summing the (potentially drained)
        // records Vec — the old path silently shrank as records aged out.
        if budget.period == BudgetPeriod::Total {
            let lifetime = self.lifetime_cost.read().unwrap_or_else(|e| e.into_inner());
            let total = lifetime.get(scope).copied().unwrap_or(0.0);
            return total >= budget.max_cost_usd;
        }
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
        // Audit R1-LP-4: Total period reads the never-evicted lifetime
        // spend (same source as is_over_budget) so the two entry points
        // never disagree on whether the budget is exhausted.
        let spend = if budget.period == BudgetPeriod::Total {
            let lifetime = self.lifetime_cost.read().unwrap_or_else(|e| e.into_inner());
            lifetime.get(scope).copied().unwrap_or(0.0)
        } else {
            self.summarize_in_period(scope, budget.period)
                .total_cost_usd
        };
        Some((budget.max_cost_usd - spend).max(0.0))
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

    /// Get per-tier breakdown. Records without a tier (written before the
    /// tier dimension existed, or by tier-less callers) land under the
    /// `"(untiered)"` key so the breakdown never silently drops usage.
    pub fn by_tier(&self) -> HashMap<String, UsageSummary> {
        let records = self.records.read().unwrap_or_else(|e| e.into_inner());
        let mut map: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
        for r in records.iter() {
            let key = r.tier.clone().unwrap_or_else(|| "(untiered)".to_string());
            map.entry(key).or_default().push(r);
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

/// One model family's canonical pricing in USD per 1M tokens. The
/// cached tier is the price providers charge for prompt-cache HITS;
/// families without a published cached price keep it equal to the plain
/// input price (conservative: cache savings then never overstate cost).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FamilyPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cached_per_million: f64,
}

/// The T3 v2 pricing schedule: the retail [`FamilyPricing`] (unchanged
/// semantics — peak-tier list prices) plus the commercial extensions:
/// an optional PURCHASE line (what WE pay the vendor per 1M tokens —
/// the margin input) and optional off-peak windows (闲时). Everything
/// is additive; existing `FamilyPricing` consumers are untouched.
#[derive(Debug, Clone, PartialEq)]
pub struct PricingSchedule {
    pub retail: FamilyPricing,
    /// Supply-cost line; `None` = same as retail (margin unknown).
    pub purchase: Option<FamilyPricing>,
    /// Off-peak windows, evaluated in UTC hour-of-day. Empty = peak
    /// around the clock (the pre-T3 behaviour).
    pub off_peak: Vec<OffPeakWindow>,
}

/// One off-peak window: UTC hour-of-day range (start inclusive, end
/// EXCLUSIVE, wrap-aware across midnight) and the multiplier applied
/// to the retail prices inside it (e.g. `0.5` = half price).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffPeakWindow {
    pub start_hour: u8,
    /// Exclusive end hour; `end <= start` wraps past midnight.
    pub end_hour: u8,
    pub multiplier: f64,
}

impl OffPeakWindow {
    /// Whether `hour` (UTC 0–23) falls inside the window. Pure;
    /// unit-tested.
    pub fn contains(&self, hour: u8) -> bool {
        let h = hour % 24;
        if self.end_hour <= self.start_hour {
            // wrap: [start, 24) ∪ [0, end)
            h >= self.start_hour || h < self.end_hour
        } else {
            h >= self.start_hour && h < self.end_hour
        }
    }
}

impl PricingSchedule {
    /// The multiplier in effect at `hour` (UTC): the first containing
    /// off-peak window wins, else 1.0 (peak). Pure; unit-tested.
    pub fn effective_multiplier_utc(&self, hour: u8) -> f64 {
        self.off_peak
            .iter()
            .find(|w| w.contains(hour))
            .map(|w| w.multiplier)
            .unwrap_or(1.0)
    }

    /// Retail prices scaled by the window in effect at `hour` (UTC).
    pub fn effective_pricing_utc(&self, hour: u8) -> FamilyPricing {
        let m = self.effective_multiplier_utc(hour);
        FamilyPricing {
            input_per_million: self.retail.input_per_million * m,
            output_per_million: self.retail.output_per_million * m,
            cached_per_million: self.retail.cached_per_million * m,
        }
    }

    /// The margin (retail − purchase) for a token count at `hour`
    /// (UTC), in USD. `None` when no purchase line is published.
    pub fn margin_usd(&self, input: u32, output: u32, hour: u8) -> Option<f64> {
        let purchase = self.purchase?;
        let m = self.effective_multiplier_utc(hour);
        let retail_usd = (input as f64 / 1_000_000.0) * self.retail.input_per_million * m
            + (output as f64 / 1_000_000.0) * self.retail.output_per_million * m;
        Some(
            retail_usd
                - (input as f64 / 1_000_000.0) * purchase.input_per_million
                - (output as f64 / 1_000_000.0) * purchase.output_per_million,
        )
    }
}

// The generated fallback pricing table. The `PricingBand` struct and the
// `PRICING_BANDS` static are emitted into OUT_DIR by build.rs from the
// CHECKED-IN `pricing/bands.toml` (explicit ordered patterns) joined with
// `pricing/models-snapshot.toml` (the last generated copy of
// provider-registry prices — refresh explicitly with `just regen-pricing`;
// the build itself never fetches). The matcher lives below as ordinary
// hand-written code so it stays unit-testable.
include!(concat!(env!("OUT_DIR"), "/pricing_table.rs"));

impl PricingBand {
    /// Whether this band's declared patterns match a LOWERCASED model id:
    /// every `all` substring present AND every `any` group contributing at
    /// least one present member. This is the declared-pattern twin of the
    /// nested `contains` ladder the table replaced (same matching
    /// semantics, arona's model_pricing override style: every pattern
    /// explicit, rows in priority order, first match wins).
    pub(crate) fn matches(&self, lower: &str) -> bool {
        self.all.iter().all(|p| lower.contains(p))
            && self.any.iter().all(|g| g.iter().any(|p| lower.contains(p)))
    }
}

/// Resolve a model id to its band pricing by the generated pattern table.
/// Band numbers come from provider-registry data (USD per 1M tokens as the
/// registry publishes them — CNY-sourced lists arrive pre-converted there,
/// e.g. GLM's ¥ list at the 6.5 reference rate, rounded to 2dp). All prices
/// are PEAK-tier — the conservative upper bound; the actual off-peak
/// billing split lives with the provider's ledger, not here. Returns
/// `None` for unknown families; callers fall back to provider-keyed
/// guesses (see `estimate_cost`).
///
/// This is the universal FALLBACK: arona's model_pricing DB rows win first,
/// and an injected source (set_pricing_source) wins over both.
pub fn lookup_pricing(model: &str) -> Option<FamilyPricing> {
    let lower = model.to_lowercase();
    PRICING_BANDS
        .iter()
        .find(|b| b.matches(&lower))
        .map(|b| b.pricing)
}

/// The id of the band that would price `model`, for diagnostics and tests
/// (e.g. "gpt-5.4-mini" → "gpt-mini-5"; `None` for unpriced models).
/// Useful wherever a consumer wants to say WHICH fallback band answered —
/// logging, the arona panel's "upstream family default" badge, and the
/// generated-table tests.
pub fn pricing_band_id(model: &str) -> Option<&'static str> {
    let lower = model.to_lowercase();
    PRICING_BANDS
        .iter()
        .find(|b| b.matches(&lower))
        .map(|b| b.id)
}

/// Rough cost estimation for metering when actual cost isn't provided.
/// Consults the canonical [`lookup_pricing`] table first, then falls back to
/// provider-keyed pricing for model families not in the table.
pub fn estimate_cost(provider: &str, model: &str, input: u64, output: u64) -> f64 {
    let p = resolve_pricing(model).unwrap_or_else(|| match (provider, model.contains("haiku")) {
        ("anthropic", true) => FamilyPricing {
            input_per_million: 0.8,
            output_per_million: 4.0,
            cached_per_million: 0.08,
        },
        ("anthropic", _) if model.contains("opus") => FamilyPricing {
            input_per_million: 15.0,
            output_per_million: 75.0,
            cached_per_million: 1.5,
        },
        ("anthropic", _) => FamilyPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cached_per_million: 0.3,
        },
        ("openai", _) if model.contains("mini") => FamilyPricing {
            input_per_million: 0.15,
            output_per_million: 0.6,
            cached_per_million: 0.0375,
        },
        ("openai", _) if model.contains("o3") || model.contains("o1") => FamilyPricing {
            input_per_million: 10.0,
            output_per_million: 40.0,
            cached_per_million: 2.5,
        },
        ("openai", _) => FamilyPricing {
            input_per_million: 2.5,
            output_per_million: 10.0,
            cached_per_million: 0.625,
        },
        ("gemini", _) if model.contains("flash") => FamilyPricing {
            input_per_million: 0.075,
            output_per_million: 0.3,
            cached_per_million: 0.0188,
        },
        ("gemini", _) => FamilyPricing {
            input_per_million: 1.25,
            output_per_million: 5.0,
            cached_per_million: 0.3125,
        },
        _ => FamilyPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cached_per_million: 0.3,
        },
    });
    (p.input_per_million * input as f64 / 1_000_000.0)
        + (p.output_per_million * output as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serializes every test that reads or writes the PROCESS-WIDE
    /// metering state (the global records Vec and the injected pricing
    /// slot) — `by_tier_breakdown` historically raced with the injection
    /// tests' unconditional zero-price source and flaked its cost
    /// assertions (audit R1 follow-up).
    pub(crate) static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
            tier: None,
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

    /// Audit R1-LP-4: a Total-period budget must survive records-Vec
    /// eviction. The old path re-summed the Vec — once it drained past
    /// the 100k cap, the "total spend" silently shrank and a scope that
    /// had blown through its budget appeared to have headroom again.
    #[test]
    fn total_budget_survives_records_eviction() {
        let engine = MeteringEngine::new();
        engine.set_budget("evict-test", Budget::total_usd(10.0));

        // Spend $15 — the budget is blown.
        engine.record_usage("evict-test", "openai", "gpt-4o", 10_000_000, 5_000_000);
        assert!(
            engine.is_over_budget("evict-test"),
            "over-budget before eviction"
        );

        // Simulate the records-Vec drain: the entries backing that $15
        // disappear, but the lifetime cost must NOT shrink.
        {
            let mut records = engine.records.write().unwrap_or_else(|e| e.into_inner());
            records.clear();
        }
        assert!(
            engine.is_over_budget("evict-test"),
            "Total-period over-budget must survive records eviction"
        );

        // A different scope must NOT be affected by evict-test's spend.
        engine.set_budget("other-scope", Budget::total_usd(10.0));
        assert!(
            !engine.is_over_budget("other-scope"),
            "a different scope must start with a clean lifetime cost"
        );
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
    fn by_tier_breakdown_buckets_tiered_and_untiered() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let engine = MeteringEngine::new();
        engine.clear();

        engine.record_usage_with_tier("a", "openai", "gpt-4o", 100, 50, Some("deep"));
        engine.record_usage_with_tier("b", "openai", "gpt-4o", 200, 100, Some("deep"));
        engine.record_usage_with_tier("c", "openai", "gpt-4o-mini", 300, 150, Some("basic"));
        // Legacy path: no tier context.
        engine.record_usage("d", "openai", "gpt-4o-mini", 10, 5);

        let by_tier = engine.by_tier();
        let deep = by_tier.get("deep").ok_or("deep tier missing")?;
        let basic = by_tier.get("basic").ok_or("basic tier missing")?;
        let untiered = by_tier
            .get("(untiered)")
            .ok_or("untiered bucket missing — legacy records must not vanish")?;
        assert_eq!(deep.request_count, 2);
        assert_eq!(basic.request_count, 1);
        assert_eq!(untiered.request_count, 1);
        // Pin the token/cost aggregation of `aggregate_from_refs` (shared by
        // by_provider / by_tier) — before these assertions a mutation that
        // dropped tokens from the refs-based aggregator survived every test.
        assert_eq!(deep.total_input_tokens, 100 + 200);
        assert_eq!(deep.total_output_tokens, 50 + 100);
        assert_eq!(basic.total_input_tokens, 300);
        assert_eq!(basic.total_output_tokens, 150);
        assert_eq!(untiered.total_input_tokens, 10);
        assert_eq!(untiered.total_output_tokens, 5);
        let deep_cost = estimate_cost("openai", "gpt-4o", 300, 150);
        assert!(
            (deep.total_cost_usd - deep_cost).abs() < 1e-9,
            "deep bucket cost should sum per-record estimates: got {:#?}, want {:#?}",
            deep.total_cost_usd,
            deep_cost
        );
        Ok(())
    }

    #[test]
    fn usage_record_deserializes_without_tier_field() -> Result<(), Box<dyn std::error::Error>> {
        // Records serialized before the tier dimension existed have no
        // `tier` key; they must deserialize with tier = None, not fail.
        let legacy = serde_json::json!({
            "agent_badge": "a",
            "provider": "openai",
            "model": "gpt-4o",
            "input_tokens": 1,
            "output_tokens": 2,
            "cost_usd": 0.001,
            "timestamp": "2026-09-29T00:00:00Z",
        });
        let record: UsageRecord = serde_json::from_value(legacy)?;
        assert_eq!(record.tier, None);
        Ok(())
    }

    #[test]
    fn usage_record_round_trips_tier() -> Result<(), Box<dyn std::error::Error>> {
        let engine = MeteringEngine::new();
        engine.clear();
        engine.record_usage_with_tier("a", "openai", "gpt-4o", 1, 1, Some("normal"));
        let serialized = serde_json::to_string(
            &engine.summarize_all().request_count, // touch aggregation to prove records exist
        )?;
        assert_eq!(serialized, "1");
        // Direct record-level round trip:
        let rec = UsageRecord {
            agent_badge: "a".into(),
            provider: "openai".into(),
            model: "gpt-4o".into(),
            input_tokens: 1,
            output_tokens: 1,
            cost_usd: 0.0,
            timestamp: Utc::now(),
            tier: Some("normal".into()),
        };
        let json = serde_json::to_string(&rec)?;
        let back: UsageRecord = serde_json::from_str(&json)?;
        assert_eq!(back.tier.as_deref(), Some("normal"));
        Ok(())
    }

    #[test]
    fn cost_estimation_scales() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let cost1 = estimate_cost("anthropic", "claude-sonnet-4-20250514", 1000, 500);
        let cost2 = estimate_cost("anthropic", "claude-sonnet-4-20250514", 2000, 1000);
        assert!((cost2 / cost1 - 2.0).abs() < 0.01, "should scale linearly");
    }

    fn f(in_p: f64, out_p: f64, cached_p: f64) -> Option<FamilyPricing> {
        Some(FamilyPricing {
            input_per_million: in_p,
            output_per_million: out_p,
            cached_per_million: cached_p,
        })
    }

    #[test]
    fn canonical_pricing_lookup() {
        assert_eq!(lookup_pricing("gpt-4o"), f(2.50, 10.00, 1.25));
        assert_eq!(lookup_pricing("gpt-4o-mini"), f(0.15, 0.60, 0.075));
        // The mini tier is its own band — never the generation headline
        // (round-1: the broad gpt-5 arm used to swallow this id).
        assert_eq!(lookup_pricing("gpt-5.4-mini"), f(0.75, 4.50, 0.075));
        assert_eq!(lookup_pricing("gpt-4-turbo"), f(30.00, 60.00, 30.00));
        assert_eq!(lookup_pricing("gpt-3.5-turbo"), f(0.50, 1.50, 0.50));
        assert_eq!(lookup_pricing("o3-mini"), f(10.00, 40.00, 2.50));
        assert_eq!(
            lookup_pricing("claude-opus-4-20250514"),
            f(15.00, 75.00, 1.50)
        );
        assert_eq!(
            lookup_pricing("claude-sonnet-4-20250514"),
            f(3.00, 15.00, 0.30)
        );
        assert_eq!(lookup_pricing("claude-3-5-haiku"), f(0.80, 4.00, 0.08));
        assert_eq!(lookup_pricing("gemini-2.5-flash"), f(0.075, 0.30, 0.0188));
        assert_eq!(lookup_pricing("gemini-2.5-pro"), f(1.25, 5.00, 0.3125));
        assert_eq!(lookup_pricing("Qwen/Qwen3-1.7B"), f(0.50, 2.00, 0.125));
        assert_eq!(lookup_pricing("llama-3-8b-instruct"), f(0.20, 0.80, 0.20));
        // Round-2 sub-band pins (variant/alias granularity).
        assert_eq!(lookup_pricing("deepseek-v4.1-flash"), f(0.30, 1.20, 0.006));
        assert_eq!(lookup_pricing("gpt-5.4-nano"), f(0.20, 1.25, 0.02));
        assert_eq!(lookup_pricing("gpt-5-nano"), f(0.05, 0.40, 0.005));
        assert_eq!(lookup_pricing("gpt-5.5-pro"), f(30.00, 180.00, 3.00));
        assert_eq!(
            lookup_pricing("gpt-daybreak-red-latest"),
            f(12.50, 75.00, 1.25)
        );
        assert_eq!(lookup_pricing("claude-fable-5-1"), f(10.00, 50.00, 0.25));
        assert_eq!(
            lookup_pricing("claude-opus-4.6-fast"),
            f(30.00, 150.00, 3.00)
        );
        assert_eq!(lookup_pricing("claude-opus-4.8"), f(5.00, 25.00, 0.50));
        assert_eq!(lookup_pricing("gemini-flash-latest"), f(1.50, 9.00, 0.15));
        assert_eq!(lookup_pricing("glm-5.3-flashx"), f(0.31, 1.08, 0.088));
        assert_eq!(lookup_pricing("glm-5.3-prime"), f(2.80, 8.80, 0.56));
        assert_eq!(lookup_pricing("mistral-7b-instruct"), f(0.20, 0.80, 0.20));
        assert_eq!(lookup_pricing("google/gemma-3-1b-it"), None);
        assert_eq!(lookup_pricing("HuggingFaceTB/SmolLM2-1.7B-Instruct"), None);
    }

    #[test]
    fn fleet_families_are_priced() {
        // The four families the fleet actually runs (Phase 0 of the
        // commercialization plan): every one must resolve WITHOUT
        // falling through to the provider-keyed guess table.
        // The bands mirror the registry's current generation (2026-10-03).
        let ds = lookup_pricing("deepseek-v4-flash").expect("ds flash");
        assert!((ds.input_per_million - 0.14).abs() < 1e-9);
        let dsp = lookup_pricing("deepseek-v4-pro").expect("ds pro");
        assert!((dsp.input_per_million - 0.435).abs() < 1e-9);
        // GLM 5.3 — the registry publishes the ¥8/¥28/¥2 list converted at
        // the 6.5 reference rate, rounded to 2dp (1.23/4.31/0.31); the
        // pre-refactor hand table computed the division unrounded. The
        // asserted relation is the conversion itself, within the rounding
        // tolerance (the full-corpus pin is the golden fixture test).
        let glm = lookup_pricing("glm-5.3").expect("glm");
        assert!((glm.input_per_million - 8.0 / 6.5).abs() <= 0.005);
        assert!((glm.cached_per_million - 2.0 / 6.5).abs() <= 0.005);
        let glmf = lookup_pricing("glm-5.3-flash").expect("glm flash");
        assert!((glmf.input_per_million - 0.12).abs() < 1e-9);
        // Kimi K3 official USD list.
        let kimi = lookup_pricing("kimi-k3").expect("kimi");
        assert!((kimi.input_per_million - 3.0).abs() < 1e-9);
        assert!((kimi.output_per_million - 15.0).abs() < 1e-9);
        // The gpt-6 generation resolves to its own bands.
        let astra = lookup_pricing("gpt-6-astra").expect("astra");
        assert!((astra.input_per_million - 10.0).abs() < 1e-9);
        let luna = lookup_pricing("gpt-6-luna").expect("luna");
        assert!((luna.output_per_million - 0.5).abs() < 1e-9);
        let sol = lookup_pricing("gpt-6.1-sol").expect("sol");
        assert!((sol.input_per_million - 2.0).abs() < 1e-9);
        // And the 5.x heads stop falling through to the generic gpt arms.
        let g56 = lookup_pricing("gpt-5.6").expect("gpt-5.6");
        assert!((g56.input_per_million - 4.0).abs() < 1e-9);
        let opus = lookup_pricing("claude-opus-5-5").expect("opus");
        assert!((opus.input_per_million - 4.0).abs() < 1e-9);
    }

    #[test]
    fn cached_never_exceeds_plain_input() {
        // The conservative invariant: for every priced family the cached
        // tier is at most the plain input price (never a premium).
        for model in [
            "claude-sonnet-4",
            "claude-3-5-haiku",
            "claude-opus-4",
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "o3-mini",
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            "deepseek-v4.1-flash",
            "deepseek-v4-pro",
            "glm-5.3",
            "glm-5.3-flash",
            "kimi-k3",
            "Qwen/Qwen3-1.7B",
            "llama-3-8b",
            "mistral-7b",
        ] {
            // .expect, not if-let: a family dropping out of the table
            // must FAIL here, not silently skip (R1's M3 evidence).
            let p = lookup_pricing(model)
                .unwrap_or_else(|| panic!("{model} dropped from the pricing table"));
            assert!(
                p.cached_per_million <= p.input_per_million + 1e-9,
                "{model}: cached {} > input {}",
                p.cached_per_million,
                p.input_per_million
            );
        }
    }

    // ── Generated-table equivalence and mismatch pins ──────────────────

    #[derive(serde::Deserialize)]
    struct GoldenFile {
        outputs: std::collections::BTreeMap<String, GoldenValue>,
    }

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum GoldenValue {
        /// The literal string "none" — the old table priced nothing.
        Unpriced(String),
        /// [input, output, cached] per million.
        Triple(Vec<f64>),
    }

    fn read_golden() -> GoldenFile {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/pricing/golden-legacy-table.toml"
        ))
        .expect("golden fixture is checked in");
        toml::from_str(&src).expect("golden fixture parses")
    }

    #[derive(serde::Deserialize)]
    struct BandsFile {
        band: Vec<BandRow>,
    }

    #[derive(serde::Deserialize)]
    struct BandRow {
        id: String,
        anchor: Option<String>,
        #[serde(rename = "fx_assert")]
        fx_assert: Option<FxAssert>,
    }

    #[derive(serde::Deserialize)]
    struct FxAssert {
        cny_input: f64,
        cny_output: f64,
        cny_cached: f64,
        rate: f64,
        tolerance: f64,
    }

    #[derive(serde::Deserialize)]
    struct SnapshotFile {
        model: Vec<SnapshotRow>,
    }

    #[derive(serde::Deserialize)]
    struct SnapshotRow {
        provider: String,
        id: String,
        input_per_million: f64,
        output_per_million: f64,
        cached_per_million: Option<f64>,
    }

    /// The generated table must reproduce the PRE-REFACTOR hand table
    /// bit-for-bit over the full corpus (every registry snapshot id plus
    /// curated edge ids), with exactly one tolerated class: bands whose
    /// registry source publishes a rounded FX conversion (declared in
    /// bands.toml `fx_assert`, asserted here against the relation, not
    /// against hard-coded numbers).
    #[test]
    fn golden_legacy_equivalence() {
        let golden = read_golden();
        assert!(
            golden.outputs.len() > 1000,
            "golden corpus shrank: {} entries",
            golden.outputs.len()
        );

        // Resolve the single fx-tolerated band from the declared sources.
        let bands: BandsFile = toml::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/pricing/bands.toml"))
                .expect("bands.toml readable"),
        )
        .expect("bands.toml parses");
        let fx_rows: Vec<&BandRow> = bands
            .band
            .iter()
            .filter(|b| b.fx_assert.is_some())
            .collect();
        assert_eq!(
            fx_rows.len(),
            1,
            "exactly one fx_assert band is supported by this test's tolerance rule"
        );
        let fx_row = fx_rows[0];
        let fx = fx_row.fx_assert.as_ref().unwrap();
        let old_fx = [
            fx.cny_input / fx.rate,
            fx.cny_output / fx.rate,
            fx.cny_cached / fx.rate,
        ];
        let snap: SnapshotFile = toml::from_str(
            &std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/pricing/models-snapshot.toml"
            ))
            .expect("snapshot readable"),
        )
        .expect("snapshot parses");
        let anchor = fx_row.anchor.as_deref().expect("fx band is anchored");
        let anchor_row = snap
            .model
            .iter()
            .find(|m| format!("{}/{}", m.provider, m.id) == anchor)
            .unwrap_or_else(|| panic!("fx anchor {anchor} missing from snapshot"));
        let new_fx = [
            anchor_row.input_per_million,
            anchor_row.output_per_million,
            anchor_row.cached_per_million.unwrap_or(f64::NAN),
        ];
        // The asserted FX consistency: the registry's USD price is the
        // CNY list at the reference rate, within the declared tolerance.
        for (field, (old, new)) in old_fx.iter().zip(new_fx.iter()).enumerate() {
            assert!(
                (old - new).abs() <= fx.tolerance,
                "{}: registry price {new} is not the {} conversion of the CNY list ({old}, tolerance {})",
                fx_row.id,
                ["input", "output", "cached"][field],
                fx.tolerance
            );
        }

        let as_pricing = |t: &[f64]| {
            Some(FamilyPricing {
                input_per_million: t[0],
                output_per_million: t[1],
                cached_per_million: t[2],
            })
        };
        let is_old_fx = |t: &[f64]| -> bool {
            t.len() == 3
                && t[0].to_bits() == old_fx[0].to_bits()
                && t[1].to_bits() == old_fx[1].to_bits()
                && t[2].to_bits() == old_fx[2].to_bits()
        };

        let mut mismatches = Vec::new();
        for (id, want) in &golden.outputs {
            match want {
                GoldenValue::Unpriced(s) => {
                    assert_eq!(s, "none", "fixture sentinel for {id:?}");
                    if lookup_pricing(id).is_some() {
                        mismatches.push(format!("{id}: was unpriced, now priced"));
                    }
                }
                GoldenValue::Triple(t) => {
                    let got = lookup_pricing(id);
                    if is_old_fx(t) {
                        // The one tolerated class: rounded FX from the
                        // registry. The band must still resolve to the
                        // anchor's registry price, within tolerance of the
                        // old unrounded values.
                        match got {
                            Some(p) => {
                                let got_t = [
                                    p.input_per_million,
                                    p.output_per_million,
                                    p.cached_per_million,
                                ];
                                for (old, new) in t.iter().zip(got_t.iter()) {
                                    if (old - new).abs() > fx.tolerance {
                                        mismatches.push(format!(
                                            "{id}: fx-tolerated band drifted beyond tolerance: {old} vs {new}"
                                        ));
                                    }
                                }
                                for (new, expected) in got_t.iter().zip(new_fx.iter()) {
                                    if new.to_bits() != expected.to_bits() {
                                        mismatches.push(format!(
                                            "{id}: fx band must use the registry anchor price: {new} vs {expected}"
                                        ));
                                    }
                                }
                            }
                            None => mismatches.push(format!("{id}: fx band stopped resolving")),
                        }
                    } else if got != as_pricing(t) {
                        mismatches.push(format!("{id}: got {got:?}, golden says {t:?}"));
                    }
                }
            }
        }
        assert!(
            mismatches.is_empty(),
            "{} golden mismatches:\n  {}",
            mismatches.len(),
            mismatches.join("\n  ")
        );
    }

    /// Negative pins for the generated pattern table: the mismatch risks
    /// of substring banding, pinned so any drift (in patterns, ordering,
    /// or the generator) is a conscious decision rather than silent rot.
    #[test]
    fn pattern_semantics_negative_pins() {
        // "o1"/"o3" are BARE substrings (pre-refactor breadth, kept for
        // equivalence): any id containing them lands in the legacy o-band.
        let o_band = lookup_pricing("o1").expect("o1 priced");
        assert_eq!(lookup_pricing("vogel-o3-mini"), Some(o_band));
        // The mini tier is its own band — never the generation headline.
        assert_ne!(lookup_pricing("gpt-5.4-mini"), lookup_pricing("gpt-5.4"));
        // A glm flash-tier id joins the flash band, not the pro band.
        assert_eq!(
            lookup_pricing("glm-4.5-flash"),
            lookup_pricing("glm-5.3-flash")
        );
        // Family prefixes must not leak across vendors.
        assert_ne!(lookup_pricing("qwen-max"), lookup_pricing("glm-5.3"));
        // Truly unknown ids stay unpriced — including the empty id.
        assert_eq!(lookup_pricing("brand-new-vendor-x"), None);
        assert_eq!(lookup_pricing(""), None);
        assert_eq!(lookup_pricing("google/gemma-3-1b-it"), None);
        // Matching is case-insensitive on the model id side.
        assert_eq!(
            lookup_pricing("GLM-5.3-FLASHX"),
            lookup_pricing("glm-5.3-flashx")
        );
        // Every generated band keeps the conservative cached invariant.
        for band in PRICING_BANDS {
            assert!(
                band.pricing.cached_per_million <= band.pricing.input_per_million + 1e-9,
                "{}: cached {} > input {}",
                band.id,
                band.pricing.cached_per_million,
                band.pricing.input_per_million
            );
        }
    }
}

// ── External pricing source injection (C2c) ──────────────────────────────
//
// The upstream stays the single ALGORITHM owner: family matching, the FX
// convention and the conservative cached-price rule live here and nowhere
// else. The DATA may come from an injected source (arona's model_pricing
// table) BEFORE the compiled-in families — the injectable source answers
// first, the hardcoded table remains the fallback, and `estimate_cost`
// inherits the chain unchanged (one call site, no consumer edits).
//
// The hook is process-wide and set-once: a service wires it at boot (an
// Arc'd closure over its DB pool); tests set and clear it. A source that
// panics poisons the process — sources must answer `None` on their own
// errors (falling back to the families), never panic.

use std::sync::Arc;

type PricingSource = Arc<dyn Fn(&str) -> Option<FamilyPricing> + Send + Sync>;

static INJECTED_SOURCE: std::sync::RwLock<Option<PricingSource>> = std::sync::RwLock::new(None);

/// Install the external pricing source. Returns whether the install took
/// effect (a second install is REJECTED — the first source wins, so a
/// misbehaving late initializer cannot silently replace the fleet's
/// pricing at runtime).
pub fn set_pricing_source(source: PricingSource) -> bool {
    let mut slot = INJECTED_SOURCE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if slot.is_some() {
        return false;
    }
    *slot = Some(source);
    true
}

/// Test-only: clear the injected source between tests.
#[cfg(test)]
pub(crate) fn clear_pricing_source_for_tests() {
    let mut slot = INJECTED_SOURCE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *slot = None;
}

/// Sanity gate for a price row from an INJECTED source (audit R1-LP-3):
/// the compiled-in families are checked by invariant tests, but an
/// injection wraps a live database row, and a corrupt row must not be
/// able to make `estimate_cost` negative or NaN (a NaN silently defeats
/// every `>= budget` comparison).
fn injected_pricing_is_sane(p: &FamilyPricing) -> bool {
    let finite_non_negative = |v: f64| v.is_finite() && v >= 0.0;
    let FamilyPricing {
        input_per_million,
        output_per_million,
        cached_per_million,
    } = p;
    finite_non_negative(*input_per_million)
        && finite_non_negative(*output_per_million)
        && finite_non_negative(*cached_per_million)
        && cached_per_million <= input_per_million
}

/// Warned-once flag for unsane injected pricing rows (audit R1
/// should-fix: the warn fires per lookup otherwise — a hot path).
static UNSANE_PRICING_WARNED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// The injected-source-aware resolution: the external source first, the
/// compiled-in families as the fallback. This is the CONSUMER-facing
/// lookup — `lookup_pricing` itself stays family-pure for direct callers.
///
/// An injected row that fails [`injected_pricing_is_sane`] is ignored
/// (warned once per process) and the compiled-in table answers instead —
/// a corrupt DB row degrades to conservative peak pricing, never to
/// free or NaN.
pub fn resolve_pricing(model: &str) -> Option<FamilyPricing> {
    let snapshot = INJECTED_SOURCE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(p) = snapshot.as_ref().and_then(|source| source(model)) {
        if injected_pricing_is_sane(&p) {
            return Some(p);
        }
        if !UNSANE_PRICING_WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!(
                target: "plana_llm_provider::metering",
                "injected pricing row failed the sanity gate (finite, non-negative, cached ≤ input); \
                 falling back to the compiled-in table (further occurrences silenced)"
            );
        }
    }
    lookup_pricing(model)
}

#[cfg(test)]
mod injection_tests {
    /// The injected slot is PROCESS-wide: the injection tests serialize on
    /// this mutex so parallel test threads do not fight over the
    /// single slot.
    use super::tests::TEST_LOCK;
    use super::*;

    #[test]
    fn injected_source_answers_before_the_families() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_pricing_source_for_tests();
        let custom = FamilyPricing {
            input_per_million: 1.25,
            output_per_million: 2.5,
            cached_per_million: 0.125,
        };
        assert!(set_pricing_source(Arc::new(move |m: &str| {
            (m == "my-private-model").then_some(custom)
        })));
        // The injected source answers for its model...
        assert_eq!(resolve_pricing("my-private-model"), Some(custom));
        // ...and falls THROUGH to the families for everything else.
        let glm = resolve_pricing("glm-4.7");
        assert!(glm.is_some(), "the family table still answers glm");
        clear_pricing_source_for_tests();
    }

    /// Audit R1-LP-3: an injected row that is negative, non-finite, or
    /// has cached > input must NOT answer — it degrades to the
    /// compiled-in table. A corrupt DB row may not make cost negative or
    /// NaN (NaN silently defeats every budget comparison).
    #[test]
    fn an_unsane_injected_row_falls_back_to_the_families() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_pricing_source_for_tests();
        for bad in [
            FamilyPricing {
                input_per_million: -1.0,
                output_per_million: 2.5,
                cached_per_million: 0.125,
            },
            FamilyPricing {
                input_per_million: f64::NAN,
                output_per_million: 2.5,
                cached_per_million: 0.125,
            },
            FamilyPricing {
                input_per_million: 1.0,
                output_per_million: 2.5,
                cached_per_million: 5.0, // cached > input breaks the invariant
            },
        ] {
            assert!(set_pricing_source(Arc::new(move |m: &str| {
                (m == "my-private-model").then_some(bad)
            })));
            // The unsane row is refused: the compiled table answers
            // instead. (Compared against `lookup_pricing` rather than
            // `!= bad` — a NaN row would never compare equal to anything,
            // making a `!=` assertion vacuous.)
            assert_eq!(
                resolve_pricing("my-private-model"),
                lookup_pricing("my-private-model"),
                "an unsane injected row must fall back to the compiled table"
            );
            clear_pricing_source_for_tests();
        }
    }

    #[test]
    fn a_failing_source_falls_through_not_panics() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_pricing_source_for_tests();
        assert!(set_pricing_source(Arc::new(|_m: &str| None)));
        assert!(
            resolve_pricing("glm-4.7").is_some(),
            "None falls to the families"
        );
        clear_pricing_source_for_tests();
    }

    #[test]
    fn the_first_source_wins() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_pricing_source_for_tests();
        let first = FamilyPricing {
            input_per_million: 9.9,
            output_per_million: 9.9,
            cached_per_million: 9.9,
        };
        assert!(set_pricing_source(Arc::new(move |_m: &str| Some(first))));
        // The second install is rejected...
        assert!(!set_pricing_source(Arc::new(|_m: &str| None)));
        // ...and the first still answers.
        assert_eq!(resolve_pricing("anything"), Some(first));
        clear_pricing_source_for_tests();
    }

    /// estimate_cost inherits the injected chain: a source pricing a
    /// private model makes its cost REAL (not the provider-keyed
    /// guess) — the whole point of C2c.
    #[test]
    fn estimate_cost_honors_the_injected_source() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_pricing_source_for_tests();
        let custom = FamilyPricing {
            input_per_million: 1.0,
            output_per_million: 2.0,
            cached_per_million: 0.1,
        };
        assert!(set_pricing_source(Arc::new(move |m: &str| {
            (m == "my-private-model").then_some(custom)
        })));
        // 1M input + 1M output at 1/2 = $3.0 — NOT the generic $3/$15
        // guess the fallback would charge for an unknown family.
        let cost = estimate_cost("any-provider", "my-private-model", 1_000_000, 1_000_000);
        assert!(
            (cost - 3.0).abs() < 1e-9,
            "the injected rates apply: {cost}"
        );
        clear_pricing_source_for_tests();
    }

    #[test]
    fn lookup_pricing_stays_family_pure() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // The raw table function is NOT source-aware: an injected source
        // must not leak into direct family callers.
        clear_pricing_source_for_tests();
        assert!(set_pricing_source(Arc::new(|_m: &str| Some(
            FamilyPricing {
                input_per_million: 0.0,
                output_per_million: 0.0,
                cached_per_million: 0.0,
            }
        ))));
        let glm = lookup_pricing("glm-4.7").expect("the family table answers");
        assert!(glm.input_per_million > 0.0, "not the injected zero");
        clear_pricing_source_for_tests();
    }

    // ── T3 pricing schedule: purchase line + off-peak windows ──

    #[test]
    fn off_peak_window_membership_is_wrap_aware() {
        let night = OffPeakWindow {
            start_hour: 22,
            end_hour: 6,
            multiplier: 0.5,
        };
        assert!(night.contains(23));
        assert!(night.contains(0));
        assert!(night.contains(5));
        assert!(!night.contains(6));
        assert!(!night.contains(12));
        let day = OffPeakWindow {
            start_hour: 9,
            end_hour: 17,
            multiplier: 0.7,
        };
        assert!(!day.contains(8));
        assert!(day.contains(9));
        assert!(day.contains(16));
        assert!(!day.contains(17));
    }

    #[test]
    fn effective_multiplier_defaults_to_peak_outside_windows() {
        let schedule = PricingSchedule {
            retail: FamilyPricing {
                input_per_million: 1.0,
                output_per_million: 2.0,
                cached_per_million: 0.1,
            },
            purchase: None,
            off_peak: vec![OffPeakWindow {
                start_hour: 0,
                end_hour: 6,
                multiplier: 0.5,
            }],
        };
        assert_eq!(schedule.effective_multiplier_utc(3), 0.5);
        assert_eq!(schedule.effective_multiplier_utc(12), 1.0);
        let peak = schedule.effective_pricing_utc(12);
        assert_eq!(peak.input_per_million, 1.0);
        let off = schedule.effective_pricing_utc(3);
        assert_eq!(off.input_per_million, 0.5);
        assert_eq!(off.output_per_million, 1.0);
        assert_eq!(off.cached_per_million, 0.05);
    }

    #[test]
    fn margin_needs_the_purchase_line_and_scales_with_the_window() {
        let retail = FamilyPricing {
            input_per_million: 1.0,
            output_per_million: 2.0,
            cached_per_million: 0.1,
        };
        let purchase = FamilyPricing {
            input_per_million: 0.4,
            output_per_million: 0.8,
            cached_per_million: 0.1,
        };
        let schedule = PricingSchedule {
            retail,
            purchase: Some(purchase),
            off_peak: vec![OffPeakWindow {
                start_hour: 0,
                end_hour: 6,
                multiplier: 0.5,
            }],
        };
        // Peak: 1M in → $1.0 retail − $0.4 purchase = $0.6 margin.
        assert_eq!(schedule.margin_usd(1_000_000, 0, 12), Some(0.6));
        // Off-peak: retail halves, purchase does NOT (our cost is
        // contracted, not windowed) → $0.5 − $0.4 = $0.1.
        let off = schedule.margin_usd(1_000_000, 0, 3);
        assert!(
            matches!(off, Some(v) if (v - 0.1).abs() < 1e-9),
            "got {off:?}"
        );
        // No purchase line → no margin answer.
        let bare = PricingSchedule {
            retail,
            purchase: None,
            off_peak: vec![],
        };
        assert_eq!(bare.margin_usd(1_000_000, 0, 12), None);
    }
}
