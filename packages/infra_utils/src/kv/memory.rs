use std::time::Duration;
use tokio::sync::broadcast;

use async_trait::async_trait;
use dashmap::DashMap;

use super::{
    KvError, KvStore, KvWatchEvent,
    event::{KvEntry, WATCH_CHANNEL_CAPACITY},
};

#[derive(Debug)]
pub struct InMemoryKvStore {
    data: DashMap<String, KvEntry>,
    watcher: broadcast::Sender<KvWatchEvent>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        let (watcher, _) = broadcast::channel(WATCH_CHANNEL_CAPACITY);
        Self {
            data: DashMap::new(),
            watcher,
        }
    }

    fn notify(&self, event: KvWatchEvent) {
        if self.watcher.receiver_count() > 0 {
            let _ = self.watcher.send(event);
        }
    }

    fn clean_expired(&self) {
        let expired: Vec<String> = self
            .data
            .iter()
            .filter(|e| e.value().is_expired())
            .map(|e| e.key().clone())
            .collect();
        for key in expired {
            self.data.remove(&key);
        }
    }
}

impl Default for InMemoryKvStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KvStore for InMemoryKvStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KvError> {
        match self.data.get(key) {
            Some(entry) if !entry.is_expired() => Ok(Some(entry.value.clone())),
            Some(_) => {
                self.data.remove(key);
                Ok(None)
            },
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> Result<(), KvError> {
        self.data
            .insert(key.to_string(), KvEntry::new(value.clone()));
        self.notify(KvWatchEvent {
            key: key.to_string(),
            kind: super::KvWatchEventKind::Set,
            value: Some(value),
        });
        Ok(())
    }

    async fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<(), KvError> {
        self.data
            .insert(key.to_string(), KvEntry::with_ttl(value.clone(), ttl));
        self.notify(KvWatchEvent {
            key: key.to_string(),
            kind: super::KvWatchEventKind::Set,
            value: Some(value),
        });
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), KvError> {
        self.data.remove(key);
        self.notify(KvWatchEvent {
            key: key.to_string(),
            kind: super::KvWatchEventKind::Delete,
            value: None,
        });
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, KvError> {
        match self.data.get(key) {
            Some(entry) if !entry.is_expired() => Ok(true),
            _ => Ok(false),
        }
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, KvError> {
        self.clean_expired();
        Ok(self
            .data
            .iter()
            .filter(|e| e.key().starts_with(prefix))
            .map(|e| (e.key().clone(), e.value().value.clone()))
            .collect())
    }

    async fn delete_prefix(&self, prefix: &str) -> Result<usize, KvError> {
        let keys: Vec<String> = self
            .data
            .iter()
            .filter(|e| e.key().starts_with(prefix))
            .map(|e| e.key().clone())
            .collect();
        let count = keys.len();
        for key in &keys {
            self.data.remove(key);
            self.notify(KvWatchEvent {
                key: key.clone(),
                kind: super::KvWatchEventKind::Delete,
                value: None,
            });
        }
        Ok(count)
    }

    fn watch(&self) -> broadcast::Receiver<KvWatchEvent> {
        self.watcher.subscribe()
    }
}
