//! Shared enums and types for the agent thread/orchestration layer.
//!
//! [`LogLevel`] is the canonical log severity enum (Error through Trace + Notice)
//! with display helpers and boot-parsing. [`LogEntry`] represents a parsed log
//! line — from both structured JSON (tracing-subscriber) and journald/journalctl
//! sources — with timestamp, level, target, and message fields.
//!
//! Also provides [`ConnectionState`] (Disconnected → Connecting → Connected ↔
//! Reconnecting → Offline) and [`ThreadCommand`] (Shutdown, Pause, Resume) used
//! by agent thread controllers and the orchestration event loop.

use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
    Notice,
}

impl LogLevel {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ERROR" => Self::Error,
            "WARN" | "WARNING" => Self::Warn,
            "INFO" => Self::Info,
            "DEBUG" => Self::Debug,
            "TRACE" => Self::Trace,
            "NOTICE" => Self::Notice,
            _ => Self::Info,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
            Self::Notice => "NOTICE",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Self::Error => "ERR",
            Self::Warn => "WRN",
            Self::Info => "INF",
            Self::Debug => "DBG",
            Self::Trace => "TRC",
            Self::Notice => "NTC",
        }
    }

    pub fn boot_tag(self) -> &'static str {
        match self {
            Self::Error => "[ FAIL ]",
            Self::Warn => "[ WARN ]",
            Self::Info => "[ INFO ]",
            Self::Debug => "[ DBG  ]",
            Self::Trace => "[ TRC  ]",
            Self::Notice => "[ INFO ]",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn is_warn(self) -> bool {
        matches!(self, Self::Warn)
    }

    pub fn is_debug_or_trace(self) -> bool {
        matches!(self, Self::Debug | Self::Trace)
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    #[serde(default)]
    pub raw: String,
}

impl LogEntry {
    pub fn new(timestamp: String, level: LogLevel, target: String, message: String) -> Self {
        Self {
            timestamp,
            level,
            target,
            message,
            raw: String::new(),
        }
    }

    pub fn parse_json(line: &str) -> Option<Self> {
        let json = serde_json::from_str::<serde_json::Value>(line).ok()?;

        let timestamp = json
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let short_timestamp = if timestamp.len() >= 16 {
            timestamp[11..16].to_string()
        } else if timestamp.len() >= 5 && timestamp.contains(':') {
            let parts: Vec<&str> = timestamp.split(':').collect();
            if parts.len() >= 2 {
                format!("{}:{}", parts[0], parts[1])
            } else {
                timestamp
            }
        } else {
            timestamp
        };

        let level = json
            .get("level")
            .and_then(|v| v.as_str())
            .map(LogLevel::from_str_loose)
            .unwrap_or_default();

        let target = json
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let message = json
            .get("fields")
            .and_then(|f| f.get("message").or_else(|| f.get("msg")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Some(Self {
            timestamp: short_timestamp,
            level,
            target,
            message,
            raw: line.to_string(),
        })
    }

    pub fn level_short(&self) -> &'static str {
        self.level.short()
    }

    pub fn level_boot_tag(&self) -> &'static str {
        self.level.boot_tag()
    }

    pub fn is_error(&self) -> bool {
        self.level.is_error()
    }

    pub fn is_warn(&self) -> bool {
        self.level.is_warn()
    }
}

impl LogEntry {
    pub fn from_journal_fields(fields: &std::collections::HashMap<String, String>) -> Self {
        let timestamp = fields
            .get("_SOURCE_REALTIME_TIMESTAMP")
            .or_else(|| fields.get("__REALTIME_TIMESTAMP"))
            .map(|ts| {
                let micros: u64 = ts.parse().unwrap_or(0);
                let secs = micros / 1_000_000;
                let remainder = micros % 1_000_000;
                DateTime::from_timestamp(secs as i64, (remainder * 1000) as u32)
                    .map(|dt| dt.format("%H:%M").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let level = fields
            .get("PRIORITY")
            .and_then(|p| p.parse::<u8>().ok())
            .map(|p| match p {
                0..=4 => LogLevel::Error,
                5 => LogLevel::Warn,
                6 => LogLevel::Info,
                7 => LogLevel::Debug,
                _ => LogLevel::Info,
            })
            .unwrap_or_default();

        let target = fields
            .get("TARGET")
            .or_else(|| fields.get("SYSLOG_IDENTIFIER"))
            .cloned()
            .unwrap_or_default();

        let message = fields.get("MESSAGE").cloned().unwrap_or_default();

        Self {
            timestamp,
            level,
            target,
            message,
            raw: String::new(),
        }
    }

    pub fn from_journalctl_json(line: &str) -> Option<Self> {
        let json: serde_json::Value = serde_json::from_str(line).ok()?;
        let mut fields = std::collections::HashMap::new();

        if let Some(obj) = json.as_object() {
            for (k, v) in obj {
                let val = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                fields.insert(k.clone(), val);
            }
        }

        let mut entry = Self::from_journal_fields(&fields);
        entry.raw = line.to_string();
        Some(entry)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Offline,
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn is_offline(&self) -> bool {
        matches!(self, Self::Offline | Self::Disconnected)
    }

    pub fn can_retry(&self) -> bool {
        matches!(
            self,
            Self::Disconnected | Self::Reconnecting | Self::Offline
        )
    }

    pub fn status_text(&self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting...",
            Self::Offline => "Offline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadCommand {
    Shutdown,
    Pause,
    Resume,
}
