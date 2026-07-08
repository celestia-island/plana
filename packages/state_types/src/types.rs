// Shared type definitions
//
// ModelTier and UnknownTierError moved to shared-core (model_tier module).
// This file retains the remaining state sync types.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use arona_core::ModelTier;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Failed,
    Warning,
    Waiting {
        #[serde(
            serialize_with = "serialize_datetime",
            deserialize_with = "deserialize_datetime"
        )]
        deadline: chrono::DateTime<chrono::Utc>,
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
    pub fn is_terminal(self) -> bool {
        matches!(&self, Self::Completed | Self::Failed)
    }

    pub fn is_waiting(&self) -> bool {
        matches!(&self, Self::Waiting { .. })
    }

    pub fn waiting_deadline(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        match &self {
            Self::Waiting { deadline, .. } => Some(deadline),
            _ => None,
        }
    }

    pub fn waiting_handle(&self) -> Option<&str> {
        match &self {
            Self::Waiting { handle, .. } => Some(handle),
            _ => None,
        }
    }

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
            },
            _ => {
                let s =
                    serde_json::to_string(self).unwrap_or_else(|_| "\"not_started\"".to_string());
                write!(f, "{}", s.trim_matches('"'))
            },
        }
    }
}

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
            },
            _ => Err(UnknownTaskStatusError(s.to_string())),
        }
    }
}
