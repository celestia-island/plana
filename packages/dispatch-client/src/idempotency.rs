//! Bounded idempotency window for `request_id` deduplication (spec §4.1).
//!
//! Every dispatched envelope carries a UUIDv4 idempotency key. The window
//! remembers recently seen keys so a retried dispatch is rejected *before*
//! any transport call — a duplicate must never reach the broker and must
//! never produce a second audit record (spec §4.3 pair invariant).
//!
//! The window is bounded twice: by capacity (oldest entry evicted first) and
//! by expiry (entries older than the TTL are forgotten, which also bounds
//! memory for idle clients).

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Default maximum number of remembered request ids.
pub const DEFAULT_WINDOW_CAPACITY: usize = 1024;

/// Default expiry for remembered request ids (~5 minutes, matching the
/// execution-time budget of the terminal route).
pub const DEFAULT_WINDOW_TTL: Duration = Duration::from_secs(300);

/// A bounded set of seen request ids.
///
/// [`IdempotencyWindow::check_insert`] returns `true` exactly once per key
/// inside the window: the first insertion succeeds, immediate repeats report
/// `false` ("already seen"). Keys are forgotten on capacity eviction
/// (oldest first) or on TTL expiry, after which the same key may be accepted
/// again — the window bounds memory, it does not promise global uniqueness.
#[derive(Debug)]
pub struct IdempotencyWindow {
    capacity: usize,
    ttl: Duration,
    seen: HashMap<String, Instant>,
    /// Insertion order ledger, `(request_id, inserted_at)`, oldest first.
    order: VecDeque<(String, Instant)>,
}

impl Default for IdempotencyWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl IdempotencyWindow {
    /// Creates a window with [`DEFAULT_WINDOW_CAPACITY`] and
    /// [`DEFAULT_WINDOW_TTL`].
    pub fn new() -> Self {
        Self::with_config(DEFAULT_WINDOW_CAPACITY, DEFAULT_WINDOW_TTL)
    }

    /// Creates a window with an explicit capacity and TTL. The capacity is
    /// clamped to at least one entry.
    pub fn with_config(capacity: usize, ttl: Duration) -> Self {
        Self {
            capacity: capacity.max(1),
            ttl,
            seen: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Registers `request_id` as seen.
    ///
    /// Returns `false` when the key is already inside the window (duplicate),
    /// `true` when this is its first sighting. Expired entries are swept
    /// first, so a key whose TTL elapsed is accepted as fresh again.
    pub fn check_insert(&mut self, request_id: &str) -> bool {
        let now = Instant::now();
        self.sweep_expired(now);
        if self.seen.contains_key(request_id) {
            return false;
        }
        while self.seen.len() >= self.capacity {
            match self.order.pop_front() {
                Some((evicted, _)) => {
                    self.seen.remove(&evicted);
                }
                None => break,
            }
        }
        self.seen.insert(request_id.to_string(), now);
        self.order.push_back((request_id.to_string(), now));
        true
    }

    /// Number of ids currently remembered.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether no ids are currently remembered.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    /// Drops entries older than the TTL. Insertion timestamps are monotonic
    /// along the queue, so sweeping from the front is complete.
    fn sweep_expired(&mut self, now: Instant) {
        while let Some((_, inserted_at)) = self.order.front() {
            if now.duration_since(*inserted_at) >= self.ttl {
                let (id, _) = self.order.pop_front().expect("front existed");
                self.seen.remove(&id);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_insert_true_repeat_false() {
        let mut window = IdempotencyWindow::new();
        assert!(window.check_insert("11111111-1111-4111-8111-111111111111"));
        assert!(!window.check_insert("11111111-1111-4111-8111-111111111111"));
        assert_eq!(window.len(), 1);
    }

    #[test]
    fn distinct_ids_coexist() {
        let mut window = IdempotencyWindow::new();
        assert!(window.check_insert("a"));
        assert!(window.check_insert("b"));
        assert!(!window.check_insert("a"));
        assert!(!window.check_insert("b"));
        assert_eq!(window.len(), 2);
    }

    #[test]
    fn capacity_evicts_oldest_first() {
        let mut window = IdempotencyWindow::with_config(3, Duration::from_secs(300));
        assert!(window.check_insert("a"));
        assert!(window.check_insert("b"));
        assert!(window.check_insert("c"));
        // Inserting "d" evicts "a", the oldest entry.
        assert!(window.check_insert("d"));
        assert_eq!(window.len(), 3);
        // Every survivor is still a duplicate (read-only in effect: a
        // duplicate hit never re-inserts, so it cannot evict anything).
        assert!(!window.check_insert("b"));
        assert!(!window.check_insert("c"));
        assert!(!window.check_insert("d"));
        assert_eq!(window.len(), 3);
        // The evicted oldest id is accepted again.
        assert!(
            window.check_insert("a"),
            "evicted id must be accepted again"
        );
    }

    #[test]
    fn ttl_expiry_frees_the_key() {
        let mut window = IdempotencyWindow::with_config(16, Duration::from_millis(50));
        assert!(window.check_insert("tick"));
        assert!(!window.check_insert("tick"));
        std::thread::sleep(Duration::from_millis(90));
        assert!(
            window.check_insert("tick"),
            "expired id must be accepted again"
        );
        assert_eq!(window.len(), 1);
    }

    #[test]
    fn capacity_is_clamped_to_at_least_one() {
        let mut window = IdempotencyWindow::with_config(0, Duration::from_secs(300));
        assert!(window.check_insert("only"));
        assert_eq!(window.len(), 1);
        assert!(!window.check_insert("only"));
    }

    #[test]
    fn default_window_uses_documented_bounds() {
        let window = IdempotencyWindow::new();
        assert_eq!(DEFAULT_WINDOW_CAPACITY, 1024);
        assert_eq!(DEFAULT_WINDOW_TTL, Duration::from_secs(300));
        assert!(window.is_empty());
    }
}
