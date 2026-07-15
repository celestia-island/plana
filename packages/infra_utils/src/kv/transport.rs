use anyhow::Result;
use tokio::sync::broadcast;

use async_trait::async_trait;
use dashmap::DashMap;

const TRANSPORT_CHANNEL_CAPACITY: usize = 4096;

#[derive(Debug, Clone)]
pub struct TransportMessage {
    pub channel: String,
    pub payload: Vec<u8>,
}

#[async_trait]
pub trait SyncTransport: Send + Sync + std::fmt::Debug {
    async fn publish(&self, channel: &str, payload: &[u8]) -> Result<()>;
    fn subscribe(&self, channel: &str) -> broadcast::Receiver<TransportMessage>;
    fn subscriber_count(&self, channel: &str) -> usize;
}

#[derive(Debug)]
pub struct InProcessTransport {
    channels: DashMap<String, broadcast::Sender<TransportMessage>>,
}

impl InProcessTransport {
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
        }
    }

    fn get_or_create_sender(&self, channel: &str) -> broadcast::Sender<TransportMessage> {
        self.channels
            .entry(channel.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(TRANSPORT_CHANNEL_CAPACITY);
                tx
            })
            .value()
            .clone()
    }
}

impl Default for InProcessTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyncTransport for InProcessTransport {
    async fn publish(&self, channel: &str, payload: &[u8]) -> Result<()> {
        let sender = self.get_or_create_sender(channel);
        let msg = TransportMessage {
            channel: channel.to_string(),
            payload: payload.to_vec(),
        };
        let _ = sender.send(msg);
        Ok(())
    }

    fn subscribe(&self, channel: &str) -> broadcast::Receiver<TransportMessage> {
        self.get_or_create_sender(channel).subscribe()
    }

    fn subscriber_count(&self, channel: &str) -> usize {
        self.channels
            .get(channel)
            .map(|s| s.receiver_count())
            .unwrap_or(0)
    }
}
