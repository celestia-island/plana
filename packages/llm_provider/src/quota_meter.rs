//! Provider quota exhaustion detection and auto-skip.
//!
//! When a provider returns a rate-limit or quota-exhaustion error (e.g. BigModel
//! Coding Plan "5-hour usage limit reached"), the response body often contains a
//! **reset timestamp**.  This module:
//!
//! 1. Scans error bodies for reset-time patterns (Chinese / English).
//! 2. Marks the provider as *exhausted* until that timestamp.
//! 3. Exposes `is_exhausted(provider_id)` so model-selection code can skip
//!    exhausted providers until the quota resets, saving needless retries.
//!
//! ## Integration
//!
//! Call [`record_error`] from the error-handling path whenever a 429/error
//! response is received.  Call [`is_exhausted`] in the model-pool builder to
//! filter out providers whose quota hasn't reset yet.

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

// ── Global singleton ────────────────────────────────────────────────

fn meter() -> &'static QuotaMeter {
    static METER: OnceLock<QuotaMeter> = OnceLock::new();
    METER.get_or_init(QuotaMeter::new)
}

// ── Public API ───────────────────────────────────────────────────────

/// Record a provider error response and update quota state if it contains
/// a reset timestamp.
pub fn record_error(provider_id: &str, status: u16, body: &str) {
    meter().record(provider_id, status, body);
}

/// Check whether a provider is currently quota-exhausted.
/// Returns `true` if the provider should be skipped.
pub fn is_exhausted(provider_id: &str) -> bool {
    meter().check(provider_id)
}

/// Parse a reset datetime from an error body, if present.
/// Returns `None` if no recognizable reset pattern is found.
pub fn parse_quota_reset(body: &str) -> Option<DateTime<Utc>> {
    extract_reset_time(body)
}

// ── Implementation ───────────────────────────────────────────────────

struct QuotaMeter {
    /// provider_id → reset_after_this_time
    exhausted_until: RwLock<HashMap<String, DateTime<Utc>>>,
}

impl QuotaMeter {
    fn new() -> Self {
        QuotaMeter {
            exhausted_until: RwLock::new(HashMap::new()),
        }
    }

    fn record(&self, provider_id: &str, status: u16, body: &str) {
        if provider_id.is_empty() || body.is_empty() {
            return;
        }
        if status != 429 && status != 529 && status != 503 {
            return;
        }

        if let Some(reset_time) = extract_reset_time(body) {
            let mut map = self
                .exhausted_until
                .write()
                .unwrap_or_else(|e| e.into_inner());
            let entry = map.entry(provider_id.to_string()).or_insert(reset_time);
            if reset_time > *entry {
                *entry = reset_time;
            }
            tracing::info!(
                provider = %provider_id,
                reset_at = %reset_time,
                status,
                "Provider marked as quota-exhausted until reset time"
            );
        }
    }

    fn check(&self, provider_id: &str) -> bool {
        let map = self
            .exhausted_until
            .read()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(until) = map.get(provider_id) {
            let now = Utc::now();
            if now < *until {
                return true;
            }
        }
        false
    }
}

// ── Datetime extraction (regex) ─────────────────────────────────────

/// Compiled regex: captures a "YYYY-MM-DD HH:MM:SS" timestamp from error bodies.
fn datetime_re() -> Option<&'static Regex> {
    static RE: OnceLock<Result<Regex, regex::Error>> = OnceLock::new();
    let result = RE.get_or_init(|| Regex::new(r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})"));
    result.as_ref().ok()
}

/// Try to find and parse a "YYYY-MM-DD HH:MM:SS" datetime in the error body.
fn extract_reset_time(body: &str) -> Option<DateTime<Utc>> {
    datetime_re()?
        .captures(body)
        .and_then(|caps| caps.get(1))
        .and_then(|m| NaiveDateTime::parse_from_str(m.as_str(), "%Y-%m-%d %H:%M:%S").ok())
        .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bigmodel_chinese_reset() -> Result<(), Box<dyn std::error::Error>> {
        let body = "已达到 5 小时的使用上限。您的限额将在 2026-06-18 16:59:41 重置。";
        let dt = parse_quota_reset(body).ok_or("no reset time found")?;
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-06-18 16:59:41"
        );
        Ok(())
    }

    #[test]
    fn parse_compact_chinese() -> Result<(), Box<dyn std::error::Error>> {
        let body = r#"{"error":{"code":"1308","message":"已达到 5 小时的使用上限。您的限额将在 2026-06-18 16:59:41 重置。"}}"#;
        let dt = parse_quota_reset(body).ok_or("no reset time found")?;
        assert!(dt.to_string().contains("2026-06-18 16:59:41"));
        Ok(())
    }

    #[test]
    fn parse_english_reset() -> Result<(), Box<dyn std::error::Error>> {
        let body = "Rate limit exceeded. Your quota will reset at 2026-12-31 00:00:00 UTC.";
        let dt = parse_quota_reset(body).ok_or("no reset time found")?;
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-12-31 00:00:00"
        );
        Ok(())
    }

    #[test]
    fn no_reset_time_returns_none() {
        assert!(parse_quota_reset("Something went wrong").is_none());
        assert!(parse_quota_reset("").is_none());
    }

    #[test]
    fn record_and_check_exhaustion() {
        let provider = "test_provider";
        let body = "您的限额将在 2099-12-31 00:00:00 重置";
        record_error(provider, 429, body);
        assert!(is_exhausted(provider));
    }

    #[test]
    fn past_reset_time_not_exhausted() {
        let provider = "old_provider";
        let body = "您的限额将在 2020-01-01 00:00:00 重置";
        record_error(provider, 429, body);
        assert!(
            !is_exhausted(provider),
            "past reset time should not be exhausted"
        );
    }
}
