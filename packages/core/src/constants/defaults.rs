pub const DEFAULT_NETWORK: &str = "entelecheia-network";

pub struct RuntimeTuningConfig {
    pub retry_backoff_secs: Vec<u64>,
    pub retry_interval_ms: u64,
    pub retry_max_count: u32,

    pub timeout_thresholds_secs: Vec<u64>,

    pub cache_default_size: usize,
    pub cache_max_size: usize,
    pub cache_ttl_secs: u64,

    pub task_queue_poll_intervals_ms: Vec<u64>,

    pub batch_default_size: usize,
    pub batch_wait_ms: u64,
    pub batch_processor_wait_ms: u64,

    pub dialogue_timeout_ms: u64,
    pub max_dialogue_history_len: usize,
    pub max_dialogue_records: usize,
    pub max_priority: u32,

    pub max_performance_reports: usize,

    pub llm_max_tokens: u32,
    pub llm_safety_filter_sample_size: usize,

    pub skill_max_retries: usize,

    pub negotiation_round_timeout_secs: u64,
    pub negotiation_total_timeout_secs: u64,
    pub max_message_length: usize, // 0 = unlimited (outer transport handles size limits)

    pub recovery_threshold_secs: u64,
    pub max_context_tokens: usize,

    pub skill_chain_temperature: f32,

    /// Whether the skill-chain loop pins `tool_choice=required` on LLM calls
    /// that carry tool definitions. Reasoning ("thinking") model backends —
    /// e.g. DeepSeek V4 thinking mode — reject a forced tool_choice with
    /// HTTP 400 (`Thinking mode does not support this tool_choice`), so
    /// deployments routing skill chains through such models must set
    /// `SKILL_CHAIN_FORCE_TOOL_CHOICE=false` to let the model pick tools
    /// freely. Defaults to `true` (the historical behavior).
    pub skill_chain_force_tool_choice: bool,
}

fn parse_u64_list(s: &str) -> Vec<u64> {
    s.split(',').filter_map(|v| v.trim().parse().ok()).collect()
}

impl RuntimeTuningConfig {
    pub fn from_env() -> Self {
        let retry_backoff_secs = std::env::var("RETRY_BACKOFF_SECS")
            .ok()
            .map(|v| parse_u64_list(&v))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![5, 10, 30, 60, 120]);

