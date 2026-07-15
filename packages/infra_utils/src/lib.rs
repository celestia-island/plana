//! Infrastructure resilience primitives: patterns for fault tolerance, concurrency
//! governance, and in-process messaging.
//!
//! Unlike the other `infra_*` crates, this one is purely a *toolbox* — it has no domain
//! knowledge of agents, containers, or storage. It provides reusable building blocks
//! consumed by higher-level crates.
//!
//! Key modules:
//! - **`circuit_breaker`** — a generic, nameable [`CircuitBreaker`] that transitions
//!   through Closed → Open → HalfOpen states; used to wrap LLM API calls and other
//!   fallible external services.
//! - **`pubsub`** — an in-process event bus ([`InProcessBus`]) built on Tokio broadcast
//!   channels, with strongly-typed topic enums; decouples producers (e.g. an agent
//!   producing streaming chunks) from subscribers (TUI, audit logger, WebSocket).
//! - **`thread_audit`** — thin wrappers around Tokio/`std::thread` that enforce named
//!   threads and emit tracing spans, making production thread dumps actionable.
//! - **`panic_guard`** / **`async_bridge`** — small utilities for safe panic recovery
//!   and bridging sync ↔ async boundaries.
//!
//! Design principle: these are *policy-free* mechanisms. The circuit breaker doesn't
//! decide which calls should be protected — the caller does. The pubsub bus doesn't
//! know what a "message" means — it just routes typed enums.
#![allow(clippy::type_complexity)]

pub mod async_bridge;
pub mod circuit_breaker;
pub mod device_id;
pub mod kv;
pub mod panic_guard;
pub mod pubsub;
pub mod soc;
pub mod thread_audit;

pub use kv::{
    InMemoryKvStore, InProcessTransport, KvError, KvStore, KvWatchEvent, SharedKvStore,
    SyncTransport,
};
pub use pubsub::{InProcessBus, PubSubBus, PubSubEvent};
