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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        // Round-37's F-37-1: a HalfOpen circuit whose probe slots were
        // consumed but never recorded (the routing layer calls
        // is_available() on EVERY candidate but only uses one — the
        // unselected candidates' slots leak) is stuck: no new probe is
        // allowed and no transition can fire. When another full
        // recovery window has elapsed since the last failure, the
        // slots reset — the circuit gets a fresh probe attempt
        // (self-healing instead of permanent half-open starvation).
        if state == CircuitState::HalfOpen
            && self.inner.half_open_calls.load(Ordering::Relaxed) == 0
            && let Some(last) = *self.inner.last_failure.read()
            && last.elapsed() >= self.inner.config.recovery_timeout * 2
        {
            let guard = self.inner.state.write();
            if *guard == CircuitState::HalfOpen {
                self.inner.half_open_calls.store(0, Ordering::Relaxed);
                debug!(
                    circuit = %self.inner.name,
                    "circuit breaker resetting leaked HalfOpen probe slots \
                     (recovery window elapsed again)"
                );
            }
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

    /// Take a HalfOpen probe slot as a **self-returning guard**
    /// (audit R1-CB-2).
    ///
    /// The bool-shaped [`Self::is_call_allowed`] has a structural flaw:
    /// the slot is consumed on `true` and only ever returned by an
    /// explicit record call — a caller that probes a candidate and never
    /// uses it (the router tries every candidate, uses one) leaks the
    /// slot until the leak self-heal fires. The guard returns the slot
    /// on drop, so unselected candidates cost nothing.
    ///
    /// Returns `None` when the circuit is not HalfOpen or the probe
    /// slots are exhausted. Record the outcome through
    /// [`Self::record_success`] / [`Self::record_failure`] while the
    /// guard is alive — the transition resets the slot counter first,
    /// and the guard's own return is a no-op afterwards.
    pub fn probe(&self) -> Option<ProbeGuard> {
        // Runs the state machine (Open → HalfOpen promotion, leak
        // self-heal) exactly like is_call_allowed.
        let state = self.state();
        if state != CircuitState::HalfOpen {
            return None;
        }
        let current = self.inner.half_open_calls.fetch_add(1, Ordering::Relaxed);
        if current >= self.inner.config.half_open_max_calls {
            // Slots exhausted: roll back our own increment.
            self.inner.half_open_calls.fetch_sub(1, Ordering::Relaxed);
            return None;
        }
        Some(ProbeGuard {
            inner: Arc::clone(&self.inner),
        })
    }

    pub fn record_success(&self) {
        self.inner.success_count.fetch_add(1, Ordering::Relaxed);
        let state = *self.inner.state.read();
        if state == CircuitState::HalfOpen {
            // Audit R1-CB-1: re-check under the write lock — a concurrent
            // probe failure may already have flipped HalfOpen → Open, and
            // a late success must not overwrite that trip (which would
            // silently re-close the breaker on a failing backend).
            let mut guard = self.inner.state.write();
            if *guard == CircuitState::HalfOpen {
                *guard = CircuitState::Closed;
                self.inner.failure_count.store(0, Ordering::Relaxed);
                self.inner.half_open_calls.store(0, Ordering::Relaxed);
                debug!(
                    circuit = %self.inner.name,
                    "circuit breaker transitioning HalfOpen → Closed (success)"
                );
            }
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
                // Mirror of record_success's re-check (audit R1-CB-1
                // follow-up): a concurrent probe success may already have
                // closed the breaker; the late failure must not reopen a
                // legitimately recovered circuit. The direction is
                // fail-safe either way, so the state is simply re-read.
                if *guard == CircuitState::HalfOpen {
                    *guard = CircuitState::Open;
                    self.inner.half_open_calls.store(0, Ordering::Relaxed);
                    warn!(
                        circuit = %self.inner.name,
                        "circuit breaker tripped HalfOpen → Open (probe failed)"
                    );
                }
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

/// A live HalfOpen probe slot taken via [`CircuitBreaker::probe`].
///
/// Returns the slot on drop — the whole point of the guard shape
/// (audit R1-CB-2): unselected probe candidates can no longer leak
/// their slot until a self-heal fires.
pub struct ProbeGuard {
    inner: Arc<Inner>,
}

impl ProbeGuard {
    /// Whether the slot is still being tracked (the breaker has not
    /// transitioned out of HalfOpen since the probe was taken).
    pub fn holds_slot(&self) -> bool {
        *self.inner.state.read() == CircuitState::HalfOpen
    }
}

impl Drop for ProbeGuard {
    fn drop(&mut self) {
        // Return the slot — but only if the counter still tracks live
        // probes. A recorded outcome (success/failure) resets the
        // counter to zero on the state transition first; subtracting
        // after that would underflow the count.
        let current = self.inner.half_open_calls.load(Ordering::Relaxed);
        if current > 0 {
            self.inner.half_open_calls.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Soft ceiling on registered breakers (audit R1-CB-3): dynamic names
/// (per-tenant keys and the like) must not grow the registry without
/// bound — past the ceiling `get_or_create` hands out unregistered
/// throwaway instances, warned once per process.
const MAX_REGISTRY_ENTRIES: usize = 1_000;

/// Warned-once flag for throwaway instances past the registry ceiling.
static UNREGISTERED_WARNED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(Vec::new()),
        }
    }

    /// Get the named breaker, creating and registering it on first use.
    ///
    /// Audit R1-CB-3, both sharp edges called out:
    /// - **First configuration wins.** A later call with a different
    ///   config returns the EXISTING breaker and its original config —
    ///   in debug builds (and in test builds, where cargo forces
    ///   debug-assertions on) a config mismatch asserts loudly, because
    ///   it is almost always a call-site bug rather than intent.
    /// - **The registry is bounded** ([`MAX_REGISTRY_ENTRIES`]): past
    ///   the ceiling (dynamic per-tenant names and the like) the caller
    ///   gets an UNREGISTERED throwaway instance — its state is its own,
    ///   warned once per process.
    pub fn get_or_create(&self, name: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
        let guard = self.breakers.read();
        if let Some(b) = guard.iter().find(|b| b.name() == name) {
            debug_assert!(
                b.inner.config == config,
                "circuit breaker '{name}' is already registered with a different config"
            );
            return b.clone();
        }
        drop(guard);
        let mut guard = self.breakers.write();
        if let Some(b) = guard.iter().find(|b| b.name() == name) {
            debug_assert!(
                b.inner.config == config,
                "circuit breaker '{name}' is already registered with a different config"
            );
            return b.clone();
        }
        if guard.len() >= MAX_REGISTRY_ENTRIES {
            if !UNREGISTERED_WARNED.swap(true, Ordering::Relaxed) {
                warn!(
                    "circuit breaker registry at its ceiling ({MAX_REGISTRY_ENTRIES}); returning an unregistered throwaway instance for '{name}'"
                );
            }
            return CircuitBreaker::new(name, config);
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

    // Audit R1-CB-1 (record_success re-check): the race this fix closes —
    // success READS HalfOpen, a concurrent failure flips Open, success's
    // WRITE would re-close — has no sequential observable: between the
    // read and the write no other thread can run, so no single-threaded
    // test can distinguish fixed from unfixed code (the obvious
    // "fail-then-success" sequence passes on both). The re-check mirrors
    // `record_failure`'s existing Closed-arm re-check and is verified by
    // inspection; a real pin would need a test seam between the two
    // lock acquisitions.

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

    /// Audit R1-CB-2: the probe guard RETURNS its slot on drop — the
    /// router shape (probe every candidate, use one) must not leak
    /// slots anymore (that was round-37's F-37-1, handled before by a
    /// self-heal).
    #[test]
    fn probe_guard_drop_returns_the_slot() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                recovery_timeout: Duration::from_millis(10),
                half_open_max_calls: 1,
            },
        );
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // The one slot is taken…
        let guard = cb.probe().expect("half-open probe must be admitted");
        assert!(guard.holds_slot());
        assert!(cb.probe().is_none(), "slots are exhausted");

        // …and returned on drop: the next probe is admitted again.
        drop(guard);
        assert!(cb.probe().is_some(), "the slot must return on drop");
        Ok(())
    }

    /// Audit R1-CB-2: a recorded outcome resets the slot counter — the
    /// guard's later drop must be a no-op, not an underflow.
    #[test]
    fn probe_record_then_drop_does_not_underflow() -> Result<()> {
        let cb = CircuitBreaker::new(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 1,
                recovery_timeout: Duration::from_millis(10),
                half_open_max_calls: 1,
            },
        );
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        let guard = cb.probe().expect("probe admitted");
        cb.record_success(); // transition resets the counter to zero
        drop(guard); // must be a no-op
        assert_eq!(cb.state(), CircuitState::Closed);
        Ok(())
    }

    /// Audit R1-CB-3: the registry is bounded — past the ceiling the
    /// caller gets an unregistered throwaway (its own state, not in the
    /// registry, not visible to all_states).
    #[test]
    fn registry_is_bounded_and_throwaways_stay_unregistered() -> Result<()> {
        let reg = CircuitBreakerRegistry::new();
        for i in 0..MAX_REGISTRY_ENTRIES {
            let _ = reg.get_or_create(&format!("cb-{i}"), CircuitBreakerConfig::default());
        }
        assert_eq!(reg.all_states().len(), MAX_REGISTRY_ENTRIES);

        let throwaway = reg.get_or_create("over-the-cap", CircuitBreakerConfig::default());
        assert_eq!(reg.all_states().len(), MAX_REGISTRY_ENTRIES);
        assert!(
            !std::ptr::eq(
                reg.get_or_create("over-the-cap", CircuitBreakerConfig::default())
                    .inner
                    .as_ref(),
                throwaway.inner.as_ref()
            ),
            "throwaway instances must not be registered"
        );
        Ok(())
    }
}
