//! The remaining shared state types.
//!
//! `ModelTier` now lives in `plana_core::model_tier` and is only re-exported
//! here. What stays in this module is `TaskStatus` with its parse error, plus
//! the RFC 3339 helpers that only the `Waiting` variant needs.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use plana_core::ModelTier;

/// Lifecycle of a task/todo record as carried in TUI snapshots and patches
/// (`TaskInfo::status`, `TaskPatch::status_changed`, the `TaskStatusUpdate`
/// message).
///
/// Wire form: `rename_all = "snake_case"`, so the unit variants are
/// `not_started` / `in_progress` / `paused` / `completed` / `failed` /
/// `warning`. `Waiting` is a struct variant and therefore externally tagged:
/// `{"waiting":{"deadline":"<rfc3339>","handle":"..."}}`; the deadline goes
/// through explicit serde functions, so it is written and parsed as RFC 3339.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Created but not picked up yet.
    NotStarted,
    /// Currently being worked on.
    InProgress,
    /// Started and then held; not terminal, so it can still resume.
    Paused,
    /// Finished successfully; terminal.
    Completed,
    /// Finished unsuccessfully; terminal.
    Failed,
    /// Finished with a warning the caller should look at; not terminal.
    Warning,
    /// Blocked on an external event until `deadline`, tracked by `handle`.
    Waiting {
        /// Instant after which the wait may be treated as expired, in UTC.
        #[serde(
            serialize_with = "serialize_datetime",
            deserialize_with = "deserialize_datetime"
        )]
        deadline: chrono::DateTime<chrono::Utc>,
        /// Opaque wait token: compared and echoed here, never interpreted.
        /// `new_waiting` fills it with a UUID v7 string.
        handle: String,
    },
}

fn serialize_datetime<S: serde::Serializer>(
    dt: &chrono::DateTime<chrono::Utc>,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dt.to_rfc3339())
}

fn deserialize_datetime<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error> {
    let s = String::deserialize(d)?;
    chrono::DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(serde::de::Error::custom)
}

impl TaskStatus {
    /// Whether the task has stopped for good: only `Completed` and `Failed` count,
    /// so `Warning` and `Paused` are still treated as live.
    pub fn is_terminal(self) -> bool {
        matches!(&self, Self::Completed | Self::Failed)
    }

    /// Whether the task is blocked in the `Waiting` state.
    pub fn is_waiting(&self) -> bool {
        matches!(&self, Self::Waiting { .. })
    }

    /// The wait deadline when the status is `Waiting`, otherwise `None`.
    pub fn waiting_deadline(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        match &self {
            Self::Waiting { deadline, .. } => Some(deadline),
            _ => None,
        }
    }

    /// The opaque wait handle when the status is `Waiting`, otherwise `None`.
    pub fn waiting_handle(&self) -> Option<&str> {
        match &self {
            Self::Waiting { handle, .. } => Some(handle),
            _ => None,
        }
    }

    /// Start a `Waiting` status that lasts `seconds` from now, with a freshly
    /// generated UUID v7 as its handle. The clock is read here, so the deadline is
    /// not reproducible from the argument alone.
    pub fn new_waiting(seconds: u64) -> Self {
        let deadline = chrono::Utc::now() + chrono::Duration::seconds(seconds as i64);
        let handle = uuid::Uuid::now_v7().to_string();
        Self::Waiting { deadline, handle }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Waiting { deadline, handle } => {
                write!(f, "waiting:{}:{}", handle, deadline.to_rfc3339())
            }
            _ => {
                let s =
                    serde_json::to_string(self).unwrap_or_else(|_| "\"not_started\"".to_string());
                write!(f, "{}", s.trim_matches('"'))
            }
        }
    }
}

/// Returned when a string cannot be parsed into a `TaskStatus`: the payload is
/// the rejected input, and `Display` renders `unknown task status: <input>`
/// through the `thiserror` attribute.
#[derive(Debug, Clone, thiserror::Error)]
#[error("unknown task status: {0}")]
pub struct UnknownTaskStatusError(pub String);

impl FromStr for TaskStatus {
    type Err = UnknownTaskStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "not_started" | "todo" | "pending" => Ok(Self::NotStarted),
            "in_progress" | "inprogress" | "running" => Ok(Self::InProgress),
            "paused" => Ok(Self::Paused),
            "completed" | "done" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "warning" | "warn" => Ok(Self::Warning),
            other if other.starts_with("waiting:") => {
                let parts: Vec<&str> = other.splitn(3, ':').collect();
                if parts.len() == 3 {
                    let deadline = chrono::DateTime::parse_from_rfc3339(parts[2])
                        .map_err(|e| UnknownTaskStatusError(e.to_string()))?
                        .with_timezone(&chrono::Utc);
                    Ok(Self::Waiting {
                        deadline,
                        handle: parts[1].to_string(),
                    })
                } else {
                    Err(UnknownTaskStatusError(s.to_string()))
                }
            }
            _ => Err(UnknownTaskStatusError(s.to_string())),
        }
    }
}
