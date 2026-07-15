//! Usage aggregation types and helper functions
//!
//! Provides model-level, session-level, and other fine-grained aggregation query result types,
//! along with helper functions for building aggregations from SeaORM query results.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────
// Aggregation result types
// ─────────────────────────────────────────────

/// Model-level aggregation result
///
/// Corresponds to the `usage_model_stats` database view or equivalent GROUP BY query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLevelAggregation {
    /// Model ID (corresponds to the model identifier in ModelConfig)
    pub model_id: Option<String>,
    /// Provider name (openai / anthropic / ...)
    pub provider: Option<String>,
    /// Model display name
    pub model_display_name: Option<String>,
    /// Model tier (standard / premium / ...)
    pub tier: Option<String>,
    /// Total request count
    pub request_count: i64,
    /// Total token usage
    pub total_tokens: i64,
    /// Total estimated cost (USD)
    pub total_cost_usd: f64,
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
    /// First usage time
    pub first_used_at: Option<DateTime<Utc>>,
    /// Last usage time
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Parameters for constructing a [`ModelLevelAggregation`] from a database row.
#[derive(Debug, Clone)]
pub struct ModelLevelAggregationFromRowParams {
    /// Model ID (corresponds to the model identifier in ModelConfig)
    pub model_id: Option<String>,
    /// Provider name (openai / anthropic / ...)
    pub provider: Option<String>,
    /// Model display name
    pub model_display_name: Option<String>,
    /// Model tier (standard / premium / ...)
    pub tier: Option<String>,
    /// Total request count
    pub request_count: i64,
    /// Total token usage
    pub total_tokens: i64,
    /// Total estimated cost (USD)
    pub total_cost_usd: f64,
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
    /// First usage time
    pub first_used_at: Option<DateTime<Utc>>,
    /// Last usage time
    pub last_used_at: Option<DateTime<Utc>>,
}

impl ModelLevelAggregation {
    /// Construct an instance from a flattened tuple:
    /// `(model_id, provider, display_name, tier, request_count, total_tokens, total_cost_usd, avg_tokens, first_used_at, last_used_at)`.
    ///
    /// This signature corresponds to the SeaORM `into_tuple` projection.
    pub fn from_row(params: ModelLevelAggregationFromRowParams) -> Self {
        Self {
            model_id: params.model_id,
            provider: params.provider,
            model_display_name: params.model_display_name,
            tier: params.tier,
            request_count: params.request_count,
            total_tokens: params.total_tokens,
            total_cost_usd: params.total_cost_usd,
            avg_tokens_per_request: params.avg_tokens_per_request,
            first_used_at: params.first_used_at,
            last_used_at: params.last_used_at,
        }
    }

    /// Compute the percentage of total token usage (0.0 ~ 100.0).
    pub fn token_share(&self, total_tokens_all_models: i64) -> f64 {
        if total_tokens_all_models == 0 {
            return 0.0;
        }
        self.total_tokens as f64 / total_tokens_all_models as f64 * 100.0
    }
}

/// Session-level aggregation result
///
/// Corresponds to a fine-grained statistics query grouped by `conversation_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLevelAggregation {
    /// Session (conversation) ID
    pub conversation_id: Uuid,
    /// Agent type associated with the session
    pub agent_type: Option<String>,
    /// Session creation time
    pub session_created_at: DateTime<Utc>,
    /// Session last update time
    pub session_updated_at: DateTime<Utc>,
    /// Total message count
    pub message_count: i64,
    /// Token total usage
    pub total_tokens: i64,
    /// Total estimated cost (USD)
    pub total_cost_usd: f64,
    /// Number of distinct models used in the session
    pub models_used: i64,
    /// First message time
    pub first_message_at: Option<DateTime<Utc>>,
    /// Last message time
    pub last_message_at: Option<DateTime<Utc>>,
}

/// Parameters for constructing a [`SessionLevelAggregation`] from a database row.
#[derive(Debug, Clone)]
pub struct SessionLevelAggregationFromRowParams {
    /// Session (conversation) ID
    pub conversation_id: Uuid,
    /// Agent type associated with the session
    pub agent_type: Option<String>,
    /// Session creation time
    pub session_created_at: DateTime<Utc>,
    /// Session last update time
    pub session_updated_at: DateTime<Utc>,
    /// Total message count
    pub message_count: i64,
    /// Token total usage
    pub total_tokens: i64,
    /// Total estimated cost (USD)
    pub total_cost_usd: f64,
    /// Number of distinct models used in the session
    pub models_used: i64,
    /// First message time
    pub first_message_at: Option<DateTime<Utc>>,
    /// Last message time
    pub last_message_at: Option<DateTime<Utc>>,
}

