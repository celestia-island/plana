mod event;
mod memory;
mod redis_stub;
mod transport;

use std::time::Duration;
use tokio::sync::broadcast;

use async_trait::async_trait;
pub use event::KvEntry;
pub use memory::InMemoryKvStore;
pub use redis_stub::RedisKvStore;
pub use transport::{InProcessTransport, SyncTransport, TransportMessage};

pub type SharedKvStore = std::sync::Arc<dyn KvStore>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvWatchEventKind {
    Set,
    Delete,
}

#[derive(Debug, Clone)]
pub struct KvWatchEvent {
    pub key: String,
    pub kind: KvWatchEventKind,
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum KvError {
    #[error("key not found: {0}")]
    NotFound(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("backend error: {0}")]
    Backend(String),
    #[error("ttl expired: {0}")]
    TtlExpired(String),
}

#[async_trait]
pub trait KvStore: Send + Sync + std::fmt::Debug {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<(), KvError>;
    async fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<(), KvError>;
    async fn delete(&self, key: &str) -> Result<(), KvError>;
    async fn exists(&self, key: &str) -> Result<bool, KvError>;
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, KvError>;
    async fn delete_prefix(&self, prefix: &str) -> Result<usize, KvError>;
    fn watch(&self) -> broadcast::Receiver<KvWatchEvent>;
}

pub async fn kv_get_json<T: serde::de::DeserializeOwned>(
    store: &dyn KvStore,
    key: &str,
) -> Result<Option<T>, KvError> {
    match store.get(key).await? {
        Some(bytes) => {
            let val = serde_json::from_slice(&bytes)
                .map_err(|e| KvError::Serialization(e.to_string()))?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

pub async fn kv_set_json<T: serde::Serialize + Sync>(
    store: &dyn KvStore,
    key: &str,
    value: &T,
) -> Result<(), KvError> {
    let bytes = serde_json::to_vec(value).map_err(|e| KvError::Serialization(e.to_string()))?;
    store.set(key, bytes).await
}
