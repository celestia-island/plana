//! YOLO Cruise Control: the autonomous background-loop tiers (realtime through
//! strategic), the tasks each tier schedules and the status reported for them.
use serde::{Deserialize, Serialize};

/// Cadence class of a YOLO background task, from 2-minute realtime checks to
/// weekly strategic passes. Serialized `snake_case` (`"realtime"`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YoloTaskTier {
    /// Fastest tier: 120 s default interval, disabled on a fresh config.
    Realtime,
    /// Hourly tier (3600 s default), enabled on a fresh config.
    Periodic,
    /// Six-hourly tier (21600 s default), enabled on a fresh config.
    Daily,
    /// Weekly tier (604800 s default), disabled on a fresh config.
    Strategic,
}

impl YoloTaskTier {
    /// All four tiers, ordered from the fastest cadence to the slowest; used to
    /// build UI lists without hard-coding the variants.
    pub fn all() -> &'static [YoloTaskTier] {
        &[
            YoloTaskTier::Realtime,
            YoloTaskTier::Periodic,
            YoloTaskTier::Daily,
            YoloTaskTier::Strategic,
        ]
    }

    /// Stable label of the tier (`"realtime"` ...), identical to its serde
    /// form, so it doubles as the wire/config token.
    pub fn name(&self) -> &'static str {
        match self {
            YoloTaskTier::Realtime => "realtime",
            YoloTaskTier::Periodic => "periodic",
            YoloTaskTier::Daily => "daily",
            YoloTaskTier::Strategic => "strategic",
        }
    }

    /// Parses a `name()` token back into a tier; case-sensitive, and returns
    /// `None` for unknown or empty input.
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "realtime" => Some(YoloTaskTier::Realtime),
            "periodic" => Some(YoloTaskTier::Periodic),
            "daily" => Some(YoloTaskTier::Daily),
            "strategic" => Some(YoloTaskTier::Strategic),
            _ => None,
        }
    }

    /// Default tick interval for the tier, in seconds (120 / 3600 / 21600 /
    /// 604800).
    pub fn default_interval_secs(&self) -> u64 {
        match self {
            YoloTaskTier::Realtime => 120,
            YoloTaskTier::Periodic => 3600,
            YoloTaskTier::Daily => 21600,
            YoloTaskTier::Strategic => 604800,
        }
    }

    /// Whether the tier starts enabled when a config is created from scratch:
    /// `Realtime` and `Strategic` start off, the middle two start on.
    pub fn default_enabled(&self) -> bool {
        match self {
            YoloTaskTier::Realtime => false,
            YoloTaskTier::Periodic => true,
            YoloTaskTier::Daily => true,
            YoloTaskTier::Strategic => false,
        }
    }

    /// Whether the tier should fire its tasks immediately on the first YOLO
    /// tick (cold-start bootstrap) or defer to its configured interval.
    ///
    /// Fast tiers (Realtime/Periodic) bootstrap to establish a baseline;
    /// slow tiers (Daily/Strategic) wait for their real schedule so a fresh
    /// YOLO start doesn't dump a wall of cold-start failures into the todo.
    pub fn should_bootstrap_on_first_run(&self) -> bool {
        match self {
            YoloTaskTier::Realtime | YoloTaskTier::Periodic => true,
            YoloTaskTier::Daily | YoloTaskTier::Strategic => false,
        }
    }
}

impl std::fmt::Display for YoloTaskTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// One scheduled task inside a tier: which agent runs which skill, and whether
/// it is currently switched on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierTaskConfig {
    /// Name of the agent that runs the task.
    pub agent: String,
    /// Name of the skill the agent executes.
    pub skill: String,
    /// Whether the task runs; an absent key defaults to `true` (the opposite of
    /// `YoloTierConfig.enabled`).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Configuration of one tier: on/off switch, cadence and the tasks it fires.