impl SessionLevelAggregation {
    /// Construct an instance from a flattened tuple.
    ///
    /// The tuple field order matches the SeaORM `into_tuple` projection:
    /// `(conversation_id, agent_type, session_created_at, session_updated_at,
    ///   message_count, total_tokens, total_cost_usd, models_used,
    ///   first_message_at, last_message_at)`
    pub fn from_row(params: SessionLevelAggregationFromRowParams) -> Self {
        Self {
            conversation_id: params.conversation_id,
            agent_type: params.agent_type,
            session_created_at: params.session_created_at,
            session_updated_at: params.session_updated_at,
            message_count: params.message_count,
            total_tokens: params.total_tokens,
            total_cost_usd: params.total_cost_usd,
            models_used: params.models_used,
            first_message_at: params.first_message_at,
            last_message_at: params.last_message_at,
        }
    }

    /// Session duration (if both first and last message times are present).
    pub fn duration(&self) -> Option<Duration> {
        match (self.first_message_at, self.last_message_at) {
            (Some(first), Some(last)) => Some(last - first),
            _ => None,
        }
    }

    /// Average tokens per message.
    pub fn avg_tokens_per_message(&self) -> f64 {
        if self.message_count == 0 {
            return 0.0;
        }
        self.total_tokens as f64 / self.message_count as f64
    }
}

// ─────────────────────────────────────────────
// Aggregation summary
// ─────────────────────────────────────────────

/// Multi-dimension aggregation summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregationSummary {
    /// Model-level aggregation list (sorted by total_tokens descending)
    pub model_aggregations: Vec<ModelLevelAggregation>,
    /// Session-level aggregation list (sorted by total_tokens descending)
    pub session_aggregations: Vec<SessionLevelAggregation>,
    /// Global total tokens (for computing share percentages)
    pub grand_total_tokens: i64,
    /// Global total cost (USD)
    pub grand_total_cost_usd: f64,
    /// Stats time range start
    pub period_start: Option<DateTime<Utc>>,
    /// Stats time range end
    pub period_end: Option<DateTime<Utc>>,
}

impl AggregationSummary {
    /// Build a summary from a model aggregation list, automatically computing global totals.
    pub fn from_model_aggregations(
        model_aggregations: Vec<ModelLevelAggregation>,
        period_start: Option<DateTime<Utc>>,
        period_end: Option<DateTime<Utc>>,
    ) -> Self {
        let grand_total_tokens: i64 = model_aggregations.iter().map(|m| m.total_tokens).sum();
        let grand_total_cost_usd: f64 = model_aggregations.iter().map(|m| m.total_cost_usd).sum();
        Self {
            model_aggregations,
            session_aggregations: Vec::new(),
            grand_total_tokens,
            grand_total_cost_usd,
            period_start,
            period_end,
        }
    }

    /// Append a session-level aggregation list.
    pub fn with_session_aggregations(
        mut self,
        session_aggregations: Vec<SessionLevelAggregation>,
    ) -> Self {
        self.session_aggregations = session_aggregations;
        self
    }

    /// Return the top N models by token usage.
    pub fn top_models(&self, n: usize) -> &[ModelLevelAggregation] {
        let end = n.min(self.model_aggregations.len());
        &self.model_aggregations[..end]
    }

    /// Return the top N sessions by token usage.
    pub fn top_sessions(&self, n: usize) -> &[SessionLevelAggregation] {
        let end = n.min(self.session_aggregations.len());
        &self.session_aggregations[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::Utc;

    #[test]
    fn model_aggregation_token_share() -> Result<()> {
        let agg = ModelLevelAggregation::from_row(ModelLevelAggregationFromRowParams {
            model_id: Some("gpt-4o".into()),
            provider: Some("openai".into()),
            model_display_name: Some("GPT-4o".into()),
            tier: Some("premium".into()),
            request_count: 10,
            total_tokens: 500,
            total_cost_usd: 0.25,
            avg_tokens_per_request: 50.0,
            first_used_at: None,
            last_used_at: None,
        });
        let share = agg.token_share(1000);
        assert!((share - 50.0).abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn session_aggregation_avg_tokens() -> Result<()> {
        let now = Utc::now();
        let agg = SessionLevelAggregation::from_row(SessionLevelAggregationFromRowParams {
            conversation_id: "550e8400-e29b-41d4-a716-446655440000".parse()?,
            agent_type: Some("aporia".into()),
            session_created_at: now,
            session_updated_at: now,
            message_count: 4,
            total_tokens: 200,
            total_cost_usd: 0.10,
            models_used: 2,
            first_message_at: Some(now),
            last_message_at: Some(now),
        });
        assert!((agg.avg_tokens_per_message() - 50.0).abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn summary_top_models() -> Result<()> {
        let models = (0..5)
            .map(|i| ModelLevelAggregation {
                model_id: Some(format!("model-{i}")),
                provider: None,
                model_display_name: None,
                tier: None,
                request_count: i,
                total_tokens: i * 100,
                total_cost_usd: 0.0,
                avg_tokens_per_request: 100.0,
                first_used_at: None,
                last_used_at: None,
            })
            .collect();
        let summary = AggregationSummary::from_model_aggregations(models, None, None);
        assert_eq!(summary.top_models(3).len(), 3);
        Ok(())
    }
}
