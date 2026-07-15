// Redis backend stub — all mutating operations return an error.
// To implement: add `redis` crate dependency and replace each method body
// with async Redis commands. The broadcast watcher channel is already wired
// for pub/sub-like notifications.

use std::time::Duration;
use tokio::sync::broadcast;

use async_trait::async_trait;

use super::{KvError, KvStore, KvWatchEvent};

#[derive(Debug)]
pub struct RedisKvStore {
    _url: String,
    watcher: broadcast::Sender<KvWatchEvent>,
}

impl RedisKvStore {
    pub fn new(url: &str) -> Self {
        let (watcher, _) = broadcast::channel(super::event::WATCH_CHANNEL_CAPACITY);
        Self {
            _url: url.to_string(),
            watcher,
        }
    }
}

#[async_trait]
impl KvStore for RedisKvStore {
    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn set(&self, _key: &str, _value: Vec<u8>) -> Result<(), KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn set_with_ttl(
        &self,
        _key: &str,
        _value: Vec<u8>,
        _ttl: Duration,
    ) -> Result<(), KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<(), KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn exists(&self, _key: &str) -> Result<bool, KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn list_prefix(&self, _prefix: &str) -> Result<Vec<(String, Vec<u8>)>, KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    async fn delete_prefix(&self, _prefix: &str) -> Result<usize, KvError> {
        Err(KvError::Backend(
            "Redis backend not yet implemented".to_string(),
        ))
    }

    fn watch(&self) -> broadcast::Receiver<KvWatchEvent> {
        self.watcher.subscribe()
    }
}
