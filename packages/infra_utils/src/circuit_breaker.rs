use parking_lot::RwLock;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use tracing::{debug, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_calls: 1,
        }
    }
}

struct Inner {
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    half_open_calls: AtomicU64,
    last_failure: RwLock<Option<Instant>>,
    config: CircuitBreakerConfig,
    name: String,
}

#[derive(Clone)]
pub struct CircuitBreaker {
    inner: Arc<Inner>,
}

impl CircuitBreaker {
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: RwLock::new(CircuitState::Closed),
                failure_count: AtomicU64::new(0),
                success_count: AtomicU64::new(0),
                half_open_calls: AtomicU64::new(0),
                last_failure: RwLock::new(None),
                config,
                name: name.to_string(),
            }),
        }
    }

    pub fn state(&self) -> CircuitState {
        let state = *self.inner.state.read();
        if state == CircuitState::Open
            && let Some(last) = *self.inner.last_failure.read()
            && last.elapsed() >= self.inner.config.recovery_timeout
        {
            let mut guard = self.inner.state.write();
            if *guard == CircuitState::Open {
                *guard = CircuitState::HalfOpen;
                self.inner.half_open_calls.store(0, Ordering::Relaxed);
                debug!(
                    circuit = %self.inner.name,
                    "circuit breaker transitioning Open → HalfOpen"
                );
            }
            return CircuitState::HalfOpen;
        }
        state
    }

    pub fn is_call_allowed(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => {
                let current = self.inner.half_open_calls.fetch_add(1, Ordering::Relaxed);
                current < self.inner.config.half_open_max_calls
            }
            CircuitState::Open => false,
        }
    }

    pub fn record_success(&self) {
        self.inner.success_count.fetch_add(1, Ordering::Relaxed);
        let state = *self.inner.state.read();
        if state == CircuitState::HalfOpen {
            let mut guard = self.inner.state.write();
            *guard = CircuitState::Closed;
            self.inner.failure_count.store(0, Ordering::Relaxed);
            self.inner.half_open_calls.store(0, Ordering::Relaxed);
            debug!(
                circuit = %self.inner.name,
                "circuit breaker transitioning HalfOpen → Closed (success)"
            );
        } else if state == CircuitState::Closed {
            self.inner.failure_count.store(0, Ordering::Relaxed);
        }
    }

    pub fn record_failure(&self) {
        let prev = self.inner.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.inner.last_failure.write() = Some(Instant::now());

        let state = *self.inner.state.read();
        match state {
            CircuitState::Closed => {
                if prev + 1 >= self.inner.config.failure_threshold {
                    let mut guard = self.inner.state.write();
                    if *guard == CircuitState::Closed {
                        *guard = CircuitState::Open;
                        warn!(
                            circuit = %self.inner.name,
                            failures = prev + 1,
                            threshold = self.inner.config.failure_threshold,
                            "circuit breaker tripped Closed → Open"
                        );
                    }
                }
            }
            CircuitState::HalfOpen => {
                let mut guard = self.inner.state.write();
                *guard = CircuitState::Open;
                self.inner.half_open_calls.store(0, Ordering::Relaxed);
                warn!(
                    circuit = %self.inner.name,
                    "circuit breaker tripped HalfOpen → Open (probe failed)"
                );
            }
            CircuitState::Open => {}
        }
    }

    pub fn failure_count(&self) -> u64 {
        self.inner.failure_count.load(Ordering::Relaxed)
    }

    pub fn success_count(&self) -> u64 {
        self.inner.success_count.load(Ordering::Relaxed)
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }
}

pub struct CircuitBreakerRegistry {
    breakers: RwLock<Vec<CircuitBreaker>>,
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(Vec::new()),
        }
    }

    pub fn get_or_create(&self, name: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
        let guard = self.breakers.read();
        if let Some(b) = guard.iter().find(|b| b.name() == name) {
            return b.clone();
        }
        drop(guard);
        let mut guard = self.breakers.write();
        if let Some(b) = guard.iter().find(|b| b.name() == name) {
            return b.clone();
        }
        let cb = CircuitBreaker::new(name, config);
        guard.push(cb.clone());
        cb
    }

    pub fn all_states(&self) -> Vec<(String, CircuitState, u64, u64)> {
        self.breakers
            .read()
            .iter()
            .map(|b| {
                (
                    b.name().to_string(),
                    b.state(),
                    b.failure_count(),
                    b.success_count(),
                )
            })
            .collect()
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_starts_closed() -> Result<()> {
        let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_call_allowed());
        Ok(())
    }

    #[test]
    fn test_trips_on_threshold() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 3,
                ..Default::default()
            },
        );
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_call_allowed());
        Ok(())
    }

    #[test]
    fn test_half_open_after_recovery_timeout() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                recovery_timeout: Duration::from_millis(10),
                ..Default::default()
            },
        );
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        Ok(())
    }

    #[test]
    fn test_success_resets_closed() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                recovery_timeout: Duration::from_millis(10),
                ..Default::default()
            },
        );
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        Ok(())
    }

    #[test]
    fn test_half_open_failure_reopens() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                recovery_timeout: Duration::from_millis(10),
                ..Default::default()
            },
        );
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        Ok(())
    }

    #[test]
    fn test_registry_get_or_create() -> Result<()> {
        let reg = CircuitBreakerRegistry::new();
        let cb1 = reg.get_or_create("bridge", CircuitBreakerConfig::default());
        let cb2 = reg.get_or_create("bridge", CircuitBreakerConfig::default());
        assert!(std::ptr::eq(cb1.inner.as_ref(), cb2.inner.as_ref()));
        let cb3 = reg.get_or_create("other", CircuitBreakerConfig::default());
        assert!(!std::ptr::eq(cb1.inner.as_ref(), cb3.inner.as_ref()));
        Ok(())
    }

    #[test]
    fn test_success_resets_failure_count_in_closed() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 5,
                ..Default::default()
            },
        );
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 3);
        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
        Ok(())
    }
}