/// Sent in `Sync.YoloConfigResponse` and edited by `Sync.YoloUpdateTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierConfig {
    /// Tier this configuration block belongs to.
    pub tier: YoloTaskTier,
    /// Whether the tier runs; an absent key defaults to `false`, so a tier only
    /// starts enabled through `with_defaults` or an explicit value.
    #[serde(default)]
    pub enabled: bool,
    /// Tick interval in seconds; an absent key defaults to `0`, which is not
    /// the tier's own default (see `with_defaults`).
    #[serde(default)]
    pub interval_secs: u64,
    /// Tasks fired by this tier; empty when the payload omits them.
    pub tasks: Vec<YoloTierTaskConfig>,
}

impl YoloTierConfig {
    /// Builds a config for `tier` pre-filled with that tier's default enabled
    /// flag and interval, and with no tasks.
    pub fn with_defaults(tier: YoloTaskTier) -> Self {
        Self {
            enabled: tier.default_enabled(),
            interval_secs: tier.default_interval_secs(),
            tier,
            tasks: Vec::new(),
        }
    }
}

/// Runtime status of one tier as reported to the UI: schedule bookends plus the
/// per-task outcome of the latest run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierStatus {
    /// Tier this status block describes.
    pub tier: YoloTaskTier,
    /// Whether the tier is currently running.
    pub enabled: bool,
    /// Effective tick interval in seconds.
    pub interval_secs: u64,
    /// Timestamp of the last execution; `None` when the tier has never run.
    pub last_run_at: Option<String>,
    /// Timestamp of the next scheduled execution; `None` when none is
    /// scheduled.
    pub next_run_at: Option<String>,
    /// Per-task status of this tier.
    pub tasks: Vec<YoloTaskStatus>,
}

/// Status of a single task inside a tier, including its most recent result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTaskStatus {
    /// Agent that runs the task.
    pub agent: String,
    /// Skill the agent executes.
    pub skill: String,
    /// Whether the task is switched on for this tier.
    pub enabled: bool,
    /// Outcome of the most recent run; `None` when it has not run yet.
    pub last_result: Option<YoloTaskResult>,
}

/// Outcome of one task execution: success flag, timing and the token spend the
/// run accounted for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTaskResult {
    /// Whether the task finished successfully.
    pub success: bool,
    /// Wall-clock duration of the run, in milliseconds.
    pub duration_ms: u64,
    /// Completion timestamp as a string; the emitters in this family write
    /// ISO-8601 UTC (`2026-06-06T12:00:00Z` in this file's tests).
    pub completed_at: String,
    /// Failure message when `success` is false; `None` on success (and for an
    /// absent key, since the field is optional).
    pub error: Option<String>,
    /// `(input, output)` token counts for the run; `None` when the producer
    /// reports none.
    #[serde(default)]
    pub token_usage: Option<(u32, u32)>,
    /// Model that served the run (e.g. `gpt-4o#1`); `None` when unreported.
    #[serde(default)]
    pub model_name: Option<String>,
}

