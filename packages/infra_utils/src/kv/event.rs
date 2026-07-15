#[derive(Debug, Clone)]
pub struct KvEntry {
    pub value: Vec<u8>,
    pub expires_at: Option<std::time::Instant>,
}

impl KvEntry {
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            value,
            expires_at: None,
        }
    }

    pub fn with_ttl(value: Vec<u8>, ttl: std::time::Duration) -> Self {
        Self {
            value,
            expires_at: Some(std::time::Instant::now() + ttl),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|t| std::time::Instant::now() > t)
            .unwrap_or(false)
    }
}

pub const WATCH_CHANNEL_CAPACITY: usize = 1024;
