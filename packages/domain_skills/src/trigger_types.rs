use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriggerTopic(String);

impl TriggerTopic {
    pub fn new(topic: &str) -> Self {
        Self(topic.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, pattern: &TriggerPattern) -> bool {
        if pattern.is_wildcard() {
            return true;
        }
        let topic_parts: Vec<&str> = self.0.split('.').collect();
        let pattern_parts: Vec<&str> = pattern.as_str().split('.').collect();
        if topic_parts.len() < pattern_parts.len() {
            return false;
        }
        for (tp, pp) in topic_parts.iter().zip(pattern_parts.iter()) {
            if *pp == "*" {
                continue;
            }
            if tp != pp {
                return false;
            }
        }
        true
    }

    pub fn parent(&self) -> Option<TriggerTopic> {
        self.0
            .rsplit_once('.')
            .map(|(parent, _)| TriggerTopic::new(parent))
    }

    pub fn parts(&self) -> Vec<&str> {
        self.0.split('.').collect()
    }
}

impl fmt::Display for TriggerTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TriggerTopic {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriggerPattern(String);

impl TriggerPattern {
    pub fn new(pattern: &str) -> Self {
        Self(pattern.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_wildcard(&self) -> bool {
        self.0 == "*" || self.0 == "#"
    }
}

impl fmt::Display for TriggerPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub use arona_prompt::prompt_loader::TriggerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub topic: TriggerTopic,
    pub source: String,
    pub payload: serde_json::Value,
    pub timestamp: SystemTime,
    pub headers: Option<serde_json::Value>,
}

impl TriggerEvent {
    pub fn new(source: &str, topic: &str, payload: serde_json::Value) -> Self {
        Self {
            topic: TriggerTopic::new(topic),
            source: source.to_string(),
            payload,
            timestamp: SystemTime::now(),
            headers: None,
        }
    }

    pub fn with_headers(mut self, headers: serde_json::Value) -> Self {
        self.headers = Some(headers);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSubscription {
    pub skill_name: String,
    pub agent_type: String,
    pub topic_pattern: TriggerPattern,
}

impl TriggerSubscription {
    pub fn matches(&self, topic: &TriggerTopic) -> bool {
        topic.matches(&self.topic_pattern)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHookConfig {
    pub path: String,
    pub topic_prefix: String,
    #[serde(default = "default_http_method")]
    pub method: String,
    #[serde(default)]
    pub secret_env: Option<String>,
}

fn default_http_method() -> String {
    "POST".to_string()
}

#[derive(Debug, Clone)]
pub struct RegisteredHttpHook {
    pub config: HttpHookConfig,
    pub subscribers: Vec<TriggerSubscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMcpToolSchema {
    pub tool_name: String,
    pub trigger_source: String,
    pub topic_prefix: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn trigger_topic_exact_match() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("github.issues.opened");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_no_match() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("github.pull_request.opened");
        assert!(!topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_wildcard_leaf() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("github.issues.*");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_wildcard_middle() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("github.*.opened");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_wildcard_global() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("*");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_hash_wildcard() -> Result<()> {
        let topic = TriggerTopic::new("discord.message");
        let pattern = TriggerPattern::new("#");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_prefix_match() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let pattern = TriggerPattern::new("github.issues");
        assert!(topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_too_short_for_pattern() -> Result<()> {
        let topic = TriggerTopic::new("github");
        let pattern = TriggerPattern::new("github.issues.*");
        assert!(!topic.matches(&pattern));
        Ok(())
    }

    #[test]
    fn trigger_topic_parent() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        assert_eq!(topic.parent(), Some(TriggerTopic::new("github.issues")));
        let root = TriggerTopic::new("github");
        assert_eq!(root.parent(), None);
        Ok(())
    }

    #[test]
    fn trigger_topic_parts() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        assert_eq!(topic.parts(), vec!["github", "issues", "opened"]);
        Ok(())
    }

    #[test]
    fn trigger_subscription_matches() -> Result<()> {
        let sub = TriggerSubscription {
            skill_name: "handle-issue".to_string(),
            agent_type: "hubris".to_string(),
            topic_pattern: TriggerPattern::new("github.issues.*"),
        };
        assert!(sub.matches(&TriggerTopic::new("github.issues.opened")));
        assert!(sub.matches(&TriggerTopic::new("github.issues.closed")));
        assert!(!sub.matches(&TriggerTopic::new("github.pull_request.opened")));
        Ok(())
    }

    #[test]
    fn trigger_event_new() -> Result<()> {
        let event = TriggerEvent::new(
            "http-hook",
            "github.push",
            serde_json::json!({"ref": "refs/heads/main"}),
        );
        assert_eq!(event.source, "http-hook");
        assert_eq!(event.topic.as_str(), "github.push");
        assert!(event.headers.is_none());
        Ok(())
    }

    #[test]
    fn trigger_event_with_headers() -> Result<()> {
        let event = TriggerEvent::new(
            "http-hook",
            "test.topic",
            serde_json::Value::Object(Default::default()),
        )
        .with_headers(serde_json::json!({"x-signature": "abc123"}));
        assert!(event.headers.is_some());
        let headers = event.headers.context("test precondition")?;
        assert_eq!(headers["x-signature"], "abc123");
        Ok(())
    }

    #[test]
    fn trigger_pattern_is_wildcard() -> Result<()> {
        assert!(TriggerPattern::new("*").is_wildcard());
        assert!(TriggerPattern::new("#").is_wildcard());
        assert!(!TriggerPattern::new("github.*").is_wildcard());
        Ok(())
    }

    #[test]
    fn trigger_topic_display() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        assert_eq!(format!("{}", topic), "github.issues.opened");
        Ok(())
    }

    #[test]
    fn trigger_topic_from_str() -> Result<()> {
        let topic: TriggerTopic = "github.push".into();
        assert_eq!(topic.as_str(), "github.push");
        Ok(())
    }

    #[test]
    fn trigger_topic_serialize_deserialize() -> Result<()> {
        let topic = TriggerTopic::new("github.issues.opened");
        let json = serde_json::to_string(&topic).context("test precondition")?;
        let deserialized: TriggerTopic =
            serde_json::from_str(&json).context("test precondition")?;
        assert_eq!(topic, deserialized);
        Ok(())
    }
}
