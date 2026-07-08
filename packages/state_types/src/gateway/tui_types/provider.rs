use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use arona_config::ConfiguredProvider;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PeriodType {
    Hour5,
    Day7,
    Month1,
}

impl PeriodType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePeriodData {
    pub user_id: Uuid,
    pub period_type: PeriodType,
    pub start_time: DateTime<Utc>,
    pub used_tokens: u64,
    pub cost: f64,
    pub remaining_tokens: Option<u64>,
    pub remaining_cost: Option<f64>,
}
