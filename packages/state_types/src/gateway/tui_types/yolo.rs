use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YoloTaskTier {
    Realtime,
    Periodic,
    Daily,
    Strategic,
}

impl YoloTaskTier {
    pub fn all() -> &'static [YoloTaskTier] {
        &[
            YoloTaskTier::Realtime,
            YoloTaskTier::Periodic,
            YoloTaskTier::Daily,
            YoloTaskTier::Strategic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            YoloTaskTier::Realtime => "realtime",
            YoloTaskTier::Periodic => "periodic",
            YoloTaskTier::Daily => "daily",
            YoloTaskTier::Strategic => "strategic",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "realtime" => Some(YoloTaskTier::Realtime),
            "periodic" => Some(YoloTaskTier::Periodic),
            "daily" => Some(YoloTaskTier::Daily),
            "strategic" => Some(YoloTaskTier::Strategic),
            _ => None,
        }
    }

    pub fn default_interval_secs(&self) -> u64 {
        match self {
            YoloTaskTier::Realtime => 120,
            YoloTaskTier::Periodic => 3600,
            YoloTaskTier::Daily => 21600,
            YoloTaskTier::Strategic => 604800,
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierTaskConfig {
    pub agent: String,
    pub skill: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierConfig {
    pub tier: YoloTaskTier,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub interval_secs: u64,
    pub tasks: Vec<YoloTierTaskConfig>,
}

impl YoloTierConfig {
    pub fn with_defaults(tier: YoloTaskTier) -> Self {
        Self {
            enabled: tier.default_enabled(),
            interval_secs: tier.default_interval_secs(),
            tier,
            tasks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTierStatus {
    pub tier: YoloTaskTier,
    pub enabled: bool,
    pub interval_secs: u64,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub tasks: Vec<YoloTaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTaskStatus {
    pub agent: String,
    pub skill: String,
    pub enabled: bool,
    pub last_result: Option<YoloTaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloTaskResult {
    pub success: bool,
    pub duration_ms: u64,
    pub completed_at: String,
    pub error: Option<String>,
    #[serde(default)]
    pub token_usage: Option<(u32, u32)>,
    #[serde(default)]
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloFullConfig {
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
