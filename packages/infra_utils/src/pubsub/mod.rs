pub mod topics;

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct PubSubEvent {
    pub topic: String,
    pub payload: Value,
}

#[async_trait]
pub trait PubSubBus: Send + Sync {
    async fn publish(&self, topic: &str, payload: Value);
    async fn subscribe(&self, pattern: &str) -> broadcast::Receiver<PubSubEvent>;
    fn subscriber_count(&self, topic: &str) -> usize;
}

pub struct InProcessBus {
    channels: Arc<DashMap<String, broadcast::Sender<PubSubEvent>>>,
    capacity: usize,
}

impl Default for InProcessBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl InProcessBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            capacity,
        }
    }

    fn get_or_create_sender(&self, topic: &str) -> broadcast::Sender<PubSubEvent> {
        if let Some(entry) = self.channels.get(topic) {
            return entry.value().clone();
        }
        let (tx, _rx) = broadcast::channel(self.capacity);
        self.channels
            .entry(topic.to_string())
            .or_insert_with(|| tx)
            .value()
            .clone()
    }

    fn match_pattern(pattern: &str, topic: &str) -> bool {
        if pattern == topic {
            return true;
        }
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let topic_parts: Vec<&str> = topic.split('.').collect();
        if pattern_parts.len() != topic_parts.len() {
            return false;
        }
        for (p, t) in pattern_parts.iter().zip(topic_parts.iter()) {
            if *p == "*" {
                continue;
            }
            if p != t {
                return false;
            }
        }
        true
    }

    pub fn topic_count(&self) -> usize {
        self.channels.len()
    }
}

#[async_trait]
impl PubSubBus for InProcessBus {
    async fn publish(&self, topic: &str, payload: Value) {
        let event = PubSubEvent {
            topic: topic.to_string(),
            payload,
        };

        for entry in self.channels.iter() {
            let registered_topic = entry.key();
            if Self::match_pattern(registered_topic, topic)
                && entry.value().send(event.clone()).is_err()
            {
                warn!(
                    topic = %topic,
                    registered = %registered_topic,
                    "PubSub: no active receivers for topic"
                );
            }
        }
    }

    async fn subscribe(&self, pattern: &str) -> broadcast::Receiver<PubSubEvent> {
        self.get_or_create_sender(pattern).subscribe()
    }

    fn subscriber_count(&self, topic: &str) -> usize {
        self.channels
            .get(topic)
            .map(|tx| tx.receiver_count())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn test_publish_subscribe_exact_topic() -> Result<()> {
        let bus = InProcessBus::new(16);
        let mut rx = bus.subscribe("agent.hubris.update").await;

        bus.publish(
            "agent.hubris.update",
            serde_json::json!({"status": "working"}),
        )
        .await;

        let event = timeout(Duration::from_millis(100), rx.recv()).await??;
        assert_eq!(event.topic, "agent.hubris.update");
        assert_eq!(event.payload["status"], "working");
        Ok(())
    }

    #[tokio::test]
    async fn test_wildcard_pattern() -> Result<()> {
        let bus = InProcessBus::new(16);
        let mut rx = bus.subscribe("agent.*.update").await;

        bus.publish("agent.hubris.update", serde_json::json!({"a": 1}))
            .await;

        let event = timeout(Duration::from_millis(100), rx.recv()).await??;
        assert_eq!(event.topic, "agent.hubris.update");
        Ok(())
    }

    #[tokio::test]
    async fn test_wildcard_no_match() -> Result<()> {
        let bus = InProcessBus::new(16);
        let mut rx = bus.subscribe("agent.*.update").await;

        bus.publish("task.123.status", serde_json::json!({"s": "done"}))
            .await;

        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err(), "should not receive non-matching event");
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_subscribers() -> Result<()> {
        let bus = InProcessBus::new(16);
        let mut rx1 = bus.subscribe("test.topic").await;
        let mut rx2 = bus.subscribe("test.topic").await;

        bus.publish("test.topic", serde_json::json!({"v": 42}))
            .await;

        let e1 = timeout(Duration::from_millis(100), rx1.recv()).await;
        let e2 = timeout(Duration::from_millis(100), rx2.recv()).await;
        assert!(e1.is_ok());
        assert!(e2.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_subscriber_count() -> Result<()> {
        let bus = InProcessBus::new(16);
        let _rx1 = bus.subscribe("counter.topic").await;
        let _rx2 = bus.subscribe("counter.topic").await;
        assert_eq!(bus.subscriber_count("counter.topic"), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_topic_count() -> Result<()> {
        let bus = InProcessBus::new(16);
        let _ = bus.subscribe("a.b").await;
        let _ = bus.subscribe("c.d").await;
        assert_eq!(bus.topic_count(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_no_duplicate_delivery() -> Result<()> {
        let bus = InProcessBus::new(16);
        let mut rx = bus.subscribe("exact.topic").await;

        bus.publish("exact.topic", serde_json::json!({"x": 1}))
            .await;

        let first = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(first.is_ok());

        let second = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(
            second.is_err(),
            "should NOT receive duplicate event for exact topic"
        );
        Ok(())
    }
}
