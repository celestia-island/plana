//! Provider-facing wire mirrors: configured-provider rows, usage-period kinds
//! and per-period usage counters.
//!
//! Masking invariant: nothing here carries API-key material. The
//! configured-provider row reduces a stored key to a presence flag, so no
//! secret ever crosses the sync socket.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A provider the user has configured, as listed by
/// `Sync.ConfiguredProvidersList`: the key itself is never carried — only the
/// `is_enabled` flag derived from "a key is configured" (definition lives in
/// `plana_config::provider_crud`).
pub use plana_config::ConfiguredProvider;

/// Billing window a usage counter is tracked over. Serialized with the variant
/// name verbatim (`"Hour5"`, `"Day7"`, `"Month1"`), and `Display` / `FromStr`
/// round-trip on that same token rather than on the human label.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PeriodType {
    /// Rolling 5-hour window; `display_name()` renders it as `5 Hours`.
    Hour5,
    /// Rolling 7-day window; `display_name()` renders it as `7 Days`.
    Day7,
    /// One-month window; `display_name()` renders it as `1 Month`.
    Month1,
}

impl PeriodType {
    /// Human-readable label for the window (`5 Hours` / `7 Days` / `1 Month`),
    /// distinct from the serialized token used on the wire.
    pub fn display_name(&self) -> &'static str {
        match self {
            PeriodType::Hour5 => "5 Hours",
            PeriodType::Day7 => "7 Days",
            PeriodType::Month1 => "1 Month",
        }
    }

    fn serial_name(&self) -> &'static str {
        match self {
            PeriodType::Hour5 => "Hour5",
            PeriodType::Day7 => "Day7",
            PeriodType::Month1 => "Month1",
        }
    }

    fn from_serial_name(s: &str) -> Option<Self> {
        match s {
            "Hour5" => Some(PeriodType::Hour5),
            "Day7" => Some(PeriodType::Day7),
            "Month1" => Some(PeriodType::Month1),
            _ => None,
        }
    }
}

impl std::str::FromStr for PeriodType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_serial_name(s).ok_or_else(|| format!("unknown PeriodType: {}", s))
    }
}

impl std::fmt::Display for PeriodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.serial_name())
    }
}

/// Usage one user accumulated inside one billing window, as sent in
/// `Sync.UsagePeriodResponse` and `Sync.UsagePeriodUpdate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePeriodData {
    /// User the counters belong to.
    pub user_id: Uuid,
    /// Window these counters cover.
    pub period_type: PeriodType,
    /// Start of the current window, in UTC.
    pub start_time: DateTime<Utc>,
    /// Tokens consumed in the window so far.
    pub used_tokens: u64,
    /// Spend accumulated in the window; the wire fixes no currency, so pair it
    /// with the provider's billing currency.
    pub cost: f64,
    /// Remaining token allowance for the window; `None` when the payload reports
    /// no token cap.
    pub remaining_tokens: Option<u64>,
    /// Remaining spend allowance for the window; `None` when the payload reports
    /// no cost cap.
    pub remaining_cost: Option<f64>,
}
