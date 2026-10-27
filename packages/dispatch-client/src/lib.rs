//! Shared evernight broker dispatch client for the PoleMos terminal command
//! routing contract (P83/M1).
//!
//! Specification of record:
//! `entelecheia/docs/zh-Hans/designs/polemos-evernight-terminal-routing.md`
//! — §4 (routing contract: request envelope, read-back bounds),
//! §4.2 (error taxonomy), §4.3 (OreXis audit digest formula), §5.2 (module
//! boundary). This crate is the M2 adapter foundation that entelecheia's
//! PoleMos agent consumes.
//!
//! Module map (mirrors spec §5.2, with `auth`/`client` as the adapter glue):
//!
//! - [`envelope`] — §4.1 request envelope + wire-params encoding (only
//!   `command` / `cwd`? / `timeout` reach the broker) and the UUIDv4
//!   idempotency key.
//! - [`idempotency`] — bounded seen-request-id window; duplicates are
//!   rejected before any transport call so no audit pair is duplicated.
//! - [`transport`] — [`transport::Transport`] trait plus the newline-framed
//!   Unix-socket IPC implementation (one connect→write→read→close exchange
//!   per call, F2 semantics); the remote wss form is a documented skeleton.
//! - [`classify`] — §4.2 error taxonomy: transport faults normalize to
//!   `peer_unreachable` (`-32603`), broker errors map by structural markers
//!   (`data.timeout`) and stable codes (`-32005`/`-32602`/`-32601`).
//! - [`guard`] — single-node target-scope guard (`-32005` +
//!   `target_out_of_scope` on violation) and the [`guard::Endpoint`] config
//!   enum.
//! - [`client`] — [`client::EvernightClient`]: guard → idempotency →
//!   `Command.Exec` → classify → parse → 1 MiB truncation → §4.3 digest.
//! - [`auth`] — method-name vocabulary (`Command.Exec`, `System.Ping`) and
//!   the request-scoped `auth` params field.
//!
//! Boundaries the crate deliberately keeps (spec §5.2): it knows nothing
//! about OreXis (audit emission wraps the client one layer up), and it never
//! invents wire fields beyond what evernight's `Command.Exec` accepts.
//!
//! Token discipline: the static shared token arrives via the environment
//! (`EVERNIGHT_TOKEN`), lives in memory only, and appears in repo text only
//! as the placeholder `<your-evernight-token>`.

pub mod auth;
pub mod classify;
pub mod client;
pub mod envelope;
pub mod guard;
pub mod idempotency;
pub mod transport;

pub use auth::{
    token_from_env, AUTH_PARAM_KEY, ENV_TOKEN_VAR, METHOD_COMMAND_EXEC, METHOD_SYSTEM_PING,
};
pub use classify::{classify_rpc, classify_transport, ClassifiedError, ErrorKind};
pub use client::{EvernightClient, EvernightClientConfig, ExecOutput, MAX_OUTPUT_BYTES};
pub use envelope::{DispatchEnvelope, TargetScope, MAX_EXEC_TIMEOUT_SECS};
pub use guard::{Endpoint, TargetScopeGuard};
pub use idempotency::{IdempotencyWindow, DEFAULT_WINDOW_CAPACITY, DEFAULT_WINDOW_TTL};
#[cfg(feature = "ws")]
pub use transport::WsTransport;
pub use transport::{IpcSocketTransport, Transport, TransportError};
