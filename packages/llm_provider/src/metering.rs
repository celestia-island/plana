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

/// Resolve a model id to its family's [`FamilyPricing`] by substring
/// match (lowercased). All prices are PEAK-tier — the conservative
/// upper bound; the actual off-peak billing split lives with the
/// provider's ledger, not here. Returns `None` for unknown families;
/// callers fall back to provider-keyed guesses (see `estimate_cost`).
///
/// FX convention: CNY prices convert at the rate documented next to
/// each family (the reference rate at the time that family's prices
/// were entered — NOT a single global rate, because official USD
/// price lists and CNY lists disagree per family; see each arm's
/// comment for the source rate).
pub fn lookup_pricing(model: &str) -> Option<FamilyPricing> {
    let lower = model.to_lowercase();
    // The compiled-in BAND HEADS mirror the provider-registry's current
    // generation (verified against the vendors' official lists,
    // 2026-10-03 — see PR for the per-family provenance). This table is
    // the universal FALLBACK: arona's model_pricing DB rows win first,
    // and an injected source (set_pricing_source) wins over both.
    // GLM (Zhipu). Pro band = glm-5.2/5.3 official ¥8/¥28/¥2 (÷6.5 →
    // 1.23/4.31/0.31); Zhipu's OWN flash tier is a distinct cheaper
    // band (glm-5.3-flash ¥0.8/¥2.8/¥0.23 → 0.12/0.43/0.035). Order
    // matters: "glm" before the generic checks.
    if lower.contains("glm") {
        if lower.contains("flashx") {
            // glm-5.3-flashx official.
            return Some(FamilyPricing {
                input_per_million: 0.31,
                output_per_million: 1.08,
                cached_per_million: 0.088,
            });
        }
        if lower.contains("prime") {
            // glm-5.3-prime official.
            return Some(FamilyPricing {
                input_per_million: 2.80,
                output_per_million: 8.80,
                cached_per_million: 0.56,
            });
        }
        if lower.contains("flash") || lower.contains("air") || lower.contains("lite") {
            return Some(FamilyPricing {
                input_per_million: 0.12,
                output_per_million: 0.43,
                cached_per_million: 0.035,
            });
        }
        return Some(FamilyPricing {
            input_per_million: 8.0 / 6.5,
            output_per_million: 28.0 / 6.5,
            cached_per_million: 2.0 / 6.5,
        });
    }
    if lower.contains("deepseek") {
        if lower.contains("pro") || lower.contains("r1") || lower.contains("reasoner") {
            // v4-pro official USD list.
            return Some(FamilyPricing {
                input_per_million: 0.435,
                output_per_million: 0.87,
                cached_per_million: 0.003625,
            });
        }
        if lower.contains("v4.1") || lower.contains("flash-latest") {
            // v4.1-flash official USD list.
            return Some(FamilyPricing {
                input_per_million: 0.30,
                output_per_million: 1.20,
                cached_per_million: 0.006,
            });
        }
        // v4-flash official USD list.
        return Some(FamilyPricing {
            input_per_million: 0.14,
            output_per_million: 0.28,
            cached_per_million: 0.0028,
        });
    }
    if lower.contains("kimi") || lower.contains("moonshot") {
        // Kimi K3 official USD list.
        return Some(FamilyPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cached_per_million: 0.3,
        });
    }
    if lower.contains("claude") {
        if lower.contains("haiku") {
            if lower.contains("haiku-4") {
                // Haiku 4.5 official.
                return Some(FamilyPricing {
                    input_per_million: 1.00,
                    output_per_million: 5.00,
                    cached_per_million: 0.10,
                });
            }
            // The 3.x haiku list.
            return Some(FamilyPricing {
                input_per_million: 0.80,
                output_per_million: 4.00,
                cached_per_million: 0.08,
            });
        }
        if lower.contains("opus-5") {
            // Opus 5.5 official.
            return Some(FamilyPricing {
                input_per_million: 4.00,
                output_per_million: 20.00,
                cached_per_million: 0.20,
            });
        }
        if lower.contains("opus-4-8-fast") || lower.contains("opus-4.8-fast") {
            // Opus 4.8-fast official.
            return Some(FamilyPricing {
                input_per_million: 10.00,
                output_per_million: 50.00,
                cached_per_million: 1.00,
            });
        }
        if lower.contains("opus-4.6-fast")
            || lower.contains("opus-4.7-fast")
            || lower.contains("opus-4-6-fast")
            || lower.contains("opus-4-7-fast")
        {
            // The 4.6/4.7 fast lanes official.
            return Some(FamilyPricing {
                input_per_million: 30.00,
                output_per_million: 150.00,
                cached_per_million: 3.00,
            });
        }
        if lower.contains("opus-4-5")
            || lower.contains("opus-4-6")
            || lower.contains("opus-4-7")
            || lower.contains("opus-4-8")
            || lower.contains("opus-4.5")
            || lower.contains("opus-4.6")
            || lower.contains("opus-4.7")
            || lower.contains("opus-4.8")
        {
            // The 4.5–4.8 line (4.8 official).
            return Some(FamilyPricing {
                input_per_million: 5.00,
                output_per_million: 25.00,
                cached_per_million: 0.50,
            });
        }
        if lower.contains("opus") {
            // The legacy opus list (opus-4 / 4.1 / 3.x).
            return Some(FamilyPricing {
                input_per_million: 15.00,
                output_per_million: 75.00,
                cached_per_million: 1.50,
            });
        }
        if lower.contains("fable") {
            // The Fable line (5 / 5.1 share 10/50; 5.1's cache read).
            return Some(FamilyPricing {
                input_per_million: 10.00,
                output_per_million: 50.00,
                cached_per_million: 0.25,
            });
        }
        if lower.contains("sonnet-5") {
            // Sonnet 5 / 5.5 official.
            return Some(FamilyPricing {
                input_per_million: 2.00,
                output_per_million: 10.00,
                cached_per_million: 0.20,
            });
        }
        // The sonnet-4 line list.
        return Some(FamilyPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
            cached_per_million: 0.30,
        });
    }
    if lower.contains("gemini-3")
        || lower.contains("gemini-flash-latest")
        || lower.contains("gemini-lite-latest")
        || lower.contains("gemini-flash-lite-latest")
    {
        if lower.contains("lite") {
            // 3.1-flash-lite official.
            return Some(FamilyPricing {
                input_per_million: 0.25,
                output_per_million: 1.50,
                cached_per_million: 0.025,
            });
        }
        if lower.contains("flash") {
            // 3.5-flash official.
            return Some(FamilyPricing {
                input_per_million: 1.50,
                output_per_million: 9.00,
                cached_per_million: 0.15,
            });
        }
        // The 3.x pro band rides 2.5-pro's official list.
        return Some(FamilyPricing {
            input_per_million: 1.25,
            output_per_million: 10.00,
            cached_per_million: 0.125,
        });
    }
    if lower.contains("gemini") {
        // The legacy 2.x bands.
        if lower.contains("flash") {
            return Some(FamilyPricing {
                input_per_million: 0.075,
                output_per_million: 0.30,
                cached_per_million: 0.0188,
            });
        }
        return Some(FamilyPricing {
            input_per_million: 1.25,
            output_per_million: 5.00,
            cached_per_million: 0.3125,
        });
    }
    if lower.contains("o1") || lower.contains("o3") {
        return Some(FamilyPricing {
            input_per_million: 10.00,
            output_per_million: 40.00,
            cached_per_million: 2.50,
        });
    }
    if lower.contains("gpt") && lower.contains("mini") {
        // BEFORE the generation bands: every *-mini id is its own cheaper
        // tier (round-1 caught the broad gpt-5 arm dominating this check —
        // gpt-5.4-mini priced at the full 5.4 rate, 3.3× overcharge).
        if lower.contains("gpt-5") {
            // gpt-5.4-mini official.
            return Some(FamilyPricing {
                input_per_million: 0.75,
                output_per_million: 4.50,
                cached_per_million: 0.075,
            });
        }
        // The 4o-mini official list.
        return Some(FamilyPricing {
            input_per_million: 0.15,
            output_per_million: 0.60,
            cached_per_million: 0.075,
        });
    }
    if lower.contains("gpt") && lower.contains("nano") {
        if lower.contains("5.4") {
            // gpt-5.4-nano official.
            return Some(FamilyPricing {
                input_per_million: 0.20,
                output_per_million: 1.25,
                cached_per_million: 0.02,
            });
        }
        if lower.contains("gpt-5") {
            // gpt-5-nano official.
            return Some(FamilyPricing {
                input_per_million: 0.05,
                output_per_million: 0.40,
                cached_per_million: 0.005,
            });
        }
        // gpt-4.1-nano legacy list.
        return Some(FamilyPricing {
            input_per_million: 0.10,
            output_per_million: 0.40,
            cached_per_million: 0.025,
        });
    }
    if lower.contains("gpt-5") && lower.contains("pro") {
        // The 5.x pro line (5.4/5.5 official 30/180; 5.2-pro 21/168
        // rides the headline).
        return Some(FamilyPricing {
            input_per_million: 30.00,
            output_per_million: 180.00,
            cached_per_million: 3.00,
        });
    }
    if lower.contains("gpt-6") {
        // The gpt-6 generation: astra (flagship) / sol (standard) /
        // luna (light) — official standard-tier USD lists.
        if lower.contains("astra") {
            return Some(FamilyPricing {
                input_per_million: 10.00,
                output_per_million: 50.00,
                cached_per_million: 1.00,
            });
        }
        if lower.contains("luna") {
            return Some(FamilyPricing {
                input_per_million: 0.10,
                output_per_million: 0.50,
                cached_per_million: 0.01,
            });
        }
        // sol band (covers 6.1-sol).
        return Some(FamilyPricing {
            input_per_million: 2.00,
            output_per_million: 10.00,
            cached_per_million: 0.20,
        });
    }
    if lower.contains("daybreak") {
        // The daybreak pair (official).
        if lower.contains("red") {
            return Some(FamilyPricing {
                input_per_million: 12.50,
                output_per_million: 75.00,
                cached_per_million: 1.25,
            });
        }
        return Some(FamilyPricing {
            input_per_million: 4.00,
            output_per_million: 20.00,
            cached_per_million: 0.40,
        });
    }
    if lower.contains("gpt-5.6") {
        // Official: base/sol 4/20/0.4, terra 2/12/0.2, luna 0.2/1.2/0.02 —
        // the band headline is the sol/base rate.
        return Some(FamilyPricing {
            input_per_million: 4.00,
            output_per_million: 20.00,
            cached_per_million: 0.40,
        });
    }
    if lower.contains("gpt-5.5") {
        return Some(FamilyPricing {
            input_per_million: 5.00,
            output_per_million: 30.00,
            cached_per_million: 0.50,
        });
    }
    if lower.contains("gpt-5") {
        // The broad 5.x band (5/5.1/5.2/5.4 base ≈ 1.25–2.5); headline
        // 5.4's official 2.5/15.
        return Some(FamilyPricing {
            input_per_million: 2.50,
            output_per_million: 15.00,
            cached_per_million: 0.25,
        });
    }
    if lower.contains("gpt-4o") {
        return Some(FamilyPricing {
            input_per_million: 2.50,
            output_per_million: 10.00,
            cached_per_million: 1.25,
        });
    }
    if lower.contains("gpt-4") {
        return Some(FamilyPricing {
            input_per_million: 30.00,
            output_per_million: 60.00,
            cached_per_million: 30.00,
        });
    }
    if lower.contains("gpt-3.5") {
        return Some(FamilyPricing {
            input_per_million: 0.50,
            output_per_million: 1.50,
            cached_per_million: 0.50,
        });
    }
    if lower.contains("qwen") {
        return Some(FamilyPricing {
            input_per_million: 0.50,
            output_per_million: 2.00,
            cached_per_million: 0.125,
        });
    }
    if lower.contains("llama-3") || lower.contains("llama3") {
        return Some(FamilyPricing {
            input_per_million: 0.20,
            output_per_million: 0.80,
            cached_per_million: 0.20,
        });
    }
    if lower.contains("mistral") {
        return Some(FamilyPricing {
            input_per_million: 0.20,
            output_per_million: 0.80,
            cached_per_million: 0.20,
        });
    }
    None
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
        // GLM 5.3 — ¥8/¥28/¥2 at FX 6.5; flash rides its own band.
        let glm = lookup_pricing("glm-5.3").expect("glm");
        assert!((glm.input_per_million - 8.0 / 6.5).abs() < 1e-9);
        assert!((glm.cached_per_million - 2.0 / 6.5).abs() < 1e-9);
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

/// The injected-source-aware resolution: the external source first, the
/// compiled-in families as the fallback. This is the CONSUMER-facing
/// lookup — `lookup_pricing` itself stays family-pure for direct callers.
pub fn resolve_pricing(model: &str) -> Option<FamilyPricing> {
    let snapshot = INJECTED_SOURCE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(p) = snapshot.as_ref().and_then(|source| source(model)) {
        return Some(p);
    }
    lookup_pricing(model)
}

#[cfg(test)]
mod injection_tests {
    use super::*;
    /// The injected slot is PROCESS-wide: the injection tests serialize on
    /// this mutex so parallel test threads do not fight over the
    /// single slot.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
}