/// Whole YOLO configuration as one payload: one configuration block per tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloFullConfig {
    /// Configuration blocks for the tiers.
    pub tiers: Vec<YoloTierConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn all_returns_four_tiers() -> Result<()> {
        assert_eq!(YoloTaskTier::all().len(), 4);
        Ok(())
    }

    #[test]
    fn all_returns_correct_order() -> Result<()> {
        let all = YoloTaskTier::all();
        assert_eq!(all[0], YoloTaskTier::Realtime);
        assert_eq!(all[1], YoloTaskTier::Periodic);
        assert_eq!(all[2], YoloTaskTier::Daily);
        assert_eq!(all[3], YoloTaskTier::Strategic);
        Ok(())
    }

    #[test]
    fn name_from_name_roundtrip() -> Result<()> {
        for tier in YoloTaskTier::all() {
            assert_eq!(YoloTaskTier::from_name(tier.name()), Some(*tier));
        }
        Ok(())
    }

    #[test]
    fn from_name_case_sensitive() -> Result<()> {
        assert_eq!(YoloTaskTier::from_name("Periodic"), None);
        assert_eq!(YoloTaskTier::from_name("REALTIME"), None);
        assert_eq!(YoloTaskTier::from_name("Daily"), None);
        Ok(())
    }

    #[test]
    fn from_name_empty() -> Result<()> {
        assert_eq!(YoloTaskTier::from_name(""), None);
        Ok(())
    }

    #[test]
    fn display_matches_name() -> Result<()> {
        for tier in YoloTaskTier::all() {
            assert_eq!(format!("{}", tier), tier.name());
        }
        Ok(())
    }

    #[test]
    fn serde_tier_roundtrip() -> Result<()> {
        for tier in YoloTaskTier::all() {
            let json = serde_json::to_string(tier).context("test precondition")?;
            let back: YoloTaskTier = serde_json::from_str(&json).context("test precondition")?;
            assert_eq!(back, *tier);
        }
        Ok(())
    }

    #[test]
    fn serde_tier_snake_case() -> Result<()> {
        let json = serde_json::to_string(&YoloTaskTier::Periodic).context("test precondition")?;
        assert!(json.contains("periodic"));
        assert!(!json.contains("Periodic"));

        let json = serde_json::to_string(&YoloTaskTier::Realtime).context("test precondition")?;
        assert!(json.contains("realtime"));
        Ok(())
    }

    #[test]
    fn tier_task_config_default_enabled() -> Result<()> {
        let json = r#"{"agent":"philia","skill":"memory_consolidate"}"#;
        let cfg: YoloTierTaskConfig = serde_json::from_str(json).context("test precondition")?;
        assert_eq!(cfg.agent, "philia");
        assert_eq!(cfg.skill, "memory_consolidate");
        assert!(cfg.enabled);
        Ok(())
    }

    #[test]
    fn tier_task_config_explicit_disabled() -> Result<()> {
        let json = r#"{"agent":"philia","skill":"memory_consolidate","enabled":false}"#;
        let cfg: YoloTierTaskConfig = serde_json::from_str(json).context("test precondition")?;
        assert!(!cfg.enabled);
        Ok(())
    }

    #[test]
    fn tier_config_default_enabled_false() -> Result<()> {
        let json = r#"{"tier":"periodic","tasks":[]}"#;
        let cfg: YoloTierConfig = serde_json::from_str(json).context("test precondition")?;
        assert!(!cfg.enabled);
        assert_eq!(cfg.interval_secs, 0);
        Ok(())
    }

    #[test]
    fn with_defaults_periodic() -> Result<()> {
        let cfg = YoloTierConfig::with_defaults(YoloTaskTier::Periodic);
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_secs, 3600);
        assert_eq!(cfg.tier, YoloTaskTier::Periodic);
        assert!(cfg.tasks.is_empty());
        Ok(())
    }

    #[test]
    fn with_defaults_realtime() -> Result<()> {
        let cfg = YoloTierConfig::with_defaults(YoloTaskTier::Realtime);
        assert!(!cfg.enabled);
        assert_eq!(cfg.interval_secs, 120);
        Ok(())
    }

    #[test]
    fn with_defaults_daily() -> Result<()> {
        let cfg = YoloTierConfig::with_defaults(YoloTaskTier::Daily);
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_secs, 21600);
        Ok(())
    }

    #[test]
    fn with_defaults_strategic() -> Result<()> {
        let cfg = YoloTierConfig::with_defaults(YoloTaskTier::Strategic);
        assert!(!cfg.enabled);
        assert_eq!(cfg.interval_secs, 604800);
        Ok(())
    }

    #[test]
    fn tier_status_optional_fields_none() -> Result<()> {
        let status = YoloTierStatus {
            tier: YoloTaskTier::Periodic,
            enabled: true,
            interval_secs: 3600,
            last_run_at: None,
            next_run_at: None,
            tasks: vec![],
        };
        let json = serde_json::to_string(&status).context("test precondition")?;
        let back: YoloTierStatus = serde_json::from_str(&json).context("test precondition")?;
        assert!(back.last_run_at.is_none());
        assert!(back.next_run_at.is_none());
        Ok(())
    }

    #[test]
    fn task_result_serialization_with_error() -> Result<()> {
        let result = YoloTaskResult {
            success: false,
            duration_ms: 500,
            completed_at: "2026-06-06T12:00:00Z".to_string(),
            error: Some("timeout".to_string()),
            token_usage: None,
            model_name: None,
        };
        let json = serde_json::to_string(&result).context("test precondition")?;
        let back: YoloTaskResult = serde_json::from_str(&json).context("test precondition")?;
        assert!(!back.success);
        assert_eq!(back.error, Some("timeout".to_string()));
        Ok(())
    }

    #[test]
    fn task_result_serialization_with_token_usage() -> Result<()> {
        let result = YoloTaskResult {
            success: true,
            duration_ms: 250,
            completed_at: "2026-06-06T12:00:00Z".to_string(),
            error: None,
            token_usage: Some((1500, 800)),
            model_name: Some("gpt-4o#1".to_string()),
        };
        let json = serde_json::to_string(&result).context("test precondition")?;
        let back: YoloTaskResult = serde_json::from_str(&json).context("test precondition")?;
        assert!(back.success);
        assert_eq!(back.token_usage, Some((1500, 800)));
        assert_eq!(back.model_name.as_deref(), Some("gpt-4o#1"));
        Ok(())
    }

    #[test]
    fn task_result_deserialization_without_new_fields() -> Result<()> {
        let json = r#"{"success":true,"duration_ms":100,"completed_at":"2026-06-06T12:00:00Z","error":null}"#;
        let result: YoloTaskResult = serde_json::from_str(json).context("test precondition")?;
        assert!(result.success);
        assert!(result.token_usage.is_none());
        assert!(result.model_name.is_none());
        Ok(())
    }

    #[test]
    fn task_result_serialization_no_error() -> Result<()> {
        let result = YoloTaskResult {
            success: true,
            duration_ms: 100,
            completed_at: "2026-06-06T12:00:00Z".to_string(),
            error: None,
            token_usage: None,
            model_name: None,
        };
        let json = serde_json::to_string(&result).context("test precondition")?;
        let back: YoloTaskResult = serde_json::from_str(&json).context("test precondition")?;
        assert!(back.success);
        assert!(back.error.is_none());
        Ok(())
    }

    #[test]
    fn full_config_serialization() -> Result<()> {
        let config = YoloFullConfig {
            tiers: vec![YoloTierConfig {
                tier: YoloTaskTier::Periodic,
                enabled: true,
                interval_secs: 3600,
                tasks: vec![YoloTierTaskConfig {
                    agent: "philia".to_string(),
                    skill: "memory_consolidate".to_string(),
                    enabled: true,
                }],
            }],
        };
        let json = serde_json::to_string(&config).context("test precondition")?;
        let back: YoloFullConfig = serde_json::from_str(&json).context("test precondition")?;
        assert_eq!(back.tiers.len(), 1);
        assert_eq!(back.tiers[0].tasks.len(), 1);
        assert_eq!(back.tiers[0].tasks[0].agent, "philia");
        Ok(())
    }

    #[test]
    fn tier_equality_and_hash() -> Result<()> {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(YoloTaskTier::Periodic);
        set.insert(YoloTaskTier::Periodic);
        set.insert(YoloTaskTier::Daily);
        assert_eq!(set.len(), 2);
        Ok(())
    }

    #[test]
    fn fast_tiers_bootstrap_slow_tiers_defer() -> Result<()> {
        // Realtime/Periodic bootstrap on the first YOLO tick (establish a
        // baseline); Daily/Strategic defer to their interval so a fresh start
        // doesn't dump a wall of cold-start dispatches.
        assert!(YoloTaskTier::Realtime.should_bootstrap_on_first_run());
        assert!(YoloTaskTier::Periodic.should_bootstrap_on_first_run());
        assert!(!YoloTaskTier::Daily.should_bootstrap_on_first_run());
        assert!(!YoloTaskTier::Strategic.should_bootstrap_on_first_run());
        Ok(())
    }
}