        let task_queue_poll_intervals_ms = std::env::var("TASK_QUEUE_POLL_INTERVALS_MS")
            .ok()
            .map(|v| parse_u64_list(&v))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![100, 200, 500]);

        let timeout_thresholds_secs = std::env::var("TIMEOUT_THRESHOLDS_SECS")
            .ok()
            .map(|v| parse_u64_list(&v))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![300, 600, 7200]);

        Self {
            retry_backoff_secs,
            retry_interval_ms: std::env::var("RETRY_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            retry_max_count: std::env::var("RETRY_MAX_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),

            timeout_thresholds_secs,

            cache_default_size: std::env::var("CACHE_DEFAULT_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            cache_max_size: std::env::var("CACHE_MAX_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10000),
            cache_ttl_secs: std::env::var("CACHE_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),

            task_queue_poll_intervals_ms,

            batch_default_size: std::env::var("BATCH_DEFAULT_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            batch_wait_ms: std::env::var("BATCH_WAIT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            batch_processor_wait_ms: std::env::var("BATCH_PROCESSOR_WAIT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),

            dialogue_timeout_ms: std::env::var("DIALOGUE_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300_000),
            max_dialogue_history_len: std::env::var("MAX_DIALOGUE_HISTORY_LEN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_dialogue_records: std::env::var("MAX_DIALOGUE_RECORDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            max_priority: std::env::var("MAX_PRIORITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),

            max_performance_reports: std::env::var("MAX_PERFORMANCE_REPORTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),

            llm_max_tokens: std::env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16384),
            llm_safety_filter_sample_size: std::env::var("LLM_SAFETY_FILTER_SAMPLE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10000),

            skill_max_retries: std::env::var("SKILL_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),

            negotiation_round_timeout_secs: std::env::var("NEGOTIATION_ROUND_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            negotiation_total_timeout_secs: std::env::var("NEGOTIATION_TOTAL_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            max_message_length: std::env::var("MAX_MESSAGE_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            recovery_threshold_secs: std::env::var("RECOVERY_THRESHOLD_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            max_context_tokens: std::env::var("MAX_CONTEXT_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8000),

            skill_chain_temperature: std::env::var("SKILL_CHAIN_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),

            skill_chain_force_tool_choice: std::env::var("SKILL_CHAIN_FORCE_TOOL_CHOICE")
                .ok()
                .map(|v| !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"))
                .unwrap_or(true),
        }
    }
}

pub struct StorageLifecycleConfig {
    pub conversation_ttl_days: u64,
    pub archived_conversation_retention_days: u64,
    pub orphan_conversation_ttl_days: u64,
    pub child_session_retention_days: u64,
    pub archived_message_retention_days: u64,
    pub max_dialogue_history_len: usize,
    pub max_dialogue_records: usize,
    pub dialogue_timeout_ms: u64,
    pub cleanup_interval_minutes: u64,
    pub retention_log_days: u64,
    pub chat_log_enabled: bool,
    pub chat_log_retention_days: u64,
    pub dialogue_event_retention_days: u64,
    pub agent_stale_days: u64,
    pub rbac_audit_retention_days: u64,
    pub model_usage_retention_days: u64,
    pub container_snapshot_retention_days: u64,
}

impl StorageLifecycleConfig {
    pub fn from_env() -> Self {
        Self {
            conversation_ttl_days: std::env::var("CONVERSATION_TTL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90),
            archived_conversation_retention_days: std::env::var(
                "ARCHIVED_CONVERSATION_RETENTION_DAYS",
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7),
            orphan_conversation_ttl_days: std::env::var("ORPHAN_CONVERSATION_TTL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            child_session_retention_days: std::env::var("CHILD_SESSION_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            archived_message_retention_days: std::env::var("ARCHIVED_MESSAGE_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            max_dialogue_history_len: std::env::var("MAX_DIALOGUE_HISTORY_LEN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_dialogue_records: std::env::var("MAX_DIALOGUE_RECORDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            dialogue_timeout_ms: std::env::var("DIALOGUE_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300_000),
            cleanup_interval_minutes: std::env::var("CLEANUP_INTERVAL_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            retention_log_days: std::env::var("RETENTION_LOG_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            chat_log_enabled: std::env::var("CHAT_LOG_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            chat_log_retention_days: std::env::var("CHAT_LOG_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            dialogue_event_retention_days: std::env::var("DIALOGUE_EVENT_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            agent_stale_days: std::env::var("AGENT_STALE_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            rbac_audit_retention_days: std::env::var("RBAC_AUDIT_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90),
            model_usage_retention_days: std::env::var("MODEL_USAGE_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90),
            container_snapshot_retention_days: std::env::var("CONTAINER_SNAPSHOT_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if self.conversation_ttl_days == 0 {
            errors.push("CONVERSATION_TTL_DAYS must be > 0".to_string());
        }
        if self.archived_conversation_retention_days == 0 {
            errors.push("ARCHIVED_CONVERSATION_RETENTION_DAYS must be > 0".to_string());
        }
        if self.cleanup_interval_minutes == 0 {
            errors.push("CLEANUP_INTERVAL_MINUTES must be > 0".to_string());
        }
        if self.max_dialogue_history_len == 0 {
            errors.push("MAX_DIALOGUE_HISTORY_LEN must be > 0".to_string());
        }
        if self.dialogue_timeout_ms == 0 {
            errors.push("DIALOGUE_TIMEOUT_MS must be > 0".to_string());
        }
        if self.retention_log_days == 0 {
            errors.push("RETENTION_LOG_DAYS must be > 0".to_string());
        }
        if self.dialogue_event_retention_days == 0 {
            errors.push("DIALOGUE_EVENT_RETENTION_DAYS must be > 0".to_string());
        }
        if self.agent_stale_days == 0 {
            errors.push("AGENT_STALE_DAYS must be > 0".to_string());
        }
        if self.rbac_audit_retention_days == 0 {
            errors.push("RBAC_AUDIT_RETENTION_DAYS must be > 0".to_string());
        }
        if self.model_usage_retention_days == 0 {
            errors.push("MODEL_USAGE_RETENTION_DAYS must be > 0".to_string());
        }
        if self.container_snapshot_retention_days == 0 {
            errors.push("CONTAINER_SNAPSHOT_RETENTION_DAYS must be > 0".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}
