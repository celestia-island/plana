use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Default,
    ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn from_deviation(deviation: f64, threshold: f64) -> Self {
        let ratio = if threshold == 0.0 {
            0.0
        } else {
            deviation / threshold
        };
        if ratio > 3.0 {
            Severity::Critical
        } else if ratio > 2.0 {
            Severity::High
        } else if ratio > 1.5 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Severity::Low => 1.0,
            Severity::Medium => 2.0,
            Severity::High => 3.0,
            Severity::Critical => 4.0,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            _ => Err(format!("unknown severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    #[default]
    Pass,
    Fail,
    NotApplicable,
}

impl CheckStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckStatus::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, CheckStatus::Fail)
    }
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckStatus::Pass => write!(f, "pass"),
            CheckStatus::Fail => write!(f, "fail"),
            CheckStatus::NotApplicable => write!(f, "not_applicable"),
        }
    }
}

impl std::str::FromStr for CheckStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pass" => Ok(CheckStatus::Pass),
            "fail" => Ok(CheckStatus::Fail),
            "not_applicable" => Ok(CheckStatus::NotApplicable),
            _ => Err(format!("unknown check status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    Threshold,
    Range,
    Condition,
    Periodic,
}

impl fmt::Display for CheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckType::Threshold => write!(f, "threshold"),
            CheckType::Range => write!(f, "range"),
            CheckType::Condition => write!(f, "condition"),
            CheckType::Periodic => write!(f, "periodic"),
        }
    }
}

impl std::str::FromStr for CheckType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "threshold" => Ok(CheckType::Threshold),
            "range" => Ok(CheckType::Range),
            "condition" => Ok(CheckType::Condition),
            "periodic" => Ok(CheckType::Periodic),
            _ => Err(format!("unknown check type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    ComplianceRules,
    ComplianceAudit,
    Full,
    Quick,
}

impl fmt::Display for ScanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanType::ComplianceRules => write!(f, "compliance_rules"),
            ScanType::ComplianceAudit => write!(f, "compliance_audit"),
            ScanType::Full => write!(f, "full"),
            ScanType::Quick => write!(f, "quick"),
        }
    }
}

impl std::str::FromStr for ScanType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compliance_rules" => Ok(ScanType::ComplianceRules),
            "compliance_audit" => Ok(ScanType::ComplianceAudit),
            "full" => Ok(ScanType::Full),
            "quick" => Ok(ScanType::Quick),
            _ => Err(format!("unknown scan type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ConversationMessageType {
    TaskRequest,
    ContextSupplement,
    Question,
}

impl fmt::Display for ConversationMessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationMessageType::TaskRequest => write!(f, "task_request"),
            ConversationMessageType::ContextSupplement => write!(f, "context_supplement"),
            ConversationMessageType::Question => write!(f, "question"),
        }
    }
}

impl std::str::FromStr for ConversationMessageType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "task_request" => Ok(ConversationMessageType::TaskRequest),
            "context_supplement" => Ok(ConversationMessageType::ContextSupplement),
            "question" => Ok(ConversationMessageType::Question),
            _ => Err(format!("unknown message type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Priority {
    High,
    #[default]
    Normal,
    Low,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::High => write!(f, "high"),
            Priority::Normal => write!(f, "normal"),
            Priority::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(Priority::High),
            "normal" => Ok(Priority::Normal),
            "low" => Ok(Priority::Low),
            _ => Err(format!("unknown priority: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ObservationType {
    Reading,
    Editing,
    Deleting,
    Watching,
}

impl fmt::Display for ObservationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObservationType::Reading => write!(f, "reading"),
            ObservationType::Editing => write!(f, "editing"),
            ObservationType::Deleting => write!(f, "deleting"),
            ObservationType::Watching => write!(f, "watching"),
        }
    }
}

impl std::str::FromStr for ObservationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reading" => Ok(ObservationType::Reading),
            "editing" => Ok(ObservationType::Editing),
            "deleting" => Ok(ObservationType::Deleting),
            "watching" => Ok(ObservationType::Watching),
            _ => Err(format!("unknown observation type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    FileChange,
    Schedule,
    Manual,
}

impl fmt::Display for TriggerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TriggerType::FileChange => write!(f, "file_change"),
            TriggerType::Schedule => write!(f, "schedule"),
            TriggerType::Manual => write!(f, "manual"),
        }
    }
}

impl std::str::FromStr for TriggerType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file_change" => Ok(TriggerType::FileChange),
            "schedule" => Ok(TriggerType::Schedule),
            "manual" => Ok(TriggerType::Manual),
            _ => Err(format!("unknown trigger type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(format!("unknown task status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AlignmentStatus {
    Aligned,
    NeedsAttention,
    #[default]
    Unknown,
    Error,
}

impl fmt::Display for AlignmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlignmentStatus::Aligned => write!(f, "aligned"),
            AlignmentStatus::NeedsAttention => write!(f, "needs_attention"),
            AlignmentStatus::Unknown => write!(f, "unknown"),
            AlignmentStatus::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for AlignmentStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aligned" => Ok(AlignmentStatus::Aligned),
            "needs_attention" => Ok(AlignmentStatus::NeedsAttention),
            "unknown" => Ok(AlignmentStatus::Unknown),
            "error" => Ok(AlignmentStatus::Error),
            _ => Err(format!("unknown alignment status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    NoData,
}

impl fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComplianceStatus::Compliant => write!(f, "compliant"),
            ComplianceStatus::NonCompliant => write!(f, "non_compliant"),
            ComplianceStatus::NoData => write!(f, "no_data"),
        }
    }
}

impl std::str::FromStr for ComplianceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compliant" => Ok(ComplianceStatus::Compliant),
            "non_compliant" => Ok(ComplianceStatus::NonCompliant),
            "no_data" => Ok(ComplianceStatus::NoData),
            _ => Err(format!("unknown compliance status: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn severity_from_deviation() {
        assert_eq!(Severity::from_deviation(1.0, 1.0), Severity::Low);
        assert_eq!(Severity::from_deviation(3.5, 1.0), Severity::Critical);
        assert_eq!(Severity::from_deviation(2.5, 1.0), Severity::High);
        assert_eq!(Severity::from_deviation(1.6, 1.0), Severity::Medium);
    }

    #[test]
    fn check_status_roundtrip() {
        for status in [
            CheckStatus::Pass,
            CheckStatus::Fail,
            CheckStatus::NotApplicable,
        ] {
            assert_eq!(status.to_string().parse::<CheckStatus>(), Ok(status));
        }
    }

    #[test]
    fn scan_type_roundtrip() {
        for st in [
            ScanType::ComplianceRules,
            ScanType::ComplianceAudit,
            ScanType::Full,
            ScanType::Quick,
        ] {
            assert_eq!(st.to_string().parse::<ScanType>(), Ok(st));
        }
    }

    #[test]
    fn serde_json_roundtrip() -> Result<()> {
        let s = serde_json::to_string(&Severity::High)?;
        assert_eq!(s, "\"high\"");
        let back: Severity = serde_json::from_str(&s)?;
        assert_eq!(back, Severity::High);
        Ok(())
    }
}
